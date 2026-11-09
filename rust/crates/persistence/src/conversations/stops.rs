//! Exact work Stop admission is separate from Computer power and native retirement.
use super::*;
#[derive(Clone, PartialEq, Eq)]
pub struct ConversationWorkTarget {
    pub source: ConversationDeliveryTarget,
    pub work_id: String,
}
pub struct AcceptConversationStop<'a> {
    pub agent_id: &'a str,
    pub conversation_id: &'a str,
    pub request_id: &'a str,
    pub target: &'a ConversationWorkTarget,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversationStopState {
    PossibleForwarding,
    CancelPublished,
    Rejected,
}
#[derive(Clone)]
pub struct ConversationStopReceipt {
    pub id: String,
    pub conversation_id: String,
    pub request_id: String,
    pub execution_id: String,
    pub target: ConversationWorkTarget,
    pub state: ConversationStopState,
}
pub enum ConversationStopAdmission {
    Accepted(ConversationStopReceipt),
    Existing(ConversationStopReceipt),
}
macro_rules! safe_debug {($($name:ty),+)=>{$(impl fmt::Debug for $name {fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result {f.write_str(concat!(stringify!($name),"([REDACTED])"))}})+};}
safe_debug!(
    ConversationWorkTarget,
    AcceptConversationStop<'_>,
    ConversationStopReceipt,
    ConversationStopAdmission
);
#[derive(diesel::QueryableByName)]
struct StopRow {
    #[diesel(sql_type=IdentitySql)]
    id: EntityId,
    #[diesel(sql_type=IdentitySql)]
    conversation_id: EntityId,
    #[diesel(sql_type=IdentitySql)]
    execution_id: EntityId,
    #[diesel(sql_type=Text)]
    request_id: String,
    #[diesel(sql_type=Text)]
    source_snapshot: String,
    #[diesel(sql_type=Text)]
    native_session_id: String,
    #[diesel(sql_type=Text)]
    work_id: String,
    #[diesel(sql_type=Text)]
    stop_state: String,
}
impl StopRow {
    fn receipt(&self) -> Result<ConversationStopReceipt, StoreError> {
        Ok(ConversationStopReceipt {
            id: self.id.to_dto(EntityKind::ConversationOperation),
            conversation_id: self.conversation_id.to_dto(EntityKind::Conversation),
            request_id: self.request_id.clone(),
            execution_id: self.execution_id.to_dto(EntityKind::Execution),
            target: ConversationWorkTarget {
                source: ConversationDeliveryTarget {
                    snapshot: SnapshotId::parse(self.source_snapshot.clone())
                        .map_err(|_| StoreError::Integrity)?,
                    native_session: NativeConversationId::parse(self.native_session_id.clone())
                        .map_err(|_| StoreError::Integrity)?,
                },
                work_id: self.work_id.clone(),
            },
            state: match self.stop_state.as_str() {
                "POSSIBLE" => ConversationStopState::PossibleForwarding,
                "CANCEL_PUBLISHED" => ConversationStopState::CancelPublished,
                "REJECTED" => ConversationStopState::Rejected,
                _ => return Err(StoreError::Integrity),
            },
        })
    }
}
macro_rules! stop {
    ($connection:expr,$dialect:expr,$conversation:expr,$operation:expr) => {{
        diesel::sql_query(
            $dialect.query("SELECT * FROM conversation_stops WHERE conversation_id=? AND id=?"),
        )
        .bind::<IdentitySql, _>($conversation)
        .bind::<IdentitySql, _>($operation)
        .get_result::<StopRow>($connection)
        .await
        .optional()?
        .ok_or(StoreError::NotFound)?
    }};
}
impl Store {
    pub async fn accept_conversation_stop(
        &self,
        actor: &SecretDigest,
        draft: &AcceptConversationStop<'_>,
        now: i64,
    ) -> Result<ConversationStopAdmission, StoreError> {
        request_id(draft.request_id)?;
        request_id(&draft.target.work_id)?;
        let agent = EntityId::from_dto(EntityKind::Agent, draft.agent_id)?;
        let id = EntityId::from_dto(EntityKind::Conversation, draft.conversation_id)?;
        operation!(self, |connection, dialect| {
            let owner = owned_agent!(connection, dialect, actor, agent, now);
            let row = conversation!(connection, dialect, id);
            super::delivery::current(&row, agent, &draft.target.source)?;
            let previous = diesel::sql_query(dialect.query(
                "SELECT * FROM conversation_stops WHERE conversation_id=? AND request_id=?",
            ))
            .bind::<IdentitySql, _>(id)
            .bind::<Text, _>(draft.request_id)
            .get_result::<StopRow>(connection)
            .await
            .optional()?;
            if let Some(previous) = previous {
                let previous = previous.receipt()?;
                if &previous.target != draft.target {
                    return Err(StoreError::Conflict);
                }
                return Ok(ConversationStopAdmission::Existing(previous));
            }
            let observed = super::native_state::native_state!(connection, dialect, id)
                .ok_or(StoreError::InvalidTransition)?;
            let native = observed.observation()?;
            if native.target != draft.target.source
                || native.work_id.as_deref() != Some(&draft.target.work_id)
            {
                return Err(StoreError::InvalidTransition);
            }
            #[derive(diesel::QueryableByName)]
            struct Execution {
                #[diesel(sql_type=Nullable<IdentitySql>)]
                current_execution_id: Option<EntityId>,
            }
            let execution=diesel::sql_query(dialect.query("SELECT current_execution_id FROM computers WHERE id=? AND desired_state<>'DELETING'"))
                .bind::<IdentitySql,_>(owner.computer_id).get_result::<Execution>(connection).await.optional()?.ok_or(StoreError::InvalidTransition)?;
            if execution.current_execution_id != Some(observed.execution_id) {
                return Err(StoreError::InvalidTransition);
            }
            let operation = EntityId::generate(now)?;
            diesel::sql_query(dialect.query("INSERT INTO conversation_stops(id,conversation_id,execution_id,request_id,source_snapshot,native_session_id,work_id,stop_state) VALUES(?,?,?,?,?,?,?,'POSSIBLE')"))
                .bind::<IdentitySql,_>(operation).bind::<IdentitySql,_>(id).bind::<IdentitySql,_>(observed.execution_id).bind::<Text,_>(draft.request_id)
                .bind::<Text,_>(draft.target.source.snapshot.as_str()).bind::<Text,_>(draft.target.source.native_session.as_str()).bind::<Text,_>(&draft.target.work_id).execute(connection).await?;
            Ok(ConversationStopAdmission::Accepted(
                stop!(connection, dialect, id, operation).receipt()?,
            ))
        })
    }
    pub async fn check_conversation_stop(
        &self,
        source: &crate::ConnectorSource<'_>,
        conversation: &str,
        operation: &str,
        target: &ConversationWorkTarget,
    ) -> Result<(), StoreError> {
        let id = EntityId::from_dto(EntityKind::Conversation, conversation)?;
        let operation = EntityId::from_dto(EntityKind::ConversationOperation, operation)?;
        operation!(self, |connection, dialect| {
            let row = super::records::source_conversation!(connection, dialect, source, id);
            let route = connector!(connection, dialect, source);
            super::delivery::current(&row, row.agent_id, &target.source)?;
            let previous = stop!(connection, dialect, id, operation);
            let receipt = previous.receipt()?;
            if previous.execution_id != route._execution_id
                || &receipt.target != target
                || receipt.state != ConversationStopState::PossibleForwarding
            {
                return Err(StoreError::Conflict);
            }
            Ok(())
        })
    }
    pub async fn complete_conversation_stop(
        &self,
        source: &crate::ConnectorSource<'_>,
        conversation: &str,
        operation: &str,
        target: &ConversationWorkTarget,
        published: bool,
    ) -> Result<(), StoreError> {
        let id = EntityId::from_dto(EntityKind::Conversation, conversation)?;
        let operation = EntityId::from_dto(EntityKind::ConversationOperation, operation)?;
        operation!(self, |connection, dialect| {
            let row = super::records::source_conversation!(connection, dialect, source, id);
            let route = connector!(connection, dialect, source);
            super::delivery::current(&row, row.agent_id, &target.source)?;
            let previous = stop!(connection, dialect, id, operation);
            let receipt = previous.receipt()?;
            if previous.execution_id != route._execution_id || &receipt.target != target {
                return Err(StoreError::Conflict);
            }
            let state = if published {
                ConversationStopState::CancelPublished
            } else {
                ConversationStopState::Rejected
            };
            if receipt.state == state {
                return Ok(());
            }
            if receipt.state != ConversationStopState::PossibleForwarding {
                return Err(StoreError::Conflict);
            }
            diesel::sql_query(
                dialect.query("UPDATE conversation_stops SET stop_state=? WHERE id=?"),
            )
            .bind::<Text, _>(if published {
                "CANCEL_PUBLISHED"
            } else {
                "REJECTED"
            })
            .bind::<IdentitySql, _>(operation)
            .execute(connection)
            .await?;
            Ok(())
        })
    }
    pub async fn conversation_stop(
        &self,
        actor: &SecretDigest,
        agent: &str,
        conversation: &str,
        request: &str,
        now: i64,
    ) -> Result<Option<ConversationStopReceipt>, StoreError> {
        request_id(request)?;
        let agent = EntityId::from_dto(EntityKind::Agent, agent)?;
        let id = EntityId::from_dto(EntityKind::Conversation, conversation)?;
        operation!(self, |connection, dialect| {
            let _ = owned_agent!(connection, dialect, actor, agent, now);
            let row = conversation!(connection, dialect, id);
            if row.agent_id != agent {
                return Err(StoreError::NotFound);
            }
            diesel::sql_query(
                dialect.query(
                    "SELECT * FROM conversation_stops WHERE conversation_id=? AND request_id=?",
                ),
            )
            .bind::<IdentitySql, _>(id)
            .bind::<Text, _>(request)
            .get_result::<StopRow>(connection)
            .await
            .optional()?
            .map(|row| row.receipt())
            .transpose()
        })
    }
}
