//! Bounded device acquisition and public-SDK central refresh.
mod execution;
mod refresh;
pub use execution::{ExecutionCredentialError, native_chatgpt_credentials};

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chapchoo_service::{
    DeviceApproval, DeviceAuthenticationProvider, DeviceAuthenticationTokens,
    DeviceAuthorizationDraft, DevicePollOutcome, DeviceProviderFailure, DeviceProviderFuture,
    ExternalSecret, MAX_AUTHENTICATION_EXPIRY_MILLIS,
};
use reqwest::{Client, ClientBuilder, Response, Url};
use rustls_platform_verifier::BuilderVerifierExt;
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt, sync::Arc, time::Duration};
use zeroize::{ZeroizeOnDrop, Zeroizing};

const ISSUER: &str = "https://auth.openai.com/";
const CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
const REDIRECT_URI: &str = "https://auth.openai.com/deviceauth/callback";
pub use chapchoo_service::DEVICE_VERIFICATION_URI;
const MAX_BODY: usize = 131_072;
const DEVICE_VALIDITY_MS: i64 = 900_000;

pub struct OpenAiDeviceClient {
    client: Client,
    issue: Url,
    poll: Url,
    exchange: Url,
}
impl fmt::Debug for OpenAiDeviceClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("OpenAiDeviceClient([REDACTED])")
    }
}
pub struct ClientPreparationError(Box<dyn Error + Send + Sync>);
impl fmt::Debug for ClientPreparationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ClientPreparationError([REDACTED])")
    }
}
impl fmt::Display for ClientPreparationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("OpenAI authentication client preparation failed")
    }
}
impl Error for ClientPreparationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.0.as_ref())
    }
}
fn preparation(error: impl Error + Send + Sync + 'static) -> ClientPreparationError {
    ClientPreparationError(Box::new(error))
}

fn builder() -> Result<ClientBuilder, ClientPreparationError> {
    // Use the existing workspace's Ring provider explicitly, without mutating
    // a process-global TLS provider or disabling platform certificate checks.
    let tls = rustls::ClientConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .map_err(preparation)?
    .with_platform_verifier()
    .map_err(preparation)?
    .with_no_client_auth();
    Ok(Client::builder()
        .use_preconfigured_tls(tls)
        .redirect(reqwest::redirect::Policy::none())
        .retry(reqwest::retry::never())
        .connect_timeout(Duration::from_secs(3))
        .timeout(Duration::from_secs(10)))
}
impl OpenAiDeviceClient {
    /// Vendor endpoints are fixed protocol constants, not caller-supplied issuers.
    pub fn new() -> Result<Self, ClientPreparationError> {
        let origin = Url::parse(ISSUER).map_err(preparation)?;
        Self::from_parts(
            builder()?.https_only(true).build().map_err(preparation)?,
            &origin,
        )
    }
    fn from_parts(client: Client, origin: &Url) -> Result<Self, ClientPreparationError> {
        Ok(Self {
            client,
            issue: origin
                .join("api/accounts/deviceauth/usercode")
                .map_err(preparation)?,
            poll: origin
                .join("api/accounts/deviceauth/token")
                .map_err(preparation)?,
            exchange: origin.join("oauth/token").map_err(preparation)?,
        })
    }
    #[cfg(test)]
    fn fixture(origin: &Url) -> Result<Self, ClientPreparationError> {
        Self::from_parts(
            builder()?
                .https_only(false)
                .no_proxy()
                .build()
                .map_err(preparation)?,
            origin,
        )
    }
}

