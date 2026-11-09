mod support;

use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::{BearerSecret, CredentialCipher, RootKey};
use chapchoo_persistence::{
    POSTGRES_MIGRATIONS, PreparationError, SQLITE_MIGRATIONS, postgres_ready, prepare_postgres,
    prepare_sqlite, sqlite_ready,
};
use diesel::{Connection, RunQueryDsl, connection::SimpleConnection, sql_types::BigInt};
use diesel_async::{AsyncPgConnection, async_connection_wrapper::AsyncConnectionWrapper};
use diesel_migrations::MigrationHarness;
use std::error::Error;

type TestError = Box<dyn Error + Send + Sync>;

fn cipher(byte: u8) -> Result<CredentialCipher, TestError> {
    Ok(CredentialCipher::new(RootKey::parse(
        URL_SAFE.encode([byte; 32]),
    )?)?)
}

fn provider_sql(id: &str, postgres: bool) -> String {
    // Only static fixture identifiers reach this helper.
    compact_fixture_sql(
        &format!(
            "INSERT INTO computer_providers(id,registration_key,owner_kind,system_owner_id,name,provider_kind,created_at) VALUES('{id}','registration-{id}','SYSTEM','owner','Fixture','KUBERNETES',0)"
        ),
        postgres,
    )
}

/// Native SQL fixtures exercise schema constraints; no DTO codec is exposed.
fn compact_fixture_sql(source: &str, postgres: bool) -> String {
    let mut sql = source.to_owned();
    if sql.starts_with("INSERT INTO agents(") {
        let handle = source
            .split("VALUES('")
            .nth(1)
            .and_then(|part| part.split('\'').next())
            .unwrap_or("invalid-fixture");
        let handle = format!("s-{}", handle.chars().take(12).collect::<String>());
        sql = sql.replace(
            "created_at) VALUES(",
            "created_at,handle,profile_image_id,palette_key) VALUES(",
        );
        if let Some(end) = sql.rfind(')') {
            sql.insert_str(
                end,
                &format!(
                    ", '{handle}', (SELECT id FROM profile_images ORDER BY id LIMIT 1), 'sage'"
                ),
            );
        }
    }
    if sql.starts_with("INSERT INTO computer_resources(") {
        sql = sql.replace("presence) VALUES(", "presence,allocation_computer_id,allocation_provider_id,allocation_execution_id) VALUES(");
        let execution = if source.contains("'POD'") {
            "'native-execution'"
        } else {
            "NULL"
        };
        if let Some(end) = sql.rfind(')') {
            sql.insert_str(
                end,
                &format!(", 'native-computer', 'native-provider', {execution}"),
            );
        }
    }
    for user in ["a", "b"] {
        sql = sql.replace(&format!("'Personal','personal-{user}'"), "'Personal',NULL");
    }
    let labels = [
        "provider-a",
        "provider-b",
        "root-guard",
        "user-a",
        "user-b",
        "personal-a",
        "personal-b",
        "cloud",
        "local",
        "cloud-agent",
        "cloud-duplicate",
        "local-one",
        "local-two",
        "foreign-owner",
        "wrong-kind",
        "replacement-agent",
        "execution-a",
        "wrong-session",
        "bound-session",
        "control",
        "foreign-route",
        "owned-route",
    ];
    for (number, label) in (1_u8..).zip(labels) {
        let mut bytes = [0_u8; 16];
        bytes[0] = 1;
        bytes[6] = 0x70;
        bytes[8] = 0x80;
        bytes[15] = number;
        let literal = if postgres {
            format!("'{}'::uuid", uuid::Uuid::from_bytes(bytes))
        } else {
            format!(
                "X'{}'",
                bytes
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>()
            )
        };
        sql = sql.replace(&format!("'{label}'"), &literal);
    }
    sql
}

struct CompactFixture<'a, C> {
    connection: &'a mut C,
    postgres: bool,
}
impl<C: SimpleConnection> SimpleConnection for CompactFixture<'_, C> {
    fn batch_execute(&mut self, query: &str) -> diesel::QueryResult<()> {
        self.connection
            .batch_execute(&compact_fixture_sql(query, self.postgres))
    }
}

