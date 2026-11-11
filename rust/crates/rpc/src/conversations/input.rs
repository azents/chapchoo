use super::*;
use chapchoo_domain::{ConversationTarget, ConversationText};
use chapchoo_service::{
    AcceptConversationInput, ConversationInputAcceptance, ConversationInputDelivery,
    ConversationInputOutcome, ConversationInputOwnership,
};

#[derive(PartialEq)]
pub(super) struct Identity {
    conversation: Option<String>,
    target: ConversationTarget,
}
pub(super) fn target(value: Option<wire::InputTarget>) -> Result<ConversationTarget, Status> {
    match value.and_then(|target| target.target) {
        Some(wire::input_target::Target::NotStarted(())) => Ok(ConversationTarget::NotStarted),
        Some(wire::input_target::Target::SnapshotId(value)) => Ok(ConversationTarget::Snapshot(
            SnapshotId::parse(value)
                .map_err(|_| Status::invalid_argument("Input snapshot is invalid"))?,
        )),
        None => Err(Status::invalid_argument("Input target is required")),
    }
}
fn wire_target(target: ConversationTarget) -> wire::InputTarget {
    wire::InputTarget {
        target: Some(match target {
            ConversationTarget::NotStarted => wire::input_target::Target::NotStarted(()),
            ConversationTarget::Snapshot(value) => {
                wire::input_target::Target::SnapshotId(value.as_str().into())
            }
        }),
    }
}
fn acceptance(value: ConversationInputAcceptance) -> wire::InputAcceptance {
    wire::InputAcceptance {
        id: value.id,
        conversation_id: value.conversation_id,
        request_id: value.request_id,
        target: Some(wire_target(value.target)),
        accepted_at: value.accepted_at,
    }
}
pub(super) async fn submit(
    service: &ConversationServices,
    request: Request<wire::OwnerInputPiece>,
) -> Result<Response<wire::OwnerInputProgress>, Status> {
    service.admit()?;
    let actor = actor(&request)?;
    let value = request.into_inner();
    crate::ingress::id(&value.request_id)?;
    let target = target(value.target)?;
    // Recheck private ownership and Team admission before reserving upload bytes.
    // Acceptance revalidates all exact conversation/source authority atomically.
    service
        .conversation
        .owner_conversation(&actor, &value.agent_id, now()?)
        .await
        .map_err(status)?;
    let progress = service.uploads.append(
        transfer::Piece {
            key: (
                *actor.as_bytes(),
                value.agent_id.clone(),
                value.request_id.clone(),
            ),
            identity: Identity {
                conversation: value.conversation_id.clone(),
                target: target.clone(),
            },
            total: value.total_bytes,
            offset: value.offset,
            bytes: &value.content,
        },
        std::time::Instant::now(),
    )?;
    let outcome = match progress {
        transfer::Progress::Uploading(next_offset) => {
            wire::owner_input_progress::Outcome::Uploading(wire::PieceAccepted { next_offset })
        }
        transfer::Progress::Complete(mut complete) => {
            let text = String::from_utf8(std::mem::take(&mut complete.bytes))
                .map_err(|_| Status::invalid_argument("Input must be complete UTF-8 text"))?;
            let accepted = service
                .inputs
                .accept(
                    actor,
                    AcceptConversationInput {
                        agent_id: value.agent_id,
                        conversation_id: value.conversation_id,
                        request_id: value.request_id,
                        target,
                        text: ConversationText::parse(text)
                            .map_err(|_| Status::invalid_argument("Input text is invalid"))?,
                    },
                    AcceptanceTime::System,
                )
                .await
                .map_err(status)?;
            // `complete` retains upload capacity through the owned handoff.
            drop(complete);
            wire::owner_input_progress::Outcome::Accepted(acceptance(accepted))
        }
    };
    Ok(Response::new(wire::OwnerInputProgress {
        outcome: Some(outcome),
    }))
}
pub(super) async fn result(
    service: &ConversationServices,
    request: Request<wire::OwnerInputResultRequest>,
) -> Result<Response<wire::OwnerInputStatus>, Status> {
    service.admit()?;
    let actor = actor(&request)?;
    let query = request.into_inner();
    let Some(value) = service
        .inputs
        .input(
            &actor,
            &query.agent_id,
            &query.conversation_id,
            &query.request_id,
            AcceptanceTime::System,
        )
        .await
        .map_err(status)?
    else {
        return Ok(Response::new(wire::OwnerInputStatus::default()));
    };
    let preparation = match service
        .conversation
        .setup_facts(
            &actor,
            &chapchoo_service::ConversationInputPreparation {
                agent_id: &query.agent_id,
                conversation_id: &value.acceptance.conversation_id,
                operation_id: &value.acceptance.id,
            },
            now()?,
        )
        .await
    {
        Ok(facts) => facts,
        Err(error) if error.reason() == chapchoo_service::DomainReason::NotFound => {
            // Native pickup may remove the input after its status was read.
            // Preserve other NotFound causes unless this exact operation ended.
            let current = service
                .inputs
                .input(
                    &actor,
                    &query.agent_id,
                    &query.conversation_id,
                    &query.request_id,
                    AcceptanceTime::System,
                )
                .await
                .map_err(status)?;
            if current
                .as_ref()
                .is_none_or(|current| current.acceptance.id != value.acceptance.id)
            {
                return Ok(Response::new(wire::OwnerInputStatus::default()));
            }
            return Err(status(error));
        }
        Err(error) => return Err(status(error)),
    };
    let preparation = preparation.map(|facts| {
        use chapchoo_service::{
            ConversationPreparationFailure as Failure, ConversationPreparationOutcome as Outcome,
            ConversationPreparationState as State,
        };
        wire::InputPreparation {
            requested_supply: Some(wire_supply(facts.requested_supply)),
            observation: Some(match facts.state {
                State::PossibleForwarding => {
                    wire::input_preparation::Observation::PossibleForwarding(())
                }
                State::Known(Outcome::SessionReady(target)) => {
                    wire::input_preparation::Observation::SessionReady(native(&target))
                }
                State::Known(Outcome::Failed(failure)) => {
                    wire::input_preparation::Observation::Failed(match failure {
                        Failure::NotAdmitted => wire::SetupFailure::NotAdmitted,
                        Failure::BeforeNativeLaunch => wire::SetupFailure::BeforeNativeLaunch,
                        Failure::NativeRejected => wire::SetupFailure::NativeRejected,
                        Failure::SourceUnavailable => wire::SetupFailure::SourceUnavailable,
                        Failure::RetirementUnknown => wire::SetupFailure::RetirementUnknown,
                    } as i32)
                }
            }),
        }
    });
    let delivery = value.delivery.map(|delivery| wire::InputDelivery {
        observation: Some(match delivery {
            ConversationInputDelivery::PossibleForwarding => {
                wire::input_delivery::Observation::PossibleForwarding(())
            }
            ConversationInputDelivery::Known(ConversationInputOutcome::Inserted {
                native_message_id,
            }) => wire::input_delivery::Observation::InsertedNativeMessageId(native_message_id),
            ConversationInputDelivery::Known(ConversationInputOutcome::Steered) => {
                wire::input_delivery::Observation::Steered(())
            }
            ConversationInputDelivery::Known(ConversationInputOutcome::Failed) => {
                wire::input_delivery::Observation::Failed(())
            }
        }),
    });
    Ok(Response::new(wire::OwnerInputStatus {
        deleted: value.deleted,
        acceptance: Some(acceptance(value.acceptance)),
        ownership: match value.ownership {
            ConversationInputOwnership::ProcessingHere => wire::InputOwnership::ProcessingHere,
            ConversationInputOwnership::NotObservedHere => wire::InputOwnership::NotObservedHere,
        } as i32,
        delivery,
        preparation,
    }))
}

