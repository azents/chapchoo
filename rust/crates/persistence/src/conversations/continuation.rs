//! The database decision precedes acknowledgement and native delivery.

use super::{delivery::current, records::source_conversation, *};
use crate::ConnectorSource;

mod forwarding;

pub(super) use forwarding::require_continuation_order;

const AUTOMATIC_ADMISSION_LIMIT: i64 = 3;

pub struct ConversationNormalEnding {
    pub target: ConversationDeliveryTarget,
    pub position: SourcePosition,
    pub work_id: String,
    pub launch_operation_id: String,
}
impl fmt::Debug for ConversationNormalEnding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ConversationNormalEnding([REDACTED])")
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum ConversationContinuationDecision {
    NoAction,
    Admitted { operation_id: String },
}
impl fmt::Debug for ConversationContinuationDecision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoAction => f.write_str("NoAction"),
            Self::Admitted { .. } => f.write_str("Admitted([REDACTED])"),
        }
    }
}

#[derive(diesel::QueryableByName)]
struct DecisionRow {
    #[diesel(sql_type=BigInt)]
    ordinal: i64,
    #[diesel(sql_type=Text)]
    work_id: String,
    #[diesel(sql_type=Text)]
    native_session_id: String,
    #[diesel(sql_type=IdentitySql)]
    launch_operation_id: EntityId,
    #[diesel(sql_type=Nullable<IdentitySql>)]
    operation_id: Option<EntityId>,
}

/// The existing authentication application is admitted only on physical
/// launch, and credential file refresh keeps that operation unchanged.
macro_rules! require_current_launch {
    ($connection:expr,$dialect:expr,$conversation:expr,$execution:expr,$launch:expr) => {{
        #[derive(diesel::QueryableByName)]
        struct Launch {
            #[diesel(sql_type=IdentitySql)]
            operation_id: EntityId,
        }
        let applied = diesel::sql_query($dialect.query(
            "SELECT operation_id FROM execution_authentication_applications WHERE execution_id=? AND conversation_id=? AND is_current=TRUE",
        ))
        .bind::<IdentitySql,_>($execution)
        .bind::<IdentitySql,_>($conversation)
        .get_result::<Launch>($connection)
        .await
        .optional()?;
        if applied.is_none_or(|applied| applied.operation_id != $launch) {
            return Err(StoreError::Conflict);
        }
    }};
}

#[derive(diesel::QueryableByName)]
struct CapturedEnding {
    #[diesel(sql_type=Binary)]
    raw_event: Vec<u8>,
}

fn verify_terminal(raw: &[u8], target: &ConversationDeliveryTarget) -> Result<(), StoreError> {
    #[derive(serde::Deserialize)]
    struct Event {
        method: String,
        params: Params,
    }
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Params {
        session_id: String,
        update: Update,
    }
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Update {
        session_update: String,
        state: String,
        stop_reason: String,
    }
    let actual: Event = serde_json::from_slice(raw).map_err(|_| StoreError::InvalidTransition)?;
    if actual.method == "session/update"
        && actual.params.session_id == target.native_session.as_str()
        && actual.params.update.session_update == "state_update"
        && actual.params.update.state == "idle"
        && actual.params.update.stop_reason == "end_turn"
    {
        Ok(())
    } else {
        Err(StoreError::InvalidTransition)
    }
}

