//! SDK owns the refresh grant; transport/errors remain bounded and body-safe.
use super::*;
use chapchoo_service::{
    ChatGptRefreshProvider, RefreshProviderFailure as Failure, RefreshProviderFuture,
};
use oauth2::{
    AuthType, ClientId, ExtraTokenFields, HttpRequest, HttpResponse, RefreshToken,
    RequestTokenError, StandardRevocableToken, StandardTokenResponse, TokenResponse as _, TokenUrl,
    basic::{
        BasicErrorResponse, BasicErrorResponseType, BasicRevocationErrorResponse,
        BasicTokenIntrospectionResponse, BasicTokenType,
    },
};
#[derive(Deserialize, Serialize)]
struct Extra {
    #[serde(default)]
    id_token: Option<String>,
}
impl fmt::Debug for Extra {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RefreshExtra([REDACTED])")
    }
}
impl ExtraTokenFields for Extra {}
type Tokens = StandardTokenResponse<Extra, BasicTokenType>;
type Sdk = oauth2::Client<
    BasicErrorResponse,
    Tokens,
    BasicTokenIntrospectionResponse,
    StandardRevocableToken,
    BasicRevocationErrorResponse,
>;

#[derive(Debug)]
struct TransportFailure(Failure);
impl fmt::Display for TransportFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
impl Error for TransportFailure {}
struct RefreshTransport<'a>(&'a OpenAiDeviceClient);
impl<'c> oauth2::AsyncHttpClient<'c> for RefreshTransport<'_> {
    type Error = TransportFailure;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<HttpResponse, TransportFailure>> + Send + 'c>,
    >;
    fn call(&'c self, request: HttpRequest) -> Self::Future {
        Box::pin(self.0.refresh_http(request))
    }
}
impl OpenAiDeviceClient {
    async fn refresh_http(&self, request: HttpRequest) -> Result<HttpResponse, TransportFailure> {
        if request.method() != oauth2::http::Method::POST
            || request.uri() != self.exchange.as_str()
            || request.body().len() > MAX_BODY
        {
            return Err(TransportFailure(Failure::NotDispatched));
        }
        let (parts, bytes) = request.into_parts();
        let request = self
            .client
            .post(self.exchange.clone())
            .headers(parts.headers)
            .body(bytes)
            .build()
            .map_err(|_| TransportFailure(Failure::NotDispatched))?;
        let response = self
            .client
            .execute(request)
            .await
            .map_err(|_| TransportFailure(Failure::Uncertain))?;
        let status = response.status();
        // Let the SDK parse bounded standard OAuth rejection bodies. Redirects
        // and retryable HTTP failures remain uncertain, never automatic retries.
        if !status.is_success() && !matches!(status.as_u16(), 400 | 401 | 403) {
            return Err(TransportFailure(Failure::Uncertain));
        }
        let headers = response.headers().clone();
        let bytes = body(response)
            .await
            .map_err(|_| TransportFailure(Failure::Uncertain))?;
        let mut output = oauth2::http::Response::builder().status(status);
        *output
            .headers_mut()
            .ok_or(TransportFailure(Failure::Uncertain))? = headers;
        output
            .body(bytes.to_vec())
            .map_err(|_| TransportFailure(Failure::Uncertain))
    }
}
impl ChatGptRefreshProvider for OpenAiDeviceClient {
    fn refresh<'a>(
        &'a self,
        bundle: &'a ExternalSecret,
        account_id: &'a str,
        now: i64,
    ) -> RefreshProviderFuture<'a> {
        Box::pin(async move {
            let old: TokenBundle =
                serde_json::from_slice(bundle.expose()).map_err(|_| Failure::NotDispatched)?;
            if !(0..=MAX_AUTHENTICATION_EXPIRY_MILLIS).contains(&now) {
                return Err(Failure::NotDispatched);
            }
            required(account_id, 4096).map_err(|_| Failure::NotDispatched)?;
            required(&old.refresh_token, 65536).map_err(|_| Failure::NotDispatched)?;
            if metadata(old.id_token.as_deref()).account_id.as_deref() != Some(account_id) {
                return Err(Failure::NotDispatched);
            }
            let sdk = Sdk::new(ClientId::new(CLIENT_ID.into()))
                .set_auth_type(AuthType::RequestBody)
                .set_token_uri(
                    TokenUrl::new(self.exchange.to_string()).map_err(|_| Failure::NotDispatched)?,
                );
            let refresh = RefreshToken::new(old.refresh_token.clone());
            let transport = RefreshTransport(self);
            let tokens = sdk
                .exchange_refresh_token(&refresh)
                .request_async(&transport)
                .await
                .map_err(|error| match error {
                    RequestTokenError::Request(TransportFailure(reason)) => reason,
                    RequestTokenError::ServerResponse(response) => match response.error() {
                        BasicErrorResponseType::InvalidClient
                        | BasicErrorResponseType::InvalidGrant
                        | BasicErrorResponseType::InvalidRequest
                        | BasicErrorResponseType::InvalidScope
                        | BasicErrorResponseType::UnauthorizedClient
                        | BasicErrorResponseType::UnsupportedGrantType => Failure::Rejected,
                        _ => Failure::Uncertain,
                    },
                    // Never return SDK errors: parse failures may retain response bodies.
                    _ => Failure::Uncertain,
                })?;
            if tokens.token_type() != &BasicTokenType::Bearer {
                return Err(Failure::Uncertain);
            }
            let access_token = tokens.access_token().secret();
            let refresh_token = tokens
                .refresh_token()
                .map_or(old.refresh_token.as_str(), |token| token.secret());
            let id_token = tokens
                .extra_fields()
                .id_token
                .as_deref()
                .or(old.id_token.as_deref());
            required(access_token, 65536).map_err(|_| Failure::Uncertain)?;
            if metadata(Some(access_token))
                .account_id
                .as_deref()
                .is_some_and(|value| value != account_id)
            {
                return Err(Failure::Uncertain);
            }
            required(refresh_token, 65536).map_err(|_| Failure::Uncertain)?;
            required(id_token.ok_or(Failure::Uncertain)?, 65536).map_err(|_| Failure::Uncertain)?;
            let metadata = metadata(id_token);
            if metadata.account_id.as_deref() != Some(account_id) {
                return Err(Failure::Uncertain);
            }
            let expires_at = tokens
                .expires_in()
                .map(|expires| {
                    let millis =
                        i64::try_from(expires.as_millis()).map_err(|_| Failure::Uncertain)?;
                    now.checked_add(millis)
                        .filter(|expiry| *expiry <= MAX_AUTHENTICATION_EXPIRY_MILLIS)
                        .ok_or(Failure::Uncertain)
                })
                .transpose()?;
            let renewed = TokenBundle {
                access_token: access_token.clone(),
                refresh_token: refresh_token.into(),
                id_token: id_token.map(Into::into),
            };
            let secret =
                ExternalSecret::new(serde_json::to_vec(&renewed).map_err(|_| Failure::Uncertain)?)
                    .map_err(|_| Failure::Uncertain)?;
            // Publish only a candidate usable by the same native encoding boundary.
            native_chatgpt_credentials(&secret, account_id, now).map_err(|_| Failure::Uncertain)?;
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
