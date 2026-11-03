//! SDK task ownership is independent of a call waiter and stream incarnation.
use std::{future::Future, sync::Arc};
use tokio::{
    sync::{Semaphore, oneshot},
    task::JoinSet,
};

#[derive(thiserror::Error)]
pub enum EffectError {
    #[error("A native lifecycle effect is already in flight")]
    Busy,
    #[error("A native lifecycle effect task failed")]
    Task(#[source] tokio::task::JoinError),
    #[error("Multiple native lifecycle effect tasks failed")]
    Tasks(#[source] EffectFailures),
}
impl std::fmt::Debug for EffectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("EffectError([REDACTED])")
    }
}

pub struct EffectFailures(Vec<tokio::task::JoinError>);
impl std::fmt::Debug for EffectFailures {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("EffectFailures([REDACTED])")
    }
}
impl std::fmt::Display for EffectFailures {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Multiple native lifecycle effect tasks failed")
    }
}
impl std::error::Error for EffectFailures {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.first().map(|cause| cause as &dyn std::error::Error)
    }
}
/// Retain this owner across stream reconnects. Never replace it merely because a
/// response deadline/cancel/connection has ended. The process supervises drain.
#[derive(Debug)]
pub struct Effects {
    permit: Arc<Semaphore>,
    tasks: JoinSet<()>,
    failures: Vec<tokio::task::JoinError>,
}
impl Default for Effects {
    fn default() -> Self {
        Self::new()
    }
}
impl Effects {
    pub fn new() -> Self {
        Self {
            permit: Arc::new(Semaphore::new(1)),
            tasks: JoinSet::new(),
            failures: Vec::new(),
        }
    }
    pub fn active(&self) -> bool {
        self.permit.available_permits() == 0
    }
    pub fn start<F, T>(&mut self, effect: F) -> Result<oneshot::Receiver<T>, EffectError>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let permit = self
            .permit
            .clone()
            .try_acquire_owned()
            .map_err(|_| EffectError::Busy)?;
        let (sender, receiver) = oneshot::channel();
        self.tasks.spawn(async move {
            let _permit = permit;
            let result = effect.await;
            let _ = sender.send(result);
        });
        Ok(receiver)
    }
    pub fn reap_completed(&mut self) -> Result<(), EffectError> {
        while let Some(result) = self.tasks.try_join_next() {
            if let Err(error) = result {
                self.failures.push(error);
            }
        }
        self.take_failures()
    }
    pub async fn drain(&mut self) -> Result<(), EffectError> {
        while let Some(result) = self.tasks.join_next().await {
            if let Err(error) = result {
                self.failures.push(error);
            }
        }
        self.take_failures()
    }
    fn take_failures(&mut self) -> Result<(), EffectError> {
        let mut failures = std::mem::take(&mut self.failures);
        match failures.len() {
            0 => Ok(()),
            1 => Err(EffectError::Task(failures.remove(0))),
            _ => Err(EffectError::Tasks(EffectFailures(failures))),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn cancelled_drain_keeps_all_collected_join_failures_until_reentry()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut effects = Effects::new();
        let first = effects.start(std::future::pending::<()>())?;
        effects.tasks.abort_all();
        assert!(first.await.is_err());
        let second = effects.start(std::future::pending::<()>())?;
        let mut draining = Box::pin(effects.drain());
        std::future::poll_fn(|context| {
            assert!(draining.as_mut().poll(context).is_pending());
            std::task::Poll::Ready(())
        })
        .await;
        drop(draining);
        assert_eq!(effects.failures.len(), 1);
        assert!(effects.active());
        effects.tasks.abort_all();
        assert!(second.await.is_err());
        let failure = effects.drain().await.err().ok_or("Missing task failures")?;
        let EffectError::Tasks(causes) = &failure else {
            return Err("Independent task failures were discarded".into());
        };
        assert_eq!(causes.0.len(), 2);
        assert!(causes.0.iter().all(tokio::task::JoinError::is_cancelled));
        assert_eq!(format!("{failure:?}"), "EffectError([REDACTED])");
        assert!(!effects.active());
        Ok(())
    }

    #[tokio::test]
    async fn drain_finishes_later_started_work_after_an_earlier_join_failure()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut effects = Effects::new();
        let failed = effects.start(std::future::pending::<()>())?;
        effects.tasks.abort_all();
        assert!(failed.await.is_err());
        let (release, released) = oneshot::channel();
        let completed = effects.start(async move {
            let _ = released.await;
            9
        })?;
        let mut draining = Box::pin(effects.drain());
        std::future::poll_fn(|context| {
            assert!(
                draining.as_mut().poll(context).is_pending(),
                "Earlier failure abandoned later native work"
            );
            std::task::Poll::Ready(())
        })
        .await;
        release.send(()).map_err(|_| "Later effect was abandoned")?;
        assert!(draining.await.is_err());
        assert_eq!(completed.await?, 9);
        assert!(!effects.active());
        Ok(())
    }

    #[tokio::test]
    async fn dropping_a_waiter_or_reconnecting_does_not_release_the_started_sdk_permit()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut effects = Effects::new();
        let (entered, entered_rx) = oneshot::channel();
        let (finish, finish_rx) = oneshot::channel();
        let receiver = effects.start(async move {
            let _ = entered.send(());
            let _ = finish_rx.await;
            7
        })?;
        entered_rx.await?;
        assert!(effects.active());
        drop(receiver);
        assert!(matches!(effects.start(async { 8 }), Err(EffectError::Busy)));
        finish
            .send(())
            .map_err(|_| "Effect ended before its explicit completion")?;
        effects.drain().await?;
        assert!(!effects.active());
        assert_eq!(effects.start(async { 9 })?.await?, 9);
        effects.drain().await?;
        Ok(())
    }
}
