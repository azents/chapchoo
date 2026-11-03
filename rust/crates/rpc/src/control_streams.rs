//! Component-initiated streams and exact connection-local dispatch. Peer/user
//! business authorization is separate from these transport ownership checks.
mod native_retirement;
mod reconciliation;
#[cfg(test)]
mod tests;
use crate::{
    control_calls::{CallFailure, Command, ConnectionCalls, OutboundReceivers, Reply},
    ingress as input,
    mapping::timestamp,
    status::status,
};
use chapchoo_crypto::SecretDigest;
use chapchoo_protocol::control as wire;
use chapchoo_service::{
    AcceptanceTime, ComputersService, PreparedComponentConnection, ProviderSource,
};
use std::{
    collections::HashMap,
    fmt,
    pin::Pin,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Poll},
    time::Duration,
};
use tokio::{
    sync::{OwnedSemaphorePermit, Semaphore, mpsc},
    task::JoinSet,
    time::Instant,
};
use tokio_stream::Stream;
use tonic::{Request, Response, Status, Streaming};

#[cfg(test)]
type ActivationCheckpoint = (
    tokio::sync::oneshot::Sender<()>,
    tokio::sync::oneshot::Receiver<()>,
);

const STREAM_LIMIT: usize = 64;
const GLOBAL_PENDING: usize = 256;
const LOCAL_PENDING: usize = 32;
const COMMAND_QUEUE: usize = 8;
pub(crate) const RESOURCE_BATCH: usize = 32;
const RESPONSE_BUDGET: Duration = Duration::from_secs(10);
fn url_policy(value: &str) -> bool {
    url::Url::parse(value).is_ok_and(|url| {
        url.scheme() == "http"
            && url.host_str().is_some()
            && url.username().is_empty()
            && url.password().is_none()
            && url.path() == "/"
            && url.query().is_none()
            && url.fragment().is_none()
    })
}
#[derive(Debug)]
enum Target {
    Provider(String),
    Connector { computer: String, execution: String },
}
struct Connection {
    target: Target,
    prepared: Arc<PreparedComponentConnection>,
    credential: SecretDigest,
    calls: Arc<ConnectionCalls>,
    _permit: OwnedSemaphorePermit,
    policy: Option<ProviderPolicy>,
    convergence: Arc<Semaphore>,
    cursor: Mutex<Option<String>>,
    cleanup: Mutex<HashMap<String, CleanupProgress>>,
    native: Mutex<HashMap<String, native_retirement::NativeProgress>>,
}
#[derive(Clone)]
struct CleanupProgress {
    computer: wire::ComputerContext,
    batch: Vec<wire::ManagedResource>,
    not_started: bool,
}
#[derive(Debug, Clone)]
struct ProviderPolicy {
    native_scope: chapchoo_domain::NativeScope,
    connector_endpoint: String,
}
impl fmt::Debug for Connection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Connection([REDACTED])")
    }
}
struct AbortWorker(tokio::task::AbortHandle);
impl Drop for AbortWorker {
    fn drop(&mut self) {
        self.0.abort();
    }
}
#[derive(Default)]
struct Entries {
    providers: HashMap<String, Arc<Connection>>,
    connectors: HashMap<String, Arc<Connection>>,
}
struct Inner {
    service: Arc<ComputersService>,
    instance_id: String,
    admission: Arc<AtomicBool>,
    streams: Arc<Semaphore>,
    pending: Arc<Semaphore>,
    entries: Mutex<Entries>,
    activation: tokio::sync::Mutex<()>,
    #[cfg(test)]
    activation_checkpoint: Mutex<Option<ActivationCheckpoint>>,
    workers: Mutex<JoinSet<()>>,
    failed: AtomicBool,
}
impl fmt::Debug for Inner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ControlConnections([REDACTED])")
    }
}
impl Inner {
    async fn activate(&self, entry: Arc<Connection>) -> Result<(), Status> {
        // Serialize durable admission with local activation for both component
        // kinds. A delayed predecessor cannot evict the committed winner.
        let _activation = self.activation.lock().await;
        self.admit()?;
        self.service
            .admit_component_connection(&entry.prepared)
            .await
            .map_err(status)?;
        #[cfg(test)]
        {
            let checkpoint = self
                .activation_checkpoint
                .lock()
                .map_err(|_| Status::internal("Test checkpoint unavailable"))?
                .take();
            if let Some((admitted, resume)) = checkpoint {
                let _ = admitted.send(());
                let _ = resume.await;
            }
        }
        entry
            .calls
            .priority(connected(self, &entry.prepared))
            .map_err(call_status)?;
        self.install(entry)
    }
    fn admit(&self) -> Result<(), Status> {
        if self.admission.load(Ordering::Acquire) && !self.failed.load(Ordering::Acquire) {
            Ok(())
        } else {
            Err(Status::unavailable("Control is not admitting new requests"))
        }
    }
    fn permit(&self) -> Result<OwnedSemaphorePermit, Status> {
        self.admit()?;
        self.streams
            .clone()
            .try_acquire_owned()
            .map_err(|_| Status::resource_exhausted("Component stream capacity is exhausted"))
    }
    fn install(&self, entry: Arc<Connection>) -> Result<(), Status> {
        self.admit()?;
        let mut entries = self
            .entries
            .lock()
            .map_err(|_| Status::unavailable("Control registry is unavailable"))?;
        let replaced = match &entry.target {
            Target::Provider(id) => entries.providers.insert(id.clone(), entry.clone()),
            Target::Connector { computer, .. } => {
                entries.connectors.insert(computer.clone(), entry.clone())
            }
        };
        if let Some(old) = replaced {
            old.calls.close();
        }
        Ok(())
    }
    fn remove(&self, entry: &Connection) {
        let Ok(mut entries) = self.entries.lock() else {
            return;
        };
        let map = match &entry.target {
            Target::Provider(_) => &mut entries.providers,
            Target::Connector { .. } => &mut entries.connectors,
        };
        let id = match &entry.target {
            Target::Provider(id) => id,
            Target::Connector { computer, .. } => computer,
        };
        if map
            .get(id)
            .is_some_and(|current| current.prepared.owner() == entry.prepared.owner())
        {
            map.remove(id);
        }
    }
    fn spawn(
        self: &Arc<Self>,
        task: impl std::future::Future<Output = ()> + Send + 'static,
    ) -> Result<(), Status> {
        self.admit()?;
        let mut workers = self
            .workers
            .lock()
            .map_err(|_| Status::unavailable("Control workers are unavailable"))?;
        while let Some(result) = workers.try_join_next() {
            if result.is_err() {
                self.failed.store(true, Ordering::Release);
                self.admission.store(false, Ordering::Release);
            }
        }
        if self.failed.load(Ordering::Acquire) {
            return Err(Status::unavailable("A Control worker failed"));
        }
        let inner = self.clone();
        workers.spawn(async move {
            let worker = tokio::spawn(task);
            let _abort = AbortWorker(worker.abort_handle());
            if worker.await.is_err() {
                inner.failed.store(true, Ordering::Release);
                inner.admission.store(false, Ordering::Release);
                if let Ok(entries) = inner.entries.lock() {
                    for entry in entries
                        .providers
                        .values()
                        .chain(entries.connectors.values())
                    {
                        entry.calls.close();
                    }
                }
            }
        });
        Ok(())
    }
    async fn finish(&self, entry: &Connection) {
        entry.calls.close();
        self.remove(entry);
        // Revocation/expiry may already have removed authority. Exact close never
        // removes another connection, and an uncertain close is not replayed.
        let _ = self
            .service
            .close_component_connection(&entry.prepared)
            .await;
    }
    async fn verify(&self, entry: &Connection) -> Result<(), Status> {
        self.admit()?;
        if entry.calls.is_closed() {
            return Err(Status::unavailable("Component connection is closed"));
        }
        self.service
            .verify_component_connection(&entry.prepared)
            .await
            .map_err(status)
    }
    fn source<'a>(&'a self, entry: &'a Connection) -> Option<ProviderSource<'a>> {
        let Target::Provider(id) = &entry.target else {
            return None;
        };
        Some(ProviderSource {
            provider_id: id,
            control_id: &self.instance_id,
            connection_id: &entry.prepared.owner().connection_id,
            credential: &entry.credential,
        })
    }
    async fn resource(
        &self,
        entry: &Connection,
        computer: &wire::ComputerContext,
        evidence: wire::ResourceEvidence,
    ) -> Result<(), Status> {
        let source = self
            .source(entry)
            .ok_or_else(|| Status::invalid_argument("Provider authority is required"))?;
        crate::reports::resource(&self.service, &source, computer, evidence).await
    }
    async fn execution(
        &self,
        entry: &Connection,
        computer: &wire::ComputerContext,
        fact: i32,
    ) -> Result<(), Status> {
        let source = self
            .source(entry)
            .ok_or_else(|| Status::invalid_argument("Provider authority is required"))?;
        crate::reports::execution(&self.service, &source, computer, fact).await
    }
    async fn heartbeat(
        &self,
        entry: &Connection,
        heartbeat: wire::Heartbeat,
    ) -> Result<(), Status> {
        if heartbeat.connection_id != entry.prepared.owner().connection_id {
            return Ok(());
        }
        let renewed = self
            .service
            .renew_component_connection(&entry.prepared)
            .await
            .map_err(status)?;
        entry
            .calls
            .priority(wire::ControlFrame {
                payload: Some(wire::control_frame::Payload::Heartbeat(
                    wire::HeartbeatAccepted {
                        connection_id: renewed.connection_id,
                        expires_at: Some(timestamp(renewed.expires_at)?),
                    },
                )),
            })
            .map_err(call_status)
    }
    async fn provider_frame(
        &self,
        entry: &Connection,
        frame: wire::ProviderFrame,
    ) -> Result<(), Status> {
        self.verify(entry).await?;
        match frame.payload {
            Some(wire::provider_frame::Payload::Heartbeat(value)) => {
                self.heartbeat(entry, value).await
            }
            Some(wire::provider_frame::Payload::Result(value)) => {
                if value.resources.len() > RESOURCE_BATCH {
                    return Err(Status::resource_exhausted("Resource batch is too large"));
                }
                let computer = value
                    .computer
                    .clone()
                    .ok_or_else(|| Status::invalid_argument("Computer context is required"))?;
                let identity = value
                    .identity
                    .as_ref()
                    .ok_or_else(|| Status::invalid_argument("Result identity is required"))?;
                if identity.connection_id != entry.prepared.owner().connection_id {
                    return Ok(());
                }
                if identity.request_id != computer.request_id {
                    return Err(Status::invalid_argument("Result identity does not match"));
                }
                for resource in &value.resources {
                    if let Err(error) = self.resource(entry, &computer, resource.clone()).await
                        && fatal_report(&error)
                    {
                        return Err(error);
                    }
                }
                if let Err(error) = self.execution(entry, &computer, value.execution).await
                    && fatal_report(&error)
                {
                    return Err(error);
                }
                native_retirement::result(entry, &value)?;
                entry.calls.provider_result(value);
                Ok(())
            }
            Some(wire::provider_frame::Payload::Resource(value)) => {
                if value.connection_id != entry.prepared.owner().connection_id {
                    return Ok(());
                }
                let computer = value
                    .computer
                    .ok_or_else(|| Status::invalid_argument("Computer context is required"))?;
                let evidence = value
                    .evidence
                    .ok_or_else(|| Status::invalid_argument("Resource evidence is required"))?;
                self.resource(entry, &computer, evidence).await
            }
            Some(wire::provider_frame::Payload::Execution(value)) => {
                if value.connection_id != entry.prepared.owner().connection_id {
                    return Ok(());
                }
                self.execution(
                    entry,
                    &value
                        .computer
                        .ok_or_else(|| Status::invalid_argument("Computer context is required"))?,
                    value.fact,
                )
                .await
            }
            _ => Err(Status::invalid_argument("Unexpected Provider frame")),
        }
    }
    async fn connector_frame(
        &self,
        entry: &Connection,
        frame: wire::ConnectorFrame,
    ) -> Result<(), Status> {
        self.verify(entry).await?;
        let Target::Connector { execution, .. } = &entry.target else {
            return Err(Status::invalid_argument("Connector authority is required"));
        };
        match frame.payload {
            Some(wire::connector_frame::Payload::Heartbeat(value)) => {
                self.heartbeat(entry, value).await
            }
            Some(wire::connector_frame::Payload::Result(value)) => {
                if value.execution_id != *execution
                    || value
                        .identity
                        .as_ref()
                        .is_none_or(|id| id.connection_id != entry.prepared.owner().connection_id)
                {
                    return Ok(());
                }
                entry.calls.connector_result(value);
                Ok(())
            }
            Some(wire::connector_frame::Payload::Observation(value)) => {
                if value.execution_id != *execution
                    || value.connection_id != entry.prepared.owner().connection_id
                {
                    return Ok(());
                }
                // This asserts only that the authenticated execution is reachable;
                // it cannot complete substrate power-off or storage cleanup.
                Ok(())
            }
            _ => Err(Status::invalid_argument("Unexpected Connector frame")),
        }
    }
}
fn fatal_report(error: &Status) -> bool {
    !matches!(
        error.code(),
        tonic::Code::NotFound
            | tonic::Code::AlreadyExists
            | tonic::Code::FailedPrecondition
            | tonic::Code::Aborted
    )
}
fn call_status(error: CallFailure) -> Status {
    match error {
        CallFailure::Invalid => Status::invalid_argument("Component call is invalid"),
        CallFailure::Overloaded => {
            Status::resource_exhausted("Component queue capacity is exhausted")
        }
        CallFailure::Unavailable => Status::unavailable("Component connection is unavailable"),
        CallFailure::Uncertain => Status::unavailable("Component effect outcome is uncertain"),
    }
}
fn outcome(error: CallFailure) -> wire::ForwardingOutcome {
    match error {
        CallFailure::Overloaded => wire::ForwardingOutcome::Overloaded,
        CallFailure::Uncertain => wire::ForwardingOutcome::Uncertain,
        _ => wire::ForwardingOutcome::Unavailable,
    }
}

