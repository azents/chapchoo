use chapchoo_crypto::{CredentialCipher, CryptoError, InstallationKeyCheck};
use diesel::{
    Connection, RunQueryDsl,
    connection::SimpleConnection,
    sql_types::{BigInt, Binary, Text},
};
use diesel_async::{AsyncPgConnection, async_connection_wrapper::AsyncConnectionWrapper};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use std::{error::Error, fs::OpenOptions, path::Path};

pub const SQLITE_MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/sqlite");
pub const POSTGRES_MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/postgres");
const VERSION: &str = "00000000000001";
const PG_LOCK: i64 = 1_960_261_029;
const PRODUCT_ROWS: &str = "SELECT
    (SELECT count(*) FROM users) +
    (SELECT count(*) FROM teams) +
    (SELECT count(*) FROM auth_sessions) +
    (SELECT count(*) FROM grants) +
    (SELECT count(*) FROM invitations) +
    (SELECT count(*) FROM login_challenges) AS n";

#[derive(Debug, thiserror::Error)]
pub enum PreparationError {
    #[error("Database preparation is already running")]
    Busy,
    #[error("Database connection failed")]
    Connection(#[source] diesel::ConnectionError),
    #[error("Database operation failed")]
    Database(#[source] diesel::result::Error),
    #[error("Database migration failed")]
    Migration(#[source] Box<dyn Error + Send + Sync>),
    #[error("Database file access failed")]
    File(#[source] std::io::Error),
    #[error("Installation encryption key validation failed")]
    Key(#[source] CryptoError),
    #[error("Database schema is not prepared")]
    Unprepared,
    #[error("Installation key binding is missing or malformed")]
    MissingBinding,
}

impl From<diesel::result::Error> for PreparationError {
    fn from(value: diesel::result::Error) -> Self {
        Self::Database(value)
    }
}

#[derive(diesel::QueryableByName)]
struct Count {
    #[diesel(sql_type = BigInt)]
    n: i64,
}

#[derive(diesel::QueryableByName)]
struct KeyRow {
    #[diesel(sql_type = Binary)]
    key_check: Vec<u8>,
}

#[derive(diesel::QueryableByName)]
struct MigrationVersion {
    #[diesel(sql_type = Text)]
    version: String,
}

#[derive(diesel::QueryableByName)]
struct Locked {
    #[diesel(sql_type = diesel::sql_types::Bool)]
    held: bool,
}

fn verify_key(rows: Vec<KeyRow>, cipher: &CredentialCipher) -> Result<(), PreparationError> {
    let [row] = rows.as_slice() else {
        return Err(PreparationError::MissingBinding);
    };
    let bytes: [u8; 32] = row
        .key_check
        .as_slice()
        .try_into()
        .map_err(|_| PreparationError::MissingBinding)?;
    cipher
        .verify_installation(&InstallationKeyCheck::from_bytes(bytes))
        .map_err(PreparationError::Key)
}

fn verify_versions(versions: Vec<MigrationVersion>) -> Result<(), PreparationError> {
    if versions.len() == 1 && versions[0].version == VERSION {
        Ok(())
    } else {
        Err(PreparationError::Unprepared)
    }
}

fn sqlite_has_binding(connection: &mut diesel::SqliteConnection) -> Result<bool, PreparationError> {
    Ok(diesel::sql_query(
        "SELECT count(*) AS n FROM sqlite_master WHERE type='table' AND name='installation_key_binding'",
    )
    .get_result::<Count>(connection)?
    .n == 1)
}

fn sqlite_binding(
    connection: &mut diesel::SqliteConnection,
    cipher: &CredentialCipher,
) -> Result<(), PreparationError> {
    verify_key(
        diesel::sql_query("SELECT key_check FROM installation_key_binding WHERE singleton=1")
            .load(connection)?,
        cipher,
    )
}

/// Synchronous preparation owns the database file lock across migration commits.
pub fn prepare_sqlite(path: &Path, cipher: &CredentialCipher) -> Result<(), PreparationError> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .map_err(PreparationError::File)?;
    match file.try_lock() {
        Ok(()) => {}
        Err(std::fs::TryLockError::WouldBlock) => return Err(PreparationError::Busy),
        Err(std::fs::TryLockError::Error(error)) => return Err(PreparationError::File(error)),
    }
    let path = path.to_str().ok_or(PreparationError::Unprepared)?;
    let mut connection =
        diesel::SqliteConnection::establish(path).map_err(PreparationError::Connection)?;
    connection.batch_execute("PRAGMA foreign_keys=ON")?;
    let binding_table = sqlite_has_binding(&mut connection)?;
    let already_bound = binding_table
        && diesel::sql_query("SELECT count(*) AS n FROM installation_key_binding")
            .get_result::<Count>(&mut connection)?
            .n
            != 0;
    if already_bound {
        sqlite_binding(&mut connection, cipher)?;
    } else if binding_table
        && diesel::sql_query(PRODUCT_ROWS)
            .get_result::<Count>(&mut connection)?
            .n
            != 0
    {
        return Err(PreparationError::MissingBinding);
    }
    connection
        .run_pending_migrations(SQLITE_MIGRATIONS)
        .map_err(PreparationError::Migration)?;
    if !already_bound {
        connection.immediate_transaction::<_, PreparationError, _>(|connection| {
            if diesel::sql_query(PRODUCT_ROWS)
                .get_result::<Count>(connection)?
                .n
                != 0
            {
                return Err(PreparationError::MissingBinding);
            }
            let check = cipher.installation_check().map_err(PreparationError::Key)?;
            diesel::sql_query(
                "INSERT INTO installation_key_binding(singleton,key_check) VALUES(1,?)",
            )
            .bind::<Binary, _>(check.as_bytes().as_slice())
            .execute(connection)?;
            Ok(())
        })?;
    }
    sqlite_binding(&mut connection, cipher)?;
    drop(connection);
    // Dropping the owned file also releases this advisory lock on every error path.
    drop(file);
    Ok(())
}

/// Read-only file opening prevents readiness from creating an absent database.
pub fn sqlite_ready(path: &Path, cipher: &CredentialCipher) -> Result<(), PreparationError> {
    if !path.is_file() {
        return Err(PreparationError::Unprepared);
    }
    let raw_path = path.to_str().ok_or(PreparationError::Unprepared)?;
    // URI metacharacters must not alter the intended filename or read-only mode.
    let escaped = raw_path
        .replace('%', "%25")
        .replace('?', "%3F")
        .replace('#', "%23");
    let mut connection = diesel::SqliteConnection::establish(&format!("file:{escaped}?mode=ro"))
        .map_err(PreparationError::Connection)?;
    if !sqlite_has_binding(&mut connection)? {
        return Err(PreparationError::Unprepared);
    }
    sqlite_binding(&mut connection, cipher)?;
    verify_versions(
        diesel::sql_query("SELECT version FROM __diesel_schema_migrations ORDER BY version")
            .load(&mut connection)?,
    )
}

type Pg = AsyncConnectionWrapper<AsyncPgConnection>;

fn pg_has_binding(connection: &mut Pg) -> Result<bool, PreparationError> {
    Ok(diesel::sql_query(
        "SELECT count(*) AS n FROM pg_class WHERE oid=to_regclass('installation_key_binding')",
    )
    .get_result::<Count>(connection)?
    .n == 1)
}

fn pg_binding(connection: &mut Pg, cipher: &CredentialCipher) -> Result<(), PreparationError> {
    verify_key(
        diesel::sql_query("SELECT key_check FROM installation_key_binding WHERE singleton=1")
            .load(connection)?,
        cipher,
    )
}

/// One-shot synchronous entrypoint; its supported adapter owns a private runtime.
/// Async callers must use their bounded blocking boundary rather than nesting it.
pub fn prepare_postgres(url: &str, cipher: &CredentialCipher) -> Result<(), PreparationError> {
    let mut connection = Pg::establish(url).map_err(PreparationError::Connection)?;
    let locked = diesel::sql_query("SELECT pg_try_advisory_lock($1) AS held")
        .bind::<BigInt, _>(PG_LOCK)
        .get_result::<Locked>(&mut connection)?
        .held;
    if !locked {
        return Err(PreparationError::Busy);
    }
    let binding_table = pg_has_binding(&mut connection)?;
    let already_bound = binding_table
        && diesel::sql_query("SELECT count(*) AS n FROM installation_key_binding")
            .get_result::<Count>(&mut connection)?
            .n
            != 0;
    if already_bound {
        pg_binding(&mut connection, cipher)?;
    } else if binding_table
        && diesel::sql_query(PRODUCT_ROWS)
            .get_result::<Count>(&mut connection)?
            .n
            != 0
    {
        return Err(PreparationError::MissingBinding);
    }
    connection
        .run_pending_migrations(POSTGRES_MIGRATIONS)
        .map_err(PreparationError::Migration)?;
    if !already_bound {
        connection.transaction::<_, PreparationError, _>(|connection| {
            if diesel::sql_query(PRODUCT_ROWS)
                .get_result::<Count>(connection)?
                .n
                != 0
            {
                return Err(PreparationError::MissingBinding);
            }
            let check = cipher.installation_check().map_err(PreparationError::Key)?;
            diesel::sql_query(
                "INSERT INTO installation_key_binding(singleton,key_check) VALUES(1,$1)",
            )
            .bind::<Binary, _>(check.as_bytes().as_slice())
            .execute(connection)?;
            Ok(())
        })?;
    }
    pg_binding(&mut connection, cipher)?;
    diesel::sql_query("SELECT pg_advisory_unlock($1) AS held")
        .bind::<BigInt, _>(PG_LOCK)
        .get_result::<Locked>(&mut connection)?;
    // Connection drop releases the session lock on every earlier error path.
    Ok(())
}

pub fn postgres_ready(url: &str, cipher: &CredentialCipher) -> Result<(), PreparationError> {
    let mut connection = Pg::establish(url).map_err(PreparationError::Connection)?;
    connection.batch_execute("BEGIN READ ONLY")?;
    if !pg_has_binding(&mut connection)? {
        return Err(PreparationError::Unprepared);
    }
    pg_binding(&mut connection, cipher)?;
    verify_versions(
        diesel::sql_query("SELECT version FROM __diesel_schema_migrations ORDER BY version")
            .load(&mut connection)?,
    )?;
    connection.batch_execute("COMMIT")?;
    Ok(())
}
