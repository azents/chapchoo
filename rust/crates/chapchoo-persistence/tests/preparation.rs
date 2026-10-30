use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::{CredentialCipher, RootKey};
use chapchoo_persistence::{
    PreparationError, SQLITE_MIGRATIONS, postgres_ready, prepare_postgres, prepare_sqlite,
    sqlite_ready,
};
use diesel::{Connection, connection::SimpleConnection};
use diesel_async::{AsyncPgConnection, async_connection_wrapper::AsyncConnectionWrapper};
use diesel_migrations::MigrationHarness;
use std::{error::Error, fs::OpenOptions};

fn cipher(byte: u8) -> Result<CredentialCipher, Box<dyn Error>> {
    Ok(CredentialCipher::new(RootKey::parse(
        URL_SAFE.encode([byte; 32]),
    )?)?)
}

#[test]
fn sqlite_preparation_and_read_only_admission() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("database?#%.sqlite");
    let original = cipher(1)?;
    let wrong = cipher(2)?;
    assert!(matches!(
        sqlite_ready(&path, &original),
        Err(PreparationError::Unprepared)
    ));
    assert!(!path.exists());
    prepare_sqlite(&path, &original)?;
    sqlite_ready(&path, &original)?;
    prepare_sqlite(&path, &original)?;
    let before = std::fs::read(&path)?;
    assert!(matches!(
        sqlite_ready(&path, &wrong),
        Err(PreparationError::Key(_))
    ));
    assert!(matches!(
        prepare_sqlite(&path, &wrong),
        Err(PreparationError::Key(_))
    ));
    assert_eq!(std::fs::read(&path)?, before);
    let lock = OpenOptions::new().read(true).write(true).open(&path)?;
    lock.try_lock()?;
    assert!(matches!(
        prepare_sqlite(&path, &original),
        Err(PreparationError::Busy)
    ));
    drop(lock);
    prepare_sqlite(&path, &original)?;

    let mut connection =
        diesel::SqliteConnection::establish(path.to_str().ok_or("Invalid fixture path")?)?;
    connection.batch_execute("PRAGMA foreign_keys=ON")?;
    // The composite owner FK is deferred, but rejects an ownerless commit.
    connection.batch_execute(
        "INSERT INTO users VALUES('user-a','a@example.invalid','a@example.invalid','synthetic',0,0)",
    )?;
    assert!(
        connection
            .transaction::<_, diesel::result::Error, _>(|connection| {
                connection.batch_execute(
                    "INSERT INTO teams VALUES('team-a','COLLABORATIVE','Team','team-a','user-a',0)",
                )
            })
            .is_err()
    );
    connection.transaction::<_, diesel::result::Error, _>(|connection| {
        connection.batch_execute(
            "INSERT INTO teams VALUES('team-a','COLLABORATIVE','Team','team-a','user-a',0);
             INSERT INTO memberships VALUES('team-a','user-a','MEMBER')",
        )
    })?;
    assert!(
        connection
            .transaction::<_, diesel::result::Error, _>(|connection| {
                connection.batch_execute("DELETE FROM memberships WHERE team_id='team-a'")
            })
            .is_err()
    );
    assert!(
        connection
            .batch_execute("UPDATE memberships SET role='OWNER'")
            .is_err()
    );
    connection.batch_execute("DELETE FROM installation_key_binding")?;
    assert!(matches!(
        prepare_sqlite(&path, &original),
        Err(PreparationError::MissingBinding)
    ));
    Ok(())
}

#[test]
fn explicit_sqlite_preparation_resumes_before_first_key_binding() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("first-binding.sqlite");
    let mut connection =
        diesel::SqliteConnection::establish(path.to_str().ok_or("Invalid fixture path")?)?;
    connection
        .run_pending_migrations(SQLITE_MIGRATIONS)
        .map_err(|error| format!("Fixture migration failed: {error}"))?;
    drop(connection);
    let original = cipher(1)?;
    assert!(sqlite_ready(&path, &original).is_err());
    prepare_sqlite(&path, &original)?;
    sqlite_ready(&path, &original)?;
    Ok(())
}

#[test]
fn postgres_preparation_requires_real_disposable_database() -> Result<(), Box<dyn Error>> {
    // Mandatory integration prerequisite, never a skip counted as success.
    let base = std::env::var("CHAPCHOO_TEST_POSTGRES_URL")?;
    let mut connection = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&base)?;
    let token = chapchoo_crypto::BearerSecret::generate()?;
    let suffix = token.digest().as_bytes()[..8]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let schema = format!("preparation_test_{suffix}");
    connection.batch_execute(&format!("CREATE SCHEMA {schema}"))?;
    let separator = if base.contains('?') { '&' } else { '?' };
    let url = format!("{base}{separator}options=-csearch_path%3D{schema}");
    let result = postgres_preparation_in_schema(&url);
    let cleanup = connection.batch_execute(&format!("DROP SCHEMA {schema} CASCADE"));
    result?;
    cleanup?;
    Ok(())
}

fn postgres_preparation_in_schema(url: &str) -> Result<(), Box<dyn Error>> {
    let original = cipher(1)?;
    let wrong = cipher(2)?;
    assert!(matches!(
        postgres_ready(url, &original),
        Err(PreparationError::Unprepared)
    ));
    prepare_postgres(url, &original)?;
    postgres_ready(url, &original)?;
    prepare_postgres(url, &original)?;
    assert!(matches!(
        prepare_postgres(url, &wrong),
        Err(PreparationError::Key(_))
    ));
    assert!(matches!(
        postgres_ready(url, &wrong),
        Err(PreparationError::Key(_))
    ));
    let mut connection = AsyncConnectionWrapper::<AsyncPgConnection>::establish(url)?;
    connection.batch_execute(
        "INSERT INTO users VALUES('user-a','a@example.invalid','a@example.invalid','synthetic',FALSE,0)",
    )?;
    assert!(
        connection
            .transaction::<_, diesel::result::Error, _>(|connection| {
                connection.batch_execute(
                    "INSERT INTO teams VALUES('team-a','COLLABORATIVE','Team','team-a','user-a',0)",
                )
            })
            .is_err()
    );
    connection.transaction::<_, diesel::result::Error, _>(|connection| {
        connection.batch_execute(
            "INSERT INTO teams VALUES('team-a','COLLABORATIVE','Team','team-a','user-a',0);
             INSERT INTO memberships VALUES('team-a','user-a','MEMBER')",
        )
    })?;
    assert!(
        connection
            .batch_execute("UPDATE memberships SET role='OWNER'")
            .is_err()
    );
    connection.batch_execute("DELETE FROM installation_key_binding")?;
    assert!(matches!(
        prepare_postgres(url, &original),
        Err(PreparationError::MissingBinding)
    ));
    Ok(())
}
