//! Trusted synthetic provider injected at fixture composition, never an issuer option.
use chapchoo_service::{
    DeviceApproval, DeviceAuthenticationProvider, DeviceAuthenticationTokens,
    DeviceAuthorizationDraft, DevicePollOutcome, DeviceProviderFuture, ExternalSecret,
};
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Default)]
pub struct Provider {
    pub issues: AtomicUsize,
    pub polls: AtomicUsize,
    pub exchanges: AtomicUsize,
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