fn status_failure(status: u16) -> DeviceProviderFailure {
    if status == 429 || status >= 500 {
        DeviceProviderFailure::Unavailable
    } else {
        DeviceProviderFailure::Rejected
    }
}
async fn body(mut response: Response) -> Result<Zeroizing<Vec<u8>>, DeviceProviderFailure> {
    if response
        .content_length()
        .is_some_and(|n| usize::try_from(n).map_or(true, |n| n > MAX_BODY))
    {
        return Err(DeviceProviderFailure::InvalidResponse);
    }
    let mut bytes = Zeroizing::new(Vec::new());
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| DeviceProviderFailure::Uncertain)?
    {
        if chunk.len() > MAX_BODY.saturating_sub(bytes.len()) {
            return Err(DeviceProviderFailure::InvalidResponse);
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}
fn required(value: &str, max: usize) -> Result<(), DeviceProviderFailure> {
    if value.is_empty() || value.len() > max || value.chars().any(char::is_control) {
        return Err(DeviceProviderFailure::InvalidResponse);
    }
    Ok(())
}
fn text(secret: &ExternalSecret, max: usize) -> Result<&str, DeviceProviderFailure> {
    let value =
        std::str::from_utf8(secret.expose()).map_err(|_| DeviceProviderFailure::InvalidResponse)?;
    required(value, max)?;
    Ok(value)
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Seconds {
    Number(u64),
    Text(String),
}
impl Seconds {
    fn value(self) -> Result<i64, DeviceProviderFailure> {
        let value = match self {
            Self::Number(n) => n,
            Self::Text(s) => s
                .trim()
                .parse::<u64>()
                .map_err(|_| DeviceProviderFailure::InvalidResponse)?,
        };
        i64::try_from(value).map_err(|_| DeviceProviderFailure::InvalidResponse)
    }
}
#[derive(Deserialize, ZeroizeOnDrop)]
struct IssueResponse {
    device_auth_id: String,
    #[serde(alias = "usercode")]
    user_code: String,
    #[serde(default)]
    #[zeroize(skip)]
    interval: Option<Seconds>,
}
#[derive(Serialize)]
struct IssueRequest {
    client_id: &'static str,
}
#[derive(Serialize)]
struct PollRequest<'a> {
    device_auth_id: &'a str,
    user_code: &'a str,
}
#[derive(Deserialize, ZeroizeOnDrop)]
struct ApprovalResponse {
    authorization_code: String,
    code_verifier: String,
}
#[derive(Serialize)]
struct ExchangeRequest<'a> {
    grant_type: &'static str,
    client_id: &'static str,
    redirect_uri: &'static str,
    code: &'a str,
    code_verifier: &'a str,
}
#[derive(Deserialize, ZeroizeOnDrop)]
struct TokenResponse {
    access_token: String,
    refresh_token: String,
    id_token: Option<String>,
    #[serde(default)]
    #[zeroize(skip)]
    expires_in: Option<Seconds>,
}
#[derive(Serialize, Deserialize, ZeroizeOnDrop)]
struct TokenBundle {
    access_token: String,
    refresh_token: String,
    id_token: Option<String>,
}
#[derive(Default, Deserialize)]
struct IdentityClaims {
    email: Option<String>,
    #[serde(rename = "https://api.openai.com/auth")]
    auth: Option<AccountClaims>,
}
#[derive(Deserialize)]
struct AccountClaims {
    chatgpt_account_id: Option<String>,
    chatgpt_plan_type: Option<String>,
}
#[derive(Default)]
struct Metadata {
    account_id: Option<String>,
    email: Option<String>,
    plan_type: Option<String>,
}
fn optional(value: Option<String>) -> Option<String> {
    value.filter(|value| required(value, 4096).is_ok() && !value.trim().is_empty())
}
fn metadata(token: Option<&str>) -> Metadata {
    let Some(token) = token else {
        return Metadata::default();
    };
    let mut parts = token.split('.');
    let (Some(_), Some(payload), Some(_), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Metadata::default();
    };
    let Ok(decoded) = URL_SAFE_NO_PAD.decode(payload) else {
        return Metadata::default();
    };
    let decoded = Zeroizing::new(decoded);
    let Ok(claims) = serde_json::from_slice::<IdentityClaims>(&decoded) else {
        return Metadata::default();
    };
    let (account_id, plan_type) = claims.auth.map_or((None, None), |auth| {
        (
            optional(auth.chatgpt_account_id),
            optional(auth.chatgpt_plan_type),
        )
    });
    Metadata {
        account_id,
        email: optional(claims.email),
        plan_type,
    }
}

