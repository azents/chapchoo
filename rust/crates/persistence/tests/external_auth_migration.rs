mod support;

use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::{BearerSecret, CredentialCipher, RootKey};
use chapchoo_persistence::{
    POSTGRES_MIGRATIONS, SQLITE_MIGRATIONS, prepare_postgres, prepare_sqlite,
};
use diesel::{Connection, RunQueryDsl, connection::SimpleConnection, sql_types::BigInt};
use diesel_async::{AsyncPgConnection, async_connection_wrapper::AsyncConnectionWrapper};
use diesel_migrations::MigrationHarness;
use std::error::Error;

type TestError = Box<dyn Error + Send + Sync>;

#[derive(diesel::QueryableByName)]
struct Count {
    #[diesel(sql_type = BigInt)]
    n: i64,
}

fn identity(value: u8, postgres: bool) -> String {
    if postgres {
        format!("'01900000-0000-7000-8000-{value:012x}'::uuid")
    } else {
        format!("X'01900000000070008000{value:012x}'")
    }
}
fn blob(size: usize, postgres: bool) -> String {
    if postgres {
        format!("decode(repeat('ab',{size}),'hex')")
    } else {
        format!("zeroblob({size})")
    }
}
fn cipher() -> Result<CredentialCipher, TestError> {
    Ok(CredentialCipher::new(RootKey::parse(
        URL_SAFE.encode([91; 32]),
    )?)?)
}

