use super::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, agent_client_protocol::JsonRpcRequest)]
#[request(method="_session/steering",response=SteeringResponse)]
#[serde(rename_all = "camelCase")]
struct Steering {
    session_id: schema::SessionId,
    prompt: Vec<schema::ContentBlock>,
    #[serde(rename = "_meta")]
    meta: SteeringMeta,
}
impl fmt::Debug for Steering {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Steering([REDACTED])")
    }
}
#[derive(Clone, Serialize, Deserialize)]
struct SteeringMeta {
    steering: IdlePolicy,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct IdlePolicy {
    idle_behavior: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, agent_client_protocol::JsonRpcResponse)]
#[serde(tag = "outcome", deny_unknown_fields)]
enum SteeringResponse {
    #[serde(rename = "injected")]
    Injected,
    #[serde(rename = "startedNewTurn")]
    StartedNewTurn,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "promptRequired")]
    PromptRequired { reason: NoRunningTurn },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
enum NoRunningTurn {
    #[serde(rename = "noRunningTurn")]
    NoRunningTurn,
}

/// None means the producer explicitly did not consume this prompt.
pub(super) async fn steer(
    session: &agent_client_protocol::v2::ActiveSession<Agent>,
    text: &ConversationText,
    route: Option<&ContinuationRoute>,
) -> Result<Option<NativePromptAcceptance>, Error> {
    let request = Steering {
        session_id: session.session_id().clone(),
        prompt: vec![schema::ContentBlock::from(text.as_str())],
        meta: SteeringMeta {
            steering: IdlePolicy {
                idle_behavior: "promptRequired".into(),
            },
        },
    };
    let publish = || Ok(session.connection().send_request(request));
    let pending = match route {
        Some(route) => route.publish(publish)?,
        None => publish()?,
    };
    let response = pending.block_task().await.map_err(|_| Error::Protocol)?;
    match response {
        SteeringResponse::Injected => Ok(Some(NativePromptAcceptance::Steered)),
        SteeringResponse::Failed => Ok(Some(NativePromptAcceptance::Failed)),
        SteeringResponse::StartedNewTurn => Err(Error::Protocol),
        SteeringResponse::PromptRequired {
            reason: NoRunningTurn::NoRunningTurn,
        } => Ok(None),
    }
}

#[cfg(test)]
pub(super) async fn submit(
    session: &agent_client_protocol::v2::ActiveSession<Agent>,
    text: &ConversationText,
    observations: Option<&Observations>,
) -> Result<NativePromptAcceptance, Error> {
    if let Some(accepted) = steer(session, text, None).await? {
        return Ok(accepted);
    }
    let publish = || session.send_prompt(text.as_str());
    let pending = match observations {
        Some(observations) => observations.publish_ordinary_prompt(publish)?,
        None => publish(),
    };
    let response = pending.block_task().await.map_err(|_| Error::Protocol)?;
    Ok(NativePromptAcceptance::Inserted {
        native_message_id: response.message_id.to_string(),
    })
}