impl DeviceAuthenticationProvider for OpenAiDeviceClient {
    fn issue(&self, now: i64) -> DeviceProviderFuture<'_, DeviceAuthorizationDraft> {
        Box::pin(async move {
            let response = self
                .client
                .post(self.issue.clone())
                .json(&IssueRequest {
                    client_id: CLIENT_ID,
                })
                .send()
                .await
                .map_err(|_| DeviceProviderFailure::Uncertain)?;
            if !response.status().is_success() {
                return Err(status_failure(response.status().as_u16()));
            }
            let bytes = body(response).await?;
            let mut value = serde_json::from_slice::<IssueResponse>(&bytes)
                .map_err(|_| DeviceProviderFailure::InvalidResponse)?;
            required(&value.device_auth_id, 4096)?;
            required(&value.user_code, 128)?;
            let seconds = value
                .interval
                .take()
                .map(Seconds::value)
                .transpose()?
                .unwrap_or(5);
            let interval_ms = seconds
                .checked_mul(1000)
                .filter(|ms| (1000..=900000).contains(ms))
                .ok_or(DeviceProviderFailure::InvalidResponse)?;
            let expires_at = now
                .checked_add(DEVICE_VALIDITY_MS)
                .filter(|_| now >= 0)
                .ok_or(DeviceProviderFailure::InvalidResponse)?;
            Ok(DeviceAuthorizationDraft {
                device_auth_id: ExternalSecret::new(
                    std::mem::take(&mut value.device_auth_id).into_bytes(),
                )
                .map_err(|_| DeviceProviderFailure::InvalidResponse)?,
                user_code: std::mem::take(&mut value.user_code),
                interval_ms,
                expires_at,
            })
        })
    }
    fn poll<'a>(
        &'a self,
        device_auth_id: &'a ExternalSecret,
        user_code: &'a str,
    ) -> DeviceProviderFuture<'a, DevicePollOutcome> {
        Box::pin(async move {
            required(user_code, 128)?;
            let response = self
                .client
                .post(self.poll.clone())
                .json(&PollRequest {
                    device_auth_id: text(device_auth_id, 4096)?,
                    user_code,
                })
                .send()
                .await
                .map_err(|_| DeviceProviderFailure::Uncertain)?;
            if matches!(response.status().as_u16(), 403 | 404) {
                return Ok(DevicePollOutcome::Pending);
            }
            if !response.status().is_success() {
                return Err(status_failure(response.status().as_u16()));
            }
            let bytes = body(response).await?;
            let mut value = serde_json::from_slice::<ApprovalResponse>(&bytes)
                .map_err(|_| DeviceProviderFailure::InvalidResponse)?;
            required(&value.authorization_code, 16384)?;
            required(&value.code_verifier, 4096)?;
            Ok(DevicePollOutcome::Approved(DeviceApproval {
                authorization_code: ExternalSecret::new(
                    std::mem::take(&mut value.authorization_code).into_bytes(),
                )
                .map_err(|_| DeviceProviderFailure::InvalidResponse)?,
                code_verifier: ExternalSecret::new(
                    std::mem::take(&mut value.code_verifier).into_bytes(),
                )
                .map_err(|_| DeviceProviderFailure::InvalidResponse)?,
            }))
        })
    }
    fn exchange<'a>(
        &'a self,
        approval: &'a DeviceApproval,
        now: i64,
    ) -> DeviceProviderFuture<'a, DeviceAuthenticationTokens> {
        Box::pin(async move {
            let response = self
                .client
                .post(self.exchange.clone())
                .form(&ExchangeRequest {
                    grant_type: "authorization_code",
                    client_id: CLIENT_ID,
                    redirect_uri: REDIRECT_URI,
                    code: text(&approval.authorization_code, 16384)?,
                    code_verifier: text(&approval.code_verifier, 4096)?,
                })
                .send()
                .await
                .map_err(|_| DeviceProviderFailure::Uncertain)?;
            if !response.status().is_success() {
                return Err(status_failure(response.status().as_u16()));
            }
            let bytes = body(response).await?;
            let mut value = serde_json::from_slice::<TokenResponse>(&bytes)
                .map_err(|_| DeviceProviderFailure::InvalidResponse)?;
            required(&value.access_token, 65536)?;
            required(&value.refresh_token, 65536)?;
            if let Some(id) = &value.id_token {
                required(id, 65536)?;
            }
            let metadata = metadata(value.id_token.as_deref());
            let expires_at = value
                .expires_in
                .take()
                .map(|seconds| {
                    let seconds = seconds.value()?;
                    now.checked_add(
                        seconds
                            .checked_mul(1000)
                            .ok_or(DeviceProviderFailure::InvalidResponse)?,
                    )
                    .filter(|expiry| {
                        now >= 0 && (0..=MAX_AUTHENTICATION_EXPIRY_MILLIS).contains(expiry)
                    })
                    .ok_or(DeviceProviderFailure::InvalidResponse)
                })
                .transpose()?;
            let bundle = TokenBundle {
                access_token: std::mem::take(&mut value.access_token),
                refresh_token: std::mem::take(&mut value.refresh_token),
                id_token: value.id_token.take(),
            };
            let secret = ExternalSecret::new(
                serde_json::to_vec(&bundle).map_err(|_| DeviceProviderFailure::InvalidResponse)?,
            )
            .map_err(|_| DeviceProviderFailure::InvalidResponse)?;
            Ok(DeviceAuthenticationTokens {
                secret,
                account_id: metadata.account_id,
                email: metadata.email,
                plan_type: metadata.plan_type,
                expires_at,
            })
        })
    }
}

#[cfg(test)]
mod tests;
