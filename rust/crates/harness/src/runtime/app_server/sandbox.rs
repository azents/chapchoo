//! Scoped Codex permission replies. An answer selects a scope, not a profile.
use crate::runtime::schema;
use chapchoo_domain::SandboxPermissionProfile;
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Request {
    thread_id: String,
    turn_id: String,
    item_id: String,
    environment_id: Option<String>,
    started_at_ms: i64,
    cwd: String,
    reason: Option<String>,
    permissions: SandboxPermissionProfile,
}

pub(super) struct Proposal(SandboxPermissionProfile);

impl Proposal {
    pub(super) fn parse(root: &str, turn: &str, params: Value) -> Option<(Self, String)> {
        let request: Request = serde_json::from_value(params).ok()?;
        if request.thread_id != root
            || request.turn_id != turn
            || !super::valid_key(turn)
            || !super::valid_key(&request.item_id)
            || request.started_at_ms < 0
        {
            return None;
        }
        let description = request.permissions.consent_description(
            &request.cwd,
            request.environment_id.as_deref(),
            request.reason.as_deref(),
        )?;
        Some((Self(request.permissions), description))
    }

    pub(super) fn options() -> Vec<schema::PermissionOption> {
        use schema::PermissionOptionKind::{AllowAlways, AllowOnce, RejectOnce};
        [
            ("grant-turn", "Grant for this turn", AllowOnce),
            (
                "grant-turn-strict",
                "Grant for this turn with strict review",
                AllowOnce,
            ),
            ("grant-session", "Grant for this session", AllowAlways),
            (
                "deny-permissions",
                "Do not grant these permissions",
                RejectOnce,
            ),
        ]
        .into_iter()
        .map(|(id, label, kind)| schema::PermissionOption::new(id, label, kind))
        .collect()
    }

    pub(super) fn answer(&self, choice: super::tools::Choice<'_>) -> Option<Value> {
        use super::tools::Choice::{Cancelled, Selected};
        let (scope, strict_review) = match choice {
            Selected("grant-turn") => ("turn", false),
            Selected("grant-turn-strict") => ("turn", true),
            Selected("grant-session") => ("session", false),
            Selected("deny-permissions") | Cancelled => {
                return Some(json!({"permissions":{},"scope":"turn","strictAutoReview":false}));
            }
            _ => return None,
        };
        Some(json!({"permissions": self.0, "scope":scope, "strictAutoReview":strict_review}))
    }
}
