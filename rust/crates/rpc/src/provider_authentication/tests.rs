use super::*;
use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::RootKey;
use std::sync::atomic::{AtomicUsize, Ordering};
type Error = Box<dyn std::error::Error + Send + Sync>;
fn cipher() -> Result<Arc<CredentialCipher>, Error> {
    Ok(Arc::new(CredentialCipher::new(RootKey::parse(
        URL_SAFE.encode([29; 32]),
    )?)?))
}
struct Reviewer {
    username: &'static str,
    audience: &'static str,
    calls: AtomicUsize,
    unavailable: bool,
}
#[tonic::async_trait]
impl ServiceAccountReviewer for Reviewer {
    async fn review(&self, token: &str, audience: &str) -> Result<ReviewedServiceAccount, Status> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert_eq!(token, "opaque-provider-token");
        assert_eq!(audience, "chapchoo-provider-auth");
        if self.unavailable {
            return Err(Status::unavailable("Review unavailable"));
        }
        Ok(ReviewedServiceAccount {
            username: self.username.into(),
            audiences: vec![self.audience.into()],
        })
    }
}
fn sa_policy() -> ProviderBootstrapConfig {
    ProviderBootstrapConfig {
        service_accounts: vec![ServiceAccountBinding {
            provider_id: "provider".into(),
            namespace: "installation".into(),
            service_account: "provider-sa".into(),
            audience: "chapchoo-provider-auth".into(),
        }],
        ..Default::default()
    }
}
#[tokio::test]
async fn reviewed_identity_and_audience_are_both_required_and_not_cached() -> Result<(), Error> {
    for (subject, audience, allowed) in [
        (
            "system:serviceaccount:installation:provider-sa",
            "chapchoo-provider-auth",
            true,
        ),
        (
            "system:serviceaccount:other:provider-sa",
            "chapchoo-provider-auth",
            false,
        ),
        (
            "system:serviceaccount:installation:other",
            "chapchoo-provider-auth",
            false,
        ),
        (
            "system:serviceaccount:installation:provider-sa",
            "kubernetes",
            false,
        ),
    ] {
        let reviewer = Arc::new(Reviewer {
            username: subject,
            audience,
            calls: AtomicUsize::new(0),
            unavailable: false,
        });
        let auth = ProviderAuthenticator::new(sa_policy(), cipher()?, Some(reviewer.clone()))?;
        for _ in 0..2 {
            let result = auth
                .authenticate(
                    "provider",
                    Some(Evidence::ServiceAccountToken(
                        "opaque-provider-token".into(),
                    )),
                    None,
                )
                .await;
            assert_eq!(result.is_ok(), allowed);
        }
        assert_eq!(reviewer.calls.load(Ordering::SeqCst), 2);
        assert!(
            auth.authenticate(
                "unpermitted",
                Some(Evidence::ServiceAccountToken(
                    "opaque-provider-token".into()
                )),
                None
            )
            .await
            .is_err()
        );
        assert_eq!(reviewer.calls.load(Ordering::SeqCst), 2);
    }
    Ok(())
}
#[tokio::test]
async fn review_outage_does_not_select_internal_trust() -> Result<(), Error> {
    let reviewer = Arc::new(Reviewer {
        username: "",
        audience: "",
        calls: AtomicUsize::new(0),
        unavailable: true,
    });
    let mut policy = sa_policy();
    policy.trust_provider_ids.push("provider".into());
    let auth = ProviderAuthenticator::new(policy, cipher()?, Some(reviewer))?;
    let result = auth
        .authenticate(
            "provider",
            Some(Evidence::ServiceAccountToken(
                "opaque-provider-token".into(),
            )),
            Some("provider"),
        )
        .await;
    assert_eq!(
        result.err().ok_or("Expected failure")?.code(),
        tonic::Code::Unavailable
    );
    Ok(())
}
#[tokio::test]
async fn trust_requires_both_private_channel_binding_and_explicit_permit() -> Result<(), Error> {
    let auth = ProviderAuthenticator::new(
        ProviderBootstrapConfig {
            trust_provider_ids: vec!["provider".into()],
            ..Default::default()
        },
        cipher()?,
        None,
    )?;
    assert!(
        auth.authenticate("provider", None, Some("provider"))
            .await
            .is_ok()
    );
    assert!(auth.authenticate("provider", None, None).await.is_err());
    assert!(
        auth.authenticate("provider", None, Some("other"))
            .await
            .is_err()
    );
    assert!(
        auth.authenticate("other", None, Some("other"))
            .await
            .is_err()
    );
    assert!(
        auth.authenticate(
            "provider",
            Some(Evidence::BootstrapCredential("invalid".into())),
            Some("provider")
        )
        .await
        .is_err()
    );
    let disabled = ProviderAuthenticator::new(ProviderBootstrapConfig::default(), cipher()?, None)?;
    assert!(
        disabled
            .authenticate("provider", None, Some("provider"))
            .await
            .is_err()
    );
    Ok(())
}
#[tokio::test]
async fn protected_proof_requires_matching_bootstrap_id_and_root() -> Result<(), Error> {
    let root = cipher()?;
    let proof = root.issue_provider_bootstrap("provider")?;
    let auth = ProviderAuthenticator::new(
        ProviderBootstrapConfig {
            secret_key_provider_ids: vec!["provider".into(), "other".into()],
            ..Default::default()
        },
        root,
        None,
    )?;
    assert!(
        auth.authenticate(
            "provider",
            Some(Evidence::BootstrapCredential(proof.expose().into())),
            None
        )
        .await
        .is_ok()
    );
    assert!(
        auth.authenticate(
            "other",
            Some(Evidence::BootstrapCredential(proof.expose().into())),
            None
        )
        .await
        .is_err()
    );
    let disabled = ProviderAuthenticator::new(ProviderBootstrapConfig::default(), cipher()?, None)?;
    assert!(
        disabled
            .authenticate(
                "provider",
                Some(Evidence::BootstrapCredential(proof.expose().into())),
                None
            )
            .await
            .is_err()
    );
    Ok(())
}
#[test]
fn invalid_or_ambiguous_config_cannot_become_trust() -> Result<(), Error> {
    assert!(ProviderAuthenticator::new(sa_policy(), cipher()?, None).is_err());
    let bad = ProviderBootstrapConfig {
        trust_provider_ids: vec!["provider".into(), "provider".into()],
        ..Default::default()
    };
    assert!(ProviderAuthenticator::new(bad, cipher()?, None).is_err());
    Ok(())
}
