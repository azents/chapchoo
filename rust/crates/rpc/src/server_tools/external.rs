use super::{ServerToolServices, ingress, not_executed, scope, status, wire};
use chapchoo_domain::{NativeConversationId, SnapshotId};
use chapchoo_service::{
    ConversationDeliveryTarget, ExternalCataloguePage, ExternalMcpCallOutcome,
    ExternalMcpRejection, McpAppliedScope, McpDiscoveryFailure, McpExecutionScope,
    McpPreparationFailure, McpPreparationScope, ServerToolScope,
};
use std::pin::Pin;
use tokio_stream::{Stream, StreamExt};
use tonic::{Request, Response, Status};

const CHUNK_BYTES: usize = 48 * 1024;
pub(super) type CatalogStream =
    Pin<Box<dyn Stream<Item = Result<wire::ExternalCatalogFrame, Status>> + Send>>;

struct Preparation {
    caller: ServerToolScope,
    execution: String,
    operation: String,
}

impl Preparation {
    fn decode(
        value: wire::ExternalPreparation,
        credential: chapchoo_crypto::SecretDigest,
    ) -> Result<Self, Status> {
        ingress::id(&value.execution_id)?;
        ingress::id(&value.operation_id)?;
        Ok(Self {
            caller: scope(value.scope, credential)?,
            execution: value.execution_id,
            operation: value.operation_id,
        })
    }

    fn scope(&self) -> McpPreparationScope<'_> {
        McpPreparationScope {
            execution: McpExecutionScope {
                agent_id: &self.caller.agent_id,
                conversation_id: &self.caller.conversation_id,
                execution_id: &self.execution,
            },
            operation_id: &self.operation,
        }
    }
}

pub(super) async fn prepare(
    services: &ServerToolServices,
    request: Request<wire::ExternalPreparation>,
) -> Result<Response<CatalogStream>, Status> {
    services.admit()?;
    let external = services
        .external
        .as_ref()
        .ok_or_else(|| Status::unimplemented("External tool preparation is unavailable"))?;
    let credential = ingress::actor(&request)?;
    let caller = Preparation::decode(request.into_inner(), credential)?;
    let permit = services
        .external_ingress
        .clone()
        .try_acquire_owned()
        .map_err(|_| Status::resource_exhausted("External tool ingress is full"))?;
    let page = external
        .prepare(&caller.caller.source(), &caller.scope())
        .await
        .map_err(status)?;
    let stream: CatalogStream = Box::pin(tokio_stream::iter(catalogue_frames(page)).map(
        move |frame| {
            let _permit = &permit;
            Ok(frame)
        },
    ));
    Ok(Response::new(stream))
}

pub(super) async fn confirm_empty(
    services: &ServerToolServices,
    request: Request<wire::ExternalEmptyReuse>,
) -> Result<Response<wire::ExternalEmptyObservation>, Status> {
    services.admit()?;
    let external = services
        .external
        .as_ref()
        .ok_or_else(|| Status::unimplemented("External tool preparation is unavailable"))?;
    let credential = ingress::actor(&request)?;
    let request = request.into_inner();
    let caller = Preparation::decode(
        request
            .preparation
            .ok_or_else(|| Status::invalid_argument("Preparation is required"))?,
        credential,
    )?;
    let target = request
        .target
        .ok_or_else(|| Status::invalid_argument("Reuse target is required"))?;
    let target = ConversationDeliveryTarget {
        snapshot: SnapshotId::parse(target.snapshot_id)
            .map_err(|_| Status::invalid_argument("Invalid reuse snapshot"))?,
        native_session: NativeConversationId::parse(target.native_session_id)
            .map_err(|_| Status::invalid_argument("Invalid reuse session"))?,
    };
    let Ok(_permit) = services.external_ingress.clone().try_acquire_owned() else {
        // This request could not record readiness. Return an explicit negative
        // observation, not a transport error that may hide a committed result.
        return Ok(Response::new(wire::ExternalEmptyObservation {
            confirmed_empty: false,
        }));
    };
    let confirmed_empty = external
        .confirm_empty(
            &caller.caller.source(),
            &McpAppliedScope {
                preparation: caller.scope(),
                target: &target,
            },
            request.observe_only,
        )
        .await
        .map_err(status)?;
    Ok(Response::new(wire::ExternalEmptyObservation {
        confirmed_empty,
    }))
}

