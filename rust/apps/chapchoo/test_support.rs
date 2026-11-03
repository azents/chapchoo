//! Private process/Control/native fixture completion in dependency order.
#![allow(
    dead_code,
    reason = "Unit and CLI fixtures use different owner operations."
)]
use std::{
    error::Error,
    fmt,
    future::{Future, IntoFuture},
    pin::Pin,
    sync::Arc,
};
use tokio::{
    process::{Child, Command},
    sync::{Mutex, oneshot},
    task::JoinSet,
};
use tokio_util::sync::CancellationToken;

type TestError = Box<dyn Error + Send + Sync>;
type Cleanup = Pin<Box<dyn Future<Output = Result<(), TestError>> + Send>>;

struct Failures(Vec<TestError>);
impl fmt::Debug for Failures {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.0.iter().map(|cause| format!("{cause:?}")))
            .finish()
    }
}
impl fmt::Display for Failures {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Fixture failed with {} retained causes", self.0.len())
    }
}
impl Error for Failures {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.0.first().map(|error| error.as_ref() as &dyn Error)
    }
}
fn finish(mut causes: Vec<TestError>) -> Result<(), TestError> {
    match causes.len() {
        0 => Ok(()),
        1 => Err(causes.remove(0)),
        _ => Err(Box::new(Failures(causes))),
    }
}
pub fn completed<T>(
    primary: Result<T, TestError>,
    cleanup: Result<(), TestError>,
) -> Result<T, TestError> {
    match (primary, cleanup) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), Ok(())) | (Ok(_), Err(error)) => Err(error),
        (Err(primary), Err(cleanup)) => Err(Box::new(Failures(vec![primary, cleanup]))),
    }
}
async fn collect(tasks: &mut JoinSet<Result<(), TestError>>, causes: &mut Vec<TestError>) {
    while let Some(result) = tasks.join_next().await {
        match result {
            Ok(Ok(())) => {}
            Ok(Err(error)) => causes.push(error),
            Err(error) => causes.push(error.into()),
        }
    }
}
pub async fn requests(tasks: &mut JoinSet<Result<(), TestError>>) -> Result<(), TestError> {
    let mut causes = Vec::new();
    collect(tasks, &mut causes).await;
    finish(causes)
}

pub struct Call<T>(oneshot::Receiver<T>);
impl<T> IntoFuture for Call<T> {
    type Output = Result<T, oneshot::error::RecvError>;
    type IntoFuture = oneshot::Receiver<T>;
    fn into_future(self) -> Self::IntoFuture {
        self.0
    }
}
#[derive(Default)]
pub struct Fixture {
    workers: JoinSet<Result<(), TestError>>,
    cancellation: Vec<CancellationToken>,
    children: Vec<Arc<Mutex<Child>>>,
    cleanup: Vec<Cleanup>,
    servers: JoinSet<Result<(), TestError>>,
    shutdown: Vec<oneshot::Sender<()>>,
}
impl Fixture {
    pub fn shutdown(&mut self) -> oneshot::Receiver<()> {
        let (stop, stopped) = oneshot::channel();
        self.shutdown.push(stop);
        stopped
    }
    pub fn server<F, E>(&mut self, server: F)
    where
        F: Future<Output = Result<(), E>> + Send + 'static,
        E: Into<TestError>,
    {
        self.servers
            .spawn(async move { server.await.map_err(Into::into) });
    }
    pub fn worker<F, T, E>(&mut self, stop: CancellationToken, worker: F) -> Call<T>
    where
        F: Future<Output = Result<T, E>> + Send + 'static,
        T: Send + 'static,
        E: Into<TestError>,
    {
        self.cancellation.push(stop);
        let (sender, receiver) = oneshot::channel();
        self.workers.spawn(async move {
            let value = worker.await.map_err(Into::into)?;
            let _ = sender.send(value);
            Ok(())
        });
        Call(receiver)
    }
    pub fn cleanup(
        &mut self,
        completion: impl Future<Output = Result<(), TestError>> + Send + 'static,
    ) {
        self.cleanup.push(Box::pin(completion));
    }
    pub fn child(&mut self, command: &mut Command) -> Result<Arc<Mutex<Child>>, TestError> {
        let child = Arc::new(Mutex::new(command.spawn()?));
        self.children.push(child.clone());
        Ok(child)
    }
}

pub async fn scope(
    exercise: impl AsyncFnOnce(&mut Fixture) -> Result<(), TestError>,
) -> Result<(), TestError> {
    let mut fixture = Fixture::default();
    let mut causes = exercise(&mut fixture)
        .await
        .err()
        .into_iter()
        .collect::<Vec<_>>();
    // Providers must retain access to Control and the native byte server while
    // their accepted work finishes. Never stop dependencies before these joins.
    for stop in fixture.cancellation {
        stop.cancel();
    }
    collect(&mut fixture.workers, &mut causes).await;
    for child in fixture.children {
        let mut child = child.lock().await;
        match child.try_wait() {
            Ok(Some(_)) => continue,
            Ok(None) => {}
            Err(error) => causes.push(error.into()),
        }
        if let Err(error) = child.start_kill() {
            causes.push(error.into());
        }
        if let Err(error) = child.wait().await {
            causes.push(error.into());
        }
    }
    for cleanup in fixture.cleanup {
        if let Err(error) = cleanup.await {
            causes.push(error);
        }
    }
    for stop in fixture.shutdown {
        let _ = stop.send(());
    }
    collect(&mut fixture.servers, &mut causes).await;
    finish(causes)
}
