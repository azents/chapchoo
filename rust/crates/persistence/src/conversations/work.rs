//! Conversation Work and its optional Chat publication share a transaction.
use super::{
    goals::{reset_continuation_count, verify_current_goal_size},
    messages::append_message,
    records::source_conversation,
    *,
};
use crate::ConnectorSource;
use chapchoo_domain::{SendAction, WorkTask, WorkTaskStatus};

const MAX_TASK_LIST_BYTES: usize = 48 * 1024;
pub(super) const WORK_OBJECTIVE: &str =
    "Reply to the user's messages and explicitly settle the Work.";

#[derive(Debug)]
pub struct ConversationWorkSummary {
    pub id: String,
    pub title: ConversationText,
    pub tasks: Vec<WorkTask>,
    pub waiting_for_input: bool,
}

#[derive(diesel::QueryableByName)]
pub(super) struct WorkRow {
    #[diesel(sql_type=Text)]
    pub(super) goal_key: String,
    #[diesel(sql_type=Text)]
    title: String,
    #[diesel(sql_type=Text)]
    tasks_json: String,
    #[diesel(sql_type=Text)]
    pub(super) disposition: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct StoredTask<T> {
    text: T,
    status: WorkTaskStatus,
}

impl WorkRow {
    fn tasks(&self) -> Result<Vec<WorkTask>, StoreError> {
        decode_tasks(&self.tasks_json)?
            .into_iter()
            .map(|task| {
                Ok(WorkTask {
                    text: ConversationText::parse(task.text).map_err(|_| StoreError::Integrity)?,
                    status: task.status,
                })
            })
            .collect()
    }

