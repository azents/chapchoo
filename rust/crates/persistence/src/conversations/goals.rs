//! Server-owned goal state. Native executions never own an objective.

use super::{delivery::current, records::source_conversation, *};
use crate::ConnectorSource;
use serde::Serialize;

pub(super) const MAX_GOAL_CONTEXT_BYTES: usize = 8 * 1024;
const MAX_GOAL_KEY_BYTES: usize = 128;

/// The key is issued by the server-side consumer and retained for its work.
/// This operation is not an interface for forwarding model-supplied identities.
pub enum ConversationGoalChange<'a> {
    Add { objective: &'a str },
    Amend { objective: &'a str },
    Complete,
}
impl fmt::Debug for ConversationGoalChange<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ConversationGoalChange([REDACTED])")
    }
}

#[derive(diesel::QueryableByName, Serialize)]
pub(super) struct GoalRow {
    #[diesel(sql_type=Text)]
    goal_key: String,
    #[diesel(sql_type=Text)]
    objective: String,
    #[diesel(sql_type=Text)]
    disposition: String,
}

#[derive(diesel::QueryableByName)]
struct GoalIdentity {
    #[diesel(sql_type=Text)]
    original_objective: String,
    #[diesel(sql_type=Text)]
    disposition: String,
}

macro_rules! reset_continuation_count {
    ($connection:expr,$dialect:expr,$conversation:expr) => {
        diesel::sql_query($dialect.query(
            "UPDATE conversation_continuation_counts SET admissions=0 WHERE conversation_id=? AND admissions<>0",
        ))
        .bind::<IdentitySql, _>($conversation)
        .execute($connection)
        .await?
    };
}
pub(super) use reset_continuation_count;

macro_rules! current_goals {
    ($connection:expr,$dialect:expr,$conversation:expr) => {
        diesel::sql_query($dialect.query(
            "SELECT goal_key,objective,disposition FROM conversation_goals WHERE conversation_id=? AND disposition IN ('active','waiting_for_input') ORDER BY goal_key",
        ))
        .bind::<IdentitySql, _>($conversation)
        .load::<$crate::conversations::goals::GoalRow>($connection)
        .await?
    };
}

macro_rules! current_projection {
    ($connection:expr,$dialect:expr,$conversation:expr) => {{
        let projection = $crate::conversations::goals::encode(
            &$crate::conversations::goals::current_goals!($connection, $dialect, $conversation),
        )?;
        if projection.len() > $crate::conversations::goals::MAX_GOAL_CONTEXT_BYTES {
            return Err(StoreError::Integrity);
        }
        projection
    }};
}
pub(super) use {current_goals, current_projection};

pub(super) fn encode(goals: &[GoalRow]) -> Result<Vec<u8>, StoreError> {
    serde_json::to_vec(goals).map_err(|_| StoreError::Integrity)
}

pub(super) fn verify_projection_size(goals: &mut [GoalRow]) -> Result<(), StoreError> {
    // Reserve the longest live status so input-wait changes cannot overflow.
    for goal in &mut *goals {
        goal.disposition = "waiting_for_input".into();
    }
    if encode(goals)?.len() > MAX_GOAL_CONTEXT_BYTES {
        return Err(StoreError::InvalidInput);
    }
    Ok(())
}

impl Store {
    /// Load the same complete, current projection used for restoration and
    /// continuation. An empty set is an authoritative success.
    pub async fn conversation_goal_context(
        &self,
        source: &ConnectorSource<'_>,
        conversation_id: &str,
        target: &ConversationDeliveryTarget,
    ) -> Result<Vec<u8>, StoreError> {
        let id = EntityId::from_dto(EntityKind::Conversation, conversation_id)?;
        operation!(self, |connection, dialect| {
            let row = source_conversation!(connection, dialect, source, id);
            current(&row, row.agent_id, target)?;
            Ok(current_projection!(connection, dialect, id))
        })
    }

    /// Change one domain-owned goal. This server-side operation is not
    /// registered as a Connector or model mutation endpoint.
    pub async fn change_conversation_goal(
        &self,
        source: &ConnectorSource<'_>,
        conversation_id: &str,
        target: &ConversationDeliveryTarget,
        goal_key: &str,
        change: ConversationGoalChange<'_>,
    ) -> Result<(), StoreError> {
        if goal_key.is_empty()
            || goal_key.len() > MAX_GOAL_KEY_BYTES
            || !goal_key.bytes().all(|c| c.is_ascii_graphic())
        {
            return Err(StoreError::InvalidInput);
        }
        if let ConversationGoalChange::Add { objective }
        | ConversationGoalChange::Amend { objective } = &change
            && (objective.trim().is_empty()
                || objective.len() > MAX_GOAL_CONTEXT_BYTES
                || objective.contains('\0'))
        {
            return Err(StoreError::InvalidInput);
        }
        let id = EntityId::from_dto(EntityKind::Conversation, conversation_id)?;
        operation!(self, |connection, dialect| {
            let row = source_conversation!(connection, dialect, source, id);
            current(&row, row.agent_id, target)?;
            let prior = diesel::sql_query(dialect.query(
                "SELECT original_objective,disposition FROM conversation_goals WHERE conversation_id=? AND goal_key=?",
            ))
            .bind::<IdentitySql, _>(id)
            .bind::<Text, _>(goal_key)
            .get_result::<GoalIdentity>(connection)
            .await
            .optional()?;
            match &change {
                ConversationGoalChange::Add { objective } => {
                    if let Some(prior) = prior {
                        return if prior.original_objective == *objective {
                            Ok(())
                        } else {
                            Err(StoreError::Conflict)
                        };
                    }
                    diesel::sql_query(dialect.query("INSERT INTO conversation_goals(conversation_id,goal_key,original_objective,objective,disposition) VALUES(?,?,?,?,'active')"))
                        .bind::<IdentitySql, _>(id)
                        .bind::<Text, _>(goal_key)
                        .bind::<Text, _>(*objective)
                        .bind::<Text, _>(*objective)
                        .execute(connection)
                        .await?;
                    verify_projection_size(&mut current_goals!(connection, dialect, id))?;
                    reset_continuation_count!(connection, dialect, id);
                }
                ConversationGoalChange::Amend { objective } => {
                    let changed = diesel::sql_query(dialect.query("UPDATE conversation_goals SET objective=? WHERE conversation_id=? AND goal_key=? AND disposition IN ('active','waiting_for_input')"))
                        .bind::<Text, _>(*objective)
                        .bind::<IdentitySql, _>(id)
                        .bind::<Text, _>(goal_key)
                        .execute(connection)
                        .await?;
                    if changed != 1 {
                        return Err(StoreError::InvalidTransition);
                    }
                    verify_projection_size(&mut current_goals!(connection, dialect, id))?;
                }
                ConversationGoalChange::Complete => {
                    let prior = prior.ok_or(StoreError::NotFound)?;
                    match prior.disposition.as_str() {
                        "completed" => return Ok(()),
                        "active" | "waiting_for_input" => {}
                        _ => return Err(StoreError::InvalidTransition),
                    }
                    diesel::sql_query(dialect.query("UPDATE conversation_goals SET disposition='completed' WHERE conversation_id=? AND goal_key=?"))
                        .bind::<IdentitySql, _>(id)
                        .bind::<Text, _>(goal_key)
                        .execute(connection)
                        .await?;
                }
            }
            Ok(())
        })
    }
}
