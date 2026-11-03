//! SDK task ownership is independent of a call waiter and stream incarnation.
use std::{future::Future, sync::Arc};
use tokio::{
    sync::{Semaphore, oneshot},
    task::JoinSet,
};

#[derive(Debug, thiserror::Error)]
pub enum EffectError {
    #[error("A native lifecycle effect is already in flight")]
    Busy,
    #[error("A native lifecycle effect task failed")]
    Task(#[source] tokio::task::JoinError),
}
/// Retain this owner across stream reconnects. Never replace it merely because a
/// response deadline/cancel/connection has ended. The process supervises drain.
#[derive(Debug)]
pub struct Effects {
    permit: Arc<Semaphore>,
    tasks: JoinSet<()>,
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
            result.map_err(EffectError::Task)?;
        }
        Ok(())
    }
    pub async fn drain(&mut self) -> Result<(), EffectError> {
        let mut failure = None;
        while let Some(result) = self.tasks.join_next().await {
            if let Err(error) = result {
                failure.get_or_insert(error);
            }
        }
        failure.map_or(Ok(()), |error| Err(EffectError::Task(error)))
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn drain_finishes_retained_work_even_after_an_earlier_task_failure()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut effects = Effects::new();
        let failed = effects.start(std::future::pending::<()>())?;
        effects.tasks.abort_all();
        // The closed waiter proves the aborted task released physical capacity.
        assert!(failed.await.is_err());
        let (release, released) = oneshot::channel();
        let completed = effects.start(async move {
            let _ = released.await;
            9
        })?;
        let mut drain = Box::pin(effects.drain());
        std::future::poll_fn(|context| {
            assert!(
                drain.as_mut().poll(context).is_pending(),
                "An earlier task error must not abandon the later owned task"
            );
            std::task::Poll::Ready(())
        })
        .await;
        release.send(()).map_err(|_| "Effect was cancelled")?;
        assert!(matches!(
            drain.await,
            Err(EffectError::Task(error)) if error.is_cancelled()
        ));
        assert_eq!(completed.await?, 9);
        assert!(!effects.active());
        assert!(effects.tasks.is_empty());
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
