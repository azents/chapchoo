//! Qualified internal input forwarding; no browser input or Chat receipt.

use super::*;
use crate::conversations::goals::{current_goals, forwarding_projection};

#[derive(diesel::QueryableByName)]
struct DeliveryRow {
    #[diesel(sql_type=Text)]
    delivery_state: String,
    #[diesel(sql_type=diesel::sql_types::Bool)]
    current_launch: bool,
}

macro_rules! require_continuation_order {
    ($connection:expr,$dialect:expr,$conversation:expr,$execution:expr,$snapshot:expr,$native:expr) => {{
        #[derive(diesel::QueryableByName)]
        struct EarlierContinuation {
            #[diesel(sql_type=diesel::sql_types::Bool)]
            earlier: bool,
        }
        let pending = diesel::sql_query($dialect.query(
            "SELECT EXISTS(SELECT 1 FROM conversation_continuation_inputs q
             JOIN conversation_ending_decisions d USING(conversation_id,execution_id,source_snapshot,occurrence_id)
             JOIN execution_authentication_applications a ON a.operation_id=d.launch_operation_id AND a.is_current=TRUE
             WHERE q.conversation_id=? AND q.execution_id=? AND q.source_snapshot=? AND d.native_session_id=? AND q.delivery_state<>'CLOSED') AS earlier",
        ))
        .bind::<IdentitySql, _>($conversation)
        .bind::<IdentitySql, _>($execution)
        .bind::<Text, _>($snapshot)
        .bind::<Text, _>($native)
        .get_result::<EarlierContinuation>($connection)
        .await?;
        if pending.earlier {
            return Err(StoreError::InvalidTransition);
        }
    }};
}
pub(in crate::conversations) use require_continuation_order;

macro_rules! qualified_continuation {
    ($connection:expr,$dialect:expr,$source:expr,$conversation:expr,$operation:expr,$ending:expr) => {{
        let conversation = source_conversation!($connection, $dialect, $source, $conversation);
        current(&conversation, conversation.agent_id, &$ending.target)?;
        let route = connector!($connection, $dialect, $source);
        diesel::sql_query($dialect.query(
            "SELECT q.delivery_state, EXISTS(SELECT 1 FROM execution_authentication_applications a WHERE a.operation_id=d.launch_operation_id AND a.is_current=TRUE) AS current_launch
             FROM conversation_continuation_inputs q JOIN conversation_ending_decisions d USING(conversation_id,execution_id,source_snapshot,occurrence_id)
             WHERE q.id=? AND q.conversation_id=? AND q.execution_id=? AND q.source_snapshot=? AND d.native_session_id=? AND q.occurrence_id=? AND d.ordinal=? AND d.work_id=? AND d.launch_operation_id=?",
        ))
        .bind::<IdentitySql, _>($operation)
        .bind::<IdentitySql, _>(conversation.id)
        .bind::<IdentitySql, _>(route._execution_id)
        .bind::<Text, _>($ending.target.snapshot.as_str())
        .bind::<Text, _>($ending.target.native_session.as_str())
        .bind::<Text, _>($ending.position.occurrence().as_str())
        .bind::<BigInt, _>($ending.position.ordinal())
        .bind::<Text, _>(&$ending.work_id)
        .bind::<IdentitySql, _>(EntityId::from_dto(EntityKind::ConversationOperation, &$ending.launch_operation_id)?)
        .get_result::<DeliveryRow>($connection)
        .await
        .optional()?
        .ok_or(StoreError::NotFound)?
    }};
}

impl Store {
    /// Grant once, from the exact admitted ending, with the current context.
    /// A lost grant response is possible forwarding, never permission to resend.
    pub async fn begin_conversation_continuation(
        &self,
        source: &ConnectorSource<'_>,
        conversation: &str,
        operation: &str,
        ending: &ConversationNormalEnding,
    ) -> Result<Option<Vec<u8>>, StoreError> {
        self.begin_conversation_continuation_guarded(source, conversation, operation, ending, false)
            .await
    }
    pub async fn begin_conversation_continuation_guarded(
        &self,
        source: &ConnectorSource<'_>,
        conversation: &str,
        operation: &str,
        ending: &ConversationNormalEnding,
        guards_model_start: bool,
    ) -> Result<Option<Vec<u8>>, StoreError> {
        let id = EntityId::from_dto(EntityKind::Conversation, conversation)?;
        let operation = EntityId::from_dto(EntityKind::ConversationOperation, operation)?;
        operation!(self, |connection, dialect| {
            let row = qualified_continuation!(connection, dialect, source, id, operation, ending);
            crate::conversations::model_selection::starts::require_model_handoff!(
                connection,
                dialect,
                id,
                guards_model_start
            );
            if row.delivery_state != "ADMITTED" || !row.current_launch {
                return Ok(None);
            }
            // Settlement (or wait) can win after admission. Recheck the
            // shared Goal set, not the ordinary input queue: later receipts
            // alone never displace an admitted slot.
            let projection = forwarding_projection(&current_goals!(connection, dialect, id))?;
            let state = if projection.is_some() {
                "POSSIBLE"
            } else {
                // Never hold ordinary input behind an ungrantable slot.
                // The original decision and charge are retained.
                "CLOSED"
            };
            let changed = diesel::sql_query(dialect.query(
                "UPDATE conversation_continuation_inputs SET delivery_state=? WHERE id=? AND delivery_state='ADMITTED'",
            ))
            .bind::<Text, _>(state)
            .bind::<IdentitySql, _>(operation)
            .execute(connection)
            .await?;
            if changed != 1 {
                return Err(StoreError::Conflict);
            }
            Ok(projection)
        })
    }

    /// Release order when the live owner will publish no further native request
    /// for this operation. This does not claim the turn ran or create a receipt.
    pub async fn close_conversation_continuation(
        &self,
        source: &ConnectorSource<'_>,
        conversation: &str,
        operation: &str,
        ending: &ConversationNormalEnding,
    ) -> Result<(), StoreError> {
        let id = EntityId::from_dto(EntityKind::Conversation, conversation)?;
        let operation = EntityId::from_dto(EntityKind::ConversationOperation, operation)?;
        operation!(self, |connection, dialect| {
            let row = qualified_continuation!(connection, dialect, source, id, operation, ending);
            if row.delivery_state != "CLOSED" {
                diesel::sql_query(dialect.query(
                    "UPDATE conversation_continuation_inputs SET delivery_state='CLOSED' WHERE id=?",
                ))
                .bind::<IdentitySql, _>(operation)
                .execute(connection)
                .await?;
            }
            Ok(())
        })
    }
}
