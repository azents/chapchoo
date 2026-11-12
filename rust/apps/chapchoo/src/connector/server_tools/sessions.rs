use super::*;
use rmcp::transport::common::server_side_http::ServerSseMessage;
use rmcp::{
    model::{ClientJsonRpcMessage, ServerJsonRpcMessage},
    transport::streamable_http_server::{
        SessionId, SessionManager, session::local::LocalSessionManager,
    },
};
use std::{collections::HashMap, io};
use tokio::sync::OwnedSemaphorePermit;
use tokio_stream::Stream;

pub(super) struct Sessions {
    sdk: LocalSessionManager,
    capacity: Arc<Semaphore>,
    owned: Mutex<HashMap<SessionId, OwnedSemaphorePermit>>,
}
redacted!(Sessions);
impl Sessions {
    pub(super) fn new() -> Self {
        let mut sdk = LocalSessionManager::default();
        sdk.session_config.channel_capacity = 3;
        sdk.session_config.keep_alive = None;
        sdk.session_config.sse_retry = None;
        sdk.session_config.completed_cache_ttl = std::time::Duration::ZERO;
        sdk.session_config.init_timeout = Some(std::time::Duration::from_secs(10));
        Self {
            sdk,
            capacity: Arc::new(Semaphore::new(4)),
            owned: Mutex::new(HashMap::new()),
        }
    }
    pub(super) async fn close_all(&self) -> Result<(), AppError> {
        self.capacity.close();
        let ids = lock(&self.owned).keys().cloned().collect::<Vec<_>>();
        let mut result = Ok(());
        for id in ids {
            result = crate::completion::complete(
                result,
                self.close_session(&id)
                    .await
                    .map_err(|error| -> AppError { error.into() }),
            );
        }
        result
    }
}
impl SessionManager for Sessions {
    type Error = io::Error;
    type Transport = <LocalSessionManager as SessionManager>::Transport;
    async fn create_session(&self) -> Result<(SessionId, Self::Transport), Self::Error> {
        let permit = self
            .capacity
            .clone()
            .try_acquire_owned()
            .map_err(|_| io::Error::other("Native MCP session capacity exhausted"))?;
        // SDK 3.9 performs no await after inserting its handle: once that await
        // completes, retaining the permit below is in the same poll boundary.
        let (id, transport) = self.sdk.create_session().await.map_err(io::Error::other)?;
        lock(&self.owned).insert(id.clone(), permit);
        Ok((id, transport))
    }
    async fn initialize_session(
        &self,
        id: &SessionId,
        message: ClientJsonRpcMessage,
    ) -> Result<ServerJsonRpcMessage, Self::Error> {
        self.sdk
            .initialize_session(id, message)
            .await
            .map_err(io::Error::other)
    }
    async fn has_session(&self, id: &SessionId) -> Result<bool, Self::Error> {
        self.sdk.has_session(id).await.map_err(io::Error::other)
    }
    async fn close_session(&self, id: &SessionId) -> Result<(), Self::Error> {
        self.sdk.close_session(id).await.map_err(io::Error::other)?;
        lock(&self.owned).remove(id);
        Ok(())
    }
    async fn create_stream(
        &self,
        id: &SessionId,
        message: ClientJsonRpcMessage,
    ) -> Result<impl Stream<Item = ServerSseMessage> + Send + Sync + 'static, Self::Error> {
        self.sdk
            .create_stream(id, message)
            .await
            .map_err(io::Error::other)
    }
    async fn accept_message(
        &self,
        id: &SessionId,
        message: ClientJsonRpcMessage,
    ) -> Result<(), Self::Error> {
        self.sdk
            .accept_message(id, message)
            .await
            .map_err(io::Error::other)
    }
    async fn create_standalone_stream(
        &self,
        id: &SessionId,
    ) -> Result<impl Stream<Item = ServerSseMessage> + Send + Sync + 'static, Self::Error> {
        self.sdk
            .create_standalone_stream(id)
            .await
            .map_err(io::Error::other)
    }
    async fn resume(
        &self,
        _: &SessionId,
        _: String,
    ) -> Result<impl Stream<Item = ServerSseMessage> + Send + Sync + 'static, Self::Error> {
        Err::<tokio_stream::Empty<ServerSseMessage>, _>(io::Error::other(
            "MCP event replay is unavailable",
        ))
    }
}