    fn summary(self) -> Result<ConversationWorkSummary, StoreError> {
        let tasks = self.tasks()?;
        Ok(ConversationWorkSummary {
            id: self.goal_key,
            title: ConversationText::parse(self.title).map_err(|_| StoreError::Integrity)?,
            tasks,
            waiting_for_input: self.disposition == "waiting_for_input",
        })
    }
}

pub(super) fn decode_tasks(json: &str) -> Result<Vec<StoredTask<String>>, StoreError> {
    if json.len() > MAX_TASK_LIST_BYTES {
        return Err(StoreError::Integrity);
    }
    serde_json::from_str(json).map_err(|_| StoreError::Integrity)
}

fn encode_tasks(tasks: &[WorkTask]) -> Result<String, StoreError> {
    let items = tasks
        .iter()
        .map(|task| StoredTask {
            text: task.text.as_str(),
            status: task.status,
        })
        .collect::<Vec<_>>();
    let json = serde_json::to_string(&items).map_err(|_| StoreError::Integrity)?;
    if json.len() > MAX_TASK_LIST_BYTES {
        return Err(StoreError::InvalidInput);
    }
    Ok(json)
}

// Work shares its associated goal's disposition. Under the ordinary
// conversation lock, only one associated goal may remain unfinished.
macro_rules! open_work {
    ($connection:expr,$dialect:expr,$conversation:expr) => {{
        let mut rows = diesel::sql_query($dialect.query(
            "SELECT w.goal_key,w.title,w.tasks_json,g.disposition
             FROM conversation_work_items w JOIN conversation_goals g USING (conversation_id,goal_key)
             WHERE w.conversation_id=? AND g.disposition IN ('active','waiting_for_input') LIMIT 2",
        ))
        .bind::<IdentitySql,_>($conversation)
        .load::<$crate::conversations::work::WorkRow>($connection)
        .await?;
        if rows.len() > 1 {
            return Err(StoreError::Integrity);
        }
        rows.pop()
    }};
}

macro_rules! insert_work {
    ($connection:expr,$dialect:expr,$conversation:expr,$now:expr,$title:expr,$tasks:expr,$ready:expr) => {{
        let key = EntityId::generate($now)?.to_dto(EntityKind::ConversationOperation);
        $crate::conversations::goals::insert_goal!(
            $connection, $dialect, $conversation, &key, $crate::conversations::work::WORK_OBJECTIVE
        );
        diesel::sql_query($dialect.query(
            "INSERT INTO conversation_work_items(conversation_id,goal_key,title,tasks_json,reply_ready) VALUES(?,?,?,?,?)",
        ))
        .bind::<IdentitySql,_>($conversation).bind::<Text,_>(&key)
        .bind::<Text,_>($title).bind::<Text,_>($tasks)
        .bind::<diesel::sql_types::Bool,_>($ready)
        .execute($connection).await?;
    }};
}

// In the fresh input transaction, an empty title encodes a plan that has
// not been published. Explicit titles are always nonblank.
macro_rules! accept_reply_work {
    ($connection:expr,$dialect:expr,$conversation:expr,$now:expr) => {{
        if let Some(work) = $crate::conversations::work::open_work!($connection, $dialect, $conversation) {
            if work.disposition == "waiting_for_input" {
                diesel::sql_query($dialect.query(
                    "UPDATE conversation_goals SET disposition='active' WHERE conversation_id=? AND goal_key=?",
                ))
                .bind::<IdentitySql,_>($conversation).bind::<Text,_>(&work.goal_key)
                .execute($connection).await?;
            }
        } else {
            // An input must remain admissible even if legacy internal Goals
            // already fill the projection. Admission still checks the bound.
            $crate::conversations::work::insert_work!($connection, $dialect, $conversation, $now, "", "[]", false);
        }
        $crate::conversations::goals::reset_continuation_count!($connection, $dialect, $conversation);
    }};
}
pub(super) use {accept_reply_work, insert_work, open_work};

impl Store {
    /// No caller-supplied work identity: the authenticated conversation selects it.
    pub async fn send_conversation_work(
        &self,
        source: &ConnectorSource<'_>,
        scope: &ConversationMessageScope<'_>,
        action: &SendAction,
    ) -> Result<Option<ConversationMessageReceipt>, StoreError> {
        let agent = EntityId::from_dto(EntityKind::Agent, scope.agent_id)?;
        let id = EntityId::from_dto(EntityKind::Conversation, scope.conversation_id)?;
        let tasks_json = match action {
            SendAction::Continue { title, tasks, .. } => {
                if title
                    .as_ref()
                    .is_some_and(|text| text.as_str().len() > goals::MAX_GOAL_CONTEXT_BYTES)
                {
                    return Err(StoreError::InvalidInput);
                }
                Some(encode_tasks(tasks)?)
            }
            _ => None,
        };
        operation!(self, |connection, dialect| {
            let row = source_conversation!(connection, dialect, source, id);
            if row.agent_id != agent {
                return Err(StoreError::NotFound);
            }
            let active = open_work!(connection, dialect, id);
            let now = crate::directory::database_now!(connection, dialect);
            match action {
                SendAction::Continue { title, tasks, .. } => {
                    let json = tasks_json.as_deref().ok_or(StoreError::Integrity)?;
                    if let Some(active) = active {
                        if active.title.is_empty() && title.is_none() {
                            return Err(StoreError::InvalidInput);
                        }
                        if active.tasks()? != *tasks {
                            reset_continuation_count!(connection, dialect, id);
                        }
                        diesel::sql_query(dialect.query(
                            "UPDATE conversation_work_items SET title=COALESCE(?,title),tasks_json=? WHERE conversation_id=? AND goal_key=?",
                        ))
                        .bind::<Nullable<Text>,_>(title.as_ref().map(ConversationText::as_str))
                        .bind::<Text,_>(json).bind::<IdentitySql,_>(id).bind::<Text,_>(&active.goal_key)
                        .execute(connection).await?;
                    } else {
                        let title = title.as_ref().ok_or(StoreError::InvalidInput)?.as_str();
                        insert_work!(connection, dialect, id, now, title, json, true);
                    }
                    verify_current_goal_size!(connection, dialect, id);
                }
                SendAction::RequestInput { .. }
                | SendAction::Finish { .. }
                | SendAction::Ignore => {
                    if let Some(active) = active {
                        let disposition = if matches!(action, SendAction::RequestInput { .. }) {
                            "waiting_for_input"
                        } else {
                            "completed"
                        };
                        // A reply to already delivered input does not settle
                        // ordinary input still awaiting delivery.
                        let settled = diesel::sql_query(dialect.query(
                            "UPDATE conversation_goals SET disposition=? WHERE conversation_id=? AND goal_key=?
                             AND NOT EXISTS (SELECT 1 FROM conversation_inputs WHERE conversation_id=? AND queue_state='PENDING' AND deleted_at IS NULL)",
                        ))
                        .bind::<Text,_>(disposition).bind::<IdentitySql,_>(id).bind::<Text,_>(&active.goal_key)
                        .bind::<IdentitySql,_>(id)
                        .execute(connection).await?;
                        if settled == 0 {
                            // Only a successful pickup can make the remaining
                            // input eligible to drive continuation.
                            diesel::sql_query(dialect.query(
                                "UPDATE conversation_work_items SET reply_ready=FALSE WHERE conversation_id=? AND goal_key=?",
                            ))
                            .bind::<IdentitySql,_>(id).bind::<Text,_>(&active.goal_key)
                            .execute(connection).await?;
                        }
                    }
                }
            }
            let message = match action.message() {
                Some(text) => Some(append_message!(
                    connection,
                    dialect,
                    id,
                    MessageOrigin::Agent,
                    text,
                    now
                )),
                None => None,
            };
            Ok(message)
        })
    }

    pub async fn conversation_work_summary(
        &self,
        actor: &SecretDigest,
        scope: &ConversationMessageScope<'_>,
        now: i64,
    ) -> Result<Option<ConversationWorkSummary>, StoreError> {
        let agent = EntityId::from_dto(EntityKind::Agent, scope.agent_id)?;
        let id = EntityId::from_dto(EntityKind::Conversation, scope.conversation_id)?;
        operation!(self, |connection, dialect| {
            let _ = owned_agent!(connection, dialect, actor, agent, now);
            let row = conversation!(connection, dialect, id);
            if row.agent_id != agent {
                return Err(StoreError::NotFound);
            }
            open_work!(connection, dialect, id)
                .filter(|work| !work.title.is_empty())
                .map(WorkRow::summary)
                .transpose()
        })
    }
}