impl Store {
    /// Commit one decision for a live-ending report from the current Connector.
    /// The captured frame is checked for consistency, not used as a producer.
    pub async fn admit_conversation_continuation(
        &self,
        source: &ConnectorSource<'_>,
        conversation_id: &str,
        ending: &ConversationNormalEnding,
    ) -> Result<ConversationContinuationDecision, StoreError> {
        if ending.work_id.is_empty()
            || ending.work_id.len() > 128
            || !ending.work_id.bytes().all(|byte| byte.is_ascii_graphic())
        {
            return Err(StoreError::InvalidInput);
        }
        let id = EntityId::from_dto(EntityKind::Conversation, conversation_id)?;
        let launch = EntityId::from_dto(
            EntityKind::ConversationOperation,
            &ending.launch_operation_id,
        )?;
        let snapshot = ending.target.snapshot.as_str();
        let native = ending.target.native_session.as_str();
        let occurrence = ending.position.occurrence().as_str();
        let ordinal = ending.position.ordinal();
        operation!(self, |connection, dialect| {
            let row = source_conversation!(connection, dialect, source, id);
            current(&row, row.agent_id, &ending.target)?;
            if !matches!(row.source()?, ConversationSource::Ready { .. }) {
                return Err(StoreError::InvalidTransition);
            }
            let execution = connector!(connection, dialect, source)._execution_id;
            require_current_launch!(connection, dialect, id, execution, launch);
            let prior = diesel::sql_query(dialect.query(
                "SELECT d.ordinal,d.work_id,d.native_session_id,d.launch_operation_id,q.id AS operation_id FROM conversation_ending_decisions d LEFT JOIN conversation_continuation_inputs q USING(conversation_id,execution_id,source_snapshot,occurrence_id) WHERE d.conversation_id=? AND d.execution_id=? AND d.source_snapshot=? AND d.occurrence_id=?",
            ))
            .bind::<IdentitySql, _>(id).bind::<IdentitySql, _>(execution)
            .bind::<Text, _>(snapshot).bind::<Text, _>(occurrence)
            .get_result::<DecisionRow>(connection).await.optional()?;
            if let Some(prior) = prior {
                if prior.ordinal != ordinal
                    || prior.work_id != ending.work_id
                    || prior.native_session_id != native
                    || prior.launch_operation_id != launch
                {
                    return Err(StoreError::Conflict);
                }
                return Ok(prior.operation_id.map_or(
                    ConversationContinuationDecision::NoAction,
                    |operation| ConversationContinuationDecision::Admitted {
                        operation_id: operation.to_dto(EntityKind::ConversationOperation),
                    },
                ));
            }
            let captured = diesel::sql_query(dialect.query(
                "SELECT e.raw_event FROM conversation_event_pairs e
                 JOIN conversation_launch_captures l ON l.operation_id=? AND l.source_snapshot=e.source_snapshot
                 WHERE e.conversation_id=? AND e.source_snapshot=? AND e.occurrence_id=? AND e.ordinal=?
                   AND e.ordinal>l.retained_ordinal
                   AND (l.retained_ordinal=0 OR EXISTS (
                     SELECT 1 FROM conversation_event_pairs retained
                     WHERE retained.conversation_id=e.conversation_id AND retained.source_snapshot=e.source_snapshot
                       AND retained.ordinal=l.retained_ordinal AND retained.occurrence_id=l.retained_occurrence_id
                   ))",
            ))
            .bind::<IdentitySql, _>(launch)
            .bind::<IdentitySql, _>(id).bind::<Text, _>(snapshot)
            .bind::<Text, _>(occurrence).bind::<BigInt, _>(ordinal)
            .get_result::<CapturedEnding>(connection).await.optional()?.ok_or(StoreError::InvalidTransition)?;
            verify_terminal(&captured.raw_event, &ending.target)?;
            let now = crate::directory::database_now!(connection, dialect);
            let inserted = diesel::sql_query(dialect.query(
                "INSERT INTO conversation_ending_decisions(conversation_id,execution_id,launch_operation_id,source_snapshot,native_session_id,ordinal,occurrence_id,work_id) VALUES(?,?,?,?,?,?,?,?) ON CONFLICT DO NOTHING",
            ))
            .bind::<IdentitySql, _>(id).bind::<IdentitySql, _>(execution)
            .bind::<IdentitySql, _>(launch)
            .bind::<Text, _>(snapshot).bind::<Text, _>(native)
            .bind::<BigInt, _>(ordinal).bind::<Text, _>(occurrence).bind::<Text, _>(&ending.work_id)
            .execute(connection).await?;
            if inserted != 1 {
                return Err(StoreError::Conflict);
            }
            // A retained draft Work can exceed the full projection budget.
            // Record the decision without charging or rewriting that data.
            if super::goals::current_projection!(connection, dialect, id).is_none() {
                return Ok(ConversationContinuationDecision::NoAction);
            }
            let charged = diesel::sql_query(dialect.query(
                "INSERT INTO conversation_continuation_counts(conversation_id,admissions)
                 SELECT ?,1 WHERE EXISTS (
                   SELECT 1 FROM conversation_goals g LEFT JOIN conversation_work_items w USING (conversation_id,goal_key)
                   WHERE g.conversation_id=? AND g.disposition='active' AND COALESCE(w.reply_ready,TRUE))
                 AND NOT EXISTS (SELECT 1 FROM conversation_inputs WHERE conversation_id=? AND queue_state='PENDING' AND deleted_at IS NULL)
                 ON CONFLICT(conversation_id) DO UPDATE SET admissions=conversation_continuation_counts.admissions+1 WHERE conversation_continuation_counts.admissions<?",
            ))
            .bind::<IdentitySql, _>(id).bind::<IdentitySql, _>(id).bind::<IdentitySql, _>(id)
            .bind::<BigInt, _>(AUTOMATIC_ADMISSION_LIMIT)
            .execute(connection).await?;
            if charged != 1 {
                return Ok(ConversationContinuationDecision::NoAction);
            }
            let operation = EntityId::generate(now)?;
            diesel::sql_query(dialect.query(
                "INSERT INTO conversation_continuation_inputs(conversation_id,execution_id,source_snapshot,occurrence_id,id) VALUES(?,?,?,?,?)",
            ))
            .bind::<IdentitySql, _>(id).bind::<IdentitySql, _>(execution)
            .bind::<Text, _>(snapshot).bind::<Text, _>(occurrence).bind::<IdentitySql, _>(operation)
            .execute(connection).await?;
            Ok(ConversationContinuationDecision::Admitted {
                operation_id: operation.to_dto(EntityKind::ConversationOperation),
            })
        })
    }
}
