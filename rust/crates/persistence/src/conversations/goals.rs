//! Server-owned goal state. Native executions never own an objective.

use super::{
    delivery::current,
    records::source_conversation,
    work::{StoredTask, decode_tasks},
    *,
};
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

#[derive(diesel::QueryableByName)]
pub(super) struct GoalRow {
    #[diesel(sql_type=Text)]
    goal_key: String,
    #[diesel(sql_type=Text)]
    objective: String,
    #[diesel(sql_type=Text)]
    disposition: String,
    #[diesel(sql_type=Nullable<Text>)]
    work_title: Option<String>,
    #[diesel(sql_type=Nullable<Text>)]
    work_tasks_json: Option<String>,
    #[diesel(sql_type=diesel::sql_types::Bool)]
    reply_ready: bool,
}

#[derive(Serialize)]
struct ProjectedGoal<'a> {
    goal_key: &'a str,
    disposition: &'a str,
    #[serde(flatten)]
    context: GoalContext<'a>,
}

#[derive(Serialize)]
#[serde(untagged)]
enum GoalContext<'a> {
    Objective {
        objective: &'a str,
    },
    Work {
        title: &'a str,
        tasks: Vec<StoredTask<String>>,
    },
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
            "SELECT g.goal_key,g.objective,g.disposition,NULLIF(w.title,'') AS work_title,
                    CASE WHEN w.title<>'' THEN w.tasks_json END AS work_tasks_json,
                    COALESCE(w.reply_ready,TRUE) AS reply_ready
             FROM conversation_goals g LEFT JOIN conversation_work_items w
               ON w.conversation_id=g.conversation_id AND w.goal_key=g.goal_key
             WHERE g.conversation_id=? AND g.disposition IN ('active','waiting_for_input') ORDER BY g.goal_key",
        ))
        .bind::<IdentitySql, _>($conversation)
        .load::<$crate::conversations::goals::GoalRow>($connection)
        .await?
    };
}

macro_rules! current_projection {
    ($connection:expr,$dialect:expr,$conversation:expr) => {{
        $crate::conversations::goals::bounded_projection(
            &$crate::conversations::goals::current_goals!($connection, $dialect, $conversation),
        )?
    }};
}
pub(super) use {current_goals, current_projection};

macro_rules! verify_current_goal_size {
    ($connection:expr,$dialect:expr,$conversation:expr) => {
        $crate::conversations::goals::verify_projection_size(
            &$crate::conversations::goals::current_goals!($connection, $dialect, $conversation),
        )?
    };
}

// Call under the conversation transaction after selecting the authorized
// goal identity. An insertion and its shared-budget reset are one effect.
macro_rules! insert_goal {
    ($connection:expr,$dialect:expr,$conversation:expr,$key:expr,$objective:expr) => {{
        diesel::sql_query($dialect.query(
            "INSERT INTO conversation_goals(conversation_id,goal_key,original_objective,objective,disposition) VALUES(?,?,?,?,'active')",
        ))
        .bind::<IdentitySql, _>($conversation)
        .bind::<Text, _>($key)
        .bind::<Text, _>($objective)
        .bind::<Text, _>($objective)
        .execute($connection)
        .await?;
        $crate::conversations::goals::reset_continuation_count!(
            $connection, $dialect, $conversation
        );
    }};
}

pub(super) use {insert_goal, verify_current_goal_size};

pub(super) fn bounded_projection(goals: &[GoalRow]) -> Result<Option<Vec<u8>>, StoreError> {
    let projection = encode_projection(goals, false)?;
    Ok((projection.len() <= MAX_GOAL_CONTEXT_BYTES).then_some(projection))
}

pub(super) fn forwarding_projection(goals: &[GoalRow]) -> Result<Option<Vec<u8>>, StoreError> {
    if !goals
        .iter()
        .any(|goal| goal.disposition == "active" && goal.reply_ready)
    {
        return Ok(None);
    }
    bounded_projection(goals)
}

pub(super) fn verify_projection_size(goals: &[GoalRow]) -> Result<(), StoreError> {
    if encode_projection(goals, true)?.len() > MAX_GOAL_CONTEXT_BYTES {
        return Err(StoreError::InvalidInput);
    }
    Ok(())
}

fn encode_projection(goals: &[GoalRow], reserve_wait: bool) -> Result<Vec<u8>, StoreError> {
    let projection = goals
        .iter()
        // Count the Work against the full size bound even before pickup,
        // but do not tell the model to answer input it hasn't received.
        .filter(|goal| reserve_wait || goal.reply_ready)
        .map(|goal| {
            let context = match (&goal.work_title, &goal.work_tasks_json) {
                (Some(title), Some(tasks)) => GoalContext::Work {
                    title,
                    tasks: decode_tasks(tasks)?,
                },
                (None, None) => GoalContext::Objective {
                    objective: &goal.objective,
                },
                _ => return Err(StoreError::Integrity),
            };
            Ok(ProjectedGoal {
                goal_key: &goal.goal_key,
                // Reserve the longest live status so input wait never fails for size.
                disposition: if reserve_wait {
                    "waiting_for_input"
                } else {
                    &goal.disposition
                },
                context,
            })
        })
        .collect::<Result<Vec<_>, StoreError>>()?;
    serde_json::to_vec(&projection).map_err(|_| StoreError::Integrity)
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
            current_projection!(connection, dialect, id).ok_or(StoreError::Integrity)
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
                    insert_goal!(connection, dialect, id, goal_key, *objective);
                    verify_current_goal_size!(connection, dialect, id);
                }
                ConversationGoalChange::Amend { objective } => {
                    let changed = diesel::sql_query(dialect.query(
                        "UPDATE conversation_goals SET objective=? WHERE conversation_id=? AND goal_key=?
                         AND disposition IN ('active','waiting_for_input') AND NOT EXISTS (
                             SELECT 1 FROM conversation_work_items w
                             WHERE w.conversation_id=conversation_goals.conversation_id AND w.goal_key=conversation_goals.goal_key)",
                    ))
                    .bind::<Text, _>(*objective)
                    .bind::<IdentitySql, _>(id)
                    .bind::<Text, _>(goal_key)
                    .execute(connection)
                    .await?;
                    if changed != 1 {
                        return Err(StoreError::InvalidTransition);
                    }
                    verify_current_goal_size!(connection, dialect, id);
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
