//! Control-owned external declarations. Prepared catalogues alone grant no I/O.

use super::{
    CompleteMcpCatalogue, ExternalMcpCall, ExternalMcpCallOutcome, ExternalMcpInvocation,
    ExternalMcpRejection, McpCatalogueDiscovery, McpDiscoveryFailure, McpPreparationFailure,
};
use crate::ServiceError;
use chapchoo_crypto::SecretDigest;
use chapchoo_persistence::{
    ConnectorSource, McpAppliedScope, McpExecutionScope, McpPreparationScope, McpRetainedSelection,
    Store,
};
use parking_lot::Mutex;
use std::{collections::VecDeque, fmt, sync::Arc, time::Duration};
use tokio::{sync::Semaphore, time::Instant};

const PREPARATION_BUDGET: Duration = Duration::from_secs(2);
const PREPARATION_CONCURRENCY: usize = 8;
const PREPARATION_PAGES: usize = 8;
const SELECTED_CATALOGUE_BYTES: usize = 4 * 1024 * 1024;
const RETAINED_SELECTIONS: usize = 32;

#[derive(Clone)]
pub struct ExternalCatalogue {
    pub integration_id: String,
    pub payload: Result<Arc<[u8]>, McpPreparationFailure>,
}

impl fmt::Debug for ExternalCatalogue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ExternalCatalogue([REDACTED])")
    }
}

#[derive(Debug, Clone)]
pub struct ExternalCataloguePage {
    pub catalogues: Vec<ExternalCatalogue>,
    pub capacity_limited: bool,
}

#[derive(Clone, PartialEq, Eq)]
struct ScopeKey {
    session: String,
    control: String,
    connection: String,
    agent: String,
    conversation: String,
    execution: String,
    operation: String,
}

impl ScopeKey {
    fn new(source: &ConnectorSource<'_>, scope: &McpPreparationScope<'_>) -> Self {
        Self {
            session: source.session_id.to_owned(),
            control: source.control_id.to_owned(),
            connection: source.connection_id.to_owned(),
            agent: scope.execution.agent_id.to_owned(),
            conversation: scope.execution.conversation_id.to_owned(),
            execution: scope.execution.execution_id.to_owned(),
            operation: scope.operation_id.to_owned(),
        }
    }

    fn retention(&self) -> McpRetainedSelection<'_> {
        McpRetainedSelection {
            preparation: McpPreparationScope {
                execution: McpExecutionScope {
                    agent_id: &self.agent,
                    conversation_id: &self.conversation,
                    execution_id: &self.execution,
                },
                operation_id: &self.operation,
            },
            session_id: &self.session,
            control_id: &self.control,
            connection_id: &self.connection,
        }
    }
}

struct Selection {
    key: ScopeKey,
    tools: Vec<(String, SecretDigest, CompleteMcpCatalogue)>,
}

#[derive(Clone)]
pub struct ExternalMcpRuntime {
    store: Store,
    discover: McpCatalogueDiscovery,
    invoke: ExternalMcpInvocation,
    preparation_slots: Arc<Semaphore>,
    selected: Arc<Mutex<VecDeque<Selection>>>,
}

impl fmt::Debug for ExternalMcpRuntime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ExternalMcpRuntime([REDACTED])")
    }
}

