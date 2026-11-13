//! Component-only captured observation transfer; no endpoint dispatches native work.
mod answers;
mod assembly;
mod authentication;
mod input;
mod observations;
mod setup;
mod stops;
use crate::{
    ingress::{actor, id},
    status::status,
};
use chapchoo_crypto::SecretDigest;
use chapchoo_domain::{
    ConversationSource, EventOccurrenceId, NativeConversationId, SnapshotId, SourcePosition,
};
use chapchoo_protocol::conversations::{
    self as wire,
    connector_capture_server::{ConnectorCapture, ConnectorCaptureServer},
};
use chapchoo_service::{
    ConnectorSource, ConversationService, ConversationSummary, ExecutionAuthenticationService,
    SourceBinding,
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};
use tonic::{Request, Response, Status};

#[derive(Debug, Clone)]
pub struct ConversationCaptureServices {
    service: Arc<ConversationService>,
    admission: Arc<AtomicBool>,
    assembly: Arc<assembly::Assembler>,
    authentication: Arc<ExecutionAuthenticationService>,
}
impl ConversationCaptureServices {
    pub fn new(
        service: Arc<ConversationService>,
        admission: Arc<AtomicBool>,
        authentication: Arc<ExecutionAuthenticationService>,
    ) -> Self {
        Self {
            service,
            admission,
            assembly: Arc::default(),
            authentication,
        }
    }
    pub fn server(&self) -> ConnectorCaptureServer<Self> {
        ConnectorCaptureServer::new(self.clone())
            .max_decoding_message_size(64 * 1024)
            .max_encoding_message_size(64 * 1024)
    }
    fn admit(&self) -> Result<(), Status> {
        if self.admission.load(Ordering::Acquire) {
            Ok(())
        } else {
            Err(Status::unavailable("Control is draining"))
        }
    }
}
fn source<'a>(value: &'a wire::CaptureSource, credential: &'a SecretDigest) -> ConnectorSource<'a> {
    ConnectorSource {
        session_id: &value.session_id,
        control_id: &value.control_id,
        connection_id: &value.connection_id,
        credential,
    }
}
fn snapshot(value: String) -> Result<SnapshotId, Status> {
    SnapshotId::parse(value).map_err(|_| Status::invalid_argument("Captured snapshot is invalid"))
}
fn position(value: wire::SourcePosition) -> Result<SourcePosition, Status> {
    SourcePosition::new(
        value.ordinal,
        EventOccurrenceId::parse(value.occurrence_id)
            .map_err(|_| Status::invalid_argument("Captured occurrence is invalid"))?,
    )
    .map_err(|_| Status::invalid_argument("Captured position is invalid"))
}
fn wire_position(value: &SourcePosition) -> wire::SourcePosition {
    wire::SourcePosition {
        ordinal: value.ordinal(),
        occurrence_id: value.occurrence().as_str().into(),
    }
}
fn state(value: ConversationSummary) -> wire::CaptureState {
    wire::CaptureState {
        conversation_id: value.id,
        status: match value.source {
            ConversationSource::NotStarted => wire::CaptureStatus::NotStarted,
            ConversationSource::Loading { .. } => wire::CaptureStatus::Loading,
            ConversationSource::Ready { .. } => wire::CaptureStatus::Ready,
        } as i32,
        snapshot_id: value.source.snapshot().map(|value| value.as_str().into()),
        native_session_id: value.native_session_id.map(|value| value.as_str().into()),
        position: value.source.position().map(wire_position),
    }
}
#[tonic::async_trait]
impl ConnectorCapture for ConversationCaptureServices {
    async fn get_authentication_binding(
        &self,
        request: Request<wire::ExecutionAuthenticationReference>,
    ) -> Result<Response<wire::OptionalAppliedAuthentication>, Status> {
        authentication::binding(self, request).await
    }
    async fn begin_authentication(
        &self,
        request: Request<wire::BeginAuthenticationRequest>,
    ) -> Result<Response<wire::AppliedAuthentication>, Status> {
        authentication::begin(self, request).await
    }
    async fn acquire_authentication(
        &self,
        request: Request<wire::ExecutionAuthenticationReference>,
    ) -> Result<Response<wire::ExecutionAuthenticationProgress>, Status> {
        authentication::acquire(self, request).await
    }
    async fn read_authentication(
        &self,
        request: Request<wire::AuthenticationReadRequest>,
    ) -> Result<Response<wire::AuthenticationPiece>, Status> {
        authentication::read(self, request).await
    }
    async fn check_authentication(
        &self,
        request: Request<wire::AuthenticationHandoffCheck>,
    ) -> Result<Response<()>, Status> {
        authentication::check(self, request).await
    }
    async fn pull_answer(
        &self,
        request: Request<wire::AnswerChunkRequest>,
    ) -> Result<Response<wire::AnswerChunk>, Status> {
        answers::pull(self, request).await
    }
    async fn report_answer(
        &self,
        request: Request<wire::AnswerReport>,
    ) -> Result<Response<()>, Status> {
        answers::report(self, request).await
    }
    async fn check_stop(&self, request: Request<wire::StopCheck>) -> Result<Response<()>, Status> {
        stops::check(self, request).await
    }
    async fn report_stop(
        &self,
        request: Request<wire::StopReport>,
    ) -> Result<Response<()>, Status> {
        stops::report(self, request).await
    }
    async fn report_native(
        &self,
        request: Request<wire::NativeStateReport>,
    ) -> Result<Response<()>, Status> {
        observations::native(self, request).await
    }
    async fn report_request(
        &self,
        request: Request<wire::LiveRequestReport>,
    ) -> Result<Response<()>, Status> {
        observations::request(self, request).await
    }
    async fn pull_input(
        &self,
        request: Request<wire::InputChunkRequest>,
    ) -> Result<Response<wire::InputChunk>, Status> {
        input::pull(self, request).await
    }
    async fn report_input(
        &self,
        request: Request<wire::InputReport>,
    ) -> Result<Response<wire::InputReportAcknowledgement>, Status> {
        input::report(self, request).await
    }
    async fn check_setup(
        &self,
        request: Request<wire::SetupCheck>,
    ) -> Result<Response<()>, Status> {
        setup::check(self, request).await
    }
    async fn report_setup(
        &self,
        request: Request<wire::SetupReport>,
    ) -> Result<Response<()>, Status> {
        setup::report(self, request).await
    }
    async fn current(
        &self,
        request: Request<wire::CurrentCaptureRequest>,
    ) -> Result<Response<wire::CurrentCaptureState>, Status> {
        self.admit()?;
        let credential = actor(&request)?;
        let value = request.into_inner();
        let origin = value
            .source
            .ok_or_else(|| Status::invalid_argument("Capture source is required"))?;
        let conversation = self
            .service
            .current_captured_source(&source(&origin, &credential))
            .await
            .map_err(status)?;
        Ok(Response::new(wire::CurrentCaptureState {
            conversation: conversation.map(state),
        }))
    }
    async fn inspect(
        &self,
        request: Request<wire::CaptureRequest>,
    ) -> Result<Response<wire::CaptureState>, Status> {
        self.admit()?;
        let credential = actor(&request)?;
        let value = request.into_inner();
        let origin = value
            .source
            .ok_or_else(|| Status::invalid_argument("Capture source is required"))?;
        Ok(Response::new(state(
            self.service
                .captured_source(&source(&origin, &credential), &value.conversation_id)
                .await
                .map_err(status)?,
        )))
    }
    async fn bind(
        &self,
        request: Request<wire::BindCaptureRequest>,
    ) -> Result<Response<wire::CaptureState>, Status> {
        self.admit()?;
        let credential = actor(&request)?;
        let value = request.into_inner();
        let origin = value
            .source
            .ok_or_else(|| Status::invalid_argument("Capture source is required"))?;
        let binding = SourceBinding {
            conversation_id: value.conversation_id,
            expected_previous: value
                .expected_previous_snapshot_id
                .map(snapshot)
                .transpose()?,
            snapshot: snapshot(value.snapshot_id)?,
            native_session: NativeConversationId::parse(value.native_session_id)
                .map_err(|_| Status::invalid_argument("Native conversation is invalid"))?,
        };
        Ok(Response::new(state(
            self.service
                .bind_capture(&source(&origin, &credential), &binding)
                .await
                .map_err(status)?,
        )))
    }
    async fn append(
        &self,
        request: Request<wire::CapturePiece>,
    ) -> Result<Response<wire::CapturePieceResult>, Status> {
        self.admit()?;
        let credential = actor(&request)?;
        let value = request.into_inner();
        id(&value.transfer_id)?;
        let origin = value
            .source
            .ok_or_else(|| Status::invalid_argument("Capture source is required"))?;
        let snapshot = snapshot(value.snapshot_id)?;
        let position = position(
            value
                .position
                .ok_or_else(|| Status::invalid_argument("Captured position is required"))?,
        )?;
        let current = self
            .service
            .captured_source(&source(&origin, &credential), &value.conversation_id)
            .await
            .map_err(status)?;
        if current.source.snapshot() != Some(&snapshot) {
            return Err(Status::aborted("Captured source changed"));
        }
        let progress = self
            .assembly
            .append(
                assembly::EventIdentity {
                    session: origin.session_id.clone(),
                    control: origin.control_id.clone(),
                    connection: origin.connection_id.clone(),
                    conversation: value.conversation_id,
                    snapshot,
                    position,
                    transfer: value.transfer_id,
                    total: value.total_bytes,
                },
                value.offset,
                &value.content,
                Instant::now(),
            )
            .map_err(|error| match error {
                assembly::Error::Invalid => Status::invalid_argument("Capture piece is invalid"),
                assembly::Error::Conflict => {
                    Status::aborted("Capture piece conflicts with its retained transfer")
                }
                assembly::Error::Capacity => {
                    Status::resource_exhausted("Capture assembly capacity is exhausted")
                }
                assembly::Error::Missing => Status::failed_precondition(
                    "Capture transfer is absent; restart byte transfer from the retained source",
                ),
                assembly::Error::Unavailable => {
                    Status::unavailable("Capture assembly is unavailable")
                }
            })?;
        let outcome = match progress {
            assembly::Progress::Accepted(next_offset) => {
                wire::capture_piece_result::Outcome::Accepted(wire::PieceAccepted { next_offset })
            }
            assembly::Progress::Complete(event) => {
                // Event owns its byte/slot permits through normalization and DB
                // commit. Cancelled or partial transfer never publishes a pair.
                self.service
                    .ingest(
                        &source(&origin, &credential),
                        &event.identity.conversation,
                        &event.identity.snapshot,
                        &event.identity.position,
                        &event.bytes,
                    )
                    .await
                    .map_err(status)?;
                wire::capture_piece_result::Outcome::Published(wire::PairPublished {
                    position: Some(wire_position(&event.identity.position)),
                })
            }
        };
        Ok(Response::new(wire::CapturePieceResult {
            outcome: Some(outcome),
        }))
    }
    async fn complete(
        &self,
        request: Request<wire::CompleteCaptureRequest>,
    ) -> Result<Response<wire::CaptureState>, Status> {
        self.admit()?;
        let credential = actor(&request)?;
        let value = request.into_inner();
        let origin = value
            .source
            .ok_or_else(|| Status::invalid_argument("Capture source is required"))?;
        let snapshot = snapshot(value.snapshot_id)?;
        let boundary = value.boundary.map(position).transpose()?;
        Ok(Response::new(state(
            self.service
                .complete_capture(
                    &source(&origin, &credential),
                    &value.conversation_id,
                    &snapshot,
                    boundary.as_ref(),
                )
                .await
                .map_err(status)?,
        )))
    }
}
