//! Consumer liveness and error state belong only to the current outgoing row.
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversationInputQueueState {
    Absent,
    Pending,
    Error,
}
#[derive(diesel::QueryableByName)]
pub(super) struct QueueRow {
    #[diesel(sql_type=Text)]
    pub(super) queue_state: String,
    #[diesel(sql_type=BigInt)]
    pub(super) consumer_expires_at: i64,
    #[diesel(sql_type=diesel::sql_types::Bool)]
    pub(super) readiness_waiting: bool,
    #[diesel(sql_type=Nullable<IdentitySql>)]
    pub(super) consumer_claim_id: Option<EntityId>,
}
impl QueueRow {
    pub(super) fn state(&self) -> Result<ConversationInputQueueState, StoreError> {
        match self.queue_state.as_str() {
            "PENDING" => Ok(ConversationInputQueueState::Pending),
            "ERROR" => Ok(ConversationInputQueueState::Error),
            _ => Err(StoreError::Integrity),
        }
    }
}
fn queue_ids(conversation: &str, input: &str) -> Result<(EntityId, EntityId), StoreError> {
    Ok((
        EntityId::from_dto(EntityKind::Conversation, conversation)?,
        EntityId::from_dto(EntityKind::ConversationOperation, input)?,
    ))
}
macro_rules! require_consumer {
    ($connection:expr,$dialect:expr,$input:expr,$claim:expr) => {{
        let claim = EntityId::from_dto(EntityKind::ConversationOperation, $claim)?;
        let query = match $dialect {
            Dialect::Postgres => "SELECT queue_state,consumer_expires_at,readiness_waiting,consumer_claim_id FROM conversation_inputs WHERE id=? FOR UPDATE",
            Dialect::Sqlite => "SELECT queue_state,consumer_expires_at,readiness_waiting,consumer_claim_id FROM conversation_inputs WHERE id=?",
        };
        let row = diesel::sql_query($dialect.query(query))
            .bind::<IdentitySql, _>($input)
            .get_result::<super::input_queue::QueueRow>($connection).await?;
        let now = crate::directory::database_now!($connection, $dialect);
        if row.state()? != super::input_queue::ConversationInputQueueState::Pending
            || row.consumer_expires_at <= now
            || row.readiness_waiting
            || row.consumer_claim_id != Some(claim)
        {
            return Err(StoreError::InvalidTransition);
        }
        #[derive(diesel::QueryableByName)]
        struct SessionCount { #[diesel(sql_type=BigInt)] n: i64 }
        let session = diesel::sql_query($dialect.query("SELECT CAST(count(*) AS BIGINT) AS n FROM conversation_inputs i JOIN auth_sessions s ON s.id=i.submitting_session_id WHERE i.id=? AND s.revoked_at IS NULL AND s.expires_at>?"))
            .bind::<IdentitySql,_>($input).bind::<BigInt,_>(now).get_result::<SessionCount>($connection).await?;
        if session.n != 1 { return Err(StoreError::Unauthenticated); }
    }};
}
pub(super) use require_consumer;
impl Store {
    /// Internal accepted-consumer observation, not public reader authority.
    pub async fn conversation_input_queue_state(
        &self,
        conversation: &str,
        input: &str,
    ) -> Result<ConversationInputQueueState, StoreError> {
        let (conversation, input) = queue_ids(conversation, input)?;
        operation!(self, |connection, dialect| {
            let row = diesel::sql_query(dialect.query(
                "SELECT queue_state,consumer_expires_at,readiness_waiting,consumer_claim_id FROM conversation_inputs WHERE conversation_id=? AND id=?",
            ))
            .bind::<IdentitySql, _>(conversation)
            .bind::<IdentitySql, _>(input)
            .get_result::<QueueRow>(connection).await.optional()?;
            row.map_or(Ok(ConversationInputQueueState::Absent), |row| row.state())
        })
    }
    /// An attempt error is not a claim that native execution rolled back.
    /// Exact old IDs cannot modify a replacement or recreate a consumed row.
    pub async fn fail_conversation_input(
        &self,
        conversation: &str,
        input: &str,
    ) -> Result<(), StoreError> {
        let (conversation, input) = queue_ids(conversation, input)?;
        operation!(self, |connection, dialect| {
            diesel::sql_query(dialect.query(
                "UPDATE conversation_inputs SET queue_state='ERROR' WHERE conversation_id=? AND id=? AND queue_state='PENDING'",
            ))
            .bind::<IdentitySql, _>(conversation)
            .bind::<IdentitySql, _>(input)
            .execute(connection).await?;
            Ok(())
        })
    }
    /// Existing recovery never renews. Lock before reading DB time so a lock
    /// wait cannot revive a lease that expired while the caller was waiting.
    pub async fn renew_conversation_input_consumer(
        &self,
        conversation: &str,
        input: &str,
        claim: &str,
        policy: crate::DirectoryPolicy,
    ) -> Result<ConversationInputQueueState, StoreError> {
        let (conversation, input) = queue_ids(conversation, input)?;
        let claim = EntityId::from_dto(EntityKind::ConversationOperation, claim)?;
        operation!(self, |connection, dialect| {
            let query = match dialect {
                Dialect::Postgres => {
                    "SELECT queue_state,consumer_expires_at,readiness_waiting,consumer_claim_id FROM conversation_inputs WHERE conversation_id=? AND id=? FOR UPDATE"
                }
                Dialect::Sqlite => {
                    "SELECT queue_state,consumer_expires_at,readiness_waiting,consumer_claim_id FROM conversation_inputs WHERE conversation_id=? AND id=?"
                }
            };
            let row = diesel::sql_query(dialect.query(query))
                .bind::<IdentitySql, _>(conversation)
                .bind::<IdentitySql, _>(input)
                .get_result::<QueueRow>(connection)
                .await
                .optional()?;
            let Some(row) = row else {
                return Ok(ConversationInputQueueState::Absent);
            };
            if row.consumer_claim_id != Some(claim) || row.readiness_waiting {
                return Ok(ConversationInputQueueState::Absent);
            }
            if row.state()? != ConversationInputQueueState::Pending {
                return row.state();
            }
            let now = crate::directory::database_now!(connection, dialect);
            if row.consumer_expires_at <= now {
                return Ok(ConversationInputQueueState::Absent);
            }
            let expires = policy.expiry(now, true)?;
            diesel::sql_query(
                dialect.query("UPDATE conversation_inputs SET consumer_expires_at=? WHERE id=?"),
            )
            .bind::<BigInt, _>(expires)
            .bind::<IdentitySql, _>(input)
            .execute(connection)
            .await?;
            Ok(ConversationInputQueueState::Pending)
        })
    }
    /// Independent of browser requests; expiry never replays an input, cancels
    /// a native task or creates deleted-item history.
    pub async fn expire_conversation_input_consumers(&self) -> Result<(), StoreError> {
        operation!(self, |connection, dialect| {
            let now = crate::directory::database_now!(connection, dialect);
            #[derive(diesel::QueryableByName)]
            struct Expired {
                #[diesel(sql_type=IdentitySql)]
                id: EntityId,
            }
            let query = match dialect {
                Dialect::Postgres => {
                    "SELECT id FROM conversation_inputs WHERE queue_state='PENDING' AND readiness_waiting=FALSE AND consumer_expires_at<=? ORDER BY consumer_expires_at,id LIMIT 32 FOR UPDATE SKIP LOCKED"
                }
                Dialect::Sqlite => {
                    "SELECT id FROM conversation_inputs WHERE queue_state='PENDING' AND readiness_waiting=FALSE AND consumer_expires_at<=? ORDER BY consumer_expires_at,id LIMIT 32"
                }
            };
            let expired = diesel::sql_query(dialect.query(query))
                .bind::<BigInt, _>(now)
                .load::<Expired>(connection)
                .await?;
            // Re-read facts in a new statement after ownership locks; a native
            // grant committed during a lock wait must never look pre-attempt.
            for row in expired {
                let now = crate::directory::database_now!(connection, dialect);
                diesel::sql_query(dialect.query(
                "UPDATE conversation_inputs SET readiness_waiting=CASE WHEN submitting_session_id IS NOT NULL AND NOT EXISTS(SELECT 1 FROM conversation_input_preparation p WHERE p.operation_id=conversation_inputs.id) AND NOT EXISTS(SELECT 1 FROM conversation_input_delivery d WHERE d.operation_id=conversation_inputs.id) THEN TRUE ELSE FALSE END,queue_state=CASE WHEN submitting_session_id IS NULL OR EXISTS(SELECT 1 FROM conversation_input_preparation p WHERE p.operation_id=conversation_inputs.id) OR EXISTS(SELECT 1 FROM conversation_input_delivery d WHERE d.operation_id=conversation_inputs.id) THEN 'ERROR' ELSE 'PENDING' END,consumer_claim_id=NULL,consumer_expires_at=0 WHERE id=? AND queue_state='PENDING' AND readiness_waiting=FALSE AND consumer_expires_at<=?",
            )).bind::<IdentitySql,_>(row.id).bind::<BigInt, _>(now).execute(connection).await?;
            }
            Ok(())
        })
    }
}
