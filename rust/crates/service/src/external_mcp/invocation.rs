use super::{
    CompleteMcpCatalogue, McpDiscoveryFailure, connect, http::OutboundMcpHttp, list_pages,
    wire::SingleDispatch,
};
use crate::{MAX_TOOL_PAYLOAD, ServiceError};
use chapchoo_crypto::SecretDigest;
use chapchoo_persistence::{ConnectorSource, McpAppliedScope, McpTestTarget, Store, StoreError};
use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, JsonObject,
};
use std::{fmt, sync::Arc, time::Duration};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

const INVOCATION_TIMEOUT: Duration = Duration::from_secs(20);
const INVOCATIONS_PER_PROCESS: usize = 16;

/// Exact catalogue and saved settings selected by the server's preparation
/// owner. The component RPC must not construct this directly from its request.
pub struct ExternalMcpCall<'a> {
    pub integration_id: &'a str,
    pub settings_key: &'a SecretDigest,
    pub selected: &'a CompleteMcpCatalogue,
    pub tool: &'a str,
    pub arguments_json: &'a [u8],
}

impl fmt::Debug for ExternalMcpCall<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ExternalMcpCall([REDACTED])")
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ExternalMcpRejection {
    Authority,
    UnknownTool,
    InvalidArguments,
    Capacity,
    Upstream(McpDiscoveryFailure),
}

pub enum ExternalMcpCallOutcome {
    NotExecuted(ExternalMcpRejection),
    Completed {
        result_json: Vec<u8>,
    },
    ToolFailed {
        result_json: Vec<u8>,
    },
    /// The upstream can have processed the request, but the outcome is unknown.
    Uncertain,
    /// The upstream answered, but the answer did not complete the supported call.
    UnsupportedResult,
}

