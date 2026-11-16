//! Bounded live ACPv2 work evidence; metadata idle is not physical completion.
use crate::Error;
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};
const MAX_SESSIONS: usize = 129;
const MAX_CALLS: usize = 128;
const MAX_ID_BYTES: usize = 512;
const MAX_FRAME_BYTES: usize = 16 * 1024 * 1024;
#[derive(Clone, Copy, PartialEq, Eq)]
enum Work {
    Unknown,
    Active,
    Idle,
}
#[derive(Default)]
pub(super) struct ActivityTracker {
    root: Option<String>,
    complete: bool,
    failed: bool,
    sessions: BTreeMap<String, Work>,
    started: BTreeSet<String>,
    unassociated: BTreeSet<String>,
    calls: BTreeMap<(String, String), bool>,
    terminals: BTreeMap<(String, String), bool>,
}
impl fmt::Debug for ActivityTracker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ActivityTracker([REDACTED])")
    }
}
#[derive(Deserialize)]
struct Method {
    method: String,
}
#[derive(Deserialize)]
struct Notification {
    params: Params,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Params {
    session_id: String,
    update: WorkEvent,
}
#[derive(Deserialize)]
#[serde(tag = "sessionUpdate", rename_all = "snake_case")]
enum WorkEvent {
    StateUpdate {
        state: Foreground,
        #[serde(rename = "stopReason", default)]
        stop_reason: Option<String>,
    },
    SubagentUpdate {
        #[serde(rename = "sessionId")]
        child: String,
    },
    ToolCallUpdate {
        #[serde(rename = "toolCallId")]
        tool: String,
        #[serde(default)]
        status: Option<CallState>,
    },
    ToolCallContentChunk {
        #[serde(rename = "toolCallId")]
        tool: String,
    },
    TerminalUpdate {
        #[serde(rename = "terminalId")]
        terminal: String,
        #[serde(rename = "exitStatus", default)]
        exit: Option<TerminalExit>,
    },
    TerminalOutputChunk {
        #[serde(rename = "terminalId")]
        terminal: String,
    },
    #[serde(other)]
    Other,
}
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum Foreground {
    Idle,
    Running,
    RequiresAction,
    #[serde(other)]
    Unknown,
}
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum CallState {
    Completed,
    #[serde(other)]
    Unresolved,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TerminalExit {
    exit_code: Option<i64>,
}
impl ActivityTracker {
    #[cfg(test)]
    fn new(root: &str, empty: bool) -> Result<Self, Error> {
        let mut value = Self::default();
        value.bind(root, empty)?;
        Ok(value)
    }
    /// Empty scope requires New/pre-work or exact safe local retirement.
    pub(super) fn bind(&mut self, root: &str, empty: bool) -> Result<(), Error> {
        valid_id(root)?;
        if let Some(previous) = &self.root {
            if previous != root {
                self.mark_uncertain();
                return Err(Error::CaptureConflict);
            }
            return Ok(());
        }
        if !self.sessions.contains_key(root) {
            self.add_session(root, Work::Unknown)?;
        }
        if empty && !self.started.contains(root) {
            self.sessions.insert(root.to_owned(), Work::Idle);
        }
        self.root = Some(root.to_owned());
        self.unassociated.remove(root);
        self.complete = empty && !self.failed;
        Ok(())
    }
    pub(super) fn observe(&mut self, line: &str) -> Result<(), Error> {
        let result = self.observe_inner(line);
        if result.is_err() {
            self.mark_uncertain();
        }
        result
    }
    pub(super) fn mark_uncertain(&mut self) {
        self.complete = false;
        self.failed = true;
    }
    pub(super) fn contains_child(&self, session: &str) -> bool {
        self.root.as_deref().is_some_and(|root| root != session)
            && self.sessions.contains_key(session)
            && !self.unassociated.contains(session)
    }
    pub(super) fn child_request(&mut self, session: &str) {
        self.started.insert(session.to_owned());
        self.sessions.insert(session.to_owned(), Work::Active);
    }
    pub(super) fn is_quiescent(&self, root: &str) -> bool {
        self.root.as_deref() == Some(root)
            && self.complete
            && self.unassociated.is_empty()
            && self.sessions.values().all(|work| *work == Work::Idle)
            && self.calls.values().all(|finished| *finished)
            && self.terminals.values().all(|finished| *finished)
    }
    fn observe_inner(&mut self, line: &str) -> Result<(), Error> {
        if line.len() > MAX_FRAME_BYTES {
            return Err(Error::NotAdmitted);
        }
        let method: Method = serde_json::from_str(line).map_err(|_| Error::Protocol)?;
        if method.method != "session/update" {
            return Ok(());
        }
        let Notification { params } = serde_json::from_str(line).map_err(|_| Error::Protocol)?;
        if matches!(params.update, WorkEvent::Other) {
            return Ok(());
        }
        valid_id(&params.session_id)?;
        if !self.sessions.contains_key(&params.session_id) {
            self.add_session(&params.session_id, Work::Unknown)?;
            self.unassociated.insert(params.session_id.clone());
        }
        match params.update {
            WorkEvent::StateUpdate { state, stop_reason } => {
                let previous = self.sessions.get(&params.session_id).copied();
                let work = match state {
                    Foreground::Running | Foreground::RequiresAction => {
                        // Collect only established completion, never unresolved work.
                        self.calls.retain(|_, finished| !*finished);
                        self.terminals.retain(|_, finished| !*finished);
                        self.started.insert(params.session_id.clone());
                        Work::Active
                    }
                    Foreground::Idle
                        if stop_reason.as_deref() == Some("end_turn")
                            && previous == Some(Work::Active) =>
                    {
                        Work::Idle
                    }
                    Foreground::Idle if stop_reason.is_none() && previous == Some(Work::Idle) => {
                        Work::Idle
                    }
                    Foreground::Idle | Foreground::Unknown => Work::Unknown,
                };
                self.sessions.insert(params.session_id, work);
            }
            WorkEvent::SubagentUpdate { child } => {
                if !self.sessions.contains_key(&child) {
                    self.add_session(&child, Work::Unknown)?;
                }
                self.unassociated.remove(&child);
            }
            WorkEvent::ToolCallUpdate { tool, status } => {
                let key = bounded_key(&self.calls, params.session_id, tool)?;
                // ACPv2 has no create event. Every first update admits the call.
                // Codex uses its call identity for the independent terminal too.
                let finished = matches!(status, Some(CallState::Completed))
                    || self.terminals.get(&key) == Some(&true);
                *self.calls.entry(key).or_insert(false) |= finished;
            }
            WorkEvent::ToolCallContentChunk { tool } => {
                let key = bounded_key(&self.calls, params.session_id, tool)?;
                self.calls.entry(key).or_insert(false);
            }
            WorkEvent::TerminalUpdate { terminal, exit } => {
                let key = bounded_key(&self.terminals, params.session_id, terminal)?;
                let finished = exit.is_some_and(|exit| exit.exit_code.is_some());
                // Null exits may be synthesized after lost completion; they
                // neither prove an exit nor regress a known physical exit.
                *self.terminals.entry(key.clone()).or_insert(false) |= finished;
                if finished && let Some(call) = self.calls.get_mut(&key) {
                    *call = true;
                }
            }
            WorkEvent::TerminalOutputChunk { terminal } => {
                let key = bounded_key(&self.terminals, params.session_id, terminal)?;
                self.terminals.entry(key).or_insert(false);
            }
            WorkEvent::Other => {}
        }
        Ok(())
    }
    fn add_session(&mut self, id: &str, work: Work) -> Result<(), Error> {
        valid_id(id)?;
        if !self.sessions.contains_key(id) && self.sessions.len() >= MAX_SESSIONS {
            return Err(Error::NotAdmitted);
        }
        self.sessions.insert(id.to_owned(), work);
        Ok(())
    }
}
fn bounded_key(
    entries: &BTreeMap<(String, String), bool>,
    session: String,
    id: String,
) -> Result<(String, String), Error> {
    valid_id(&id)?;
    let key = (session, id);
    if !entries.contains_key(&key) && entries.len() >= MAX_CALLS {
        return Err(Error::NotAdmitted);
    }
    Ok(key)
}
fn valid_id(id: &str) -> Result<(), Error> {
    if id.is_empty() || id.len() > MAX_ID_BYTES || id.chars().any(char::is_control) {
        return Err(Error::Protocol);
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    type TestError = Box<dyn std::error::Error>;
    fn update(session: &str, body: serde_json::Value) -> String {
        json!({"method":"session/update","params":{"sessionId":session,"update":body}}).to_string()
    }
    fn running(session: &str) -> String {
        update(
            session,
            json!({"sessionUpdate":"state_update","state":"running"}),
        )
    }
    fn end(session: &str) -> String {
        update(
            session,
            json!({"sessionUpdate":"state_update","state":"idle","stopReason":"end_turn"}),
        )
    }
    fn call(session: &str, id: &str) -> String {
        update(
            session,
            json!({"sessionUpdate":"tool_call_update","toolCallId":id,"status":"in_progress"}),
        )
    }
    fn terminal(session: &str, id: &str) -> String {
        update(
            session,
            json!({"sessionUpdate":"terminal_update","terminalId":id,"command":"fixture command","cwd":"/project"}),
        )
    }
    fn exit(session: &str, id: &str, code: serde_json::Value) -> String {
        update(
            session,
            json!({"sessionUpdate":"terminal_update","terminalId":id,"exitStatus":{"exitCode":code,"signal":null}}),
        )
    }
    #[test]
    fn v2_first_upsert_and_independent_terminal_outlive_root_idle() -> Result<(), TestError> {
        let mut tracker = ActivityTracker::new("root", true)?;
        tracker.observe(&running("root"))?;
        // These are stock ACPv2 producer shapes, with terminal before call.
        tracker.observe(&terminal("root", "shell"))?;
        tracker.observe(&call("root", "shell"))?;
        tracker.observe(&end("root"))?;
        assert!(!tracker.is_quiescent("root"));
        tracker.observe(&update(
            "root",
            json!({"sessionUpdate":"tool_call_update","toolCallId":"shell","status":"completed"}),
        ))?;
        assert!(!tracker.is_quiescent("root"));
        tracker.observe(&exit("root", "shell", json!(null)))?;
        assert!(!tracker.is_quiescent("root"));
        tracker.observe(&exit("root", "shell", json!(0)))?;
        assert!(tracker.is_quiescent("root"));
        Ok(())
    }
    #[test]
    fn terminal_output_without_start_and_synthetic_failure_cannot_prove_exit()
    -> Result<(), TestError> {
        let mut tracker = ActivityTracker::new("root", true)?;
        tracker.observe(&update(
            "root",
            json!({"sessionUpdate":"terminal_output_chunk","terminalId":"shell","data":"b3V0cHV0"}),
        ))?;
        tracker.observe(&exit("root", "shell", json!(null)))?;
        tracker.observe(&update(
            "root",
            json!({"sessionUpdate":"tool_call_update","toolCallId":"shell","status":"failed"}),
        ))?;
        assert!(!tracker.is_quiescent("root"));
        tracker.observe(&exit("root", "shell", json!(1)))?;
        assert!(tracker.is_quiescent("root"));
        // A following failed tool patch cannot erase the physical exit proof.
        tracker.observe(&update(
            "root",
            json!({"sessionUpdate":"tool_call_update","toolCallId":"shell","status":"failed"}),
        ))?;
        assert!(tracker.is_quiescent("root"));
        Ok(())
    }
    #[test]
    fn nonterminal_first_upsert_and_synthetic_failure_stay_owned() -> Result<(), TestError> {
        let mut tracker = ActivityTracker::new("root", true)?;
        tracker.observe(&running("root"))?;
        tracker.observe(&call("root", "read"))?;
        tracker.observe(&end("root"))?;
        tracker.observe(&update(
            "root",
            json!({"sessionUpdate":"tool_call_update","toolCallId":"read","status":"failed"}),
        ))?;
        assert!(!tracker.is_quiescent("root"));
        tracker.observe(&update(
            "root",
            json!({"sessionUpdate":"tool_call_update","toolCallId":"read","status":"completed"}),
        ))?;
        assert!(tracker.is_quiescent("root"));
        Ok(())
    }
    #[test]
    fn child_requires_live_success_and_retains_its_independent_terminal() -> Result<(), TestError> {
        let mut tracker = ActivityTracker::new("root", true)?;
        tracker.observe(&update(
            "root",
            json!({"sessionUpdate":"subagent_update","sessionId":"child"}),
        ))?;
        tracker.observe(&update(
            "child",
            json!({"sessionUpdate":"state_update","state":"idle"}),
        ))?;
        assert!(!tracker.is_quiescent("root"));
        tracker.observe(&running("child"))?;
        tracker.observe(&terminal("child", "shell"))?;
        tracker.observe(&call("child", "shell"))?;
        tracker.observe(&update(
            "child",
            json!({"sessionUpdate":"state_update","state":"idle","stopReason":"cancelled"}),
        ))?;
        assert!(!tracker.is_quiescent("root"));
        tracker.observe(&running("child"))?;
        tracker.observe(&end("child"))?;
        assert!(!tracker.is_quiescent("root"));
        tracker.observe(&exit("child", "shell", json!(0)))?;
        assert!(tracker.is_quiescent("root"));
        Ok(())
    }
    #[test]
    fn restored_or_unknown_lineage_never_becomes_empty_from_idle() -> Result<(), TestError> {
        let mut restored = ActivityTracker::new("root", false)?;
        restored.observe(&running("root"))?;
        restored.observe(&end("root"))?;
        assert!(!restored.is_quiescent("root"));
        let mut tracker = ActivityTracker::new("root", true)?;
        tracker.observe(&running("orphan"))?;
        tracker.observe(&end("orphan"))?;
        assert!(!tracker.is_quiescent("root"));
        Ok(())
    }
    #[test]
    fn binding_after_live_ingress_preserves_started_work() -> Result<(), TestError> {
        let mut tracker = ActivityTracker::default();
        tracker.observe(&running("root"))?;
        tracker.observe(&terminal("root", "shell"))?;
        tracker.bind("root", true)?;
        assert!(!tracker.is_quiescent("root"));
        assert!(tracker.bind("other", true).is_err());
        assert!(!tracker.is_quiescent("root"));
        Ok(())
    }
    #[test]
    fn malformed_work_and_overflow_never_evict_and_completed_work_is_collected()
    -> Result<(), TestError> {
        let mut tracker = ActivityTracker::new("root", true)?;
        for index in 0..MAX_CALLS {
            tracker.observe(&call("root", &format!("tool-{index}")))?;
        }
        assert!(tracker.observe(&call("root", "overflow")).is_err());
        assert_eq!(tracker.calls.len(), MAX_CALLS);
        assert!(!tracker.is_quiescent("root"));
        let mut tracker = ActivityTracker::new("root", true)?;
        for index in 0..MAX_CALLS {
            tracker.observe(&call("root", &format!("tool-{index}")))?;
            tracker.observe(&update("root",json!({"sessionUpdate":"tool_call_update","toolCallId":format!("tool-{index}"),"status":"completed"})))?;
        }
        tracker.observe(&running("root"))?;
        assert!(tracker.calls.is_empty());
        assert!(
            tracker
                .observe(&update("root", json!({"sessionUpdate":"terminal_update"})))
                .is_err()
        );
        tracker.observe(&end("root"))?;
        assert!(!tracker.is_quiescent("root"));
        assert_eq!(format!("{tracker:?}"), "ActivityTracker([REDACTED])");
        Ok(())
    }
}
