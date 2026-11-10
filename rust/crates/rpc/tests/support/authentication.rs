//! Trusted synthetic provider injected at fixture composition, never an issuer option.
use chapchoo_service::{
    DeviceApproval, DeviceAuthenticationProvider, DeviceAuthenticationTokens,
    DeviceAuthorizationDraft, DevicePollOutcome, DeviceProviderFuture, ExternalSecret,
};
use std::sync::atomic::{AtomicUsize, Ordering};

pub async fn register_personal(
    store: &chapchoo_persistence::Store,
    actor: &chapchoo_crypto::SecretDigest,
    now: i64,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    use chapchoo_persistence::{AuthenticationAcceptedResult, AuthenticationScope, RegisterApiKey};
    match store
        .register_external_api_key(
            actor,
            &RegisterApiKey {
                scope: AuthenticationScope::Personal,
                request_id: "fixture-agent-authentication".into(),
                name: "Fixture authentication".into(),
                key: ExternalSecret::new(b"synthetic-fixture-api-key".to_vec())?,
            },
            now,
        )
        .await?
    {
        AuthenticationAcceptedResult::Record { authentication_id } => Ok(authentication_id),
        _ => Err("Missing fixture authentication identity".into()),
    }
}

#[derive(Debug, Default)]
pub struct Provider {
    pub issues: AtomicUsize,
    pub polls: AtomicUsize,
    pub exchanges: AtomicUsize,
    pub refreshes: AtomicUsize,
}
impl DeviceAuthenticationProvider for Provider {
    fn issue(&self, now: i64) -> DeviceProviderFuture<'_, DeviceAuthorizationDraft> {
        Box::pin(async move {
            self.issues.fetch_add(1, Ordering::SeqCst);
            Ok(DeviceAuthorizationDraft {
                device_auth_id: ExternalSecret::new(b"fixture-device-secret".to_vec())
                    .map_err(|_| chapchoo_service::DeviceProviderFailure::InvalidResponse)?,
                user_code: "FIXTURE-CODE".into(),
                interval_ms: 1000,
                expires_at: now + 900_000,
            })
        })
    }
    fn poll<'a>(
        &'a self,
        _: &'a ExternalSecret,
        _: &'a str,
    ) -> DeviceProviderFuture<'a, DevicePollOutcome> {
        Box::pin(async move {
            self.polls.fetch_add(1, Ordering::SeqCst);
            Ok(DevicePollOutcome::Approved(DeviceApproval {
                authorization_code: ExternalSecret::new(b"fixture-code".to_vec())
                    .map_err(|_| chapchoo_service::DeviceProviderFailure::InvalidResponse)?,
                code_verifier: ExternalSecret::new(b"fixture-verifier".to_vec())
                    .map_err(|_| chapchoo_service::DeviceProviderFailure::InvalidResponse)?,
            }))
        })
    }
    fn exchange<'a>(
        &'a self,
        _: &'a DeviceApproval,
        now: i64,
    ) -> DeviceProviderFuture<'a, DeviceAuthenticationTokens> {
        Box::pin(async move {
            self.exchanges.fetch_add(1, Ordering::SeqCst);
            Ok(DeviceAuthenticationTokens {
                secret: ExternalSecret::new(b"fixture-token-bundle".to_vec())
                    .map_err(|_| chapchoo_service::DeviceProviderFailure::InvalidResponse)?,
                account_id: Some("fixture-account".into()),
                email: Some("fixture@example.test".into()),
                plan_type: Some("fixture".into()),
                expires_at: Some(now + 3_600_000),
            })
        })
    }
}
impl chapchoo_service::ChatGptRefreshProvider for Provider {
    fn refresh<'a>(
        &'a self,
        _: &'a ExternalSecret,
        account: &'a str,
        now: i64,
    ) -> chapchoo_service::RefreshProviderFuture<'a> {
        Box::pin(async move {
            self.refreshes.fetch_add(1, Ordering::SeqCst);
            Ok(DeviceAuthenticationTokens {
                secret: ExternalSecret::new(b"fixture-refreshed-server-bundle".to_vec())
                    .map_err(|_| chapchoo_service::RefreshProviderFailure::Uncertain)?,
                account_id: Some(account.into()),
                email: None,
                plan_type: None,
                expires_at: Some(now + 3_600_000),
            })
        })
    }
}
impl chapchoo_service::NativeTokenEncoder for Provider {
    fn encode(
        &self,
        _: &ExternalSecret,
        account: &str,
        _: i64,
    ) -> Result<ExternalSecret, chapchoo_service::NativeTokenEncodingFailure> {
        if account != "fixture-account" {
            return Err(chapchoo_service::NativeTokenEncodingFailure);
        }
        ExternalSecret::new(br#"{"auth_mode":"chatgptAuthTokens","OPENAI_API_KEY":null,"last_refresh":"2026-11-10T02:00:00Z","tokens":{"id_token":"fixture-access.jwt","access_token":"fixture-access.jwt","refresh_token":"","account_id":"fixture-account"}}"#.to_vec())
            .map_err(|_|chapchoo_service::NativeTokenEncodingFailure)
    }
}
pub fn execution(
    fixture: &mut super::Fixture,
    store: chapchoo_persistence::Store,
) -> Result<
    (
        std::sync::Arc<chapchoo_service::ExecutionAuthenticationService>,
        std::sync::Arc<Provider>,
    ),
    Box<dyn std::error::Error + Send + Sync>,
> {
    let provider = std::sync::Arc::new(Provider::default());
    let service = std::sync::Arc::new(chapchoo_service::ExecutionAuthenticationService::new(
        store,
        provider.clone(),
        provider.clone(),
        4,
    )?);
    let owner = service.clone();
    let stopped = fixture.shutdown();
    fixture.server(async move {
        let _ = stopped.await;
        owner.close();
        owner.finish().await
    });
    Ok((service, provider))
}
