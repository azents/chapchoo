use super::*;
use crate::DomainReason;
use std::{
    collections::{HashMap, VecDeque},
    error::Error,
    sync::Weak,
    time::Duration,
};
use tokio::{
    sync::{Mutex, Notify, Semaphore, oneshot},
    task::{JoinError, JoinSet},
    time::Instant,
};

const EXECUTING: usize = 16;
const WAITING: usize = 128;
const WAITING_BYTES: usize = 8 * 1024 * 1024;
const AGENT_WAITING: usize = 8;
const AGENT_BYTES: usize = 512 * 1024;
const QUEUE_WAIT: Duration = Duration::from_secs(10);
const RESPONSE_WAIT: Duration = Duration::from_secs(50);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Authorizing,
    Queued,
    Reserved,
    Running,
    Terminal,
}
struct Entry {
    phase: Phase,
    agent: String,
    bytes: usize,
    waiting_budget: bool,
    slot: bool,
    queued_at: Option<Instant>,
    admitted: Instant,
    grant: Option<oneshot::Sender<()>>,
    wake: Arc<Notify>,
    cancellation: ToolCancellation,
    observations: Arc<Observations>,
}
#[derive(Default)]
struct Budget {
    count: usize,
    bytes: usize,
}
#[derive(Default)]
struct State {
    closing: bool,
    next_id: u64,
    executing: usize,
    waiting: Budget,
    agents: HashMap<String, Budget>,
    queues: HashMap<String, VecDeque<u64>>,
    turns: VecDeque<String>,
    entries: HashMap<u64, Entry>,
}
#[derive(Default)]
struct ObservationState {
    queue: VecDeque<ServerToolObservation>,
    expired: bool,
    final_sent: bool,
    consumed: bool,
    started: bool,
}
#[derive(Default)]
struct Observations {
    state: parking_lot::Mutex<ObservationState>,
    changed: Notify,
    consumed: Notify,
}
impl Observations {
    fn emit(&self, observation: ServerToolObservation) {
        let mut state = self.state.lock();
        if state.expired || state.final_sent {
            return;
        }
        if matches!(observation, ServerToolObservation::Started) {
            state.started = true;
        }
        if matches!(observation, ServerToolObservation::Final(_)) {
            state.final_sent = true;
        }
        // The owner emits each of Waiting/Started/Final at most once.
        state.queue.push_back(observation);
        drop(state);
        self.changed.notify_one();
    }
    fn expire(&self, outcome: ServerToolOutcome) {
        let mut state = self.state.lock();
        if state.consumed || state.expired {
            return;
        }
        state.expired = true;
        state.final_sent = true;
        // Release any protected body retained for an unresponsive observer.
        state
            .queue
            .retain(|event| !matches!(event, ServerToolObservation::Final(_)));
        state.queue.push_back(ServerToolObservation::Final(outcome));
        drop(state);
        self.changed.notify_one();
    }
    async fn observed_or_expired(&self, deadline: Instant) {
        loop {
            let consumed = self.consumed.notified();
            tokio::pin!(consumed);
            consumed.as_mut().enable();
            if self.state.lock().consumed {
                return;
            }
            if tokio::time::timeout_at(deadline, consumed).await.is_err() {
                return;
            }
        }
    }
}
struct Inner {
    backend: Arc<dyn ServerToolBackend>,
    state: parking_lot::Mutex<State>,
    tasks: Mutex<JoinSet<()>>,
    failures: parking_lot::Mutex<Vec<ServiceError>>,
    // Bound pre-authorization and retained responses as well as actual calls.
    // This is no second waiting scheduler: acquisition rejects immediately.
    owners: Arc<Semaphore>,
    payload: Arc<Semaphore>,
}
#[derive(Clone)]
pub struct ServerTools {
    inner: Arc<Inner>,
}
pub struct ToolInvocation {
    control: ToolInvocationControl,
    observations: Arc<Observations>,
    deadline: Instant,
    finished: bool,
}
#[derive(Clone)]
pub struct ToolInvocationControl {
    inner: Weak<Inner>,
    id: u64,
}
redacted!(ServerTools, ToolInvocation, ToolInvocationControl);
impl ToolInvocationControl {
    /// This handle is sealed to one call; callers cannot choose an invocation ID.
    pub fn cancel(&self) {
        if let Some(inner) = self.inner.upgrade() {
            inner.stop_before_start(self.id, ServerToolRejection::Cancelled);
        }
    }
}
impl ToolInvocation {
    pub fn control(&self) -> ToolInvocationControl {
        self.control.clone()
    }
    pub async fn next(&mut self) -> Option<ServerToolObservation> {
        if self.finished {
            return None;
        }
        loop {
            let changed = self.observations.changed.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            if Instant::now() >= self.deadline {
                if let Some(inner) = self.control.inner.upgrade() {
                    inner.expire(self.control.id, &self.observations);
                } else {
                    let started = self.observations.state.lock().started;
                    self.observations.expire(if started {
                        ServerToolOutcome::Uncertain
                    } else {
                        ServerToolOutcome::NotExecuted(ServerToolRejection::ResponseExpired)
                    });
                }
            }
            let event = {
                let mut state = self.observations.state.lock();
                let event = state.queue.pop_front();
                if matches!(event, Some(ServerToolObservation::Final(_))) {
                    state.consumed = true;
                    self.finished = true;
                    self.observations.consumed.notify_one();
                }
                event
            };
            if event.is_some() {
                return event;
            }
            let _ = tokio::time::timeout_at(self.deadline, changed).await;
        }
    }
}

