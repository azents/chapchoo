use super::*;
use crate::{
    entity_id::IdentitySql,
    preparation::{POSTGRES_MIGRATIONS, SQLITE_MIGRATIONS},
};
use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::{BearerSecret, RecoveryEnvelope, RootKey, SecretDigest};
use diesel::{Connection, RunQueryDsl, connection::SimpleConnection, sql_types::BigInt};
use diesel_async::{AsyncPgConnection, async_connection_wrapper::AsyncConnectionWrapper};
use diesel_migrations::MigrationHarness;

type TestError = Box<dyn std::error::Error + Send + Sync>;

async fn recover_historical_results(
    store: crate::Store,
    credential: &CredentialFixture,
) -> Result<(), TestError> {
    use crate::AgentCreationResult;
    let actor = credential.token.digest();
    let present = store
        .agent_creation_result(&actor, "create-request", 10)
        .await?
        .ok_or("Missing accepted creation result")?;
    let AgentCreationResult::Present {
        assigned_agent_id,
        team_id,
    } = present
    else {
        return Err("Existing Agent was reported removed".into());
    };
    EntityId::from_dto(EntityKind::Agent, &assigned_agent_id)?;
    EntityId::from_dto(EntityKind::Team, &team_id)?;
    let removed = store
        .agent_creation_result(&actor, "removed-request", 10)
        .await?
        .ok_or("Missing removed creation result")?;
    let AgentCreationResult::Removed {
        assigned_agent_id: removed,
    } = removed
    else {
        return Err("Removed Agent was reported present".into());
    };
    assert_ne!(assigned_agent_id, removed);
    EntityId::from_dto(EntityKind::Agent, &removed)?;
    assert!(
        store
            .agent_creation_result(&actor, "never-accepted", 10)
            .await?
            .is_none()
    );
    assert!(matches!(
        store
            .agent_creation_result(&BearerSecret::generate()?.digest(), "create-request", 10)
            .await,
        Err(StoreError::Unauthenticated)
    ));
    Ok(())
}

fn cipher() -> Result<CredentialCipher, TestError> {
    Ok(CredentialCipher::new(RootKey::parse(
        URL_SAFE.encode([71; 32]),
    )?)?)
}
fn blob(bytes: &[u8], postgres: bool) -> String {
    let hex = bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    if postgres {
        format!("decode('{hex}','hex')")
    } else {
        format!("X'{hex}'")
    }
}

struct CredentialFixture {
    token: BearerSecret,
    original: String,
    kind: EntityKind,
    namespace: &'static str,
    table: &'static str,
    id_column: &'static str,
    hash: &'static str,
    nonce: &'static str,
    ciphertext: &'static str,
}

fn credential_fixtures() -> Result<Vec<CredentialFixture>, TestError> {
    [
        (
            "account-session",
            EntityKind::AuthSession,
            "",
            "auth_sessions",
            "id",
            "current_hash",
            "recovery_nonce",
            "recovery_ciphertext",
        ),
        (
            "native-provider",
            EntityKind::Provider,
            "computer-provider:",
            "provider_keys",
            "provider_id",
            "current_hash",
            "recovery_nonce",
            "recovery_ciphertext",
        ),
        (
            "native-session",
            EntityKind::ConnectorSession,
            "computer-connector-bootstrap:",
            "connector_sessions",
            "id",
            "initial_hash",
            "initial_nonce",
            "initial_ciphertext",
        ),
        (
            "native-session",
            EntityKind::ConnectorSession,
            "computer-connector-operational:",
            "connector_sessions",
            "id",
            "current_hash",
            "recovery_nonce",
            "recovery_ciphertext",
        ),
    ]
    .into_iter()
    .map(
        |(id, kind, namespace, table, id_column, hash, nonce, ciphertext)| {
            Ok(CredentialFixture {
                token: BearerSecret::generate()?,
                original: id.to_owned(),
                kind,
                namespace,
                table,
                id_column,
                hash,
                nonce,
                ciphertext,
            })
        },
    )
    .collect()
}

