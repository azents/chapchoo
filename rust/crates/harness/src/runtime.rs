use crate::{AppliedSupply, Error, NativeStorage, Preparer};
use crate::{NativeAuthenticationBinding, NativeAuthenticationProvider, NativeCredentials};
use agent_client_protocol::{
    Agent, Client, Lines,
    schema::ProtocolVersion,
    v2::{ConnectionTo, Responder, schema},
};
use chapchoo_domain::{ConversationText, HarnessSupply, NativeConversationId};
use futures_util::{SinkExt, StreamExt};
use nix::{
    errno::Errno,
    sys::{
        signal::{Signal, kill},
        wait::{WaitPidFlag, WaitStatus, waitpid},
    },
    unistd::{Pid, getgid, getpid, getuid},
};
use serde::Deserialize;
use std::{collections::BTreeMap, fmt, fs, process::Stdio, sync::Arc, time::Duration};
use tokio::{
    process::Child,
    sync::{Mutex, OwnedSemaphorePermit, Semaphore, mpsc, oneshot},
    task::JoinHandle,
};
use tokio_util::{
    codec::{FramedRead, FramedWrite, LinesCodec},
    sync::CancellationToken,
};
mod answers;
mod authentication;
mod reported;
pub use reported::NativeReportedOperation;
mod inputs;
use answers::AnswerRegistry;
pub use answers::{NativeAnswerObservation, NativeAnswerOperation};
mod activity;
mod app_server;
mod continuation_route;
pub use continuation_route::{ContinuationRoute, ContinuationRouteLease};
mod goal_restore;
mod model_selection;
pub use model_selection::{
    NativeModelAuthority, NativeModelChoice, NativeModelDispatch, NativeModelFuture,
    NativeModelObservation, NativeModelOptions, NativeModelReservation, NativeModelStart,
    NativeModelStartOutcome,
};
mod observations;
mod protocol;
mod requests;
mod session;
use inputs::InputRegistry;
pub use inputs::{NativeInputObservation, NativeInputOperation, NativeTextTarget};
mod setup;
mod steering;
mod stops;
use observations::Observations;
pub use observations::{NativeForeground, NativeNormalEnding, NativeObservation, NativeWorkTarget};
use session::Session;
use setup::SetupRegistry;
pub use setup::{
    NativeSetupFailure, NativeSetupIntent, NativeSetupObservation, NativeSetupOperation,
};
pub use stops::NativeStopObservation;
use stops::StopRegistry;

const MAX_LINE: usize = 16 * 1024 * 1024;
const MAX_REQUESTS: usize = 32;
type Recorder = Arc<dyn Fn(&str) -> Result<(), Error> + Send + Sync>;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureReplay {
    FromStart,
    AlreadyCaptured,
}
#[derive(Debug, Clone)]
pub enum NativeContext {
    New,
    Resume {
        session: NativeConversationId,
        replay: CaptureReplay,
    },
}

/// Actual verified namespace-init substrate, not a provider intention.
#[derive(Debug)]
pub struct ScopeEvidence {
    _private: (),
}
impl ScopeEvidence {
    pub fn verify(storage: &NativeStorage) -> Result<Self, Error> {
        if std::env::consts::OS != "linux"
            || getpid().as_raw() != 1
            || getuid().as_raw() != 10001
            || getgid().as_raw() != 10001
        {
            return Err(Error::UnsupportedRuntime);
        }
        let status =
            fs::read_to_string("/proc/self/status").map_err(|_| Error::UnsupportedRuntime)?;
        if !status.lines().any(|l| l == "CapEff:\t0000000000000000")
            || !status.lines().any(|l| l == "NoNewPrivs:\t1")
            || !status.lines().any(|l| l == "Seccomp:\t2")
            || !status.lines().any(|l| {
                l.strip_prefix("NSpid:")
                    .is_some_and(|v| v.split_whitespace().last() == Some("1"))
            })
        {
            return Err(Error::UnsupportedRuntime);
        }
        let mounts =
            fs::read_to_string("/proc/self/mountinfo").map_err(|_| Error::UnsupportedRuntime)?;
        let root_readonly = mounts.lines().any(|line| {
            let fields: Vec<_> = line.split_whitespace().collect();
            fields.get(4) == Some(&"/")
                && fields
                    .get(5)
                    .is_some_and(|v| v.split(',').any(|o| o == "ro"))
        });
        let saved_writable = mounts.lines().any(|line| {
            let fields: Vec<_> = line.split_whitespace().collect();
            fields
                .get(4)
                .is_some_and(|v| std::path::Path::new(v) == storage.saved_root())
                && fields
                    .get(5)
                    .is_some_and(|v| v.split(',').any(|o| o == "rw"))
        });
        if !root_readonly || !saved_writable {
            return Err(Error::UnsupportedRuntime);
        }
        Ok(Self { _private: () })
    }
    /// Close the verified private process scope and reap every descendant.
    pub async fn retire(self) -> Result<Retirement, Error> {
        retire_scope().await.map(|_| Retirement::Quiescent)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Retirement {
    NoNativeStarted,
    Quiescent,
}
/// The exact native insertion identity; not a work-completion claim.
#[derive(Clone, PartialEq, Eq)]
pub enum NativePromptAcceptance {
    Inserted { native_message_id: String },
    Steered,
    CommandAccepted,
    Failed,
}
impl fmt::Debug for NativePromptAcceptance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("NativePromptAcceptance([REDACTED])")
    }
}

