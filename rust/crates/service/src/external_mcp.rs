//! Owner-side MCP connection tests. The outbound profile owns network authority.

mod http;
#[cfg(test)]
mod tests;
mod wire;

use crate::{AcceptanceTime, ServiceError};
use chapchoo_crypto::SecretDigest;
use chapchoo_persistence::{McpAuthEdit, McpEndpoint, McpTestTarget, Store};
use rmcp::{
    model::{ErrorCode, PaginatedRequestParams, ProtocolVersion, ResultType},
    service::{ClientInitializeError, ClientLifecycleMode, ClientServiceExt},
    transport::{
        DynamicTransportError,
        common::client_side_sse::NeverRetry,
        streamable_http_client::{
            HttpStatusError, StreamableHttpClientTransport, StreamableHttpClientTransportConfig,
            StreamableHttpError,
        },
    },
};
use std::{collections::HashSet, error::Error, sync::Arc, time::Duration};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

const MAX_PAGES: usize = 32;
const MAX_TOOLS: usize = 512;
const MAX_CATALOGUE_BYTES: usize = 2 * 1024 * 1024;
const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpDiscoveryFailure {
    DestinationDenied,
    Busy,
    Unavailable,
    Authentication,
    UnsupportedVersion,
    MissingDiscovery,
    MissingToolsCapability,
    InvalidResponse,
    IncompleteCatalogue,
    TimedOut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentMcpTestOutcome {
    Listed { tool_count: usize },
    Failed(McpDiscoveryFailure),
}

#[derive(Debug)]
pub struct McpTestEdit<'a> {
    pub integration_id: Option<&'a str>,
    pub endpoint: McpEndpoint,
    pub auth: McpAuthEdit,
}

#[derive(Clone, Debug)]
pub struct AgentMcpTest {
    store: Store,
    http: http::OutboundMcpHttp,
    admission: Arc<Semaphore>,
}

impl AgentMcpTest {
    pub fn new(store: Store) -> Self {
        Self {
            store,
            http: http::OutboundMcpHttp::new(),
            admission: Arc::new(Semaphore::new(16)),
        }
    }

    pub async fn test(
        &self,
        actor: &SecretDigest,
        agent_id: &str,
        edit: McpTestEdit<'_>,
        now: AcceptanceTime,
    ) -> Result<AgentMcpTestOutcome, ServiceError> {
        let target = self
            .store
            .prepare_agent_mcp_test(
                actor,
                agent_id,
                edit.integration_id,
                edit.endpoint,
                edit.auth,
                now.now()?,
            )
            .await?;
        let Ok(slot) = self.admission.clone().try_acquire_owned() else {
            return Ok(AgentMcpTestOutcome::Failed(McpDiscoveryFailure::Busy));
        };
        Ok(match self.discover(&target, slot).await {
            Ok(tool_count) => AgentMcpTestOutcome::Listed { tool_count },
            Err(failure) => AgentMcpTestOutcome::Failed(failure),
        })
    }

    async fn discover(
        &self,
        target: &McpTestTarget,
        slot: OwnedSemaphorePermit,
    ) -> Result<usize, McpDiscoveryFailure> {
        let client = self.http.client(target)?;
        tokio::time::timeout(DISCOVERY_TIMEOUT, list_tools(client, target, slot))
            .await
            .map_err(|_| McpDiscoveryFailure::TimedOut)?
    }
}

async fn list_tools(
    client: reqwest::Client,
    target: &McpTestTarget,
    slot: OwnedSemaphorePermit,
) -> Result<usize, McpDiscoveryFailure> {
    let mut config = StreamableHttpClientTransportConfig::with_uri(target.endpoint.url().as_str())
        .max_concurrent_requests(1)
        .max_sse_event_size(MAX_CATALOGUE_BYTES)
        .reinit_on_expired_session(false);
    config.allow_stateless = true;
    config.retry_config = Arc::new(NeverRetry::default());
    let transport = wire::CompleteDiscovery(StreamableHttpClientTransport::with_client(
        wire::DiscoveryHttp::new(client, slot),
        config,
    ));
    let service = ()
        .serve_with_lifecycle(
            transport,
            ClientLifecycleMode::Discover {
                preferred_versions: vec![ProtocolVersion::V_2026_07_28],
            },
        )
        .await
        .map_err(startup_failure)?;
    let result = list_pages(&service).await;
    // The drop guard also cancels the SDK owner if the enclosing budget expires.
    let _ = service.cancel().await;
    result
}

