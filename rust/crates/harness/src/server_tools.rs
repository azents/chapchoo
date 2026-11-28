//! Consumer-owned MCP declaration and actual native-owner lifetime.
use crate::{ContinuationRoute, NativeAuthenticationBinding, NativeTextTarget, NativeWorkTarget};
use chapchoo_crypto::BearerSecret;
use chapchoo_domain::local_mcp;
use std::{error::Error, fmt, future::Future, pin::Pin, sync::Arc};

#[derive(Clone, PartialEq, Eq)]
pub struct NativeToolScope {
    pub agent_id: String,
    pub conversation_id: String,
}
impl NativeToolScope {
    pub(crate) fn matches(&self, binding: &NativeAuthenticationBinding) -> bool {
        self.agent_id == binding.agent_id && self.conversation_id == binding.conversation_id
    }
}
pub struct NativeToolsError(Box<dyn Error + Send + Sync>);
impl NativeToolsError {
    pub fn new(source: Box<dyn Error + Send + Sync>) -> Self {
        Self(source)
    }
}
impl fmt::Display for NativeToolsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Native server tool boundary failed")
    }
}
impl Error for NativeToolsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.0.as_ref())
    }
}
pub type NativeToolsFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, NativeToolsError>> + Send + 'a>>;
pub type NativeGoalRestore =
    Arc<dyn Fn(NativeWorkTarget) -> NativeToolsFuture<'static, NativeGoalContext> + Send + Sync>;

pub struct NativeGoalContext {
    pub(crate) projection: Vec<u8>,
    pub(crate) route: ContinuationRoute,
}

impl NativeGoalContext {
    pub fn new(projection: Vec<u8>, route: ContinuationRoute) -> Self {
        Self { projection, route }
    }
}
/// Identity of the normalized local declaration set, never a call grant.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct NativeExternalSupply([u8; 32]);

