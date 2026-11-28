//! Receive complete Control selections and relay one request on its exact route.
use super::*;
use chapchoo_harness::{NativeExternalPreparation, NativeExternalSupply, NativeTextTarget};
use chapchoo_protocol::{conversations::NativeTarget, server_tools as wire};
use rmcp::{
    ErrorData,
    model::{CallToolResult, ContentBlock, ListToolsResult},
};
use std::{collections::BTreeMap, time::Duration};

const SELECTION_BYTES: usize = 4 * 1024 * 1024;
const CATALOGUE_BYTES: usize = 2 * 1024 * 1024;
const MAX_REGISTRATIONS: usize = 128;
const PREPARATION_TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Clone, Default)]
pub(in crate::connector) struct PreparedExternal {
    pub(super) entries: BTreeMap<String, ListToolsResult>,
    pub(super) model_slots: BTreeMap<String, u64>,
    pub(super) selection: Option<NativeExternalPreparation>,
    pub(super) supply: Option<NativeExternalSupply>,
    pub(super) diagnostics: SelectionDiagnostics,
}

/// Bounded, typed observations only. No upstream URL, credentials, or body.
#[derive(Clone, Default)]
pub(super) struct SelectionDiagnostics {
    pub unavailable: BTreeMap<String, wire::ExternalCatalogFailure>,
    pub capacity_limited: bool,
    pub transport_unavailable: bool,
}

#[derive(Clone)]
pub(super) struct AppliedSelection {
    pub(super) preparation: NativeExternalPreparation,
    pub(super) target: NativeTextTarget,
}

#[derive(Clone)]
pub(super) struct Registration {
    pub(super) id: String,
    pub(super) applied: Arc<Mutex<Option<AppliedSelection>>>,
}
redacted!(PreparedExternal, Registration);

impl PreparedExternal {
    pub(super) async fn confirm_reuse(
        current: &CurrentRoute,
        selection: NativeExternalPreparation,
        recipient: Recipient,
        target: &NativeTextTarget,
        applied: Option<(&NativeExternalPreparation, NativeExternalSupply)>,
        observe_only: bool,
    ) -> Result<bool, AppError> {
        let route = current.snapshot()?;
        if !matches_route(&selection, &route.source) {
            return Ok(false);
        }
        let (applied_operation_id, applied_supply) = match applied {
            Some((applied, supply))
                if matches_route(applied, &route.source)
                    && applied.execution_id == selection.execution_id =>
            {
                (applied.operation_id.clone(), supply.as_bytes().to_vec())
            }
            Some(_) => return Ok(false),
            None => (String::new(), Vec::new()),
        };
        let mut request = rpc::request(
            wire::ExternalReuse {
                preparation: Some(preparation_request(&selection, &route, &recipient)),
                target: Some(NativeTarget {
                    snapshot_id: target.snapshot.as_str().into(),
                    native_session_id: target.native_session.as_str().into(),
                }),
                observe_only,
                applied_operation_id,
                applied_supply,
            },
            &route,
        )?;
        request.set_timeout(PREPARATION_TIMEOUT);
        let result = match tokio::time::timeout(
            PREPARATION_TIMEOUT,
            rpc::client(&route).confirm_external_reuse(request),
        )
        .await?
        {
            Ok(result) => result.into_inner().confirmed,
            Err(status) if status.code() == tonic::Code::Unimplemented => false,
            Err(status) => return Err(status.into()),
        };
        if !matches_route(&selection, &current.snapshot()?.source) {
            return Err("External MCP preparation route changed".into());
        }
        Ok(result)
    }

    #[cfg(test)]
    pub(super) fn fixture(
        source: CaptureSource,
        entries: BTreeMap<String, ListToolsResult>,
    ) -> Result<Self, AppError> {
        Ok(Self {
            supply: effective_supply(entries.iter())?,
            entries,
            model_slots: BTreeMap::new(),
            selection: Some(selection(source, "execution".into(), "setup".into())),
            diagnostics: SelectionDiagnostics::default(),
        })
    }

    pub(super) fn transport_unavailable() -> Self {
        Self {
            diagnostics: SelectionDiagnostics {
                transport_unavailable: true,
                ..SelectionDiagnostics::default()
            },
            ..Self::default()
        }
    }

