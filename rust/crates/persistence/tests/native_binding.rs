//! The server owns immutable native binding; external Providers use scoped RPC.
mod support;
use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::{BearerSecret, CredentialCipher, RootKey, SystemOwnerVerifier};
use chapchoo_domain::{DockerDaemonId, ProviderSubstrate};
use chapchoo_persistence::{
    ProviderCapabilities, Store, StoreError, prepare_postgres, prepare_sqlite,
};
use diesel::{Connection, connection::SimpleConnection};
use diesel_async::{AsyncPgConnection, async_connection_wrapper::AsyncConnectionWrapper};
use std::sync::Arc;

type Error = Box<dyn std::error::Error + Send + Sync>;

fn cipher() -> Result<Arc<CredentialCipher>, Error> {
    Ok(Arc::new(CredentialCipher::new(RootKey::parse(
        URL_SAFE.encode([7; 32]),
    )?)?))
}

async fn exercise(store: Store) -> Result<(), Error> {
    let token = BearerSecret::generate()?;
    let verifier = SystemOwnerVerifier::new(
        "system".into(),
        vec!["docker".into(), "race".into()],
        token.digest(),
    )?;
    let proof = verifier.verify("docker", &token)?;
    let caps = ProviderCapabilities {
        create: true,
        start: true,
        stop: true,
        restart: true,
    };
    let original = ProviderSubstrate::Docker(DockerDaemonId::parse("daemon:original".into())?);
    let substituted =
        ProviderSubstrate::Docker(DockerDaemonId::parse("daemon:replacement".into())?);
    store
        .configure_system_provider(&proof, "Original", caps, &original, 1)
        .await?;
    store
        .configure_system_provider(&proof, "Same-binding metadata", caps, &original, 2)
        .await?;
    assert!(matches!(
        store
            .configure_system_provider(&proof, "Substitute", caps, &substituted, 3)
            .await,
        Err(StoreError::Conflict)
    ));
    assert!(matches!(
        store
            .configure_system_provider(&proof, "Relabel", caps, &ProviderSubstrate::Kubernetes, 4)
            .await,
        Err(StoreError::Conflict)
    ));
    let other_token = BearerSecret::generate()?;
    let other_owner = SystemOwnerVerifier::new(
        "other-system".into(),
        vec!["docker".into()],
        other_token.digest(),
    )?;
    let wrong_owner = other_owner.verify("docker", &other_token)?;
    assert!(matches!(
        store
            .configure_system_provider(&wrong_owner, "Wrong owner", caps, &original, 5)
            .await,
        Err(StoreError::Forbidden)
    ));
    assert!(verifier.verify("outside-scope", &token).is_err());
    // No failed substitution or owner operation can clear the original binding.
    store
        .configure_system_provider(&proof, "Original retained", caps, &original, 6)
        .await?;

    let race_proof = verifier.verify("race", &token)?;
    let (a, b) = tokio::join!(
        store.configure_system_provider(&race_proof, "Candidate A", caps, &original, 7),
        store.configure_system_provider(&race_proof, "Candidate B", caps, &substituted, 7),
    );
    assert_ne!(
        a.is_ok(),
        b.is_ok(),
        "Exactly one different first binding may win"
    );
    let (winner, loser) = if a.is_ok() {
        (&original, &substituted)
    } else {
        (&substituted, &original)
    };
    assert!(matches!(
        if a.is_ok() { b } else { a },
        Err(StoreError::Conflict)
    ));
    // Same-value recovery is a DB retry, not a new native target or mutation replay.
    store
        .configure_system_provider(&race_proof, "Recovered", caps, winner, 8)
        .await?;
    assert!(matches!(
        store
            .configure_system_provider(&race_proof, "Late substitute", caps, loser, 9)
            .await,
        Err(StoreError::Conflict)
    ));
    Ok(())
}

#[test]
fn sqlite_native_binding_is_scoped_immutable_and_race_safe() -> Result<(), Error> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("binding.sqlite");
    let cipher = cipher()?;
    prepare_sqlite(&path, &cipher)?;
    tokio::runtime::Runtime::new()?.block_on(async {
        let store = Store::connect_sqlite(path, cipher).await?;
        exercise(store).await
    })
}

#[test]
fn postgres_native_binding_is_scoped_immutable_and_race_safe() -> Result<(), Error> {
    let base = std::env::var("CHAPCHOO_TEST_POSTGRES_URL")?;
    let mut admin = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&base)?;
    let suffix = BearerSecret::generate()?.digest().as_bytes()[..8]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let schema = format!("native_binding_{suffix}");
    admin.batch_execute(&format!("CREATE SCHEMA {schema}"))?;
    let separator = if base.contains('?') { '&' } else { '?' };
    let url = format!("{base}{separator}options=-csearch_path%3D{schema}");
    let result = (|| {
        let cipher = cipher()?;
        prepare_postgres(&url, &cipher)?;
        tokio::runtime::Runtime::new()?.block_on(async {
            let store = Store::connect_postgres(url, cipher).await?;
            exercise(store).await
        })
    })();
    let cleanup = admin.batch_execute(&format!("DROP SCHEMA {schema} CASCADE"));
    support::combine(result, cleanup.map_err(Into::into)).map(|_| ())
}