/// A pending responder remains owned independently of a server Control stream.
#[derive(Clone)]
pub enum InteractiveRequest {
    Permission {
        id: String,
        occurrence: chapchoo_domain::SourcePosition,
        request: schema::RequestPermissionRequest,
    },
    Form {
        id: String,
        occurrence: chapchoo_domain::SourcePosition,
        request: schema::CreateElicitationRequest,
    },
}
impl fmt::Debug for InteractiveRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("InteractiveRequest([REDACTED])")
    }
}
#[derive(Clone)]
pub enum InteractiveResponse {
    Permission {
        id: String,
        response: schema::RequestPermissionResponse,
    },
    Form {
        id: String,
        response: schema::CreateElicitationResponse,
    },
}
impl fmt::Debug for InteractiveResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("InteractiveResponse([REDACTED])")
    }
}
enum Pending {
    Permission(
        schema::RequestPermissionRequest,
        Responder<schema::RequestPermissionResponse>,
    ),
    Form(
        schema::CreateElicitationRequest,
        Responder<schema::CreateElicitationResponse>,
    ),
}
impl Pending {
    fn cancelled(&self) -> bool {
        match self {
            Self::Permission(_, r) => r.cancellation().is_cancelled(),
            Self::Form(_, r) => r.cancellation().is_cancelled(),
        }
    }
}
type Requests = Arc<Mutex<BTreeMap<String, (chapchoo_domain::SourcePosition, Pending)>>>;
enum Command {
    Preparation(oneshot::Sender<Option<crate::NativePreparationObservation>>),
    AuthenticationBinding(oneshot::Sender<Result<Option<NativeAuthenticationBinding>, Error>>),
    PublishAuthentication {
        expected: NativeAuthenticationBinding,
        update: crate::NativeAuthentication,
        checker: Arc<dyn NativeAuthenticationProvider>,
        _permit: OwnedSemaphorePermit,
        result: oneshot::Sender<Result<(), Error>>,
    },
    Reported(NativeReportedOperation, oneshot::Sender<Result<(), Error>>),
    Answer(
        NativeAnswerOperation,
        OwnedSemaphorePermit,
        oneshot::Sender<Result<NativeAnswerObservation, Error>>,
    ),
    StopOperation(
        String,
        NativeWorkTarget,
        oneshot::Sender<Result<NativeStopObservation, Error>>,
    ),
    Setup(
        NativeSetupOperation,
        Arc<dyn NativeAuthenticationProvider>,
        Arc<dyn crate::NativeToolsProvider>,
        oneshot::Sender<Result<NativeSetupObservation, Error>>,
    ),
    SetupObservation(
        String,
        oneshot::Sender<Result<NativeSetupObservation, Error>>,
    ),
    Prompt(
        NativeInputOperation,
        Option<NativeAuthenticationBinding>,
        Option<NativeModelDispatch>,
        OwnedSemaphorePermit,
        oneshot::Sender<Result<NativePromptAcceptance, Error>>,
    ),
    Continuation(
        NativeNormalEnding,
        ConversationText,
        String,
        NativeModelDispatch,
        OwnedSemaphorePermit,
        oneshot::Sender<Result<NativePromptAcceptance, Error>>,
    ),
    Stop(NativeWorkTarget, oneshot::Sender<Result<(), Error>>),
    Observe(oneshot::Sender<Result<NativeObservation, Error>>),
    NormalEnding(oneshot::Sender<Result<Option<NativeNormalEnding>, Error>>),
    PendingEnding(oneshot::Sender<Result<Option<NativeNormalEnding>, Error>>),
    DecideEnding(NativeNormalEnding, oneshot::Sender<Result<(), Error>>),
    CapturedSource(oneshot::Sender<Result<Option<crate::CaptureLog>, Error>>),
    InputObservation {
        id: String,
        target: NativeTextTarget,
        result: oneshot::Sender<Result<NativeInputObservation, Error>>,
    },
    Requests(oneshot::Sender<Vec<InteractiveRequest>>),
    Respond(InteractiveResponse, oneshot::Sender<Result<(), Error>>),
}
/// Narrow read authority for the current retained copy; it cannot start native work.
#[derive(Clone)]
pub struct NativeCaptureReader {
    sender: mpsc::Sender<Command>,
}
impl fmt::Debug for NativeCaptureReader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("NativeCaptureReader([REDACTED])")
    }
}
impl NativeCaptureReader {
    pub async fn current_source(&self) -> Result<Option<crate::CaptureLog>, Error> {
        let (result, rx) = oneshot::channel();
        self.sender
            .try_send(Command::CapturedSource(result))
            .map_err(|_| Error::NotAdmitted)?;
        rx.await.map_err(|_| Error::Process)?
    }
}
/// Cloneable operation authority, without namespace-retirement ownership.
#[derive(Clone)]
pub struct NativeHandle {
    sender: mpsc::Sender<Command>,
    payload_budget: Arc<Semaphore>,
}
impl fmt::Debug for NativeHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("NativeHandle([REDACTED])")
    }
}
/// The process retains this sole physical lifetime owner while operation handles
/// are held by stream-independent workers.
pub struct NativeOwner {
    sender: mpsc::Sender<Command>,
    task: JoinHandle<Result<Retirement, Error>>,
    shutdown: CancellationToken,
    payload_budget: Arc<Semaphore>,
}
impl fmt::Debug for NativeOwner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("NativeOwner([REDACTED])")
    }
}
impl NativeOwner {
    pub fn new(storage: NativeStorage) -> Result<Self, Error> {
        let preparer = Preparer::new(storage.clone())?;
        let (sender, mut receiver) = mpsc::channel(32);
        let shutdown = CancellationToken::new();
        let stopping = shutdown.clone();
        let task = tokio::spawn(async move {
            let mut state = State {
                storage,
                preparer,
                live: None,
                scope: None,
                blocked: false,
                inputs: InputRegistry::default(),
                retained_capture: None,
                setups: SetupRegistry::default(),
                stops: StopRegistry::default(),
                answers: AnswerRegistry::default(),
                credential_write: None,
                project_write: None,
                pending_tools: None,
                cleanup_errors: Vec::new(),
            };
            let mut deferred = Vec::<DeferredSetup>::new();
            let mut cadence = tokio::time::interval(Duration::from_millis(100));
            cadence.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                let native_ended = state.live.as_ref().map(|live| live.ended.clone());
                let command = tokio::select! {
                    biased;
                    _ = stopping.cancelled() => break,
                    _=async {
                        match native_ended {
                            Some(ended)=>ended.cancelled().await,
                            None=>std::future::pending::<()>().await,
                        }
                    }=>{
                        // Reuse the owned physical shutdown path after a
                        // protocol/capture failure; never leave unrecorded
                        // native work running until a later Control command.
                        state.retire().await?;
                        continue;
                    },
                    command = receiver.recv() => match command { Some(c)=>c, None=>break },
                    _ = cadence.tick(), if !deferred.is_empty() => {
                        let expired = deferred[0].deadline <= tokio::time::Instant::now();
                        if expired || state.refresh_safe() {
                            let pending = deferred.remove(0);
                            let outcome = if expired {
                                state.setups.expire_unstarted(&pending.operation)
                            } else {
                                state.setup(pending.operation, pending.authentication, pending.tools).await
                            };
                            let _ = pending.result.send(outcome);
                        }
                        continue;
                    },
                };
                match command {
                    Command::Preparation(result) => {
                        let observation =
                            state
                                .live
                                .as_ref()
                                .map(|live| crate::NativePreparationObservation {
                                    freshness: live._supply.freshness,
                                    versions: live._supply.versions.clone(),
                                });
                        let _ = result.send(observation);
                    }
                    Command::AuthenticationBinding(result) => {
                        let _ = result.send(state.current_authentication());
                    }
                    Command::PublishAuthentication {
                        expected,
                        update,
                        checker,
                        _permit,
                        result,
                    } => {
                        let outcome = tokio::select! {
                            _=stopping.cancelled()=>Err(Error::DeliveryUncertain),
                            result=state.publish_authentication(expected,update,checker)=>result,
                        };
                        let _ = result.send(outcome);
                    }
                    Command::StopOperation(id, target, result) => {
                        let _ = result.send(state.stop_operation(&id, &target));
                    }
                    Command::Setup(operation, authentication, tools, result) => {
                        if let Some(pending) = deferred.iter().find(|pending| {
                            pending.operation.operation_id == operation.operation_id
                        }) {
                            let observed = if pending.operation == operation {
                                Ok(NativeSetupObservation::PossibleForwarding)
                            } else {
                                Err(Error::OperationConflict)
                            };
                            let _ = result.send(observed);
                            continue;
                        }
                        if state.needs_refresh(&operation) && !state.refresh_safe() {
                            if deferred.len() >= 32 {
                                let _ = result.send(Err(Error::NotAdmitted));
                            } else {
                                deferred.push(DeferredSetup {
                                    operation,
                                    authentication,
                                    tools,
                                    result,
                                    deadline: tokio::time::Instant::now()
                                        + Duration::from_secs(300),
                                });
                            }
                            continue;
                        }
                        let outcome = tokio::select! {
                            _=stopping.cancelled()=>Err(Error::DeliveryUncertain),
                            outcome=state.setup(operation,authentication,tools)=>outcome,
                        };
                        let _ = result.send(outcome);
                    }
                    Command::SetupObservation(id, result) => {
                        let observed = if deferred
                            .iter()
                            .any(|pending| pending.operation.operation_id == id)
                        {
                            Ok(NativeSetupObservation::PossibleForwarding)
                        } else {
                            state.setups.observe(&id)
                        };
                        let _ = result.send(observed);
                    }
                    Command::Prompt(operation, expected, models, _permit, result) => {
                        if let Some(expected) = expected.as_ref() {
                            match state.current_authentication() {
                                Ok(Some(current))
                                    if authentication::same_application(&current, expected) => {}
                                Ok(_) => {
                                    let _ = result.send(Err(Error::NotAdmitted));
                                    continue;
                                }
                                Err(error) => {
                                    let _ = result.send(Err(error));
                                    continue;
                                }
                            }
                        }
                        let outcome = tokio::select! {
                            _=stopping.cancelled()=>Err(Error::DeliveryUncertain),
                            outcome=state.input(operation,expected.is_some(),models.as_ref())=>outcome,
                        };
                        let _ = result.send(outcome);
                    }
                    Command::Continuation(ending, text, operation, models, _permit, result) => {
                        let outcome = tokio::select! {
                            _=stopping.cancelled()=>Err(Error::DeliveryUncertain),
                            outcome=state.continue_after_ending(ending,text,operation,&models)=>outcome,
                        };
                        let _ = result.send(outcome);
                    }
                    Command::Stop(target, result) => {
                        let _ = result.send(state.stop(&target));
                    }
                    Command::Observe(result) => {
                        let _ = result.send(state.observe());
                    }
                    Command::NormalEnding(result) => {
                        let _ = result.send(state.normal_ending());
                    }
                    Command::PendingEnding(result) => {
                        let _ = result.send(
                            state
                                .live
                                .as_ref()
                                .filter(|live| !state.blocked && !live.protocol.is_finished())
                                .map_or(Ok(None), |live| live.observations.pending_ending()),
                        );
                    }
                    Command::DecideEnding(ending, result) => {
                        let _ = result.send(
                            state
                                .live
                                .as_ref()
                                .map_or(Ok(()), |live| live.observations.decide_ending(&ending)),
                        );
                    }
                    Command::CapturedSource(result) => {
                        let outcome = state.captured_source().await;
                        let _ = result.send(outcome);
                    }
                    Command::InputObservation { id, target, result } => {
                        let _ = result.send(state.inputs.observe(&id, &target));
                    }
                    Command::Requests(result) => {
                        let mut visible = Vec::new();
                        if let Some(live) = &state.live
                            && !state.blocked
                            && !live.protocol.is_finished()
                        {
                            if let Some(Session::AppServer(thread)) = &live.session {
                                let _ = result.send(thread.pending().unwrap_or_default());
                                continue;
                            }
                            let mut pending = live.requests.lock().await;
                            pending.retain(|_, (_, r)| !r.cancelled());
                            for (id, (occurrence, r)) in pending.iter() {
                                visible.push(match r {
                                    Pending::Permission(request, _) => {
                                        InteractiveRequest::Permission {
                                            id: id.clone(),
                                            occurrence: occurrence.clone(),
                                            request: request.clone(),
                                        }
                                    }
                                    Pending::Form(request, _) => InteractiveRequest::Form {
                                        id: id.clone(),
                                        occurrence: occurrence.clone(),
                                        request: request.clone(),
                                    },
                                });
                            }
                        }
                        let _ = result.send(visible);
                    }
                    Command::Reported(reported, result) => {
                        let _ = result.send(state.release_reported(reported));
                    }
                    Command::Answer(operation, _permit, result) => {
                        let _ = result.send(state.answer(operation).await);
                    }
                    Command::Respond(response, result) => {
                        let _ = result.send(state.respond(response).await);
                    }
                }
            }
            receiver.close();
            state.retire().await
        });
        Ok(Self {
            sender,
            task,
            shutdown,
            payload_budget: Arc::new(Semaphore::new(32 * 1024 * 1024)),
        })
    }
    pub fn handle(&self) -> NativeHandle {
        NativeHandle {
            sender: self.sender.clone(),
            payload_budget: self.payload_budget.clone(),
        }
    }
    pub async fn preparation(&self) -> Result<Option<crate::NativePreparationObservation>, Error> {
        self.handle().preparation().await
    }
    pub async fn prompt(
        &self,
        operation: NativeInputOperation,
    ) -> Result<NativePromptAcceptance, Error> {
        self.handle().prompt(operation).await
    }
    pub async fn stop_work(&self, target: NativeWorkTarget) -> Result<(), Error> {
        self.handle().stop_work(target).await
    }
    pub fn capture_reader(&self) -> NativeCaptureReader {
        self.handle().capture_reader()
    }
    pub async fn observe(&self) -> Result<NativeObservation, Error> {
        self.handle().observe().await
    }
    pub async fn normal_ending(&self) -> Result<Option<NativeNormalEnding>, Error> {
        self.handle().normal_ending().await
    }
    pub async fn input_observation(
        &self,
        id: String,
        target: NativeTextTarget,
    ) -> Result<NativeInputObservation, Error> {
        self.handle().input_observation(id, target).await
    }
    pub async fn pending_requests(&self) -> Result<Vec<InteractiveRequest>, Error> {
        self.handle().pending_requests().await
    }
    pub async fn respond(&self, response: InteractiveResponse) -> Result<(), Error> {
        self.handle().respond(response).await
    }
    pub async fn finish(self) -> Result<Retirement, Error> {
        self.shutdown.cancel();
        self.task.await.map_err(|_| Error::RetirementUnknown)?
    }
}
impl NativeHandle {
    /// Read the physically started combination without preparing or refreshing it.
    pub async fn preparation(&self) -> Result<Option<crate::NativePreparationObservation>, Error> {
        let (result, rx) = oneshot::channel();
        self.sender
            .try_send(Command::Preparation(result))
            .map_err(|_| Error::NotAdmitted)?;
        rx.await.map_err(|_| Error::Process)
    }
    /// Acknowledges native prompt acceptance only, never completion or consumption.
    pub async fn prompt(
        &self,
        operation: NativeInputOperation,
    ) -> Result<NativePromptAcceptance, Error> {
        self.submit_input(operation, None, None).await
    }
    /// Bind dispatch to the launch whose tool publication fence was installed.
    pub async fn prompt_for_owner(
        &self,
        operation: NativeInputOperation,
        owner: NativeAuthenticationBinding,
        models: NativeModelDispatch,
    ) -> Result<NativePromptAcceptance, Error> {
        self.submit_input(operation, Some(owner), Some(models))
            .await
    }
    /// Consume a caller-authorized continuation at its exact live ending. A
    /// lost acceptance response cannot make that ending usable again.
    pub async fn continue_after_ending(
        &self,
        ending: NativeNormalEnding,
        text: ConversationText,
        operation_id: String,
        models: NativeModelDispatch,
    ) -> Result<NativePromptAcceptance, Error> {
        let bytes = u32::try_from(text.as_str().len()).map_err(|_| Error::NotAdmitted)?;
        if text.as_str().len() > MAX_LINE {
            return Err(Error::NotAdmitted);
        }
        let permit = self
            .payload_budget
            .clone()
            .try_acquire_many_owned(bytes)
            .map_err(|_| Error::NotAdmitted)?;
        let (result, rx) = oneshot::channel();
        self.sender
            .try_send(Command::Continuation(
                ending,
                text,
                operation_id,
                models,
                permit,
                result,
            ))
            .map_err(|_| Error::NotAdmitted)?;
        rx.await.map_err(|_| Error::Process)?
    }
    async fn submit_input(
        &self,
        operation: NativeInputOperation,
        owner: Option<NativeAuthenticationBinding>,
        models: Option<NativeModelDispatch>,
    ) -> Result<NativePromptAcceptance, Error> {
        operation.validate()?;
        let bytes = u32::try_from(operation.text.as_str().len()).map_err(|_| Error::NotAdmitted)?;
        let permit = self
            .payload_budget
            .clone()
            .try_acquire_many_owned(bytes)
            .map_err(|_| Error::NotAdmitted)?;
        let (result, rx) = oneshot::channel();
        self.sender
            .try_send(Command::Prompt(operation, owner, models, permit, result))
            .map_err(|_| Error::NotAdmitted)?;
        rx.await.map_err(|_| Error::Process)?
    }
    pub async fn stop_work(&self, target: NativeWorkTarget) -> Result<(), Error> {
        let (result, rx) = oneshot::channel();
        self.sender
            .try_send(Command::Stop(target, result))
            .map_err(|_| Error::NotAdmitted)?;
        rx.await.map_err(|_| Error::Process)?
    }
    pub fn capture_reader(&self) -> NativeCaptureReader {
        NativeCaptureReader {
            sender: self.sender.clone(),
        }
    }
    pub async fn observe(&self) -> Result<NativeObservation, Error> {
        let (result, rx) = oneshot::channel();
        self.sender
            .try_send(Command::Observe(result))
            .map_err(|_| Error::NotAdmitted)?;
        rx.await.map_err(|_| Error::Process)?
    }
    /// Read the latest qualified occurrence while its live owner is still current.
    pub async fn normal_ending(&self) -> Result<Option<NativeNormalEnding>, Error> {
        let (result, rx) = oneshot::channel();
        self.sender
            .try_send(Command::NormalEnding(result))
            .map_err(|_| Error::NotAdmitted)?;
        rx.await.map_err(|_| Error::Process)?
    }
    /// A live ending awaiting its owner's decision, never reconstructed from capture.
    pub async fn pending_ending(&self) -> Result<Option<NativeNormalEnding>, Error> {
        let (result, rx) = oneshot::channel();
        self.sender
            .try_send(Command::PendingEnding(result))
            .map_err(|_| Error::NotAdmitted)?;
        rx.await.map_err(|_| Error::Process)?
    }
    pub async fn decide_ending(&self, ending: NativeNormalEnding) -> Result<(), Error> {
        let (result, rx) = oneshot::channel();
        self.sender
            .try_send(Command::DecideEnding(ending, result))
            .map_err(|_| Error::NotAdmitted)?;
        rx.await.map_err(|_| Error::Process)?
    }
    pub async fn input_observation(
        &self,
        id: String,
        target: NativeTextTarget,
    ) -> Result<NativeInputObservation, Error> {
        if id.is_empty() || id.len() > 128 || !id.bytes().all(|b| b.is_ascii_graphic()) {
            return Err(Error::NotAdmitted);
        }
        let (result, rx) = oneshot::channel();
        self.sender
            .try_send(Command::InputObservation { id, target, result })
            .map_err(|_| Error::NotAdmitted)?;
        rx.await.map_err(|_| Error::Process)?
    }
    pub async fn pending_requests(&self) -> Result<Vec<InteractiveRequest>, Error> {
        let (result, rx) = oneshot::channel();
        self.sender
            .try_send(Command::Requests(result))
            .map_err(|_| Error::NotAdmitted)?;
        rx.await.map_err(|_| Error::Process)
    }
    pub async fn respond(&self, response: InteractiveResponse) -> Result<(), Error> {
        let (result, rx) = oneshot::channel();
        self.sender
            .try_send(Command::Respond(response, result))
            .map_err(|_| Error::NotAdmitted)?;
        rx.await.map_err(|_| Error::Process)?
    }
}
struct State {
    storage: NativeStorage,
    preparer: Preparer,
    live: Option<Live>,
    scope: Option<ScopeEvidence>,
    blocked: bool,
    inputs: InputRegistry,
    retained_capture: Option<crate::CaptureLog>,
    setups: SetupRegistry,
    stops: StopRegistry,
    answers: AnswerRegistry,
    credential_write: Option<JoinHandle<Result<NativeCredentials, Error>>>,
    project_write: Option<JoinHandle<Result<(), Error>>>,
    pending_tools: Option<crate::NativeToolProvision>,
    cleanup_errors: Vec<Error>,
}
struct DeferredSetup {
    operation: NativeSetupOperation,
    authentication: Arc<dyn NativeAuthenticationProvider>,
    tools: Arc<dyn crate::NativeToolsProvider>,
    result: oneshot::Sender<Result<NativeSetupObservation, Error>>,
    deadline: tokio::time::Instant,
}
#[derive(Clone, Copy)]
enum Transport {
    CodexAppServer,
    // Retained for an ACP harness composition; it is not a Codex supply.
    #[allow(dead_code)]
    Acp,
}
impl Transport {
    fn for_supply(supply: &HarnessSupply) -> Self {
        match supply {
            HarnessSupply::Managed | HarnessSupply::Existing(_) => Self::CodexAppServer,
        }
    }
}
enum PendingSession {
    Acp(oneshot::Receiver<Result<agent_client_protocol::v2::ActiveSession<Agent>, Error>>),
    AppServer(app_server::Connection, NativeCredentials),
}
struct Live {
    child: Child,
    protocol: JoinHandle<Result<(), Error>>,
    stderr: JoinHandle<Result<(), std::io::Error>>,
    done: CancellationToken,
    ended: CancellationToken,
    restore: Option<goal_restore::Listener>,
    session: Option<Session>,
    requests: Requests,
    _supply: AppliedSupply,
    requested_supply: HarnessSupply,
    _launch: tempfile::TempDir,
    capture: crate::CaptureLog,
    observations: Arc<Observations>,
    _authentication: NativeAuthenticationBinding,
    api_key_source: Option<[u8; 24]>,
    tools: Option<crate::NativeToolProvision>,
    instructions: chapchoo_domain::ProjectInstructions,
    tool_catalog: Vec<u8>,
}
struct StartConfiguration<'a> {
    scope: &'a crate::NativeToolScope,
    instructions: &'a chapchoo_domain::ProjectInstructions,
    tool_catalog: &'a [u8],
    empty_scope: bool,
}
impl State {
    async fn captured_source(&mut self) -> Result<Option<crate::CaptureLog>, Error> {
        if let Some(capture) = &self.retained_capture {
            return Ok(Some(capture.clone()));
        }
        let storage = self.storage.clone();
        // The actor owns this finite blocking read beyond the caller's waiter.
        // A missing first copy is absence; an existing corrupt copy is an error.
        let source = tokio::task::spawn_blocking(move || {
            match fs::symlink_metadata(storage.capture_root().join("current")) {
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
                Err(_) => Err(Error::CaptureUnavailable),
                Ok(_) => crate::CaptureLog::open(&storage).map(Some),
            }
        })
        .await
        .map_err(|_| Error::Process)??;
        self.retained_capture = source;
        Ok(self.retained_capture.clone())
    }
    async fn input(
        &mut self,
        operation: NativeInputOperation,
        managed: bool,
        models: Option<&NativeModelDispatch>,
    ) -> Result<NativePromptAcceptance, Error> {
        if let Some(previous) = self.inputs.recover(&operation)? {
            return Ok(previous);
        }
        let live = self.live.as_ref().ok_or(Error::NotAdmitted)?;
        let session = live.session.as_ref().ok_or(Error::NotAdmitted)?;
        if self.blocked
            || live.protocol.is_finished()
            || session.is_closed()
            || live.capture.snapshot()? != operation.target.snapshot
            || session.id() != operation.target.native_session.as_str()
        {
            return Err(Error::NotAdmitted);
        }
        if let Some(previous) = self.inputs.begin(&operation)? {
            return Ok(previous);
        }
        let outcome = match tokio::time::timeout(Duration::from_secs(10), async {
            if !model_selection::is_dispatched_command(session, operation.text.as_str())
                && let Some(accepted) = session
                    .steer(&operation.text, models.map(|model| &model.route))
                    .await?
            {
                return Ok(accepted);
            }
            model_selection::prompt(
                live,
                model_selection::PromptKind::Ordinary { managed },
                &operation.operation_id,
                &operation.text,
                models,
            )
            .await
        })
        .await
        {
            Ok(Ok(outcome)) => outcome,
            Ok(Err(Error::EndingDecisionPending)) => {
                self.inputs.discard_unconsumed(&operation.operation_id)?;
                return Err(Error::EndingDecisionPending);
            }
            _ => return Err(Error::DeliveryUncertain),
        };
        self.inputs
            .completed(&operation.operation_id, outcome.clone())?;
        Ok(outcome)
    }
    async fn continue_after_ending(
        &mut self,
        ending: NativeNormalEnding,
        text: ConversationText,
        operation_id: String,
        models: &NativeModelDispatch,
    ) -> Result<NativePromptAcceptance, Error> {
        let live = self.live.as_ref().ok_or(Error::NotAdmitted)?;
        let session = live.session.as_ref().ok_or(Error::NotAdmitted)?;
        if self.blocked || live.protocol.is_finished() || session.is_closed() {
            return Err(Error::NotAdmitted);
        }
        tokio::time::timeout(
            Duration::from_secs(10),
            model_selection::prompt(
                live,
                model_selection::PromptKind::Continuation(&ending),
                &operation_id,
                &text,
                Some(models),
            ),
        )
        .await
        .map_err(|_| Error::DeliveryUncertain)?
    }
    fn observe(&self) -> Result<NativeObservation, Error> {
        let live = self.live.as_ref().ok_or(Error::NotAdmitted)?;
        if live.protocol.is_finished() || self.blocked {
            return Err(Error::Protocol);
        }
        let mut observed = live.observations.current()?;
        observed.preparation = Some(crate::NativePreparationObservation {
            freshness: live._supply.freshness,
            versions: live._supply.versions.clone(),
        });
        observed.authentication_id = Some(live._authentication.authentication_id.clone());
        observed.applied_supply = Some(live.requested_supply.clone());
        observed.model = Some(model_selection::observe(live)?);
        Ok(observed)
    }
    fn stop(&self, target: &NativeWorkTarget) -> Result<(), Error> {
        let live = self.live.as_ref().ok_or(Error::NotAdmitted)?;
        if self.blocked || live.protocol.is_finished() {
            return Err(Error::NotAdmitted);
        }
        live.observations.publish_stop_at(target, |native_turn| {
            live.session
                .as_ref()
                .ok_or(Error::NotAdmitted)?
                .stop(native_turn)
        })
    }
    fn normal_ending(&self) -> Result<Option<NativeNormalEnding>, Error> {
        let live = self.live.as_ref().ok_or(Error::NotAdmitted)?;
        if self.blocked || live.protocol.is_finished() {
            return Err(Error::NotAdmitted);
        }
        live.observations.normal_ending()
    }
    async fn start(
        &mut self,
        supply: HarnessSupply,
        context: NativeContext,
        source: crate::CaptureLog,
        authentication: Arc<dyn NativeAuthenticationProvider>,
        tools: Arc<dyn crate::NativeToolsProvider>,
        config: StartConfiguration<'_>,
    ) -> Result<NativeConversationId, Error> {
        if self.blocked {
            return Err(Error::RetirementUnknown);
        }
        check_source(&context, &source, &self.storage)?;
        if let Some(retained) = &self.retained_capture
            && retained.snapshot()? != source.snapshot()?
        {
            return Err(Error::CaptureConflict);
        }
        self.retained_capture = Some(source.clone());
        if let Some(live) = &mut self.live {
            if !config.scope.matches(&live._authentication) {
                return Err(Error::OperationConflict);
            }
            if live.capture.snapshot()? != source.snapshot()? {
                return Err(Error::CaptureConflict);
            }
            if live.protocol.is_finished()
                || live.child.try_wait().map_err(|_| Error::Process)?.is_some()
            {
                return Err(Error::Protocol);
            }
            match &context {
                NativeContext::Resume {
                    session,
                    replay: CaptureReplay::AlreadyCaptured,
                } if live
                    .session
                    .as_ref()
                    .is_some_and(|live| live.id() == session.as_str()) => {}
                _ => return Err(Error::NotAdmitted),
            }
            return NativeConversationId::parse(
                live.session
                    .as_ref()
                    .ok_or(Error::Protocol)?
                    .id()
                    .to_owned(),
            )
            .map_err(|_| Error::Protocol);
        }
        self.scope = Some(ScopeEvidence::verify(&self.storage)?);
        // Any partially started preparation is owned by this actor. Failure must
        // retire the namespace scope before another start can be attempted.
        self.blocked = true;
        let result = self
            .launch(supply, context, source, authentication, tools, config)
            .await;
        match result {
            Ok(id) => {
                self.blocked = false;
                Ok(id)
            }
            Err(error) => {
                if let Err(cleanup) = self.retire().await {
                    return Err(Error::StartAndRetirement {
                        primary: Box::new(error),
                        cleanup: Box::new(cleanup),
                    });
                }
                Err(error)
            }
        }
    }
    async fn launch(
        &mut self,
        supply: HarnessSupply,
        context: NativeContext,
        source: crate::CaptureLog,
        authentication: Arc<dyn NativeAuthenticationProvider>,
        tools: Arc<dyn crate::NativeToolsProvider>,
        config: StartConfiguration<'_>,
    ) -> Result<NativeConversationId, Error> {
        let applied = self.preparer.prepare(&supply).await?;
        let transport = Transport::for_supply(&supply);
        let admitted = authentication
            .acquire(chapchoo_domain::LaunchCapture {
                snapshot: source.snapshot()?,
                through: source
                    .current_boundary()?
                    .and_then(|boundary| boundary.through),
            })
            .await?;
        authentication.check(&admitted.binding).await?;
        let binding = admitted.binding;
        if !config.scope.matches(&binding) {
            return Err(Error::Authentication);
        }
        self.pending_tools = Some(
            tools
                .provide(&binding)
                .await
                .map_err(Error::ToolsProvision)?,
        );
        let storage = self.storage.clone();
        let credentials = admitted.credentials;
        // State owns physical completion beyond cancellation of this launch.
        self.credential_write = Some(tokio::task::spawn_blocking(move || {
            if credentials.initial_api_key().is_some() {
                storage.clear_credentials()?;
            } else {
                storage.publish_credentials(&credentials)?;
            }
            Ok(credentials)
        }));
        let written = self.credential_write.as_mut().ok_or(Error::Process)?.await;
        self.credential_write = None;
        let credentials = written.map_err(|_| Error::Storage)??;
        let api_key_source = credentials.initial_api_key().map(|_| binding.source);
        authentication.check(&binding).await?;
        let launch = tempfile::Builder::new()
            .prefix("launch-")
            .tempdir_in(self.storage.temporary())
            .map_err(|_| Error::Storage)?;
        let observations = Arc::new(Observations::default());
        if let NativeContext::Resume { session, .. } = &context {
            observations.bind(source.snapshot()?, session.clone(), config.empty_scope)?;
        }
        let restore = if let Some(read) = self
            .pending_tools
            .as_ref()
            .and_then(crate::NativeToolProvision::goal_restore)
        {
            let script = launch.path().join("goal-restore.cjs");
            // Restore is optional for an otherwise valid native turn.
            fs::write(script, include_str!("runtime/goal-restore.cjs"))
                .and_then(|()| {
                    goal_restore::Listener::start(launch.path(), observations.clone(), read)
                })
                .ok()
        } else {
            None
        };
        let mut command = match transport {
            Transport::CodexAppServer => {
                let mut command = self.storage.command(&applied.codex);
                command.args(["-c", "cli_auth_credentials_store=\"file\"", "app-server"]);
                command
            }
            Transport::Acp => {
                let executable =
                    serde_json::to_string(&applied.codex).map_err(|_| Error::Integrity)?;
                let marker = serde_json::to_string(&launch.path().join("native-launch-claimed"))
                    .map_err(|_| Error::Integrity)?;
                let wrapper = include_str!("runtime/launch.js")
                    .replace("__CHAPCHOO_EXECUTABLE__", &executable)
                    .replace("__CHAPCHOO_MARKER__", &marker);
                let entrypoint = launch.path().join("codex-launch.cjs");
                fs::write(&entrypoint, wrapper).map_err(|_| Error::Storage)?;
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&entrypoint, fs::Permissions::from_mode(0o700))
                    .map_err(|_| Error::Storage)?;
                let mut command = self.storage.command(&applied.node);
                command
                    .arg(&applied.adapter)
                    .env("CODEX_PATH", entrypoint)
                    .env("CODEX_CONFIG", native_tool_config());
                command
            }
        };
        if restore.is_some() {
            command
                .env("CHAPCHOO_GOAL_RESTORE_NODE", &applied.node)
                .env(
                    "CHAPCHOO_GOAL_RESTORE_SCRIPT",
                    launch.path().join("goal-restore.cjs"),
                )
                .env(
                    "CHAPCHOO_GOAL_RESTORE_SOCKET",
                    launch.path().join(goal_restore::SOCKET),
                );
        }
        command
            .env("PATH", explicit_path(&applied.node)?)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        let mut child = command.spawn().map_err(|_| Error::Process)?;
        let stdin = child.stdin.take().ok_or(Error::Process)?;
        let stdout = child.stdout.take().ok_or(Error::Process)?;
        let mut stderr = child.stderr.take().ok_or(Error::Process)?;
        // Drain opaque diagnostics without logging credentials, prompts or bodies.
        let stderr = tokio::spawn(async move {
            tokio::io::copy(&mut stderr, &mut tokio::io::sink())
                .await
                .map(|_| ())
        });
        let requests: Requests = Arc::default();
        let done = CancellationToken::new();
        let ended = CancellationToken::new();
        let completion = ended.clone();
        let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
        let (pending, protocol) = match transport {
            Transport::CodexAppServer => {
                let (connection, driver) = app_server::Connection::connect(
                    stdout,
                    stdin,
                    done.clone(),
                    source.clone(),
                    observations.clone(),
                );
                (
                    PendingSession::AppServer(connection, credentials),
                    tokio::spawn(async move {
                        let result = driver.await;
                        completion.cancel();
                        result
                    }),
                )
            }
            Transport::Acp => {
                let log = source.clone();
                let observed = observations.clone();
                let record: Recorder = Arc::new(move |line| observed.record_live(&log, line));
                let incoming = FramedRead::new(stdout, LinesCodec::new_with_max_length(MAX_LINE))
                    .map(move |line| {
                        let line = line.map_err(std::io::Error::other)?;
                        capture(&line, record.as_ref()).map_err(std::io::Error::other)?;
                        Ok::<_, std::io::Error>(line)
                    });
                let outgoing = SinkExt::<String>::sink_map_err(
                    FramedWrite::new(stdin, LinesCodec::new_with_max_length(MAX_LINE)),
                    std::io::Error::other,
                );
                let (ready, ready_rx) = oneshot::channel();
                let declarations = self
                    .pending_tools
                    .as_ref()
                    .ok_or(Error::NotAdmitted)?
                    .declarations();
                let driver = protocol::connect_authenticated(
                    Lines::new(outgoing, incoming),
                    requests.clone(),
                    observations.clone(),
                    ready,
                    done.clone(),
                    self.storage.workspace(),
                    (context.clone(), Some(credentials), declarations),
                );
                (
                    PendingSession::Acp(ready_rx),
                    tokio::spawn(async move {
                        let result = driver.await.map_err(|_| Error::Protocol);
                        completion.cancel();
                        result
                    }),
                )
            }
        };
        self.live = Some(Live {
            child,
            protocol,
            stderr,
            done,
            ended,
            restore,
            session: None,
            requests,
            _supply: applied,
            requested_supply: supply,
            _launch: launch,
            capture: source,
            observations,
            _authentication: binding,
            api_key_source,
            tools: self.pending_tools.take(),
            instructions: config.instructions.clone(),
            tool_catalog: config.tool_catalog.to_vec(),
        });
        let session = tokio::time::timeout_at(deadline, async {
            match pending {
                PendingSession::Acp(ready) => Ok(Session::from_acp(
                    ready.await.map_err(|_| Error::Protocol)??,
                )),
                PendingSession::AppServer(connection, credentials) => {
                    let declaration = self
                        .live
                        .as_ref()
                        .and_then(|live| live.tools.as_ref())
                        .ok_or(Error::NotAdmitted)?
                        .declaration();
                    connection
                        .open(
                            &self.storage.workspace(),
                            &context,
                            declaration,
                            credentials.initial_api_key(),
                            config.empty_scope,
                        )
                        .await
                        .map(Session::AppServer)
                }
            }
        })
        .await
        .map_err(|_| Error::Protocol)??;
        let id =
            NativeConversationId::parse(session.id().to_owned()).map_err(|_| Error::Protocol)?;
        self.live
            .as_ref()
            .ok_or(Error::Process)?
            .capture
            .bind_native(&id)?;
        let live = self.live.as_ref().ok_or(Error::Process)?;
        live.observations
            .bind(live.capture.snapshot()?, id.clone(), config.empty_scope)?;
        self.live.as_mut().ok_or(Error::Process)?.session = Some(session);
        let provision = self
            .live
            .as_ref()
            .and_then(|live| live.tools.as_ref())
            .ok_or(Error::NotAdmitted)?;
        tokio::time::timeout_at(deadline, provision.verify())
            .await
            .map_err(|_| Error::ProtocolRejected {
                stage: crate::ProtocolStage::ServerTools,
                code: -32603,
            })?
            .map_err(Error::ToolsVerification)?;
        self.live
            .as_ref()
            .ok_or(Error::Process)?
            .observations
            .enable_normal_endings()?;
        Ok(id)
    }
    async fn respond(&self, response: InteractiveResponse) -> Result<(), Error> {
        self.respond_to(response, None).await
    }
    async fn respond_to(
        &self,
        response: InteractiveResponse,
        exact: Option<&chapchoo_domain::SourcePosition>,
    ) -> Result<(), Error> {
        if self.blocked {
            return Err(Error::NotAdmitted);
        }
        let live = self.live.as_ref().ok_or(Error::NotAdmitted)?;
        let session = live.session.as_ref().ok_or(Error::NotAdmitted)?;
        if live.protocol.is_finished() || session.is_closed() {
            return Err(Error::NotAdmitted);
        }
        session.answer(&live.requests, response, exact).await
    }
    async fn retire(&mut self) -> Result<Retirement, Error> {
        let project_publication = match self.project_write.as_mut() {
            Some(work) => work
                .await
                .map_err(|_| Error::Storage)
                .and_then(|result| result),
            None => Ok(()),
        };
        self.project_write = None;
        if let Err(error) = project_publication {
            self.cleanup_errors.push(error);
        }
        if let Some(tools) = &self.pending_tools {
            tools.close();
        }
        if let Some(tools) = self.live.as_ref().and_then(|live| live.tools.as_ref()) {
            tools.close();
        }
        let publication = match self.credential_write.as_mut() {
            Some(work) => work
                .await
                .map_err(|_| Error::Storage)
                .and_then(|result| result.map(|_| ())),
            None => Ok(()),
        };
        self.credential_write = None;
        if let Err(error) = publication {
            self.cleanup_errors.push(error);
        }
        let had_scope = self.scope.is_some();
        self.blocked = true;
        if let Some(live) = &mut self.live {
            live.restore.take();
            if let Some(session) = &live.session {
                session.close().await;
            }
            live.done.cancel();
        }
        let outcome = if had_scope {
            retire_scope().await
        } else {
            Ok(())
        };
        let quiescent = outcome.is_ok();
        if let Err(error) = outcome {
            self.cleanup_errors.push(error);
        }
        if let Some(tools) = &mut self.pending_tools {
            match tools.finish().await {
                Ok(()) => self.pending_tools = None,
                Err(error) => self.cleanup_errors.push(Error::ToolsCleanup(error)),
            }
        }
        if let Some(live) = &mut self.live
            && let Some(tools) = &mut live.tools
        {
            match tools.finish().await {
                Ok(()) => live.tools = None,
                Err(error) => self.cleanup_errors.push(Error::ToolsCleanup(error)),
            }
        }
        if quiescent
            && self.live.as_ref().is_some_and(|live| live.tools.is_none())
            && let Some(live) = self.live.take()
        {
            let Live {
                mut child,
                mut protocol,
                mut stderr,
                ..
            } = live;
            let _ = child.wait().await;
            if tokio::time::timeout(Duration::from_secs(1), &mut protocol)
                .await
                .is_err()
            {
                protocol.abort();
                let _ = protocol.await;
            }
            if tokio::time::timeout(Duration::from_secs(1), &mut stderr)
                .await
                .is_err()
            {
                stderr.abort();
                let _ = stderr.await;
            }
        }
        if quiescent && self.pending_tools.is_none() && self.cleanup_errors.is_empty() {
            self.blocked = false;
        }
        let mut errors = std::mem::take(&mut self.cleanup_errors).into_iter();
        if let Some(mut failure) = errors.next() {
            for secondary in errors {
                failure = Error::CleanupFailed {
                    primary: Box::new(failure),
                    secondary: Box::new(secondary),
                };
            }
            return Err(failure);
        }
        Ok(if had_scope {
            Retirement::Quiescent
        } else {
            Retirement::NoNativeStarted
        })
    }
}
fn native_tool_config() -> &'static str {
    r#"{"code_mode":{"direct_only_tool_namespaces":["mcp__chapchoo"]},"features":{"code_mode_only_strict_3p_tools":false}}"#
}

