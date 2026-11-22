use super::*;
use captured::connector_capture_server::{ConnectorCapture, ConnectorCaptureServer};
use chapchoo_domain::NativeConversationId;
use chapchoo_harness::{ContinuationRouteLease, NativeWorkTarget};
use std::sync::Mutex;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::Response;

#[derive(Clone, Copy)]
enum Loss {
    Report,
    Begin,
    Close,
}

#[derive(Default)]
struct ControlState {
    through: Option<captured::SourcePosition>,
    reports: u8,
    begins: u8,
    closes: u8,
    slot_open: bool,
    close_connections: Vec<String>,
}

#[derive(Clone)]
struct FaultControl {
    loss: Loss,
    state: Arc<Mutex<ControlState>>,
}

macro_rules! capture_fixture {
    ($($method:ident: $request:ident -> $response:ty),* $(,)?) => {
        #[tonic::async_trait]
        impl ConnectorCapture for FaultControl {
            async fn current(&self, _: Request<captured::CurrentCaptureRequest>) -> Result<Response<captured::CurrentCaptureState>, Status> {
                let state = self.state.lock().map_err(|_| Status::internal("fixture lock"))?;
                Ok(Response::new(captured::CurrentCaptureState { conversation: Some(captured::CaptureState {
                    conversation_id: "conversation".into(),
                    status: captured::CaptureStatus::Ready as i32,
                    snapshot_id: Some("snapshot".into()),
                    native_session_id: Some("native".into()),
                    position: state.through.clone(),
                }) }))
            }

            async fn report_native_hook(&self, req: Request<captured::NativeHookReport>) -> Result<Response<captured::NativeHookOutcome>, Status> {
                let mut state = self.state.lock().map_err(|_| Status::internal("fixture lock"))?;
                match req.into_inner().event {
                    Some(captured::native_hook_report::Event::NormalEnding(ending))
                        if ending.launch_operation_id == "launch" && ending.work_id == "work" &&
                            ending.position == state.through => {}
                    _ => return Err(Status::invalid_argument("wrong ending")),
                }
                state.reports += 1;
                if state.reports == 1 {
                    state.slot_open = true;
                    if matches!(self.loss, Loss::Report) {
                        return Err(Status::unavailable("report reply unavailable after admission"));
                    }
                }
                Ok(Response::new(captured::NativeHookOutcome {
                    outcome: Some(captured::native_hook_outcome::Outcome::ContinuationOperationId("operation".into())),
                }))
            }

            async fn begin_continuation(&self, req: Request<captured::ContinuationHandoff>) -> Result<Response<captured::ContinuationGrant>, Status> {
                let mut state = self.state.lock().map_err(|_| Status::internal("fixture lock"))?;
                check_operation(&req.into_inner())?;
                state.begins += 1;
                let context = (state.begins == 1).then(|| captured::GoalContext { projection: b"[]".to_vec() });
                if state.begins == 1 && matches!(self.loss, Loss::Begin) {
                    return Err(Status::unavailable("begin reply unavailable after consuming the grant"));
                }
                Ok(Response::new(captured::ContinuationGrant { context }))
            }

            async fn close_continuation(&self, req: Request<captured::ContinuationHandoff>) -> Result<Response<()>, Status> {
                let mut state = self.state.lock().map_err(|_| Status::internal("fixture lock"))?;
                let command = req.into_inner();
                check_operation(&command)?;
                let connection = command.source.ok_or_else(|| Status::invalid_argument("missing route"))?.connection_id;
                state.close_connections.push(connection);
                state.closes += 1;
                state.slot_open = false;
                if state.closes == 1 && matches!(self.loss, Loss::Close) {
                    return Err(Status::unavailable("close reply unavailable after settlement"));
                }
                Ok(Response::new(()))
            }

            $(
                async fn $method(&self, _: Request<captured::$request>) -> Result<Response<$response>, Status> {
                    Err(Status::unimplemented(stringify!($method)))
                }
            )*
        }
    };
}

