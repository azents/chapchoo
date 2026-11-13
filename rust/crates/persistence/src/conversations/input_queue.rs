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
    ($connection:expr,$dialect:expr,$input:expr) => {{
        let query = match $dialect {
            Dialect::Postgres => "SELECT queue_state,consumer_expires_at FROM conversation_inputs WHERE id=? FOR UPDATE",
            Dialect::Sqlite => "SELECT queue_state,consumer_expires_at FROM conversation_inputs WHERE id=?",
        };
        let row = diesel::sql_query($dialect.query(query))
            .bind::<IdentitySql, _>($input)
            .get_result::<super::input_queue::QueueRow>($connection).await?;
        let now = crate::directory::database_now!($connection, $dialect);
        if row.state()? != super::input_queue::ConversationInputQueueState::Pending
            || row.consumer_expires_at <= now
        {
            return Err(StoreError::InvalidTransition);
        }
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
                "SELECT queue_state,consumer_expires_at FROM conversation_inputs WHERE conversation_id=? AND id=?",
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
        policy: crate::DirectoryPolicy,
    ) -> Result<ConversationInputQueueState, StoreError> {
        let (conversation, input) = queue_ids(conversation, input)?;
        operation!(self, |connection, dialect| {
            let query = match dialect {
                Dialect::Postgres => {
                    "SELECT queue_state,consumer_expires_at FROM conversation_inputs WHERE conversation_id=? AND id=? FOR UPDATE"
                }
                Dialect::Sqlite => {
                    "SELECT queue_state,consumer_expires_at FROM conversation_inputs WHERE conversation_id=? AND id=?"
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
            if row.state()? != ConversationInputQueueState::Pending {
                return row.state();
            }
            let now = crate::directory::database_now!(connection, dialect);
            if row.consumer_expires_at <= now {
                diesel::sql_query(
                    dialect.query("UPDATE conversation_inputs SET queue_state='ERROR' WHERE id=?"),
                )
                .bind::<IdentitySql, _>(input)
                .execute(connection)
                .await?;
                return Ok(ConversationInputQueueState::Error);
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
            diesel::sql_query(dialect.query(
                "UPDATE conversation_inputs SET queue_state='ERROR' WHERE queue_state='PENDING' AND consumer_expires_at<=?",
            )).bind::<BigInt, _>(now).execute(connection).await?;
            Ok(())
        })
    }
}