fn check_source(
    context: &NativeContext,
    source: &crate::CaptureLog,
    storage: &NativeStorage,
) -> Result<(), Error> {
    if !source.belongs_to(storage) {
        return Err(Error::CaptureConflict);
    }
    let native = source.native_session()?;
    let empty = source.is_empty()?;
    match context {
        NativeContext::New if native.is_none() && empty => Ok(()),
        NativeContext::Resume {
            session,
            replay: CaptureReplay::AlreadyCaptured,
        } if native.as_ref() == Some(session) => Ok(()),
        NativeContext::Resume {
            session,
            replay: CaptureReplay::FromStart,
        } if empty && native.as_ref().is_none_or(|id| id == session) => Ok(()),
        _ => Err(Error::CaptureConflict),
    }
}
pub(crate) fn explicit_path(node: &std::path::Path) -> Result<std::ffi::OsString, Error> {
    std::env::join_paths([
        node.parent().ok_or(Error::Integrity)?,
        std::path::Path::new("/usr/bin"),
        std::path::Path::new("/bin"),
    ])
    .map_err(|_| Error::Integrity)
}
#[derive(Deserialize)]
struct IncomingEnvelope {
    method: Option<String>,
}
#[derive(Deserialize)]
struct SessionEnvelope {
    params: SessionParams,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SessionParams {
    session_id: Option<String>,
}
fn capture(
    line: &str,
    record: &(dyn Fn(&str) -> Result<(), Error> + Send + Sync),
) -> Result<(), Error> {
    let envelope: IncomingEnvelope = serde_json::from_str(line).map_err(|_| Error::Protocol)?;
    if let Some("session/update" | "session/request_permission" | "elicitation/create") =
        envelope.method.as_deref()
    {
        let session: SessionEnvelope = serde_json::from_str(line).map_err(|_| Error::Protocol)?;
        if session
            .params
            .session_id
            .as_ref()
            .is_none_or(String::is_empty)
        {
            if envelope.method.as_deref() == Some("elicitation/create") {
                return Ok(());
            }
            return Err(Error::Protocol);
        }
        record(line)?;
    }
    Ok(())
}
fn processes() -> Result<Vec<Pid>, Error> {
    let mut pids = Vec::new();
    for entry in fs::read_dir("/proc").map_err(|_| Error::RetirementUnknown)? {
        let entry = entry.map_err(|_| Error::RetirementUnknown)?;
        if let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|s| s.parse::<i32>().ok())
            .filter(|p| *p > 1)
        {
            pids.push(Pid::from_raw(pid));
        }
    }
    Ok(pids)
}
async fn retire_scope() -> Result<(), Error> {
    if getpid().as_raw() != 1 {
        return Err(Error::RetirementUnknown);
    }
    for (signal, budget) in [
        (Signal::SIGTERM, Duration::from_secs(2)),
        (Signal::SIGKILL, Duration::from_secs(3)),
    ] {
        let deadline = tokio::time::Instant::now() + budget;
        loop {
            let pids = processes()?;
            if pids.is_empty() {
                return Ok(());
            }
            for pid in pids {
                match kill(pid, signal) {
                    Ok(()) | Err(Errno::ESRCH) => {}
                    Err(_) => return Err(Error::RetirementUnknown),
                }
            }
            loop {
                match waitpid(Pid::from_raw(-1), Some(WaitPidFlag::WNOHANG)) {
                    Ok(WaitStatus::StillAlive) | Err(Errno::ECHILD) => break,
                    Ok(_) => {}
                    Err(Errno::EINTR) => continue,
                    Err(_) => return Err(Error::RetirementUnknown),
                }
            }
            if tokio::time::Instant::now() >= deadline {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
    if processes()?.is_empty() {
        Ok(())
    } else {
        Err(Error::RetirementUnknown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn actual_start_policy_directly_exposes_only_the_conversation_namespace()
    -> Result<(), Box<dyn std::error::Error>> {
        let config: serde_json::Value = serde_json::from_str(native_tool_config())?;
        assert_eq!(
            config,
            serde_json::json!({
                "code_mode": {"direct_only_tool_namespaces": ["mcp__chapchoo"]},
                "features": {"code_mode_only_strict_3p_tools": false}
            })
        );
        Ok(())
    }

    #[test]
    fn context_intent_requires_the_owned_copy_and_does_not_replay_existing_capture()
    -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let storage = NativeStorage::open(dir.path())?;
        let log = crate::CaptureLog::create(&storage)?;
        let id = NativeConversationId::parse("native".into())?;
        check_source(&NativeContext::New, &log, &storage)?;
        assert!(matches!(
            check_source(
                &NativeContext::Resume {
                    session: id.clone(),
                    replay: CaptureReplay::AlreadyCaptured
                },
                &log,
                &storage
            ),
            Err(Error::CaptureConflict)
        ));
        check_source(
            &NativeContext::Resume {
                session: id.clone(),
                replay: CaptureReplay::FromStart,
            },
            &log,
            &storage,
        )?;
        log.bind_native(&id)?;
        assert!(matches!(
            check_source(&NativeContext::New, &log, &storage),
            Err(Error::CaptureConflict)
        ));
        log.append(r#"{"method":"session/update","params":{"sessionId":"native","update":{"sessionUpdate":"_future"}}}"#,None)?;
        assert!(matches!(
            check_source(
                &NativeContext::Resume {
                    session: id.clone(),
                    replay: CaptureReplay::FromStart
                },
                &log,
                &storage
            ),
            Err(Error::CaptureConflict)
        ));
        check_source(
            &NativeContext::Resume {
                session: id,
                replay: CaptureReplay::AlreadyCaptured,
            },
            &log,
            &storage,
        )?;
        let foreign = tempfile::tempdir()?;
        assert!(matches!(
            check_source(
                &NativeContext::New,
                &log,
                &NativeStorage::open(foreign.path())?
            ),
            Err(Error::CaptureConflict)
        ));
        Ok(())
    }
    #[test]
    fn raw_events_survive_unknown_fields_but_credentials_never_enter_capture()
    -> Result<(), Box<dyn std::error::Error>> {
        let observed = std::sync::Mutex::new(Vec::new());
        let record = |line: &str| {
            observed
                .lock()
                .map_err(|_| Error::Storage)?
                .push(line.to_owned());
            Ok(())
        };
        let event = r#"{"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"native","update":{"sessionUpdate":"_future","untouched":[1,2]}}}"#;
        capture(event, &record)?;
        capture(
            r#"{"jsonrpc":"2.0","method":"authenticate","params":{"token":"secret-marker"}}"#,
            &record,
        )?;
        capture(
            r#"{"jsonrpc":"2.0","id":3,"result":{"accessToken":"secret-marker"}}"#,
            &record,
        )?;
        capture(
            r#"{"jsonrpc":"2.0","id":4,"method":"elicitation/create","params":{"mode":"form","requestId":3,"message":"secret-marker"}}"#,
            &record,
        )?;
        assert_eq!(
            *observed.lock().map_err(|_| Error::Process)?,
            vec![event.to_owned()]
        );
        Ok(())
    }
    #[tokio::test]
    async fn connectivity_owner_does_not_start_native_or_require_pid1()
    -> Result<(), Box<dyn std::error::Error>> {
        let saved = tempfile::tempdir()?;
        let owner = NativeOwner::new(NativeStorage::open(saved.path())?)?;
        assert_eq!(owner.preparation().await?, None);
        let operations = owner.handle();
        assert!(owner.capture_reader().current_source().await?.is_none());
        assert!(owner.pending_requests().await?.is_empty());
        assert!(matches!(owner.observe().await, Err(Error::NotAdmitted)));
        let input = NativeInputOperation {
            operation_id: "unavailable".into(),
            target: NativeTextTarget {
                snapshot: chapchoo_domain::SnapshotId::parse("snapshot".into())?,
                native_session: NativeConversationId::parse("native".into())?,
            },
            text: ConversationText::parse("not sent".into())?,
        };
        assert!(matches!(
            owner
                .input_observation(input.operation_id.clone(), input.target.clone())
                .await?,
            NativeInputObservation::Unseen
        ));
        assert!(matches!(
            owner.prompt(input.clone()).await,
            Err(Error::NotAdmitted)
        ));
        assert!(matches!(owner.prompt(input).await, Err(Error::NotAdmitted)));
        assert_eq!(owner.finish().await?, Retirement::NoNativeStarted);
        assert!(matches!(
            operations.observe().await,
            Err(Error::NotAdmitted)
        ));
        Ok(())
    }
    #[tokio::test]
    async fn captured_source_reader_opens_retained_history_without_native_start()
    -> Result<(), Box<dyn std::error::Error>> {
        let saved = tempfile::tempdir()?;
        let storage = NativeStorage::open(saved.path())?;
        let log = crate::CaptureLog::create(&storage)?;
        log.bind_native(&NativeConversationId::parse("native".into())?)?;
        let snapshot = log.snapshot()?;
        drop(log);
        let owner = NativeOwner::new(storage)?;
        let source = owner
            .capture_reader()
            .current_source()
            .await?
            .ok_or("Missing retained source")?;
        assert_eq!(source.snapshot()?, snapshot);
        assert!(source.current_boundary()?.is_some());
        assert!(matches!(owner.observe().await, Err(Error::NotAdmitted)));
        assert_eq!(owner.finish().await?, Retirement::NoNativeStarted);
        Ok(())
    }
}
