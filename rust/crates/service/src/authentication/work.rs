//! One process-owned set of bounded acquisition calls, independent of RPC waiters.

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
    orphaned: Option<ServiceError>,
    _permit: OwnedSemaphorePermit,
}
#[derive(Debug)]
struct WorkFailures(Vec<ServiceError>);
impl fmt::Display for WorkFailures {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Authentication work retained {} failure(s)",
            self.0.len()
        )
    }
}
impl Error for WorkFailures {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.0.first().map(|error| error as &(dyn Error + 'static))
    }
}
#[derive(Default)]
struct Book {
    active: HashMap<Id, String>,
    failures: Vec<ServiceError>,
}

pub(super) struct AuthenticationWork {
    tasks: Mutex<JoinSet<Completed>>,
    book: parking_lot::Mutex<Book>,
    closed: AtomicBool,
    capacity: Arc<Semaphore>,
    #[cfg(test)]
    collected: tokio::sync::Notify,
}
impl fmt::Debug for AuthenticationWork {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("AuthenticationWork([REDACTED])")
    }
}
impl AuthenticationWork {
    pub(super) fn new(capacity: usize) -> Result<Self, ServiceError> {
        if !(1..=64).contains(&capacity) {
            return Err(ServiceError::domain(DomainReason::InvalidInput));
        }
        Ok(Self {
            tasks: Mutex::new(JoinSet::new()),
            book: parking_lot::Mutex::new(Book::default()),
            closed: AtomicBool::new(false),
            capacity: Arc::new(Semaphore::new(capacity)),
            #[cfg(test)]
            collected: tokio::sync::Notify::new(),
        })
    }
    pub(super) fn mark(&self, attempt: String) {
        self.book.lock().active.insert(tokio::task::id(), attempt);
    }
    pub(super) fn owns(&self, attempt: &str) -> bool {
        self.book.lock().active.values().any(|id| id == attempt)
    }
    fn collect(&self, result: Result<(Id, Completed), JoinError>) {
        // Record a joined failure synchronously before any later await. A
        // cancelled finish cannot lose a cause already removed from the JoinSet.
        let mut book = self.book.lock();
        match result {
            Ok((id, completed)) => {
                book.active.remove(&id);
                if let Some(error) = completed.orphaned {
                    book.failures.push(error);
                }
            }
            Err(error) => {
                book.active.remove(&error.id());
                book.failures.push(ServiceError::infrastructure(error));
            }
        }
        #[cfg(test)]
        self.collected.notify_one();
    }
    fn take_failure(&self) -> Result<(), ServiceError> {
        let failures = std::mem::take(&mut self.book.lock().failures);
        if failures.is_empty() {
            Ok(())
        } else {
            Err(ServiceError::infrastructure(WorkFailures(failures)))
        }
    }
    pub(super) fn reap(&self) -> Result<(), ServiceError> {
        let mut tasks = self
            .tasks
            .try_lock()
            .map_err(|_| ServiceError::domain(DomainReason::AuthenticationCapacityExhausted))?;
        while let Some(result) = tasks.try_join_next_with_id() {
            self.collect(result);
        }
        self.take_failure()
    }
    pub(super) async fn run<T, F, Fut>(self: &Arc<Self>, operation: F) -> Result<T, ServiceError>
    where
        T: Send + 'static,
        F: FnOnce(Arc<Self>) -> Fut + Send + 'static,
        Fut: Future<Output = Result<T, ServiceError>> + Send + 'static,
    {
        let (sender, receiver) = oneshot::channel();
        {
            // Reject contention/overflow before any Store admission, rather than
            // retaining an unbounded queue of accepted mutation futures.
            let mut tasks = self
                .tasks
                .try_lock()
                .map_err(|_| ServiceError::domain(DomainReason::AuthenticationCapacityExhausted))?;
            while let Some(result) = tasks.try_join_next_with_id() {
                self.collect(result);
            }
            self.take_failure()?;
            if self.closed.load(Ordering::Acquire) {
                return Err(ServiceError::domain(DomainReason::InvalidTransition));
            }
            let permit =
                self.capacity.clone().try_acquire_owned().map_err(|_| {
                    ServiceError::domain(DomainReason::AuthenticationCapacityExhausted)
                })?;
            let owner = self.clone();
            tasks.spawn(async move {
                let outcome = operation(owner).await;
                let orphaned = match sender.send(outcome) {
                    Err(Err(error)) if error.report().is_some() => Some(error),
                    _ => None,
                };
                Completed {
                    orphaned,
                    _permit: permit,
                }
            });
        }
        // Dropping this receiver never cancels the owned task.
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
        self.take_failure()
    }
}
pub(super) fn combined(primary: ServiceError, cleanup: ServiceError) -> ServiceError {
    ServiceError::infrastructure(WorkFailures(vec![primary, cleanup]))
}

#[cfg(test)]
mod tests;
