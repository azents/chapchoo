use chapchoo_crypto::ExternalSecret;
use chapchoo_persistence::{DeviceAuthenticationTokens, DeviceAuthorizationDraft};
use std::{fmt, future::Future, pin::Pin};

/// Only these acquisition endpoints are covered by the scoped direct-HTTP decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceProviderFailure {
    Rejected,
    Unavailable,
    Uncertain,
    InvalidResponse,
}
impl fmt::Display for DeviceProviderFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Rejected => "Authentication provider rejected the request",
            Self::Unavailable => "Authentication provider is unavailable",
            Self::Uncertain => "Authentication provider outcome is uncertain",
            Self::InvalidResponse => "Authentication provider response is invalid",
        })
    }
}
impl std::error::Error for DeviceProviderFailure {}

pub struct DeviceApproval {
    pub authorization_code: ExternalSecret,
    pub code_verifier: ExternalSecret,
}
impl fmt::Debug for DeviceApproval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DeviceApproval([REDACTED])")
    }
}
#[derive(Debug)]
pub enum DevicePollOutcome {
    Pending,
    Approved(DeviceApproval),
}

pub type DeviceProviderFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, DeviceProviderFailure>> + Send + 'a>>;

/// A bounded single invocation. Implementations do not poll in a loop or replay
/// an uncertain authorization-code exchange. The service owns dispatch lifetime.
pub trait DeviceAuthenticationProvider: Send + Sync {
    fn issue(&self, now: i64) -> DeviceProviderFuture<'_, DeviceAuthorizationDraft>;
    fn poll<'a>(
        &'a self,
        device_auth_id: &'a ExternalSecret,
        user_code: &'a str,
    ) -> DeviceProviderFuture<'a, DevicePollOutcome>;
    fn exchange<'a>(
        &'a self,
        approval: &'a DeviceApproval,
        now: i64,
    ) -> DeviceProviderFuture<'a, DeviceAuthenticationTokens>;
}
/// One SDK-owned refresh grant. No retry after uncertain provider consumption.
pub trait ChatGptRefreshProvider: Send + Sync {
    fn refresh<'a>(
        &'a self,
        bundle: &'a ExternalSecret,
        account_id: &'a str,
        now: i64,
    ) -> RefreshProviderFuture<'a>;
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefreshProviderFailure {
    /// Local validation/preparation failed before external exchange.
    NotDispatched,
    Rejected,
    /// Transport, malformed success and unpublishable results never allow replay.
    Uncertain,
}
impl fmt::Display for RefreshProviderFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NotDispatched => "Credential refresh was not dispatched",
            Self::Rejected => "Credential refresh was rejected",
            Self::Uncertain => "Credential refresh outcome is uncertain",
        })
    }
}
impl std::error::Error for RefreshProviderFailure {}
pub type RefreshProviderFuture<'a> = Pin<
    Box<
        dyn Future<Output = Result<DeviceAuthenticationTokens, RefreshProviderFailure>> + Send + 'a,
    >,
>;

/// Pure provider-to-native schema translation. It performs no I/O or refresh;
/// execution/source authority is owned by the application service.
pub trait NativeTokenEncoder: Send + Sync {
    fn encode(
        &self,
        bundle: &ExternalSecret,
        account_id: &str,
    ) -> Result<ExternalSecret, NativeTokenEncodingFailure>;
}
#[derive(Debug, Clone, Copy)]
pub struct NativeTokenEncodingFailure;
impl fmt::Display for NativeTokenEncodingFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Credentials are not eligible for native token application")
    }
}
impl std::error::Error for NativeTokenEncodingFailure {}