fn exercise_constraints(
    connection: &mut impl SimpleConnection,
    postgres: bool,
) -> Result<(), TestError> {
    let connection = &mut CompactFixture {
        connection,
        postgres,
    };
    connection.batch_execute(&provider_sql("provider-a", postgres))?;
    let hash = if postgres {
        "decode(repeat('00',32),'hex')"
    } else {
        "zeroblob(32)"
    };
    for user in ["a", "b"] {
        connection.batch_execute(&format!("INSERT INTO users(id,email_key,delivery_email,password_hash,created_at,language,time_zone,region) VALUES('user-{user}','{user}@example.invalid','{user}@example.invalid','synthetic',0,'en','UTC','en-US')"))?;
        connection.batch_execute(&format!("BEGIN; INSERT INTO teams(id,kind,name,handle,owner_user_id,created_at) VALUES('personal-{user}','PERSONAL','Personal','personal-{user}','user-{user}',0); INSERT INTO memberships VALUES('personal-{user}','user-{user}','MEMBER'); INSERT INTO personal_contexts VALUES('user-{user}','personal-{user}'); COMMIT;"))?;
    }
    for (id, kind) in [("cloud", "CLOUD"), ("local", "LOCAL")] {
        connection.batch_execute(&format!("INSERT INTO computers(id,owner_user_id,kind,provider_id,desired_state,current_request_id,current_payload_hash,current_action,current_phase,created_at) VALUES('{id}','user-a','{kind}','provider-a','RUNNING','request-{id}',{hash},'CREATE','ACCEPTED',0)"))?;
    }
    connection.batch_execute("INSERT INTO agents(id,owner_user_id,team_id,name,computer_id,computer_kind,created_at) VALUES('cloud-agent','user-a','personal-a','Cloud','cloud','CLOUD',0)")?;
    assert!(connection.batch_execute("INSERT INTO agents(id,owner_user_id,team_id,name,computer_id,computer_kind,created_at) VALUES('cloud-duplicate','user-a','personal-a','Duplicate','cloud','CLOUD',0)").is_err());
    for id in ["local-one", "local-two"] {
        connection.batch_execute(&format!("INSERT INTO agents(id,owner_user_id,team_id,name,computer_id,computer_kind,created_at) VALUES('{id}','user-a','personal-a','Local','local','LOCAL',0)"))?;
    }
    assert!(connection.batch_execute("INSERT INTO agents(id,owner_user_id,team_id,name,computer_id,computer_kind,created_at) VALUES('foreign-owner','user-b','personal-b','Foreign','local','LOCAL',0)").is_err());
    assert!(connection.batch_execute("INSERT INTO agents(id,owner_user_id,team_id,name,computer_id,computer_kind,created_at) VALUES('wrong-kind','user-a','personal-a','Wrong','cloud','LOCAL',0)").is_err());
    connection.batch_execute(&format!("INSERT INTO agent_creation_receipts(owner_user_id,team_id,request_id,payload_hash,assigned_agent_id) VALUES('user-a','personal-a','creation',{hash},'cloud-agent'); DELETE FROM agents WHERE id='cloud-agent'"))?;
    // The receipt remains unique after its Agent disappears; it is not a live FK.
    assert!(connection.batch_execute(&format!("INSERT INTO agent_creation_receipts(owner_user_id,team_id,request_id,payload_hash,assigned_agent_id) VALUES('user-a','personal-a','creation',{hash},'replacement-agent')")).is_err());
    connection.batch_execute("INSERT INTO computer_executions(id,computer_id,initial_allocation,created_at) VALUES('execution-a','cloud',FALSE,0); UPDATE computers SET current_execution_id='execution-a' WHERE id='cloud'")?;
    assert!(connection.batch_execute(&format!("INSERT INTO connector_sessions(id,computer_id,execution_id,initial_request_id,initial_payload_hash,initial_hash,revoked_at,created_at) VALUES('wrong-session','local','execution-a','initial',{hash},{hash},0,0)")).is_err());
    connection.batch_execute(&format!("INSERT INTO connector_sessions(id,computer_id,execution_id,initial_request_id,initial_payload_hash,initial_hash,revoked_at,created_at) VALUES('bound-session','cloud','execution-a','initial',{hash},{hash},0,0)"))?;
    connection.batch_execute("INSERT INTO control_instances(id,endpoint,renewed_at,expires_at) VALUES('control','http://127.0.0.1:1',0,1000)")?;
    assert!(connection.batch_execute(&format!("INSERT INTO connector_connections(computer_id,session_id,control_id,connection_id,credential_hash,expires_at) VALUES('local','bound-session','control','foreign-route',{hash},1000)")).is_err());
    connection.batch_execute(&format!("INSERT INTO connector_connections(computer_id,session_id,control_id,connection_id,credential_hash,expires_at) VALUES('cloud','bound-session','control','owned-route',{hash},1000)"))?;
    assert!(connection.batch_execute("UPDATE connector_connections SET computer_id='local' WHERE session_id='bound-session'").is_err());
    assert!(
        connection
            .batch_execute(
                "UPDATE computers SET current_execution_id='execution-a' WHERE id='local'"
            )
            .is_err()
    );
    assert!(connection.batch_execute("INSERT INTO computer_resources(computer_id,provider_id,resource_kind,native_scope,name,execution_id,presence) VALUES('local','provider-a','POD','fixture','foreign','execution-a','PRESENT')").is_err());
    connection.batch_execute("INSERT INTO computer_resources(computer_id,provider_id,resource_kind,native_scope,name,resource_uid,presence) VALUES('cloud','provider-a','PVC','fixture','saved','uid-saved','PRESENT')")?;
    assert!(connection.batch_execute("INSERT INTO computer_resources(computer_id,provider_id,resource_kind,native_scope,name,presence) VALUES('cloud','provider-a','PVC','fixture','empty-replacement','PRESENT')").is_err());
    // The same Provider resource cannot be assigned to a second Computer.
    assert!(connection.batch_execute("INSERT INTO computer_resources(computer_id,provider_id,resource_kind,native_scope,name,presence) VALUES('local','provider-a','PVC','fixture','saved','UNKNOWN')").is_err());
    connection.batch_execute(&provider_sql("provider-b", postgres))?;
    assert!(connection.batch_execute("INSERT INTO computer_resources(computer_id,provider_id,resource_kind,native_scope,name,presence) VALUES('local','provider-b','PVC','fixture','wrong-provider','UNKNOWN')").is_err());
    assert!(
        connection
            .batch_execute("UPDATE computers SET agent_removed_at=1 WHERE id='cloud'")
            .is_err()
    );
    connection.batch_execute("UPDATE computers SET desired_state='DELETING',current_action='DELETE',current_phase='DELETING',agent_removed_at=1 WHERE id='cloud'")?;
    Ok(())
}

