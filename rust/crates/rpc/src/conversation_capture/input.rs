use super::*;
use chapchoo_service::{
    ConversationDeliveryTarget, ConversationInputOutcome, ConversationOperationForwarding,
};
fn target(command: &wire::InputCommand) -> Result<ConversationDeliveryTarget, Status> {
    let value = command
        .target
        .as_ref()
        .ok_or_else(|| Status::invalid_argument("Input target is required"))?;
    Ok(ConversationDeliveryTarget {
        snapshot: snapshot(value.snapshot_id.clone())?,
        native_session: NativeConversationId::parse(value.native_session_id.clone())
            .map_err(|_| Status::invalid_argument("Native target is invalid"))?,
    })
}
pub(super) async fn pull(
    service: &ConversationCaptureServices,
    request: Request<wire::InputChunkRequest>,
) -> Result<Response<wire::InputChunk>, Status> {
    service.admit()?;
    let credential = actor(&request)?;
    let value = request.into_inner();
    let origin = value
        .source
        .ok_or_else(|| Status::invalid_argument("Input source is required"))?;
    let command = value
        .command
        .ok_or_else(|| Status::invalid_argument("Input operation is required"))?;
    let target = target(&command)?;
    let chunk = service
        .service
        .input_chunk(
            &source(&origin, &credential),
            &ConversationOperationForwarding {
                agent_id: &command.agent_id,
                conversation_id: &command.conversation_id,
                operation_id: &command.operation_id,
                target: &target,
            },
            value.offset,
        )
        .await
        .map_err(status)?;
    Ok(Response::new(wire::InputChunk {
        content: chunk.bytes,
        total_bytes: chunk.total_bytes,
        offset: chunk.offset,
    }))
}
pub(super) async fn report(
    service: &ConversationCaptureServices,
    request: Request<wire::InputReport>,
) -> Result<Response<()>, Status> {
    service.admit()?;
    let credential = actor(&request)?;
    let value = request.into_inner();
    let origin = value
        .source
        .ok_or_else(|| Status::invalid_argument("Input source is required"))?;
    let command = value
        .command
        .ok_or_else(|| Status::invalid_argument("Input operation is required"))?;
    let target = target(&command)?;
    let outcome = match value.result {
        Some(wire::input_report::Result::InsertedNativeMessageId(native_message_id)) => {
            ConversationInputOutcome::Inserted { native_message_id }
        }
        Some(wire::input_report::Result::Steered(())) => ConversationInputOutcome::Steered,
        Some(wire::input_report::Result::Failed(())) => ConversationInputOutcome::Failed,
        None => {
            return Err(Status::invalid_argument(
                "Known native input outcome is required",
            ));
        }
    };
    service
        .service
        .report_input(
            &source(&origin, &credential),
            &ConversationOperationForwarding {
                agent_id: &command.agent_id,
                conversation_id: &command.conversation_id,
                operation_id: &command.operation_id,
                target: &target,
            },
            &outcome,
        )
        .await
        .map_err(status)?;
    Ok(Response::new(()))
}