/// Separate component-facing factory. It does not register internal PeerRelay.
#[derive(Debug, Clone)]
pub struct ComponentConnectionServices {
    inner: Arc<Inner>,
}
impl ComponentConnectionServices {
    pub fn new(
        service: Arc<ComputersService>,
        instance_id: String,
        admission: Arc<AtomicBool>,
    ) -> Self {
        Self {
            inner: Arc::new(Inner {
                service,
                instance_id,
                admission,
                streams: Arc::new(Semaphore::new(STREAM_LIMIT)),
                pending: Arc::new(Semaphore::new(GLOBAL_PENDING)),
                entries: Mutex::new(Entries::default()),
                activation: tokio::sync::Mutex::new(()),
                #[cfg(test)]
                activation_checkpoint: Mutex::new(None),
                workers: Mutex::new(JoinSet::new()),
                failed: AtomicBool::new(false),
            }),
        }
    }
    pub fn provider_server(
        &self,
    ) -> wire::provider_connection_server::ProviderConnectionServer<ProviderConnectionAdapter> {
        wire::provider_connection_server::ProviderConnectionServer::new(ProviderConnectionAdapter {
            inner: self.inner.clone(),
        })
        .max_decoding_message_size(64 * 1024)
        .max_encoding_message_size(64 * 1024)
    }
    pub fn connector_server(
        &self,
    ) -> wire::connector_connection_server::ConnectorConnectionServer<ConnectorConnectionAdapter>
    {
        wire::connector_connection_server::ConnectorConnectionServer::new(
            ConnectorConnectionAdapter {
                inner: self.inner.clone(),
            },
        )
        .max_decoding_message_size(64 * 1024)
        .max_encoding_message_size(64 * 1024)
    }
    pub fn active_streams(&self) -> usize {
        STREAM_LIMIT - self.inner.streams.available_permits()
    }
    pub fn pending_calls(&self) -> usize {
        GLOBAL_PENDING - self.inner.pending.available_permits()
    }
    pub fn ensure_healthy(&self) -> Result<(), Status> {
        self.inner.admit()
    }
    pub async fn drain(&self) -> Result<(), Status> {
        self.inner.admission.store(false, Ordering::Release);
        {
            let entries = self
                .inner
                .entries
                .lock()
                .map_err(|_| Status::unavailable("Control registry is unavailable"))?;
            for entry in entries
                .providers
                .values()
                .chain(entries.connectors.values())
            {
                entry.calls.close();
            }
        }
        let mut workers = {
            let mut guard = self
                .inner
                .workers
                .lock()
                .map_err(|_| Status::unavailable("Control workers are unavailable"))?;
            std::mem::replace(&mut *guard, JoinSet::new())
        };
        let done = async {
            while let Some(result) = workers.join_next().await {
                result.map_err(|_| Status::internal("A Control worker failed"))?;
            }
            Ok::<(), Status>(())
        };
        match tokio::time::timeout(Duration::from_secs(20), done).await {
            Ok(result) => result,
            Err(_) => {
                workers.abort_all();
                while workers.join_next().await.is_some() {}
                Err(Status::unavailable("Control stream drain deadline expired"))
            }
        }
    }
    /// Transport primitive for an already-authorized domain invocation. The peer
    /// adapter must validate actor/current-intent authority before calling this.
    async fn dispatch_entry(
        &self,
        entry: &Connection,
        command: wire::ProviderCommand,
        deadline: Instant,
    ) -> Result<wire::ProviderInvocationResult, Status> {
        self.inner.verify(entry).await?;
        native_retirement::before_forward(entry, &command)?;
        let expected = command.clone();
        match entry
            .calls
            .invoke(Command::Provider(Box::new(command)), deadline)
            .await
        {
            Ok(Reply::Provider(result)) => {
                native_retirement::result(entry, &result)?;
                Ok(wire::ProviderInvocationResult {
                    outcome: wire::ForwardingOutcome::Completed as i32,
                    result: Some(*result),
                })
            }
            Ok(_) => Err(Status::internal("Provider reply kind mismatch")),
            Err(error) => {
                native_retirement::forwarding_failed(entry, &expected, error)?;
                Ok(wire::ProviderInvocationResult {
                    outcome: outcome(error) as i32,
                    result: None,
                })
            }
        }
    }

