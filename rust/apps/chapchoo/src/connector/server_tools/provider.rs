use super::*;
use chapchoo_harness::{
    NativeAuthenticationBinding, NativeMcpDeclaration, NativeToolLifetime, NativeToolProvision,
    NativeToolScope, NativeToolsError, NativeToolsFuture, NativeToolsProvider,
};

pub(in crate::connector) struct Provider {
    route: CurrentRoute,
    scope: NativeToolScope,
    ordering: Ordering,
    expected_catalog: Option<rmcp::model::ListToolsResult>,
}
redacted!(Provider);
impl Provider {
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
                )),
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
