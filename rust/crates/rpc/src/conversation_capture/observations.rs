use super::*;
use chapchoo_service::{
    ConversationDeliveryTarget, ConversationForeground as F, ConversationNativeObservation,
    ConversationRequestKind, LiveConversationRequest,
};
fn target(value: Option<wire::NativeTarget>) -> Result<ConversationDeliveryTarget, Status> {
    let value = value.ok_or_else(|| Status::invalid_argument("Native target is required"))?;
    Ok(ConversationDeliveryTarget {
        snapshot: snapshot(value.snapshot_id)?,
        native_session: NativeConversationId::parse(value.native_session_id)
            .map_err(|_| Status::invalid_argument("Native session is invalid"))?,
    })
}
pub(super) async fn native(
    service: &ConversationCaptureServices,
    request: Request<wire::NativeStateReport>,
) -> Result<Response<()>, Status> {
    service.admit()?;
    let credential = actor(&request)?;
    let value = request.into_inner();
    let origin = value
        .source
        .ok_or_else(|| Status::invalid_argument("Native source is required"))?;
    let foreground = match wire::Foreground::try_from(value.foreground) {
        Ok(wire::Foreground::Idle) => F::Idle,
        Ok(wire::Foreground::Running) => F::Running,
        Ok(wire::Foreground::RequiresAction) => F::RequiresAction,
        Ok(wire::Foreground::OtherActive) => F::OtherActive,
        _ => return Err(Status::invalid_argument("Native foreground is invalid")),
    };
    service
        .service
        .observe_native(
            &source(&origin, &credential),
            &value.conversation_id,
            &ConversationNativeObservation {
                target: target(value.target)?,
                position: position(value.position.ok_or_else(|| {
                    Status::invalid_argument("State capture occurrence is required")
                })?)?,
                foreground,
                work_id: value.work_id,
                preparation: value
                    .preparation
                    .map(|value| {
                        use chapchoo_service::{
                            ConversationNativePreparation, ConversationPreparationFreshness as F,
                        };
                        let freshness = match wire::PreparationFreshness::try_from(value.freshness)
                        {
                            Ok(wire::PreparationFreshness::Current) => F::Current,
                            Ok(wire::PreparationFreshness::TransientLookupFailure) => {
                                F::TransientLookupFailure
                            }
                            _ => {
                                return Err(Status::invalid_argument(
                                    "Preparation freshness is invalid",
                                ));
                            }
                        };
                        Ok(ConversationNativePreparation {
                            freshness,
                            node: value.node,
                            adapter: value.adapter,
                            codex: value.codex,
                        })
                    })
                    .transpose()?,
                authentication_id: value.authentication_id,
            },
        )
        .await
        .map_err(status)?;
    Ok(Response::new(()))
}
pub(super) async fn request(
    service: &ConversationCaptureServices,
    request: Request<wire::LiveRequestReport>,
) -> Result<Response<()>, Status> {
    service.admit()?;
    let credential = actor(&request)?;
    let value = request.into_inner();
    let origin = value
        .source
        .ok_or_else(|| Status::invalid_argument("Request source is required"))?;
    let kind = match wire::RequestKind::try_from(value.kind) {
        Ok(wire::RequestKind::Form) => ConversationRequestKind::Form,
        Ok(wire::RequestKind::Permission) => ConversationRequestKind::Permission,
        _ => return Err(Status::invalid_argument("Request kind is invalid")),
    };
    service
        .service
        .observe_request(
            &source(&origin, &credential),
            &value.conversation_id,
            &target(value.target)?,
            &LiveConversationRequest {
                responder_token: value.responder_token,
                occurrence: position(value.position.ok_or_else(|| {
                    Status::invalid_argument("Request capture occurrence is required")
                })?)?,
                kind,
                open: value.open,
            },
        )
        .await
        .map_err(status)?;
    Ok(Response::new(()))
}