capture_fixture!(
    read_project_instructions: ProjectInstructionsRequest -> captured::ProjectInstructionEvidence,
    inspect: CaptureRequest -> captured::CaptureState,
    bind: BindCaptureRequest -> captured::CaptureState,
    append: CapturePiece -> captured::CapturePieceResult,
    complete: CompleteCaptureRequest -> captured::CaptureState,
    check_setup: SetupCheck -> (),
    report_setup: SetupReport -> (),
    pull_input: InputChunkRequest -> captured::InputChunk,
    report_input: InputReport -> captured::InputReportAcknowledgement,
    report_native: NativeStateReport -> (),
    report_model: ModelObservationReport -> (),
    reserve_model_start: ModelStartRequest -> captured::ModelStartAdmission,
    settle_model_start: ModelStartReport -> (),
    read_goal_context: GoalContextRequest -> captured::GoalContext,
    report_request: LiveRequestReport -> (),
    check_stop: StopCheck -> (),
    report_stop: StopReport -> (),
    pull_answer: AnswerChunkRequest -> captured::AnswerChunk,
    report_answer: AnswerReport -> (),
    get_authentication_binding: ExecutionAuthenticationReference -> captured::OptionalAppliedAuthentication,
    begin_authentication: BeginAuthenticationRequest -> captured::AppliedAuthentication,
    acquire_authentication: ExecutionAuthenticationReference -> captured::ExecutionAuthenticationProgress,
    read_authentication: AuthenticationReadRequest -> captured::AuthenticationPiece,
    check_authentication: AuthenticationHandoffCheck -> (),
);

fn check_operation(command: &captured::ContinuationHandoff) -> Result<(), Status> {
    if command.operation_id != "operation"
        || command.conversation_id != "conversation"
        || command
            .ending
            .as_ref()
            .is_none_or(|ending| ending.launch_operation_id != "launch")
    {
        return Err(Status::invalid_argument("wrong operation"));
    }
    Ok(())
}

struct Owner {
    ending: NativeNormalEnding,
    pending: Mutex<bool>,
    submissions: Mutex<u8>,
}

impl EndingOwner for Owner {
    async fn ending(&self) -> Result<Option<NativeNormalEnding>, Error> {
        Ok((*self.pending.lock().map_err(|_| Error::Process)?).then(|| self.ending.clone()))
    }
    async fn binding(&self) -> Result<Option<NativeAuthenticationBinding>, Error> {
        Ok(Some(NativeAuthenticationBinding {
            agent_id: "agent".into(),
            conversation_id: "conversation".into(),
            operation_id: "launch".into(),
            execution_id: "execution".into(),
            authentication_id: "authentication".into(),
            source: [0; 24],
        }))
    }
    async fn decide(&self, ending: NativeNormalEnding) -> Result<(), Error> {
        assert_eq!(ending, self.ending);
        *self.pending.lock().map_err(|_| Error::Process)? = false;
        Ok(())
    }
    async fn submit(
        &self,
        ending: NativeNormalEnding,
        text: ConversationText,
        _operation_id: String,
        models: NativeModelDispatch,
    ) -> Result<(), Error> {
        assert_eq!(ending, self.ending);
        assert!(models.route.is_open());
        assert_eq!(
            text.as_str(),
            "[System-generated input]\nkind: continuation\n\n\
             Continue active goals using the current authoritative goal context. \
             Leave goals waiting for user input on hold. \
             For Conversation Work, use the listed tasks and states; checking off \
             the tasks does not itself settle the work.\n[]"
        );
        *self.submissions.lock().map_err(|_| Error::Process)? += 1;
        Ok(())
    }
}

