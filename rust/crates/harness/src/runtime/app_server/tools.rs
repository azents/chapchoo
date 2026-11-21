//! Launch-local HTTP tool declaration. The credential is only serialized into
//! the private transport; neither argv nor a native config file carries it.
use crate::{Error, NativeMcpDeclaration, runtime::schema};
use chapchoo_domain::SourcePosition;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::sync::oneshot;
use zeroize::Zeroizing;

pub(super) struct Prepared<'a> {
    url: &'a str,
    authorization: Zeroizing<String>,
}

impl<'a> Prepared<'a> {
    pub(super) fn new(declaration: &'a NativeMcpDeclaration) -> Self {
        let (url, key) = declaration.http_credentials();
        Self {
            url,
            authorization: Zeroizing::new(format!("Bearer {}", key.expose())),
        }
    }

    pub(super) fn config(&self) -> Config<'_> {
        Config {
            mcp_servers: Servers {
                chapchoo: Server {
                    url: self.url,
                    http_headers: Headers {
                        authorization: self.authorization.as_str(),
                    },
                    required: true,
                    default_tools_approval_mode: "prompt",
                    omit_tools_from: ["code_mode", "deferred"],
                },
            },
            code_mode: CodeMode {
                direct_only_tool_namespaces: ["mcp__chapchoo"],
            },
            features: Features {
                code_mode_only_strict_3p_tools: false,
            },
        }
    }
}

#[derive(Serialize)]
pub(super) struct Config<'a> {
    mcp_servers: Servers<'a>,
    code_mode: CodeMode,
    features: Features,
}

#[derive(Serialize)]
struct Servers<'a> {
    chapchoo: Server<'a>,
}

#[derive(Serialize)]
struct Server<'a> {
    url: &'a str,
    http_headers: Headers<'a>,
    required: bool,
    default_tools_approval_mode: &'static str,
    omit_tools_from: [&'static str; 2],
}

#[derive(Serialize)]
struct Headers<'a> {
    #[serde(rename = "Authorization")]
    authorization: &'a str,
}