macro_rules! exercise {
    ($connection:expr,$source:expr,$postgres:expr) => {{
        let connection = &mut *$connection;
        let source = || $source;
        let pg = $postgres;
        // Empty new tables permit an explicit downgrade to the previous schema.
        connection.revert_last_migration(source())?;
        assert_eq!(connection.applied_migrations()?.len(),9);
        let user = identity(1,pg);
        let team = identity(2,pg);
        let record = identity(3,pg);
        let attempt = identity(4,pg);
        let removed_agent = identity(5,pg);
        let hash = blob(32,pg);
        let nonce = blob(24,pg);
        let payload = blob(33,pg);
        connection.batch_execute(&format!(
            "INSERT INTO users(id,email_key,delivery_email,password_hash,created_at,language,time_zone,region) VALUES({user},'original@example.invalid','Original@example.invalid','synthetic',1,'en','UTC','en-US');
             INSERT INTO agent_creation_receipts(owner_user_id,request_id,payload_hash,assigned_agent_id,team_id) VALUES({user},'historical-create',{hash},{removed_agent},{team})"
        ))?;
        connection.run_pending_migrations(source())?;
        assert_eq!(connection.applied_migrations()?.len(),10);
        assert_eq!(diesel::sql_query(format!("SELECT count(*) AS n FROM agent_creation_receipts WHERE owner_user_id={user} AND request_id='historical-create' AND payload_hash={hash} AND assigned_agent_id={removed_agent} AND team_id={team}")).get_result::<Count>(connection)?.n,1);
        assert_eq!(diesel::sql_query("SELECT count(*) AS n FROM external_authentications").get_result::<Count>(connection)?.n,0);
        assert_eq!(diesel::sql_query("SELECT count(*) AS n FROM agent_authentication_selections").get_result::<Count>(connection)?.n,0);
        let insert = format!("INSERT INTO external_authentications(id,owner_user_id,kind,name,nonce,ciphertext,current_request_id,created_at,updated_at) VALUES({record},{user},'OPENAI_API_KEY','Synthetic',{nonce},{payload},'register',1,1)");
        connection.batch_execute(&insert)?;
        for invalid in [
            format!("UPDATE external_authentications SET owner_user_id=NULL WHERE id={record}"),
            format!("UPDATE external_authentications SET owner_team_id={team} WHERE id={record}"),
            format!("UPDATE external_authentications SET kind='OTHER' WHERE id={record}"),
            format!("UPDATE external_authentications SET ciphertext={} WHERE id={record}",blob(16,pg)),
            format!("UPDATE external_authentications SET nonce={} WHERE id={record}",blob(23,pg)),
            format!("UPDATE external_authentications SET account_id='invented' WHERE id={record}"),
            format!("INSERT INTO agent_authentication_selections VALUES({removed_agent},{record},'select',1)"),
        ] {
            assert!(connection.batch_execute(&invalid).is_err(),"Invalid contract accepted");
        }
        // Real registered credentials and even accepted result receipts cannot be erased by down migration.
        assert!(connection.revert_last_migration(source()).is_err());
        assert_eq!(connection.applied_migrations()?.len(),10);
        connection.batch_execute(&format!("INSERT INTO external_auth_attempts(id,initiated_by,owner_user_id,request_id,phase,step_id,expires_at,created_at,updated_at) VALUES({attempt},{user},{user},'device','STARTING','step',1000,1,1)"))?;
        connection.batch_execute(&format!("UPDATE external_auth_attempts SET phase='PENDING',step_id=NULL,pending_nonce={nonce},pending_ciphertext={payload},interval_ms=5000,next_check_at=10 WHERE id={attempt}"))?;
        assert!(connection.batch_execute(&format!("UPDATE external_auth_attempts SET phase='EXCHANGING' WHERE id={attempt}")).is_err());
        assert!(connection.batch_execute(&format!("UPDATE external_auth_attempts SET phase='CANCELLED' WHERE id={attempt}")).is_err());
        connection.batch_execute(&format!("UPDATE external_auth_attempts SET phase='CANCELLED',pending_nonce=NULL,pending_ciphertext=NULL WHERE id={attempt}"))?;
        connection.batch_execute(&format!("DELETE FROM external_auth_attempts WHERE id={attempt}; DELETE FROM external_authentications WHERE id={record}; INSERT INTO external_auth_requests VALUES({user},'register','REGISTER_KEY',{hash},{record},NULL,NULL,1,{user},NULL)"))?;
        assert!(connection.batch_execute("UPDATE external_auth_requests SET owner_user_id=NULL").is_err());
        assert_eq!(diesel::sql_query(format!("SELECT count(*) AS n FROM external_auth_requests WHERE owner_user_id={user} AND assigned_authentication_id={record}")).get_result::<Count>(connection)?.n,1);
        assert!(connection.revert_last_migration(source()).is_err());
        assert_eq!(diesel::sql_query("SELECT count(*) AS n FROM external_auth_requests").get_result::<Count>(connection)?.n,1);
        connection.batch_execute("DELETE FROM external_auth_requests")?;
        connection.revert_last_migration(source())?;
        connection.run_pending_migrations(source())?;
        assert_eq!(diesel::sql_query("SELECT count(*) AS n FROM agent_creation_receipts WHERE request_id='historical-create'").get_result::<Count>(connection)?.n,1);
        if !pg {
            assert_eq!(diesel::sql_query("SELECT count(*) AS n FROM pragma_foreign_key_check").get_result::<Count>(connection)?.n,0);
        }
    }};
}

#[test]
fn sqlite_external_auth_schema_preserves_existing_receipts_and_rejects_invalid_state()
-> Result<(), TestError> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("auth.sqlite");
    prepare_sqlite(&path, &cipher()?)?;
    let mut connection = diesel::SqliteConnection::establish(path.to_str().ok_or("Invalid path")?)?;
    connection.batch_execute("PRAGMA foreign_keys=ON")?;
    exercise!(&mut connection, SQLITE_MIGRATIONS, false);
    Ok(())
}

