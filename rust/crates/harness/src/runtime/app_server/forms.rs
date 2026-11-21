//! Translate scoped MCP form requests without forwarding their extension metadata.
use crate::runtime::{Error, requests, schema};
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::sync::oneshot;

pub(super) struct Form {
    pub(super) request: schema::CreateElicitationRequest,
    reply: oneshot::Sender<Result<Value, i32>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Elicitation {
    server_name: String,
    turn_id: String,
    mode: String,
    message: String,
    requested_schema: schema::ElicitationSchema,
    #[serde(rename = "_meta")]
    meta: Option<Meta>,
}

#[derive(Deserialize)]
struct Meta {
    codex_approval_kind: Option<String>,
}

impl Form {
    pub(super) fn offer(
        root: &str,
        turn: &str,
        params: Value,
    ) -> Option<(Self, super::wire::ServerReply)> {
        let native: Elicitation = serde_json::from_value(params).ok()?;
        if native.server_name != "chapchoo"
            || native.turn_id != turn
            || native.mode != "form"
            || native
                .meta
                .is_some_and(|meta| meta.codex_approval_kind.is_some())
            || native.message.is_empty()
            || native.message.len() > 8192
        {
            return None;
        }
        // An unmarked empty form cannot stand in for a tool approval.
        if native.requested_schema.properties.is_empty() {
            return None;
        }
        let request = schema::CreateElicitationRequest::new(
            schema::ElicitationFormMode::new(
                schema::ElicitationSessionScope::new(root),
                native.requested_schema,
            ),
            native.message,
        );
        if !requests::supported_form(&request) {
            return None;
        }
        let (reply, receive) = oneshot::channel();
        Some((
            Self { request, reply },
            super::wire::ServerReply::Deferred(receive),
        ))
    }

    pub(super) fn cancelled(&self) -> bool {
        self.reply.is_closed()
    }

    pub(super) fn prepare_reply(
        &self,
        response: schema::CreateElicitationResponse,
    ) -> Result<Value, Error> {
        if !requests::form_response(&self.request, &response) {
            return Err(Error::NotAdmitted);
        }
        match response.action {
            schema::ElicitationAction::Accept(answer) => {
                Ok(json!({"action":"accept","content":answer.content}))
            }
            schema::ElicitationAction::Decline => Ok(json!({"action":"decline","content":null})),
            schema::ElicitationAction::Cancel => Ok(json!({"action":"cancel","content":null})),
            _ => Err(Error::NotAdmitted),
        }
    }

    pub(super) fn answer(self, reply: Value) -> Result<(), Error> {
        self.reply.send(Ok(reply)).map_err(|_| Error::NotAdmitted)
    }
}
