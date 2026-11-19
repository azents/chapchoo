use super::*;
use chapchoo_harness::{
    NativeAuthenticationBinding, NativeGoalRestore, NativeMcpDeclaration, NativeToolLifetime,
    NativeToolProvision, NativeToolScope, NativeToolsError, NativeToolsFuture, NativeToolsProvider,
};

pub(in crate::connector) struct Provider {
    route: CurrentRoute,
    scope: NativeToolScope,
    ordering: Ordering,
    expected_catalog: Option<rmcp::model::ListToolsResult>,
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
        }
    }
    pub(in crate::connector) fn with_expected_catalog(
        mut self,
        bytes: &[u8],
    ) -> Result<Self, AppError> {
        self.expected_catalog = Some(serde_json::from_slice(bytes)?);
        Ok(self)
    }
}
struct Lifetime(Relay);
impl NativeToolLifetime for Lifetime {
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
            let mut relay = Relay::start(
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
                    Box::new(Lifetime(relay)),
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