    pub(in crate::connector) async fn fetch(
        current: &CurrentRoute,
        source: CaptureSource,
        agent: String,
        conversation: String,
        execution: String,
        operation: String,
    ) -> Result<Self, AppError> {
        let route = current.snapshot()?;
        if route.source != source {
            return Err("External MCP preparation route changed".into());
        }
        let selection = selection(source, execution, operation);
        let recipient = Recipient {
            agent_id: agent,
            conversation_id: conversation,
        };
        let mut request =
            rpc::request(preparation_request(&selection, &route, &recipient), &route)?;
        request.set_timeout(PREPARATION_TIMEOUT);
        let (entries, model_slots, diagnostics) =
            tokio::time::timeout(PREPARATION_TIMEOUT, async {
                let mut stream = rpc::client(&route)
                    .prepare_external(request)
                    .await?
                    .into_inner();
                let mut assembly = Assembly::default();
                while let Some(frame) = stream.message().await? {
                    assembly.push(frame)?;
                }
                assembly.finish()
            })
            .await??;
        if !matches_route(&selection, &current.snapshot()?.source) {
            return Err("External MCP preparation route changed".into());
        }
        Ok(Self {
            supply: effective_supply(entries.iter())?,
            entries,
            model_slots,
            selection: Some(selection),
            diagnostics,
        })
    }
}

pub(super) fn effective_supply<'a>(
    entries: impl Iterator<Item = (&'a String, &'a ListToolsResult)>,
) -> Result<Option<NativeExternalSupply>, AppError> {
    let entries = entries.collect::<BTreeMap<_, _>>();
    if entries.is_empty() {
        return Ok(None);
    }
    let mut value = serde_json::to_value(entries)?;
    value.sort_all_objects();
    Ok(Some(NativeExternalSupply::from_catalogue(
        &serde_json::to_vec(&value)?,
    )))
}

pub(super) fn selection(
    source: CaptureSource,
    execution_id: String,
    operation_id: String,
) -> NativeExternalPreparation {
    NativeExternalPreparation {
        session_id: source.session_id,
        control_id: source.control_id,
        connection_id: source.connection_id,
        execution_id,
        operation_id,
    }
}
fn matches_route(selected: &NativeExternalPreparation, current: &CaptureSource) -> bool {
    selected.session_id == current.session_id
        && selected.control_id == current.control_id
        && selected.connection_id == current.connection_id
}
fn preparation_request(
    selected: &NativeExternalPreparation,
    route: &Route,
    recipient: &Recipient,
) -> wire::ExternalPreparation {
    wire::ExternalPreparation {
        scope: Some(rpc::scope(route, recipient)),
        execution_id: selected.execution_id.clone(),
        operation_id: selected.operation_id.clone(),
    }
}

impl Registration {
    fn selection_on(&self, source: &CaptureSource) -> Option<AppliedSelection> {
        lock(&self.applied)
            .as_ref()
            .filter(|applied| matches_route(&applied.preparation, source))
            .cloned()
    }

    pub(super) async fn invoke(
        &self,
        state: &State,
        name: String,
        arguments_json: Vec<u8>,
    ) -> Result<CallToolResult, ErrorData> {
        if !state.catalog.tools.iter().any(|tool| tool.name == name) {
            return Err(rpc::rejection(wire::Rejection::UnknownTool.into()));
        }
        let route = state
            .route
            .snapshot()
            .map_err(|_| rpc::rejection(wire::Rejection::Authority.into()))?;
        let applied = self
            .selection_on(&route.source)
            .ok_or_else(|| rpc::rejection(wire::Rejection::Authority.into()))?;
        let mut request = rpc::request(
            wire::ExternalInvocation {
                preparation: Some(preparation_request(
                    &applied.preparation,
                    &route,
                    &state.recipient,
                )),
                target: Some(NativeTarget {
                    snapshot_id: applied.target.snapshot.as_str().into(),
                    native_session_id: applied.target.native_session.as_str().into(),
                }),
                integration_id: self.id.clone(),
                name,
                arguments_json,
            },
            &route,
        )
        .map_err(|_| rpc::rejection(wire::Rejection::InvalidEnvelope.into()))?;
        request.set_timeout(Duration::from_secs(25));
        let result = rpc::client(&route)
            .invoke_external(request)
            .await
            .map_err(rpc::opening_error)?
            .into_inner();
        translate_result(result)
    }
}

