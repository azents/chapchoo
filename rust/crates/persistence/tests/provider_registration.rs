//! Scoped later-registration capability, without adding an onboarding API.
mod support;
use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::{BearerSecret, CredentialCipher, RootKey};
use chapchoo_persistence::{
    ComponentKeyDraft, ComponentKeyPolicy, ProviderOwner, ProviderRegistrationOwner, Store,
    provider_registration_context,
};
use diesel::{Connection, connection::SimpleConnection};
use diesel_async::{AsyncPgConnection, async_connection_wrapper::AsyncConnectionWrapper};
use std::sync::Arc;
type Error = Box<dyn std::error::Error + Send + Sync>;
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
            request_id: "exchange-one".into(),
            token: BearerSecret::generate()?,
        };
        let digest = credentials[index].digest();
        let key = store
            .rotate_provider_key(registered(&digest), id, &draft, policy, 1000)
            .await?;
        assert_eq!(key.expires_at, 901_000);
        assert_eq!(key.proactive_after, 301_000);
        assert_ne!(key.token.expose(), credentials[index].expose());
        let retry = ComponentKeyDraft {
            request_id: "exchange-one".into(),
            token: BearerSecret::generate()?,
        };
        let recovered = store
            .rotate_provider_key(registered(&digest), id, &retry, policy, 2000)
            .await?;
        assert_eq!(key.token.expose(), recovered.token.expose());
        assert_eq!(key.expires_at, recovered.expires_at);
        let foreign = credentials[(index + 1) % credentials.len()].digest();
        assert!(
            store
                .rotate_provider_key(registered(&foreign), id, &retry, policy, 2000)
                .await
                .is_err()
        );
        assert!(
            store
                .rotate_provider_key(registered(&key.token.digest()), id, &retry, policy, 2000)
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
    chapchoo_persistence::prepare_sqlite(&path, &cipher)?;
    let credentials = (0..3)
        .map(|_| BearerSecret::generate())
        .collect::<Result<Vec<_>, _>>()?;
    let mut connection = diesel::SqliteConnection::establish(path.to_str().ok_or("Invalid path")?)?;
    connection.batch_execute("PRAGMA foreign_keys=ON")?;
    connection.batch_execute(&seed(&cipher, &credentials, false)?)?;
    tokio::runtime::Runtime::new()?.block_on(async {
        let store=Store::connect_sqlite(path,cipher).await?;
        exercise(&store,&credentials).await?;
        connection.batch_execute("UPDATE provider_registration_credentials SET revoked_at=2000 WHERE provider_id='server-provider'; UPDATE teams SET deletion_requested_at=2000 WHERE id='auth-team'")?;
        for (id,index) in [("server-provider",0),("team-provider",2)] {
            assert!(store.rotate_provider_key(registered(&credentials[index].digest()),id,&ComponentKeyDraft{request_id:"denied".into(),token:BearerSecret::generate()?},ComponentKeyPolicy::default(),3000).await.is_err());
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
        chapchoo_persistence::prepare_postgres(&url, &cipher)?;
        let credentials = (0..3)
            .map(|_| BearerSecret::generate())
            .collect::<Result<Vec<_>, _>>()?;
        let mut connection = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&url)?;
        connection.batch_execute(&seed(&cipher, &credentials, true)?)?;
        tokio::runtime::Runtime::new()?.block_on(async {
            let store=Store::connect_postgres(url,cipher).await?;
            exercise(&store,&credentials).await?;
            tokio::task::block_in_place(||connection.batch_execute("UPDATE provider_registration_credentials SET revoked_at=2000 WHERE provider_id='server-provider'; UPDATE teams SET deletion_requested_at=2000 WHERE id='auth-team'"))?;
            for (id,index) in [("server-provider",0),("team-provider",2)] {
                assert!(store.rotate_provider_key(registered(&credentials[index].digest()),id,&ComponentKeyDraft{request_id:"denied".into(),token:BearerSecret::generate()?},ComponentKeyPolicy::default(),3000).await.is_err());
            }
            Ok::<_,Error>(())
        })
    })();
    let cleanup = admin.batch_execute(&format!("DROP SCHEMA {schema} CASCADE"));
    support::combine(result, cleanup.map_err(Into::into)).map(|_| ())
}
