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
    pub(super) tool: String,
    description: Option<String>,
    responder: Option<oneshot::Sender<Result<Value, i32>>>,
}

#[derive(Clone, Copy)]
pub(super) enum Choice {
    ApproveOnce,
    Decline,
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
        Ok(Self::decode(&target, active_turn, &request)
            .map(|details| Self::prepare(target, position, active_turn, details)))
    }

    pub(super) fn decode(
        target: &super::super::NativeTextTarget,
        active_turn: &str,
        request: &Value,
    ) -> Option<(String, Option<String>)> {
        if request["method"] != "mcpServer/elicitation/request" || request.get("id").is_none() {
            return None;
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
            .filter(|message| {
                message.len() <= 8192
                    && !message.trim().is_empty()
                    && !message
                        .chars()
                        .any(|ch| ch.is_control() && ch != '\n' && ch != '\t')
            })
            .map(str::to_owned);
        Some((tool, description))
    }

    pub(super) fn prepare(
        target: super::super::NativeTextTarget,
        position: SourcePosition,
        active_turn: &str,
        (tool, description): (String, Option<String>),
    ) -> (Self, super::wire::ServerReply) {
        let (responder, reply) = oneshot::channel();
        (
            Self {
                target,
                position,
                turn: active_turn.to_owned(),
                tool,
                description,
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
        schema::RequestPermissionRequest::new(
            root,
            format!("Allow {}?", self.tool),
            vec![
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
        )
        .description(self.description.clone())
    }

    pub(super) fn answer(
        &mut self,
        target: &super::super::NativeTextTarget,
        position: &SourcePosition,
        active_turn: &str,
        choice: Choice,
    ) -> Result<(), Error> {
        if target != &self.target || position != &self.position || active_turn != self.turn {
            return Err(Error::NotAdmitted);
        }
        let reply = match choice {
            Choice::ApproveOnce => json!({"action": "accept", "content": {}}),
            Choice::Decline => json!({"action": "decline", "content": null}),
        };
        self.responder
            .take()
            .ok_or(Error::NotAdmitted)?
            .send(Ok(reply))
            .map_err(|_| Error::NotAdmitted)
    }
}
