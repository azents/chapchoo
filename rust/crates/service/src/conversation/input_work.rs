//! Bounded process-owned input work; a browser waiter owns no accepted effect.
use crate::{DomainReason, ServiceError};
use std::{
    collections::HashMap,
    error::Error,
    fmt,
    future::Future,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::{
    sync::{Mutex, OwnedSemaphorePermit, Semaphore, oneshot},
    task::{Id, JoinError, JoinSet},
};

struct Completed {
    failure: Option<ServiceError>,
    _slot: OwnedSemaphorePermit,
    _bytes: OwnedSemaphorePermit,
}
#[derive(Default)]
struct Book {
    active: HashMap<Id, String>,
    admissions: HashMap<Id, Arc<AtomicBool>>,
    failures: Vec<ServiceError>,
}
#[derive(Debug)]
struct Failures(Vec<ServiceError>);
impl fmt::Display for Failures {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Conversation input work retained {} failure(s)",
            self.0.len()
        )
    }
}
impl Error for Failures {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.0.first().map(|error| error as &(dyn Error + 'static))
    }
}

pub(super) struct InputWork {
    tasks: Mutex<JoinSet<Completed>>,
    book: parking_lot::Mutex<Book>,
    closed: AtomicBool,
    slots: Arc<Semaphore>,
    bytes: Arc<Semaphore>,
    changed: tokio::sync::Notify,
}
impl fmt::Debug for InputWork {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("InputWork([REDACTED])")
    }
}
impl InputWork {
    pub(super) fn new(capacity: usize) -> Result<Self, ServiceError> {
        if !(1..=64).contains(&capacity) {
            return Err(ServiceError::domain(DomainReason::InvalidInput));
        }
        Ok(Self {
            tasks: Mutex::new(JoinSet::new()),
            book: parking_lot::Mutex::new(Book::default()),
            closed: AtomicBool::new(false),
            slots: Arc::new(Semaphore::new(capacity)),
            bytes: Arc::new(Semaphore::new(32 * 1024 * 1024)),
            changed: tokio::sync::Notify::new(),
        })
    }
    pub(super) fn admission(self: &Arc<Self>) -> Admission {
        let id = tokio::task::id();
        let settled = Arc::new(AtomicBool::new(false));
        self.book.lock().admissions.insert(id, settled.clone());
        Admission {
            work: self.clone(),
            id,
            settled,
        }
    }
    pub(super) fn mark(&self, operation: &str) {
        self.book
            .lock()
            .active
            .insert(tokio::task::id(), operation.into());
    }
    pub(super) async fn owns_after_admission(&self, operation: &str) -> bool {
        let pending = {
            let book = self.book.lock();
            if book.active.values().any(|id| id == operation) {
                return true;
            }
            book.admissions.values().cloned().collect::<Vec<_>>()
        };
        loop {
            let changed = self.changed.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            {
                let book = self.book.lock();
                if pending
                    .iter()
                    .all(|settled| settled.load(Ordering::Acquire))
                {
                    return book.active.values().any(|id| id == operation);
                }
            }
            changed.await;
        }
    }
    fn collect(&self, result: Result<(Id, Completed), JoinError>) {
        // Keep joined causes before another await, including cancelled drain reentry.
        let mut book = self.book.lock();
        match result {
            Ok((id, completed)) => {
                book.active.remove(&id);
                if let Some(error) = completed.failure {
                    book.failures.push(error);
                }
            }
            Err(error) => {
                book.active.remove(&error.id());
                book.failures.push(ServiceError::infrastructure(error));
            }
        }
    }
    fn failure(&self) -> Result<(), ServiceError> {
        let failures = std::mem::take(&mut self.book.lock().failures);
        if failures.is_empty() {
            Ok(())
        } else {
            Err(ServiceError::infrastructure(Failures(failures)))
        }
    }
    pub(super) fn reap(&self) -> Result<(), ServiceError> {
        let Ok(mut tasks) = self.tasks.try_lock() else {
            return Ok(());
        };
        while let Some(result) = tasks.try_join_next_with_id() {
            self.collect(result);
        }
        self.failure()
    }
    pub(super) async fn run<T, F, Fut>(
        self: &Arc<Self>,
        payload_bytes: usize,
        operation: F,
    ) -> Result<T, ServiceError>
    where
        T: Send + 'static,
        F: FnOnce(Arc<Self>, oneshot::Sender<Result<T, ServiceError>>) -> Fut + Send + 'static,
        Fut: Future<Output = Result<(), ServiceError>> + Send + 'static,
    {
        if payload_bytes == 0 || payload_bytes > 16 * 1024 * 1024 {
            return Err(ServiceError::domain(DomainReason::InvalidInput));
        }
        let (sender, receiver) = oneshot::channel();
        {
            // Reject before Store acceptance rather than retaining an unbounded
            // backlog of mutation futures after reporting product success.
            let mut tasks = self
                .tasks
                .try_lock()
                .map_err(|_| ServiceError::domain(DomainReason::CapacityExhausted))?;
            while let Some(result) = tasks.try_join_next_with_id() {
                self.collect(result);
            }
            self.failure()?;
            if self.closed.load(Ordering::Acquire) {
                return Err(ServiceError::domain(DomainReason::InvalidTransition));
            }
            let slot = self
                .slots
                .clone()
                .try_acquire_owned()
                .map_err(|_| ServiceError::domain(DomainReason::CapacityExhausted))?;
            let bytes = self
                .bytes
                .clone()
                .try_acquire_many_owned(
                    u32::try_from(payload_bytes)
                        .map_err(|_| ServiceError::domain(DomainReason::InvalidInput))?,
                )
                .map_err(|_| ServiceError::domain(DomainReason::CapacityExhausted))?;
            let owner = self.clone();
            tasks.spawn(async move {
                let failure = operation(owner, sender)
                    .await
                    .err()
                    .filter(|error| error.report().is_some());
                // The caller may already have received acceptance or disappeared.
                // Work and its memory budget live through actual processing/reap.
                Completed {
                    failure,
                    _slot: slot,
                    _bytes: bytes,
                }
            });
        }
        receiver.await.map_err(ServiceError::infrastructure)?
    }
    pub(super) fn close(&self) {
        self.closed.store(true, Ordering::Release);
    }
    pub(super) async fn finish(&self) -> Result<(), ServiceError> {
        self.close();
        let mut tasks = self.tasks.lock().await;
        while let Some(result) = tasks.join_next_with_id().await {
            self.collect(result);
        }
        self.failure()
    }
}
pub(super) struct Admission {
    work: Arc<InputWork>,
    id: Id,
    settled: Arc<AtomicBool>,
}
impl Drop for Admission {
    fn drop(&mut self) {
        self.work.book.lock().admissions.remove(&self.id);
        self.settled.store(true, Ordering::Release);
        self.work.changed.notify_waiters();
    }
}
#[cfg(test)]
mod tests;