    pub async fn invoke_provider(
        &self,
        provider_id: &str,
        owner: &wire::ExpectedOwner,
        command: wire::ProviderCommand,
        deadline: Instant,
    ) -> Result<wire::ProviderInvocationResult, Status> {
        let deadline = deadline.min(Instant::now() + RESPONSE_BUDGET);
        if self.inner.admit().is_err() {
            return Ok(wire::ProviderInvocationResult {
                outcome: wire::ForwardingOutcome::Unavailable as i32,
                result: None,
            });
        }
        let route = match tokio::time::timeout_at(
            deadline,
            self.inner.service.resolve_provider_route(provider_id),
        )
        .await
        {
            Ok(Ok(Some(route))) => route,
            _ => {
                return Ok(wire::ProviderInvocationResult {
                    outcome: wire::ForwardingOutcome::Unavailable as i32,
                    result: None,
                });
            }
        };
        if owner.instance_id != self.inner.instance_id
            || route.component.control_id != owner.instance_id
            || route.component.connection_id != owner.connection_id
        {
            return Ok(wire::ProviderInvocationResult {
                outcome: wire::ForwardingOutcome::NotOwner as i32,
                result: None,
            });
        }
        let entry = self
            .inner
            .entries
            .lock()
            .map_err(|_| Status::unavailable("Control registry is unavailable"))?
            .providers
            .get(provider_id)
            .cloned();
        let Some(entry) = entry.filter(|entry| {
            entry.prepared.owner().connection_id == owner.connection_id && !entry.calls.is_closed()
        }) else {
            return Ok(wire::ProviderInvocationResult {
                outcome: wire::ForwardingOutcome::NotOwner as i32,
                result: None,
            });
        };
        if !matches!(
            tokio::time::timeout_at(deadline, self.inner.verify(&entry)).await,
            Ok(Ok(()))
        ) {
            return Ok(wire::ProviderInvocationResult {
                outcome: wire::ForwardingOutcome::Unavailable as i32,
                result: None,
            });
        }
        self.dispatch_entry(&entry, command, deadline).await
    }
    pub async fn invoke_connector(
        &self,
        computer_id: &str,
        owner: &wire::ExpectedOwner,
        command: wire::ConnectorCommand,
        deadline: Instant,
    ) -> Result<wire::ConnectorInvocationResult, Status> {
        let deadline = deadline.min(Instant::now() + RESPONSE_BUDGET);
        if self.inner.admit().is_err() {
            return Ok(wire::ConnectorInvocationResult {
                outcome: wire::ForwardingOutcome::Unavailable as i32,
                result: None,
            });
        }
        let route = match tokio::time::timeout_at(
            deadline,
            self.inner.service.resolve_connector_route(computer_id),
        )
        .await
        {
            Ok(Ok(Some(route))) => route,
            _ => {
                return Ok(wire::ConnectorInvocationResult {
                    outcome: wire::ForwardingOutcome::Unavailable as i32,
                    result: None,
                });
            }
        };
        if owner.instance_id != self.inner.instance_id
            || route.component.control_id != owner.instance_id
            || route.component.connection_id != owner.connection_id
        {
            return Ok(wire::ConnectorInvocationResult {
                outcome: wire::ForwardingOutcome::NotOwner as i32,
                result: None,
            });
        }
        let entry = self
            .inner
            .entries
            .lock()
            .map_err(|_| Status::unavailable("Control registry is unavailable"))?
            .connectors
            .get(computer_id)
            .cloned();
        let Some(entry) = entry.filter(|entry| {
            entry.prepared.owner().connection_id == owner.connection_id && !entry.calls.is_closed()
        }) else {
            return Ok(wire::ConnectorInvocationResult {
                outcome: wire::ForwardingOutcome::NotOwner as i32,
                result: None,
            });
        };
        if !matches!(
            tokio::time::timeout_at(deadline, self.inner.verify(&entry)).await,
            Ok(Ok(()))
        ) {
            return Ok(wire::ConnectorInvocationResult {
                outcome: wire::ForwardingOutcome::Unavailable as i32,
                result: None,
            });
        }
        match entry
            .calls
            .invoke(Command::Connector(Box::new(command)), deadline)
            .await
        {
            Ok(Reply::Connector(result)) => Ok(wire::ConnectorInvocationResult {
                outcome: wire::ForwardingOutcome::Completed as i32,
                result: Some(*result),
            }),
            Ok(_) => Err(Status::internal("Component reply kind does not match")),
            Err(CallFailure::Invalid) => Err(Status::invalid_argument("Component call is invalid")),
            Err(error) => Ok(wire::ConnectorInvocationResult {
                outcome: outcome(error) as i32,
                result: None,
            }),
        }
    }
}
#[derive(Debug, Clone)]
pub struct ProviderConnectionAdapter {
    inner: Arc<Inner>,
}
#[derive(Debug, Clone)]
pub struct ConnectorConnectionAdapter {
    inner: Arc<Inner>,
}
struct ComponentOutbound {
    entry: Arc<Connection>,
    receivers: OutboundReceivers,
}
impl fmt::Debug for ComponentOutbound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ComponentOutbound([REDACTED])")
    }
}
impl Stream for ComponentOutbound {
    type Item = Result<wire::ControlFrame, Status>;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.entry.calls.is_closed() {
            return Poll::Ready(None);
        }
        if let Poll::Ready(Some(frame)) = self.receivers.priority.poll_recv(cx) {
            return Poll::Ready(Some(frame));
        }
        loop {
            match self.receivers.commands.poll_recv(cx) {
                Poll::Ready(Some(command)) => {
                    if let Some(frame) = command.into_frame() {
                        return Poll::Ready(Some(Ok(frame)));
                    }
                }
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}
impl Drop for ComponentOutbound {
    fn drop(&mut self) {
        self.entry.calls.close();
    }
}
type ComponentStream =
    Pin<Box<dyn Stream<Item = Result<wire::ControlFrame, Status>> + Send + 'static>>;
fn connected(inner: &Inner, prepared: &PreparedComponentConnection) -> wire::ControlFrame {
    wire::ControlFrame {
        payload: Some(wire::control_frame::Payload::Connected(wire::Connected {
            instance_id: inner.instance_id.clone(),
            connection_id: prepared.owner().connection_id.clone(),
            heartbeat_interval_ms: inner.service.policy().heartbeat_ms,
            validity_ms: inner.service.policy().directory.component_validity_ms as u32,
        })),
    }
}
#[tonic::async_trait]
impl wire::provider_connection_server::ProviderConnection for ProviderConnectionAdapter {
    type OpenStream = ComponentStream;
    async fn open(
        &self,
        request: Request<Streaming<wire::ProviderFrame>>,
    ) -> Result<Response<Self::OpenStream>, Status> {
        let credential = input::actor(&request)?;
        let permit = self.inner.permit()?;
        let mut incoming = request.into_inner();
        let first = tokio::time::timeout(RESPONSE_BUDGET, incoming.message())
            .await
            .map_err(|_| Status::deadline_exceeded("Provider hello deadline expired"))??
            .ok_or_else(|| Status::invalid_argument("Provider hello is required"))?;
        let Some(wire::provider_frame::Payload::Hello(hello)) = first.payload else {
            return Err(Status::invalid_argument("Provider hello must be first"));
        };
        let native_scope = crate::native_resources::hello_scope(&hello)?;
        if !url_policy(&hello.connector_endpoint) {
            return Err(Status::invalid_argument(
                "Provider Connector endpoint is invalid",
            ));
        }
        let identity = self
            .inner
            .service
            .provider_identity(&hello.provider_id, &credential, AcceptanceTime::System)
            .await
            .map_err(status)?;
        if !identity.substrate.accepts_scope(&native_scope) {
            return Err(Status::permission_denied(
                "Provider native target does not match its binding",
            ));
        }
        let prepared = Arc::new(
            self.inner
                .service
                .prepare_provider_connection(
                    &hello.provider_id,
                    &credential,
                    &self.inner.instance_id,
                )
                .await
                .map_err(status)?,
        );
        let (calls, receivers) = ConnectionCalls::new(
            prepared.owner().connection_id.clone(),
            COMMAND_QUEUE,
            LOCAL_PENDING,
            self.inner.pending.clone(),
        )
        .map_err(call_status)?;
        let entry = Arc::new(Connection {
            target: Target::Provider(hello.provider_id),
            policy: Some(ProviderPolicy {
                native_scope,
                connector_endpoint: hello.connector_endpoint,
            }),
            convergence: Arc::new(Semaphore::new(1)),
            cursor: Mutex::new(None),
            cleanup: Mutex::new(HashMap::new()),
            native: Mutex::new(HashMap::new()),
            prepared,
            credential,
            calls,
            _permit: permit,
        });
        self.inner.activate(entry.clone()).await?;
        let (reports, mut report_rx) = mpsc::channel(COMMAND_QUEUE);
        let worker = entry.clone();
        let inner = self.inner.clone();
        if let Err(error)=self.inner.spawn(async move {
            loop {tokio::select! {
                _=worker.calls.closed()=>break,
                frame=report_rx.recv()=>match frame {
                    Some(frame)=>if let Err(error)=inner.provider_frame(&worker,frame).await && fatal_report(&error){break;},
                    None=>break,
                }
            }}
            worker.calls.close();
        }) {self.inner.finish(&entry).await;return Err(error);}
        let inner = self.inner.clone();
        let reader = entry.clone();
        if let Err(error)=self.inner.spawn(async move {
            let mut interval=tokio::time::interval(Duration::from_millis(u64::from(inner.service.policy().heartbeat_ms)));interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                tokio::select! {
                    _=reader.calls.closed()=>break,
                    _=interval.tick()=>if inner.verify(&reader).await.is_err(){break;},
                    frame=incoming.message()=>match frame {
                        Ok(Some(frame))=>{
                            if let Some(wire::provider_frame::Payload::Heartbeat(heartbeat))=frame.payload.as_ref() {
                                if inner.verify(&reader).await.is_err()||inner.heartbeat(&reader,heartbeat.clone()).await.is_err(){break;}
                            } else if reports.try_send(frame).is_err(){break;}
                        },
                        _=>break,
                    }
                }
            }
            inner.finish(&reader).await;
        }) {self.inner.finish(&entry).await;return Err(error);}
        Ok(Response::new(Box::pin(ComponentOutbound {
            entry,
            receivers,
        })))
    }
}
#[tonic::async_trait]
impl wire::connector_connection_server::ConnectorConnection for ConnectorConnectionAdapter {
    type OpenStream = ComponentStream;
    async fn open(
        &self,
        request: Request<Streaming<wire::ConnectorFrame>>,
    ) -> Result<Response<Self::OpenStream>, Status> {
        let credential = input::actor(&request)?;
        let permit = self.inner.permit()?;
        let mut incoming = request.into_inner();
        let first = tokio::time::timeout(RESPONSE_BUDGET, incoming.message())
            .await
            .map_err(|_| Status::deadline_exceeded("Connector hello deadline expired"))??
            .ok_or_else(|| Status::invalid_argument("Connector hello is required"))?;
        let Some(wire::connector_frame::Payload::Hello(hello)) = first.payload else {
            return Err(Status::invalid_argument("Connector hello must be first"));
        };
        let identity = self
            .inner
            .service
            .connector_identity(&hello.session_id, &credential, AcceptanceTime::System)
            .await
            .map_err(status)?;
        let prepared = Arc::new(
            self.inner
                .service
                .prepare_connector_connection(
                    &hello.session_id,
                    &credential,
                    &self.inner.instance_id,
                )
                .await
                .map_err(status)?,
        );
        let (calls, receivers) = ConnectionCalls::new(
            prepared.owner().connection_id.clone(),
            COMMAND_QUEUE,
            LOCAL_PENDING,
            self.inner.pending.clone(),
        )
        .map_err(call_status)?;
        let entry = Arc::new(Connection {
            target: Target::Connector {
                computer: identity.computer_id,
                execution: identity.execution_id,
            },
            policy: None,
            convergence: Arc::new(Semaphore::new(1)),
            cursor: Mutex::new(None),
            cleanup: Mutex::new(HashMap::new()),
            native: Mutex::new(HashMap::new()),
            prepared,
            credential,
            calls,
            _permit: permit,
        });
        self.inner.activate(entry.clone()).await?;
        let (reports, mut report_rx) = mpsc::channel(COMMAND_QUEUE);
        let worker = entry.clone();
        let inner = self.inner.clone();
        if let Err(error)=self.inner.spawn(async move {
            loop {tokio::select! {
                _=worker.calls.closed()=>break,
                frame=report_rx.recv()=>match frame {
                    Some(frame)=>if let Err(error)=inner.connector_frame(&worker,frame).await && fatal_report(&error){break;},
                    None=>break,
                }
            }}
            worker.calls.close();
        }) {self.inner.finish(&entry).await;return Err(error);}
        let inner = self.inner.clone();
        let reader = entry.clone();
        if let Err(error)=self.inner.spawn(async move {
            let mut interval=tokio::time::interval(Duration::from_millis(u64::from(inner.service.policy().heartbeat_ms)));interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                tokio::select! {
                    _=reader.calls.closed()=>break,
                    _=interval.tick()=>if inner.verify(&reader).await.is_err(){break;},
                    frame=incoming.message()=>match frame {
                        Ok(Some(frame))=>{
                            if let Some(wire::connector_frame::Payload::Heartbeat(heartbeat))=frame.payload.as_ref() {
                                if inner.verify(&reader).await.is_err()||inner.heartbeat(&reader,heartbeat.clone()).await.is_err(){break;}
                            } else if reports.try_send(frame).is_err(){break;}
                        },
                        _=>break,
                    }
                }
            }
            inner.finish(&reader).await;
        }) {self.inner.finish(&entry).await;return Err(error);}
        Ok(Response::new(Box::pin(ComponentOutbound {
            entry,
            receivers,
        })))
    }
}
