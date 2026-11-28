use super::*;
use chapchoo_harness::{
    NativeAuthenticationBinding, NativeExternalPreparation, NativeExternalSupply,
    NativeGoalRestore, NativeMcpDeclaration, NativeOptionalTools, NativeTextTarget,
    NativeToolLifetime, NativeToolProvision, NativeToolScope, NativeToolsError, NativeToolsFuture,
    NativeToolsProvider,
};

pub(in crate::connector) struct Provider {
    route: CurrentRoute,
    setup_route: Option<(chapchoo_harness::ContinuationRoute, String)>,
    scope: NativeToolScope,
    ordering: Ordering,
    expected_catalog: Option<rmcp::model::ListToolsResult>,
    external_request: Option<ExternalRequest>,
    external: PreparedExternal,
}
#[derive(Clone)]
struct ExternalRequest {
    source: CaptureSource,
    execution: String,
    operation: String,
}
impl ExternalRequest {
    async fn fetch(
        &self,
        route: &CurrentRoute,
        agent: &str,
        conversation: &str,
    ) -> PreparedExternal {
        let selected = match PreparedExternal::fetch(
            route,
            self.source.clone(),
            agent.to_owned(),
            conversation.to_owned(),
            self.execution.clone(),
            self.operation.clone(),
        )
        .await
        {
            Ok(external) => external,
            Err(_) => {
                tracing::debug!("External MCP preparation unavailable");
                PreparedExternal::transport_unavailable()
            }
        };
        let diagnostics = &selected.diagnostics;
        if !diagnostics.unavailable.is_empty()
            || diagnostics.capacity_limited
            || diagnostics.transport_unavailable
        {
            tracing::debug!(
                unavailable = diagnostics.unavailable.len(),
                capacity_limited = diagnostics.capacity_limited,
                transport_unavailable = diagnostics.transport_unavailable,
                "Optional MCP selection completed with unavailable catalogues"
            );
        }
        selected
    }
}
redacted!(Provider);
impl Provider {
    fn goal_restore(&self, conversation_id: String) -> NativeGoalRestore {
        use chapchoo_protocol::conversations as capture;
        let routes = self.route.clone();
        Arc::new(move |target| {
            let routes = routes.clone();
            let conversation_id = conversation_id.clone();
            Box::pin(async move {
                let route = routes.snapshot().map_err(NativeToolsError::new)?;
                let mut client = capture::connector_capture_client::ConnectorCaptureClient::new(
                    route.channel.clone(),
                )
                .max_decoding_message_size(16 * 1024)
                .max_encoding_message_size(16 * 1024);
                let request = super::super::request(
                    capture::GoalContextRequest {
                        source: Some(route.source.clone()),
                        conversation_id,
                        target: Some(capture::NativeTarget {
                            snapshot_id: target.snapshot.as_str().into(),
                            native_session_id: target.native_session.as_str().into(),
                        }),
                    },
                    &route.key,
                )
                .map_err(NativeToolsError::new)?;
                let response = client
                    .read_goal_context(request)
                    .await
                    .map_err(|error| NativeToolsError::new(error.into()))?
                    .into_inner();
                Ok(chapchoo_harness::NativeGoalContext::new(
                    response.projection,
                    route.restore_route,
                ))
            })
        })
    }

