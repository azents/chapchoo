//! Preserving Provider authentication cutover and canonical Team ownership.
mod support;

use chapchoo_crypto::BearerSecret;
use chapchoo_persistence::{POSTGRES_MIGRATIONS, SQLITE_MIGRATIONS};
use diesel::{
    Connection, RunQueryDsl,
    connection::SimpleConnection,
    sql_types::{BigInt, Text},
};
use diesel_async::{AsyncPgConnection, async_connection_wrapper::AsyncConnectionWrapper};
use diesel_migrations::MigrationHarness;

type Error = Box<dyn std::error::Error + Send + Sync>;
#[derive(diesel::QueryableByName)]
struct Value {
    #[diesel(sql_type = Text)]
    value: String,
}
#[derive(diesel::QueryableByName)]
struct Count {
    #[diesel(sql_type = BigInt)]
    n: i64,
}

const RETAINED: [&str; 14] = [
    "computer_providers",
    "provider_key_requests",
    "computers",
    "agents",
    "computer_executions",
    "computer_resources",
    "connector_sessions",
    "connector_key_requests",
    "connector_connections",
    "users",
    "teams",
    "control_instances",
    "agent_creation_receipts",
    "installation_key_binding",
];

fn blob(postgres: bool, byte: u8, size: usize) -> String {
    let hex = format!("{byte:02x}").repeat(size);
    if postgres {
        format!("decode('{hex}','hex')")
    } else {
        format!("X'{hex}'")
    }
}
fn fixture(postgres: bool) -> String {
    let mut sql = include_str!("fixtures/native_migration.sql").to_owned();
    for i in 1..=7 {
        sql = sql.replace(&format!("{{hash{i}}}"), &blob(postgres, i, 32));
    }
    for i in 1..=2 {
        sql = sql.replace(&format!("{{nonce{i}}}"), &blob(postgres, i, 24));
        sql = sql.replace(&format!("{{cipher{i}}}"), &blob(postgres, i, 41));
    }
    format!("BEGIN;{sql}COMMIT;")
}
fn query(table: &str, columns: &[String], postgres: bool) -> String {
    if postgres {
        format!(
            "SELECT row_to_json(record)::text AS value FROM (SELECT {} FROM {table}) record",
            columns.join(",")
        )
    } else {
        let columns = columns.iter().map(|c| format!("'{c}',json_array(typeof({c}),CASE WHEN typeof({c})='blob' THEN hex({c}) ELSE {c} END)")).collect::<Vec<_>>().join(",");
        format!("SELECT json_object({columns}) AS value FROM {table}")
    }
}
macro_rules! values {
    ($connection:expr,$query:expr) => {{
        let mut values = diesel::sql_query($query)
            .load::<Value>($connection)?
            .into_iter()
            .map(|r| r.value)
            .collect::<Vec<_>>();
        values.sort();
        values
    }};
}
macro_rules! count {
    ($connection:expr,$query:expr) => {{
        diesel::sql_query($query)
            .get_result::<Count>($connection)?
            .n
    }};
}

