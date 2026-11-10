//! Real owner wire transfers complete text; duplicate acceptance never processes it.
use super::*;
use chapchoo_domain::{ConversationTarget, ConversationText};
use chapchoo_persistence::AcceptConversationInput;
use chapchoo_protocol::conversations::{self as c, conversations_client::ConversationsClient};

pub(super) async fn exercise(
    context: &super::capture::Context<'_>,
    conversation: &str,
) -> Result<(), TestError> {
    let mut owner = ConversationsClient::connect(context.endpoint.to_owned())
        .await?
        .max_encoding_message_size(64 * 1024)
        .max_decoding_message_size(64 * 1024);
    let text = " Keep whitespace 한글🦀\n".repeat(6000);
    let request_id = "owner-large-input";
    let expected = context
        .store
        .accept_conversation_input(
            context.owner,
            &AcceptConversationInput {
                agent_id: context.agent.id.clone(),
                conversation_id: Some(conversation.into()),
                request_id: request_id.into(),
                target: ConversationTarget::NotStarted,
                text: ConversationText::parse(text.clone())?,
            },
            AcceptanceTime::System.now()?,
        )
        .await?;
    let result = c::OwnerInputResultRequest {
        agent_id: context.agent.id.clone(),
        conversation_id: conversation.into(),
        request_id: request_id.into(),
    };
    // Another server's accepted operation is recovered, not dispatched again.
    for round in 0..2 {
        for (index, bytes) in text.as_bytes().chunks(32768).enumerate() {
            let offset = index * 32768;
            let piece = c::OwnerInputPiece {
                agent_id: context.agent.id.clone(),
                conversation_id: Some(conversation.into()),
                request_id: request_id.into(),
                target: Some(c::InputTarget {
                    target: Some(c::input_target::Target::NotStarted(())),
                }),
                total_bytes: u32::try_from(text.len())?,
                offset: u32::try_from(offset)?,
                content: bytes.to_vec(),
            };
            let progress = owner
                .submit_input(authenticated(piece.clone(), context.user_key)?)
                .await?
                .into_inner();
            if offset + bytes.len() == text.len() {
                let Some(c::owner_input_progress::Outcome::Accepted(accepted)) = progress.outcome
                else {
                    return Err("Missing acceptance".into());
                };
                assert_eq!(accepted.id, expected.id);
            } else {
                assert!(matches!(
                    progress.outcome,
                    Some(c::owner_input_progress::Outcome::Uploading(_))
                ));
            }
            if index == 0 && round == 0 {
                let mut changed = piece.clone();
                changed.content[0] ^= 1;
                assert_eq!(
                    owner
                        .submit_input(authenticated(changed, context.user_key)?)
                        .await
                        .err()
                        .ok_or("Conflicting bytes accepted")?
                        .code(),
                    Code::AlreadyExists
                );
                let mut partial = piece;
                partial.request_id = "only-partial".into();
                owner
                    .submit_input(authenticated(partial, context.user_key)?)
                    .await?;
                assert!(
                    owner
                        .input_result(authenticated(
                            c::OwnerInputResultRequest {
                                request_id: "only-partial".into(),
                                ..result.clone()
                            },
                            context.user_key
                        )?)
                        .await?
                        .into_inner()
                        .acceptance
                        .is_none()
                );
            }
        }
    }
    let status = owner
        .input_result(authenticated(result.clone(), context.user_key)?)
        .await?
        .into_inner();
    assert_eq!(
        status.acceptance.ok_or("Missing persisted acceptance")?.id,
        expected.id
    );
    assert_eq!(status.ownership, c::InputOwnership::NotObservedHere as i32);
    assert!(status.delivery.is_none());
    let scope = c::OwnerInputScope {
        agent_id: context.agent.id.clone(),
        conversation_id: conversation.into(),
        target: Some(c::InputTarget {
            target: Some(c::input_target::Target::NotStarted(())),
        }),
    };
    let listed = owner
        .inputs(authenticated(
            c::OwnerInputsRequest {
                scope: Some(scope.clone()),
                before: None,
                limit: 25,
            },
            context.user_key,
        )?)
        .await?
        .into_inner();
    let listed = listed
        .inputs
        .iter()
        .find(|input| {
            input
                .acceptance
                .as_ref()
                .is_some_and(|receipt| receipt.id == expected.id)
        })
        .ok_or("Missing retained input discovery")?;
    assert_eq!(listed.total_bytes, i64::try_from(text.len())?);
    let read = c::OwnerInputRead {
        scope: Some(scope.clone()),
        request_id: request_id.into(),
        offset: 0,
        length: 32768,
    };
    let mut complete = Vec::new();
    loop {
        let chunk = owner
            .read_input(authenticated(
                c::OwnerInputRead {
                    offset: u32::try_from(complete.len())?,
                    ..read.clone()
                },
                context.user_key,
            )?)
            .await?
            .into_inner();
        assert_eq!(chunk.total_bytes, listed.total_bytes);
        assert!(chunk.content.len() <= 32768);
        if chunk.content.is_empty() {
            assert_eq!(complete.len(), text.len());
            break;
        }
        complete.extend(chunk.content);
    }
    assert_eq!(String::from_utf8(complete)?, text);
    assert_eq!(
        owner
            .read_input(authenticated(read.clone(), context.key)?)
            .await
            .err()
            .ok_or("Component gained retained input read")?
            .code(),
        Code::Unauthenticated
    );
    let stale = c::OwnerInputScope {
        target: Some(c::InputTarget {
            target: Some(c::input_target::Target::SnapshotId("old-source".into())),
        }),
        ..scope
    };
    assert_eq!(
        owner
            .read_input(authenticated(
                c::OwnerInputRead {
                    scope: Some(stale),
                    ..read
                },
                context.user_key
            )?)
            .await
            .err()
            .ok_or("Stale source read retained input")?
            .code(),
        Code::AlreadyExists
    );
    assert_eq!(
        owner
            .input_result(authenticated(result, context.key)?)
            .await
            .err()
            .ok_or("Component gained owner read")?
            .code(),
        Code::Unauthenticated
    );
    assert_eq!(
        context
            .store
            .conversation_input(
                context.owner,
                &context.agent.id,
                conversation,
                request_id,
                AcceptanceTime::System.now()?
            )
            .await?
            .ok_or("Missing original text")?
            .text
            .as_str(),
        text
    );
    Ok(())
}
