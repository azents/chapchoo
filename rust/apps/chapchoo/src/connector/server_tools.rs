//! Native-owner-local MCP. Credentials stay in Connector memory; HTTP access
//! never grants server authority independently of the current component route.
mod external;
mod handler;
mod http;
mod ordering;
mod provider;
mod rpc;
mod sessions;
#[cfg(test)]
mod tests;

use crate::AppError;
use chapchoo_crypto::BearerSecret;
use chapchoo_protocol::conversations::CaptureSource;
use std::{
    collections::BTreeMap,
    fmt,
    sync::{Arc, Mutex, OnceLock},
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
pub(super) use external::PreparedExternal;
pub(super) use ordering::{Ordering, Ticket};
pub(super) use provider::Provider;
use redacted;
fn lock<T>(value: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    value
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
pub(super) struct Route {
    pub(super) channel: Channel,
    pub(super) source: CaptureSource,
    pub(super) key: BearerSecret,
    pub(super) restore_route: chapchoo_harness::ContinuationRoute,
}
/// Operational updates replace server credentials, never a local capability's
/// Agent/conversation or its actual native owner's lifetime.
#[derive(Clone, Default)]
pub(super) struct CurrentRoute(
    Arc<Mutex<Option<(Route, chapchoo_harness::ContinuationRouteLease)>>>,
);
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
        let lease = chapchoo_harness::ContinuationRouteLease::open();
        *lock(&self.0) = Some((
            Route {
                channel,
                source,
                key: BearerSecret::parse(key.expose().to_owned())?,
                restore_route: lease.route(),
            },
            lease,
        ));
        Ok(())
    }
    pub(super) fn withdraw(&self) {
        *lock(&self.0) = None;
    }
    pub(super) fn lease(&self) -> RouteLease {
        RouteLease(self.clone())
    }
    pub(super) fn setup_route(
        &self,
        source: &CaptureSource,
    ) -> Option<chapchoo_harness::ContinuationRoute> {
        lock(&self.0)
            .as_ref()
            .filter(|(route, _)| &route.source == source)
            .map(|(route, _)| route.restore_route.clone())
    }
    fn snapshot(&self) -> Result<Route, AppError> {
        self.try_snapshot()?
            .ok_or("No current server tool route".into())
    }
    pub(super) fn try_snapshot(&self) -> Result<Option<Route>, AppError> {
        let route = lock(&self.0);
        route
            .as_ref()
            .map(|(route, _)| {
                Ok(Route {
                    channel: route.channel.clone(),
                    source: route.source.clone(),
                    key: BearerSecret::parse(route.key.expose().to_owned())?,
                    restore_route: route.restore_route.clone(),
                })
            })
            .transpose()
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
    ordering: Arc<ordering::Owner>,
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
    external: Option<external::Registration>,
    uncertain_effects: std::sync::atomic::AtomicBool,
}
impl State {
    fn new(
        route: CurrentRoute,
        recipient: Recipient,
        ordering: Arc<ordering::Owner>,
        key: BearerSecret,
        stop: CancellationToken,
        catalog: rmcp::model::ListToolsResult,
        external: Option<external::Registration>,
    ) -> Self {
        let (requests, work, bytes) = if external.is_some() {
            (8, 4, 1024 * 1024)
        } else {
            (128, 128, 8 * 1024 * 1024)
        };
        Self {
            route,
            recipient,
            ordering,
            key,
            stop,
            requests: Arc::new(Semaphore::new(requests)),
            connections: Arc::new(Semaphore::new(requests)),
            work: Arc::new(Semaphore::new(work)),
            bytes: Arc::new(Semaphore::new(bytes)),
            calls: tokio::sync::Mutex::new(JoinSet::new()),
            failures: Mutex::new(Vec::new()),
            initialized: std::sync::atomic::AtomicBool::new(false),
            catalogued: std::sync::atomic::AtomicBool::new(false),
            observed: Notify::new(),
            catalog,
            external,
            uncertain_effects: std::sync::atomic::AtomicBool::new(false),
        }
    }
    fn collect(&self, result: Result<(), JoinError>) {
        if let Err(error) = result
            && self.external.is_none()
        {
            lock(&self.failures).push(error);
        }
    }
    fn calls_quiescent(&self) -> bool {
        let Ok(mut calls) = self.calls.try_lock() else {
            return false;
        };
        while let Some(result) = calls.try_join_next() {
            self.collect(result);
        }
        calls.is_empty() && lock(&self.failures).is_empty()
    }
}
pub(super) struct Relay {
    url: String,
    state: Arc<State>,
    builtin: http::Endpoint,
    optional: BTreeMap<String, http::Endpoint>,
    optional_routes: Arc<OnceLock<BTreeMap<String, http::Endpoint>>>,
    address: std::net::SocketAddr,
    applied: Arc<Mutex<Option<external::AppliedSelection>>>,
    listener: Option<JoinHandle<Result<(), AppError>>>,
    cleanup_errors: Vec<AppError>,
}
redacted!(Relay);
impl Relay {
    #[cfg(test)]
    pub(super) async fn start(
        route: CurrentRoute,
        recipient: Recipient,
        ordering: Arc<ordering::Owner>,
    ) -> Result<Self, AppError> {
        Self::start_with_external(route, recipient, ordering, &PreparedExternal::default()).await
    }
    #[cfg(test)]
    pub(super) async fn start_with_external(
        route: CurrentRoute,
        recipient: Recipient,
        ordering: Arc<ordering::Owner>,
        prepared: &PreparedExternal,
    ) -> Result<Self, AppError> {
        let mut relay = Self::start_builtin(route, recipient, ordering).await?;
        relay.install_optional(prepared);
        Ok(relay)
    }
    pub(super) async fn start_builtin(
        route: CurrentRoute,
        recipient: Recipient,
        ordering: Arc<ordering::Owner>,
    ) -> Result<Self, AppError> {
        let catalog = rpc::catalog(&route, &recipient).await?;
        handler::validate_catalog(&catalog)?;
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await?;
        let address = listener.local_addr()?;
        let stop = CancellationToken::new();
        let state = Arc::new(State::new(
            route,
            recipient,
            ordering,
            BearerSecret::generate()?,
            stop.clone(),
            catalog,
            None,
        ));
        let applied = Arc::new(Mutex::new(None));
        let builtin = http::Endpoint::new(address, state.clone());
        let optional_routes = Arc::new(OnceLock::new());
        let task = tokio::spawn({
            let builtin = builtin.clone();
            let routes = optional_routes.clone();
            async move { http::serve(listener, stop, builtin, routes).await }
        });
        Ok(Self {
            url: format!("http://{address}/mcp"),
            state,
            builtin,
            optional: BTreeMap::new(),
            optional_routes,
            address,
            applied,
            listener: Some(task),
            cleanup_errors: Vec::new(),
        })
    }
    fn install_optional(&mut self, prepared: &PreparedExternal) {
        let mut routes = BTreeMap::new();
        for (id, catalog) in &prepared.entries {
            let registration = external::Registration {
                id: id.clone(),
                applied: self.applied.clone(),
            };
            let Ok(key) = BearerSecret::generate() else {
                tracing::warn!("Could not allocate a local external MCP capability");
                continue;
            };
            let endpoint = http::Endpoint::new(
                self.address,
                Arc::new(State::new(
                    self.state.route.clone(),
                    self.state.recipient.clone(),
                    self.state.ordering.clone(),
                    key,
                    self.state.stop.clone(),
                    catalog.clone(),
                    Some(registration),
                )),
            );
            routes.insert(format!("/mcp/{id}"), endpoint.clone());
            self.optional.insert(id.clone(), endpoint);
        }
        let _ = self.optional_routes.set(routes);
    }
    pub(super) fn url(&self) -> &str {
        &self.url
    }
    /// Native HTTP setup boundary only. Never log this value or put it in URLs.
    pub(super) fn bearer(&self) -> &str {
        self.state.key.expose()
    }
    fn optional_credentials(&self) -> impl Iterator<Item = (&str, String, &str)> {
        self.optional.iter().map(|(id, endpoint)| {
            (
                id.as_str(),
                format!("{}/{id}", self.url),
                endpoint.state.key.expose(),
            )
        })
    }
    fn bind_external(
        &self,
        target: chapchoo_harness::NativeTextTarget,
        preparation: chapchoo_harness::NativeExternalPreparation,
    ) {
        *lock(&self.applied) = Some(external::AppliedSelection {
            target,
            preparation,
        });
    }
    pub(super) fn close(&self) {
        self.state.ordering.close();
        self.state.stop.cancel();
    }
    pub(super) fn quiescent(&self) -> bool {
        if self
            .state
            .uncertain_effects
            .load(std::sync::atomic::Ordering::Acquire)
        {
            return false;
        }
        self.state.calls_quiescent()
            && self
                .optional
                .values()
                .all(|endpoint| endpoint.state.calls_quiescent())
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
        if let Err(error) = self.builtin.sessions.close_all().await {
            self.cleanup_errors.push(error);
        }
        for endpoint in self.optional.values() {
            if endpoint.sessions.close_all().await.is_err() {
                tracing::debug!("Optional native MCP sessions could not be closed cleanly");
            }
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
        for endpoint in self.optional.values() {
            let mut tasks = endpoint.state.calls.lock().await;
            while tasks.join_next().await.is_some() {}
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
