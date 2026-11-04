use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::{BootstrapProviderPermit, CredentialCipher, RootKey};
use chapchoo_persistence::{Store, prepare_sqlite};
use chapchoo_service::{AcceptanceTime, ComputersService, DomainReason, ProviderCapabilities};
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
            ..Default::default()
        };
        let app = ComputersService::new(store).with_policy(policy)?;
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
        let proof = BootstrapProviderPermit::new("provider".into())?.authenticated();
        let key = app
            .exchange_bootstrap_provider(
                &proof,
                "Fixture",
                ProviderCapabilities {
                    create: true,
                    start: true,
                    stop: true,
                    restart: true,
                },
                &chapchoo_domain::ProviderSubstrate::Kubernetes,
                &format!(
                    "{}:key",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)?
                        .as_millis()
                ),
                AcceptanceTime::System,
            )
            .await?;
        assert_eq!(
            key.expires_at - key.issued_at,
            policy.provider_keys.validity_ms
        );
        assert_eq!(
            key.proactive_after - key.issued_at,
            policy.provider_keys.proactive_after_ms
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
fn bootstrap_exchange_recovers_exact_result_and_preserves_connector_policy() -> Result<(), TestError>
{
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("exchange-service.sqlite");
    let cipher = Arc::new(CredentialCipher::new(RootKey::parse(
        URL_SAFE.encode([7; 32]),
    )?)?);
    prepare_sqlite(&path, &cipher)?;
    tokio::runtime::Runtime::new()?.block_on(async {
        let store = Store::connect_sqlite(path, cipher).await?;
        let app = ComputersService::new(store.clone());
        let proof = BootstrapProviderPermit::new("allowed-provider".into())?.authenticated();
        let caps = ProviderCapabilities {
            create: true,
            start: true,
            stop: true,
            restart: true,
        };
        let substrate = chapchoo_domain::ProviderSubstrate::Kubernetes;
        let key = app
            .exchange_bootstrap_provider(
                &proof,
                "Fixture",
                caps,
                &substrate,
                "1000:issue",
                AcceptanceTime::Fixed(1000),
            )
            .await?;
        let recovered = app
            .exchange_bootstrap_provider(
                &proof,
                "Fixture",
                caps,
                &substrate,
                "1000:issue",
                AcceptanceTime::Fixed(2000),
            )
            .await?;
        assert_eq!(key.token.expose(), recovered.token.expose());
        assert_eq!(key.expires_at, 901_000);
        assert_eq!(key.proactive_after, 301_000);
        assert_eq!(key.expires_at, recovered.expires_at);
        assert_eq!(app.policy().keys.validity_ms, 86_400_000);
        assert_eq!(app.policy().keys.proactive_after_ms, 43_200_000);
        // Changing the canonical exchange intent cannot recover a previous result.
        assert_eq!(
            app.exchange_bootstrap_provider(
                &proof,
                "Other",
                caps,
                &substrate,
                "1000:issue",
                AcceptanceTime::Fixed(2000)
            )
            .await
            .err()
            .map(|e| e.reason()),
            Some(DomainReason::Conflict)
        );
        assert_eq!(
            app.provider_identity(
                "foreign-provider",
                &key.token.digest(),
                AcceptanceTime::Fixed(2000)
            )
            .await
            .err()
            .map(|e| e.reason()),
            Some(DomainReason::InvalidSession)
        );
        assert_eq!(
            app.exchange_registered_provider(
                "allowed-provider",
                &key.token.digest(),
                &substrate,
                "2000:self-renew",
                AcceptanceTime::Fixed(2000)
            )
            .await
            .err()
            .map(|e| e.reason()),
            Some(DomainReason::PermissionDenied)
        );
        assert!(
            app.provider_identity(
                "allowed-provider",
                &key.token.digest(),
                AcceptanceTime::Fixed(901_000)
            )
            .await
            .is_err()
        );
        // Metadata belongs to the DB after initial bootstrap, not startup claims.
        app.exchange_bootstrap_provider(
            &proof,
            "Claimed replacement",
            ProviderCapabilities {
                create: false,
                ..caps
            },
            &substrate,
            "3000:next",
            AcceptanceTime::Fixed(3000),
        )
        .await?;
        let inventory = store
            .provider_key_identity("allowed-provider", &recovered.token.digest(), 3001)
            .await?;
        assert_eq!(inventory.name, "Fixture");
        store
            .revoke_provider_key(
                chapchoo_persistence::ProviderOwner::System(&proof),
                "allowed-provider",
                "4000:revoke",
                4000,
            )
            .await?;
        assert!(
            app.exchange_bootstrap_provider(
                &proof,
                "Fixture",
                caps,
                &substrate,
                "3000:next",
                AcceptanceTime::Fixed(5000)
            )
            .await
            .is_err()
        );
        assert!(!format!("{app:?} {key:?} {recovered:?}").contains(key.token.expose()));
        Ok::<(), TestError>(())
    })
}