#[derive(Serialize)]
struct CodeMode {
    direct_only_tool_namespaces: [&'static str; 1],
}

#[derive(Serialize)]
struct Features {
    code_mode_only_strict_3p_tools: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Elicitation {
    thread_id: String,
    turn_id: Option<String>,
    server_name: String,
    mode: String,
    #[serde(rename = "_meta")]
    meta: Option<Metadata>,
    requested_schema: Option<Schema>,
}

#[derive(Deserialize)]
struct Metadata {
    codex_approval_kind: Option<String>,
    tool_name: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Schema {
    #[serde(rename = "type")]
    kind: String,
    properties: serde_json::Map<String, Value>,
}

pub(super) struct Approval {
    target: super::super::NativeTextTarget,
    position: SourcePosition,
    turn: String,
    details: ApprovalDetails,
    responder: Option<oneshot::Sender<Result<Value, i32>>>,
}

pub(super) struct ApprovalDetails {
    title: String,
    description: Option<String>,
    reply: ApprovalReply,
}

enum ApprovalReply {
    Elicitation,
    Item(Vec<ItemDecision>),
    Sandbox(super::sandbox::Proposal),
}

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
enum ItemDecision {
    Accept,
    AcceptForSession,
    Decline,
    Cancel,
}

impl ItemDecision {
    fn option(self) -> schema::PermissionOption {
        use schema::PermissionOptionKind::{AllowAlways, AllowOnce, RejectOnce};
        let (id, label, kind) = match self {
            Self::Accept => ("approve-once", "Allow once", AllowOnce),
            Self::AcceptForSession => {
                ("approve-for-session", "Allow for this session", AllowAlways)
            }
            Self::Decline => ("decline", "Decline and continue", RejectOnce),
            Self::Cancel => ("cancel", "Decline and stop the turn", RejectOnce),
        };
        schema::PermissionOption::new(id, label, kind)
    }
}

#[derive(Deserialize)]
pub(super) struct FileChange {
    path: String,
    kind: PatchKind,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum PatchKind {
    Add,
    Delete,
    Update { move_path: Option<String> },
}

pub(super) fn file_change_locations(item: &Value) -> Option<String> {
    let changes: Vec<FileChange> = serde_json::from_value(item["changes"].clone()).ok()?;
    if changes.is_empty() || changes.len() > 100 {
        return None;
    }
    let lines = changes
        .into_iter()
        .map(|FileChange { path, kind }| {
            if !visible(&path) || path.chars().any(char::is_control) {
                return None;
            }
            match kind {
                PatchKind::Add => Some(format!("Add {path}")),
                PatchKind::Delete => Some(format!("Delete {path}")),
                PatchKind::Update { move_path: None } => Some(format!("Update {path}")),
                PatchKind::Update {
                    move_path: Some(destination),
                } if visible(&destination) && !destination.chars().any(char::is_control) => {
                    Some(format!("Move {path} → {destination}"))
                }
                _ => None,
            }
        })
        .collect::<Option<Vec<_>>>()?
        .join("\n");
    visible(&lines).then_some(lines)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ItemApproval {
    thread_id: String,
    turn_id: String,
    item_id: String,
    started_at_ms: i64,
    reason: Option<String>,
    command: Option<String>,
    cwd: Option<String>,
    network_approval_context: Option<NetworkApproval>,
    available_decisions: Option<Vec<Value>>,
    additional_permissions: Option<Value>,
    grant_root: Option<String>,
}

#[derive(Deserialize)]
struct NetworkApproval {
    host: String,
    protocol: String,
}

pub(super) fn visible(text: &str) -> bool {
    text.len() <= 8192
        && !text.trim().is_empty()
        && !text
            .chars()
            .any(|ch| ch.is_control() && ch != '\n' && ch != '\t')
}

fn item_approval(
    target: &super::super::NativeTextTarget,
    turn: &str,
    params: Value,
    command: bool,
    file_locations: Option<&str>,
) -> Option<ApprovalDetails> {
    let item: ItemApproval = serde_json::from_value(params).ok()?;
    if item.thread_id != target.native_session.as_str()
        || item.turn_id != turn
        || !super::valid_key(turn)
        || !super::valid_key(&item.item_id)
        || item.started_at_ms < 0
    {
        return None;
    }
    let (title, mut lines, decisions) = if command {
        let is_network = item.network_approval_context.is_some();
        let context = match (item.command, item.cwd, item.network_approval_context) {
            (Some(command), Some(cwd), None) if visible(&command) && visible(&cwd) => {
                format!("Directory: {cwd}\nCommand: {command}")
            }
            (None, None, Some(network)) if visible(&network.host) && visible(&network.protocol) => {
                format!(
                    "Network destination: {} ({})",
                    network.host, network.protocol
                )
            }
            _ => return None,
        };
        let mut decisions = Vec::new();
        if let Some(offered) = item.available_decisions {
            for decision in offered {
                if let Ok(decision) = serde_json::from_value::<ItemDecision>(decision)
                    && !decisions.contains(&decision)
                {
                    decisions.push(decision);
                }
            }
        } else if !is_network && item.additional_permissions.is_some() {
            decisions.extend([ItemDecision::Accept, ItemDecision::Cancel]);
        } else {
            // Non-experimental connections omit the advertised decisions.
            decisions.extend([
                ItemDecision::Accept,
                ItemDecision::AcceptForSession,
                ItemDecision::Decline,
                ItemDecision::Cancel,
            ]);
        }
        if !decisions
            .iter()
            .any(|decision| matches!(decision, ItemDecision::Decline | ItemDecision::Cancel))
        {
            return None;
        }
        ("Allow command execution?", vec![context], decisions)
    } else {
        let scope = item
            .grant_root
            .filter(|root| visible(root))
            .map(|root| format!("Requested root: {root}"));
        (
            "Allow file changes?",
            std::iter::once(file_locations?.to_owned())
                .chain(scope)
                .collect(),
            vec![
                ItemDecision::Accept,
                ItemDecision::AcceptForSession,
                ItemDecision::Cancel,
            ],
        )
    };
    if let Some(reason) = item.reason.filter(|reason| visible(reason)) {
        lines.push(reason);
    }
    Some(ApprovalDetails {
        title: title.into(),
        description: (!lines.is_empty()).then(|| lines.join("\n")),
        reply: ApprovalReply::Item(decisions),
    })
}

#[derive(Clone, Copy)]
pub(super) enum Choice<'a> {
    Selected(&'a str),
    Cancelled,
}

impl Approval {
    #[cfg(test)]
    /// Decode only the pinned native tool-approval shape, never a plain
    /// elicitation, persistence request, or another server's form.
    pub(super) fn offer(
        target: super::super::NativeTextTarget,
        position: SourcePosition,
        active_turn: &str,
        request: Value,
    ) -> Result<Option<(Self, super::wire::ServerReply)>, Error> {
        Ok(Self::decode(&target, active_turn, &request, None)
            .map(|details| Self::prepare(target, position, active_turn, details)))
    }

    pub(super) fn decode(
        target: &super::super::NativeTextTarget,
        active_turn: &str,
        request: &Value,
        file_locations: Option<&str>,
    ) -> Option<ApprovalDetails> {
        request.get("id")?;
        match request["method"].as_str()? {
            "item/commandExecution/requestApproval" => {
                return item_approval(target, active_turn, request["params"].clone(), true, None);
            }
            "item/fileChange/requestApproval" => {
                return item_approval(
                    target,
                    active_turn,
                    request["params"].clone(),
                    false,
                    file_locations,
                );
            }
            "item/permissions/requestApproval" => {
                let (proposal, description) = super::sandbox::Proposal::parse(
                    target.native_session.as_str(),
                    active_turn,
                    request["params"].clone(),
                )?;
                return Some(ApprovalDetails {
                    title: "Grant sandbox permissions?".into(),
                    description: Some(description),
                    reply: ApprovalReply::Sandbox(proposal),
                });
            }
            "mcpServer/elicitation/request" => {}
            _ => return None,
        }
        let elicitation: Elicitation = serde_json::from_value(request["params"].clone()).ok()?;
        if !super::valid_key(active_turn)
            || elicitation.thread_id != target.native_session.as_str()
            || elicitation.turn_id.as_deref() != Some(active_turn)
            || elicitation.server_name != "chapchoo"
            || elicitation.mode != "form"
            || !elicitation
                .requested_schema
                .is_some_and(|schema| schema.kind == "object" && schema.properties.is_empty())
        {
            return None;
        }
        let tool = elicitation
            .meta
            .filter(|meta| meta.codex_approval_kind.as_deref() == Some("mcp_tool_call"))
            .and_then(|meta| meta.tool_name)
            .filter(|name| super::valid_key(name))?;
        let description = request["params"]["message"]
            .as_str()
            .filter(|message| visible(message))
            .map(str::to_owned);
        Some(ApprovalDetails {
            title: format!("Allow {tool}?"),
            description,
            reply: ApprovalReply::Elicitation,
        })
    }

    pub(super) fn prepare(
        target: super::super::NativeTextTarget,
        position: SourcePosition,
        active_turn: &str,
        details: ApprovalDetails,
    ) -> (Self, super::wire::ServerReply) {
        let (responder, reply) = oneshot::channel();
        (
            Self {
                target,
                position,
                turn: active_turn.to_owned(),
                details,
                responder: Some(responder),
            },
            super::wire::ServerReply::Deferred(reply),
        )
    }

    pub(super) fn cancelled(&self) -> bool {
        self.responder
            .as_ref()
            .is_none_or(oneshot::Sender::is_closed)
    }

    pub(super) fn question(&self, root: &str) -> schema::RequestPermissionRequest {
        let options = match &self.details.reply {
            ApprovalReply::Sandbox(_) => super::sandbox::Proposal::options(),
            ApprovalReply::Item(decisions) => {
                decisions.iter().map(|decision| decision.option()).collect()
            }
            ApprovalReply::Elicitation => vec![
                schema::PermissionOption::new(
                    "approve-once",
                    "Allow once",
                    schema::PermissionOptionKind::AllowOnce,
                ),
                schema::PermissionOption::new(
                    "decline",
                    "Decline",
                    schema::PermissionOptionKind::RejectOnce,
                ),
            ],
        };
        schema::RequestPermissionRequest::new(root, self.details.title.clone(), options)
            .description(self.details.description.clone())
    }

    pub(super) fn answer(
        &mut self,
        target: &super::super::NativeTextTarget,
        position: &SourcePosition,
        active_turn: &str,
        choice: Choice<'_>,
    ) -> Result<(), Error> {
        if target != &self.target || position != &self.position || active_turn != self.turn {
            return Err(Error::NotAdmitted);
        }
        let reply = match (&self.details.reply, choice) {
            (ApprovalReply::Sandbox(proposal), choice) => {
                proposal.answer(choice).ok_or(Error::NotAdmitted)?
            }
            (ApprovalReply::Elicitation, Choice::Selected("approve-once")) => {
                json!({"action": "accept", "content": {}})
            }
            (ApprovalReply::Elicitation, Choice::Selected("decline") | Choice::Cancelled) => {
                json!({"action": "decline", "content": null})
            }
            (ApprovalReply::Item(decisions), Choice::Selected(id)) => {
                let decision = decisions
                    .iter()
                    .find(|decision| decision.option().option_id.0.as_ref() == id)
                    .ok_or(Error::NotAdmitted)?;
                json!({"decision": decision})
            }
            (ApprovalReply::Item(decisions), Choice::Cancelled)
                if decisions.contains(&ItemDecision::Cancel) =>
            {
                json!({"decision": ItemDecision::Cancel})
            }
            _ => return Err(Error::NotAdmitted),
        };
        self.responder
            .take()
            .ok_or(Error::NotAdmitted)?
            .send(Ok(reply))
            .map_err(|_| Error::NotAdmitted)
    }
}
