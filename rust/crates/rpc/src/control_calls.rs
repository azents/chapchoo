//! Bounded connection-local calls. Socket/DB admission belongs to the stream
//! adapter; this module never infers non-dispatch from a transport failure.
use chapchoo_protocol::control as wire;
use prost::Message;
use std::{
    collections::HashMap,
    fmt,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::{
    sync::{Notify, OwnedSemaphorePermit, Semaphore, mpsc, oneshot},
    time::Instant,
};
use tonic::Status;

const FRAME_LIMIT: usize = 64 * 1024;
const PRIORITY_CAPACITY: usize = 4;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CallFailure {
    Unavailable,
    Overloaded,
    Uncertain,
    Invalid,
}
#[derive(Debug)]
pub(crate) enum Command {
    Provider(Box<wire::ProviderCommand>),
    Connector(Box<wire::ConnectorCommand>),
}
#[derive(Debug)]
pub(crate) enum Reply {
    Provider(Box<wire::ProviderResult>),
    Connector(Box<wire::ConnectorResult>),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Provider,
    Connector,
}
#[derive(Debug)]
struct Expected {
    identity: wire::CallIdentity,
    computer: wire::ComputerContext,
    kind: Kind,
}
impl Command {
    fn expected(&self) -> Result<Expected, CallFailure> {
        let (identity, computer, kind) = match self {
            Self::Provider(value) => (&value.identity, &value.computer, Kind::Provider),
            Self::Connector(value) => (&value.identity, &value.computer, Kind::Connector),
        };
        let identity = identity.as_ref().ok_or(CallFailure::Invalid)?;
        let computer = computer.as_ref().ok_or(CallFailure::Invalid)?;
        if ![
            identity.call_id.as_str(),
            identity.request_id.as_str(),
            identity.connection_id.as_str(),
            computer.computer_id.as_str(),
            computer.request_id.as_str(),
        ]
        .into_iter()
        .all(valid_id)
            || (!computer.execution_id.is_empty() && !valid_id(&computer.execution_id))
            || computer.request_id != identity.request_id
        {
            return Err(CallFailure::Invalid);
        }
        Ok(Expected {
            identity: identity.clone(),
            computer: computer.clone(),
            kind,
        })
    }
    fn lifecycle(&self) -> bool {
        matches!(self,Self::Provider(value) if !matches!(value.operation,Some(wire::provider_command::Operation::Observe(_))))
    }
    fn frame(self) -> wire::ControlFrame {
        let payload = match self {
            Self::Provider(value) => wire::control_frame::Payload::Provider(*value),
            Self::Connector(value) => wire::control_frame::Payload::Connector(*value),
        };
        wire::ControlFrame {
            payload: Some(payload),
        }
    }
}
fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b':'))
}