fn catalogue_frames(page: ExternalCataloguePage) -> Vec<wire::ExternalCatalogFrame> {
    let mut frames = Vec::new();
    use wire::external_catalog_frame::Payload;
    for item in page.catalogues {
        match item.payload {
            Ok(bytes) => {
                let mut chunks = bytes.chunks(CHUNK_BYTES).peekable();
                while let Some(json) = chunks.next() {
                    frames.push(wire::ExternalCatalogFrame {
                        integration_id: item.integration_id.clone(),
                        payload: Some(Payload::Catalogue(wire::ExternalCatalogChunk {
                            json: json.to_vec(),
                            final_chunk: chunks.peek().is_none(),
                        })),
                    });
                }
            }
            Err(reason) => frames.push(wire::ExternalCatalogFrame {
                integration_id: item.integration_id,
                payload: Some(Payload::Unavailable(catalog_failure(reason).into())),
            }),
        }
    }
    frames.push(wire::ExternalCatalogFrame {
        integration_id: String::new(),
        payload: Some(Payload::End(wire::ExternalCatalogEnd {
            capacity_limited: page.capacity_limited,
        })),
    });
    frames
}

fn catalog_failure(reason: McpPreparationFailure) -> wire::ExternalCatalogFailure {
    use wire::ExternalCatalogFailure as F;
    match reason {
        McpPreparationFailure::Superseded => F::Superseded,
        McpPreparationFailure::Discovery(reason) => match reason {
            McpDiscoveryFailure::DestinationDenied => F::Destination,
            McpDiscoveryFailure::Busy => F::Capacity,
            McpDiscoveryFailure::Unavailable => F::Unavailable,
            McpDiscoveryFailure::Authentication => F::Authentication,
            McpDiscoveryFailure::UnsupportedVersion | McpDiscoveryFailure::MissingDiscovery => {
                F::Protocol
            }
            McpDiscoveryFailure::MissingToolsCapability => F::ToolsUnavailable,
            McpDiscoveryFailure::InvalidResponse | McpDiscoveryFailure::IncompleteCatalogue => {
                F::InvalidResponse
            }
            McpDiscoveryFailure::TimedOut => F::Timeout,
        },
    }
}

pub(super) async fn invoke(
    services: &ServerToolServices,
    request: Request<wire::ExternalInvocation>,
) -> Result<Response<wire::Final>, Status> {
    use wire::Rejection as R;
    services
        .admit()
        .map_err(|error| not_executed(error, R::Closing))?;
    let external = services.external.as_ref().ok_or_else(|| {
        not_executed(
            Status::unimplemented("External tool invocation is unavailable"),
            R::Authority,
        )
    })?;
    let _permit = services
        .external_ingress
        .clone()
        .try_acquire_owned()
        .map_err(|_| {
            not_executed(
                Status::resource_exhausted("External tool ingress is full"),
                R::Capacity,
            )
        })?;
    let credential = ingress::actor(&request).map_err(|error| not_executed(error, R::Authority))?;
    let value = request.into_inner();
    let (owner, target) = (|| {
        ingress::id(&value.integration_id)?;
        let owner = Preparation::decode(
            value
                .preparation
                .ok_or_else(|| Status::invalid_argument("External preparation is required"))?,
            credential,
        )?;
        let target = value
            .target
            .ok_or_else(|| Status::invalid_argument("External native target is required"))?;
        let target = ConversationDeliveryTarget {
            snapshot: SnapshotId::parse(target.snapshot_id)
                .map_err(|_| Status::invalid_argument("External native target is invalid"))?,
            native_session: NativeConversationId::parse(target.native_session_id)
                .map_err(|_| Status::invalid_argument("External native target is invalid"))?,
        };
        Ok::<_, Status>((owner, target))
    })()
    .map_err(|error| not_executed(error, R::InvalidEnvelope))?;
    let applied = McpAppliedScope {
        preparation: owner.scope(),
        target: &target,
    };
    let outcome = external
        .call(
            &owner.caller.source(),
            &applied,
            &value.integration_id,
            &value.name,
            &value.arguments_json,
        )
        .await
        .map_err(|error| not_executed(status(error), R::Authority))?;
    Ok(Response::new(call_result(outcome)))
}

