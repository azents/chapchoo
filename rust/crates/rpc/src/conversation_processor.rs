//! Actual server input processing over existing exact-call PeerDelivery.
use crate::PeerDelivery;
use chapchoo_domain::{ConversationSource, ConversationTarget, HarnessSupply};
use chapchoo_protocol::{control, conversations as wire};
use chapchoo_service::{
    AcceptanceTime, AcceptedConversationInput, ComputersService, ConversationForwardingAdmission,
    ConversationInputFuture, ConversationInputPreparation, ConversationInputProcessor,
    ConversationOperationForwarding, ConversationPreparationAdmission,
    ConversationPreparationOutcome, ConversationPreparationState, ConversationService,
    DomainReason, ServiceError,
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::time::Instant;
#[derive(Clone)]
pub struct ConversationControlProcessor {
    computers: Arc<ComputersService>,
    conversation: Arc<ConversationService>,
    delivery: PeerDelivery,
    admission: Arc<AtomicBool>,
}
impl std::fmt::Debug for ConversationControlProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ConversationControlProcessor([REDACTED])")
    }
}
impl ConversationControlProcessor {
    pub fn new(
        computers: Arc<ComputersService>,
        conversation: Arc<ConversationService>,
        delivery: PeerDelivery,
        admission: Arc<AtomicBool>,
    ) -> Self {
        Self {
            computers,
            conversation,
            delivery,
            admission,
        }
    }
    async fn process_owned(&self, input: AcceptedConversationInput) -> Result<(), ServiceError> {
        if !self.admission.load(Ordering::Acquire) {
            return Ok(());
        }
        let actor = &input.actor;
        let mut agent = self
            .computers
            .agent(actor, &input.agent_id, AcceptanceTime::System)
            .await?;
        let Some(route) = self
            .computers
            .resolve_connector_route(&agent.computer.id)
            .await?
        else {
            return Ok(());
        };
        let draft = ConversationInputPreparation {
            agent_id: &input.agent_id,
            conversation_id: &input.receipt.conversation_id,
            operation_id: &input.receipt.id,
        };
        let admission = self
            .conversation
            .begin_setup(actor, &route, &draft, AcceptanceTime::System.now()?)
            .await?;
        let (execution, supply, fresh, require_retained_source, startup_operation) = match admission
        {
            ConversationPreparationAdmission::Granted {
                grant,
                requested_supply,
                fresh_native_allowed,
                require_retained_source,
                ..
            } => (
                grant.execution_id,
                requested_supply,
                fresh_native_allowed,
                require_retained_source,
                None,
            ),
            // A processing task has no authority to repeat another setup grant.
            ConversationPreparationAdmission::Existing(_) => return Ok(()),
            ConversationPreparationAdmission::ObservedStartup {
                operation_id,
                facts,
            } => (
                facts.execution_id,
                facts.requested_supply,
                facts.fresh_native_allowed,
                facts.require_retained_source,
                Some(operation_id),
            ),
        };
        let current = self
            .conversation
            .owner_conversation(actor, &input.agent_id, AcceptanceTime::System.now()?)
            .await?;
        let Some(context) = setup_context(
            &input.receipt.target,
            current.as_ref(),
            fresh,
            require_retained_source,
        )?
        else {
            return Ok(());
        };
        let supply = wire_supply(supply);
        let mut startup_operation = startup_operation;
        let mut execution = execution;
        if startup_operation.is_none() {
            let setup = wire::SetupCommand {
                agent_id: input.agent_id.clone(),
                conversation_id: input.receipt.conversation_id.clone(),
                operation_id: input.receipt.id.clone(),
                supply: Some(supply),
                context: Some(context),
            };
            if !self
                .invoke(
                    &input.actor,
                    &agent.computer.id,
                    &agent.computer.current_request_id,
                    &execution,
                    control::connector_command::Operation::Setup(setup),
                )
                .await?
            {
                return Ok(());
            }
        }
        // Product acceptance has already returned. This finite setup observation
        // budget is not a native task queue or model-completion waiter.
        let deadline = Instant::now() + Duration::from_secs(300);
        let target = loop {
            if !self.admission.load(Ordering::Acquire) || Instant::now() >= deadline {
                return Ok(());
            }
            let state = if let Some(operation_id) = &startup_operation {
                let Some(observed) = self
                    .conversation
                    .connection_startup(actor, &input.agent_id, AcceptanceTime::System.now()?)
                    .await?
                else {
                    return Ok(());
                };
                if &observed.operation_id != operation_id {
                    return Ok(());
                }
                observed.facts.state
            } else {
                let Some(facts) = self
                    .conversation
                    .setup_facts(actor, &draft, AcceptanceTime::System.now()?)
                    .await?
                else {
                    return Ok(());
                };
                facts.state
            };
            match state {
                ConversationPreparationState::Known(ConversationPreparationOutcome::Failed(
                    failure,
                )) if startup_operation.is_some()
                    && matches!(
                        failure,
                        chapchoo_service::ConversationPreparationFailure::NotAdmitted
                            | chapchoo_service::ConversationPreparationFailure::BeforeNativeLaunch
                    ) =>
                {
                    agent = self
                        .computers
                        .agent(actor, &input.agent_id, AcceptanceTime::System)
                        .await?;
                    let Some(route) = self
                        .computers
                        .resolve_connector_route(&agent.computer.id)
                        .await?
                    else {
                        return Ok(());
                    };
                    match self
                        .conversation
                        .begin_setup(actor, &route, &draft, AcceptanceTime::System.now()?)
                        .await?
                    {
                        ConversationPreparationAdmission::Granted {
                            grant,
                            requested_supply,
                            fresh_native_allowed,
                            require_retained_source,
                            ..
                        } => {
                            execution = grant.execution_id;
                            startup_operation = None;
                            let current = self
                                .conversation
                                .owner_conversation(
                                    actor,
                                    &input.agent_id,
                                    AcceptanceTime::System.now()?,
                                )
                                .await?;
                            let Some(context) = setup_context(
                                &input.receipt.target,
                                current.as_ref(),
                                fresh_native_allowed,
                                require_retained_source,
                            )?
                            else {
                                return Ok(());
                            };
                            let setup = wire::SetupCommand {
                                agent_id: input.agent_id.clone(),
                                conversation_id: input.receipt.conversation_id.clone(),
                                operation_id: input.receipt.id.clone(),
                                supply: Some(wire_supply(requested_supply)),
                                context: Some(context),
                            };
                            if !self
                                .invoke(
                                    &input.actor,
                                    &agent.computer.id,
                                    &agent.computer.current_request_id,
                                    &execution,
                                    control::connector_command::Operation::Setup(setup),
                                )
                                .await?
                            {
                                return Ok(());
                            }
                        }
                        ConversationPreparationAdmission::Existing(_) => return Ok(()),
                        ConversationPreparationAdmission::ObservedStartup {
                            operation_id, ..
                        } => {
                            startup_operation = Some(operation_id);
                        }
                    }
                }
                ConversationPreparationState::Known(ConversationPreparationOutcome::Failed(_)) => {
                    return Ok(());
                }
                ConversationPreparationState::Known(
                    ConversationPreparationOutcome::SessionReady(target),
                ) => {
                    let Some(current) = self
                        .conversation
                        .owner_conversation(actor, &input.agent_id, AcceptanceTime::System.now()?)
                        .await?
                    else {
                        return Ok(());
                    };
                    if matches!(current.source, ConversationSource::Ready { .. })
                        && current.source.snapshot() == Some(&target.snapshot)
                        && current.native_session_id.as_ref() == Some(&target.native_session)
                    {
                        break target;
                    }
                }
                ConversationPreparationState::PossibleForwarding => {}
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        };
        let Some(route) = self
            .computers
            .resolve_connector_route(&agent.computer.id)
            .await?
        else {
            return Ok(());
        };
        agent = self
            .computers
            .agent(actor, &input.agent_id, AcceptanceTime::System)
            .await?;
        let grant = self
            .conversation
            .begin_input(
                actor,
                &route,
                &ConversationOperationForwarding {
                    agent_id: &input.agent_id,
                    conversation_id: &input.receipt.conversation_id,
                    operation_id: &input.receipt.id,
                    target: &target,
                },
                AcceptanceTime::System.now()?,
            )
            .await?;
        let ConversationForwardingAdmission::Granted(grant) = grant else {
            return Ok(());
        };
        let command = wire::InputCommand {
            agent_id: input.agent_id.clone(),
            conversation_id: input.receipt.conversation_id.clone(),
            operation_id: input.receipt.id.clone(),
            target: Some(wire::NativeTarget {
                snapshot_id: target.snapshot.as_str().into(),
                native_session_id: target.native_session.as_str().into(),
            }),
        };
        let dispatched = self
            .invoke(
                &input.actor,
                &agent.computer.id,
                &agent.computer.current_request_id,
                &grant.execution_id,
                control::connector_command::Operation::Input(command),
            )
            .await?;
        if !dispatched {
            return Ok(());
        }
        // This is a queue-consumer/report lifetime, not model completion.
        // Connector terminal errors are explicitly reported even when no
        // physical outcome can be asserted.
        while self.admission.load(Ordering::Acquire)
            && self
                .conversation
                .input_pending(
                    actor,
                    &input.agent_id,
                    &input.receipt.conversation_id,
                    &input.receipt.id,
                )
                .await?
        {
            let current = self
                .computers
                .resolve_connector_route(&agent.computer.id)
                .await?;
            if current.is_none_or(|current| {
                current.component.control_id != route.component.control_id
                    || current.component.connection_id != route.component.connection_id
            }) {
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
        Ok(())
    }
    async fn invoke(
        &self,
        actor: &chapchoo_crypto::SecretDigest,
        computer: &str,
        request: &str,
        execution: &str,
        operation: control::connector_command::Operation,
    ) -> Result<bool, ServiceError> {
        let call =
            chapchoo_crypto::BearerSecret::generate().map_err(ServiceError::infrastructure)?;
        let call_id = call
            .digest()
            .as_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        let result = self
            .delivery
            .connector(
                control::ConnectorInvocation {
                    computer_id: computer.into(),
                    owner: None,
                    authority: Some(control::PeerAuthority {
                        context: Some(control::peer_authority::Context::User(
                            control::UserAuthority {
                                session_credential_digest: actor.as_bytes().into(),
                            },
                        )),
                    }),
                    command: Some(control::ConnectorCommand {
                        identity: Some(control::CallIdentity {
                            request_id: request.into(),
                            call_id,
                            connection_id: String::new(),
                        }),
                        computer: Some(control::ComputerContext {
                            computer_id: computer.into(),
                            request_id: request.into(),
                            execution_id: execution.into(),
                        }),
                        remaining_ms: 10_000,
                        operation: Some(operation),
                    }),
                },
                Instant::now() + Duration::from_secs(10),
            )
            .await
            .map_err(ServiceError::infrastructure)?;
        Ok(
            result.outcome == control::ForwardingOutcome::Completed as i32
                && result
                    .result
                    .is_some_and(|result| result.admitted == Some(true)),
        )
    }
}
fn wire_supply(supply: HarnessSupply) -> wire::HarnessSupply {
    wire::HarnessSupply {
        choice: Some(match supply {
            HarnessSupply::Managed => wire::harness_supply::Choice::Managed(()),
            HarnessSupply::Existing(path) => {
                wire::harness_supply::Choice::ExistingCodexPath(path.as_str().into())
            }
        }),
    }
}
fn setup_context(
    target: &ConversationTarget,
    current: Option<&chapchoo_service::ConversationSummary>,
    fresh: bool,
    require_retained_source: bool,
) -> Result<Option<wire::setup_command::Context>, ServiceError> {
    Ok(Some(match (target, current) {
        (ConversationTarget::NotStarted, Some(current))
            if matches!(current.source, ConversationSource::Ready { .. }) =>
        {
            let snapshot = current
                .source
                .snapshot()
                .ok_or_else(|| ServiceError::domain(DomainReason::InvalidTransition))?;
            let native = current
                .native_session_id
                .as_ref()
                .ok_or_else(|| ServiceError::domain(DomainReason::InvalidTransition))?;
            wire::setup_command::Context::Existing(wire::NativeTarget {
                snapshot_id: snapshot.as_str().into(),
                native_session_id: native.as_str().into(),
            })
        }
        (ConversationTarget::NotStarted, _) => {
            wire::setup_command::Context::Initial(wire::InitialNativeContext {
                allow_fresh: fresh,
                require_retained_source,
            })
        }
        (ConversationTarget::Snapshot(snapshot), _) => {
            let Some(current) = current else {
                return Ok(None);
            };
            if current.source.snapshot() != Some(snapshot) {
                return Ok(None);
            }
            let Some(native) = current.native_session_id.as_ref() else {
                return Err(ServiceError::domain(DomainReason::InvalidTransition));
            };
            wire::setup_command::Context::Existing(wire::NativeTarget {
                snapshot_id: snapshot.as_str().into(),
                native_session_id: native.as_str().into(),
            })
        }
    }))
}
impl ConversationInputProcessor for ConversationControlProcessor {
    fn process(&self, input: AcceptedConversationInput) -> ConversationInputFuture {
        let processor = self.clone();
        Box::pin(async move { processor.process_owned(input).await })
    }
}
impl chapchoo_service::ConversationAnswerProcessor for ConversationControlProcessor {
    fn answer(
        &self,
        answer: chapchoo_service::AcceptedConversationAnswer,
    ) -> chapchoo_service::ConversationAnswerFuture {
        let processor = self.clone();
        Box::pin(async move {
            if !processor.admission.load(Ordering::Acquire) {
                return Ok(());
            }
            let agent = processor
                .computers
                .agent(&answer.actor, &answer.agent_id, AcceptanceTime::System)
                .await?;
            let Some(route) = processor
                .computers
                .resolve_connector_route(&agent.computer.id)
                .await?
            else {
                return Ok(());
            };
            let grant = processor
                .conversation
                .begin_answer(
                    &answer.actor,
                    &route,
                    &ConversationOperationForwarding {
                        agent_id: &answer.agent_id,
                        conversation_id: &answer.receipt.conversation_id,
                        operation_id: &answer.receipt.id,
                        target: &answer.target,
                    },
                    AcceptanceTime::System.now()?,
                )
                .await?;
            let chapchoo_service::ConversationAnswerForwardingAdmission::Granted(grant) = grant
            else {
                return Ok(());
            };
            let command = wire::AnswerCommand {
                agent_id: answer.agent_id,
                conversation_id: answer.receipt.conversation_id,
                operation_id: answer.receipt.id,
                target: Some(wire::NativeTarget {
                    snapshot_id: answer.target.snapshot.as_str().into(),
                    native_session_id: answer.target.native_session.as_str().into(),
                }),
            };
            let _ = processor
                .invoke(
                    &answer.actor,
                    &agent.computer.id,
                    &agent.computer.current_request_id,
                    &grant.execution_id,
                    control::connector_command::Operation::Answer(command),
                )
                .await?;
            Ok(())
        })
    }
}
impl chapchoo_service::ConversationStopProcessor for ConversationControlProcessor {
    fn stop(
        &self,
        stop: chapchoo_service::AcceptedConversationStop,
    ) -> chapchoo_service::ConversationStopFuture {
        let processor = self.clone();
        Box::pin(async move {
            if !processor.admission.load(Ordering::Acquire) {
                return Ok(());
            }
            let agent = processor
                .computers
                .agent(&stop.actor, &stop.agent_id, AcceptanceTime::System)
                .await?;
            let command = wire::StopCommand {
                conversation_id: stop.receipt.conversation_id,
                operation_id: stop.receipt.id,
                target: Some(wire::WorkTarget {
                    source: Some(wire::NativeTarget {
                        snapshot_id: stop.receipt.target.source.snapshot.as_str().into(),
                        native_session_id: stop
                            .receipt
                            .target
                            .source
                            .native_session
                            .as_str()
                            .into(),
                    }),
                    work_id: stop.receipt.target.work_id,
                }),
            };
            let _ = processor
                .invoke(
                    &stop.actor,
                    &agent.computer.id,
                    &agent.computer.current_request_id,
                    &stop.receipt.execution_id,
                    control::connector_command::Operation::Stop(command),
                )
                .await?;
            Ok(())
        })
    }
}