async fn list_pages(
    peer: &rmcp::service::Peer<rmcp::RoleClient>,
) -> Result<usize, McpDiscoveryFailure> {
    let info = peer
        .peer_info()
        .ok_or(McpDiscoveryFailure::InvalidResponse)?;
    if info.capabilities.tools.is_none() {
        return Err(McpDiscoveryFailure::MissingToolsCapability);
    }
    let mut cursor = None;
    let mut cursors = HashSet::new();
    let mut names = HashSet::new();
    let mut bytes = 0usize;
    for _ in 0..MAX_PAGES {
        let page = peer
            .list_tools(
                cursor.map(|cursor| PaginatedRequestParams::default().with_cursor(Some(cursor))),
            )
            .await
            .map_err(list_failure)?;
        if page
            .result_type
            .as_ref()
            .is_some_and(|kind| kind != &ResultType::COMPLETE)
        {
            return Err(McpDiscoveryFailure::InvalidResponse);
        }
        // Bound accumulated material rather than only the number of requests.
        let page_bytes = serde_json::to_vec(&page)
            .map_err(|_| McpDiscoveryFailure::InvalidResponse)?
            .len();
        bytes = bytes.saturating_add(page_bytes);
        if bytes > MAX_CATALOGUE_BYTES || names.len().saturating_add(page.tools.len()) > MAX_TOOLS {
            return Err(McpDiscoveryFailure::IncompleteCatalogue);
        }
        for tool in page.tools {
            if tool.name.is_empty() || !names.insert(tool.name) {
                return Err(McpDiscoveryFailure::InvalidResponse);
            }
        }
        match page.next_cursor {
            None => return Ok(names.len()),
            Some(next) if !next.is_empty() && cursors.insert(next.clone()) => cursor = Some(next),
            Some(_) => return Err(McpDiscoveryFailure::IncompleteCatalogue),
        }
    }
    Err(McpDiscoveryFailure::IncompleteCatalogue)
}

fn startup_failure(error: ClientInitializeError) -> McpDiscoveryFailure {
    match error {
        ClientInitializeError::NoCompatibleProtocolVersion { .. } => {
            McpDiscoveryFailure::UnsupportedVersion
        }
        ClientInitializeError::TransportError { error, .. } => transport_failure(&error),
        ClientInitializeError::ConnectionClosed(_) => McpDiscoveryFailure::Unavailable,
        ClientInitializeError::JsonRpcError(data) if data.code == ErrorCode::METHOD_NOT_FOUND => {
            McpDiscoveryFailure::MissingDiscovery
        }
        _ => McpDiscoveryFailure::InvalidResponse,
    }
}

fn list_failure(error: rmcp::ServiceError) -> McpDiscoveryFailure {
    match error {
        rmcp::ServiceError::TransportSend(error) => transport_failure(&error),
        rmcp::ServiceError::TransportClosed => McpDiscoveryFailure::Unavailable,
        rmcp::ServiceError::Timeout { .. } => McpDiscoveryFailure::TimedOut,
        rmcp::ServiceError::McpError(data) if data.code == ErrorCode::METHOD_NOT_FOUND => {
            McpDiscoveryFailure::MissingToolsCapability
        }
        _ => McpDiscoveryFailure::InvalidResponse,
    }
}

fn transport_failure(error: &DynamicTransportError) -> McpDiscoveryFailure {
    let mut cause = error.source();
    while let Some(current) = cause {
        if current
            .downcast_ref::<HttpStatusError>()
            .is_some_and(|error| matches!(error.status, 401 | 403))
        {
            return McpDiscoveryFailure::Authentication;
        }
        if let Some(http_error) = current.downcast_ref::<StreamableHttpError<reqwest::Error>>() {
            match http_error {
                StreamableHttpError::HttpStatus(status) if matches!(status.status, 401 | 403) => {
                    return McpDiscoveryFailure::Authentication;
                }
                StreamableHttpError::ResponseBodyTooLarge { .. } => {
                    return McpDiscoveryFailure::IncompleteCatalogue;
                }
                StreamableHttpError::UnexpectedContentType(_)
                | StreamableHttpError::Deserialize(_)
                | StreamableHttpError::Sse(_) => {
                    return McpDiscoveryFailure::InvalidResponse;
                }
                StreamableHttpError::Client(request) => {
                    // The SDK's Client variant does not expose its wrapped error
                    // through std::error::Error::source.
                    cause = Some(request);
                    continue;
                }
                _ => {}
            }
        }
        if current.is::<http::NonPublicDestination>() {
            return McpDiscoveryFailure::DestinationDenied;
        }
        if let Some(request) = current.downcast_ref::<reqwest::Error>() {
            if request.is_timeout() {
                return McpDiscoveryFailure::TimedOut;
            }
            if request.is_decode() {
                return McpDiscoveryFailure::InvalidResponse;
            }
        }
        cause = current.source();
    }
    McpDiscoveryFailure::Unavailable
}