pub(crate) struct QueuedCommand {
    frame: wire::ControlFrame,
    deadline: Instant,
}
impl fmt::Debug for QueuedCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("QueuedCommand([REDACTED])")
    }
}
impl QueuedCommand {
    /// Queue time consumes the original caller budget; an expired queued frame
    /// is discarded, never forwarded as a fresh command with a new timeout.
    pub(crate) fn into_frame(mut self) -> Option<wire::ControlFrame> {
        let remaining = self.deadline.checked_duration_since(Instant::now())?;
        let milliseconds = u32::try_from(remaining.as_millis().min(10_000)).ok()?;
        if milliseconds == 0 {
            return None;
        }
        match self.frame.payload.as_mut()? {
            wire::control_frame::Payload::Provider(value) => value.remaining_ms = milliseconds,
            wire::control_frame::Payload::Connector(value) => value.remaining_ms = milliseconds,
            _ => return None,
        }
        Some(self.frame)
    }
}
struct Pending {
    expected: Expected,
    reply: oneshot::Sender<Reply>,
    _local: OwnedSemaphorePermit,
    _global: OwnedSemaphorePermit,
    _lifecycle: Option<OwnedSemaphorePermit>,
}
pub(crate) struct OutboundReceivers {
    pub(crate) priority: mpsc::Receiver<Result<wire::ControlFrame, Status>>,
    pub(crate) commands: mpsc::Receiver<QueuedCommand>,
}
impl fmt::Debug for OutboundReceivers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("OutboundReceivers([REDACTED])")
    }
}
pub(crate) struct ConnectionCalls {
    connection_id: String,
    priority: mpsc::Sender<Result<wire::ControlFrame, Status>>,
    commands: mpsc::Sender<QueuedCommand>,
    local: Arc<Semaphore>,
    global: Arc<Semaphore>,
    lifecycle: Arc<Semaphore>,
    pending: Mutex<HashMap<String, Pending>>,
    closed: AtomicBool,
    changed: Notify,
}
impl fmt::Debug for ConnectionCalls {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ConnectionCalls([REDACTED])")
    }
}
impl ConnectionCalls {
    pub(crate) fn new(
        connection_id: String,
        queue: usize,
        pending: usize,
        global: Arc<Semaphore>,
    ) -> Result<(Arc<Self>, OutboundReceivers), CallFailure> {
        if !valid_id(&connection_id) || queue == 0 || pending == 0 {
            return Err(CallFailure::Invalid);
        }
        let (priority, priority_rx) = mpsc::channel(PRIORITY_CAPACITY);
        let (commands, commands_rx) = mpsc::channel(queue);
        Ok((
            Arc::new(Self {
                connection_id,
                priority,
                commands,
                local: Arc::new(Semaphore::new(pending)),
                global,
                lifecycle: Arc::new(Semaphore::new(1)),
                pending: Mutex::new(HashMap::new()),
                closed: AtomicBool::new(false),
                changed: Notify::new(),
            }),
            OutboundReceivers {
                priority: priority_rx,
                commands: commands_rx,
            },
        ))
    }
    pub(crate) fn is_closed(&self) -> bool {
        self.closed.load(Ordering::Acquire)
    }
    pub(crate) async fn closed(&self) {
        loop {
            let notified = self.changed.notified();
            if self.is_closed() {
                return;
            }
            notified.await;
        }
    }
    pub(crate) fn close(&self) {
        if !self.closed.swap(true, Ordering::AcqRel) {
            if let Ok(mut pending) = self.pending.lock() {
                pending.clear();
            }
            // Wake an idle outbound Stream as well as the reader workers.
            let _ = self
                .priority
                .try_send(Err(Status::unavailable("Component connection is closed")));
            self.changed.notify_waiters();
        }
    }
    pub(crate) fn priority(&self, frame: wire::ControlFrame) -> Result<(), CallFailure> {
        if self.is_closed() {
            return Err(CallFailure::Unavailable);
        }
        if frame.encoded_len() > FRAME_LIMIT {
            return Err(CallFailure::Invalid);
        }
        self.priority
            .try_send(Ok(frame))
            .map_err(|error| match error {
                mpsc::error::TrySendError::Full(_) => CallFailure::Overloaded,
                mpsc::error::TrySendError::Closed(_) => CallFailure::Unavailable,
            })
    }
    fn remove(&self, call: &str) -> bool {
        self.pending
            .lock()
            .ok()
            .and_then(|mut pending| pending.remove(call))
            .is_some()
    }
    pub(crate) async fn invoke(
        self: &Arc<Self>,
        command: Command,
        deadline: Instant,
    ) -> Result<Reply, CallFailure> {
        let expected = command.expected()?;
        if expected.identity.connection_id != self.connection_id {
            return Err(CallFailure::Invalid);
        }
        if self.is_closed() || deadline <= Instant::now() {
            return Err(CallFailure::Unavailable);
        }
        let local = self
            .local
            .clone()
            .try_acquire_owned()
            .map_err(|_| CallFailure::Overloaded)?;
        let global = self
            .global
            .clone()
            .try_acquire_owned()
            .map_err(|_| CallFailure::Overloaded)?;
        let lifecycle = if command.lifecycle() {
            Some(
                self.lifecycle
                    .clone()
                    .try_acquire_owned()
                    .map_err(|_| CallFailure::Overloaded)?,
            )
        } else {
            None
        };
        let identity = expected.identity.clone();
        let (reply, receiver) = oneshot::channel();
        let frame = command.frame();
        if frame.encoded_len() > FRAME_LIMIT {
            return Err(CallFailure::Invalid);
        }
        {
            let mut pending = self.pending.lock().map_err(|_| CallFailure::Unavailable)?;
            if pending.contains_key(&identity.call_id) {
                return Err(CallFailure::Invalid);
            }
            pending.insert(
                identity.call_id.clone(),
                Pending {
                    expected,
                    reply,
                    _local: local,
                    _global: global,
                    _lifecycle: lifecycle,
                },
            );
        }
        let mut guard = InvocationGuard {
            connection: self.clone(),
            identity,
            forwarded: false,
        };
        if self.is_closed() {
            return Err(CallFailure::Unavailable);
        }
        self.commands
            .try_send(QueuedCommand { frame, deadline })
            .map_err(|error| match error {
                mpsc::error::TrySendError::Full(_) => CallFailure::Overloaded,
                mpsc::error::TrySendError::Closed(_) => CallFailure::Unavailable,
            })?;
        // Queue acceptance may lead to forwarding. Neither caller cancellation
        // nor a later socket/DB failure is proof that the effect did not happen.
        guard.forwarded = true;
        tokio::time::timeout_at(deadline, receiver)
            .await
            .map_err(|_| CallFailure::Uncertain)?
            .map_err(|_| CallFailure::Uncertain)
    }
    fn complete(
        &self,
        identity: &wire::CallIdentity,
        computer: Option<&wire::ComputerContext>,
        execution: &str,
        reply: Reply,
        kind: Kind,
    ) -> bool {
        let Ok(mut pending) = self.pending.lock() else {
            return false;
        };
        let Some(current) = pending.get(&identity.call_id) else {
            return false;
        };
        let expected = &current.expected;
        if expected.kind != kind
            || expected.identity != *identity
            || expected.computer.execution_id != execution
            || computer.is_some_and(|value| *value != expected.computer)
        {
            return false;
        }
        let Some(current) = pending.remove(&identity.call_id) else {
            return false;
        };
        current.reply.send(reply).is_ok()
    }
    pub(crate) fn provider_result(&self, result: wire::ProviderResult) -> bool {
        if self.is_closed() || result.encoded_len() > FRAME_LIMIT {
            return false;
        }
        let Some(identity) = result.identity.clone() else {
            return false;
        };
        let Some(computer) = result.computer.clone() else {
            return false;
        };
        self.complete(
            &identity,
            Some(&computer),
            &computer.execution_id,
            Reply::Provider(Box::new(result)),
            Kind::Provider,
        )
    }
    pub(crate) fn connector_result(&self, result: wire::ConnectorResult) -> bool {
        if self.is_closed() || result.encoded_len() > FRAME_LIMIT {
            return false;
        }
        let Some(identity) = result.identity.clone() else {
            return false;
        };
        let execution = result.execution_id.clone();
        self.complete(
            &identity,
            None,
            &execution,
            Reply::Connector(Box::new(result)),
            Kind::Connector,
        )
    }
}
struct InvocationGuard {
    connection: Arc<ConnectionCalls>,
    identity: wire::CallIdentity,
    forwarded: bool,
}
impl Drop for InvocationGuard {
    fn drop(&mut self) {
        if self.connection.remove(&self.identity.call_id) && self.forwarded {
            // Best effort, scoped to this exact call and connection. Saturated
            // cancellation never closes the shared stream or affects other calls.
            let _ = self.connection.priority(wire::ControlFrame {
                payload: Some(wire::control_frame::Payload::Cancel(wire::CancelCall {
                    identity: Some(self.identity.clone()),
                })),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    type TestError = Box<dyn std::error::Error + Send + Sync>;
    fn command(call: &str, lifecycle: bool) -> Command {
        Command::Provider(Box::new(wire::ProviderCommand {
            identity: Some(wire::CallIdentity {
                request_id: "request".into(),
                call_id: call.into(),
                connection_id: "connection".into(),
            }),
            computer: Some(wire::ComputerContext {
                computer_id: "computer".into(),
                request_id: "request".into(),
                execution_id: "execution".into(),
            }),
            remaining_ms: 10_000,
            operation: Some(if lifecycle {
                wire::provider_command::Operation::Retire(wire::RetireExecution {
                    pod: None,
                    require_terminal: false,
                    docker_container: None,
                    saved_volume: None,
                })
            } else {
                wire::provider_command::Operation::Observe(wire::ObserveResources {
                    resources: vec![],
                })
            }),
        }))
    }
    fn result(call: &str) -> wire::ProviderResult {
        wire::ProviderResult {
            identity: Some(wire::CallIdentity {
                request_id: "request".into(),
                call_id: call.into(),
                connection_id: "connection".into(),
            }),
            computer: Some(wire::ComputerContext {
                computer_id: "computer".into(),
                request_id: "request".into(),
                execution_id: "execution".into(),
            }),
            outcome: wire::CommandOutcome::Completed as i32,
            resources: vec![],
            execution: wire::ExecutionFact::Running as i32,
            execution_terminal: false,
        }
    }
    #[test]
    fn cancellation_and_stale_responses_are_call_local() -> Result<(), TestError> {
        tokio::runtime::Runtime::new()?.block_on(async {
            let global=Arc::new(Semaphore::new(4));
            let (connection,mut output)=ConnectionCalls::new("connection".into(),2,4,global.clone()).map_err(|_|"Invalid fixture")?;
            let c=connection.clone();
            let cancelled=tokio::spawn(async move {c.invoke(command("cancelled",false),Instant::now()+Duration::from_secs(5)).await});
            assert!(output.commands.recv().await.ok_or("Missing first command")?.into_frame().is_some());
            cancelled.abort();
            assert!(cancelled.await.is_err());
            let cancel=output.priority.recv().await.ok_or("Missing exact cancellation")??;
            assert!(matches!(cancel.payload,Some(wire::control_frame::Payload::Cancel(value)) if value.identity.as_ref().is_some_and(|id|id.call_id=="cancelled"&&id.connection_id=="connection")));
            assert!(!connection.is_closed());
            assert_eq!(global.available_permits(),4);
            connection.priority(wire::ControlFrame {payload:Some(wire::control_frame::Payload::Heartbeat(wire::HeartbeatAccepted {connection_id:"connection".into(),expires_at:None}))}).map_err(|_|"Heartbeat blocked")?;
            assert!(matches!(output.priority.recv().await.ok_or("Missing heartbeat")??.payload,Some(wire::control_frame::Payload::Heartbeat(_))));
            let c=connection.clone();
            let next=tokio::spawn(async move {c.invoke(command("next",false),Instant::now()+Duration::from_secs(5)).await});
            let _=output.commands.recv().await.ok_or("Missing next command")?;
            assert!(!connection.provider_result(result("cancelled")));
            let mut wrong=result("next");wrong.identity.as_mut().ok_or("Missing identity")?.connection_id="old-connection".into();
            assert!(!connection.provider_result(wrong));
            let mut wrong=result("next");wrong.computer.as_mut().ok_or("Missing context")?.execution_id="old-execution".into();
            assert!(!connection.provider_result(wrong));
            assert!(connection.provider_result(result("next")));
            assert!(matches!(next.await?,Ok(Reply::Provider(_))));
            assert_eq!(global.available_permits(),4);
            connection.close();
            connection.closed().await;
            assert!(matches!(connection.invoke(command("closed",false),Instant::now()+Duration::from_secs(1)).await,Err(CallFailure::Unavailable)));
            Ok::<(),TestError>(())
        })
    }
    #[test]
    fn bounded_queue_and_lifecycle_rejection_do_not_forward() -> Result<(), TestError> {
        tokio::runtime::Runtime::new()?.block_on(async {
            let (connection, mut output) =
                ConnectionCalls::new("connection".into(), 1, 4, Arc::new(Semaphore::new(4)))
                    .map_err(|_| "Invalid fixture")?;
            let c = connection.clone();
            let first = tokio::spawn(async move {
                c.invoke(
                    command("effect", true),
                    Instant::now() + Duration::from_secs(5),
                )
                .await
            });
            let _ = output.commands.recv().await.ok_or("Missing effect")?;
            assert!(matches!(
                connection
                    .invoke(
                        command("second-effect", true),
                        Instant::now() + Duration::from_secs(1)
                    )
                    .await,
                Err(CallFailure::Overloaded)
            ));
            assert!(connection.provider_result(result("effect")));
            assert!(matches!(first.await?, Ok(Reply::Provider(_))));
            let c = connection.clone();
            let observation = tokio::spawn(async move {
                c.invoke(
                    command("observation", false),
                    Instant::now() + Duration::from_secs(5),
                )
                .await
            });
            // Receiving the first queued command is the synchronization boundary;
            // queue saturation itself is established through reserved capacity.
            let item = output.commands.recv().await.ok_or("Missing observation")?;
            connection
                .commands
                .try_send(item)
                .map_err(|_| "Could not fill owned queue")?;
            assert!(matches!(
                connection
                    .invoke(
                        command("overflow", false),
                        Instant::now() + Duration::from_secs(1)
                    )
                    .await,
                Err(CallFailure::Overloaded)
            ));
            let _ = output
                .commands
                .recv()
                .await
                .ok_or("Missing retained observation")?;
            assert!(connection.provider_result(result("observation")));
            assert!(matches!(observation.await?, Ok(Reply::Provider(_))));
            let expired = QueuedCommand {
                frame: command("expired", false).frame(),
                deadline: Instant::now(),
            };
            assert!(expired.into_frame().is_none());
            Ok::<(), TestError>(())
        })
    }
    #[test]
    fn pending_capacity_is_global_across_independent_connections() -> Result<(), TestError> {
        tokio::runtime::Runtime::new()?.block_on(async {
            let global = Arc::new(Semaphore::new(1));
            let (a, mut output_a) = ConnectionCalls::new("connection".into(), 2, 2, global.clone())
                .map_err(|_| "Invalid fixture")?;
            let (b, mut output_b) = ConnectionCalls::new("connection".into(), 2, 2, global.clone())
                .map_err(|_| "Invalid fixture")?;
            let c = a.clone();
            let first = tokio::spawn(async move {
                c.invoke(command("a", false), Instant::now() + Duration::from_secs(5))
                    .await
            });
            let _ = output_a.commands.recv().await.ok_or("Missing first call")?;
            assert_eq!(global.available_permits(), 0);
            assert!(matches!(
                b.invoke(
                    command("b-rejected", false),
                    Instant::now() + Duration::from_secs(1)
                )
                .await,
                Err(CallFailure::Overloaded)
            ));
            assert!(matches!(
                output_b.commands.try_recv(),
                Err(mpsc::error::TryRecvError::Empty)
            ));
            assert!(a.provider_result(result("a")));
            assert!(matches!(first.await?, Ok(Reply::Provider(_))));
            let c = b.clone();
            let second = tokio::spawn(async move {
                c.invoke(command("b", false), Instant::now() + Duration::from_secs(5))
                    .await
            });
            let _ = output_b
                .commands
                .recv()
                .await
                .ok_or("Missing admitted second call")?;
            assert!(b.provider_result(result("b")));
            assert!(matches!(second.await?, Ok(Reply::Provider(_))));
            assert_eq!(global.available_permits(), 1);
            Ok::<(), TestError>(())
        })
    }
    #[test]
    fn supported_resource_batch_fits_the_existing_frame_ceiling() {
        let resources = (0..crate::control_streams::RESOURCE_BATCH)
            .map(|_| wire::ResourceEvidence {
                resource: Some(wire::ManagedResource {
                    kind: wire::ResourceKind::Pod as i32,
                    namespace: "n".repeat(63),
                    name: "n".repeat(253),
                    uid: Some("u".repeat(128)),
                    execution_id: Some("e".repeat(128)),
                    absence_confirmed: false,
                    native: None,
                }),
                evidence: Some(wire::resource_evidence::Evidence::Present(
                    wire::OwnedPresent {
                        uid: "u".repeat(128),
                        native_identity: None,
                    },
                )),
            })
            .collect();
        let value = wire::ProviderFrame {
            payload: Some(wire::provider_frame::Payload::Result(
                wire::ProviderResult {
                    identity: Some(wire::CallIdentity {
                        request_id: "r".repeat(128),
                        call_id: "c".repeat(128),
                        connection_id: "s".repeat(128),
                    }),
                    computer: Some(wire::ComputerContext {
                        computer_id: "c".repeat(128),
                        request_id: "r".repeat(128),
                        execution_id: "e".repeat(128),
                    }),
                    resources,
                    outcome: wire::CommandOutcome::Completed as i32,
                    execution: wire::ExecutionFact::Unspecified as i32,
                    execution_terminal: false,
                },
            )),
        };
        assert!(value.encoded_len() <= FRAME_LIMIT);
    }
    #[test]
    fn timeout_and_connection_loss_are_uncertain_not_safe_replay() -> Result<(), TestError> {
        tokio::runtime::Runtime::new()?.block_on(async {
            let (connection, mut output) =
                ConnectionCalls::new("connection".into(), 2, 4, Arc::new(Semaphore::new(4)))
                    .map_err(|_| "Invalid fixture")?;
            let c = connection.clone();
            let call = tokio::spawn(async move {
                c.invoke(
                    command("timeout", false),
                    Instant::now() + Duration::from_millis(30),
                )
                .await
            });
            let _ = output
                .commands
                .recv()
                .await
                .ok_or("Missing command before timeout")?;
            assert!(matches!(call.await?, Err(CallFailure::Uncertain)));
            let c = connection.clone();
            let call = tokio::spawn(async move {
                c.invoke(
                    command("disconnected", false),
                    Instant::now() + Duration::from_secs(5),
                )
                .await
            });
            let _ = output
                .commands
                .recv()
                .await
                .ok_or("Missing command before disconnect")?;
            connection.close();
            assert!(matches!(call.await?, Err(CallFailure::Uncertain)));
            assert!(!connection.provider_result(result("disconnected")));
            Ok::<(), TestError>(())
        })
    }
}
