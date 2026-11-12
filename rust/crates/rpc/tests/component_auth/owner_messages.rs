use super::*;
use chapchoo_protocol::conversations::{self as c, conversations_client::ConversationsClient};

pub(super) async fn exercise(
    context: &super::capture::Context<'_>,
    conversation: &str,
    input_id: &str,
    expected_text: &str,
) -> Result<(), TestError> {
    let mut client = ConversationsClient::connect(context.endpoint.to_owned())
        .await?
        .max_encoding_message_size(64 * 1024)
        .max_decoding_message_size(64 * 1024);
    let scope = c::MessageScope {
        agent_id: context.agent.id.clone(),
        conversation_id: conversation.into(),
    };
    let request = c::OwnerMessagesRequest {
        scope: Some(scope.clone()),
        window: None,
        limit: 25,
    };
    let region = client
        .messages(authenticated(request.clone(), context.user_key)?)
        .await?
        .into_inner();
    let matches = region
        .messages
        .iter()
        .filter(|m| m.source_input_id.as_deref() == Some(input_id))
        .collect::<Vec<_>>();
    assert_eq!(matches.len(), 1, "Chunked receipt recovery duplicated chat");
    let message = *matches.first().ok_or("Missing accepted message")?;
    assert_eq!(message.role, c::MessageRole::User as i32);
    assert_eq!(
        message.source_input_request_id.as_deref(),
        Some("owner-large-input")
    );
    assert_eq!(message.total_bytes, i64::try_from(expected_text.len())?);
    assert!(message.receipt_order > 0 && message.receipt_order <= region.through);
    let read = c::OwnerMessageRead {
        scope: Some(scope),
        message_id: message.id.clone(),
        offset: 0,
        length: 32768,
    };
    let mut complete = Vec::new();
    loop {
        let chunk = client
            .read_message(authenticated(
                c::OwnerMessageRead {
                    offset: u32::try_from(complete.len())?,
                    ..read.clone()
                },
                context.user_key,
            )?)
            .await?
            .into_inner();
        assert_eq!(chunk.offset, u32::try_from(complete.len())?);
        assert_eq!(chunk.total_bytes, message.total_bytes);
        assert!(chunk.content.len() <= 32768);
        if chunk.content.is_empty() {
            break;
        }
        complete.extend(chunk.content);
    }
    assert_eq!(String::from_utf8(complete)?, expected_text);
    let older = client
        .messages(authenticated(
            c::OwnerMessagesRequest {
                window: Some(c::owner_messages_request::Window::Before(
                    message.receipt_order,
                )),
                ..request.clone()
            },
            context.user_key,
        )?)
        .await?
        .into_inner();
    assert!(
        older
            .messages
            .iter()
            .all(|m| m.receipt_order < message.receipt_order)
    );
    let bounded = client
        .messages(authenticated(
            c::OwnerMessagesRequest {
                window: Some(c::owner_messages_request::Window::After(c::MessageAfter {
                    order: 0,
                    through: Some(region.through),
                })),
                ..request.clone()
            },
            context.user_key,
        )?)
        .await?
        .into_inner();
    assert_eq!(bounded.through, region.through);
    for request in [
        c::OwnerMessagesRequest {
            limit: 26,
            ..request.clone()
        },
        c::OwnerMessagesRequest {
            scope: None,
            ..request.clone()
        },
        c::OwnerMessagesRequest {
            window: Some(c::owner_messages_request::Window::After(c::MessageAfter {
                order: 0,
                through: Some(region.through + 1),
            })),
            ..request.clone()
        },
    ] {
        assert_eq!(
            client
                .messages(authenticated(request, context.user_key)?)
                .await
                .err()
                .ok_or("Invalid message page accepted")?
                .code(),
            Code::InvalidArgument
        );
    }
    for read in [
        c::OwnerMessageRead {
            length: 32769,
            ..read.clone()
        },
        c::OwnerMessageRead {
            offset: u32::try_from(expected_text.len())? + 1,
            ..read.clone()
        },
        c::OwnerMessageRead {
            message_id: input_id.into(),
            ..read.clone()
        },
    ] {
        assert_eq!(
            client
                .read_message(authenticated(read, context.user_key)?)
                .await
                .err()
                .ok_or("Invalid message body read accepted")?
                .code(),
            Code::InvalidArgument
        );
    }
    assert_eq!(
        client
            .messages(authenticated(request, context.key)?)
            .await
            .err()
            .ok_or("Component gained private chat access")?
            .code(),
        Code::Unauthenticated
    );
    assert_eq!(
        client
            .read_message(authenticated(read, context.key)?)
            .await
            .err()
            .ok_or("Component gained private chat body access")?
            .code(),
        Code::Unauthenticated
    );
    Ok(())
}

pub(super) async fn native_output_is_not_chat(
    context: &super::capture::Context<'_>,
    conversation: &str,
) -> Result<(), TestError> {
    let mut client = ConversationsClient::connect(context.endpoint.to_owned()).await?;
    let region = client
        .messages(authenticated(
            c::OwnerMessagesRequest {
                scope: Some(c::MessageScope {
                    agent_id: context.agent.id.clone(),
                    conversation_id: conversation.into(),
                }),
                window: None,
                limit: 25,
            },
            context.user_key,
        )?)
        .await?
        .into_inner();
    assert!(!region.messages.is_empty());
    assert!(
        region
            .messages
            .iter()
            .all(|m| m.role == c::MessageRole::User as i32)
    );
    Ok(())
}
