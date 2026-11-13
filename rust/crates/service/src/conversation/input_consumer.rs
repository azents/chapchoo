//! Queue ownership renewal is co-owned with processing, never detached from it.
use crate::{ComputerPolicy, ServiceError};
use chapchoo_persistence::{ConversationInputQueueState, Store};
use std::{error::Error, fmt, future::Future, sync::Arc, time::Duration};
use tokio::{
    sync::{Mutex, watch},
    task::JoinHandle,
};

struct Failures(Vec<ServiceError>);
impl fmt::Debug for Failures {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "InputConsumerFailures({})", self.0.len())
    }
}
impl fmt::Display for Failures {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Input consumer retained {} failure(s)", self.0.len())
    }
}
impl Error for Failures {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.0.first().map(|e| e as &(dyn Error + 'static))
    }
}
pub(super) fn combine(
    one: Result<(), ServiceError>,
    two: Result<(), ServiceError>,
) -> Result<(), ServiceError> {
    let errors = [one, two]
        .into_iter()
        .filter_map(Result::err)
        .collect::<Vec<_>>();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(ServiceError::infrastructure(Failures(errors)))
    }
}
pub(super) struct Consumer {
    policy: ComputerPolicy,
    stop: watch::Sender<bool>,
    task: Mutex<Option<JoinHandle<Result<(), ServiceError>>>>,
    // Coalesce a continuing database outage until an ordinary handling boundary
    // takes its first retained cause; retries do not grow an unbounded log.
    failure: Arc<parking_lot::Mutex<Option<ServiceError>>>,
}
impl Consumer {
    pub(super) fn new(store: Store, policy: ComputerPolicy) -> Result<Self, ServiceError> {
        let policy = policy.validate()?;
        let handle = tokio::runtime::Handle::try_current().map_err(ServiceError::infrastructure)?;
        let (stop, mut stopped) = watch::channel(false);
        let failure = Arc::new(parking_lot::Mutex::new(None));
        let failed = failure.clone();
        let task = handle.spawn(async move {
            let mut cadence =
                tokio::time::interval(Duration::from_millis(u64::from(policy.heartbeat_ms)));
            cadence.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                tokio::select! {
                    changed = stopped.changed() => {
                        if changed.is_err() || *stopped.borrow() { return Ok(()); }
                    }
                    _ = cadence.tick() => {
                        if let Err(error) = store.expire_conversation_input_consumers().await {
                            let mut retained = failed.lock();
                            if retained.is_none() { *retained = Some(error.into()); }
                        }
                    }
                }
            }
        });
        Ok(Self {
            policy,
            stop,
            task: Mutex::new(Some(task)),
            failure,
        })
    }
    pub(super) fn policy(&self) -> chapchoo_persistence::DirectoryPolicy {
        self.policy.directory
    }
    pub(super) fn check(&self) -> Result<(), ServiceError> {
        self.failure.lock().take().map_or(Ok(()), Err)
    }
    pub(super) async fn run(
        &self,
        store: Store,
        conversation: String,
        id: String,
        processing: impl Future<Output = Result<(), ServiceError>> + Send,
    ) -> Result<(), ServiceError> {
        let result = {
            tokio::pin!(processing);
            let mut cadence =
                tokio::time::interval(Duration::from_millis(u64::from(self.policy.heartbeat_ms)));
            cadence.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            let (stop, mut stopped) = watch::channel(false);
            let renewing = async {
                loop {
                    tokio::select! {
                        _ = stopped.changed() => return Ok(()),
                        _ = cadence.tick() => {
                            match store.renew_conversation_input_consumer(&conversation, &id, self.policy.directory).await {
                                Ok(ConversationInputQueueState::Pending) => {}
                                Ok(ConversationInputQueueState::Absent | ConversationInputQueueState::Error) => return Ok(()),
                                Err(error) => return Err(error.into()),
                            }
                        }
                    }
                }
            };
            tokio::pin!(renewing);
            // Keep polling the processor while renewal waits for its DB lock.
            // Pausing it inside a cadence branch could strand its own transaction.
            tokio::select! {
                result = &mut processing => {
                    let _ = stop.send(true);
                    // A started renewal owns its DB work through completion.
                    combine(result, renewing.await)
                }
                result = &mut renewing => result,
            }
        };
        combine(
            result,
            store
                .fail_conversation_input(&conversation, &id)
                .await
                .map_err(Into::into),
        )
    }
    pub(super) async fn finish(&self) -> Result<(), ServiceError> {
        let _ = self.stop.send(true);
        let mut task = self.task.lock().await;
        let joined = match task.as_mut() {
            Some(handle) => {
                let result = handle
                    .await
                    .map_err(ServiceError::infrastructure)
                    .and_then(|value| value);
                *task = None;
                result
            }
            None => Ok(()),
        };
        combine(joined, self.check())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        future::Future,
        task::{Context, Waker},
    };

    #[tokio::test]
    async fn cancelled_finish_keeps_reaper_join_and_failure_owned() -> Result<(), ServiceError> {
        let (release, blocked) = tokio::sync::oneshot::channel::<()>();
        let (stop, _stopped) = watch::channel(false);
        let consumer = Consumer {
            policy: ComputerPolicy::default(),
            stop,
            task: Mutex::new(Some(tokio::spawn(async move {
                blocked.await.map_err(ServiceError::infrastructure)?;
                Err(ServiceError::domain(crate::DomainReason::InvalidTransition))
            }))),
            failure: Arc::new(parking_lot::Mutex::new(None)),
        };
        {
            let mut finishing = std::pin::pin!(consumer.finish());
            assert!(
                finishing
                    .as_mut()
                    .poll(&mut Context::from_waker(Waker::noop()))
                    .is_pending()
            );
        }
        assert!(consumer.task.lock().await.is_some());
        release
            .send(())
            .map_err(|_| ServiceError::domain(crate::DomainReason::InvalidTransition))?;
        assert!(consumer.finish().await.is_err());
        assert!(consumer.task.lock().await.is_none());
        consumer.finish().await?;
        Ok(())
    }
}
