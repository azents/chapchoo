use super::*;
use chapchoo_service::{
    ConversationMessageRead, ConversationMessageRole, ConversationMessageScope,
    ConversationMessageWindow,
};

fn scope(value: Option<&wire::MessageScope>) -> Result<ConversationMessageScope<'_>, Status> {
    let scope = value.ok_or_else(|| Status::invalid_argument("Message scope is required"))?;
    Ok(ConversationMessageScope {
        agent_id: &scope.agent_id,
        conversation_id: &scope.conversation_id,
    })
}

pub(super) async fn list(
    service: &ConversationServices,
    request: Request<wire::OwnerMessagesRequest>,
) -> Result<Response<wire::MessageRegion>, Status> {
    service.admit()?;
    let actor = actor(&request)?;
    let query = request.into_inner();
    let window = match query.window {
        None => ConversationMessageWindow::Latest,
        Some(wire::owner_messages_request::Window::Before(order)) => {
            ConversationMessageWindow::Before(order)
        }
        Some(wire::owner_messages_request::Window::After(after)) => {
            ConversationMessageWindow::After {
                order: after.order,
                through: after.through,
            }
        }
    };
    let region = service
        .conversation
        .messages(
            &actor,
            &scope(query.scope.as_ref())?,
            window,
            i64::from(query.limit),
            now()?,
        )
        .await
        .map_err(status)?;
    let messages = region
        .messages
        .into_iter()
        .map(|message| wire::MessageMetadata {
            id: message.receipt.id,
            role: match message.receipt.role {
                ConversationMessageRole::User => wire::MessageRole::User,
                ConversationMessageRole::Agent => wire::MessageRole::Agent,
            } as i32,
            receipt_order: message.receipt.receipt_order,
            received_at: message.receipt.received_at,
            total_bytes: message.bytes,
            source_input_id: message.receipt.source_input_id,
            source_input_request_id: message.receipt.source_input_request_id,
            original_accepted_at: message.receipt.original_accepted_at,
        })
        .collect();
    Ok(Response::new(wire::MessageRegion {
        messages,
        through: region.through,
    }))
}

pub(super) async fn read(
    service: &ConversationServices,
    request: Request<wire::OwnerMessageRead>,
) -> Result<Response<wire::FieldChunk>, Status> {
    service.admit()?;
    let actor = actor(&request)?;
    let query = request.into_inner();
    let chunk = service
        .conversation
        .message_chunk(
            &actor,
            &ConversationMessageRead {
                scope: scope(query.scope.as_ref())?,
                message_id: &query.message_id,
                offset: query.offset,
                length: query.length,
            },
            now()?,
        )
        .await
        .map_err(status)?;
    Ok(Response::new(wire::FieldChunk {
        content: chunk.bytes,
        offset: chunk.offset,
        total_bytes: chunk.total_bytes,
    }))
}