#[test]
fn sqlite_external_auth_binary_columns_reject_equal_length_text() -> Result<(), TestError> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("binary.sqlite");
    prepare_sqlite(&path, &cipher()?)?;
    let mut connection = diesel::SqliteConnection::establish(path.to_str().ok_or("Invalid path")?)?;
    // Disable foreign keys to isolate the column contract from unrelated FK
    // rejection. Each candidate is otherwise valid and passes with BLOB values.
    connection.batch_execute("PRAGMA foreign_keys=OFF")?;
    let id = "zeroblob(16)";
    let cases = [
        (
            "external_authentications",
            "id,owner_user_id,kind,name,nonce,ciphertext,current_request_id,active_attempt_id,created_at,updated_at",
            vec![
                id,
                id,
                "'OPENAI_API_KEY'",
                "'Synthetic'",
                "zeroblob(24)",
                "zeroblob(33)",
                "'register'",
                id,
                "1",
                "1",
            ],
            vec![(0, 16), (1, 16), (4, 24), (5, 33), (7, 16)],
        ),
        (
            "external_authentications",
            "id,owner_team_id,kind,name,nonce,ciphertext,current_request_id,created_at,updated_at",
            vec![
                id,
                id,
                "'OPENAI_API_KEY'",
                "'Synthetic'",
                "zeroblob(24)",
                "zeroblob(33)",
                "'register'",
                "1",
                "1",
            ],
            vec![(1, 16)],
        ),
        (
            "external_auth_attempts",
            "id,initiated_by,owner_user_id,request_id,target_id,expected_target_request_id,phase,pending_nonce,pending_ciphertext,interval_ms,next_check_at,expires_at,created_at,updated_at",
            vec![
                id,
                id,
                id,
                "'device'",
                id,
                "'prior'",
                "'PENDING'",
                "zeroblob(24)",
                "zeroblob(33)",
                "5000",
                "10",
                "1000",
                "1",
                "1",
            ],
            vec![(0, 16), (1, 16), (2, 16), (4, 16), (7, 24), (8, 33)],
        ),
        (
            "external_auth_attempts",
            "id,initiated_by,owner_team_id,request_id,phase,result_authentication_id,expires_at,created_at,updated_at",
            vec![id, id, id, "'device'", "'REGISTERED'", id, "1000", "1", "1"],
            vec![(2, 16), (5, 16)],
        ),
        (
            "external_auth_requests",
            "actor_user_id,request_id,operation,fingerprint,assigned_authentication_id,agent_id,attempt_id,accepted_at,owner_user_id",
            vec![
                id,
                "'register'",
                "'REGISTER_KEY'",
                "zeroblob(32)",
                id,
                id,
                id,
                "1",
                id,
            ],
            vec![(0, 16), (3, 32), (4, 16), (5, 16), (6, 16), (8, 16)],
        ),
        (
            "external_auth_requests",
            "actor_user_id,request_id,operation,fingerprint,accepted_at,owner_team_id",
            vec![id, "'register'", "'REGISTER_KEY'", "zeroblob(32)", "1", id],
            vec![(5, 16)],
        ),
        (
            "agent_authentication_selections",
            "agent_id,authentication_id,current_request_id,selected_at",
            vec![id, id, "'select'", "1"],
            vec![(0, 16), (1, 16)],
        ),
    ];
    for (table, columns, values, binaries) in cases {
        let insert = |values: &[&str]| {
            format!(
                "INSERT INTO {table}({columns}) VALUES({})",
                values.join(",")
            )
        };
        connection.batch_execute(&insert(&values))?;
        connection.batch_execute(&format!("DELETE FROM {table}"))?;
        for (index, length) in binaries {
            let text = format!("'{}'", "a".repeat(length));
            let mut invalid = values.clone();
            invalid[index] = &text;
            assert!(
                connection.batch_execute(&insert(&invalid)).is_err(),
                "TEXT accepted by {table} column {}",
                columns.split(',').nth(index).ok_or("Invalid column")?
            );
        }
    }
    Ok(())
}

#[test]
fn postgres_external_auth_schema_preserves_existing_receipts_and_rejects_invalid_state()
-> Result<(), TestError> {
    let base = std::env::var("CHAPCHOO_TEST_POSTGRES_URL")?;
    let mut connection = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&base)?;
    let suffix = BearerSecret::generate()?.digest().as_bytes()[..8]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let schema = format!("external_auth_test_{suffix}");
    connection.batch_execute(&format!("CREATE SCHEMA {schema}"))?;
    let separator = if base.contains('?') { '&' } else { '?' };
    let url = format!("{base}{separator}options=-csearch_path%3D{schema}");
    let result = (|| -> Result<(), TestError> {
        prepare_postgres(&url, &cipher()?)?;
        let mut scoped = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&url)?;
        exercise!(&mut scoped, POSTGRES_MIGRATIONS, true);
        Ok(())
    })();
    let cleanup = connection.batch_execute(&format!("DROP SCHEMA {schema} CASCADE"));
    support::combine(result, cleanup.map_err(Into::into)).map(|_| ())
}
