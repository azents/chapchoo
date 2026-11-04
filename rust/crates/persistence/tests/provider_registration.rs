//! Scoped later-registration capability, without adding an onboarding API.
#[path = "support/legacy.rs"]
mod legacy;
mod support;
use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::{BearerSecret, CredentialCipher, RootKey};
use chapchoo_persistence::{
    ComponentKeyDraft, ComponentKeyPolicy, ProviderOwner, ProviderRegistrationOwner, Store,
    provider_registration_context,
};
use diesel::{Connection, RunQueryDsl, connection::SimpleConnection, sql_types::BigInt};
use diesel_async::{AsyncPgConnection, async_connection_wrapper::AsyncConnectionWrapper};
use std::sync::Arc;
type Error = Box<dyn std::error::Error + Send + Sync>;
#[derive(diesel::QueryableByName)]
struct ReceiptCounts {
    #[diesel(sql_type = BigInt)]
    own: i64,
    #[diesel(sql_type = BigInt)]
    other: i64,
}
const COUNTS: &str = "SELECT (SELECT COUNT(*) FROM provider_key_requests r JOIN computer_providers p ON p.id=r.provider_id WHERE p.registration_key='server-provider') AS own, (SELECT COUNT(*) FROM provider_key_requests r JOIN computer_providers p ON p.id=r.provider_id WHERE p.registration_key<>'server-provider') AS other";

async fn retention(
    store: &Store,
    credential: &BearerSecret,
    mut check: impl FnMut(i64) -> Result<(), Error>,
) -> Result<(), Error> {
    let digest = credential.digest();
    let policy = ComponentKeyPolicy::default();
    let draft = |request: &str| -> Result<ComponentKeyDraft, Error> {
        Ok(ComponentKeyDraft {
            request_id: request.into(),
            token: BearerSecret::generate()?,
        })
    };
    let original = store
        .rotate_provider_key(
            registered(&digest),
            "server-provider",
            &draft("1000:exchange-one")?,
            policy,
            2000,
        )
        .await?;
    // Deletion is rolled back when a later key-mutation invariant rejects.
    let failed = ComponentKeyDraft {
        request_id: "901000:failed".into(),
        token: original.key.token,
    };
    assert!(
        store
            .rotate_provider_key(
                registered(&digest),
                "server-provider",
                &failed,
                policy,
                901_000
            )
            .await
            .is_err()
    );
    check(1)?;
    let fresh = store
        .rotate_provider_key(
            registered(&digest),
            "server-provider",
            &draft("901000:fresh")?,
            policy,
            901_000,
        )
        .await?;
    check(1)?;
    assert!(matches!(
        store
            .rotate_provider_key(
                registered(&digest),
                "server-provider",
                &draft("1000:exchange-one")?,
                policy,
                901_000
            )
            .await,
        Err(chapchoo_persistence::StoreError::InvalidTransition)
    ));
    let retry = store
        .rotate_provider_key(
            registered(&digest),
            "server-provider",
            &draft("901000:fresh")?,
            policy,
            901_001,
        )
        .await?;
    assert_eq!(fresh.provider_id, retry.provider_id);
    assert_eq!(fresh.key.token.expose(), retry.key.token.expose());
    assert_eq!(fresh.key.issued_at, retry.key.issued_at);
    assert_eq!(fresh.key.expires_at, retry.key.expires_at);
    // A future creation timestamp cannot be used to extend retention arbitrarily.
    assert!(matches!(
        store
            .rotate_provider_key(
                registered(&digest),
                "server-provider",
                &draft("961002:future")?,
                policy,
                901_001
            )
            .await,
        Err(chapchoo_persistence::StoreError::InvalidInput)
    ));
    check(1)?;
    Ok(())
}