macro_rules! exercise { ($connection:expr,$source:expr,$postgres:expr) => {{
    let connection = $connection;
    let postgres = $postgres;
    let pending = connection.pending_migrations($source)?;
    assert_eq!(pending.len(),7);
    connection.run_migrations(&pending[..5])?;
    connection.batch_execute(&fixture(postgres))?;
    connection.run_migration(&*pending[5])?;
    connection.batch_execute(&format!(
        "BEGIN;
        INSERT INTO computer_providers(id,owner_kind,user_owner_id,name,provider_kind,created_at) VALUES('user-provider','USER','native-owner','User original','KUBERNETES',1);
        INSERT INTO computer_providers(id,owner_kind,system_owner_id,name,provider_kind,docker_daemon_id,created_at) VALUES('docker-provider','SYSTEM','native-system','Docker original','DOCKER','daemon:original',1);
        INSERT INTO computers(id,owner_user_id,kind,provider_id,desired_state,current_request_id,current_payload_hash,current_action,current_phase,created_at) VALUES('docker-computer','native-owner','CLOUD','docker-provider','RUNNING','create',{},'CREATE','ACCEPTED',1);
        INSERT INTO computer_resources(computer_id,provider_id,resource_kind,native_scope_kind,native_scope,name,docker_volume_metadata,presence) VALUES('docker-computer','docker-provider','DOCKER_VOLUME','DOCKER_DAEMON','daemon:original','saved','{{\"driver\":\"local\",\"scope\":\"local\",\"options\":{{}},\"created_at\":null,\"labels\":{{}}}}','PRESENT');
        COMMIT;", blob(postgres,9,32)))?;
    let mut snapshots = Vec::new();
    for table in RETAINED {
        let columns_query = if postgres { format!("SELECT column_name AS value FROM information_schema.columns WHERE table_schema=current_schema() AND table_name='{table}' ORDER BY ordinal_position") } else { format!("SELECT name AS value FROM pragma_table_xinfo('{table}') ORDER BY cid") };
        let columns = values!(connection, columns_query);
        let data_query = query(table,&columns,postgres);
        let data = values!(connection,&data_query);
        assert!(!data.is_empty(),"Fixture must populate {table}");
        snapshots.push((table,data_query,data));
    }
    let key_columns=["provider_id","current_hash","issued_at","expires_at","current_request_id"].map(str::to_owned);
    let key_query=query("provider_keys",&key_columns,postgres);
    let key_data=values!(connection,&key_query);
    snapshots.push(("provider_keys",key_query,key_data));
    assert_eq!(count!(connection,"SELECT count(*) AS n FROM provider_connections"),1);
    assert_eq!(count!(connection,"SELECT count(*) AS n FROM provider_keys WHERE revoked_at IS NULL AND recovery_nonce IS NOT NULL"),1);
    if !postgres {
        connection.batch_execute("CREATE TABLE unexpected_provider_auth_dependency(id TEXT REFERENCES computer_providers(id))")?;
        assert!(connection.run_migration(&*pending[6]).is_err());
        assert_eq!(connection.applied_migrations()?.len(),6);
        for (table,query,data) in &snapshots { assert_eq!(&values!(connection,query),data,"{table}"); }
        assert_eq!(count!(connection,"SELECT count(*) AS n FROM provider_connections"),1);
        connection.batch_execute("DROP TABLE unexpected_provider_auth_dependency")?;
    }
    connection.run_migration(&*pending[6])?;
    for (table,query,data) in &snapshots { assert_eq!(&values!(connection,query),data,"{table}"); }
    assert_eq!(count!(connection,"SELECT count(*) AS n FROM provider_keys"),2);
    assert_eq!(count!(connection,"SELECT count(*) AS n FROM provider_keys WHERE revoked_at IS NOT NULL AND previous_hash IS NULL AND previous_valid_until IS NULL AND recovery_nonce IS NULL AND recovery_ciphertext IS NULL"),2);
    assert_eq!(count!(connection,"SELECT count(*) AS n FROM provider_connections"),0);
    assert_eq!(count!(connection,"SELECT count(*) AS n FROM provider_registration_credentials"),0);
    connection.batch_execute("INSERT INTO computer_providers(id,owner_kind,owner_team_id,name,provider_kind,created_at) VALUES('team-provider','TEAM','native-team','Team Provider','KUBERNETES',1)")?;
    assert_eq!(count!(connection,"SELECT count(*) AS n FROM computer_providers WHERE id='team-provider' AND owner_team_id='native-team' AND system_owner_id IS NULL AND user_owner_id IS NULL"),1);
    for invalid in [
        "UPDATE computer_providers SET owner_team_id='missing-team' WHERE id='team-provider'",
        "UPDATE computer_providers SET owner_team_id=NULL WHERE id='team-provider'",
        "UPDATE computer_providers SET system_owner_id='server' WHERE id='team-provider'",
        "UPDATE computer_providers SET user_owner_id='native-owner' WHERE id='team-provider'",
        "UPDATE computer_providers SET owner_team_id='native-team' WHERE id='native-provider'",
        "UPDATE computer_providers SET provider_kind='DOCKER' WHERE id='team-provider'",
        "DELETE FROM teams WHERE id='native-team'",
    ] { assert!(connection.batch_execute(invalid).is_err(),"Must reject {invalid}"); }
    connection.batch_execute(&format!("INSERT INTO provider_registration_credentials(provider_id,credential_hash,recovery_nonce,recovery_ciphertext) VALUES('team-provider',{},{},{})",blob(postgres,21,32),blob(postgres,22,24),blob(postgres,23,59)))?;
    for (column,size) in [("credential_hash",31),("recovery_nonce",23),("recovery_ciphertext",58)] {
        assert!(connection.batch_execute(&format!("UPDATE provider_registration_credentials SET {column}={} WHERE provider_id='team-provider'",blob(postgres,24,size))).is_err());
    }
    assert!(connection.revert_last_migration($source).is_err());
    assert_eq!(connection.applied_migrations()?.len(),7);
    if !postgres {
        assert_eq!(count!(connection,"SELECT count(*) AS n FROM pragma_foreign_key_check"),0);
        assert_eq!(count!(connection,"SELECT count(*) AS n FROM sqlite_schema WHERE name GLOB '_auth_new_*'"),0);
        assert_eq!(count!(connection,"SELECT count(*) AS n FROM sqlite_temp_schema WHERE name GLOB '_auth_*'"),0);
    }
}}; }

#[test]
fn sqlite_provider_auth_cutover_preserves_native_and_connector_authority() -> Result<(), Error> {
    let mut connection = diesel::SqliteConnection::establish(":memory:")?;
    connection.batch_execute("PRAGMA foreign_keys=ON")?;
    exercise!(&mut connection, SQLITE_MIGRATIONS, false);
    Ok(())
}
#[test]
fn postgres_provider_auth_cutover_preserves_native_and_connector_authority() -> Result<(), Error> {
    let base = std::env::var("CHAPCHOO_TEST_POSTGRES_URL")?;
    let mut admin = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&base)?;
    let suffix = BearerSecret::generate()?.digest().as_bytes()[..8]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let schema = format!("provider_auth_{suffix}");
    admin.batch_execute(&format!("CREATE SCHEMA {schema}"))?;
    let separator = if base.contains('?') { '&' } else { '?' };
    let url = format!("{base}{separator}options=-csearch_path%3D{schema}");
    let result = (|| {
        let mut connection = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&url)?;
        exercise!(&mut connection, POSTGRES_MIGRATIONS, true);
        Ok::<_, Error>(())
    })();
    let cleanup = admin.batch_execute(&format!("DROP SCHEMA {schema} CASCADE"));
    support::combine(result, cleanup.map_err(Into::into)).map(|_| ())
}

