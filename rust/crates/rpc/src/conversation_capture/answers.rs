use super::*;
use chapchoo_service::{
    ConversationAnswerResult, ConversationDeliveryTarget, ConversationOperationForwarding,
    ConversationRequestKind,
};
fn target(command: &wire::AnswerCommand) -> Result<ConversationDeliveryTarget, Status> {
    let value = command
        .target
        .as_ref()
        .ok_or_else(|| Status::invalid_argument("Answer target is required"))?;
    Ok(ConversationDeliveryTarget {
        snapshot: snapshot(value.snapshot_id.clone())?,
        native_session: NativeConversationId::parse(value.native_session_id.clone())
            .map_err(|_| Status::invalid_argument("Native answer target is invalid"))?,
    })
}
pub(super) async fn pull(
    service: &ConversationCaptureServices,
    request: Request<wire::AnswerChunkRequest>,
) -> Result<Response<wire::AnswerChunk>, Status> {
    service.admit()?;
    let credential = actor(&request)?;
    let value = request.into_inner();
    let origin = value
        .source
        .ok_or_else(|| Status::invalid_argument("Answer source is required"))?;
    let command = value
        .command
        .ok_or_else(|| Status::invalid_argument("Answer command is required"))?;
    let target = target(&command)?;
    let chunk = service
        .service
        .answer_chunk(
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
    Ok(Response::new(wire::AnswerChunk {
        content: chunk.content.bytes,
        offset: chunk.content.offset,
        total_bytes: chunk.content.total_bytes,
        responder_token: chunk.responder_token,
        occurrence: Some(wire_position(&chunk.occurrence)),
        kind: match chunk.kind {
            ConversationRequestKind::Permission => wire::RequestKind::Permission,
            ConversationRequestKind::Form => wire::RequestKind::Form,
        } as i32,
    }))
}
pub(super) async fn report(
    service: &ConversationCaptureServices,
    request: Request<wire::AnswerReport>,
) -> Result<Response<()>, Status> {
    service.admit()?;
    let credential = actor(&request)?;
    let value = request.into_inner();
    let origin = value
        .source
        .ok_or_else(|| Status::invalid_argument("Answer source is required"))?;
    let command = value
        .command
        .ok_or_else(|| Status::invalid_argument("Answer command is required"))?;
    let target = target(&command)?;
    service
        .service
        .report_answer(
            &source(&origin, &credential),
            &ConversationOperationForwarding {
                agent_id: &command.agent_id,
                conversation_id: &command.conversation_id,
                operation_id: &command.operation_id,
                target: &target,
            },
            if value.responder_accepted {
                ConversationAnswerResult::ResponderAccepted
            } else {
                ConversationAnswerResult::Rejected
            },
        )
        .await
        .map_err(status)?;
    Ok(Response::new(()))
}
