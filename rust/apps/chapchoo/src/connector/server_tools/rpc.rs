use super::*;
use chapchoo_protocol::server_tools::{self as wire, server_tools_client::ServerToolsClient};
use rmcp::{
    ErrorData,
    model::{CallToolResult, ListToolsResult},
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Request;

pub(super) fn request<T>(value: T, route: &Route) -> Result<Request<T>, AppError> {
    let mut request = Request::new(value);
    request.metadata_mut().insert(
        "authorization",
        format!("Bearer {}", route.key.expose()).parse()?,
    );
    Ok(request)
}
pub(super) fn scope(route: &Route, recipient: &Recipient) -> wire::CallerScope {
    wire::CallerScope {
        source: Some(route.source.clone()),
        agent_id: recipient.agent_id.clone(),
        conversation_id: recipient.conversation_id.clone(),
    }
}
pub(super) fn client(route: &Route) -> ServerToolsClient<Channel> {
    ServerToolsClient::new(route.channel.clone())
        .max_decoding_message_size(64 * 1024)
        .max_encoding_message_size(64 * 1024)
}
pub(super) async fn catalog(
    route: &CurrentRoute,
    recipient: &Recipient,
) -> Result<ListToolsResult, AppError> {
    let route = route.snapshot()?;
    let mut query = request(
        wire::CatalogRequest {
            scope: Some(scope(&route, recipient)),
        },
        &route,
    )?;
    query.set_timeout(std::time::Duration::from_secs(10));
    let value = client(&route)
        .catalog(query)
        .await?
        .into_inner()
        .catalog_json;
    if value.len() > chapchoo_service::MAX_TOOL_PAYLOAD {
        return Err("Tool catalog is too large".into());
    }
    Ok(serde_json::from_slice(&value)?)
}
pub(super) fn unavailable() -> ErrorData {
    ErrorData::internal_error(
        "Tool response transport unavailable; an effect may have occurred",
        None,
    )
}
pub(super) fn representation() -> ErrorData {
    ErrorData::internal_error(
        "Tool result cannot be represented; an effect may have occurred",
        None,
    )
}
pub(super) fn opening_error(status: tonic::Status) -> ErrorData {
    let mut values = status
        .metadata()
        .get_all(wire::NOT_EXECUTED_METADATA)
        .iter();
    let reason = values
        .next()
        .and_then(|value| value.to_str().ok())
        .and_then(wire::Rejection::from_str_name)
        .filter(|reason| *reason != wire::Rejection::Unspecified);
    if let Some(reason) = reason
        && values.next().is_none()
    {
        return rejection(reason as i32);
    }
    // A status code alone never proves that the peer did not accept Submit.
    tracing::warn!(code=?status.code(), "Server tool invocation stream could not be established");
    unavailable()
}
struct EffectBoundary<'a> {
    state: &'a State,
    finished: bool,
}
impl Drop for EffectBoundary<'_> {
    fn drop(&mut self) {
        if !self.finished {
            self.state
                .uncertain_effects
                .store(true, std::sync::atomic::Ordering::Release);
        }
    }
}
pub(super) async fn invoke(
    state: &State,
    name: String,
    arguments: Vec<u8>,
    cancellation: Arc<handler::Cancel>,
) -> Result<CallToolResult, ErrorData> {
    let route = state.route.snapshot().map_err(|_| {
        ErrorData::internal_error("No current tool route; invocation was not sent", None)
    })?;
    let (sender, receiver) = mpsc::channel(2);
    sender
        .try_send(wire::InvocationControl {
            control: Some(wire::invocation_control::Control::Submit(wire::Submit {
                scope: Some(scope(&route, &state.recipient)),
                name,
                arguments_json: arguments,
            })),
        })
        .map_err(|_| ErrorData::internal_error("Tool Submit was not sent", None))?;
    let open = request(ReceiverStream::new(receiver), &route).map_err(|_| unavailable())?;
    let mut effect = EffectBoundary {
        state,
        finished: false,
    };
    let mut client = client(&route);
    // This dedicated stream has no Control unary helper/ten-second deadline.
    let opening = client.invoke(open);
    tokio::pin!(opening);
    let mut cancelled = false;
    let response = tokio::select! {
        response = &mut opening => response,
        _ = cancellation.cancelled() => {
            cancelled = true;
            let _ = sender.try_send(cancel());
            opening.await
        }
    };
    let mut stream = match response {
        Ok(response) => response.into_inner(),
        Err(status) => {
            let mut markers = status
                .metadata()
                .get_all(wire::NOT_EXECUTED_METADATA)
                .iter();
            effect.finished = markers
                .next()
                .and_then(|value| value.to_str().ok())
                .and_then(wire::Rejection::from_str_name)
                .is_some_and(|reason| reason != wire::Rejection::Unspecified)
                && markers.next().is_none();
            return Err(opening_error(status));
        }
    };
    let mut waiting = false;
    let mut started = false;
    loop {
        let frame = tokio::select! {
            frame = stream.message() => frame.map_err(|_| unavailable())?,
            _ = cancellation.cancelled(), if !cancelled => {
                cancelled = true;
                let _ = sender.try_send(cancel());
                continue;
            }
        }
        .ok_or_else(unavailable)?;
        use wire::invocation_observation::Observation;
        match frame.observation {
            Some(Observation::Waiting(())) if !waiting && !started => waiting = true,
            Some(Observation::Started(())) if !started => started = true,
            Some(Observation::Final(wire::Final {
                outcome: Some(outcome),
            })) => {
                use wire::r#final::Outcome;
                effect.finished = matches!(
                    &outcome,
                    Outcome::ResultJson(_) | Outcome::UncertainRepresentation(())
                ) || matches!(&outcome, Outcome::NotExecuted(_)) && !started;
                return match outcome {
                    Outcome::ResultJson(bytes) => {
                        if bytes.len() > chapchoo_service::MAX_TOOL_PAYLOAD {
                            return Err(representation());
                        }
                        let result: CallToolResult =
                            serde_json::from_slice(&bytes).map_err(|_| representation())?;
                        if result
                            .structured_content
                            .as_ref()
                            .is_some_and(|value| !value.is_object())
                            || result.result_type.is_some()
                        {
                            return Err(representation());
                        }
                        Ok(result)
                    }
                    Outcome::NotExecuted(reason) if !started => Err(rejection(reason)),
                    Outcome::UncertainRepresentation(()) => Err(representation()),
                    Outcome::Uncertain(()) => Err(ErrorData::internal_error(
                        "Tool effect or protected result is unknown",
                        None,
                    )),
                    _ => Err(unavailable()),
                };
            }
            _ => return Err(unavailable()),
        }
    }
}
fn cancel() -> wire::InvocationControl {
    wire::InvocationControl {
        control: Some(wire::invocation_control::Control::Cancel(())),
    }
}
pub(super) fn rejection(reason: i32) -> ErrorData {
    let message = match wire::Rejection::try_from(reason) {
        Ok(wire::Rejection::UnknownTool) => {
            return ErrorData::invalid_params("Unknown tool; invocation was not executed", None);
        }
        Ok(wire::Rejection::InvalidEnvelope) => {
            return ErrorData::invalid_request(
                "Invalid tool envelope; invocation was not executed",
                None,
            );
        }
        Ok(wire::Rejection::Capacity) => "Tool capacity exhausted; invocation was not executed",
        Ok(wire::Rejection::Cancelled) => "Tool cancelled before execution",
        Ok(wire::Rejection::QueueExpired | wire::Rejection::ResponseExpired) => {
            "Tool wait expired before execution"
        }
        Ok(wire::Rejection::Authority) => "Tool authority denied; invocation was not executed",
        Ok(wire::Rejection::Closing) => "Tool service is closing; invocation was not executed",
        Ok(wire::Rejection::UpstreamUnavailable) => {
            "External tool server unavailable; invocation was not executed. Check that the server is reachable."
        }
        Ok(wire::Rejection::UpstreamAuthentication) => {
            "External tool server rejected authentication; invocation was not executed. Check or replace the Bearer token in the Agent's MCP settings."
        }
        Ok(wire::Rejection::UpstreamDestination) => {
            "External tool server destination was denied; invocation was not executed. Check the public HTTPS endpoint in the Agent's MCP settings."
        }
        Ok(wire::Rejection::UpstreamProtocol) => {
            "External tool server did not negotiate the supported MCP revision; invocation was not executed. Check the server's MCP configuration."
        }
        Ok(wire::Rejection::UpstreamInvalidResponse) => {
            "External tool server sent an invalid setup response; invocation was not executed. Check the server before trying again."
        }
        Ok(wire::Rejection::UpstreamTimeout) => {
            "External tool server timed out during setup; invocation was not executed. Check the server before trying again."
        }
        Ok(wire::Rejection::UpstreamToolsUnavailable) => {
            "External tool server did not expose tools; invocation was not executed. Check which tools the server makes available."
        }
        _ => "Invalid tool invocation; it was not executed",
    };
    ErrorData::internal_error(message, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn external_authentication_rejection_contains_only_fixed_recovery_guidance() {
        let reason = rejection(wire::Rejection::UpstreamAuthentication.into());
        assert!(reason.message.contains("was not executed"));
        assert!(reason.message.contains("replace the Bearer token"));
        assert!(reason.data.is_none());
    }

    #[test]
    fn only_an_explicit_opening_marker_establishes_nonexecution() {
        let mut marked = tonic::Status::resource_exhausted("Private detail");
        marked.metadata_mut().insert(
            wire::NOT_EXECUTED_METADATA,
            tonic::metadata::MetadataValue::from_static(wire::Rejection::Capacity.as_str_name()),
        );
        assert!(opening_error(marked).message.contains("was not executed"));
        let unmarked = tonic::Status::resource_exhausted("Private detail");
        let uncertain = opening_error(unmarked);
        assert!(uncertain.message.contains("may have occurred"));
        assert!(!uncertain.message.contains("Private detail"));
        let mut malformed = tonic::Status::unavailable("Private detail");
        malformed.metadata_mut().insert(
            wire::NOT_EXECUTED_METADATA,
            tonic::metadata::MetadataValue::from_static("unknown-reason"),
        );
        assert!(
            opening_error(malformed)
                .message
                .contains("may have occurred")
        );
    }
}