fn team_fixture(
    cipher: &chapchoo_crypto::CredentialCipher,
    token: &BearerSecret,
    postgres: bool,
) -> Result<String, Error> {
    let hex = |bytes: &[u8]| bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();
    let binary = |bytes: &[u8]| {
        let hex = hex(bytes);
        if postgres {
            format!("decode('{hex}','hex')")
        } else {
            format!("X'{hex}'")
        }
    };
    let recovery = cipher.protect_latest("owner-session", token)?;
    let mut sql = format!(
        "BEGIN; INSERT INTO users(id,email_key,delivery_email,password_hash,created_at,language,time_zone,region) VALUES('team-owner','team@example.invalid','team@example.invalid','synthetic',0,'en','UTC','en-US'); INSERT INTO auth_sessions(id,user_id,current_hash,recovery_nonce,recovery_ciphertext,created_at,rotated_at,expires_at) VALUES('owner-session','team-owner',{},{},{},0,0,1000000);",
        binary(token.digest().as_bytes()),
        binary(recovery.nonce()),
        binary(recovery.ciphertext())
    );
    for (index, team) in ["empty-team", "retained-team"].iter().enumerate() {
        let index = u8::try_from(index)?;
        sql.push_str(&format!("INSERT INTO teams(id,kind,name,handle,owner_user_id,created_at) VALUES('{team}','COLLABORATIVE','Team','{team}','team-owner',0); INSERT INTO memberships VALUES('{team}','team-owner','MANAGER'); INSERT INTO computer_providers(id,owner_kind,owner_team_id,name,provider_kind,created_at) VALUES('{team}-provider','TEAM','{team}','Provider','KUBERNETES',0); INSERT INTO provider_registration_credentials(provider_id,credential_hash,recovery_nonce,recovery_ciphertext) VALUES('{team}-provider',{},{},{}); INSERT INTO provider_keys(provider_id,current_hash,issued_at,expires_at,current_request_id,recovery_nonce,recovery_ciphertext) VALUES('{team}-provider',{},1,1000000,'key',{},{});",blob(postgres,40+index,32),blob(postgres,42,24),blob(postgres,43,59),blob(postgres,50+index,32),blob(postgres,52,24),blob(postgres,53,59)));
    }
    sql.push_str(&format!("INSERT INTO computers(id,owner_user_id,kind,provider_id,desired_state,current_request_id,current_payload_hash,current_action,current_phase,created_at) VALUES('retained-computer','team-owner','CLOUD','retained-team-provider','DELETING','delete',{},'DELETE','DELETING',0); INSERT INTO control_instances(id,endpoint,renewed_at,expires_at) VALUES('control','http://localhost:1',1,1000000); INSERT INTO provider_connections(provider_id,control_id,connection_id,credential_hash,expires_at) VALUES('retained-team-provider','control','connection',{},1000000);COMMIT;",blob(postgres,60,32),blob(postgres,51,32)));
    Ok(sql)
}
macro_rules! team_deletion { ($connection:expr,$store:expr,$token:expr) => {{
    let connection=$connection;
    let store=$store;
    store.delete_team(&$token.digest(),"empty-team",100).await?;
    tokio::task::block_in_place(|| -> Result<(),Error> {
        assert_eq!(count!(connection,"SELECT count(*) AS n FROM teams WHERE id='empty-team'"),0);
        assert_eq!(count!(connection,"SELECT count(*) AS n FROM computer_providers WHERE id='empty-team-provider'"),0);
        assert_eq!(count!(connection,"SELECT count(*) AS n FROM provider_registration_credentials WHERE provider_id='empty-team-provider'"),0);
        Ok(())
    })?;
    store.delete_team(&$token.digest(),"retained-team",100).await?;
    assert!(matches!(store.team(&$token.digest(),"retained-team",100).await,Err(chapchoo_persistence::StoreError::NotFound)));
    tokio::task::block_in_place(|| -> Result<(),Error> {
        assert_eq!(count!(connection,"SELECT count(*) AS n FROM teams WHERE id='retained-team' AND deletion_requested_at=100"),1);
        assert_eq!(count!(connection,"SELECT count(*) AS n FROM computer_providers WHERE id='retained-team-provider' AND owner_team_id='retained-team'"),1);
        assert_eq!(count!(connection,"SELECT count(*) AS n FROM provider_registration_credentials WHERE provider_id='retained-team-provider' AND revoked_at=100"),1);
        assert_eq!(count!(connection,"SELECT count(*) AS n FROM provider_keys WHERE provider_id='retained-team-provider' AND revoked_at=100 AND recovery_nonce IS NULL"),1);
        assert_eq!(count!(connection,"SELECT count(*) AS n FROM provider_connections WHERE provider_id='retained-team-provider'"),0);
        // This synthetic Computer has no native resources. Real cleanup remains
        // independently owned; collection never deletes a referenced Computer.
        connection.batch_execute("DELETE FROM computers WHERE id='retained-computer'")?;
        Ok(())
    })?;
    store.delete_team(&$token.digest(),"retained-team",101).await?;
    tokio::task::block_in_place(|| -> Result<(),Error> {
        assert_eq!(count!(connection,"SELECT count(*) AS n FROM teams WHERE id='retained-team'"),0);
        assert_eq!(count!(connection,"SELECT count(*) AS n FROM computer_providers WHERE id='retained-team-provider'"),0);
        Ok(())
    })?;
}}; }