#[test]
fn sqlite_computer_constraints_and_provider_only_root_guard() -> Result<(), TestError> {
    let postgres = false;
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("computers.sqlite");
    let original = cipher(1)?;
    prepare_sqlite(&path, &original)?;
    let mut connection =
        diesel::SqliteConnection::establish(path.to_str().ok_or("Invalid fixture path")?)?;
    connection.batch_execute("PRAGMA foreign_keys=ON")?;
    connection.batch_execute(&provider_sql("root-guard", postgres))?;
    connection.batch_execute("DELETE FROM installation_key_binding")?;
    assert!(matches!(
        prepare_sqlite(&path, &cipher(2)?),
        Err(PreparationError::MissingBinding)
    ));
    connection.batch_execute(&compact_fixture_sql(
        "DELETE FROM computer_providers WHERE id='root-guard'",
        postgres,
    ))?;
    prepare_sqlite(&path, &original)?;
    exercise_constraints(&mut connection, false)?;
    assert!(
        connection
            .transaction::<_, TestError, _>(|connection| {
                connection.revert_last_migration(SQLITE_MIGRATIONS)?;
                connection.revert_last_migration(SQLITE_MIGRATIONS)?;
                connection.revert_last_migration(SQLITE_MIGRATIONS)?;
                connection.revert_last_migration(SQLITE_MIGRATIONS)?;
                connection.revert_last_migration(SQLITE_MIGRATIONS)?;
                Ok(())
            })
            .is_err()
    );
    sqlite_ready(&path, &original)?;
    Ok(())
}

#[derive(diesel::QueryableByName)]
struct Count {
    #[diesel(sql_type = BigInt)]
    n: i64,
}

#[test]
fn postgres_computer_constraints_and_provider_only_root_guard() -> Result<(), TestError> {
    let base = std::env::var("CHAPCHOO_TEST_POSTGRES_URL")?;
    let mut admin = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&base)?;
    let token = BearerSecret::generate()?;
    let suffix = token.digest().as_bytes()[..8]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let schema = format!("computer_schema_{suffix}");
    admin.batch_execute(&format!("CREATE SCHEMA {schema}"))?;
    let separator = if base.contains('?') { '&' } else { '?' };
    let url = format!("{base}{separator}options=-csearch_path%3D{schema}");
    let result = postgres_in_schema(&url);
    let cleanup = admin.batch_execute(&format!("DROP SCHEMA {schema} CASCADE"));
    support::combine(result, cleanup.map_err(Into::into)).map(|_| ())
}

fn postgres_in_schema(url: &str) -> Result<(), TestError> {
    let postgres = true;
    let original = cipher(1)?;
    prepare_postgres(url, &original)?;
    let mut connection = AsyncConnectionWrapper::<AsyncPgConnection>::establish(url)?;
    connection.batch_execute(&provider_sql("root-guard", postgres))?;
    connection.batch_execute("DELETE FROM installation_key_binding")?;
    assert!(matches!(
        prepare_postgres(url, &cipher(2)?),
        Err(PreparationError::MissingBinding)
    ));
    connection.batch_execute(&compact_fixture_sql(
        "DELETE FROM computer_providers WHERE id='root-guard'",
        postgres,
    ))?;
    prepare_postgres(url, &original)?;
    let actual = diesel::sql_query("SELECT count(*) AS n FROM pg_constraint WHERE conname='current_execution_belongs_to_computer' AND conrelid='computers'::regclass AND contype='f' AND condeferrable").get_result::<Count>(&mut connection)?;
    assert_eq!(actual.n, 1);
    exercise_constraints(&mut connection, true)?;
    assert!(
        connection
            .transaction::<_, TestError, _>(|connection| {
                connection.revert_last_migration(POSTGRES_MIGRATIONS)?;
                connection.revert_last_migration(POSTGRES_MIGRATIONS)?;
                connection.revert_last_migration(POSTGRES_MIGRATIONS)?;
                connection.revert_last_migration(POSTGRES_MIGRATIONS)?;
                connection.revert_last_migration(POSTGRES_MIGRATIONS)?;
                Ok(())
            })
            .is_err()
    );
    postgres_ready(url, &original)?;
    Ok(())
}
