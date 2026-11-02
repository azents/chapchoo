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
fn generated_connection_admission_context_cannot_steal_a_winner() -> Result<(), TestError> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("directory-service.sqlite");
    let cipher = Arc::new(CredentialCipher::new(RootKey::parse(
        URL_SAFE.encode([8; 32]),
    )?)?);
    prepare_sqlite(&path, &cipher)?;
    let owner = BearerSecret::generate()?;
    let verifier =
        SystemOwnerVerifier::new("owner".into(), vec!["provider".into()], owner.digest())?;
    tokio::runtime::Runtime::new()?.block_on(async {
        let store = Store::connect_sqlite(path, cipher).await?;
        let policy = chapchoo_service::ComputerPolicy {
            heartbeat_ms: 30_000,
            directory: chapchoo_service::DirectoryPolicy {
                control_validity_ms: 90_000,
                component_validity_ms: 120_000,
            },
            keys: chapchoo_service::ComponentKeyPolicy {
                validity_ms: 7_200_000,
                proactive_after_ms: 3_600_000,
                overlap_ms: 60_000,
            },
            start_delayed_ms: 600_000,
        };
        let app = ComputersService::new(store, Some(verifier)).with_policy(policy)?;
        for endpoint in [
            "http://0.0.0.0:5000",
            "http://[::]:5000",
            "ftp://localhost:1",
            "http://user@localhost:1",
            "http://localhost:1/peer",
            "http://localhost:1/?secret=1",
        ] {
            assert_eq!(
                app.register_control(endpoint)
                    .await
                    .err()
                    .map(|e| e.reason()),
                Some(DomainReason::InvalidInput)
            );
        }
        let first_control = app.register_control("http://127.0.0.1:1").await?;
        let second_control = app.register_control("http://127.0.0.1:1").await?;
        assert_ne!(first_control.id, second_control.id);
        app.configure_system_provider(
            &owner,
            "provider",
            "Fixture",
            ProviderCapabilities {
                create: true,
                start: true,
                stop: true,
                restart: true,
            },
            AcceptanceTime::System,
        )
        .await?;
        let key = app
            .rotate_system_provider_key(&owner, "provider", "key", AcceptanceTime::System)
            .await?;
        assert_eq!(key.expires_at - key.issued_at, policy.keys.validity_ms);
        assert_eq!(
            key.proactive_after - key.issued_at,
            policy.keys.proactive_after_ms
        );
        let first = app
            .prepare_provider_connection("provider", &key.token.digest(), &first_control.id)
            .await?;
        let second = app
            .prepare_provider_connection("provider", &key.token.digest(), &second_control.id)
            .await?;
        assert_ne!(first.owner().connection_id, second.owner().connection_id);
        let admitted = app.admit_component_connection(&first).await?;
        assert_eq!(admitted.owner(), *first.owner());
        assert_eq!(
            app.admit_component_connection(&second)
                .await
                .err()
                .map(|e| e.reason()),
            Some(DomainReason::Conflict)
        );
        app.verify_component_connection(&first).await?;
        assert_eq!(
            app.resolve_provider_route("provider")
                .await?
                .ok_or("Missing admitted route")?
                .component
                .expires_at,
            admitted.expires_at,
            "Frame authorization must not renew heartbeat expiry"
        );
        app.renew_component_connection(&first).await?;
        let replacement = app
            .prepare_provider_connection("provider", &key.token.digest(), &second_control.id)
            .await?;
        app.admit_component_connection(&replacement).await?;
        assert!(app.verify_component_connection(&first).await.is_err());
        assert!(!app.close_component_connection(&first).await?);
        app.drain_control(&first_control.id).await?;
        assert_eq!(
            app.withdraw_drained_control_connections(&first_control.id)
                .await?,
            0
        );
        app.verify_component_connection(&replacement).await?;
        assert!(!format!("{replacement:?} {admitted:?}").contains(key.token.expose()));
        app.drain_control(&second_control.id).await?;
        assert!(app.verify_component_connection(&replacement).await.is_err());
        assert_eq!(
            app.withdraw_drained_control_connections(&second_control.id)
                .await?,
            1
        );
        assert!(app.resolve_provider_route("provider").await?.is_none());
        Ok::<(), TestError>(())
    })
}

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