fn fixture_sql(
    postgres: bool,
    cipher: &CredentialCipher,
    credentials: &[CredentialFixture],
) -> Result<String, TestError> {
    let hash = blob(&[1; 32], postgres);
    let nonce = blob(&[2; 24], postgres);
    let ciphertext = blob(&[3; 59], postgres);
    let previous = blob(&[4; 32], postgres);
    let binding = blob(cipher.installation_check()?.as_bytes(), postgres);
    let mut sql = format!("INSERT INTO installation_key_binding VALUES(1,{binding});
      INSERT INTO display_defaults VALUES(1,'en','UTC','en-US');
      INSERT INTO users(id,email_key,delivery_email,password_hash,is_admin,created_at,language,time_zone,region) VALUES('owner','fixture@example.invalid','fixture@example.invalid','synthetic',FALSE,0,'en','UTC','en-US');
      INSERT INTO teams(id,kind,name,handle,owner_user_id,created_at) VALUES('personal','PERSONAL','Personal','personal-old','owner',0),('a-transformed','COLLABORATIVE','Transformed','DuBu','owner',0),('z-canonical','COLLABORATIVE','Canonical','dubu','owner',0);
      INSERT INTO memberships VALUES('personal','owner','MANAGER'),('a-transformed','owner','MANAGER'),('z-canonical','owner','MANAGER');
      INSERT INTO personal_contexts VALUES('owner','personal');
      INSERT INTO grants(id,kind,secret_hash,email_key,delivery_email,user_id,issued_by,issued_at,expires_at) VALUES('grant','PASSWORD_RESET',{hash},'fixture@example.invalid','fixture@example.invalid','owner','owner',0,100);
      INSERT INTO invitations VALUES('invitation','a-transformed','other@example.invalid','other@example.invalid','owner','PENDING',1,NULL);
      INSERT INTO login_challenges VALUES('challenge','fixture@example.invalid',{hash},{previous},0,100,NULL);
      INSERT INTO auth_sessions(id,user_id,current_hash,previous_hash,recovery_nonce,recovery_ciphertext,created_at,rotated_at,expires_at,absolute_expires_at,previous_valid_until) VALUES('account-session','owner',{hash},{previous},{nonce},{ciphertext},0,1,100,200,20);
      INSERT INTO computer_providers(id,owner_kind,system_owner_id,name,provider_kind,can_create,created_at) VALUES('native-provider','SYSTEM','original-system','Provider','KUBERNETES',TRUE,0);
      INSERT INTO computer_providers(id,owner_kind,user_owner_id,name,provider_kind,created_at) VALUES('user-provider','USER','owner','User provider','KUBERNETES',0);
      INSERT INTO computer_providers(id,owner_kind,owner_team_id,name,provider_kind,created_at) VALUES('team-provider','TEAM','a-transformed','Team provider','KUBERNETES',0);
      INSERT INTO provider_keys(provider_id,current_hash,previous_hash,previous_valid_until,issued_at,expires_at,current_request_id,recovery_nonce,recovery_ciphertext) VALUES('native-provider',{hash},{previous},20,1,100,'provider-key-request',{nonce},{ciphertext});
      INSERT INTO provider_key_requests VALUES('native-provider','provider-key-request',{hash},{previous},300);
      INSERT INTO computers(id,owner_user_id,kind,provider_id,desired_state,current_request_id,current_payload_hash,current_action,current_phase,current_execution_id,expected_old_execution_id,power_source_connection_id,power_source_execution_id,environment_source_connection_id,created_at,prepare_admitted_at,saved_environment) VALUES('native-computer','owner','CLOUD','native-provider','RUNNING','create-request',{hash},'CREATE','COMPLETED','native-execution','retired-execution','old-provider-route','native-execution','old-provider-route',1,2,'VERIFIED');
      INSERT INTO agents(id,owner_user_id,team_id,name,computer_id,computer_kind,created_at) VALUES('old-agent','owner','personal','두부','native-computer','CLOUD',1);
      INSERT INTO agent_creation_receipts(owner_user_id,team_id,request_id,payload_hash,assigned_agent_id) VALUES('owner','personal','create-request',{hash},'old-agent'),('owner','deleted-team','removed-request',{previous},'removed-agent');
      INSERT INTO computer_executions VALUES('native-execution','native-computer',TRUE,3,NULL,2),('retired-execution','native-computer',FALSE,1,2,1);
      INSERT INTO computer_resources(computer_id,provider_id,resource_kind,native_scope,name,resource_uid,execution_id,presence,observed_at,source_connection_id) VALUES('native-computer','native-provider','POD','original-namespace','original-pod','original-uid','native-execution','PRESENT',4,'old-provider-route'),('native-computer','native-provider','PVC','original-namespace','original-volume','volume-uid',NULL,'PRESENT',4,'old-provider-route');
      INSERT INTO connector_sessions(id,computer_id,execution_id,initial_request_id,initial_payload_hash,initial_hash,initial_nonce,initial_ciphertext,current_hash,previous_hash,previous_valid_until,issued_at,expires_at,current_request_id,recovery_nonce,recovery_ciphertext,created_at) VALUES('native-session','native-computer','native-execution','bootstrap-request',{hash},{previous},{nonce},{ciphertext},{hash},{previous},20,2,100,'connector-request',{nonce},{ciphertext},1);
      INSERT INTO connector_key_requests VALUES('native-session','connector-request','EXCHANGE',{hash},{previous});
      INSERT INTO control_instances VALUES('control','http://control.invalid',FALSE,1,100);
      INSERT INTO provider_connections VALUES('native-provider','control','old-provider-route',{hash},100);
      INSERT INTO connector_connections VALUES('native-computer','native-session','control','old-connector-route',{hash},100);");
    for credential in credentials {
        let context = format!("{}{}", credential.namespace, credential.original);
        let envelope = cipher.protect_latest(&context, &credential.token)?;
        let hash = blob(credential.token.digest().as_bytes(), postgres);
        let nonce = blob(envelope.nonce(), postgres);
        let ciphertext = blob(envelope.ciphertext(), postgres);
        sql.push_str(&format!(
            "UPDATE {} SET {}={hash},{}={nonce},{}={ciphertext} WHERE {}='{}';",
            credential.table,
            credential.hash,
            credential.nonce,
            credential.ciphertext,
            credential.id_column,
            credential.original
        ));
    }
    // Exercise all owner context variants; operator registration keys stay
    // separate from the canonical Provider ID after conversion.
    for (id, owner) in [
        (
            "native-provider",
            crate::ProviderRegistrationOwner::System("original-system"),
        ),
        (
            "user-provider",
            crate::ProviderRegistrationOwner::User("owner"),
        ),
        (
            "team-provider",
            crate::ProviderRegistrationOwner::Team("a-transformed"),
        ),
    ] {
        let token = BearerSecret::generate()?;
        let context = crate::provider_registration_context(id, owner)?;
        let envelope = cipher.protect_latest(&context, &token)?;
        sql.push_str(&format!(
            "INSERT INTO provider_registration_credentials VALUES('{id}',{},{},{},NULL);",
            blob(token.digest().as_bytes(), postgres),
            blob(envelope.nonce(), postgres),
            blob(envelope.ciphertext(), postgres)
        ));
    }
    Ok(sql)
}

#[derive(diesel::QueryableByName)]
struct Count {
    #[diesel(sql_type=BigInt)]
    n: i64,
}
#[derive(diesel::QueryableByName)]
struct Identity {
    #[diesel(sql_type=IdentitySql)]
    id: EntityId,
}
#[derive(diesel::QueryableByName)]
struct Profile {
    #[diesel(sql_type=IdentitySql)]
    id: EntityId,
    #[diesel(sql_type=Text)]
    handle: String,
    #[diesel(sql_type=Text)]
    palette_key: String,
    #[diesel(sql_type=Text)]
    internal_name: String,
}
#[derive(diesel::QueryableByName)]
struct StoredEnvelope {
    #[diesel(sql_type=IdentitySql)]
    id: EntityId,
    #[diesel(sql_type=Binary)]
    digest: Vec<u8>,
    #[diesel(sql_type=Binary)]
    nonce: Vec<u8>,
    #[diesel(sql_type=Binary)]
    ciphertext: Vec<u8>,
}

#[derive(diesel::QueryableByName)]
struct SnapshotValue {
    #[diesel(sql_type=Text)]
    value: String,
}

macro_rules! postgres_snapshot {
    ($connection:expr) => {{
        let mut snapshots = Vec::new();
        for columns in crate::identity_transition::IDENTITY_COLUMNS {
            let table = columns[0].table;
            let rows = diesel::sql_query(format!(
                "SELECT row_to_json(original)::text AS value FROM {table} original",
            ))
            .load::<SnapshotValue>($connection)?;
            let mut values = rows.into_iter().map(|row| row.value).collect::<Vec<_>>();
            values.sort();
            snapshots.push((table, values));
        }
        snapshots
    }};
}

macro_rules! setup {
    ($connection:expr,$source:expr,$postgres:expr,$cipher:expr,$credentials:expr) => {{
        let connection = &mut *$connection;
        let pending = connection.pending_migrations($source)?;
        assert!(pending.len() >= 9);
        connection.run_migrations(&pending[..8])?;
        connection.transaction::<_, TestError, _>(|connection| {
            connection
                .batch_execute(&fixture_sql($postgres, $cipher, $credentials)?)
                .map_err(Into::into)
        })?;
    }};
}
macro_rules! verify {
    ($connection:expr,$postgres:expr,$cipher:expr,$credentials:expr) => {{
        let connection = &mut *$connection;
        assert_eq!(connection.applied_migrations()?.len(),10);
        let source = || if $postgres { POSTGRES_MIGRATIONS } else { SQLITE_MIGRATIONS };
        assert!(connection.transaction::<_, TestError, _>(|connection| {
            connection.revert_last_migration(source())?;
            connection.revert_last_migration(source())?;
            Ok(())
        }).is_err());
        assert_eq!(connection.applied_migrations()?.len(),10);
        let query = "SELECT a.id,a.handle,a.palette_key,i.internal_name FROM agents a JOIN profile_images i ON i.id=a.profile_image_id";
        let profiles = diesel::sql_query(query).load::<Profile>(connection)?;
        assert_eq!(profiles.len(),1);
        assert_eq!(profiles[0].handle,"dubu");
        assert_eq!(profiles[0].palette_key,"lilac");
        assert_eq!(profiles[0].internal_name,"border-collie");
        let canonical = profiles[0].id.to_dto(EntityKind::Agent);
        assert_eq!(EntityId::from_dto(EntityKind::Agent,&canonical)?,profiles[0].id);
        let q="SELECT count(*) AS n FROM computer_resources r JOIN computers c ON c.id=r.computer_id JOIN computer_providers p ON p.id=r.provider_id WHERE c.id=r.computer_id AND p.id=c.provider_id AND r.allocation_computer_id='native-computer' AND r.allocation_provider_id='native-provider' AND ((r.resource_kind='POD' AND r.allocation_execution_id='native-execution' AND r.name='original-pod' AND r.resource_uid='original-uid') OR (r.resource_kind='PVC' AND r.allocation_execution_id IS NULL AND r.name='original-volume' AND r.resource_uid='volume-uid'))";
        assert_eq!(diesel::sql_query(q).get_result::<Count>(connection)?.n,2);
        assert_eq!(diesel::sql_query("SELECT count(*) AS n FROM teams WHERE (kind='PERSONAL' AND handle IS NULL) OR (name='Canonical' AND handle='dubu') OR (name='Transformed' AND handle LIKE 'dubu-%')").get_result::<Count>(connection)?.n,3);
        assert_eq!(diesel::sql_query("SELECT count(*) AS n FROM personal_contexts p JOIN teams t ON t.id=p.team_id JOIN memberships m ON m.team_id=t.id AND m.user_id=p.user_id WHERE t.owner_user_id=m.user_id").get_result::<Count>(connection)?.n,1);
        assert_eq!(diesel::sql_query("SELECT count(*) AS n FROM agent_creation_receipts r LEFT JOIN agents a ON a.id=r.assigned_agent_id WHERE a.id IS NULL AND r.request_id='removed-request'").get_result::<Count>(connection)?.n,1);
        assert_eq!(diesel::sql_query("SELECT count(*) AS n FROM profile_images").get_result::<Count>(connection)?.n,6);
        assert_eq!(diesel::sql_query("SELECT count(*) AS n FROM auth_sessions WHERE expires_at=100 AND absolute_expires_at=200 AND previous_valid_until=20 AND revoked_at IS NULL").get_result::<Count>(connection)?.n,1);
        assert_eq!(diesel::sql_query("SELECT count(*) AS n FROM computer_providers WHERE registration_key IN ('native-provider','user-provider','team-provider')").get_result::<Count>(connection)?.n,3);
        for fixture in $credentials {
            let query=format!("SELECT {} AS id,{} AS digest,{} AS nonce,{} AS ciphertext FROM {}",fixture.id_column,fixture.hash,fixture.nonce,fixture.ciphertext,fixture.table);
            let row=diesel::sql_query(query).get_result::<StoredEnvelope>(connection)?;
            let id=row.id.to_dto(fixture.kind);
            let envelope=RecoveryEnvelope::from_parts(row.nonce.as_slice().try_into()?,row.ciphertext)?;
            let digest=SecretDigest::from_bytes(row.digest.as_slice().try_into()?);
            let recovered=$cipher.recover_latest(&format!("{}{id}",fixture.namespace),&digest,&envelope)?;
            assert_eq!(recovered.expose(),fixture.token.expose());
            assert!($cipher.recover_latest(&format!("{}{}",fixture.namespace,fixture.original),&digest,&envelope).is_err());
        }
        let ids=diesel::sql_query("SELECT id FROM computer_providers").load::<Identity>(connection)?;
        assert_eq!(ids.len(),3);
        for identity in ids { assert!(identity.id.to_dto(EntityKind::Provider).starts_with('P')); }
        for invalid in ["", "ab", "Upper", "a_b", "-abc", "abc-", "abc\n", "abcdefghijklmnopq", "한글", "abc\0", "abc\0UPPER", "abc\0/evil"] {
            assert!(diesel::sql_query(if $postgres { "UPDATE agents SET handle=$1" } else { "UPDATE agents SET handle=?" })
                .bind::<Text,_>(invalid).execute(connection).is_err(), "Database accepted invalid handle {invalid:?}");
            assert!(diesel::sql_query(if $postgres {
                "UPDATE teams SET handle=$1 WHERE kind='COLLABORATIVE' AND name='Canonical'"
            } else {
                "UPDATE teams SET handle=? WHERE kind='COLLABORATIVE' AND name='Canonical'"
            }).bind::<Text,_>(invalid).execute(connection).is_err(), "Database accepted invalid Team handle {invalid:?}");
        }
        assert!(connection.batch_execute("UPDATE teams SET handle='personal-alias' WHERE kind='PERSONAL'").is_err());
        assert!(connection.batch_execute("UPDATE agents SET palette_key='custom'").is_err());
        if !$postgres {
            assert_eq!(diesel::sql_query("SELECT count(*) AS n FROM pragma_foreign_key_check").get_result::<Count>(connection)?.n,0);
            assert_eq!(diesel::sql_query("SELECT count(*) AS n FROM sqlite_temp_master WHERE name LIKE '_profile_%'").get_result::<Count>(connection)?.n,0);
            assert_eq!(diesel::sql_query("SELECT count(*) AS n FROM agents WHERE typeof(id)='blob' AND length(id)=16 AND typeof(profile_image_id)='blob' AND length(profile_image_id)=16").get_result::<Count>(connection)?.n,1);
        }
    }};
}

#[test]
fn sqlite_populated_preparation_preserves_credentials_relations_and_native_evidence()
-> Result<(), TestError> {
    let root = cipher()?;
    let credentials = credential_fixtures()?;
    let temp = tempfile::tempdir()?;
    let path = temp.path().join("profile.sqlite");
    let mut connection =
        diesel::SqliteConnection::establish(path.to_str().ok_or("Invalid fixture path")?)?;
    connection.batch_execute("PRAGMA foreign_keys=ON")?;
    setup!(
        &mut connection,
        SQLITE_MIGRATIONS,
        false,
        &root,
        &credentials
    );
    drop(connection);
    crate::prepare_sqlite(&path, &root)?;
    crate::sqlite_ready(&path, &root)?;
    let before = std::fs::read(&path)?;
    crate::prepare_sqlite(&path, &root)?;
    assert_eq!(std::fs::read(&path)?, before);
    let owned_root = std::sync::Arc::new(cipher()?);
    tokio::runtime::Runtime::new()?.block_on(async {
        recover_historical_results(
            crate::Store::connect_sqlite(path.clone(), owned_root).await?,
            &credentials[0],
        )
        .await
    })?;
    let mut connection =
        diesel::SqliteConnection::establish(path.to_str().ok_or("Invalid fixture path")?)?;
    verify!(&mut connection, false, &root, &credentials);
    connection.batch_execute(
        "UPDATE agents SET name='Renamed after upgrade',palette_key='rose',
         profile_image_id=(SELECT id FROM profile_images WHERE internal_name='retriever')",
    )?;
    let extension = EntityId::generate(2000)?;
    diesel::sql_query("INSERT INTO profile_images VALUES(?,'AGENT','installation-extra','BUILTIN','installation-extra-art')")
        .bind::<IdentitySql,_>(extension).execute(&mut connection)?;
    drop(connection);
    let before = std::fs::read(&path)?;
    crate::prepare_sqlite(&path, &root)?;
    assert_eq!(std::fs::read(&path)?, before);
    crate::sqlite_ready(&path, &root)?;
    Ok(())
}

#[test]
fn sqlite_failed_rewrap_rolls_back_and_allows_explicit_retry() -> Result<(), TestError> {
    let root = cipher()?;
    let credentials = credential_fixtures()?;
    let temp = tempfile::tempdir()?;
    let path = temp.path().join("rollback.sqlite");
    let raw = path.to_str().ok_or("Invalid fixture path")?;
    let mut connection = diesel::SqliteConnection::establish(raw)?;
    connection.batch_execute("PRAGMA foreign_keys=ON")?;
    setup!(
        &mut connection,
        SQLITE_MIGRATIONS,
        false,
        &root,
        &credentials
    );
    let original = diesel::sql_query(
        "SELECT provider_id AS id,credential_hash AS digest,recovery_nonce AS nonce,recovery_ciphertext AS ciphertext FROM provider_registration_credentials WHERE provider_id='native-provider'",
    ).get_result::<EnvelopeRow>(&mut connection)?;
    // Fail after the account/Provider/Connector envelopes have all been rewritten
    // in the tentative transaction, not before its first data mutation.
    connection.batch_execute("UPDATE provider_registration_credentials SET recovery_ciphertext=zeroblob(59) WHERE provider_id='native-provider'")?;
    drop(connection);
    let before = std::fs::read(&path)?;
    assert!(crate::prepare_sqlite(&path, &root).is_err());
    assert_eq!(std::fs::read(&path)?, before);
    let mut connection = diesel::SqliteConnection::establish(raw)?;
    assert_eq!(connection.applied_migrations()?.len(), 8);
    diesel::sql_query("UPDATE provider_registration_credentials SET recovery_nonce=?,recovery_ciphertext=? WHERE provider_id='native-provider'")
        .bind::<Binary, _>(original.nonce.as_deref().ok_or("Missing original nonce")?)
        .bind::<Binary, _>(original.ciphertext.as_deref().ok_or("Missing original envelope")?)
        .execute(&mut connection)?;
    connection.batch_execute(
        "CREATE TABLE unexpected_identity_dependency(owner TEXT REFERENCES users(id))",
    )?;
    drop(connection);
    let before = std::fs::read(&path)?;
    assert!(crate::prepare_sqlite(&path, &root).is_err());
    assert_eq!(std::fs::read(&path)?, before);
    let mut connection = diesel::SqliteConnection::establish(raw)?;
    connection.batch_execute("DROP TABLE unexpected_identity_dependency")?;
    connection.batch_execute(
        "CREATE TRIGGER unexpected_identity_guard BEFORE UPDATE ON users
         BEGIN SELECT RAISE(ABORT,'fixture identity guard'); END",
    )?;
    drop(connection);
    let before = std::fs::read(&path)?;
    assert!(crate::prepare_sqlite(&path, &root).is_err());
    assert_eq!(std::fs::read(&path)?, before);
    let mut connection = diesel::SqliteConnection::establish(raw)?;
    assert_eq!(connection.applied_migrations()?.len(), 8);
    assert_eq!(diesel::sql_query("SELECT count(*) AS n FROM sqlite_schema WHERE type='trigger' AND name='unexpected_identity_guard'").get_result::<Count>(&mut connection)?.n,1);
    connection.batch_execute("DROP TRIGGER unexpected_identity_guard")?;
    drop(connection);
    crate::prepare_sqlite(&path, &root)?;
    let mut connection = diesel::SqliteConnection::establish(raw)?;
    verify!(&mut connection, false, &root, &credentials);
    Ok(())
}

#[test]
fn postgres_populated_preparation_preserves_credentials_relations_and_native_evidence()
-> Result<(), TestError> {
    let _fixture_guard = crate::preparation::POSTGRES_PREPARATION_TEST_LOCK
        .lock()
        .map_err(|_| "PostgreSQL preparation test lock poisoned")?;
    let root = cipher()?;
    let credentials = credential_fixtures()?;
    let base = std::env::var("CHAPCHOO_TEST_POSTGRES_URL")?;
    let mut admin = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&base)?;
    let token = BearerSecret::generate()?;
    let suffix = token.digest().as_bytes()[..8]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let schema = format!("profile_transition_{suffix}");
    admin.batch_execute(&format!("CREATE SCHEMA {schema}"))?;
    let separator = if base.contains('?') { '&' } else { '?' };
    let url = format!("{base}{separator}options=-csearch_path%3D{schema}");
    let result = (|| -> Result<(), TestError> {
        let mut connection = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&url)?;
        setup!(
            &mut connection,
            POSTGRES_MIGRATIONS,
            true,
            &root,
            &credentials
        );
        let original = diesel::sql_query(
            "SELECT provider_id AS id,credential_hash AS digest,recovery_nonce AS nonce,recovery_ciphertext AS ciphertext FROM provider_registration_credentials WHERE provider_id='native-provider'",
        ).get_result::<EnvelopeRow>(&mut connection)?;
        connection.batch_execute("UPDATE provider_registration_credentials SET recovery_ciphertext=decode(repeat('00',59),'hex') WHERE provider_id='native-provider'")?;
        let before = postgres_snapshot!(&mut connection);
        drop(connection);
        assert!(crate::prepare_postgres(&url, &root).is_err());
        let mut connection = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&url)?;
        assert_eq!(connection.applied_migrations()?.len(), 8);
        assert_eq!(postgres_snapshot!(&mut connection), before);
        diesel::sql_query("UPDATE provider_registration_credentials SET recovery_nonce=$1,recovery_ciphertext=$2 WHERE provider_id='native-provider'")
            .bind::<Binary, _>(original.nonce.as_deref().ok_or("Missing original nonce")?)
            .bind::<Binary, _>(original.ciphertext.as_deref().ok_or("Missing original envelope")?)
            .execute(&mut connection)?;
        drop(connection);
        crate::prepare_postgres(&url, &root)?;
        crate::postgres_ready(&url, &root)?;
        crate::prepare_postgres(&url, &root)?;
        let owned_root = std::sync::Arc::new(cipher()?);
        tokio::runtime::Runtime::new()?.block_on(async {
            recover_historical_results(
                crate::Store::connect_postgres(url.clone(), owned_root).await?,
                &credentials[0],
            )
            .await
        })?;
        let mut connection = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&url)?;
        verify!(&mut connection, true, &root, &credentials);
        connection.batch_execute(
            "UPDATE agents SET name='Renamed after upgrade',palette_key='rose',
             profile_image_id=(SELECT id FROM profile_images WHERE internal_name='retriever')",
        )?;
        let extension = EntityId::generate(2000)?;
        diesel::sql_query("INSERT INTO profile_images VALUES($1,'AGENT','installation-extra','BUILTIN','installation-extra-art')")
            .bind::<IdentitySql,_>(extension).execute(&mut connection)?;
        let before = postgres_snapshot!(&mut connection);
        let image_ids = diesel::sql_query("SELECT id FROM profile_images ORDER BY id")
            .load::<Identity>(&mut connection)?
            .into_iter()
            .map(|row| row.id)
            .collect::<Vec<_>>();
        drop(connection);
        crate::prepare_postgres(&url, &root)?;
        let mut connection = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&url)?;
        assert_eq!(postgres_snapshot!(&mut connection), before);
        assert_eq!(
            diesel::sql_query("SELECT id FROM profile_images ORDER BY id")
                .load::<Identity>(&mut connection)?
                .into_iter()
                .map(|row| row.id)
                .collect::<Vec<_>>(),
            image_ids
        );
        Ok(())
    })();
    let cleanup = admin.batch_execute(&format!("DROP SCHEMA {schema} CASCADE"));
    match (result, cleanup) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(primary), Ok(())) => Err(primary),
        (Ok(()), Err(cleanup)) => Err(cleanup.into()),
        (Err(primary), Err(cleanup)) => {
            Err(format!("{primary}; fixture cleanup also failed: {cleanup}").into())
        }
    }
}
