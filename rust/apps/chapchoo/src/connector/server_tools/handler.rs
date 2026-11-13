use super::*;
use rmcp::{
    ErrorData, RoleServer, ServerHandler,
    model::{
        CallToolRequestParams, CallToolResponse, CancelledNotificationParam, Implementation,
        InitializeRequestParams, InitializeResult, ListToolsResult, PaginatedRequestParams,
        ProtocolVersion, RequestId, ServerCapabilities, ServerConfig,
    },
    service::{NotificationContext, RequestContext},
};
use std::{
    borrow::Cow,
    collections::HashMap,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use tokio::sync::oneshot;

#[derive(Default)]
pub(super) struct Cancel {
    sent: AtomicBool,
    changed: Notify,
}
impl Cancel {
    fn notify(&self) {
        self.sent.store(true, Ordering::Release);
        self.changed.notify_waiters();
    }
    pub(super) async fn cancelled(&self) {
        loop {
            let changed = self.changed.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            if self.sent.load(Ordering::Acquire) {
                return;
            }
            changed.await;
        }
    }
}
#[derive(Clone)]
pub(super) struct Handler {
    state: Arc<State>,
    // Request IDs are local to this SDK protocol session, not the capability.
    controls: Arc<Mutex<HashMap<RequestId, Arc<Cancel>>>>,
}
redacted!(Handler);
impl Handler {
    pub(super) fn new(state: Arc<State>) -> Self {
        Self {
            state,
            controls: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
pub(super) fn validate_catalog(catalog: &ListToolsResult) -> Result<(), AppError> {
    // One production declaration, no artificial test tool in the native catalog.
    if catalog.next_cursor.is_some()
        || catalog.tools.len() != 1
        || catalog.tools[0].name != "send_message"
    {
        return Err("The expected server tool catalog is unavailable".into());
    }
    Ok(())
}
fn unavailable() -> ErrorData {
    ErrorData::internal_error("Tool result unavailable; an effect may have occurred", None)
}
impl ServerHandler for Handler {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_protocol_version(ProtocolVersion::V_2025_06_18)
            .with_server_info(Implementation::new("chapchoo", env!("CARGO_PKG_VERSION")))
    }
    fn supported_protocol_versions(&self) -> Cow<'static, [ProtocolVersion]> {
        Cow::Borrowed(&[ProtocolVersion::V_2025_06_18])
    }
    async fn initialize(
        &self,
        request: InitializeRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, ErrorData> {
        if request.protocol_version != ProtocolVersion::V_2025_06_18 {
            return Err(ErrorData::invalid_request("Unsupported MCP profile", None));
        }
        context.peer.set_peer_info(request.clone());
        let response = self.negotiate_initialize(&request)?;
        if response.protocol_version != ProtocolVersion::V_2025_06_18 {
            return Err(ErrorData::invalid_request("Unsupported MCP profile", None));
        }
        self.state.initialized.store(true, Ordering::Release);
        self.state.observed.notify_waiters();
        Ok(response)
    }
    async fn list_tools(
        &self,
        request: Option<PaginatedRequestParams>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        if request.is_some_and(|request| request.cursor.is_some()) {
            return Err(ErrorData::invalid_params(
                "The catalog has no continuation",
                None,
            ));
        }
        let catalog = rpc::catalog(&self.state.route, &self.state.recipient)
            .await
            .map_err(|_| ErrorData::internal_error("Current tool catalog is unavailable", None))?;
        // Definitions remain fixed for this actual native owner's lifetime.
        if catalog != self.state.catalog {
            return Err(ErrorData::internal_error(
                "Tool declarations changed; this owner cannot use them",
                None,
            ));
        }
        self.state.catalogued.store(true, Ordering::Release);
        self.state.observed.notify_waiters();
        Ok(catalog)
    }
    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        if self.state.stop.is_cancelled() {
            return Err(ErrorData::internal_error(
                "MCP owner retired; call was not admitted",
                None,
            ));
        }
        if request.input_responses.is_some() || request.request_state.is_some() {
            return Err(ErrorData::invalid_params(
                "This MCP profile has no continuation",
                None,
            ));
        }
        let arguments = serde_json::to_vec(&request.arguments.unwrap_or_default())
            .map_err(|_| ErrorData::invalid_params("Invalid tool arguments", None))?;
        if arguments.len() > chapchoo_service::MAX_TOOL_PAYLOAD {
            return Err(ErrorData::invalid_params(
                "Encoded tool arguments are too large",
                None,
            ));
        }
        let permit = self.state.work.clone().try_acquire_owned().map_err(|_| {
            ErrorData::internal_error("Local tool capacity exhausted; call was not admitted", None)
        })?;
        // Retain both arguments and the largest possible complete result, in
        // addition to the HTTP body's separately reserved memory.
        let bytes = self
            .state
            .bytes
            .clone()
            .try_acquire_many_owned(
                u32::try_from(arguments.len() + chapchoo_service::MAX_TOOL_PAYLOAD).map_err(
                    |_| ErrorData::invalid_params("Tool representation is too large", None),
                )?,
            )
            .map_err(|_| {
                ErrorData::internal_error(
                    "Local tool memory exhausted; call was not admitted",
                    None,
                )
            })?;
        let mut tasks = self
            .state
            .calls
            .try_lock()
            .map_err(|_| ErrorData::internal_error("Local tool admission is closing", None))?;
        while let Some(result) = tasks.try_join_next() {
            self.state.collect(result);
        }
        let cancel = Arc::new(Cancel::default());
        {
            let mut controls = lock(&self.controls);
            if controls.len() >= 128 || controls.contains_key(&context.id) {
                return Err(ErrorData::invalid_request(
                    "Duplicate or excessive MCP request ID",
                    None,
                ));
            }
            controls.insert(context.id.clone(), cancel.clone());
        }
        let controls = self.controls.clone();
        let id = context.id;
        let state = self.state.clone();
        // Snapshot only already-dispatched inputs. Future inputs must not
        // indefinitely extend a call's receipt dependency set.
        let ordering = state.ordering.snapshot();
        let (sender, receiver) = oneshot::channel();
        tasks.spawn(async move {
            let _permit = permit;
            let _bytes = bytes;
            let invocation = async {
                let waiting = async {
                    ordering
                        .map_err(|_| unavailable())?
                        .ready()
                        .await
                        .map_err(|_| unavailable())
                };
                tokio::select! {
                    result = waiting => result?,
                    _ = cancel.cancelled() => return Err(unavailable()),
                }
                rpc::invoke(&state, request.name.into_owned(), arguments, cancel).await
            };
            let result = tokio::select! {
                result = tokio::time::timeout(Duration::from_secs(50), invocation) =>
                    result.unwrap_or_else(|_| Err(unavailable())),
                _ = state.stop.cancelled() => Err(unavailable()),
            };
            lock(&controls).remove(&id);
            // Dropping the SDK handler waiter is not a cancellation signal.
            let _ = sender.send(result);
        });
        drop(tasks);
        receiver.await.map_err(|_| unavailable())?.map(Into::into)
    }
    async fn on_cancelled(
        &self,
        notification: CancelledNotificationParam,
        _: NotificationContext<RoleServer>,
    ) {
        if let Some(id) = notification.request_id
            && let Some(cancel) = lock(&self.controls).get(&id)
        {
            cancel.notify();
        }
    }
}