pub(super) async fn list(
    service: &ConversationServices,
    request: Request<wire::OwnerInputsRequest>,
) -> Result<Response<wire::RetainedInputs>, Status> {
    service.admit()?;
    let actor = actor(&request)?;
    let value = request.into_inner();
    let scope = value
        .scope
        .ok_or_else(|| Status::invalid_argument("Input scope is required"))?;
    let target = target(scope.target)?;
    let before = value
        .before
        .map(|value| chapchoo_service::ConversationInputAnchor {
            accepted_at: value.accepted_at,
            request_id: value.request_id,
        });
    let values = service
        .conversation
        .inputs(
            &actor,
            &chapchoo_service::ConversationInputScope {
                agent_id: &scope.agent_id,
                conversation_id: &scope.conversation_id,
                target: &target,
            },
            before.as_ref(),
            i64::from(value.limit),
            now()?,
        )
        .await
        .map_err(status)?;
    let latest = service
        .conversation
        .latest_input(
            &actor,
            &chapchoo_service::ConversationInputScope {
                agent_id: &scope.agent_id,
                conversation_id: &scope.conversation_id,
                target: &target,
            },
            now()?,
        )
        .await
        .map_err(status)?
        .map(|value| {
            acceptance(ConversationInputAcceptance {
                id: value.reference.id,
                conversation_id: value.reference.conversation_id,
                request_id: value.reference.request_id,
                target: value.reference.target,
                accepted_at: value.reference.accepted_at,
            })
        });
    Ok(Response::new(wire::RetainedInputs {
        closed_ids: service
            .conversation
            .closed_inputs(
                &actor,
                &chapchoo_service::ConversationInputScope {
                    agent_id: &scope.agent_id,
                    conversation_id: &scope.conversation_id,
                    target: &target,
                },
                &value.known_ids,
                now()?,
            )
            .await
            .map_err(status)?,
        latest,
        inputs: values
            .into_iter()
            .map(|value| wire::RetainedInput {
                acceptance: Some(acceptance(ConversationInputAcceptance {
                    id: value.reference.id,
                    conversation_id: value.reference.conversation_id,
                    request_id: value.reference.request_id,
                    target: value.reference.target,
                    accepted_at: value.reference.accepted_at,
                })),
                total_bytes: value.bytes,
                deleted: value.deleted,
            })
            .collect(),
    }))
}
pub(super) async fn delete(
    service: &ConversationServices,
    request: Request<wire::OwnerDeleteInput>,
) -> Result<Response<()>, Status> {
    service.admit()?;
    let actor = actor(&request)?;
    let value = request.into_inner();
    let scope = value
        .scope
        .ok_or_else(|| Status::invalid_argument("Input scope is required"))?;
    let target = target(scope.target)?;
    service
        .conversation
        .delete_input(
            &actor,
            &chapchoo_service::ConversationInputScope {
                agent_id: &scope.agent_id,
                conversation_id: &scope.conversation_id,
                target: &target,
            },
            &value.request_id,
            now()?,
        )
        .await
        .map_err(status)?;
    Ok(Response::new(()))
}
pub(super) async fn read(
    service: &ConversationServices,
    request: Request<wire::OwnerInputRead>,
) -> Result<Response<wire::FieldChunk>, Status> {
    service.admit()?;
    let actor = actor(&request)?;
    let value = request.into_inner();
    let scope = value
        .scope
        .ok_or_else(|| Status::invalid_argument("Input scope is required"))?;
    let target = target(scope.target)?;
    let chunk = service
        .conversation
        .input_text(
            &actor,
            &chapchoo_service::ConversationInputScope {
                agent_id: &scope.agent_id,
                conversation_id: &scope.conversation_id,
                target: &target,
            },
            &value.request_id,
            value.offset,
            value.length,
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
