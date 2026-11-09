use super::*;
use chapchoo_service::{ConversationDeliveryTarget, ConversationWorkTarget};
fn target(value: Option<wire::WorkTarget>) -> Result<ConversationWorkTarget, Status> {
    let value = value.ok_or_else(|| Status::invalid_argument("Stop work target is required"))?;
    let native = value
        .source
        .ok_or_else(|| Status::invalid_argument("Stop source target is required"))?;
    Ok(ConversationWorkTarget {
        source: ConversationDeliveryTarget {
            snapshot: snapshot(native.snapshot_id)?,
            native_session: NativeConversationId::parse(native.native_session_id)
                .map_err(|_| Status::invalid_argument("Stop native session is invalid"))?,
        },
        work_id: value.work_id,
    })
}
pub(super) async fn check(
    service: &ConversationCaptureServices,
    request: Request<wire::StopCheck>,
) -> Result<Response<()>, Status> {
    service.admit()?;
    let credential = actor(&request)?;
    let value = request.into_inner();
    let origin = value
        .source
        .ok_or_else(|| Status::invalid_argument("Stop source is required"))?;
    let command = value
        .command
        .ok_or_else(|| Status::invalid_argument("Stop intent is required"))?;
    service
        .service
        .check_stop(
            &source(&origin, &credential),
            &command.conversation_id,
            &command.operation_id,
            &target(command.target)?,
        )
        .await
        .map_err(status)?;
    Ok(Response::new(()))
}
pub(super) async fn report(
    service: &ConversationCaptureServices,
    request: Request<wire::StopReport>,
) -> Result<Response<()>, Status> {
    service.admit()?;
    let credential = actor(&request)?;
    let value = request.into_inner();
    let origin = value
        .source
        .ok_or_else(|| Status::invalid_argument("Stop source is required"))?;
    let command = value
        .command
        .ok_or_else(|| Status::invalid_argument("Stop intent is required"))?;
    service
        .service
        .report_stop(
            &source(&origin, &credential),
            &command.conversation_id,
            &command.operation_id,
            &target(command.target)?,
            value.cancel_published,
        )
        .await
        .map_err(status)?;
    Ok(Response::new(()))
}