impl fmt::Debug for ExternalMcpCallOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotExecuted(reason) => f.debug_tuple("NotExecuted").field(reason).finish(),
            Self::Completed { .. } => f.write_str("Completed([REDACTED])"),
            Self::ToolFailed { .. } => f.write_str("ToolFailed([REDACTED])"),
            Self::Uncertain => f.write_str("Uncertain"),
            Self::UnsupportedResult => f.write_str("UnsupportedResult"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ExternalMcpInvocation {
    store: Store,
    http: OutboundMcpHttp,
    capacity: Arc<Semaphore>,
}

impl ExternalMcpInvocation {
    pub fn new(store: Store) -> Self {
        Self {
            store,
            http: OutboundMcpHttp::new(),
            capacity: Arc::new(Semaphore::new(INVOCATIONS_PER_PROCESS)),
        }
    }

    pub async fn invoke(
        &self,
        source: &ConnectorSource<'_>,
        scope: &McpAppliedScope<'_>,
        call: ExternalMcpCall<'_>,
    ) -> Result<ExternalMcpCallOutcome, ServiceError> {
        use ExternalMcpCallOutcome::NotExecuted;
        use ExternalMcpRejection::{Authority, Capacity, InvalidArguments, UnknownTool, Upstream};
        let Some(tool) = call.selected.tool(call.tool) else {
            return Ok(NotExecuted(UnknownTool));
        };
        if call.arguments_json.len() > MAX_TOOL_PAYLOAD {
            return Ok(NotExecuted(InvalidArguments));
        }
        let arguments = match serde_json::from_slice::<JsonObject>(call.arguments_json) {
            Ok(arguments) => arguments,
            _ => return Ok(NotExecuted(InvalidArguments)),
        };
        let Ok(slot) = self.capacity.clone().try_acquire_owned() else {
            return Ok(NotExecuted(Capacity));
        };
        let target = match self
            .store
            .admit_mcp_settings_for_call(source, scope, call.integration_id, call.settings_key)
            .await
        {
            Ok(target) => target,
            Err(
                StoreError::NotFound
                | StoreError::Conflict
                | StoreError::Unauthenticated
                | StoreError::InvalidTransition,
            ) => {
                return Ok(NotExecuted(Authority));
            }
            Err(error) => return Err(error.into()),
        };
        let client = match self.http.client(&target) {
            Ok(client) => client,
            Err(reason) => return Ok(NotExecuted(Upstream(reason))),
        };
        let input_schema = tool.input_schema.clone();
        let params = CallToolRequestParams::new(call.tool.to_owned()).with_arguments(arguments);
        // Dropping the RPC waiter does not re-open admission or drop started work.
        // The SDK retains the capacity permit through its worker and response stream.
        let task = tokio::spawn(run(
            client,
            target,
            slot,
            params,
            input_schema,
            INVOCATION_TIMEOUT,
        ));
        Ok(task.await.unwrap_or(ExternalMcpCallOutcome::Uncertain))
    }
}

pub(super) async fn run(
    client: reqwest::Client,
    target: McpTestTarget,
    slot: OwnedSemaphorePermit,
    params: CallToolRequestParams,
    input_schema: Arc<JsonObject>,
    budget: Duration,
) -> ExternalMcpCallOutcome {
    use ExternalMcpCallOutcome::{NotExecuted, Uncertain};
    let dispatch = Arc::new(SingleDispatch::default());
    let operation = async {
        let service = connect(
            client,
            target.endpoint.url().as_str(),
            slot,
            Some(dispatch.clone()),
        )
        .await?;
        async {
            if !service
                .peer_info()
                .is_some_and(|info| info.capabilities.tools.is_some())
            {
                return Err(McpDiscoveryFailure::MissingToolsCapability);
            }
            // The SDK computes protocol-owned Mcp-Param headers from a list on
            // this connection. For annotated schemas, verify the same schema
            // before letting it derive those headers.
            if input_schema.values().any(uses_mcp_header) {
                let current = list_pages(&service).await?;
                if !current
                    .tool(&params.name)
                    .is_some_and(|tool| tool.input_schema == input_schema)
                {
                    return Err(McpDiscoveryFailure::InvalidResponse);
                }
            }
            let response = service
                .call_tool_once(params)
                .await
                .map_err(super::list_failure)?;
            Ok(match response {
                CallToolResponse::Complete(result) => supported_result(result),
                _ => ExternalMcpCallOutcome::UnsupportedResult,
            })
        }
        // The SDK owner retains the permit through session cleanup. Do not
        // turn a definitive result into uncertainty while cleanup is pending.
        .await
    };
    let result = tokio::time::timeout(budget, operation).await;
    let possible = dispatch.close();
    match result {
        Ok(Ok(outcome)) => outcome,
        Ok(Err(_)) | Err(_) if possible => Uncertain,
        Ok(Err(reason)) => NotExecuted(ExternalMcpRejection::Upstream(reason)),
        Err(_) => NotExecuted(ExternalMcpRejection::Upstream(
            McpDiscoveryFailure::TimedOut,
        )),
    }
}

fn uses_mcp_header(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Object(values) => {
            values.contains_key("x-mcp-header") || values.values().any(uses_mcp_header)
        }
        serde_json::Value::Array(values) => values.iter().any(uses_mcp_header),
        _ => false,
    }
}

fn supported_result(mut result: CallToolResult) -> ExternalMcpCallOutcome {
    use ExternalMcpCallOutcome::{Completed, ToolFailed, UnsupportedResult};
    if result
        .content
        .iter()
        .any(|block| !matches!(block, ContentBlock::Text(_)))
    {
        return UnsupportedResult;
    }
    // Upstream protocol metadata is not a native result or credential source.
    result.meta = None;
    for block in &mut result.content {
        if let ContentBlock::Text(text) = block {
            *block = ContentBlock::text(std::mem::take(&mut text.text));
        }
    }
    let Ok(result_json) = serde_json::to_vec(&result) else {
        return UnsupportedResult;
    };
    if result_json.len() > MAX_TOOL_PAYLOAD {
        return UnsupportedResult;
    }
    if result.is_error == Some(true) {
        ToolFailed { result_json }
    } else {
        Completed { result_json }
    }
}
