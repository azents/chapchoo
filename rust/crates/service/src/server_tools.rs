//! Process-owned server tool calls, independent of native readiness and Control.
mod backend;
mod owner;
#[cfg(test)]
mod tests;

use crate::{ConnectorSource, SecretDigest, ServiceError};
pub use backend::ConversationToolBackend;
pub use owner::{ServerTools, ToolInvocation, ToolInvocationControl};
use std::{
    fmt,
    future::Future,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

pub const MAX_TOOL_PAYLOAD: usize = 48 * 1024;
pub type ServerToolFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, ServiceError>> + Send + 'a>>;

/// The component adapter supplies this authenticated source; JSON arguments do
/// not select the target. Rechecked at each protected boundary.
#[derive(Clone)]
pub struct ServerToolScope {
    pub session_id: String,
    pub control_id: String,
    pub connection_id: String,
    pub credential: SecretDigest,
    pub agent_id: String,
    pub conversation_id: String,
}
impl ServerToolScope {
    pub fn source(&self) -> ConnectorSource<'_> {
        ConnectorSource {
            session_id: &self.session_id,
            control_id: &self.control_id,
            connection_id: &self.connection_id,
            credential: &self.credential,
        }
    }
}
pub struct ServerToolRequest {
    pub scope: ServerToolScope,
    pub name: String,
    pub arguments: Vec<u8>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerToolValidation {
    UnknownTool,
    InvalidArguments,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerToolRejection {
    InvalidEnvelope,
    UnknownTool,
    Capacity,
    Closing,
    Cancelled,
    QueueExpired,
    ResponseExpired,
    Authority,
}
pub enum ServerToolOutcome {
    NotExecuted(ServerToolRejection),
    /// Complete encoded MCP result. The common owner preserves its fields.
    Result(Vec<u8>),
    /// An effect or its protected result is not known to this observer.
    Uncertain,
    /// Processing settled, but the effect's result exceeds the representation
    /// contract. This never means that the effect did not occur.
    UncertainRepresentation,
}
pub enum ServerToolObservation {
    Waiting,
    Started,
    Final(ServerToolOutcome),
}

/// A request-local notification, not ownership of an effect or permission to
/// abort its future. Each handler implements its actual cancellation guarantee.
#[derive(Default)]
struct CancellationSignal {
    cancelled: AtomicBool,
    changed: tokio::sync::Notify,
}
#[derive(Clone, Default)]
pub struct ToolCancellation(Arc<CancellationSignal>);
impl ToolCancellation {
    pub fn is_cancelled(&self) -> bool {
        self.0.cancelled.load(Ordering::Acquire)
    }
    pub async fn cancelled(&self) {
        loop {
            let changed = self.0.changed.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            if self.is_cancelled() {
                return;
            }
            changed.await;
        }
    }
    pub(super) fn cancel(&self) {
        self.0.cancelled.store(true, Ordering::Release);
        self.0.changed.notify_waiters();
    }
}

/// Real substitution seam for domain handlers and deterministic slow/error
/// fixtures. Production exposes only the conversation publication backend.
pub trait ServerToolBackend: Send + Sync {
    fn authorize<'a>(&'a self, scope: &'a ServerToolScope) -> ServerToolFuture<'a, ()>;
    fn catalog(&self) -> Result<Vec<u8>, ServiceError>;
    fn validate(&self, name: &str, arguments: &[u8]) -> Result<(), ServerToolValidation>;
    fn execute<'a>(
        &'a self,
        request: &'a ServerToolRequest,
        cancellation: ToolCancellation,
    ) -> ServerToolFuture<'a, Vec<u8>>;
}

macro_rules! redacted {
    ($($ty:ty),+ $(,)?) => {$(
        impl fmt::Debug for $ty {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(concat!(stringify!($ty), "([REDACTED])"))
            }
        }
    )+};
}
pub(super) use redacted;
redacted!(
    ServerToolScope,
    ServerToolRequest,
    ServerToolOutcome,
    ServerToolObservation,
    ToolCancellation
);
