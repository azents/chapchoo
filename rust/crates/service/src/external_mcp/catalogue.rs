use super::{MAX_CATALOGUE_BYTES, MAX_TOOLS, McpDiscoveryFailure};
use rmcp::model::{ListToolsResult, ResultType, Tool};
use std::{collections::BTreeMap, fmt, sync::Arc, time::Duration};
use tokio::time::Instant;

// Cap process-local reuse even when a peer advertises very long freshness.
const MAX_REUSE: Duration = Duration::from_secs(15 * 60);

/// Native 2025-profile lists omit the upstream result envelope's completion
/// and freshness fields. Tool definitions are preserved byte-for-byte in the
/// typed representation used to compare and apply the selection.
pub fn native_mcp_list(mut list: ListToolsResult) -> ListToolsResult {
    list.result_type = None;
    list.ttl_ms = None;
    list
}

/// Complete normalized tool supply. Expiry is not part of its representation.
#[derive(Clone)]
pub struct CompleteMcpCatalogue {
    tools: Arc<[Tool]>,
    reusable_until: Instant,
}

impl fmt::Debug for CompleteMcpCatalogue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CompleteMcpCatalogue")
            .field("tool_count", &self.tool_count())
            .finish_non_exhaustive()
    }
}

impl CompleteMcpCatalogue {
    pub(super) fn tool(&self, name: &str) -> Option<&Tool> {
        let index = self
            .tools
            .binary_search_by(|tool| tool.name.as_ref().cmp(name))
            .ok()?;
        self.tools.get(index)
    }

    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }

    pub fn may_reuse_at(&self, at: Instant) -> bool {
        at < self.reusable_until
    }

    /// Encodes effective supply without upstream page or freshness metadata.
    pub fn local_list_json(&self) -> Result<Vec<u8>, McpDiscoveryFailure> {
        serde_json::to_vec(&ListToolsResult::with_all_items(self.tools.to_vec()))
            .map_err(|_| McpDiscoveryFailure::InvalidResponse)
    }
}

#[derive(Default)]
pub(super) struct CataloguePages {
    tools: BTreeMap<Arc<str>, Tool>,
    bytes: usize,
    earliest_expiry: Option<Instant>,
    finished: bool,
}

impl CataloguePages {
    pub(super) fn append(
        &mut self,
        page: ListToolsResult,
        observed_at: Instant,
    ) -> Result<Option<String>, McpDiscoveryFailure> {
        if self.finished
            || page
                .result_type
                .as_ref()
                .is_some_and(|kind| kind != &ResultType::COMPLETE)
        {
            return Err(McpDiscoveryFailure::InvalidResponse);
        }
        self.bytes = self.bytes.saturating_add(
            serde_json::to_vec(&page)
                .map_err(|_| McpDiscoveryFailure::InvalidResponse)?
                .len(),
        );
        if self.bytes > MAX_CATALOGUE_BYTES
            || self.tools.len().saturating_add(page.tools.len()) > MAX_TOOLS
        {
            return Err(McpDiscoveryFailure::IncompleteCatalogue);
        }
        for tool in page.tools {
            if tool.name.is_empty()
                || self
                    .tools
                    .insert(Arc::from(tool.name.as_ref()), tool)
                    .is_some()
            {
                return Err(McpDiscoveryFailure::InvalidResponse);
            }
        }
        let ttl = Duration::from_millis(page.ttl_ms.unwrap_or_default()).min(MAX_REUSE);
        let expiry = observed_at + ttl;
        self.earliest_expiry = Some(
            self.earliest_expiry
                .map_or(expiry, |previous| previous.min(expiry)),
        );
        self.finished = page.next_cursor.is_none();
        Ok(page.next_cursor)
    }

    pub(super) fn finish(self) -> Result<CompleteMcpCatalogue, McpDiscoveryFailure> {
        let reusable_until = self
            .earliest_expiry
            .filter(|_| self.finished)
            .ok_or(McpDiscoveryFailure::IncompleteCatalogue)?;
        Ok(CompleteMcpCatalogue {
            tools: self.tools.into_values().collect(),
            reusable_until,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(
        names: &[&str],
        next: Option<&str>,
        ttl: u64,
    ) -> Result<ListToolsResult, serde_json::Error> {
        let tools = names
            .iter()
            .map(|name| serde_json::json!({"name":name,"inputSchema":{"type":"object"}}))
            .collect::<Vec<_>>();
        serde_json::from_value(serde_json::json!({"tools":tools,"nextCursor":next,"ttlMs":ttl}))
    }

    #[test]
    fn normalizes_complete_pages_and_uses_earliest_expiry() -> Result<(), Box<dyn std::error::Error>>
    {
        let start = Instant::now();
        let mut pages = CataloguePages::default();
        assert_eq!(
            pages.append(page(&["second"], Some("next"), 50)?, start),
            Ok(Some("next".into()))
        );
        assert_eq!(
            pages.append(
                page(&["first"], None, 500)?,
                start + Duration::from_millis(10)
            ),
            Ok(None)
        );
        let result = pages
            .finish()
            .map_err(|error| std::io::Error::other(format!("{error:?}")))?;
        assert!(result.may_reuse_at(start + Duration::from_millis(49)));
        assert!(!result.may_reuse_at(start + Duration::from_millis(50)));
        let json = result
            .local_list_json()
            .map_err(|error| std::io::Error::other(format!("{error:?}")))?;
        let value: serde_json::Value = serde_json::from_slice(&json)?;
        assert_eq!(value["tools"][0]["name"], "first");
        assert_eq!(value["tools"][1]["name"], "second");
        assert!(value.get("nextCursor").is_none());
        Ok(())
    }

    #[test]
    fn zero_ttl_empty_catalogue_remains_valid_without_granting_reuse()
    -> Result<(), serde_json::Error> {
        let start = Instant::now();
        let mut pages = CataloguePages::default();
        assert_eq!(pages.append(page(&[], None, 0)?, start), Ok(None));
        assert!(
            pages.finish().is_ok_and(
                |catalogue| catalogue.tool_count() == 0 && !catalogue.may_reuse_at(start)
            )
        );
        Ok(())
    }
}
