//! Consumer-owned MCP declaration and actual native-owner lifetime.
use crate::NativeAuthenticationBinding;
use chapchoo_crypto::BearerSecret;
use std::{error::Error, fmt, future::Future, pin::Pin};

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
pub trait NativeToolsProvider: Send + Sync {
    /// Only the actor's actual launch invokes this, after scope/auth binding.
    fn provide<'a>(
        &'a self,
        binding: &'a NativeAuthenticationBinding,
    ) -> NativeToolsFuture<'a, NativeToolProvision>;
}
pub trait NativeToolLifetime: Send + Sync {
    fn close(&self);
    fn verify(&self) -> NativeToolsFuture<'_, ()>;
    fn finish(&mut self) -> NativeToolsFuture<'_, ()>;
}
pub struct NativeMcpDeclaration {
    url: reqwest::Url,
    key: BearerSecret,
}
impl NativeMcpDeclaration {
    /// Validate the declaration before handing any physical owner to a provision.
    pub fn http(url: &str, key: BearerSecret) -> Result<Self, NativeToolsError> {
        let url = reqwest::Url::parse(url).map_err(|error| NativeToolsError::new(error.into()))?;
        if url.scheme() != "http"
            || url.host_str() != Some("127.0.0.1")
            || url.port().is_none()
            || url.path() != "/mcp"
            || url.query().is_some()
            || url.fragment().is_some()
            || !url.username().is_empty()
            || url.password().is_some()
        {
            return Err(NativeToolsError::new(
                std::io::Error::other("Invalid local MCP declaration").into(),
            ));
        }
        Ok(Self { url, key })
    }
    pub(crate) fn wire(&self) -> agent_client_protocol::v2::schema::McpServer {
        use agent_client_protocol::v2::schema;
        schema::McpServer::Http(
            schema::McpServerHttp::new("chapchoo", self.url.as_str()).headers(vec![
                schema::HttpHeader::new("Authorization", format!("Bearer {}", self.key.expose())),
            ]),
        )
    }
}
pub struct NativeToolProvision {
    declaration: NativeMcpDeclaration,
    lifetime: Box<dyn NativeToolLifetime>,
}
impl NativeToolProvision {
    pub fn new(declaration: NativeMcpDeclaration, lifetime: Box<dyn NativeToolLifetime>) -> Self {
        Self {
            declaration,
            lifetime,
        }
    }
    pub(crate) fn declarations(&self) -> Vec<agent_client_protocol::v2::schema::McpServer> {
        vec![self.declaration.wire()]
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
    NativeToolScope,
    NativeToolsError,
    NativeMcpDeclaration,
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
