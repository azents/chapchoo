use super::*;
use chapchoo_service::{
    ConversationAnswer as Answer, ConversationAnswerAcceptance, ConversationAnswerDelivery,
    ConversationAnswerDraft, ConversationAnswerResult, ConversationAnswerValue as Value,
    ConversationFormAnswer as Form,
};
use prost::Message;
use std::collections::BTreeMap;
#[derive(PartialEq)]
pub(super) struct Identity {
    conversation: String,
    target: ConversationDeliveryTarget,
    responder: String,
}
fn decode(value: wire::TypedAnswer) -> Result<Answer, Status> {
    Ok(match value.answer.ok_or_else(invalid)? {
        wire::typed_answer::Answer::PermissionOptionId(option_id) => {
            Answer::PermissionSelected { option_id }
        }
        wire::typed_answer::Answer::PermissionCancelled(()) => Answer::PermissionCancelled,
        wire::typed_answer::Answer::FormDecline(()) => Answer::Form(Form::Decline),
        wire::typed_answer::Answer::FormCancel(()) => Answer::Form(Form::Cancel),
        wire::typed_answer::Answer::FormAccept(value) => {
            let content = value
                .content
                .map(|values| {
                    if values.fields.len() > 64 {
                        return Err(invalid());
                    }
                    let mut content = BTreeMap::new();
                    for field in values.fields {
                        if field.name.is_empty() || field.name.len() > 512 {
                            return Err(invalid());
                        }
                        let value = match field
                            .value
                            .and_then(|value| value.value)
                            .ok_or_else(invalid)?
                        {
                            wire::answer_value::Value::Text(value) => Value::Text(value),
                            wire::answer_value::Value::Integer(value) => Value::Integer(value),
                            wire::answer_value::Value::Number(value) if value.is_finite() => {
                                Value::Number(value)
                            }
                            wire::answer_value::Value::Number(_) => return Err(invalid()),
                            wire::answer_value::Value::Boolean(value) => Value::Boolean(value),
                            wire::answer_value::Value::Strings(value) => {
                                Value::Strings(value.values)
                            }
                        };
                        if content.insert(field.name, value).is_some() {
                            return Err(invalid());
                        }
                    }
                    Ok(content)
                })
                .transpose()?;
            Answer::Form(Form::Accept { content })
        }
    })
}
fn encode(answer: Answer) -> wire::TypedAnswer {
    let answer = match answer {
        Answer::PermissionSelected { option_id } => {
            wire::typed_answer::Answer::PermissionOptionId(option_id)
        }
        Answer::PermissionCancelled => wire::typed_answer::Answer::PermissionCancelled(()),
        Answer::Form(Form::Decline) => wire::typed_answer::Answer::FormDecline(()),
        Answer::Form(Form::Cancel) => wire::typed_answer::Answer::FormCancel(()),
        Answer::Form(Form::Accept { content }) => {
            wire::typed_answer::Answer::FormAccept(wire::FormAcceptance {
                content: content.map(|fields| wire::AnswerFields {
                    fields: fields
                        .into_iter()
                        .map(|(name, value)| {
                            let value = match value {
                                Value::Text(value) => wire::answer_value::Value::Text(value),
                                Value::Integer(value) => wire::answer_value::Value::Integer(value),
                                Value::Number(value) => wire::answer_value::Value::Number(value),
                                Value::Boolean(value) => wire::answer_value::Value::Boolean(value),
                                Value::Strings(values) => {
                                    wire::answer_value::Value::Strings(wire::AnswerStrings {
                                        values,
                                    })
                                }
                            };
                            wire::AnswerField {
                                name,
                                value: Some(wire::AnswerValue { value: Some(value) }),
                            }
                        })
                        .collect(),
                }),
            })
        }
    };
    wire::TypedAnswer {
        answer: Some(answer),
    }
}
pub(super) async fn read(
    service: &ConversationServices,
    request: Request<wire::OwnerAnswerRead>,
) -> Result<Response<wire::FieldChunk>, Status> {
    service.admit()?;
    let actor = actor(&request)?;
    let value = request.into_inner();
    if value.length == 0 || value.length > 32768 {
        return Err(invalid());
    }
    // Transcoding is bounded by the original 16-MiB answer and one shared reader.
    // The permit follows physical decoding/encoding through the complete response.
    let _permit = service
        .answer_readers
        .clone()
        .try_acquire_owned()
        .map_err(|_| Status::resource_exhausted("Retained answer read capacity is exhausted"))?;
    let source = value.source.ok_or_else(invalid)?;
    let snapshot = SnapshotId::parse(source.snapshot_id).map_err(|_| invalid())?;
    let answer = service
        .conversation
        .retained_answer(
            &actor,
            &source.agent_id,
            &source.conversation_id,
            &snapshot,
            &value.operation_id,
            now()?,
        )
        .await
        .map_err(status)?;
    let bytes = encode(answer).encode_to_vec();
    let offset = usize::try_from(value.offset).map_err(|_| invalid())?;
    if offset > bytes.len() {
        return Err(invalid());
    }
    let end = offset
        .saturating_add(value.length as usize)
        .min(bytes.len());
    Ok(Response::new(wire::FieldChunk {
        content: bytes[offset..end].to_vec(),
        offset: value.offset,
        total_bytes: i64::try_from(bytes.len()).map_err(|_| invalid())?,
    }))
}
fn invalid() -> Status {
    Status::invalid_argument("Typed answer is invalid")
}
pub(super) async fn live(
    service: &ConversationServices,
    request: Request<wire::OwnerLiveRequests>,
) -> Result<Response<wire::LiveRequestList>, Status> {
    service.admit()?;
    let actor = actor(&request)?;
    let value = request.into_inner();
    let target = value.target.ok_or_else(invalid)?;
    let target = ConversationDeliveryTarget {
        snapshot: SnapshotId::parse(target.snapshot_id).map_err(|_| invalid())?,
        native_session: NativeConversationId::parse(target.native_session_id)
            .map_err(|_| invalid())?,
    };
    let requests = service
        .conversation
        .live_requests(
            &actor,
            &value.agent_id,
            &value.conversation_id,
            &target,
            now()?,
        )
        .await
        .map_err(status)?;
    Ok(Response::new(wire::LiveRequestList {
        requests: requests
            .into_iter()
            .map(|value| wire::LiveRequest {
                responder_token: value.responder_token,
                occurrence: Some(wire::SourcePosition {
                    ordinal: value.occurrence.ordinal(),
                    occurrence_id: value.occurrence.occurrence().as_str().into(),
                }),
                kind: match value.kind {
                    chapchoo_service::ConversationRequestKind::Permission => {
                        wire::RequestKind::Permission
                    }
                    chapchoo_service::ConversationRequestKind::Form => wire::RequestKind::Form,
                } as i32,
                answer_request_id: value.answer_request_id,
            })
            .collect(),
    }))
}
fn acceptance(value: ConversationAnswerAcceptance) -> wire::AnswerAcceptance {
    wire::AnswerAcceptance {
        id: value.id,
        conversation_id: value.conversation_id,
        request_id: value.request_id,
        responder_token: value.responder_token,
        occurrence: Some(wire::SourcePosition {
            ordinal: value.occurrence.ordinal(),
            occurrence_id: value.occurrence.occurrence().as_str().into(),
        }),
        accepted_at: value.accepted_at,
        delivery: match value.delivery {
            ConversationAnswerDelivery::Accepted => wire::AnswerDelivery::Accepted,
            ConversationAnswerDelivery::PossibleForwarding => {
                wire::AnswerDelivery::PossibleForwarding
            }
            ConversationAnswerDelivery::Known(ConversationAnswerResult::ResponderAccepted) => {
                wire::AnswerDelivery::ResponderAccepted
            }
            ConversationAnswerDelivery::Known(ConversationAnswerResult::Rejected) => {
                wire::AnswerDelivery::Rejected
            }
        } as i32,
    }
}
pub(super) async fn submit(
    service: &ConversationServices,
    request: Request<wire::OwnerAnswerPiece>,
) -> Result<Response<wire::OwnerAnswerProgress>, Status> {
    service.admit()?;
    let actor = actor(&request)?;
    let value = request.into_inner();
    crate::ingress::id(&value.request_id)?;
    crate::ingress::id(&value.responder_token)?;
    let target = value.target.ok_or_else(invalid)?;
    let target = ConversationDeliveryTarget {
        snapshot: SnapshotId::parse(target.snapshot_id).map_err(|_| invalid())?,
        native_session: NativeConversationId::parse(target.native_session_id)
            .map_err(|_| invalid())?,
    };
    service
        .conversation
        .owner_conversation(&actor, &value.agent_id, now()?)
        .await
        .map_err(status)?;
    let progress = service.answer_uploads.append(
        transfer::Piece {
            key: (
                *actor.as_bytes(),
                value.agent_id.clone(),
                value.request_id.clone(),
            ),
            identity: Identity {
                conversation: value.conversation_id.clone(),
                target: target.clone(),
                responder: value.responder_token.clone(),
            },
            total: value.total_bytes,
            offset: value.offset,
            bytes: &value.content,
        },
        std::time::Instant::now(),
    )?;
    let outcome = match progress {
        transfer::Progress::Uploading(next_offset) => {
            wire::owner_answer_progress::Outcome::Uploading(wire::PieceAccepted { next_offset })
        }
        transfer::Progress::Complete(complete) => {
            let answer =
                wire::TypedAnswer::decode(complete.bytes.as_slice()).map_err(|_| invalid())?;
            let accepted = service
                .answers
                .accept(
                    actor,
                    ConversationAnswerDraft {
                        agent_id: value.agent_id,
                        conversation_id: value.conversation_id,
                        request_id: value.request_id,
                        target,
                        responder_token: value.responder_token,
                        answer: decode(answer)?,
                    },
                    AcceptanceTime::System,
                )
                .await
                .map_err(status)?;
            drop(complete);
            wire::owner_answer_progress::Outcome::Accepted(acceptance(accepted))
        }
    };
    Ok(Response::new(wire::OwnerAnswerProgress {
        outcome: Some(outcome),
    }))
}
pub(super) async fn result(
    service: &ConversationServices,
    request: Request<wire::OwnerAnswerResultRequest>,
) -> Result<Response<wire::OwnerAnswerStatus>, Status> {
    service.admit()?;
    let actor = actor(&request)?;
    let value = request.into_inner();
    let accepted = service
        .answers
        .current(
            &actor,
            &value.agent_id,
            &value.conversation_id,
            &value.request_id,
            AcceptanceTime::System,
        )
        .await
        .map_err(status)?;
    Ok(Response::new(wire::OwnerAnswerStatus {
        acceptance: accepted.map(acceptance),
    }))
}
