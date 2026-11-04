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
use std::error::Error;

type TestError = Box<dyn Error + Send + Sync>;
#[derive(diesel::QueryableByName)]
struct Scope {
    #[diesel(sql_type = Text)]
    team_id: String,
}
#[derive(diesel::QueryableByName)]
struct Count {
    #[diesel(sql_type = BigInt)]
    n: i64,
}

macro_rules! exercise_migration {
    ($connection:expr, $source:expr, $postgres:expr) => {{
        let connection = $connection;
        let pending = connection.pending_migrations($source)?;
        assert_eq!(pending.len(), 7);
        connection.run_migrations(&pending[..4])?;
        let hash = if $postgres { "decode(repeat('00',32),'hex')" } else { "zeroblob(32)" };
        connection.batch_execute(&format!("INSERT INTO users(id,email_key,delivery_email,password_hash,created_at,language,time_zone,region) VALUES('owner','owner@example.invalid','owner@example.invalid','synthetic',0,'en','UTC','en-US'); INSERT INTO computer_providers(id,owner_kind,system_owner_id,name,provider_kind,created_at) VALUES('provider','SYSTEM','system','Provider','KUBERNETES',0); INSERT INTO computers(id,owner_user_id,kind,provider_id,desired_state,current_request_id,current_payload_hash,current_action,current_phase,created_at) VALUES('computer','owner','CLOUD','provider','RUNNING','create',{hash},'CREATE','ACCEPTED',0); INSERT INTO agents(id,owner_user_id,name,computer_id,computer_kind,created_at) VALUES('agent','owner','Original','computer','CLOUD',0); INSERT INTO agent_creation_receipts(owner_user_id,request_id,payload_hash,assigned_agent_id) VALUES('owner','create',{hash},'agent')"))?;
        // A missing owner mapping must roll the entire migration back, including
        // the temporary table rebuild and the Team deletion marker.
        connection.batch_execute("BEGIN; INSERT INTO teams(id,kind,name,handle,owner_user_id,created_at) VALUES('existing','COLLABORATIVE','Existing','existing','owner',0); INSERT INTO memberships VALUES('existing','owner','MANAGER'); INSERT INTO invitations VALUES('invitation','existing','recipient@example.invalid','Recipient@example.invalid','owner','PENDING',0,NULL); COMMIT;")?;
        assert!(connection.run_migration(&*pending[4]).is_err());
        assert_eq!(connection.applied_migrations()?.len(),4);
        assert_eq!(diesel::sql_query("SELECT count(*) AS n FROM agents WHERE id='agent' AND computer_id='computer' AND name='Original'").get_result::<Count>(connection)?.n,1);
        connection.batch_execute("BEGIN; INSERT INTO teams(id,kind,name,handle,owner_user_id,created_at) VALUES('personal','PERSONAL','Personal','personal','owner',0); INSERT INTO memberships VALUES('personal','owner','MEMBER'); INSERT INTO personal_contexts VALUES('owner','personal'); COMMIT;")?;
        connection.run_migration(&*pending[4])?;
        assert_eq!(connection.applied_migrations()?.len(),5);
        assert_eq!(diesel::sql_query("SELECT count(*) AS n FROM memberships WHERE team_id='existing' AND user_id='owner' AND role='MANAGER'").get_result::<Count>(connection)?.n,1);
        assert_eq!(diesel::sql_query("SELECT count(*) AS n FROM invitations WHERE id='invitation' AND team_id='existing' AND recipient_delivery_email='Recipient@example.invalid' AND status='PENDING'").get_result::<Count>(connection)?.n,1);
        assert_eq!(diesel::sql_query("SELECT team_id FROM personal_contexts WHERE user_id='owner'").get_result::<Scope>(connection)?.team_id,"personal");
        assert_eq!(diesel::sql_query("SELECT team_id FROM agents WHERE id='agent' AND computer_id='computer'").get_result::<Scope>(connection)?.team_id,"personal");
        assert_eq!(diesel::sql_query("SELECT team_id FROM agent_creation_receipts WHERE owner_user_id='owner' AND request_id='create' AND assigned_agent_id='agent'").get_result::<Scope>(connection)?.team_id,"personal");
        assert_eq!(diesel::sql_query("SELECT count(*) AS n FROM computers WHERE id='computer' AND current_request_id='create'").get_result::<Count>(connection)?.n,1);
        assert!(connection.batch_execute("UPDATE agents SET team_id=NULL WHERE id='agent'").is_err());
        assert!(connection.batch_execute("UPDATE agents SET team_id='missing' WHERE id='agent'").is_err());
        connection.batch_execute("BEGIN; INSERT INTO teams(id,kind,name,handle,owner_user_id,created_at) VALUES('collaborative','COLLABORATIVE','Other','other','owner',0); INSERT INTO memberships VALUES('collaborative','owner','MEMBER'); UPDATE agents SET team_id='collaborative' WHERE id='agent'; COMMIT;")?;
        assert!(connection.batch_execute("DELETE FROM teams WHERE id='collaborative'").is_err());
        assert_eq!(diesel::sql_query("SELECT count(*) AS n FROM agents WHERE id='agent'").get_result::<Count>(connection)?.n,1);
        assert!(connection.revert_last_migration($source).is_err());
        assert_eq!(connection.applied_migrations()?.len(),5);
    }};
}

#[test]
fn sqlite_agent_team_backfill_is_preserving_and_atomic() -> Result<(), TestError> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("migration.sqlite");
    let mut connection = diesel::SqliteConnection::establish(path.to_str().ok_or("Invalid path")?)?;
    connection.batch_execute("PRAGMA foreign_keys=ON")?;
    exercise_migration!(&mut connection, SQLITE_MIGRATIONS, false);
    Ok(())
}
#[test]
fn postgres_agent_team_backfill_is_preserving_and_atomic() -> Result<(), TestError> {
    let base = std::env::var("CHAPCHOO_TEST_POSTGRES_URL")?;
    let mut admin = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&base)?;
    let suffix = BearerSecret::generate()?.digest().as_bytes()[..8]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let schema = format!("agent_team_migration_{suffix}");
    admin.batch_execute(&format!("CREATE SCHEMA {schema}"))?;
    let separator = if base.contains('?') { '&' } else { '?' };
    let url = format!("{base}{separator}options=-csearch_path%3D{schema}");
    let result = postgres_in_schema(&url);
    let cleanup = admin.batch_execute(&format!("DROP SCHEMA {schema} CASCADE"));
    support::combine(result, cleanup.map_err(Into::into)).map(|_| ())
}
fn postgres_in_schema(url: &str) -> Result<(), TestError> {
    let mut connection = AsyncConnectionWrapper::<AsyncPgConnection>::establish(url)?;
    exercise_migration!(&mut connection, POSTGRES_MIGRATIONS, true);
    assert_eq!(diesel::sql_query("SELECT count(*) AS n FROM pg_constraint c JOIN pg_attribute a ON a.attrelid=c.conrelid AND a.attnum=ANY(c.conkey) WHERE c.conrelid='agents'::regclass AND c.confrelid='teams'::regclass AND c.contype='f' AND c.confdeltype='a' AND a.attname='team_id' AND a.attnotnull").get_result::<Count>(&mut connection)?.n,1);
    assert_eq!(diesel::sql_query("SELECT count(*) AS n FROM pg_index WHERE indexrelid='teams_handle_key'::regclass AND indisunique AND pg_get_expr(indpred,indrelid)='(deletion_requested_at IS NULL)'").get_result::<Count>(&mut connection)?.n,1);
    Ok(())
}
