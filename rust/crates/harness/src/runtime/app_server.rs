//! Versioned operations for the official Codex App Server.
pub(super) mod commands;
pub(super) mod events;
mod forms;
mod live;
pub(super) use live::{Connection, LiveThread};
mod questions;
mod sandbox;
mod tools;
mod wire;

use crate::{Error, ProtocolStage};
use chapchoo_domain::{
    ConversationModelCapabilities, ConversationSpeed, ConversationText, NativeConversationId,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{collections::BTreeSet, path::Path};

use wire::{CallFailure, Client, Reply};

struct Thread<'a> {
    link: &'a Client,
    id: NativeConversationId,
    model: Option<String>,
}

async fn call(
    link: &Client,
    stage: ProtocolStage,
    method: &str,
    params: impl Serialize,
) -> Result<Value, Error> {
    link.enqueue(method, params)?
        .receive()
        .await
        .map_err(|error| match error {
            CallFailure::Rejected(code) => Error::ProtocolRejected { stage, code },
            CallFailure::Uncertain => Error::Protocol,
        })
}

async fn initialize(link: &Client, initial_key: Option<&str>) -> Result<(), Error> {
    call(
        link,
        ProtocolStage::Initialize,
        "initialize",
        json!({"clientInfo": {"name": "chapchoo", "version": env!("CARGO_PKG_VERSION")}}),
    )
    .await?;
    link.initialized()?;
    if let Some(api_key) = initial_key {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct KeyLogin<'a> {
            #[serde(rename = "type")]
            kind: &'static str,
            api_key: &'a str,
        }
        call(
            link,
            ProtocolStage::Authentication,
            "account/login/start",
            KeyLogin {
                kind: "apiKey",
                api_key,
            },
        )
        .await?;
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ModelPage {
    data: Vec<Model>,
    next_cursor: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Model {
    model: String,
    display_name: String,
    supported_reasoning_efforts: Vec<Effort>,
    default_reasoning_effort: String,
    service_tiers: Vec<Tier>,
    default_service_tier: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Effort {
    reasoning_effort: String,
}

#[derive(Deserialize)]
struct Tier {
    id: String,
}

#[derive(Debug, PartialEq, Eq)]
struct Candidate {
    id: String,
    label: String,
    efforts: Vec<String>,
    default_effort: Option<String>,
    tiers: Vec<String>,
    default_tier: Option<String>,
}

impl Candidate {
    fn capabilities(&self) -> ConversationModelCapabilities {
        // The per-turn baseline is supported independently of the paid tiers
        // listed by the applied connection.
        let mut speeds = vec![ConversationSpeed::Standard];
        if self.tiers.iter().any(|tier| tier == "priority") {
            speeds.push(ConversationSpeed::Fast);
        }
        if self.tiers.iter().any(|tier| tier == "ultrafast") {
            speeds.push(ConversationSpeed::Ultrafast);
        }
        ConversationModelCapabilities {
            efforts: self.efforts.clone(),
            speeds,
        }
    }
}

fn service_tier(speed: ConversationSpeed) -> &'static str {
    match speed {
        ConversationSpeed::Standard => "default",
        ConversationSpeed::Fast => "priority",
        ConversationSpeed::Ultrafast => "ultrafast",
    }
}

#[derive(Default)]
struct TurnOptions<'a> {
    model: Option<&'a str>,
    effort: Option<&'a str>,
    tier: Option<&'a str>,
}

const MAX_CANDIDATES: usize = 128;
const MAX_OPTIONS: usize = 32;

async fn model_catalog(link: &Client) -> Result<Vec<Candidate>, Error> {
    let mut candidates = Vec::new();
    let mut cursors = BTreeSet::new();
    let mut ids = BTreeSet::new();
    let mut cursor: Option<String> = None;
    loop {
        let page: ModelPage = serde_json::from_value(
            call(
                link,
                ProtocolStage::Session,
                "model/list",
                json!({"limit": MAX_CANDIDATES, "cursor": cursor, "includeHidden": false}),
            )
            .await?,
        )
        .map_err(|_| Error::Protocol)?;
        if candidates.len() + page.data.len() > MAX_CANDIDATES {
            return Err(Error::Protocol);
        }
        for model in page.data {
            let valid_label = !model.display_name.trim().is_empty()
                && model.display_name.len() <= 256
                && !model.display_name.chars().any(char::is_control);
            if !valid_key(&model.model) || !valid_label || !ids.insert(model.model.clone()) {
                return Err(Error::Protocol);
            }
            let efforts = validate_options(
                model
                    .supported_reasoning_efforts
                    .into_iter()
                    .map(|effort| effort.reasoning_effort),
            )?;
            let tiers = validate_options(model.service_tiers.into_iter().map(|tier| tier.id))?;
            let default_effort = efforts
                .contains(&model.default_reasoning_effort)
                .then_some(model.default_reasoning_effort);
            let default_tier = model
                .default_service_tier
                .filter(|tier| tiers.contains(tier));
            candidates.push(Candidate {
                id: model.model,
                label: model.display_name,
                efforts,
                default_effort,
                tiers,
                default_tier,
            });
        }
        let Some(next) = page.next_cursor else {
            return Ok(candidates);
        };
        if next.is_empty()
            || next.len() > 4096
            || cursors.len() == MAX_OPTIONS
            || !cursors.insert(next.clone())
        {
            return Err(Error::Protocol);
        }
        cursor = Some(next);
    }
}

fn validate_options(values: impl ExactSizeIterator<Item = String>) -> Result<Vec<String>, Error> {
    if values.len() > MAX_OPTIONS {
        return Err(Error::Protocol);
    }
    let values: Vec<_> = values.collect();
    let mut unique = BTreeSet::new();
    if values
        .iter()
        .any(|key| !valid_key(key) || !unique.insert(key))
    {
        return Err(Error::Protocol);
    }
    Ok(values)
}

fn valid_key(key: &str) -> bool {
    !key.is_empty() && key.len() <= 256 && key.bytes().all(|byte| byte.is_ascii_graphic())
}

impl<'a> Thread<'a> {
    #[cfg(test)]
    async fn resume(
        link: &'a Client,
        id: &NativeConversationId,
        cwd: &Path,
    ) -> Result<Self, Error> {
        Self::open(link, cwd, Some(id), None).await
    }

    async fn open(
        link: &'a Client,
        cwd: &Path,
        existing: Option<&NativeConversationId>,
        mcp: Option<&crate::NativeMcpDeclaration>,
    ) -> Result<Self, Error> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Open<'a> {
            cwd: &'a Path,
            #[serde(skip_serializing_if = "Option::is_none")]
            thread_id: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            exclude_turns: Option<bool>,
            #[serde(skip_serializing_if = "Option::is_none")]
            approval_policy: Option<&'static str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            approvals_reviewer: Option<&'static str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            config: Option<tools::Config<'a>>,
        }
        let prepared = mcp.map(tools::Prepared::new);
        let response = call(
            link,
            ProtocolStage::Session,
            if existing.is_some() {
                "thread/resume"
            } else {
                "thread/start"
            },
            Open {
                cwd,
                thread_id: existing.map(NativeConversationId::as_str),
                exclude_turns: existing.map(|_| true),
                approval_policy: mcp.map(|_| "on-request"),
                approvals_reviewer: mcp.map(|_| "user"),
                config: prepared.as_ref().map(tools::Prepared::config),
            },
        )
        .await?;
        let id = response["thread"]["id"].as_str().ok_or(Error::Protocol)?;
        if existing.is_some_and(|existing| existing.as_str() != id) {
            return Err(Error::CaptureConflict);
        }
        Ok(Self {
            link,
            id: NativeConversationId::parse(id.to_owned()).map_err(|_| Error::Protocol)?,
            model: response["model"].as_str().map(str::to_owned),
        })
    }

    #[cfg(test)]
    fn submit(
        &self,
        insertions: &events::Insertions,
        input: &ConversationText,
        service_tier_for_turn: Option<&str>,
    ) -> Result<impl Future<Output = Result<events::Inserted, Error>> + use<>, Error> {
        self.submit_with_options(
            insertions,
            input,
            TurnOptions {
                tier: service_tier_for_turn,
                ..TurnOptions::default()
            },
        )
    }

    fn submit_with_options(
        &self,
        insertions: &events::Insertions,
        input: &ConversationText,
        options: TurnOptions<'_>,
    ) -> Result<impl Future<Output = Result<events::Inserted, Error>> + use<>, Error> {
        let (reply, receipt) =
            insertions.publish(None, |marker| self.enqueue_start(marker, input, options))?;
        Ok(attributed_insertion(reply, receipt, "/turn/id"))
    }

    fn steer(
        &self,
        insertions: &events::Insertions,
        expected_turn: &str,
        input: &ConversationText,
    ) -> Result<impl Future<Output = Result<events::Inserted, Error>> + use<>, Error> {
        if !valid_key(expected_turn) {
            return Err(Error::NotAdmitted);
        }
        let (reply, receipt) = insertions.publish(Some(expected_turn), |marker| {
            self.link.enqueue(
                "turn/steer",
                json!({
                    "threadId": self.id.as_str(),
                    "expectedTurnId": expected_turn,
                    "clientUserMessageId": marker,
                    "input": [{"type": "text", "text": input.as_str()}],
                }),
            )
        })?;
        Ok(attributed_insertion(reply, receipt, "/turnId"))
    }

    fn interrupt(&self, exact_turn: &str) -> Result<Reply, Error> {
        if !valid_key(exact_turn) {
            return Err(Error::NotAdmitted);
        }
        self.link.enqueue(
            "turn/interrupt",
            json!({"threadId": self.id.as_str(), "turnId": exact_turn}),
        )
    }

    fn enqueue_start(
        &self,
        client_message_id: &str,
        input: &ConversationText,
        options: TurnOptions<'_>,
    ) -> Result<Reply, Error> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Start<'a> {
            thread_id: &'a str,
            client_user_message_id: &'a str,
            input: [Value; 1],
            #[serde(skip_serializing_if = "Option::is_none")]
            model: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            effort: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            service_tier_for_turn: Option<&'a str>,
        }
        self.link.enqueue(
            "turn/start",
            Start {
                thread_id: self.id.as_str(),
                client_user_message_id: client_message_id,
                input: [json!({"type": "text", "text": input.as_str()})],
                model: options.model,
                effort: options.effort,
                service_tier_for_turn: options.tier,
            },
        )
    }
}

async fn attributed_insertion(
    reply: Reply,
    receipt: tokio::sync::oneshot::Receiver<events::Inserted>,
    turn_pointer: &'static str,
) -> Result<events::Inserted, Error> {
    // The notification may arrive on either side of the RPC reply. The reply
    // alone doesn't prove insertion or a new turn.
    let response = reply
        .receive()
        .await
        .map_err(|_| Error::DeliveryUncertain)?;
    let turn = response
        .pointer(turn_pointer)
        .and_then(Value::as_str)
        .ok_or(Error::DeliveryUncertain)?;
    let inserted = receipt.await.map_err(|_| Error::DeliveryUncertain)?;
    if inserted.turn != turn {
        return Err(Error::DeliveryUncertain);
    }
    Ok(inserted)
}

#[cfg(test)]
mod tests;
