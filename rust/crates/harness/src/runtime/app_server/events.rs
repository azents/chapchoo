//! Decode only evidence for the exact App Server root thread. Historical RPC
//! results do not enter this live notification lane.
use chapchoo_domain::NativeConversationId;
use serde_json::Value;

use super::valid_key;
use crate::Error;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Observation<'a> {
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

pub(super) fn root_observation<'a>(
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
        "turn/started" => Observation::Started {
            turn: required_key(&params["turn"]["id"])?,
        },
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