impl ExternalMcpRuntime {
    pub fn new(store: Store) -> Self {
        Self {
            discover: McpCatalogueDiscovery::new(),
            invoke: ExternalMcpInvocation::new(store.clone()),
            store,
            preparation_slots: Arc::new(Semaphore::new(PREPARATION_CONCURRENCY)),
            selected: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub async fn prepare(
        &self,
        source: &ConnectorSource<'_>,
        scope: &McpPreparationScope<'_>,
    ) -> Result<ExternalCataloguePage, ServiceError> {
        let Ok(_slot) = self.preparation_slots.try_acquire() else {
            return Ok(ExternalCataloguePage {
                catalogues: Vec::new(),
                capacity_limited: true,
            });
        };
        let key = ScopeKey::new(source, scope);
        let deadline = Instant::now() + PREPARATION_BUDGET;
        let mut after = None;
        let mut entries = Vec::new();
        let mut selected = Vec::new();
        let mut remaining = SELECTED_CATALOGUE_BYTES;
        let mut capacity_limited = false;
        for _ in 0..PREPARATION_PAGES {
            let page = self
                .discover
                .prepare_current_page(&self.store, source, scope, after.as_deref(), deadline)
                .await?;
            for item in page.items {
                let integration_id = item.integration_id;
                let payload = item.catalogue.and_then(|catalogue| {
                    let json = catalogue
                        .local_list_json()
                        .map_err(McpPreparationFailure::Discovery)?;
                    if json.len() > remaining {
                        capacity_limited = true;
                        return Err(McpPreparationFailure::Discovery(McpDiscoveryFailure::Busy));
                    }
                    remaining -= json.len();
                    selected.push((integration_id.clone(), item.settings_key, catalogue));
                    Ok(Arc::from(json))
                });
                entries.push(ExternalCatalogue {
                    integration_id,
                    payload,
                });
            }
            after = page.next_after;
            if after.is_none() {
                break;
            }
        }
        capacity_limited |= after.is_some();
        // Earlier pages can have been edited while a later page was in flight.
        for page in selected.chunks(16) {
            let candidates = page
                .iter()
                .map(|(id, key, _)| (id.as_str(), key))
                .collect::<Vec<_>>();
            let current = self
                .store
                .current_mcp_preparation_candidates(source, scope, &candidates)
                .await?;
            for ((id, _, _), current) in page.iter().zip(current) {
                if !current
                    && let Some(projected) =
                        entries.iter_mut().find(|entry| entry.integration_id == *id)
                {
                    projected.payload = Err(McpPreparationFailure::Superseded);
                }
            }
        }
        selected.retain(|(id, _, _)| {
            entries
                .iter()
                .any(|entry| entry.integration_id == *id && entry.payload.is_ok())
        });
        if selected.is_empty() {
            return Ok(ExternalCataloguePage {
                catalogues: entries,
                capacity_limited,
            });
        }
        self.reclaim_retired().await?;
        let mut retained = self.selected.lock();
        if retained.iter().any(|selection| selection.key == key) {
            // A duplicate preparation cannot replace the selection an existing
            // connector might already be applying.
            return Err(chapchoo_persistence::StoreError::Conflict.into());
        }
        if retained.len() == RETAINED_SELECTIONS {
            // An unrelated active execution never loses its admitted selection
            // to make room for a new optional one.
            for item in &mut entries {
                if item.payload.is_ok() {
                    item.payload = Err(McpPreparationFailure::Discovery(McpDiscoveryFailure::Busy));
                }
            }
            return Ok(ExternalCataloguePage {
                catalogues: entries,
                capacity_limited: true,
            });
        }
        retained.push_back(Selection {
            key,
            tools: selected,
        });
        Ok(ExternalCataloguePage {
            catalogues: entries,
            capacity_limited,
        })
    }

    async fn reclaim_retired(&self) -> Result<(), ServiceError> {
        let snapshot: Vec<_> = {
            let retained = self.selected.lock();
            if retained.len() < RETAINED_SELECTIONS {
                return Ok(());
            }
            retained
                .iter()
                .map(|selection| selection.key.clone())
                .collect()
        };
        let candidates = snapshot.iter().map(ScopeKey::retention).collect::<Vec<_>>();
        let live = self.store.retained_mcp_selections(&candidates).await?;
        let retired = snapshot
            .iter()
            .zip(live)
            .filter_map(|(key, live)| (!live).then_some(key))
            .collect::<Vec<_>>();
        self.selected
            .lock()
            .retain(|selection| !retired.contains(&&selection.key));
        Ok(())
    }

    pub async fn call(
        &self,
        source: &ConnectorSource<'_>,
        scope: &McpAppliedScope<'_>,
        integration_id: &str,
        tool: &str,
        arguments_json: &[u8],
    ) -> Result<ExternalMcpCallOutcome, ServiceError> {
        self.store
            .check_mcp_execution_route(source, &scope.preparation.execution)
            .await?;
        let key = ScopeKey::new(source, &scope.preparation);
        let selection = self
            .selected
            .lock()
            .iter()
            .find(|entry| entry.key == key)
            .and_then(|entry| {
                entry
                    .tools
                    .iter()
                    .find(|(id, _, _)| id == integration_id)
                    .map(|(_, settings, catalogue)| (settings.clone(), catalogue.clone()))
            });
        let Some((setting, catalogue)) = selection else {
            return Ok(ExternalMcpCallOutcome::NotExecuted(
                ExternalMcpRejection::Authority,
            ));
        };
        self.invoke
            .invoke(
                source,
                scope,
                ExternalMcpCall {
                    integration_id,
                    settings_key: &setting,
                    selected: &catalogue,
                    tool,
                    arguments_json,
                },
            )
            .await
    }
}
