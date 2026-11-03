//! Private safe completion causes shared by transport-owned workers and processes.
use crate::Error;
use std::{error::Error as StdError, fmt};
use tokio::task::JoinSet;

pub struct CompletionFailures(Vec<Error>);

impl fmt::Debug for CompletionFailures {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DockerCompletionFailures([REDACTED])")
    }
}
impl fmt::Display for CompletionFailures {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Owned Docker transport completion failed")
    }
}
impl StdError for CompletionFailures {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.0.first().map(|cause| cause as &dyn StdError)
    }
}

pub(crate) fn complete<T>(
    primary: Result<T, Error>,
    cleanup: Result<(), Error>,
) -> Result<T, Error> {
    match (primary, cleanup) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), Ok(())) | (Ok(_), Err(error)) => Err(error),
        (Err(primary), Err(cleanup)) => Err(Error::Completion(CompletionFailures(vec![
            primary, cleanup,
        ]))),
    }
}

#[derive(Default)]
pub(crate) struct OwnedTasks {
    pub tasks: JoinSet<Result<(), Error>>,
    failures: Vec<Error>,
}

impl OwnedTasks {
    fn retain(&mut self, result: Result<Result<(), Error>, tokio::task::JoinError>) {
        match result {
            Ok(Ok(())) => {}
            Ok(Err(error)) => self.failures.push(error),
            Err(error) => self.failures.push(Error::TransportTask(error)),
        }
    }

    pub fn reap(&mut self) {
        while let Some(result) = self.tasks.try_join_next() {
            self.retain(result);
        }
    }

    /// Keep this state in the physical owner, not in a cancellable drain waiter.
    pub async fn join(&mut self) {
        while let Some(result) = self.tasks.join_next().await {
            self.retain(result);
        }
    }

    pub fn finish(&mut self) -> Result<(), Error> {
        let mut failures = std::mem::take(&mut self.failures);
        match failures.len() {
            0 => Ok(()),
            1 => Err(failures.remove(0)),
            _ => Err(Error::Completion(CompletionFailures(failures))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn cancelled_completion_wait_keeps_collected_causes_and_pending_work()
    -> Result<(), Box<dyn StdError>> {
        let mut owned = OwnedTasks::default();
        let (done, complete) = tokio::sync::oneshot::channel();
        owned.tasks.spawn(async move {
            let _ = done.send(());
            Err(Error::TransportIo(std::io::Error::other("private-first")))
        });
        complete.await?;
        let (release, released) = tokio::sync::oneshot::channel();
        owned.tasks.spawn(async move {
            released.await.map_err(|_| Error::Transport)?;
            Err(Error::TransportIo(std::io::Error::other("private-second")))
        });
        let mut draining = Box::pin(owned.join());
        std::future::poll_fn(|context| {
            assert!(draining.as_mut().poll(context).is_pending());
            std::task::Poll::Ready(())
        })
        .await;
        drop(draining);
        assert_eq!(owned.failures.len(), 1);
        assert_eq!(owned.tasks.len(), 1);
        release
            .send(())
            .map_err(|_| "Pending owned worker was abandoned")?;
        owned.join().await;
        let failure = owned.finish().err().ok_or("Missing completion causes")?;
        let Error::Completion(causes) = &failure else {
            return Err("Expected two retained causes".into());
        };
        assert_eq!(causes.0.len(), 2);
        assert!(!failure.to_string().contains("private"));
        assert!(!format!("{failure:?}").contains("private"));
        assert!(StdError::source(&causes.0[0]).is_some());
        Ok(())
    }
}