async fn exercise(loss: Loss) -> Result<(), AppError> {
    crate::test_support::scope(async |fixture| {
        let owner = Owner {
            ending: NativeNormalEnding {
                work: NativeWorkTarget {
                    snapshot: SnapshotId::parse("snapshot".into())?,
                    native_session: NativeConversationId::parse("native".into())?,
                    work_id: "work".into(),
                },
                position: SourcePosition::new(3, EventOccurrenceId::parse("ending".into())?)?,
            },
            pending: Mutex::new(true),
            submissions: Mutex::new(0),
        };
        let state = Arc::new(Mutex::new(ControlState::default()));
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await?;
        let endpoint = Endpoint::from_shared(format!("http://{}", listener.local_addr()?))?;
        let shutdown = fixture.shutdown();
        let server = ConnectorCaptureServer::new(FaultControl {
            loss,
            state: state.clone(),
        });
        fixture.server(async move {
            tonic::transport::Server::builder()
                .add_service(server)
                .serve_with_incoming_shutdown(TcpListenerStream::new(listener), async move {
                    let _ = shutdown.await;
                })
                .await
        });
        let channel = endpoint.clone().connect().await?;
        let key = BearerSecret::generate()?;
        let stop = CancellationToken::new();
        let strand = ContinuationRouteLease::open();
        let mut source = captured::CaptureSource {
            session_id: "session".into(),
            control_id: "control".into(),
            connection_id: "original".into(),
        };
        let ordering = super::super::super::server_tools::Ordering::default();
        let binding = owner.binding().await?.ok_or("Missing fixture binding")?;
        let _input_owner = ordering.open(binding.clone());
        let mut earlier_input = ordering.begin(&binding, "earlier-input")?;
        earlier_input.attempting(true);
        let mut delivery = Delivery::new(ordering.clone(), Default::default());
        // An exact local ending alone is never permission to report.
        delivery
            .advance(
                &owner,
                channel.clone(),
                &source,
                &key,
                &stop,
                strand.route(),
            )
            .await?;
        {
            let mut control = state.lock().map_err(|_| "fixture lock")?;
            assert_eq!(control.reports, 0);
            control.through = Some(position(&owner.ending.position));
        }
        delivery
            .advance(
                &owner,
                channel.clone(),
                &source,
                &key,
                &stop,
                strand.route(),
            )
            .await?;
        assert_eq!(state.lock().map_err(|_| "fixture lock")?.reports, 0);
        assert!(owner.ending().await?.is_some());
        earlier_input.recorded();
        let _parked_input = ordering.begin(&binding, "parked-input")?;
        let error = delivery
            .advance(&owner, channel, &source, &key, &stop, strand.route())
            .await
            .err()
            .ok_or("fixture must discard one committed reply")?;
        assert_eq!(
            error.downcast_ref::<Status>().map(Status::code),
            Some(tonic::Code::Unavailable)
        );
        assert!(owner.ending().await?.is_some());
        drop(strand);
        let successor = ContinuationRouteLease::open();
        source.connection_id = "replacement".into();
        let channel = endpoint.connect().await?;
        delivery
            .settle(&owner, channel.clone(), &source, &key)
            .await?;
        delivery
            .advance(
                &owner,
                channel.clone(),
                &source,
                &key,
                &stop,
                successor.route(),
            )
            .await?;
        delivery.settle(&owner, channel, &source, &key).await?;
        assert!(owner.ending().await?.is_none());
        let control = state.lock().map_err(|_| "fixture lock")?;
        assert!(!control.slot_open, "the order fence must be released");
        assert_eq!(
            (control.reports, control.begins, control.closes),
            match loss {
                Loss::Report => (2, 1, 1),
                Loss::Begin => (2, 2, 1),
                Loss::Close => (1, 1, 2),
            }
        );
        assert_eq!(
            control.close_connections.last().map(String::as_str),
            Some("replacement")
        );
        assert_eq!(
            *owner.submissions.lock().map_err(|_| "fixture lock")?,
            u8::from(!matches!(loss, Loss::Begin))
        );
        Ok(())
    })
    .await
}

#[tokio::test]
async fn lost_report_reply_recovers_the_single_committed_admission() -> Result<(), AppError> {
    exercise(Loss::Report).await
}

#[tokio::test]
async fn lost_begin_reply_does_not_replay_native_input() -> Result<(), AppError> {
    exercise(Loss::Begin).await
}

#[tokio::test]
async fn lost_close_reply_retries_only_settlement() -> Result<(), AppError> {
    exercise(Loss::Close).await
}