    pub(in crate::connector) fn new(
        route: CurrentRoute,
        scope: NativeToolScope,
        ordering: Ordering,
    ) -> Self {
        Self {
            route,
            scope,
            ordering,
            expected_catalog: None,
            external: PreparedExternal::default(),
            external_request: None,
            setup_route: None,
        }
    }
    pub(in crate::connector) fn with_setup_route(
        mut self,
        route: chapchoo_harness::ContinuationRoute,
        connection_id: String,
    ) -> Self {
        self.setup_route = Some((route, connection_id));
        self
    }
    pub(in crate::connector) fn select_external(
        mut self,
        source: CaptureSource,
        execution: String,
        operation: String,
    ) -> Self {
        self.external_request = Some(ExternalRequest {
            source,
            execution,
            operation,
        });
        self
    }
    pub(in crate::connector) fn with_expected_catalog(
        mut self,
        bytes: &[u8],
    ) -> Result<Self, AppError> {
        self.expected_catalog = Some(serde_json::from_slice(bytes)?);
        Ok(self)
    }
}
enum OptionalSelection {
    Pending(ExternalRequest),
    Selected(PreparedExternal),
}
struct Lifetime(Relay, Option<OptionalSelection>);
impl NativeToolLifetime for Lifetime {
    fn complete_optional(&mut self) -> NativeToolsFuture<'_, NativeOptionalTools> {
        Box::pin(async move {
            let Some(selection) = self.1.take() else {
                return Err(NativeToolsError::new(
                    "Optional MCP selection was already consumed".into(),
                ));
            };
            let relay = &mut self.0;
            let selected = match selection {
                OptionalSelection::Pending(request) => {
                    request
                        .fetch(
                            &relay.state.route,
                            &relay.state.recipient.agent_id,
                            &relay.state.recipient.conversation_id,
                        )
                        .await
                }
                OptionalSelection::Selected(prepared) => prepared,
            };
            relay.install_optional(&selected);
            let declarations = relay
                .optional_credentials()
                .filter_map(|(id, url, bearer)| {
                    let slot = selected.model_slots.get(id).copied().unwrap_or_default();
                    let tools = relay.optional[id]
                        .state
                        .catalog
                        .tools
                        .iter()
                        .map(|tool| tool.name.to_string())
                        .collect();
                    let declaration = BearerSecret::parse(bearer.to_owned())
                        .map_err(|error| NativeToolsError::new(error.into()))
                        .and_then(|key| {
                            NativeMcpDeclaration::external_http(id, slot, &url, key, tools)
                        });
                    match declaration {
                        Ok(declaration) => Some((id.to_owned(), declaration)),
                        Err(_) => {
                            tracing::debug!("Could not materialize optional local MCP declaration");
                            None
                        }
                    }
                })
                .collect::<Vec<_>>();
            let supply = super::external::effective_supply(
                declarations
                    .iter()
                    .map(|(id, _)| (id, &relay.optional[id].state.catalog)),
            )
            .unwrap_or_else(|_| {
                tracing::debug!("Could not encode adopted external MCP supply");
                None
            });
            Ok(NativeOptionalTools {
                declarations: declarations
                    .into_iter()
                    .map(|(_, declaration)| declaration)
                    .collect(),
                supply,
                preparation: selected.selection,
            })
        })
    }
    fn bind_external(
        &self,
        target: chapchoo_harness::NativeTextTarget,
        preparation: NativeExternalPreparation,
    ) {
        self.0.bind_external(target, preparation);
    }
    fn quiescent(&self) -> bool {
        self.0.quiescent()
    }
    fn close(&self) {
        self.0.close();
    }
    fn verify(&self) -> NativeToolsFuture<'_, ()> {
        Box::pin(async { self.0.verify().await.map_err(NativeToolsError::new) })
    }
    fn finish(&mut self) -> NativeToolsFuture<'_, ()> {
        Box::pin(async { self.0.finish().await.map_err(NativeToolsError::new) })
    }
}
impl NativeToolsProvider for Provider {
    fn setup_route(&self) -> Option<&chapchoo_harness::ContinuationRoute> {
        self.setup_route.as_ref().map(|(route, _)| route)
    }
    fn setup_connection_id(&self) -> Option<&str> {
        self.setup_route.as_ref().map(|(_, id)| id.as_str())
    }