impl Drop for ToolInvocation {
    fn drop(&mut self) {
        // Losing an observer releases response retention, not the effect owner
        // and not an explicit request cancellation notification.
        let mut state = self.observations.state.lock();
        state.consumed = true;
        state.queue.clear();
        drop(state);
        self.observations.consumed.notify_one();
    }
}

impl ServerTools {
    pub fn new(backend: Arc<dyn ServerToolBackend>) -> Self {
        Self {
            inner: Arc::new(Inner {
                backend,
                state: parking_lot::Mutex::new(State::default()),
                tasks: Mutex::new(JoinSet::new()),
                failures: parking_lot::Mutex::new(Vec::new()),
                owners: Arc::new(Semaphore::new(EXECUTING + WAITING)),
                payload: Arc::new(Semaphore::new(WAITING_BYTES)),
            }),
        }
    }
    pub fn submit(&self, request: ServerToolRequest) -> Result<ToolInvocation, ServiceError> {
        if request.name.is_empty()
            || request.name.len() > 128
            || request.arguments.len() > MAX_TOOL_PAYLOAD
        {
            return Err(ServiceError::domain(DomainReason::InvalidInput));
        }
        let mut tasks = self
            .inner
            .tasks
            .try_lock()
            .map_err(|_| ServiceError::domain(DomainReason::CapacityExhausted))?;
        self.inner.reap(&mut tasks);
        self.inner.failures()?;
        let owner = self
            .inner
            .owners
            .clone()
            .try_acquire_owned()
            .map_err(|_| ServiceError::domain(DomainReason::CapacityExhausted))?;
        let bytes = self
            .inner
            .payload
            .clone()
            .try_acquire_many_owned(
                u32::try_from(request.arguments.len().max(1))
                    .map_err(|_| ServiceError::domain(DomainReason::InvalidInput))?,
            )
            .map_err(|_| ServiceError::domain(DomainReason::CapacityExhausted))?;
        let observations = Arc::new(Observations::default());
        let admitted = Instant::now();
        let id = {
            let mut state = self.inner.state.lock();
            if state.closing {
                return Err(ServiceError::domain(DomainReason::InvalidTransition));
            }
            state.next_id = state
                .next_id
                .checked_add(1)
                .ok_or_else(|| ServiceError::domain(DomainReason::CapacityExhausted))?;
            let id = state.next_id;
            state.entries.insert(
                id,
                Entry {
                    phase: Phase::Authorizing,
                    agent: request.scope.agent_id.clone(),
                    bytes: request.arguments.len(),
                    waiting_budget: false,
                    slot: false,
                    queued_at: None,
                    admitted,
                    grant: None,
                    wake: Arc::new(Notify::new()),
                    cancellation: ToolCancellation::default(),
                    observations: observations.clone(),
                },
            );
            id
        };
        let inner = self.inner.clone();
        tasks.spawn(async move {
            let _owner = owner;
            let _bytes = bytes;
            inner.run(id, request).await;
        });
        Ok(ToolInvocation {
            control: ToolInvocationControl {
                inner: Arc::downgrade(&self.inner),
                id,
            },
            observations,
            deadline: admitted + RESPONSE_WAIT,
            finished: false,
        })
    }
    pub async fn catalog(&self, scope: ServerToolScope) -> Result<Vec<u8>, ServiceError> {
        let (sender, receiver) = oneshot::channel();
        {
            let mut tasks = self
                .inner
                .tasks
                .try_lock()
                .map_err(|_| ServiceError::domain(DomainReason::CapacityExhausted))?;
            self.inner.reap(&mut tasks);
            self.inner.failures()?;
            if self.inner.state.lock().closing {
                return Err(ServiceError::domain(DomainReason::InvalidTransition));
            }
            let permit = self
                .inner
                .owners
                .clone()
                .try_acquire_owned()
                .map_err(|_| ServiceError::domain(DomainReason::CapacityExhausted))?;
            let inner = self.inner.clone();
            tasks.spawn(async move {
                let _permit = permit;
                let result = async {
                    inner.backend.authorize(&scope).await?;
                    let bytes = inner.backend.catalog()?;
                    #[derive(serde::Deserialize)]
                    struct Catalog {
                        tools: Vec<serde_json::Map<String, serde_json::Value>>,
                    }
                    if bytes.len() > MAX_TOOL_PAYLOAD {
                        return Err(ServiceError::domain(DomainReason::InvalidInput));
                    }
                    let catalog: Catalog =
                        serde_json::from_slice(&bytes).map_err(ServiceError::infrastructure)?;
                    if catalog.tools.len() > 32 {
                        return Err(ServiceError::domain(DomainReason::InvalidInput));
                    }
                    Ok(bytes)
                }
                .await;
                if let Err(Err(error)) = sender.send(result) {
                    inner.retain(error);
                }
            });
        }
        receiver.await.map_err(ServiceError::infrastructure)?
    }
    pub fn close(&self) {
        self.inner.close();
    }
    #[cfg(test)]
    pub(super) fn executing(&self) -> usize {
        self.inner.state.lock().executing
    }
    /// Keeping the JoinSet in its mutex makes a cancelled drain re-enterable.
    pub async fn finish(&self) -> Result<(), ServiceError> {
        self.close();
        let mut tasks = self.inner.tasks.lock().await;
        while let Some(result) = tasks.join_next().await {
            self.inner.collect(result);
        }
        self.inner.failures()
    }
}