fn call_result(outcome: ExternalMcpCallOutcome) -> wire::Final {
    use wire::Rejection as R;
    use wire::r#final::Outcome;
    let outcome = match outcome {
        ExternalMcpCallOutcome::NotExecuted(reason) => Outcome::NotExecuted(
            match reason {
                ExternalMcpRejection::Authority => R::Authority,
                ExternalMcpRejection::UnknownTool => R::UnknownTool,
                ExternalMcpRejection::InvalidArguments => R::InvalidEnvelope,
                ExternalMcpRejection::Capacity => R::Capacity,
                ExternalMcpRejection::Upstream(reason) => match reason {
                    McpDiscoveryFailure::DestinationDenied => R::UpstreamDestination,
                    McpDiscoveryFailure::Busy => R::Capacity,
                    McpDiscoveryFailure::Unavailable => R::UpstreamUnavailable,
                    McpDiscoveryFailure::Authentication => R::UpstreamAuthentication,
                    McpDiscoveryFailure::UnsupportedVersion
                    | McpDiscoveryFailure::MissingDiscovery => R::UpstreamProtocol,
                    McpDiscoveryFailure::MissingToolsCapability => R::UpstreamToolsUnavailable,
                    McpDiscoveryFailure::InvalidResponse
                    | McpDiscoveryFailure::IncompleteCatalogue => R::UpstreamInvalidResponse,
                    McpDiscoveryFailure::TimedOut => R::UpstreamTimeout,
                },
            }
            .into(),
        ),
        ExternalMcpCallOutcome::Completed { result_json }
        | ExternalMcpCallOutcome::ToolFailed { result_json } => Outcome::ResultJson(result_json),
        ExternalMcpCallOutcome::Uncertain => Outcome::Uncertain(()),
        ExternalMcpCallOutcome::UnsupportedResult => Outcome::UncertainRepresentation(()),
    };
    wire::Final {
        outcome: Some(outcome),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chapchoo_service::ExternalCatalogue;
    use prost::Message;
    use std::sync::Arc;

    #[test]
    fn external_catalogue_stream_delimits_complete_chunked_empty_and_unavailable_lists() {
        use wire::external_catalog_frame::Payload;
        let frames = catalogue_frames(ExternalCataloguePage {
            catalogues: vec![
                ExternalCatalogue {
                    integration_id: "chunked".into(),
                    payload: Ok(Arc::from(vec![42; CHUNK_BYTES + 1])),
                },
                ExternalCatalogue {
                    integration_id: "empty".into(),
                    payload: Ok(Arc::from(&b"{\"tools\":[]}"[..])),
                },
                ExternalCatalogue {
                    integration_id: "failed".into(),
                    payload: Err(McpPreparationFailure::Superseded),
                },
            ],
            capacity_limited: true,
        });
        assert!(frames.iter().all(|frame| frame.encoded_len() < 64 * 1024));
        assert!(matches!(
            frames.as_slice(),
            [
                wire::ExternalCatalogFrame { payload: Some(Payload::Catalogue(first)), .. },
                wire::ExternalCatalogFrame { payload: Some(Payload::Catalogue(last)), .. },
                wire::ExternalCatalogFrame { payload: Some(Payload::Catalogue(empty)), .. },
                wire::ExternalCatalogFrame { payload: Some(Payload::Unavailable(failure)), .. },
                wire::ExternalCatalogFrame { payload: Some(Payload::End(end)), .. },
            ] if first.json.len() == CHUNK_BYTES && !first.final_chunk
                && last.json == [42] && last.final_chunk && empty.final_chunk
                && empty.json == br#"{"tools":[]}"#
                && *failure == i32::from(wire::ExternalCatalogFailure::Superseded)
                && end.capacity_limited
        ));
    }

    #[test]
    fn external_call_outcomes_keep_positive_failure_and_uncertainty_separate() {
        use wire::r#final::Outcome;
        let result_json =
            br#"{"content":[],"structuredContent":{"failure":7},"isError":true}"#.to_vec();
        assert_eq!(
            call_result(ExternalMcpCallOutcome::ToolFailed {
                result_json: result_json.clone()
            })
            .outcome,
            Some(Outcome::ResultJson(result_json))
        );
        assert_eq!(
            call_result(ExternalMcpCallOutcome::NotExecuted(
                ExternalMcpRejection::Upstream(McpDiscoveryFailure::TimedOut)
            ))
            .outcome,
            Some(Outcome::NotExecuted(
                wire::Rejection::UpstreamTimeout.into()
            ))
        );
        assert_eq!(
            call_result(ExternalMcpCallOutcome::NotExecuted(
                ExternalMcpRejection::Upstream(McpDiscoveryFailure::Authentication)
            ))
            .outcome,
            Some(Outcome::NotExecuted(
                wire::Rejection::UpstreamAuthentication.into()
            ))
        );
        assert_eq!(
            call_result(ExternalMcpCallOutcome::Uncertain).outcome,
            Some(Outcome::Uncertain(()))
        );
        assert_eq!(
            call_result(ExternalMcpCallOutcome::UnsupportedResult).outcome,
            Some(Outcome::UncertainRepresentation(()))
        );
    }
}
