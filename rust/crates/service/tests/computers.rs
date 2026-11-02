use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::{CredentialCipher, RootKey};
use chapchoo_persistence::{Store, prepare_sqlite};
use chapchoo_service::{
    AcceptanceTime, BearerSecret, ComputersService, DomainReason, ProviderCapabilities,
    SystemOwnerVerifier,
};
use std::{error::Error, sync::Arc};
type TestError = Box<dyn Error + Send + Sync>;

#[test]
fn configured_owner_scope_and_generated_key_recovery_are_composed() -> Result<(), TestError> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("computer-service.sqlite");
    let cipher = Arc::new(CredentialCipher::new(RootKey::parse(
        URL_SAFE.encode([7; 32]),
    )?)?);
    prepare_sqlite(&path, &cipher)?;
    let owner = BearerSecret::generate()?;
    let wrong = BearerSecret::generate()?;
    let verifier = SystemOwnerVerifier::new(
        "configured-owner".into(),
        vec!["allowed-provider".into()],
        owner.digest(),
    )?;
    tokio::runtime::Runtime::new()?.block_on(async {
        let store = Store::connect_sqlite(path, cipher).await?;
        let unconfigured = ComputersService::new(store.clone(), None);
        let app = ComputersService::new(store, Some(verifier));
        let clock = AcceptanceTime::Fixed(1000);
        let capabilities = ProviderCapabilities {
            create: true,
            start: true,
            stop: true,
            restart: true,
        };
        for (service, credential, provider) in [
            (&unconfigured, &owner, "allowed-provider"),
            (&app, &wrong, "allowed-provider"),
            (&app, &owner, "asserted-provider"),
        ] {
            assert_eq!(
                service
                    .configure_system_provider(credential, provider, "Fixture", capabilities, clock)
                    .await
                    .err()
                    .map(|e| e.reason()),
                Some(DomainReason::PermissionDenied)
            );
        }
        app.configure_system_provider(&owner, "allowed-provider", "Fixture", capabilities, clock)
            .await?;
        let key = app
            .rotate_system_provider_key(&owner, "allowed-provider", "key-one", clock)
            .await?;
        let recovered = app
            .rotate_system_provider_key(
                &owner,
                "allowed-provider",
                "key-one",
                AcceptanceTime::Fixed(2000),
            )
            .await?;
        assert!(key.token.expose() == recovered.token.expose());
        assert_eq!(key.expires_at, 86_401_000);
        assert_eq!(key.expires_at, recovered.expires_at);
        assert_eq!(key.proactive_after, 43_201_000);
        assert_eq!(
            app.provider_identity("allowed-provider", &key.token.digest(), clock)
                .await?
                .id,
            "allowed-provider"
        );
        // A component key is not the configured owner proof, even for its own Provider.
        assert_eq!(
            app.rotate_system_provider_key(&key.token, "allowed-provider", "self-rotation", clock)
                .await
                .err()
                .map(|e| e.reason()),
            Some(DomainReason::PermissionDenied)
        );
        assert_eq!(
            app.rotate_user_provider_key(
                &key.token.digest(),
                "allowed-provider",
                "wrong-role",
                clock
            )
            .await
            .err()
            .map(|e| e.reason()),
            Some(DomainReason::InvalidSession)
        );
        assert!(!format!("{app:?} {key:?} {recovered:?}").contains(key.token.expose()));
        app.revoke_system_provider_key(&owner, "allowed-provider", "revoke-one", clock)
            .await?;
        assert_eq!(
            app.provider_identity("allowed-provider", &key.token.digest(), clock)
                .await
                .err()
                .map(|e| e.reason()),
            Some(DomainReason::InvalidSession)
        );
        assert_eq!(
            app.rotate_system_provider_key(&owner, "allowed-provider", "key-one", clock)
                .await
                .err()
                .map(|e| e.reason()),
            Some(DomainReason::InvalidTransition)
        );
        assert_eq!(
            app.rotate_system_provider_key(&owner, "allowed-provider", "bad request", clock)
                .await
                .err()
                .map(|e| e.reason()),
            Some(DomainReason::InvalidInput)
        );
        Ok::<(), TestError>(())
    })
}
