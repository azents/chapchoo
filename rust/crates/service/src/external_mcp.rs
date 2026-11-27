//! Owner-side MCP connection tests. The outbound profile owns network authority.

mod catalogue;
mod http;
mod invocation;
mod runtime;
mod runtime_catalogues;
#[cfg(test)]
mod tests;
mod wire;

use crate::{AcceptanceTime, ServiceError};
pub use catalogue::CompleteMcpCatalogue;
use chapchoo_crypto::SecretDigest;
use chapchoo_persistence::{McpAuthEdit, McpEndpoint, McpTestTarget, Store};
pub use invocation::{
    ExternalMcpCall, ExternalMcpCallOutcome, ExternalMcpInvocation, ExternalMcpRejection,
};
use rmcp::{
    model::{ClientConfig, ErrorCode, PaginatedRequestParams, ProtocolVersion},
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
pub use runtime::{ExternalCatalogue, ExternalCataloguePage, ExternalMcpRuntime};
pub use runtime_catalogues::{
    McpCatalogueCandidate, McpCatalogueCandidatePage, McpCatalogueDiscovery, McpPreparationFailure,
};
use std::{collections::HashSet, error::Error, sync::Arc, time::Duration};
use tokio::sync::OwnedSemaphorePermit;
use tokio::time::Instant;

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
    discovery: McpCatalogueDiscovery,
}

impl AgentMcpTest {
    pub fn new(store: Store) -> Self {
        Self {
            store,
            discovery: McpCatalogueDiscovery::new(),
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
        Ok(
            match self
                .discovery
                .fresh(&target, Instant::now() + DISCOVERY_TIMEOUT)
                .await
            {
                Ok(catalogue) => AgentMcpTestOutcome::Listed {
                    tool_count: catalogue.tool_count(),
                },
                Err(failure) => AgentMcpTestOutcome::Failed(failure),
            },
        )
    }
}

async fn list_tools(
    client: reqwest::Client,
    target: &McpTestTarget,
    slot: OwnedSemaphorePermit,
) -> Result<CompleteMcpCatalogue, McpDiscoveryFailure> {
    let service = connect(client, target.endpoint.url().as_str(), slot, None).await?;
    // Dropping the service signals the SDK owner, which keeps its HTTP permit
    // through bounded session cleanup. Cleanup cannot rewrite a completed list.
    list_pages(&service).await
}

async fn connect(
    client: reqwest::Client,
    uri: &str,
    slot: OwnedSemaphorePermit,
    dispatch: Option<Arc<wire::SingleDispatch>>,
) -> Result<rmcp::service::RunningService<rmcp::RoleClient, ClientConfig>, McpDiscoveryFailure> {
    let mut config = StreamableHttpClientTransportConfig::with_uri(uri)
        .max_concurrent_requests(1)
        .max_sse_event_size(MAX_CATALOGUE_BYTES)
        .reinit_on_expired_session(false);
    config.allow_stateless = true;
    config.retry_config = Arc::new(NeverRetry::default());
    let slot = Arc::new(slot);
    let transport = |negotiation| {
        let lists = wire::ListWitness::default();
        wire::CompleteDiscovery(
            StreamableHttpClientTransport::with_client(
                wire::ProfileHttp::new(
                    client.clone(),
                    slot.clone(),
                    lists.clone(),
                    dispatch.clone(),
                    negotiation,
                ),
                config.clone(),
            ),
            lists,
        )
    };
    let probe = Arc::new(wire::Probe::default());
    match ClientConfig::default()
        .serve_with_lifecycle(
            transport(wire::Negotiation::Modern(probe.clone())),
            ClientLifecycleMode::Discover {
                preferred_versions: vec![ProtocolVersion::V_2026_07_28],
            },
        )
        .await
    {
        Ok(service) => return Ok(service),
        Err(error) if !probe.fallback_allowed() => return Err(startup_failure(error)),
        Err(_) => {}
    }
    let earlier = Arc::new(parking_lot::Mutex::new(wire::Earlier::default()));
    let mut info = ClientConfig::default();
    info.protocol_version = ProtocolVersion::V_2025_11_25;
    info.serve_with_lifecycle(
        transport(wire::Negotiation::Earlier(earlier.clone())),
        ClientLifecycleMode::Initialize,
    )
    .await
    .map_err(|error| {
        if earlier.lock().unsupported_version {
            McpDiscoveryFailure::UnsupportedVersion
        } else {
            startup_failure(error)
        }
    })
}

async fn list_pages(
    peer: &rmcp::service::Peer<rmcp::RoleClient>,
) -> Result<CompleteMcpCatalogue, McpDiscoveryFailure> {
    let info = peer
        .peer_info()
        .ok_or(McpDiscoveryFailure::InvalidResponse)?;
    if info.capabilities.tools.is_none() {
        return Err(McpDiscoveryFailure::MissingToolsCapability);
    }
    let mut cursor = None;
    let mut cursors = HashSet::new();
    let mut catalogue = catalogue::CataloguePages::default();
    for _ in 0..MAX_PAGES {
        let page = peer
            .list_tools(
                cursor.map(|cursor| PaginatedRequestParams::default().with_cursor(Some(cursor))),
            )
            .await
            .map_err(list_failure)?;
        match catalogue.append(page, Instant::now())? {
            None => return catalogue.finish(),
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
                | StreamableHttpError::Sse(_)
                | StreamableHttpError::UnexpectedServerResponse(_) => {
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
