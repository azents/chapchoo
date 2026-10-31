use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::{BearerSecret, CredentialCipher, RootKey};
use chapchoo_domain::DisplaySettings;
use chapchoo_persistence::{
    POSTGRES_MIGRATIONS, SQLITE_MIGRATIONS, postgres_ready, prepare_postgres, prepare_sqlite,
    sqlite_ready,
};
use diesel::{
    Connection, RunQueryDsl,
    connection::SimpleConnection,
    sql_types::{BigInt, Binary, Text},
};
use diesel_async::{AsyncPgConnection, async_connection_wrapper::AsyncConnectionWrapper};
use diesel_migrations::MigrationHarness;
use std::error::Error;
type TestError = Box<dyn Error + Send + Sync>;
#[derive(Debug, diesel::QueryableByName)]
struct Count {
    #[diesel(sql_type = BigInt)]
    n: i64,
}
#[derive(Debug, diesel::QueryableByName)]
struct Settings {
    #[diesel(sql_type = Text)]
    language: String,
    #[diesel(sql_type = Text)]
    time_zone: String,
    #[diesel(sql_type = Text)]
    region: String,
}
fn cipher() -> Result<CredentialCipher, TestError> {
    Ok(CredentialCipher::new(RootKey::parse(
        URL_SAFE.encode([3; 32]),
    )?)?)
}
const LEGACY_USER: &str = "INSERT INTO users(id,email_key,delivery_email,password_hash,is_admin,created_at) VALUES('legacy','legacy@example.invalid','legacy@example.invalid','synthetic',FALSE,0)";
const PENDING_USER: &str = "INSERT INTO users(id,email_key,delivery_email,password_hash,is_admin,created_at) VALUES('pending','pending@example.invalid','pending@example.invalid','synthetic',FALSE,0)";
fn assert_environment(row: &Settings, env: &DisplaySettings) {
    assert_eq!(row.language, env.language());
    assert_eq!(row.time_zone, env.time_zone());
    assert_eq!(row.region, env.region());
}
macro_rules! assert_upgrade_data {
    ($connection:expr, $environment:expr, $changed_zone:expr, $prepare:expr, $ready:expr) => {{
        let row = diesel::sql_query("SELECT language,time_zone,region FROM users WHERE id='legacy'").get_result::<Settings>($connection)?;
        assert_environment(&row, $environment);
        let defaults = diesel::sql_query("SELECT language,time_zone,region FROM display_defaults WHERE singleton=1").get_result::<Settings>($connection)?;
        assert_environment(&defaults, $environment);
        $connection.batch_execute(&format!("UPDATE display_defaults SET language='fr',time_zone='{}',region='fr-FR' WHERE singleton=1", $changed_zone))?;
        $connection.batch_execute(PENDING_USER)?;
        assert!($ready.is_err());
        $prepare?;
        $ready?;
        let legacy = diesel::sql_query("SELECT language,time_zone,region FROM users WHERE id='legacy'").get_result::<Settings>($connection)?;
        assert_environment(&legacy, $environment);
        let pending = diesel::sql_query("SELECT language,time_zone,region FROM users WHERE id='pending'").get_result::<Settings>($connection)?;
        assert_eq!(pending.language, "fr");
        assert_eq!(pending.region, "fr-FR");
        assert_eq!(pending.time_zone, $environment.time_zone());
        let defaults = diesel::sql_query("SELECT language,time_zone,region FROM display_defaults WHERE singleton=1").get_result::<Settings>($connection)?;
        assert_eq!(defaults.language, "fr"); assert_eq!(defaults.region, "fr-FR"); assert_eq!(defaults.time_zone, $changed_zone);
        $prepare?;
        let after = diesel::sql_query("SELECT language,time_zone,region FROM users WHERE id='pending'").get_result::<Settings>($connection)?;
        assert_eq!(after.time_zone, pending.time_zone);
    }};
}
#[test]
fn sqlite_populated_upgrade_is_atomic_and_preserves_default_and_backfill_authorities()
-> Result<(), TestError> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("upgrade.sqlite");
    let cipher = cipher()?;
    let environment = DisplaySettings::from_environment()?;
    let changed_zone = if environment.time_zone() == "Europe/London" {
        "Asia/Seoul"
    } else {
        "Europe/London"
    };
    let mut connection = diesel::SqliteConnection::establish(path.to_str().ok_or("Invalid path")?)?;
    connection
        .run_next_migration(SQLITE_MIGRATIONS)
        .map_err(|_| "Legacy fixture migration failed")?;
    diesel::sql_query("INSERT INTO installation_key_binding(singleton,key_check) VALUES(1,?)")
        .bind::<Binary, _>(cipher.installation_check()?.as_bytes().as_slice())
        .execute(&mut connection)?;
    connection.batch_execute(LEGACY_USER)?;
    connection.batch_execute("CREATE TRIGGER fail_backfill BEFORE UPDATE ON users BEGIN SELECT RAISE(ABORT,'fixture backfill interruption'); END")?;
    assert!(prepare_sqlite(&path, &cipher).is_err());
    assert_eq!(
        diesel::sql_query(
            "SELECT count(*) AS n FROM __diesel_schema_migrations WHERE version='00000000000002'"
        )
        .get_result::<Count>(&mut connection)?
        .n,
        0
    );
    assert_eq!(
        diesel::sql_query("SELECT count(*) AS n FROM sqlite_master WHERE name='display_defaults'")
            .get_result::<Count>(&mut connection)?
            .n,
        0
    );
    assert!(sqlite_ready(&path, &cipher).is_err());
    connection.batch_execute("DROP TRIGGER fail_backfill")?;
    prepare_sqlite(&path, &cipher)?;
    assert_upgrade_data!(
        &mut connection,
        &environment,
        changed_zone,
        prepare_sqlite(&path, &cipher),
        sqlite_ready(&path, &cipher)
    );
    Ok(())
}
#[test]
fn postgres_populated_upgrade_is_atomic_and_preserves_default_and_backfill_authorities()
-> Result<(), TestError> {
    let base = std::env::var("CHAPCHOO_TEST_POSTGRES_URL")?;
    let mut admin = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&base)?;
    let token = BearerSecret::generate()?;
    let suffix = token.digest().as_bytes()[..8]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let schema = format!("display_upgrade_{suffix}");
    admin.batch_execute(&format!("CREATE SCHEMA {schema}"))?;
    let separator = if base.contains('?') { '&' } else { '?' };
    let url = format!("{base}{separator}options=-csearch_path%3D{schema}");
    let result = postgres_upgrade(&url);
    let cleanup = admin.batch_execute(&format!("DROP SCHEMA {schema} CASCADE"));
    result?;
    cleanup?;
    Ok(())
}
fn postgres_upgrade(url: &str) -> Result<(), TestError> {
    let cipher = cipher()?;
    let environment = DisplaySettings::from_environment()?;
    let changed_zone = if environment.time_zone() == "Europe/London" {
        "Asia/Seoul"
    } else {
        "Europe/London"
    };
    let mut connection = AsyncConnectionWrapper::<AsyncPgConnection>::establish(url)?;
    connection
        .run_next_migration(POSTGRES_MIGRATIONS)
        .map_err(|_| "Legacy fixture migration failed")?;
    diesel::sql_query("INSERT INTO installation_key_binding(singleton,key_check) VALUES(1,$1)")
        .bind::<Binary, _>(cipher.installation_check()?.as_bytes().as_slice())
        .execute(&mut connection)?;
    connection.batch_execute(LEGACY_USER)?;
    connection.batch_execute("CREATE FUNCTION reject_backfill() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'fixture backfill interruption'; END $$; CREATE TRIGGER fail_backfill BEFORE UPDATE ON users FOR EACH ROW EXECUTE FUNCTION reject_backfill()")?;
    assert!(prepare_postgres(url, &cipher).is_err());
    assert_eq!(
        diesel::sql_query(
            "SELECT count(*) AS n FROM __diesel_schema_migrations WHERE version='00000000000002'"
        )
        .get_result::<Count>(&mut connection)?
        .n,
        0
    );
    assert_eq!(
        diesel::sql_query(
            "SELECT count(*) AS n FROM pg_class WHERE oid=to_regclass('display_defaults')"
        )
        .get_result::<Count>(&mut connection)?
        .n,
        0
    );
    assert!(postgres_ready(url, &cipher).is_err());
    connection
        .batch_execute("DROP TRIGGER fail_backfill ON users; DROP FUNCTION reject_backfill()")?;
    prepare_postgres(url, &cipher)?;
    assert_upgrade_data!(
        &mut connection,
        &environment,
        changed_zone,
        prepare_postgres(url, &cipher),
        postgres_ready(url, &cipher)
    );
    Ok(())
}
