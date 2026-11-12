//! Native-owner-local MCP. Credentials stay in Connector memory; HTTP access
//! never grants server authority independently of the current component route.
mod handler;
mod http;
mod provider;
mod rpc;
mod sessions;
#[cfg(test)]
mod tests;

use crate::AppError;
use chapchoo_crypto::BearerSecret;
use chapchoo_protocol::conversations::CaptureSource;
use std::{
    fmt,
    sync::{Arc, Mutex},
};
use tokio::{
    sync::{Notify, Semaphore},
    task::{JoinError, JoinHandle, JoinSet},
};
use tokio_util::sync::CancellationToken;
use tonic::transport::Channel;

macro_rules! redacted {
    ($($name:ty),+ $(,)?) => {$(
        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(concat!(stringify!($name), "([REDACTED])"))
            }
        }
    )+};
}
pub(super) use provider::Provider;
use redacted;
fn lock<T>(value: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    value
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
struct Route {
    channel: Channel,
    source: CaptureSource,
    key: BearerSecret,
}
/// Operational updates replace server credentials, never a local capability's
/// Agent/conversation or its actual native owner's lifetime.
#[derive(Clone, Default)]
pub(super) struct CurrentRoute(Arc<Mutex<Option<Route>>>);
pub(super) struct RouteLease(CurrentRoute);
redacted!(CurrentRoute, RouteLease);
impl Drop for RouteLease {
    fn drop(&mut self) {
        self.0.withdraw();
    }
}
impl CurrentRoute {
    pub(super) fn update(
        &self,
        channel: Channel,
        source: CaptureSource,
        key: &BearerSecret,
    ) -> Result<(), AppError> {
        *lock(&self.0) = Some(Route {
            channel,
            source,
            key: BearerSecret::parse(key.expose().to_owned())?,
        });
        Ok(())
    }
    pub(super) fn withdraw(&self) {
        *lock(&self.0) = None;
    }
    pub(super) fn lease(&self) -> RouteLease {
        RouteLease(self.clone())
    }
    fn snapshot(&self) -> Result<Route, AppError> {
        let route = lock(&self.0);
        let route = route.as_ref().ok_or("No current server tool route")?;
        Ok(Route {
            channel: route.channel.clone(),
            source: route.source.clone(),
            key: BearerSecret::parse(route.key.expose().to_owned())?,
        })
    }
}
#[derive(Clone)]
pub(super) struct Recipient {
    pub agent_id: String,
    pub conversation_id: String,
}
redacted!(Recipient);
struct State {
    route: CurrentRoute,
    recipient: Recipient,
    key: BearerSecret,
    stop: CancellationToken,
    requests: Arc<Semaphore>,
    connections: Arc<Semaphore>,
    work: Arc<Semaphore>,
    bytes: Arc<Semaphore>,
    calls: tokio::sync::Mutex<JoinSet<()>>,
    failures: Mutex<Vec<JoinError>>,
    initialized: std::sync::atomic::AtomicBool,
    catalogued: std::sync::atomic::AtomicBool,
    observed: Notify,
    catalog: rmcp::model::ListToolsResult,
}
impl State {
    fn collect(&self, result: Result<(), JoinError>) {
        if let Err(error) = result {
            lock(&self.failures).push(error);
        }
    }
}
pub(super) struct Relay {
    url: String,
    state: Arc<State>,
    sessions: Arc<sessions::Sessions>,
    listener: Option<JoinHandle<Result<(), AppError>>>,
    cleanup_errors: Vec<AppError>,
}
redacted!(Relay);
impl Relay {
    pub(super) async fn start(route: CurrentRoute, recipient: Recipient) -> Result<Self, AppError> {
        let catalog = rpc::catalog(&route, &recipient).await?;
        handler::validate_catalog(&catalog)?;
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await?;
        let address = listener.local_addr()?;
        let state = Arc::new(State {
            route,
            recipient,
            key: BearerSecret::generate()?,
            stop: CancellationToken::new(),
            requests: Arc::new(Semaphore::new(128)),
            connections: Arc::new(Semaphore::new(128)),
            work: Arc::new(Semaphore::new(128)),
            bytes: Arc::new(Semaphore::new(8 * 1024 * 1024)),
            calls: tokio::sync::Mutex::new(JoinSet::new()),
            failures: Mutex::new(Vec::new()),
            initialized: std::sync::atomic::AtomicBool::new(false),
            catalogued: std::sync::atomic::AtomicBool::new(false),
            observed: Notify::new(),
            catalog,
        });
        let sessions = Arc::new(sessions::Sessions::new());
        let owner = state.clone();
        let manager = sessions.clone();
        let task = tokio::spawn(async move { http::serve(listener, owner, manager).await });
        Ok(Self {
            url: format!("http://{address}/mcp"),
            state,
            sessions,
            listener: Some(task),
            cleanup_errors: Vec::new(),
        })
    }
    pub(super) fn url(&self) -> &str {
        &self.url
    }
    /// Native HTTP setup boundary only. Never log this value or put it in URLs.
    pub(super) fn bearer(&self) -> &str {
        self.state.key.expose()
    }
    pub(super) fn close(&self) {
        self.state.stop.cancel();
    }
    pub(super) async fn verify(&self) -> Result<(), AppError> {
        use std::sync::atomic::Ordering;
        let observed = async {
            loop {
                let changed = self.state.observed.notified();
                tokio::pin!(changed);
                changed.as_mut().enable();
                if self.state.stop.is_cancelled() {
                    return Err("Native MCP owner retired".into());
                }
                if self.state.initialized.load(Ordering::Acquire)
                    && self.state.catalogued.load(Ordering::Acquire)
                {
                    return Ok(());
                }
                tokio::select! {
                    _ = changed => (),
                    _ = self.state.stop.cancelled() => return Err("Native MCP owner retired".into()),
                }
            }
        };
        tokio::time::timeout(std::time::Duration::from_secs(10), observed)
            .await
            .map_err(|_| -> AppError {
                "Native did not establish the expected MCP endpoint and catalog".into()
            })?
    }
    pub(super) async fn finish(&mut self) -> Result<(), AppError> {
        self.close();
        if let Err(error) = self.sessions.close_all().await {
            self.cleanup_errors.push(error);
        }
        if let Some(listener) = &mut self.listener {
            let result = listener.await;
            self.listener = None;
            match result {
                Ok(Ok(())) => (),
                Ok(Err(error)) => self.cleanup_errors.push(error),
                Err(error) => self.cleanup_errors.push(error.into()),
            }
        }
        let mut tasks = self.state.calls.lock().await;
        while let Some(result) = tasks.join_next().await {
            self.state.collect(result);
        }
        let failures = std::mem::take(&mut *lock(&self.state.failures));
        let mut result = Ok(());
        for error in std::mem::take(&mut self.cleanup_errors) {
            result = crate::completion::complete(result, Err(error));
        }
        for error in failures {
            result = crate::completion::complete(result, Err(error.into()));
        }
        result
    }
}
impl Drop for Relay {
    fn drop(&mut self) {
        // Explicit finish owns joining; drop withdraws access even on unwind.
        self.close();
    }
}