#[test]
fn sqlite_team_provider_deletion_revokes_and_preserves_referenced_identity() -> Result<(), Error> {
    use base64::{Engine, engine::general_purpose::URL_SAFE};
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("team-provider.sqlite");
    let cipher = std::sync::Arc::new(chapchoo_crypto::CredentialCipher::new(
        chapchoo_crypto::RootKey::parse(URL_SAFE.encode([1; 32]))?,
    )?);
    chapchoo_persistence::prepare_sqlite(&path, &cipher)?;
    let token = BearerSecret::generate()?;
    let mut connection = diesel::SqliteConnection::establish(path.to_str().ok_or("Invalid path")?)?;
    connection.batch_execute("PRAGMA foreign_keys=ON")?;
    connection.batch_execute(&team_fixture(&cipher, &token, false)?)?;
    tokio::runtime::Runtime::new()?.block_on(async {
        let store = chapchoo_persistence::Store::connect_sqlite(path, cipher).await?;
        team_deletion!(&mut connection, store, token);
        Ok::<_, Error>(())
    })
}
#[test]
fn postgres_team_provider_deletion_revokes_and_preserves_referenced_identity() -> Result<(), Error>
{
    use base64::{Engine, engine::general_purpose::URL_SAFE};
    let base = std::env::var("CHAPCHOO_TEST_POSTGRES_URL")?;
    let mut admin = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&base)?;
    let token = BearerSecret::generate()?;
    let suffix = token.digest().as_bytes()[..8]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let schema = format!("team_provider_{suffix}");
    admin.batch_execute(&format!("CREATE SCHEMA {schema}"))?;
    let separator = if base.contains('?') { '&' } else { '?' };
    let url = format!("{base}{separator}options=-csearch_path%3D{schema}");
    let result = (|| {
        let cipher = std::sync::Arc::new(chapchoo_crypto::CredentialCipher::new(
            chapchoo_crypto::RootKey::parse(URL_SAFE.encode([1; 32]))?,
        )?);
        chapchoo_persistence::prepare_postgres(&url, &cipher)?;
        let mut connection = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&url)?;
        connection.batch_execute(&team_fixture(&cipher, &token, true)?)?;
        tokio::runtime::Runtime::new()?.block_on(async {
            let store = chapchoo_persistence::Store::connect_postgres(url, cipher).await?;
            team_deletion!(&mut connection, store, token);
            Ok::<_, Error>(())
        })
    })();
    let cleanup = admin.batch_execute(&format!("DROP SCHEMA {schema} CASCADE"));
    support::combine(result, cleanup.map_err(Into::into)).map(|_| ())
}