fn translate_result(result: wire::Final) -> Result<CallToolResult, ErrorData> {
    use wire::r#final::Outcome;
    match result.outcome {
        Some(Outcome::ResultJson(bytes)) if bytes.len() <= chapchoo_service::MAX_TOOL_PAYLOAD => {
            let mut result: CallToolResult =
                serde_json::from_slice(&bytes).map_err(|_| rpc::representation())?;
            // The harness-facing 2025 profile has no result discriminator
            // and permits only an object in structuredContent. Give its
            // consumer lossless JSON text when no native text is available.
            result.result_type = None;
            if let Some(value) = &result.structured_content {
                if result.content.is_empty() || !value.is_object() {
                    result.content.push(ContentBlock::text(value.to_string()));
                }
                if !value.is_object() {
                    result.structured_content = None;
                }
            }
            Ok(result)
        }
        Some(Outcome::NotExecuted(reason)) => Err(rpc::rejection(reason)),
        Some(Outcome::UncertainRepresentation(()) | Outcome::ResultJson(_)) => {
            Err(rpc::representation())
        }
        _ => Err(rpc::unavailable()),
    }
}

pub(super) fn integration_identity(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 48
        && id
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'_' | b'-'))
}

#[derive(Default)]
struct Assembly {
    entries: BTreeMap<String, ListToolsResult>,
    model_slots: BTreeMap<String, u64>,
    unavailable: BTreeMap<String, wire::ExternalCatalogFailure>,
    partial: Option<(String, u64, Vec<u8>)>,
    bytes: usize,
    closed: bool,
    capacity_limited: bool,
}
type CatalogueProjection = (
    BTreeMap<String, ListToolsResult>,
    BTreeMap<String, u64>,
    SelectionDiagnostics,
);

impl Assembly {
    fn push(&mut self, frame: wire::ExternalCatalogFrame) -> Result<(), AppError> {
        use wire::external_catalog_frame::Payload;
        if self.closed {
            return Err("External MCP stream continued after its end".into());
        }
        let id = frame.integration_id;
        if let Some(Payload::End(end)) = frame.payload {
            if !id.is_empty() || self.partial.is_some() {
                return Err("Incomplete external MCP selection".into());
            }
            self.closed = true;
            self.capacity_limited = end.capacity_limited;
            return Ok(());
        }
        if !integration_identity(&id)
            || self.entries.contains_key(&id)
            || self.unavailable.contains_key(&id)
            || self.entries.len() + self.unavailable.len() >= MAX_REGISTRATIONS
        {
            return Err("Invalid external MCP integration in selection".into());
        }
        match frame.payload {
            Some(Payload::Catalogue(chunk)) if !chunk.json.is_empty() => {
                let (current, slot, bytes) = self
                    .partial
                    .get_or_insert_with(|| (id.clone(), chunk.model_slot, Vec::new()));
                self.bytes = self.bytes.saturating_add(chunk.json.len());
                if *current != id
                    || *slot != chunk.model_slot
                    || self.bytes > SELECTION_BYTES
                    || bytes.len().saturating_add(chunk.json.len()) > CATALOGUE_BYTES
                {
                    return Err("External MCP catalogue framing or capacity exceeded".into());
                }
                bytes.extend(chunk.json);
                if chunk.final_chunk {
                    let list: ListToolsResult = serde_json::from_slice(bytes)?;
                    if list.next_cursor.is_some()
                        || list.tools.len() > 512
                        || list.tools.iter().any(|tool| tool.name.is_empty())
                        || list
                            .tools
                            .windows(2)
                            .any(|pair| pair[0].name >= pair[1].name)
                    {
                        return Err("External MCP catalogue is incomplete".into());
                    }
                    if *slot != 0 {
                        if chapchoo_domain::local_mcp::short_external_name(*slot).is_none()
                            || self.model_slots.values().any(|other| other == slot)
                        {
                            return Err("Invalid external MCP model slot".into());
                        }
                        self.model_slots.insert(id.clone(), *slot);
                    }
                    self.entries
                        .insert(id, chapchoo_service::native_mcp_list(list));
                    self.partial = None;
                }
            }
            Some(Payload::Unavailable(reason)) if self.partial.is_none() => {
                let reason = wire::ExternalCatalogFailure::try_from(reason)?;
                if reason == wire::ExternalCatalogFailure::Unspecified {
                    return Err("Missing external MCP failure reason".into());
                }
                self.unavailable.insert(id, reason);
            }
            _ => return Err("Invalid external MCP selection frame".into()),
        }
        Ok(())
    }