enum Admission {
    Direct,
    Waiting(oneshot::Receiver<()>, Arc<Notify>),
}
impl Inner {
    fn close(&self) {
        let ids = {
            let mut state = self.state.lock();
            state.closing = true;
            state.entries.keys().copied().collect::<Vec<_>>()
        };
        for id in ids {
            self.stop_before_start(id, ServerToolRejection::Closing);
        }
    }
    fn retain(&self, error: ServiceError) {
        if error.report().is_some() {
            self.failures.lock().push(error);
        }
    }
    fn collect(&self, result: Result<(), JoinError>) {
        if let Err(error) = result {
            self.close();
            self.retain(ServiceError::infrastructure(error));
        }
    }
    fn reap(&self, tasks: &mut JoinSet<()>) {
        while let Some(result) = tasks.try_join_next() {
            self.collect(result);
        }
    }
    fn failures(&self) -> Result<(), ServiceError> {
        #[derive(Debug)]
        struct Failures(Vec<ServiceError>);
        impl fmt::Display for Failures {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "Server tool owner retained {} failure(s)", self.0.len())
            }
        }
        impl Error for Failures {
            fn source(&self) -> Option<&(dyn Error + 'static)> {
                self.0.first().map(|error| error as &(dyn Error + 'static))
            }
        }
        let failures = std::mem::take(&mut *self.failures.lock());
        if failures.is_empty() {
            Ok(())
        } else {
            Err(ServiceError::infrastructure(Failures(failures)))
        }
    }
    fn stop_before_start(&self, id: u64, reason: ServerToolRejection) {
        let mut state = self.state.lock();
        let Some(entry) = state.entries.get_mut(&id) else {
            return;
        };
        if reason == ServerToolRejection::Cancelled {
            entry.cancellation.cancel();
        }
        if entry.phase != Phase::Running && entry.phase != Phase::Terminal {
            entry.phase = Phase::Terminal;
            entry.observations.emit(ServerToolObservation::Final(
                ServerToolOutcome::NotExecuted(reason),
            ));
            entry.wake.notify_one();
        }
    }
    fn expire(&self, id: u64, observations: &Observations) {
        self.stop_before_start(id, ServerToolRejection::ResponseExpired);
        let started = observations.state.lock().started;
        observations.expire(if started {
            ServerToolOutcome::Uncertain
        } else {
            ServerToolOutcome::NotExecuted(ServerToolRejection::ResponseExpired)
        });
    }
    fn queue(&self, id: u64) -> Result<Admission, ServerToolRejection> {
        let mut state = self.state.lock();
        if state.closing {
            return Err(ServerToolRejection::Closing);
        }
        let entry = state.entries.get(&id).ok_or(ServerToolRejection::Closing)?;
        if entry.phase == Phase::Terminal {
            return Err(ServerToolRejection::Cancelled);
        }
        if state.executing < EXECUTING {
            state.executing += 1;
            let entry = state
                .entries
                .get_mut(&id)
                .ok_or(ServerToolRejection::Closing)?;
            entry.slot = true;
            entry.phase = Phase::Reserved;
            return Ok(Admission::Direct);
        }
        let agent = entry.agent.clone();
        let bytes = entry.bytes;
        let agent_budget = state.agents.get(&agent);
        if state.waiting.count >= WAITING
            || state.waiting.bytes + bytes > WAITING_BYTES
            || agent_budget
                .is_some_and(|v| v.count >= AGENT_WAITING || v.bytes + bytes > AGENT_BYTES)
        {
            return Err(ServerToolRejection::Capacity);
        }
        state.waiting.count += 1;
        state.waiting.bytes += bytes;
        let budget = state.agents.entry(agent.clone()).or_default();
        budget.count += 1;
        budget.bytes += bytes;
        let queue = state.queues.entry(agent.clone()).or_default();
        let new_turn = queue.is_empty();
        queue.push_back(id);
        if new_turn {
            state.turns.push_back(agent);
        }
        let (grant, receiver) = oneshot::channel();
        let entry = state
            .entries
            .get_mut(&id)
            .ok_or(ServerToolRejection::Closing)?;
        entry.waiting_budget = true;
        entry.queued_at = Some(Instant::now());
        entry.phase = Phase::Queued;
        entry.grant = Some(grant);
        entry.observations.emit(ServerToolObservation::Waiting);
        Ok(Admission::Waiting(receiver, entry.wake.clone()))
    }
    fn begin(&self, id: u64) -> bool {
        let mut state = self.state.lock();
        let closing = state.closing;
        let Some(entry) = state.entries.get_mut(&id) else {
            return false;
        };
        if entry.phase == Phase::Terminal {
            return false;
        }
        let response_expired = Instant::now() >= entry.admitted + RESPONSE_WAIT;
        let queue_expired = entry
            .queued_at
            .is_some_and(|at| Instant::now() >= at + QUEUE_WAIT);
        if closing || response_expired || queue_expired {
            entry.phase = Phase::Terminal;
            entry.observations.emit(ServerToolObservation::Final(
                ServerToolOutcome::NotExecuted(if closing {
                    ServerToolRejection::Closing
                } else if response_expired {
                    ServerToolRejection::ResponseExpired
                } else {
                    ServerToolRejection::QueueExpired
                }),
            ));
            return false;
        }
        entry.phase = Phase::Running;
        entry.observations.emit(ServerToolObservation::Started);
        true
    }
    fn release_budget(state: &mut State, entry: &mut Entry) {
        if !entry.waiting_budget {
            return;
        }
        entry.waiting_budget = false;
        state.waiting.count -= 1;
        state.waiting.bytes -= entry.bytes;
        if let Some(budget) = state.agents.get_mut(&entry.agent) {
            budget.count -= 1;
            budget.bytes -= entry.bytes;
            if budget.count == 0 {
                state.agents.remove(&entry.agent);
            }
        }
    }
    fn completed(&self, id: u64) {
        let mut state = self.state.lock();
        if let Some(mut entry) = state.entries.remove(&id) {
            Self::release_budget(&mut state, &mut entry);
            if entry.slot {
                state.executing -= 1;
            }
        }
        while !state.closing && state.executing < EXECUTING {
            let Some(agent) = state.turns.pop_front() else {
                break;
            };
            let Some(queue) = state.queues.get_mut(&agent) else {
                continue;
            };
            let Some(next) = queue.pop_front() else {
                continue;
            };
            if queue.is_empty() {
                state.queues.remove(&agent);
            } else {
                state.turns.push_back(agent);
            }
            let Some(mut entry) = state.entries.remove(&next) else {
                continue;
            };
            if entry.phase == Phase::Queued {
                Self::release_budget(&mut state, &mut entry);
                entry.phase = Phase::Reserved;
                entry.slot = true;
                state.executing += 1;
                if let Some(grant) = entry.grant.take() {
                    let _ = grant.send(());
                }
            }
            state.entries.insert(next, entry);
        }
    }
    async fn process(&self, id: u64, request: &ServerToolRequest) -> Result<(), ServiceError> {
        let observations = {
            let state = self.state.lock();
            let Some(entry) = state.entries.get(&id) else {
                return Ok(());
            };
            entry.observations.clone()
        };
        if let Err(error) = self.backend.authorize(&request.scope).await {
            self.stop_before_start(id, ServerToolRejection::Authority);
            return Err(error);
        }
        match self.backend.validate(&request.name, &request.arguments) {
            Ok(()) => (),
            Err(ServerToolValidation::UnknownTool) => {
                self.stop_before_start(id, ServerToolRejection::UnknownTool);
                return Ok(());
            }
            Err(ServerToolValidation::InvalidArguments) => {
                observations.emit(ServerToolObservation::Final(ServerToolOutcome::Result(
                    tool_error("Invalid tool arguments")?,
                )));
                return Ok(());
            }
        }
        match self.queue(id) {
            Err(reason) => {
                self.stop_before_start(id, reason);
                return Ok(());
            }
            Ok(Admission::Direct) => (),
            Ok(Admission::Waiting(grant, wake)) => {
                let queued_at = self
                    .state
                    .lock()
                    .entries
                    .get(&id)
                    .and_then(|entry| entry.queued_at);
                let Some(queued_at) = queued_at else {
                    return Ok(());
                };
                tokio::select! {
                    _ = grant => (),
                    _ = wake.notified() => return Ok(()),
                    _ = tokio::time::sleep_until(queued_at + QUEUE_WAIT) => {
                        self.stop_before_start(id, ServerToolRejection::QueueExpired);
                        return Ok(());
                    }
                }
            }
        }
        // Authorization can change after discovery/admission and lock waits.
        if let Err(error) = self.backend.authorize(&request.scope).await {
            self.stop_before_start(id, ServerToolRejection::Authority);
            return Err(error);
        }
        if !self.begin(id) {
            return Ok(());
        }
        let cancellation = self
            .state
            .lock()
            .entries
            .get(&id)
            .map(|entry| entry.cancellation.clone())
            .unwrap_or_default();
        let result = self.backend.execute(request, cancellation).await;
        // The domain operation rechecks at commit too. Disclosure is independently
        // current, and its denial says nothing about rollback of an earlier write.
        if let Err(error) = self.backend.authorize(&request.scope).await {
            observations.emit(ServerToolObservation::Final(ServerToolOutcome::Uncertain));
            if let Err(execution) = result {
                self.retain(execution);
            }
            return Err(error);
        }
        let outcome = match result {
            Ok(bytes) if bytes.len() <= MAX_TOOL_PAYLOAD => ServerToolOutcome::Result(bytes),
            Ok(_) => ServerToolOutcome::UncertainRepresentation,
            Err(error) if error.report().is_none() => {
                ServerToolOutcome::Result(tool_error("Tool execution was rejected")?)
            }
            Err(error) => {
                observations.emit(ServerToolObservation::Final(ServerToolOutcome::Uncertain));
                return Err(error);
            }
        };
        observations.emit(ServerToolObservation::Final(outcome));
        Ok(())
    }
    async fn run(self: Arc<Self>, id: u64, request: ServerToolRequest) {
        struct Completion {
            inner: Arc<Inner>,
            id: u64,
        }
        impl Drop for Completion {
            fn drop(&mut self) {
                if std::thread::panicking() {
                    // Do not hand off queued effects after an unexpected owner
                    // failure; retain the JoinError for the composition drain.
                    self.inner.close();
                }
                self.inner.completed(self.id);
            }
        }
        let completion = Completion {
            inner: self.clone(),
            id,
        };
        let (observations, deadline) = {
            let state = self.state.lock();
            let Some(entry) = state.entries.get(&id) else {
                return;
            };
            (entry.observations.clone(), entry.admitted + RESPONSE_WAIT)
        };
        let result = {
            let physical = self.process(id, &request);
            tokio::pin!(physical);
            tokio::select! {
                result = &mut physical => result,
                _ = tokio::time::sleep_until(deadline) => {
                    self.expire(id, &observations);
                    // The observer's deadline never drops started work.
                    physical.await
                }
            }
        };
        drop(request);
        drop(completion);
        if let Err(error) = result {
            self.retain(error);
        }
        observations.observed_or_expired(deadline).await;
        let started = observations.state.lock().started;
        observations.expire(if started {
            ServerToolOutcome::Uncertain
        } else {
            ServerToolOutcome::NotExecuted(ServerToolRejection::ResponseExpired)
        });
    }
}
fn tool_error(message: &str) -> Result<Vec<u8>, ServiceError> {
    serde_json::to_vec(&serde_json::json!({
        "content":[{"type":"text","text":message}],"isError":true
    }))
    .map_err(ServiceError::infrastructure)
}
