use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::{CredentialCipher, RootKey};
use chapchoo_persistence::{Store, prepare_sqlite};
use chapchoo_service::{
    AccountsConfig, AccountsService, DomainReason, PasswordCost, PasswordWork, SessionPolicy,
};
use std::{error::Error, sync::Arc};

type TestError = Box<dyn Error + Send + Sync>;

#[test]
fn origin_and_session_configuration_is_explicit_and_validated() -> Result<(), TestError> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("configuration.sqlite");
    let cipher = Arc::new(CredentialCipher::new(RootKey::parse(
        URL_SAFE.encode([1; 32]),
    )?)?);
    prepare_sqlite(&path, &cipher)?;
    tokio::runtime::Runtime::new()?.block_on(async {
        let store = Store::connect_sqlite(path, cipher).await?;
        let passwords = PasswordWork::new(1, PasswordCost::new(10)?)?;
        for url in [
            "http://localhost:3000",
            "http://127.0.0.1:3000",
            "http://[::1]:3000",
            "https://web.example.invalid/product",
        ] {
            let service = AccountsService::new(
                store.clone(),
                passwords.clone(),
                None,
                AccountsConfig {
                    general_web_url: url.to_owned(),
                    setup_proof: None,
                    session_policy: SessionPolicy::default(),
                },
            )?;
            assert!(!service.setup_status().await?);
        }
        for url in [
            "http://192.168.1.2",
            "http://example.invalid",
            "http://localhost.example.invalid",
            "https://user:password@example.invalid",
            "https://web.example.invalid?token=private",
            "https://web.example.invalid#private",
            "file:///private",
        ] {
            let result = AccountsService::new(
                store.clone(),
                passwords.clone(),
                None,
                AccountsConfig {
                    general_web_url: url.to_owned(),
                    setup_proof: None,
                    session_policy: SessionPolicy::default(),
                },
            );
            assert!(result.is_err_and(|error| error.reason() == DomainReason::InvalidInput));
        }
        let result = AccountsService::new(
            store,
            passwords,
            None,
            AccountsConfig {
                general_web_url: "http://localhost:3000".to_owned(),
                setup_proof: None,
                session_policy: SessionPolicy {
                    rotation_ms: 0,
                    ..Default::default()
                },
            },
        );
        assert!(result.is_err_and(|error| error.reason() == DomainReason::InvalidInput));
        Ok::<(), TestError>(())
    })
}
