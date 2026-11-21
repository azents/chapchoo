//! Decode only evidence for the exact App Server root thread. Historical RPC
//! results do not enter this live notification lane.
use chapchoo_domain::NativeConversationId;
use serde_json::Value;
use std::sync::Mutex;
use tokio::sync::oneshot;

use super::valid_key;
use crate::Error;

#[derive(Debug, PartialEq, Eq)]
pub(in crate::runtime) enum Observation<'a> {
    Started {
        turn: &'a str,
    },
    Terminal {
        turn: &'a str,
        normal: bool,
    },
    InputInserted {
        turn: &'a str,
        item: &'a str,
        operation: &'a str,
    },
}

pub(in crate::runtime) fn root_observation<'a>(
    root: &NativeConversationId,
    frame: &'a Value,
) -> Result<Option<Observation<'a>>, Error> {
    let Some(method) = frame["method"].as_str() else {
        return Ok(None);
    };
    if !matches!(method, "turn/started" | "turn/completed" | "item/started")
        || frame.get("id").is_some()
    {
        return Ok(None);
    }
    let params = &frame["params"];
    let thread = required_key(&params["threadId"])?;
    if thread != root.as_str() {
        return Ok(None);
    }
    let observation = match method {
        "turn/started" if params["turn"]["status"] == "inProgress" => Observation::Started {
            turn: required_key(&params["turn"]["id"])?,
        },
        "turn/started" => return Err(Error::Protocol),
        "turn/completed" => {
            let normal = match params["turn"]["status"].as_str() {
                Some("completed") => true,
                Some("failed" | "interrupted") => false,
                _ => return Err(Error::Protocol),
            };
            Observation::Terminal {
                turn: required_key(&params["turn"]["id"])?,
                normal,
            }
        }
        "item/started" => {
            let item = &params["item"];
            if item["type"] != "userMessage" || item["clientId"].is_null() {
                return Ok(None);
            }
            Observation::InputInserted {
                turn: required_key(&params["turnId"])?,
                item: required_key(&item["id"])?,
                operation: required_key(&item["clientId"])?,
            }
        }
        _ => return Ok(None),
    };
    Ok(Some(observation))
}

fn required_key(value: &Value) -> Result<&str, Error> {
    value
        .as_str()
        .filter(|key| valid_key(key))
        .ok_or(Error::Protocol)
}

/// One insertion can be in flight on the actor's serial submission lane.
/// Dropping the waiter does not discard this reservation or grant a retry.
pub(super) struct Insertions {
    root: NativeConversationId,
    pending: Mutex<Option<Pending>>,
}

struct Pending {
    marker: String,
    expected_turn: Option<String>,
    reply: oneshot::Sender<Inserted>,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Inserted {
    pub(super) turn: String,
    pub(super) item: String,
}

impl Insertions {
    pub(super) fn new(root: NativeConversationId) -> Self {
        Self {
            root,
            pending: Mutex::new(None),
        }
    }

    /// Reserve and enqueue together. Only a known pre-publication failure
    /// clears the reservation without a matching live event.
    pub(super) fn publish<T>(
        &self,
        expected_turn: Option<&str>,
        enqueue: impl FnOnce(&str) -> Result<T, Error>,
    ) -> Result<(T, oneshot::Receiver<Inserted>), Error> {
        let mut slot = self.pending.lock().map_err(|_| Error::Protocol)?;
        if slot.is_some() {
            return Err(Error::DeliveryUncertain);
        }
        let marker = super::super::observations::token("insertion")?;
        let (reply, receipt) = oneshot::channel();
        let posted = enqueue(&marker)?;
        *slot = Some(Pending {
            marker,
            expected_turn: expected_turn.map(str::to_owned),
            reply,
        });
        Ok((posted, receipt))
    }

    pub(super) fn observe(&self, frame: &Value) -> Result<(), Error> {
        let Some(Observation::InputInserted {
            turn,
            item,
            operation,
        }) = root_observation(&self.root, frame)?
        else {
            return Ok(());
        };
        let mut slot = self.pending.lock().map_err(|_| Error::Protocol)?;
        let Some(pending) = slot.as_ref().filter(|pending| pending.marker == operation) else {
            return Ok(());
        };
        if pending
            .expected_turn
            .as_deref()
            .is_some_and(|target| target != turn)
        {
            return Err(Error::Protocol);
        }
        if let Some(pending) = slot.take() {
            let _ = pending.reply.send(Inserted {
                turn: turn.to_owned(),
                item: item.to_owned(),
            });
        }
        Ok(())
    }
}
