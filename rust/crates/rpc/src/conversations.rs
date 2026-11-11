//! Owner-only conversation observations, saved supply and exact-work Stop.
mod answers;
mod input;
mod records;
mod transfer;
use crate::{
    ingress::{actor, now},
    status::status,
};
use chapchoo_domain::{
    ConversationSource, ExistingCodexPath, HarnessSupply, NativeConversationId, SnapshotId,
};
use chapchoo_protocol::conversations::{
    self as wire,
    conversations_server::{Conversations, ConversationsServer},
};
use chapchoo_service::{
    AcceptanceTime, ConversationDeliveryTarget, ConversationForeground, ConversationInputService,
    ConversationPreparationFacts, ConversationService, ConversationStopReceipt,
    ConversationStopService, ConversationStopState, ConversationWorkTarget, SaveHarnessSupply,
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tonic::{Request, Response, Status};
#[derive(Debug, Clone)]
pub struct ConversationServices {
    conversation: Arc<ConversationService>,
    inputs: Arc<ConversationInputService>,
    answers: Arc<chapchoo_service::ConversationAnswerService>,
    stops: Arc<ConversationStopService>,
    admission: Arc<AtomicBool>,
    uploads: Arc<transfer::Assembler<input::Identity>>,
    answer_uploads: Arc<transfer::Assembler<answers::Identity>>,
    answer_readers: Arc<tokio::sync::Semaphore>,
}
impl ConversationServices {
    pub fn new(
        conversation: Arc<ConversationService>,
        inputs: Arc<ConversationInputService>,
        stops: Arc<ConversationStopService>,
        answers: Arc<chapchoo_service::ConversationAnswerService>,
        admission: Arc<AtomicBool>,
    ) -> Self {
        Self {
            conversation,
            inputs,
            stops,
            answers,
            admission,
            uploads: Arc::default(),
            answer_uploads: Arc::default(),
            answer_readers: Arc::new(tokio::sync::Semaphore::new(1)),
        }
    }
    pub fn server(&self) -> ConversationsServer<Self> {
        ConversationsServer::new(self.clone())
            .max_decoding_message_size(64 * 1024)
            .max_encoding_message_size(64 * 1024)
    }
    pub fn close(&self) {
        self.inputs.close();
        self.stops.close();
        self.answers.close();
    }
    pub async fn finish(&self) -> Result<(), chapchoo_service::ServiceError> {
        self.close();
        let (input, stop, answer) = tokio::join!(
            self.inputs.finish(),
            self.stops.finish(),
            self.answers.finish()
        );
        let mut failures = [input, stop, answer]
            .into_iter()
            .filter_map(Result::err)
            .collect::<Vec<_>>();
        match failures.len() {
            0 => Ok(()),
            1 => Err(failures.remove(0)),
            _ => Err(chapchoo_service::ServiceError::infrastructure(Failures(
                failures,
            ))),
        }
    }
    fn admit(&self) -> Result<(), Status> {
        if self.admission.load(Ordering::Acquire) {
            Ok(())
        } else {
            Err(Status::unavailable("Conversation service is draining"))
        }
    }
}
#[derive(Debug)]
struct Failures(Vec<chapchoo_service::ServiceError>);
impl std::fmt::Display for Failures {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Conversation operation processing retained multiple failures")
    }
}
impl std::error::Error for Failures {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0
            .first()
            .map(|error| error as &(dyn std::error::Error + 'static))
    }
}
pub(crate) fn supply(value: Option<wire::HarnessSupply>) -> Result<HarnessSupply, Status> {
    match value.and_then(|value| value.choice) {
        Some(wire::harness_supply::Choice::Managed(())) => Ok(HarnessSupply::Managed),
        Some(wire::harness_supply::Choice::ExistingCodexPath(path)) => Ok(HarnessSupply::Existing(
            ExistingCodexPath::parse(path)
                .map_err(|_| Status::invalid_argument("Codex executable path is invalid"))?,
        )),
        None => Err(Status::invalid_argument("Supply choice is required")),
    }
}
fn wire_supply(value: HarnessSupply) -> wire::HarnessSupply {
    wire::HarnessSupply {
        choice: Some(match value {
            HarnessSupply::Managed => wire::harness_supply::Choice::Managed(()),
            HarnessSupply::Existing(path) => {
                wire::harness_supply::Choice::ExistingCodexPath(path.as_str().into())
            }
        }),
    }
}
fn native(value: &ConversationDeliveryTarget) -> wire::NativeTarget {
    wire::NativeTarget {
        snapshot_id: value.snapshot.as_str().into(),
        native_session_id: value.native_session.as_str().into(),
    }
}
fn startup_preparation(facts: ConversationPreparationFacts) -> wire::InputPreparation {
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
}
fn work(value: Option<wire::WorkTarget>) -> Result<ConversationWorkTarget, Status> {
    let value = value.ok_or_else(|| Status::invalid_argument("Exact work target is required"))?;
    let source = value
        .source
        .ok_or_else(|| Status::invalid_argument("Source target is required"))?;
    Ok(ConversationWorkTarget {
        source: ConversationDeliveryTarget {
            snapshot: SnapshotId::parse(source.snapshot_id)
                .map_err(|_| Status::invalid_argument("Snapshot is invalid"))?,
            native_session: NativeConversationId::parse(source.native_session_id)
                .map_err(|_| Status::invalid_argument("Native session is invalid"))?,
        },
        work_id: value.work_id,
    })
}
fn stop_receipt(value: ConversationStopReceipt) -> wire::StopReceipt {
    wire::StopReceipt {
        id: value.id,
        conversation_id: value.conversation_id,
        request_id: value.request_id,
        target: Some(wire::WorkTarget {
            source: Some(native(&value.target.source)),
            work_id: value.target.work_id,
        }),
        state: match value.state {
            ConversationStopState::PossibleForwarding => wire::StopState::PossibleForwarding,
            ConversationStopState::CancelPublished => wire::StopState::CancelPublished,
            ConversationStopState::Rejected => wire::StopState::Rejected,
        } as i32,
    }
}
#[tonic::async_trait]
impl Conversations for ConversationServices {
    async fn inputs(
        &self,
        request: Request<wire::OwnerInputsRequest>,
    ) -> Result<Response<wire::RetainedInputs>, Status> {
        input::list(self, request).await
    }
    async fn read_input(
        &self,
        request: Request<wire::OwnerInputRead>,
    ) -> Result<Response<wire::FieldChunk>, Status> {
        input::read(self, request).await
    }
    async fn delete_input(
        &self,
        request: Request<wire::OwnerDeleteInput>,
    ) -> Result<Response<()>, Status> {
        input::delete(self, request).await
    }
    async fn latest_plan(
        &self,
        request: Request<wire::RecordSource>,
    ) -> Result<Response<wire::RecordRegion>, Status> {
        records::latest_plan(self, request).await
    }
    async fn request_answer(
        &self,
        request: Request<wire::OwnerRequestAnswer>,
    ) -> Result<Response<wire::OwnerAnswerStatus>, Status> {
        answers::historical(self, request).await
    }
    async fn read_answer(
        &self,
        request: Request<wire::OwnerAnswerRead>,
    ) -> Result<Response<wire::FieldChunk>, Status> {
        answers::read(self, request).await
    }
    async fn live_requests(
        &self,
        request: Request<wire::OwnerLiveRequests>,
    ) -> Result<Response<wire::LiveRequestList>, Status> {
        answers::live(self, request).await
    }
    async fn submit_answer(
        &self,
        request: Request<wire::OwnerAnswerPiece>,
    ) -> Result<Response<wire::OwnerAnswerProgress>, Status> {
        answers::submit(self, request).await
    }
    async fn answer_result(
        &self,
        request: Request<wire::OwnerAnswerResultRequest>,
    ) -> Result<Response<wire::OwnerAnswerStatus>, Status> {
        answers::result(self, request).await
    }
    async fn records(
        &self,
        request: Request<wire::OwnerRecordsRequest>,
    ) -> Result<Response<wire::RecordRegion>, Status> {
        records::region(self, request).await
    }
    async fn fields(
        &self,
        request: Request<wire::OwnerFieldsRequest>,
    ) -> Result<Response<wire::RecordFields>, Status> {
        records::fields(self, request).await
    }
    async fn read_field(
        &self,
        request: Request<wire::OwnerFieldRead>,
    ) -> Result<Response<wire::FieldChunk>, Status> {
        records::read(self, request).await
    }
    async fn submit_input(
        &self,
        request: Request<wire::OwnerInputPiece>,
    ) -> Result<Response<wire::OwnerInputProgress>, Status> {
        input::submit(self, request).await
    }
    async fn input_result(
        &self,
        request: Request<wire::OwnerInputResultRequest>,
    ) -> Result<Response<wire::OwnerInputStatus>, Status> {
        input::result(self, request).await
    }
    async fn current(
        &self,
        request: Request<wire::OwnerConversationRequest>,
    ) -> Result<Response<wire::OwnerConversation>, Status> {
        self.admit()?;
        let actor = actor(&request)?;
        let value = request.into_inner();
        let now = now()?;
        let saved_authentication_id = self
            .conversation
            .saved_authentication_id(&actor, &value.agent_id, now)
            .await
            .map_err(status)?;
        let Some(current) = self
            .conversation
            .owner_conversation(&actor, &value.agent_id, now)
            .await
            .map_err(status)?
        else {
            return Ok(Response::new(wire::OwnerConversation {
                saved_authentication_id,
                ..Default::default()
            }));
        };
        let startup = self
            .conversation
            .connection_startup(&actor, &value.agent_id, now)
            .await
            .map_err(status)?
            .map(|observed| startup_preparation(observed.facts));
        let observed = self
            .conversation
            .native_observation(&actor, &value.agent_id, &current.id, now)
            .await
            .map_err(status)?;
        if observed.as_ref().is_some_and(|observed| {
            current.source.snapshot() != Some(&observed.target.snapshot)
                || current.native_session_id.as_ref() != Some(&observed.target.native_session)
        }) {
            return Err(Status::aborted(
                "Conversation source changed during observation read",
            ));
        }
        let observed_native = observed.map(|observed| wire::ObservedNative {
            target: Some(native(&observed.target)),
            position: Some(wire::SourcePosition {
                ordinal: observed.position.ordinal(),
                occurrence_id: observed.position.occurrence().as_str().into(),
            }),
            foreground: match observed.foreground {
                ConversationForeground::Idle => wire::Foreground::Idle,
                ConversationForeground::Running => wire::Foreground::Running,
                ConversationForeground::RequiresAction => wire::Foreground::RequiresAction,
                ConversationForeground::OtherActive => wire::Foreground::OtherActive,
            } as i32,
            work_id: observed.work_id,
            preparation: observed.preparation.map(|value| wire::NativePreparation {
                freshness: match value.freshness {
                    chapchoo_service::ConversationPreparationFreshness::Current => {
                        wire::PreparationFreshness::Current
                    }
                    chapchoo_service::ConversationPreparationFreshness::TransientLookupFailure => {
                        wire::PreparationFreshness::TransientLookupFailure
                    }
                } as i32,
                node: value.node,
                adapter: value.adapter,
                codex: value.codex,
            }),
            authentication_id: observed.authentication_id,
            applied_supply: observed.applied_supply.map(wire_supply),
        });
        let conversation = wire::CaptureState {
            conversation_id: current.id,
            status: match current.source {
                ConversationSource::NotStarted => wire::CaptureStatus::NotStarted,
                ConversationSource::Loading { .. } => wire::CaptureStatus::Loading,
                ConversationSource::Ready { .. } => wire::CaptureStatus::Ready,
            } as i32,
            snapshot_id: current
                .source
                .snapshot()
                .map(|snapshot| snapshot.as_str().into()),
            native_session_id: current
                .native_session_id
                .map(|native| native.as_str().into()),
            position: current
                .source
                .position()
                .map(|position| wire::SourcePosition {
                    ordinal: position.ordinal(),
                    occurrence_id: position.occurrence().as_str().into(),
                }),
        };
        Ok(Response::new(wire::OwnerConversation {
            conversation: Some(conversation),
            observed_native,
            saved_authentication_id,
            startup,
        }))
    }
    async fn get_supply(
        &self,
        request: Request<wire::OwnerConversationRequest>,
    ) -> Result<Response<wire::SavedSupply>, Status> {
        self.admit()?;
        let actor = actor(&request)?;
        let value = request.into_inner();
        let saved = self
            .conversation
            .saved_supply(&actor, &value.agent_id, now()?)
            .await
            .map_err(status)?;
        Ok(Response::new(wire::SavedSupply {
            supply: Some(wire_supply(saved.supply)),
            request_id: saved.saved.map(|saved| saved.request_id),
        }))
    }
    async fn save_supply(
        &self,
        request: Request<wire::SaveSupplyRequest>,
    ) -> Result<Response<wire::SavedSupply>, Status> {
        self.admit()?;
        let actor = actor(&request)?;
        let value = request.into_inner();
        self.conversation
            .save_supply(
                &actor,
                &SaveHarnessSupply {
                    agent_id: value.agent_id.clone(),
                    request_id: value.request_id,
                    expected_request_id: value.expected_request_id,
                    supply: supply(value.supply)?,
                },
                now()?,
            )
            .await
            .map_err(status)?;
        let saved = self
            .conversation
            .saved_supply(&actor, &value.agent_id, now()?)
            .await
            .map_err(status)?;
        Ok(Response::new(wire::SavedSupply {
            supply: Some(wire_supply(saved.supply)),
            request_id: saved.saved.map(|saved| saved.request_id),
        }))
    }
    async fn stop(
        &self,
        request: Request<wire::OwnerStopRequest>,
    ) -> Result<Response<wire::StopReceipt>, Status> {
        self.admit()?;
        let actor = actor(&request)?;
        let value = request.into_inner();
        let receipt = self
            .stops
            .accept(
                actor,
                value.agent_id,
                value.conversation_id,
                value.request_id,
                work(value.target)?,
                AcceptanceTime::System,
            )
            .await
            .map_err(status)?;
        Ok(Response::new(stop_receipt(receipt)))
    }
    async fn stop_result(
        &self,
        request: Request<wire::OwnerStopResultRequest>,
    ) -> Result<Response<wire::OptionalStopReceipt>, Status> {
        self.admit()?;
        let actor = actor(&request)?;
        let value = request.into_inner();
        let receipt = self
            .stops
            .current(
                &actor,
                &value.agent_id,
                &value.conversation_id,
                &value.request_id,
                AcceptanceTime::System,
            )
            .await
            .map_err(status)?;
        Ok(Response::new(wire::OptionalStopReceipt {
            receipt: receipt.map(stop_receipt),
        }))
    }
}
