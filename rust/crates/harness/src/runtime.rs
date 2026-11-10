use crate::{AppliedSupply, Error, NativeStorage, Preparer};
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
    sync::{Mutex, mpsc, oneshot},
    task::JoinHandle,
};
use tokio_util::{
    codec::{FramedRead, FramedWrite, LinesCodec},
    sync::CancellationToken,
};
mod protocol;

const MAX_LINE: usize = 16 * 1024 * 1024;
const MAX_REQUESTS: usize = 32;
type Recorder = Arc<dyn Fn(&str) -> Result<(), Error> + Send + Sync>;

/// Original incoming event envelope, before SDK deserialization. Never credential traffic.
pub type SessionEventRecorder = Recorder;

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
#[derive(Clone)]
pub struct NativePromptAcceptance {
    pub native_message_id: String,
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
        request: schema::RequestPermissionRequest,
    },
    Form {
        id: String,
        request: schema::CreateElicitationRequest,
    },
}
impl fmt::Debug for InteractiveRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("InteractiveRequest([REDACTED])")
    }
}
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
type Requests = Arc<Mutex<BTreeMap<String, Pending>>>;
enum Command {
    Preparation(oneshot::Sender<Option<crate::NativePreparationObservation>>),
    Start {
        supply: HarnessSupply,
        resume: Option<NativeConversationId>,
        record: Recorder,
        result: oneshot::Sender<Result<NativeConversationId, Error>>,
    },
    Prompt(
        ConversationText,
        oneshot::Sender<Result<NativePromptAcceptance, Error>>,
    ),
    Stop(oneshot::Sender<Result<(), Error>>),
    Requests(oneshot::Sender<Vec<InteractiveRequest>>),
    Respond(InteractiveResponse, oneshot::Sender<Result<(), Error>>),
}
/// Execution lifetime owner; bounded operations survive cancellation of their waiter.
pub struct NativeOwner {
    sender: mpsc::Sender<Command>,
    task: JoinHandle<Result<Retirement, Error>>,
    shutdown: CancellationToken,
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
            };
            loop {
                let command = tokio::select! {
                    biased;
                    _ = stopping.cancelled() => break,
                    command = receiver.recv() => match command { Some(c)=>c, None=>break },
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
                    Command::Start {
                        supply,
                        resume,
                        record,
                        result,
                    } => {
                        let outcome = tokio::select! {
                            _ = stopping.cancelled() => Err(Error::Process),
                            outcome = state.start(supply,resume,record) => outcome,
                        };
                        let _ = result.send(outcome);
                    }
                    Command::Prompt(text, result) => {
                        let outcome = match state.live.as_ref().and_then(|l| l.session.as_ref()) {
                            Some(session) if !state.blocked && text.as_str().len() <= MAX_LINE => {
                                tokio::select! {
                                    _ = stopping.cancelled() => Err(Error::Protocol),
                                    outcome = tokio::time::timeout(Duration::from_secs(10),session.send_prompt(text.as_str()).block_task()) =>
                                        outcome.map_err(|_| Error::Protocol).and_then(|r|r.map(|r|NativePromptAcceptance { native_message_id:r.message_id.to_string() }).map_err(|_|Error::Protocol)),
                                }
                            }
                            _ => Err(Error::NotAdmitted),
                        };
                        // Publication can precede a lost acknowledgement. Fence
                        // later prompts until the physical owner is retired.
                        if matches!(&outcome, Err(Error::Protocol)) {
                            state.blocked = true;
                        }
                        let _ = result.send(outcome);
                    }
                    Command::Stop(result) => {
                        let _ = result.send(
                            state
                                .live
                                .as_ref()
                                .and_then(|l| l.session.as_ref())
                                .ok_or(Error::NotAdmitted)
                                .and_then(|s| s.cancel_active_work().map_err(|_| Error::Protocol)),
                        );
                    }
                    Command::Requests(result) => {
                        let mut visible = Vec::new();
                        if let Some(live) = &state.live {
                            let mut pending = live.requests.lock().await;
                            pending.retain(|_, r| !r.cancelled());
                            for (id, r) in pending.iter() {
                                visible.push(match r {
                                    Pending::Permission(request, _) => {
                                        InteractiveRequest::Permission {
                                            id: id.clone(),
                                            request: request.clone(),
                                        }
                                    }
                                    Pending::Form(request, _) => InteractiveRequest::Form {
                                        id: id.clone(),
                                        request: request.clone(),
                                    },
                                });
                            }
                        }
                        let _ = result.send(visible);
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
        })
    }
    pub async fn start(
        &self,
        supply: HarnessSupply,
        resume: Option<NativeConversationId>,
        record: SessionEventRecorder,
    ) -> Result<NativeConversationId, Error> {
        let (result, rx) = oneshot::channel();
        self.sender
            .try_send(Command::Start {
                supply,
                resume,
                record,
                result,
            })
            .map_err(|_| Error::NotAdmitted)?;
        rx.await.map_err(|_| Error::Process)?
    }
    /// Read the physically started combination without preparing or refreshing it.
    pub async fn preparation(&self) -> Result<Option<crate::NativePreparationObservation>, Error> {
        let (result, rx) = oneshot::channel();
        self.sender
            .try_send(Command::Preparation(result))
            .map_err(|_| Error::NotAdmitted)?;
        rx.await.map_err(|_| Error::Process)
    }
    /// Acknowledges native prompt acceptance only, never completion or consumption.
    pub async fn prompt(&self, text: ConversationText) -> Result<NativePromptAcceptance, Error> {
        let (result, rx) = oneshot::channel();
        self.sender
            .try_send(Command::Prompt(text, result))
            .map_err(|_| Error::NotAdmitted)?;
        rx.await.map_err(|_| Error::Process)?
    }
    pub async fn stop_work(&self) -> Result<(), Error> {
        let (result, rx) = oneshot::channel();
        self.sender
            .try_send(Command::Stop(result))
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
    pub async fn finish(self) -> Result<Retirement, Error> {
        self.shutdown.cancel();
        self.task.await.map_err(|_| Error::RetirementUnknown)?
    }
}
struct State {
    storage: NativeStorage,
    preparer: Preparer,
    live: Option<Live>,
    scope: Option<ScopeEvidence>,
    blocked: bool,
}
struct Live {
    child: Child,
    protocol: JoinHandle<Result<(), agent_client_protocol::Error>>,
    stderr: JoinHandle<Result<(), std::io::Error>>,
    done: CancellationToken,
    session: Option<agent_client_protocol::v2::ActiveSession<Agent>>,
    requests: Requests,
    _supply: AppliedSupply,
    _launch: tempfile::TempDir,
}
impl State {
    async fn start(
        &mut self,
        supply: HarnessSupply,
        resume: Option<NativeConversationId>,
        record: Recorder,
    ) -> Result<NativeConversationId, Error> {
        if self.blocked {
            return Err(Error::RetirementUnknown);
        }
        if let Some(live) = &mut self.live {
            if live.protocol.is_finished()
                || live.child.try_wait().map_err(|_| Error::Process)?.is_some()
            {
                return Err(Error::Protocol);
            }
            return NativeConversationId::parse(
                live.session
                    .as_ref()
                    .ok_or(Error::Protocol)?
                    .session_id()
                    .to_string(),
            )
            .map_err(|_| Error::Protocol);
        }
        self.scope = Some(ScopeEvidence::verify(&self.storage)?);
        // Any partially started preparation is owned by this actor. Failure must
        // retire the namespace scope before another start can be attempted.
        self.blocked = true;
        let result = self.launch(supply, resume, record).await;
        match result {
            Ok(id) => {
                self.blocked = false;
                Ok(id)
            }
            Err(error) => {
                if self.retire().await.is_err() {
                    return Err(Error::StartAndRetirement {
                        primary: Box::new(error),
                    });
                }
                Err(error)
            }
        }
    }
    async fn launch(
        &mut self,
        supply: HarnessSupply,
        resume: Option<NativeConversationId>,
        record: Recorder,
    ) -> Result<NativeConversationId, Error> {
        let applied = self.preparer.prepare(&supply).await?;
        let launch = tempfile::Builder::new()
            .prefix("launch-")
            .tempdir_in(self.storage.temporary())
            .map_err(|_| Error::Storage)?;
        let executable = serde_json::to_string(&applied.codex).map_err(|_| Error::Integrity)?;
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
        let incoming =
            FramedRead::new(stdout, LinesCodec::new_with_max_length(MAX_LINE)).map(move |line| {
                let line = line.map_err(std::io::Error::other)?;
                capture(&line, record.as_ref()).map_err(std::io::Error::other)?;
                Ok::<_, std::io::Error>(line)
            });
        let outgoing = SinkExt::<String>::sink_map_err(
            FramedWrite::new(stdin, LinesCodec::new_with_max_length(MAX_LINE)),
            std::io::Error::other,
        );
        let requests: Requests = Arc::default();
        let (ready, ready_rx) = oneshot::channel();
        let done = CancellationToken::new();
        let cwd = self.storage.workspace();
        let protocol = tokio::spawn(protocol::connect(
            Lines::new(outgoing, incoming),
            requests.clone(),
            ready,
            done.clone(),
            cwd,
            resume,
        ));
        self.live = Some(Live {
            child,
            protocol,
            stderr,
            done,
            session: None,
            requests,
            _supply: applied,
            _launch: launch,
        });
        let session = tokio::time::timeout(Duration::from_secs(30), ready_rx)
            .await
            .map_err(|_| Error::Protocol)?
            .map_err(|_| Error::Protocol)??;
        let id = NativeConversationId::parse(session.session_id().to_string())
            .map_err(|_| Error::Protocol)?;
        self.live.as_mut().ok_or(Error::Process)?.session = Some(session);
        Ok(id)
    }
    async fn respond(&self, response: InteractiveResponse) -> Result<(), Error> {
        if self.blocked {
            return Err(Error::NotAdmitted);
        }
        let live = self.live.as_ref().ok_or(Error::NotAdmitted)?;
        let session = live.session.as_ref().ok_or(Error::NotAdmitted)?;
        if live.protocol.is_finished() || session.connection().is_incoming_closed() {
            return Err(Error::NotAdmitted);
        }
        let id = match &response {
            InteractiveResponse::Permission { id, .. } | InteractiveResponse::Form { id, .. } => id,
        };
        let mut pending = live.requests.lock().await;
        let request = pending.get(id).ok_or(Error::NotAdmitted)?;
        if let Pending::Permission(request, _) = request
            && &request.session_id != session.session_id()
        {
            return Err(Error::NotAdmitted);
        }
        if let Pending::Form(request, _) = request {
            let schema::ElicitationMode::Form(form) = &request.mode else {
                return Err(Error::NotAdmitted);
            };
            let schema::ElicitationScope::Session(scope) = &form.scope else {
                return Err(Error::NotAdmitted);
            };
            if &scope.session_id != session.session_id() {
                return Err(Error::NotAdmitted);
            }
        }
        if request.cancelled()
            || !matches!(
                (request, &response),
                (
                    Pending::Permission(..),
                    InteractiveResponse::Permission { .. }
                ) | (Pending::Form(..), InteractiveResponse::Form { .. })
            )
        {
            return Err(Error::NotAdmitted);
        }
        match (pending.remove(id), response) {
            (Some(Pending::Permission(_, r)), InteractiveResponse::Permission { response, .. }) => {
                r.respond(response).map_err(|_| Error::Protocol)
            }
            (Some(Pending::Form(_, r)), InteractiveResponse::Form { response, .. }) => {
                r.respond(response).map_err(|_| Error::Protocol)
            }
            _ => Err(Error::NotAdmitted),
        }
    }
    async fn retire(&mut self) -> Result<Retirement, Error> {
        if self.scope.is_none() {
            return Ok(Retirement::NoNativeStarted);
        }
        self.blocked = true;
        if let Some(live) = &mut self.live {
            if let Some(session) = &live.session {
                let _ = session.cancel_active_work();
                let _ = tokio::time::timeout(Duration::from_secs(2), session.close().block_task())
                    .await;
            }
            live.done.cancel();
        }
        let outcome = retire_scope().await;
        if outcome.is_ok()
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
        if outcome.is_ok() {
            self.blocked = false;
        }
        outcome.map(|_| Retirement::Quiescent)
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
        assert!(owner.pending_requests().await?.is_empty());
        assert!(matches!(owner.stop_work().await, Err(Error::NotAdmitted)));
        assert_eq!(owner.finish().await?, Retirement::NoNativeStarted);
        Ok(())
    }
}