macro_rules! check_counts {
    ($connection:expr, $expected:expr) => {{
        tokio::task::block_in_place(|| {
            let counts = diesel::sql_query(COUNTS).get_result::<ReceiptCounts>($connection)?;
            assert_eq!(counts.own, $expected);
            assert_eq!(counts.other, 2, "Other Providers must not be swept");
            Ok::<_, Error>(())
        })
    }};
}
fn registered(credential: &chapchoo_crypto::SecretDigest) -> ProviderOwner<'_> {
    ProviderOwner::Registration {
        credential,
        substrate: &chapchoo_domain::ProviderSubstrate::Kubernetes,
    }
}
fn blob(bytes: &[u8], postgres: bool) -> String {
    let value = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();
    if postgres {
        format!("decode('{value}','hex')")
    } else {
        format!("X'{value}'")
    }
}
fn seed(
    cipher: &CredentialCipher,
    credentials: &[BearerSecret],
    postgres: bool,
) -> Result<String, Error> {
    let mut sql = String::from("BEGIN;
        INSERT INTO users(id,email_key,delivery_email,password_hash,created_at,language,time_zone,region) VALUES('owner','owner@example.invalid','owner@example.invalid','fixture',0,'en','UTC','en-US');
        INSERT INTO teams(id,kind,name,handle,owner_user_id,created_at) VALUES('auth-team','COLLABORATIVE','Auth','auth-team','owner',0);
        INSERT INTO memberships(team_id,user_id,role) VALUES('auth-team','owner','MANAGER');
        INSERT INTO computer_providers(id,owner_kind,system_owner_id,name,provider_kind,created_at) VALUES('server-provider','SYSTEM','installation','Server','KUBERNETES',0);
        INSERT INTO computer_providers(id,owner_kind,user_owner_id,name,provider_kind,created_at) VALUES('user-provider','USER','owner','User','KUBERNETES',0);
        INSERT INTO computer_providers(id,owner_kind,owner_team_id,name,provider_kind,created_at) VALUES('team-provider','TEAM','auth-team','Team','KUBERNETES',0);");
    for (id, token) in ["server-provider", "user-provider", "team-provider"]
        .iter()
        .zip(credentials)
    {
        let owner = match *id {
            "server-provider" => ProviderRegistrationOwner::System("installation"),
            "user-provider" => ProviderRegistrationOwner::User("owner"),
            _ => ProviderRegistrationOwner::Team("auth-team"),
        };
        let context = provider_registration_context(id, owner)?;
        let protected = cipher.protect_latest(&context, token)?;
        // Storage encryption and token randomness are distinct capabilities.
        assert!(token.digest().matches(&cipher.recover_latest(
            &context,
            &token.digest(),
            &protected
        )?));
        assert!(
            cipher
                .recover_latest("provider-registration:other", &token.digest(), &protected)
                .is_err()
        );
        sql.push_str(&format!("INSERT INTO provider_registration_credentials(provider_id,credential_hash,recovery_nonce,recovery_ciphertext) VALUES('{id}',{},{},{});",
            blob(token.digest().as_bytes(),postgres),blob(protected.nonce(),postgres),blob(protected.ciphertext(),postgres)));
    }
    sql.push_str("COMMIT;");
    Ok(sql)
}
async fn exercise(store: &Store, credentials: &[BearerSecret]) -> Result<(), Error> {
    let policy = ComponentKeyPolicy {
        validity_ms: 900_000,
        proactive_after_ms: 300_000,
        overlap_ms: 300_000,
    };
    for (index, id) in ["server-provider", "user-provider", "team-provider"]
        .iter()
        .enumerate()
    {
        let draft = ComponentKeyDraft {
            request_id: "1000:exchange-one".into(),
            token: BearerSecret::generate()?,
        };
        let digest = credentials[index].digest();
        let key = store
            .rotate_provider_key(registered(&digest), id, &draft, policy, 1000)
            .await?;
        assert_eq!(key.key.expires_at, 901_000);
        assert_eq!(key.key.proactive_after, 301_000);
        assert_ne!(key.key.token.expose(), credentials[index].expose());
        let retry = ComponentKeyDraft {
            request_id: "1000:exchange-one".into(),
            token: BearerSecret::generate()?,
        };
        let recovered = store
            .rotate_provider_key(registered(&digest), id, &retry, policy, 2000)
            .await?;
        assert_eq!(key.provider_id, recovered.provider_id);
        assert_eq!(key.key.token.expose(), recovered.key.token.expose());
        assert_eq!(key.key.expires_at, recovered.key.expires_at);
        let foreign = credentials[(index + 1) % credentials.len()].digest();
        assert!(
            store
                .rotate_provider_key(registered(&foreign), id, &retry, policy, 2000)
                .await
                .is_err()
        );
        assert!(
            store
                .rotate_provider_key(
                    registered(&key.key.token.digest()),
                    id,
                    &retry,
                    policy,
                    2000
                )
                .await
                .is_err()
        );
    }
    Ok(())
}
#[test]
fn sqlite_server_user_team_exchange_and_revocation() -> Result<(), Error> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("registration.sqlite");
    let cipher = Arc::new(CredentialCipher::new(RootKey::parse(
        URL_SAFE.encode([21; 32]),
    )?)?);
    let credentials = (0..3)
        .map(|_| BearerSecret::generate())
        .collect::<Result<Vec<_>, _>>()?;
    let mut connection = diesel::SqliteConnection::establish(path.to_str().ok_or("Invalid path")?)?;
    connection.batch_execute("PRAGMA foreign_keys=ON")?;
    legacy::sqlite(&mut connection, &cipher)?;
    connection.batch_execute(&seed(&cipher, &credentials, false)?)?;
    chapchoo_persistence::prepare_sqlite(&path, &cipher)?;
    tokio::runtime::Runtime::new()?.block_on(async {
        let store=Store::connect_sqlite(path,cipher).await?;
        exercise(&store,&credentials).await?;
        retention(&store,&credentials[0],|count|check_counts!(&mut connection,count)).await?;
        connection.batch_execute("UPDATE provider_registration_credentials SET revoked_at=2000 WHERE provider_id=(SELECT id FROM computer_providers WHERE registration_key='server-provider'); UPDATE teams SET deletion_requested_at=2000 WHERE handle='auth-team'")?;
        for (id,index) in [("server-provider",0),("team-provider",2)] {
            assert!(store.rotate_provider_key(registered(&credentials[index].digest()),id,&ComponentKeyDraft{request_id:"3000:denied".into(),token:BearerSecret::generate()?},ComponentKeyPolicy::default(),3000).await.is_err());
        }
        Ok::<_,Error>(())
    })
}
#[test]
fn postgres_server_user_team_exchange_and_revocation() -> Result<(), Error> {
    let base = std::env::var("CHAPCHOO_TEST_POSTGRES_URL")?;
    let mut admin = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&base)?;
    let marker = BearerSecret::generate()?;
    let suffix = marker.digest().as_bytes()[..8]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let schema = format!("provider_registration_{suffix}");
    admin.batch_execute(&format!("CREATE SCHEMA {schema}"))?;
    let separator = if base.contains('?') { '&' } else { '?' };
    let url = format!("{base}{separator}options=-csearch_path%3D{schema}");
    let result = (|| {
        let cipher = Arc::new(CredentialCipher::new(RootKey::parse(
            URL_SAFE.encode([21; 32]),
        )?)?);
        let credentials = (0..3)
            .map(|_| BearerSecret::generate())
            .collect::<Result<Vec<_>, _>>()?;
        let mut connection = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&url)?;
        legacy::postgres(&mut connection, &cipher)?;
        connection.batch_execute(&seed(&cipher, &credentials, true)?)?;
        chapchoo_persistence::prepare_postgres(&url, &cipher)?;
        tokio::runtime::Runtime::new()?.block_on(async {
            let store=Store::connect_postgres(url,cipher).await?;
            exercise(&store,&credentials).await?;
            retention(&store,&credentials[0],|count|check_counts!(&mut connection,count)).await?;
            tokio::task::block_in_place(||connection.batch_execute("UPDATE provider_registration_credentials SET revoked_at=2000 WHERE provider_id=(SELECT id FROM computer_providers WHERE registration_key='server-provider'); UPDATE teams SET deletion_requested_at=2000 WHERE handle='auth-team'"))?;
            for (id,index) in [("server-provider",0),("team-provider",2)] {
                assert!(store.rotate_provider_key(registered(&credentials[index].digest()),id,&ComponentKeyDraft{request_id:"3000:denied".into(),token:BearerSecret::generate()?},ComponentKeyPolicy::default(),3000).await.is_err());
            }
            Ok::<_,Error>(())
        })
    })();
    let cleanup = admin.batch_execute(&format!("DROP SCHEMA {schema} CASCADE"));
    support::combine(result, cleanup.map_err(Into::into)).map(|_| ())
}
