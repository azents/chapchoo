use chapchoo_crypto::{PasswordCost, PasswordHash, PasswordHasher};
use chapchoo_domain::Password;
use chapchoo_persistence::{CredentialSnapshot, StoreError, VerifiedCredential};
use std::{fmt, sync::Arc};
use tokio::sync::Semaphore;
use zeroize::Zeroizing;

use crate::{DomainReason, ServiceError};

#[derive(Clone)]
pub struct PasswordWork {
    capacity: Arc<Semaphore>,
    cost: PasswordCost,
}

impl fmt::Debug for PasswordWork {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PasswordWork")
            .field("available_capacity", &self.capacity.available_permits())
            .finish_non_exhaustive()
    }
}

impl PasswordWork {
    pub fn new(capacity: usize, cost: PasswordCost) -> Result<Self, ServiceError> {
        if capacity == 0 || capacity > 1024 {
            return Err(ServiceError::domain(DomainReason::InvalidInput));
        }
        Ok(Self {
            capacity: Arc::new(Semaphore::new(capacity)),
            cost,
        })
    }

    async fn run<T, F>(&self, work: F) -> Result<T, ServiceError>
    where
        T: Send + 'static,
        F: FnOnce(PasswordHasher) -> Result<T, ServiceError> + Send + 'static,
    {
        // There is no unbounded wait queue. A cancelled caller cannot relinquish
        // capacity while already-started CPU work is still executing.
        let permit = self
            .capacity
            .clone()
            .try_acquire_owned()
            .map_err(|_| ServiceError::domain(DomainReason::CapacityExhausted))?;
        let cost = self.cost;
        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            work(PasswordHasher::new(cost))
        })
        .await
        .map_err(ServiceError::infrastructure)?
    }

    pub async fn hash(&self, password: Password) -> Result<PasswordHash, ServiceError> {
        self.run(move |hasher| hasher.hash(&password).map_err(ServiceError::infrastructure))
            .await
    }

    pub async fn verify(
        &self,
        snapshot: CredentialSnapshot,
        input: Zeroizing<String>,
    ) -> Result<VerifiedCredential, ServiceError> {
        self.run(move |hasher| {
            snapshot.verify(&input, &hasher).map_err(|error| {
                if matches!(error, StoreError::Unauthenticated) {
                    ServiceError::domain(DomainReason::InvalidCredentials)
                } else {
                    error.into()
                }
            })
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{error::Error, sync::mpsc};

    #[tokio::test]
    async fn cancelled_call_keeps_capacity_until_its_worker_finishes() -> Result<(), Box<dyn Error>>
    {
        let work = PasswordWork::new(1, PasswordCost::new(10)?)?;
        let (entered_tx, entered_rx) = tokio::sync::oneshot::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let worker = work.clone();
        let caller = tokio::spawn(async move {
            worker
                .run(move |_| {
                    entered_tx
                        .send(())
                        .map_err(|_| ServiceError::domain(DomainReason::Internal))?;
                    release_rx.recv().map_err(ServiceError::infrastructure)?;
                    Ok(())
                })
                .await
        });
        entered_rx.await?;
        caller.abort();
        let cancelled = caller.await;
        assert!(cancelled.is_err_and(|error| error.is_cancelled()));
        assert_eq!(work.capacity.available_permits(), 0);
        let blocked = work.run(|_| Ok(())).await;
        assert!(blocked.is_err_and(|error| error.reason() == DomainReason::CapacityExhausted));
        release_tx.send(())?;
        // This acquisition observes actual worker completion, not elapsed time.
        let permit = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            work.capacity.clone().acquire_owned(),
        )
        .await??;
        drop(permit);
        work.run(|_| Ok(())).await?;
        Ok(())
    }
}