impl NativeExternalSupply {
    pub fn from_catalogue(canonical: &[u8]) -> Self {
        Self(chapchoo_crypto::payload_fingerprint(
            "native-external-supply",
            &[canonical],
        ))
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Coordinates of a sealed selection, independent of its effective supply.
/// The provider and Control still validate the live route and applied target.
#[derive(Clone)]
pub struct NativeExternalPreparation {
    pub session_id: String,
    pub control_id: String,
    pub connection_id: String,
    pub execution_id: String,
    pub operation_id: String,
}

pub trait NativeToolsProvider: Send + Sync {
    /// Route that delivered this Setup, distinct from the live relay route.
    fn setup_route(&self) -> Option<&ContinuationRoute> {
        None
    }
    /// Identity of the Connector connection owning the setup route.
    fn setup_connection_id(&self) -> Option<&str> {
        None
    }

    /// Full selection and replacement still require the existing setup cut.
    fn has_pending_external(&self) -> bool {
        false
    }
    /// Confirm exact owner reuse without replacing the callable selection.
    fn confirm_external_reuse<'a>(
        &'a self,
        _target: &'a NativeTextTarget,
        _applied: Option<(&'a NativeExternalPreparation, NativeExternalSupply)>,
        _observe_only: bool,
    ) -> NativeToolsFuture<'a, bool> {
        Box::pin(async { Ok(false) })
    }
    fn resolve_external(&self) -> NativeToolsFuture<'_, Option<Arc<dyn NativeToolsProvider>>> {
        Box::pin(async { Ok(None) })
    }
    fn external_supply(&self) -> Option<NativeExternalSupply> {
        None
    }
    fn external_preparation(&self) -> Option<&NativeExternalPreparation> {
        None
    }
    /// Only the actor's actual launch invokes this, after scope/auth binding.
    fn provide<'a>(
        &'a self,
        binding: &'a NativeAuthenticationBinding,
    ) -> NativeToolsFuture<'a, NativeToolProvision>;
}
pub trait NativeToolLifetime: Send + Sync {
    fn quiescent(&self) -> bool {
        false
    }
    fn close(&self);
    /// Resolve optional declarations once, after owning the required listener.
    fn complete_optional(&mut self) -> NativeToolsFuture<'_, NativeOptionalTools> {
        Box::pin(async { Ok(NativeOptionalTools::default()) })
    }
    fn bind_external(&self, _target: NativeTextTarget, _preparation: NativeExternalPreparation) {}
    fn verify(&self) -> NativeToolsFuture<'_, ()>;
    fn finish(&mut self) -> NativeToolsFuture<'_, ()>;
}
#[derive(Default)]
pub struct NativeOptionalTools {
    pub declarations: Vec<NativeMcpDeclaration>,
    pub supply: Option<NativeExternalSupply>,
    pub preparation: Option<NativeExternalPreparation>,
}
pub struct NativeMcpDeclaration {
    name: String,
    url: reqwest::Url,
    key: BearerSecret,
}
impl NativeMcpDeclaration {
    /// Validate the declaration before handing any physical owner to a provision.
    pub fn http(url: &str, key: BearerSecret) -> Result<Self, NativeToolsError> {
        Self::local("chapchoo".into(), "/mcp", url, key)
    }
    /// The managed namespace is distinct from built-in and display names.
    pub fn external_http(id: &str, url: &str, key: BearerSecret) -> Result<Self, NativeToolsError> {
        if !local_mcp::valid_integration_id(id) {
            return Err(invalid_declaration());
        }
        Self::local(
            format!("{}{id}", local_mcp::EXTERNAL_PREFIX),
            &format!("/mcp/{id}"),
            url,
            key,
        )
    }
    fn local(
        name: String,
        path: &str,
        url: &str,
        key: BearerSecret,
    ) -> Result<Self, NativeToolsError> {
        let url = reqwest::Url::parse(url).map_err(|error| NativeToolsError::new(error.into()))?;
        if url.scheme() != "http"
            || url.host_str() != Some("127.0.0.1")
            || url.port().is_none()
            || url.path() != path
            || url.query().is_some()
            || url.fragment().is_some()
            || !url.username().is_empty()
            || url.password().is_some()
        {
            return Err(invalid_declaration());
        }
        Ok(Self { name, url, key })
    }
    pub(crate) fn name(&self) -> &str {
        &self.name
    }
    pub(crate) fn wire(&self) -> agent_client_protocol::v2::schema::McpServer {
        use agent_client_protocol::v2::schema;
        schema::McpServer::Http(
            schema::McpServerHttp::new(self.name(), self.url.as_str()).headers(vec![
                schema::HttpHeader::new("Authorization", format!("Bearer {}", self.key.expose())),
            ]),
        )
    }
    pub(crate) fn http_credentials(&self) -> (&str, &BearerSecret) {
        (self.url.as_str(), &self.key)
    }
}
fn invalid_declaration() -> NativeToolsError {
    NativeToolsError::new(std::io::Error::other("Invalid local MCP declaration").into())
}
pub struct NativeToolProvision {
    declaration: NativeMcpDeclaration,
    external: Vec<NativeMcpDeclaration>,
    external_supply: Option<NativeExternalSupply>,
    external_preparation: Option<NativeExternalPreparation>,
    applied_external: Option<NativeExternalPreparation>,
    lifetime: Box<dyn NativeToolLifetime>,
    restore_goal: Option<NativeGoalRestore>,
}
impl NativeToolProvision {
    pub(crate) fn quiescent(&self) -> bool {
        self.lifetime.quiescent()
    }
    pub fn new(declaration: NativeMcpDeclaration, lifetime: Box<dyn NativeToolLifetime>) -> Self {
        Self {
            declaration,
            external: Vec::new(),
            external_supply: None,
            external_preparation: None,
            applied_external: None,
            lifetime,
            restore_goal: None,
        }
    }
    pub(crate) async fn complete_optional(&mut self) -> Result<(), NativeToolsError> {
        let optional = self.lifetime.complete_optional().await?;
        self.external = optional.declarations;
        self.external_supply = optional.supply;
        self.external_preparation = optional.preparation;
        Ok(())
    }
    pub(crate) fn external_preparation(&self) -> Option<&NativeExternalPreparation> {
        self.external_preparation.as_ref()
    }
    pub(crate) fn external_supply(&self) -> Option<NativeExternalSupply> {
        self.external_supply
    }
    pub(crate) fn bind_external(
        &mut self,
        target: NativeTextTarget,
        preparation: Option<&NativeExternalPreparation>,
    ) {
        self.applied_external = preparation
            .filter(|_| self.external_supply.is_some())
            .cloned();
        if let Some(preparation) = &self.applied_external {
            self.lifetime.bind_external(target, preparation.clone());
        }
    }
    pub(crate) fn applied_external(&self) -> Option<&NativeExternalPreparation> {
        self.applied_external.as_ref()
    }
    pub fn with_goal_restore(mut self, restore: NativeGoalRestore) -> Self {
        self.restore_goal = Some(restore);
        self
    }
    pub(crate) fn goal_restore(&self) -> Option<NativeGoalRestore> {
        self.restore_goal.clone()
    }
    pub(crate) fn declarations(&self) -> Vec<agent_client_protocol::v2::schema::McpServer> {
        self.local_declarations()
            .into_iter()
            .map(NativeMcpDeclaration::wire)
            .collect()
    }
    pub(crate) fn local_declarations(&self) -> Vec<&NativeMcpDeclaration> {
        std::iter::once(&self.declaration)
            .chain(&self.external)
            .collect()
    }
    pub(crate) fn close(&self) {
        self.lifetime.close();
    }
    pub(crate) async fn verify(&self) -> Result<(), NativeToolsError> {
        self.lifetime.verify().await
    }
    pub(crate) async fn finish(&mut self) -> Result<(), NativeToolsError> {
        self.close();
        self.lifetime.finish().await
    }
}
impl Drop for NativeToolProvision {
    fn drop(&mut self) {
        self.close();
    }
}
macro_rules! redacted {($($ty:ty),+)=>{$(
    impl fmt::Debug for $ty {fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result {
        f.write_str(concat!(stringify!($ty),"([REDACTED])"))
    }}
)+};}
redacted!(
    NativeExternalPreparation,
    NativeExternalSupply,
    NativeToolScope,
    NativeToolsError,
    NativeGoalContext,
    NativeMcpDeclaration,
    NativeOptionalTools,
    NativeToolProvision
);

#[cfg(test)]
pub(crate) fn unused_tools() -> std::sync::Arc<dyn NativeToolsProvider> {
    struct Unused;
    impl NativeToolsProvider for Unused {
        fn provide<'a>(
            &'a self,
            _: &'a NativeAuthenticationBinding,
        ) -> NativeToolsFuture<'a, NativeToolProvision> {
            Box::pin(async { Err(NativeToolsError::new(Box::new(crate::Error::NotAdmitted))) })
        }
    }
    std::sync::Arc::new(Unused)
}
