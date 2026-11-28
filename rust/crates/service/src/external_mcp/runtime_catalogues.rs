use super::{CompleteMcpCatalogue, McpDiscoveryFailure, http::OutboundMcpHttp, list_tools};
use crate::ServiceError;
use chapchoo_crypto::SecretDigest;
use chapchoo_persistence::{
    ConnectorSource, McpPreparationScope, McpTestTarget, SavedMcpConnection, Store,
};
use futures_util::{StreamExt, stream};
use parking_lot::Mutex;
use std::{collections::VecDeque, fmt, sync::Arc};
use tokio::{sync::Semaphore, time::Instant};

const DISCOVERIES_PER_PROCESS: usize = 16;
const DISCOVERIES_PER_BATCH: usize = 8;
const CACHE_ENTRIES: usize = 16;

/// A candidate does not authorize publication or invocation. Recheck the
/// exact current scope and settings before native application.
#[derive(Debug)]
pub struct McpCatalogueCandidate {
    pub integration_id: String,
    pub model_slot: u64,
    pub settings_key: SecretDigest,
    pub catalogue: Result<CompleteMcpCatalogue, McpPreparationFailure>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpPreparationFailure {
    Discovery(McpDiscoveryFailure),
    Superseded,
}

#[derive(Debug)]
pub struct McpCatalogueCandidatePage {
    pub items: Vec<McpCatalogueCandidate>,
    pub next_after: Option<String>,
}

/// Process-owned discovery budget and private, bounded cache. Cache loss
/// causes rediscovery, not a grant or a different configuration.
#[derive(Clone)]
pub struct McpCatalogueDiscovery {
    http: OutboundMcpHttp,
    admission: Arc<Semaphore>,
    cache: Arc<Mutex<CatalogueCache>>,
}

impl fmt::Debug for McpCatalogueDiscovery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("McpCatalogueDiscovery([REDACTED])")
    }
}

impl Default for McpCatalogueDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

impl McpCatalogueDiscovery {
    pub fn new() -> Self {
        Self {
            http: OutboundMcpHttp::new(),
            admission: Arc::new(Semaphore::new(DISCOVERIES_PER_PROCESS)),
            cache: Arc::new(Mutex::new(CatalogueCache::default())),
        }
    }

    /// Use one per-execution deadline for all pages. This observes current
    /// settings after I/O but does not bind them to an applied native lifetime.
    pub async fn prepare_current_page(
        &self,
        store: &Store,
        source: &ConnectorSource<'_>,
        scope: &McpPreparationScope<'_>,
        after: Option<&str>,
        deadline: Instant,
    ) -> Result<McpCatalogueCandidatePage, ServiceError> {
        let page = store
            .capture_mcp_settings_for_preparation(source, scope, after)
            .await?;
        let mut items = self.prepare_batch(page.items, deadline).await;
        let keys = items
            .iter()
            .map(|item| (item.integration_id.as_str(), &item.settings_key))
            .collect::<Vec<_>>();
        let still_current = store
            .current_mcp_preparation_candidates(source, scope, &keys)
            .await?;
        for (item, current) in items.iter_mut().zip(still_current) {
            if !current {
                item.catalogue = Err(McpPreparationFailure::Superseded);
            }
        }
        Ok(McpCatalogueCandidatePage {
            items,
            next_after: page.next_after,
        })
    }

    async fn prepare_batch(
        &self,
        saved: Vec<SavedMcpConnection>,
        deadline: Instant,
    ) -> Vec<McpCatalogueCandidate> {
        let mut result: Vec<_> = stream::iter(saved)
            .map(|saved| async move {
                let cached = self.cache.lock().get(&saved.settings_key, Instant::now());
                let catalogue = match cached {
                    Some(catalogue) => Ok(catalogue),
                    None => {
                        let result = self.fresh(&saved.target, deadline).await;
                        if let Ok(catalogue) = &result {
                            self.cache.lock().remember(
                                saved.settings_key.clone(),
                                catalogue.clone(),
                                Instant::now(),
                            );
                        }
                        result
                    }
                };
                McpCatalogueCandidate {
                    integration_id: saved.integration_id,
                    model_slot: saved.model_slot,
                    settings_key: saved.settings_key,
                    catalogue: catalogue.map_err(McpPreparationFailure::Discovery),
                }
            })
            .buffer_unordered(DISCOVERIES_PER_BATCH)
            .collect()
            .await;
        result.sort_unstable_by(|left, right| left.integration_id.cmp(&right.integration_id));
        result
    }

    pub(super) async fn fresh(
        &self,
        target: &McpTestTarget,
        deadline: Instant,
    ) -> Result<CompleteMcpCatalogue, McpDiscoveryFailure> {
        if Instant::now() >= deadline {
            return Err(McpDiscoveryFailure::TimedOut);
        }
        let slot = self
            .admission
            .clone()
            .try_acquire_owned()
            .map_err(|_| McpDiscoveryFailure::Busy)?;
        let client = self.http.client(target)?;
        tokio::time::timeout_at(deadline, list_tools(client, target, slot))
            .await
            .map_err(|_| McpDiscoveryFailure::TimedOut)?
    }
}

#[derive(Default)]
struct CatalogueCache(VecDeque<(SecretDigest, CompleteMcpCatalogue)>);

impl CatalogueCache {
    fn get(&mut self, key: &SecretDigest, at: Instant) -> Option<CompleteMcpCatalogue> {
        self.0.retain(|(_, entry)| entry.may_reuse_at(at));
        let index = self
            .0
            .iter()
            .position(|(expected, _)| expected.matches_digest(key))?;
        let entry = self.0.remove(index)?;
        let catalogue = entry.1.clone();
        self.0.push_back(entry);
        Some(catalogue)
    }

    fn remember(&mut self, key: SecretDigest, catalogue: CompleteMcpCatalogue, at: Instant) {
        if !catalogue.may_reuse_at(at) {
            return;
        }
        self.0
            .retain(|(current, entry)| !key.matches_digest(current) && entry.may_reuse_at(at));
        if self.0.len() >= CACHE_ENTRIES {
            self.0.pop_front();
        }
        self.0.push_back((key, catalogue));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::external_mcp::catalogue::CataloguePages;
    use rmcp::model::ListToolsResult;
    use std::time::Duration;

    #[test]
    fn cache_requires_exact_settings_and_complete_fresh_result() -> Result<(), std::io::Error> {
        let now = Instant::now();
        let key = SecretDigest::from_bytes([1; 32]);
        let edited = SecretDigest::from_bytes([2; 32]);
        let mut pages = CataloguePages::default();
        assert_eq!(
            pages.append(ListToolsResult::default().with_ttl_ms(30), now),
            Ok(None),
        );
        let catalogue = pages
            .finish()
            .map_err(|error| std::io::Error::other(format!("{error:?}")))?;
        let mut cache = CatalogueCache::default();
        cache.remember(key.clone(), catalogue, now);
        assert!(cache.get(&edited, now).is_none());
        assert!(
            cache
                .get(&key, now)
                .is_some_and(|catalogue| catalogue.tool_count() == 0)
        );
        assert!(cache.get(&key, now + Duration::from_millis(30)).is_none());
        Ok(())
    }
}