    fn finish(self) -> Result<CatalogueProjection, AppError> {
        if !self.closed {
            return Err("External MCP selection stream ended without completion".into());
        }
        Ok((
            self.entries,
            self.model_slots,
            SelectionDiagnostics {
                unavailable: self.unavailable,
                capacity_limited: self.capacity_limited,
                transport_unavailable: false,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wire::external_catalog_frame::Payload;

    #[test]
    fn unchanged_supply_can_bind_a_new_preparation_on_the_same_native_target()
    -> Result<(), AppError> {
        use chapchoo_domain::{NativeConversationId, SnapshotId};

        let id = "external".to_owned();
        let first: ListToolsResult = serde_json::from_str(
            r#"{"tools":[{"name":"read","inputSchema":{"type":"object","properties":{"z":{},"a":{}}}}]}"#,
        )?;
        let equivalent: ListToolsResult = serde_json::from_str(
            r#"{"tools":[{"inputSchema":{"properties":{"a":{},"z":{}},"type":"object"},"name":"read"}]}"#,
        )?;
        let changed: ListToolsResult = serde_json::from_str(
            r#"{"tools":[{"name":"write","inputSchema":{"type":"object"}}]}"#,
        )?;
        let fingerprint = |catalogue| effective_supply(std::iter::once((&id, catalogue)));
        assert_eq!(fingerprint(&first)?, fingerprint(&equivalent)?);
        assert_ne!(fingerprint(&first)?, fingerprint(&changed)?);
        assert_ne!(fingerprint(&first)?, effective_supply(std::iter::empty())?);

        let route = CaptureSource {
            session_id: "session".into(),
            control_id: "control".into(),
            connection_id: "first".into(),
        };
        let successor = CaptureSource {
            connection_id: "second".into(),
            ..route.clone()
        };
        let target = NativeTextTarget {
            snapshot: SnapshotId::parse("snapshot".into())?,
            native_session: NativeConversationId::parse("native".into())?,
        };
        let applied = Arc::new(Mutex::new(None));
        let registration = Registration {
            id: id.clone(),
            applied: applied.clone(),
        };
        assert!(registration.selection_on(&route).is_none());
        *lock(&applied) = Some(AppliedSelection {
            preparation: selection(route.clone(), "execution".into(), "before".into()),
            target: target.clone(),
        });
        assert!(registration.selection_on(&route).is_some());
        assert!(registration.selection_on(&successor).is_none());
        *lock(&applied) = Some(AppliedSelection {
            preparation: selection(successor.clone(), "execution".into(), "after".into()),
            target: target.clone(),
        });
        let current = registration
            .selection_on(&successor)
            .ok_or("Missing selected preparation")?;
        assert_eq!(current.preparation.operation_id, "after");
        assert!(current.target == target);
        assert!(registration.selection_on(&route).is_none());
        Ok(())
    }

    #[test]
    fn translates_structured_only_results_and_distinguishes_failures() -> Result<(), AppError> {
        use wire::r#final::Outcome;
        for structured in [
            serde_json::json!({"answer":42}),
            serde_json::json!([1, 2, 3]),
        ] {
            let wire = wire::Final {
                outcome: Some(Outcome::ResultJson(serde_json::to_vec(
                    &serde_json::json!({
                        "content": [], "structuredContent": structured, "isError": true
                    }),
                )?)),
            };
            let result = translate_result(wire)?;
            assert_eq!(result.is_error, Some(true));
            assert_eq!(
                result.structured_content.as_ref(),
                structured.is_object().then_some(&structured)
            );
            let [ContentBlock::Text(text)] = &result.content[..] else {
                return Err("Expected lossless JSON text for the native consumer".into());
            };
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(&text.text)?,
                structured
            );
        }
        for (outcome, message) in [
            (
                Outcome::NotExecuted(wire::Rejection::Authority.into()),
                "invocation was not executed",
            ),
            (
                Outcome::UncertainRepresentation(()),
                "cannot be represented",
            ),
            (Outcome::Uncertain(()), "an effect may have occurred"),
        ] {
            let Err(error) = translate_result(wire::Final {
                outcome: Some(outcome),
            }) else {
                return Err("Failed invocation was represented as a successful result".into());
            };
            assert!(error.message.contains(message));
        }
        Ok(())
    }

    fn chunk(id: &str, json: &[u8], final_chunk: bool) -> wire::ExternalCatalogFrame {
        wire::ExternalCatalogFrame {
            integration_id: id.into(),
            payload: Some(Payload::Catalogue(wire::ExternalCatalogChunk {
                json: json.to_vec(),
                final_chunk,
                model_slot: 0,
            })),
        }
    }

    fn unavailable(id: &str) -> wire::ExternalCatalogFrame {
        wire::ExternalCatalogFrame {
            integration_id: id.into(),
            payload: Some(Payload::Unavailable(
                wire::ExternalCatalogFailure::Timeout.into(),
            )),
        }
    }

    fn end() -> wire::ExternalCatalogFrame {
        wire::ExternalCatalogFrame {
            integration_id: String::new(),
            payload: Some(Payload::End(wire::ExternalCatalogEnd {
                capacity_limited: false,
            })),
        }
    }

    #[test]
    fn complete_large_and_empty_lists_survive_independent_unavailability() -> Result<(), AppError> {
        let description = "a".repeat(170_000);
        let json = serde_json::to_vec(&serde_json::json!({"tools":[{
            "name": "lookup", "description": description, "inputSchema": {"type":"object"}
        }]}))?;
        let mut assembly = Assembly::default();
        let parts = json.chunks(32 * 1024);
        let count = parts.len();
        for (index, bytes) in parts.enumerate() {
            assembly.push(chunk("first", bytes, index + 1 == count))?;
        }
        assembly.push(unavailable("second"))?;
        assembly.push(chunk("third", br#"{"tools":[]}"#, true))?;
        assembly.push(end())?;
        let (entries, slots, diagnostics) = assembly.finish()?;
        assert!(slots.is_empty());
        assert_eq!(entries.len(), 2);
        assert_eq!(
            entries["first"].tools[0].description.as_deref(),
            Some(description.as_str())
        );
        assert!(entries["third"].tools.is_empty());
        assert!(!entries.contains_key("second"));
        assert_eq!(
            diagnostics.unavailable.get("second"),
            Some(&wire::ExternalCatalogFailure::Timeout)
        );
        assert!(!diagnostics.transport_unavailable);
        Ok(())
    }

    #[test]
    fn refuses_unterminated_interleaved_or_post_terminal_frames() -> Result<(), AppError> {
        let mut assembly = Assembly::default();
        assembly.push(chunk("first", br#"{"tools":[]}"#, true))?;
        assert!(assembly.finish().is_err());
        for invalid in [end(), chunk("second", b"}", true), unavailable("second")] {
            let mut assembly = Assembly::default();
            assembly.push(chunk("first", br#"{"tools":[]"#, false))?;
            assert!(assembly.push(invalid).is_err());
        }
        let mut assembly = Assembly::default();
        assembly.push(end())?;
        assert!(assembly.push(unavailable("first")).is_err());
        Ok(())
    }

    #[test]
    fn refuses_malformed_paged_duplicate_and_over_budget_catalogues() -> Result<(), AppError> {
        for json in [
            "invalid json",
            r#"{"tools":[],"nextCursor":"other"}"#,
            r#"{"tools":[{"name":"x","inputSchema":{}},{"name":"x","inputSchema":{}}]}"#,
        ] {
            assert!(
                Assembly::default()
                    .push(chunk("first", json.as_bytes(), true))
                    .is_err()
            );
        }
        let mut assembly = Assembly::default();
        assembly.push(unavailable("first"))?;
        assert!(
            assembly
                .push(chunk("first", br#"{"tools":[]}"#, true))
                .is_err()
        );
        let mut assembly = Assembly::default();
        assert!(
            assembly
                .push(chunk("first", &vec![b' '; CATALOGUE_BYTES + 1], false))
                .is_err()
        );
        let mut json = br#"{"tools":[]}"#.to_vec();
        json.resize(CATALOGUE_BYTES, b' ');
        let mut assembly = Assembly::default();
        assembly.push(chunk("first", &json, true))?;
        assembly.push(chunk("second", &json, true))?;
        assert!(
            assembly
                .push(chunk("third", br#"{"tools":[]}"#, true))
                .is_err()
        );
        Ok(())
    }
}
