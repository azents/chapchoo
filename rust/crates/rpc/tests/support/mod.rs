#![allow(
    dead_code,
    reason = "Integration binaries use different subsets of this private fixture owner."
)]

use std::{
    collections::HashSet,
    error::Error,
    fmt,
    future::{Future, IntoFuture},
    time::Duration,
};
use tokio::{
    sync::oneshot,
    task::{AbortHandle, Id, JoinSet},
};

type TestError = Box<dyn Error + Send + Sync>;

struct CompletionFailure(Vec<TestError>);

impl fmt::Debug for CompletionFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.0.iter().map(ToString::to_string))
            .finish()
    }
}
impl fmt::Display for CompletionFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "RPC fixture failed with {} retained causes",
            self.0.len()
        )
    }
}
impl Error for CompletionFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.0.first().map(|cause| cause.as_ref() as &dyn Error)
    }
}

#[derive(Debug)]
pub struct Call<T> {
    task: AbortHandle,
    result: oneshot::Receiver<T>,
}
impl<T> IntoFuture for Call<T> {
    type Output = Result<T, oneshot::error::RecvError>;
    type IntoFuture = oneshot::Receiver<T>;

    fn into_future(self) -> Self::IntoFuture {
        self.result
    }
}

#[derive(Debug, Default)]
pub struct Fixture {
    tasks: JoinSet<Result<(), TestError>>,
    stops: Vec<Shutdown>,
    expected_cancellations: HashSet<Id>,
    shutdown_budget: Option<Duration>,
}

#[derive(Debug)]
enum Shutdown {
    IfRunning(oneshot::Sender<()>),
    Required(oneshot::Sender<()>, &'static str),
}

impl Fixture {
    pub fn shutdown_budget(&mut self, budget: Duration) {
        self.shutdown_budget = Some(budget);
    }

    pub fn shutdown(&mut self) -> oneshot::Receiver<()> {
        let (stop, stopped) = oneshot::channel();
        self.stops.push(Shutdown::IfRunning(stop));
        stopped
    }

    pub fn required_shutdown(&mut self, missing: &'static str) -> oneshot::Receiver<()> {
        let (stop, stopped) = oneshot::channel();
        self.stops.push(Shutdown::Required(stop, missing));
        stopped
    }

    pub fn server<F, E>(&mut self, server: F)
    where
        F: Future<Output = Result<(), E>> + Send + 'static,
        E: Into<TestError>,
    {
        self.tasks
            .spawn(async move { server.await.map_err(Into::into) });
    }

    pub fn call<F, T, E>(&mut self, call: F) -> Call<T>
    where
        F: Future<Output = Result<T, E>> + Send + 'static,
        T: Send + 'static,
        E: Into<TestError>,
    {
        let (sender, result) = oneshot::channel();
        let task = self.tasks.spawn(async move {
            // Retain failures in this owner even if the call receiver is dropped
            // before or after completion. Only successful values use the channel.
            let value = call.await.map_err(Into::into)?;
            let _ = sender.send(value);
            Ok(())
        });
        Call { task, result }
    }

    pub fn abort<T>(&mut self, call: &Call<T>) {
        self.expected_cancellations.insert(call.task.id());
        call.task.abort();
    }
}

/// Normal failure must finish synthetic transport ownership before propagation.
pub async fn scope<T>(
    exercise: impl AsyncFnOnce(&mut Fixture) -> Result<T, TestError>,
) -> Result<T, TestError> {
    let mut fixture = Fixture::default();
    let result = exercise(&mut fixture).await;
    let mut causes = Vec::new();
    let value = match result {
        Ok(value) => Some(value),
        Err(error) => {
            causes.push(error);
            None
        }
    };
    for stop in fixture.stops {
        match stop {
            // A completed memory acceptor may already have closed its receiver.
            Shutdown::IfRunning(stop) => {
                let _ = stop.send(());
            }
            Shutdown::Required(stop, missing) => {
                if stop.send(()).is_err() {
                    causes.push(missing.into());
                }
            }
        }
    }
    let timed_out = {
        let completion = async {
            while let Some(exit) = fixture.tasks.join_next_with_id().await {
                match exit {
                    Ok((_, Ok(()))) => {}
                    Ok((_, Err(error))) => causes.push(error),
                    Err(error)
                        if error.is_cancelled()
                            && fixture.expected_cancellations.contains(&error.id()) => {}
                    Err(error) => causes.push(error.into()),
                }
            }
        };
        tokio::pin!(completion);
        if let Some(budget) = fixture.shutdown_budget {
            let expired = tokio::time::timeout(budget, &mut completion).await.is_err();
            // Preserve already collected causes and actual completion ownership.
            // Deadline failure is not evidence that a started operation stopped.
            if expired {
                completion.await;
            }
            expired
        } else {
            completion.await;
            false
        }
    };
    if timed_out {
        causes.push("RPC fixture shutdown deadline exceeded".into());
    }
    if causes.len() == 1 {
        return Err(causes.remove(0));
    }
    if !causes.is_empty() {
        return Err(Box::new(CompletionFailure(causes)));
    }
    value.ok_or_else(|| "RPC fixture lost its successful result".into())
}