    fn has_pending_external(&self) -> bool {
        self.external_request.is_some()
    }
    fn confirm_external_reuse<'a>(
        &'a self,
        target: &'a NativeTextTarget,
        applied: Option<(&'a NativeExternalPreparation, NativeExternalSupply)>,
        observe_only: bool,
    ) -> NativeToolsFuture<'a, bool> {
        Box::pin(async move {
            let Some(request) = &self.external_request else {
                return Ok(false);
            };
            PreparedExternal::confirm_reuse(
                &self.route,
                super::external::selection(
                    request.source.clone(),
                    request.execution.clone(),
                    request.operation.clone(),
                ),
                Recipient {
                    agent_id: self.scope.agent_id.clone(),
                    conversation_id: self.scope.conversation_id.clone(),
                },
                target,
                applied,
                observe_only,
            )
            .await
            .map_err(NativeToolsError::new)
        })
    }
    fn resolve_external(&self) -> NativeToolsFuture<'_, Option<Arc<dyn NativeToolsProvider>>> {
        Box::pin(async move {
            let Some(request) = &self.external_request else {
                return Ok(None);
            };
            let external = request
                .fetch(
                    &self.route,
                    &self.scope.agent_id,
                    &self.scope.conversation_id,
                )
                .await;
            Ok(Some(Arc::new(Self {
                route: self.route.clone(),
                setup_route: self.setup_route.clone(),
                scope: self.scope.clone(),
                ordering: self.ordering.clone(),
                expected_catalog: self.expected_catalog.clone(),
                external_request: None,
                external,
            }) as Arc<dyn NativeToolsProvider>))
        })
    }
    fn external_supply(&self) -> Option<NativeExternalSupply> {
        self.external.supply
    }
    fn external_preparation(&self) -> Option<&NativeExternalPreparation> {
        self.external.selection.as_ref()
    }
    fn provide<'a>(
        &'a self,
        binding: &'a NativeAuthenticationBinding,
    ) -> NativeToolsFuture<'a, NativeToolProvision> {
        Box::pin(async move {
            if self.scope.agent_id != binding.agent_id
                || self.scope.conversation_id != binding.conversation_id
            {
                return Err(NativeToolsError::new(
                    std::io::Error::other("Native tool scope conflicts").into(),
                ));
            }
            let mut relay = Relay::start_builtin(
                self.route.clone(),
                Recipient {
                    agent_id: self.scope.agent_id.clone(),
                    conversation_id: self.scope.conversation_id.clone(),
                },
                self.ordering.open(binding.clone()),
            )
            .await
            .map_err(NativeToolsError::new)?;
            if self
                .expected_catalog
                .as_ref()
                .is_some_and(|expected| expected != &relay.state.catalog)
            {
                relay.finish().await.map_err(NativeToolsError::new)?;
                return Err(NativeToolsError::new(
                    std::io::Error::other("Tool contract changed before preparation").into(),
                ));
            }
            let declaration = BearerSecret::parse(relay.bearer().to_owned())
                .map_err(|error| NativeToolsError::new(error.into()))
                .and_then(|key| NativeMcpDeclaration::http(relay.url(), key));
            match declaration {
                Ok(declaration) => Ok(NativeToolProvision::new(
                    declaration,
                    Box::new(Lifetime(
                        relay,
                        Some(match &self.external_request {
                            Some(request) => OptionalSelection::Pending(request.clone()),
                            None => OptionalSelection::Selected(self.external.clone()),
                        }),
                    )),
                )
                .with_goal_restore(self.goal_restore(binding.conversation_id.clone()))),
                Err(primary) => {
                    let primary = match relay.finish().await {
                        Ok(()) => primary,
                        Err(cleanup) => NativeToolsError::new(Box::new(
                            chapchoo_harness::Error::CleanupFailed {
                                primary: Box::new(chapchoo_harness::Error::ToolsProvision(primary)),
                                secondary: Box::new(chapchoo_harness::Error::ToolsCleanup(
                                    NativeToolsError::new(cleanup),
                                )),
                            },
                        )),
                    };
                    Err(primary)
                }
            }
        })
    }
}
