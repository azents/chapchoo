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
pub(crate) const VERSIONS: [&str; 22] = [
    "00000000000001",
    "00000000000002",
    "00000000000003",
    "00000000000004",
    "00000000000005",
    "00000000000006",
    "00000000000007",
    "00000000000008",
    "00000000000009",
    "00000000000010",
    "00000000000011",
    "00000000000012",
    "00000000000013",
    "00000000000014",
    "00000000000015",
    "00000000000016",
    "00000000000017",
    "00000000000018",
    "00000000000019",
    "00000000000020",
    "00000000000021",
    "00000000000022",
];
const PG_LOCK: i64 = 1_960_261_029;
#[cfg(test)]
pub(crate) static POSTGRES_PREPARATION_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
const PRODUCT_ROWS: &str = "SELECT
    (SELECT count(*) FROM users) +
    (SELECT count(*) FROM teams) +
    (SELECT count(*) FROM auth_sessions) +
    (SELECT count(*) FROM grants) +
    (SELECT count(*) FROM invitations) +
    (SELECT count(*) FROM login_challenges) AS n";

// Computer tables exist only after migration. Keep the pre-migration account
// guard valid for older installations and include every new product table before
// permitting a first root binding after preparation.
fn prepared_product_query() -> String {
    let tables = [
        "computer_providers",
        "provider_keys",
        "provider_key_requests",
        "provider_registration_credentials",
        "computers",
        "agents",
        "agent_creation_receipts",
        "computer_executions",
        "computer_resources",
        "connector_sessions",
        "connector_key_requests",
        "control_instances",
        "provider_connections",
        "connector_connections",
        "external_authentications",
        "external_auth_attempts",
        "external_auth_requests",
        "agent_authentication_selections",
        "agent_harness_supplies",
        "agent_harness_supply_requests",
        "conversations",
        "agent_conversations",
        "conversation_inputs",
        "conversation_event_pairs",
        "conversation_records",
        "conversation_record_fields",
        "conversation_field_extents",
        "conversation_input_delivery",
        "conversation_requests",
        "conversation_answers",
        "conversation_input_preparation",
        "conversation_startups",
        "conversation_native_state",
        "conversation_native_details",
        "conversation_applied_supply",
        "conversation_stops",
        "authentication_refreshes",
        "execution_authentication_applications",
    ];
    let additions = tables
        .iter()
        .map(|table| format!(" + (SELECT count(*) FROM {table})"))
        .collect::<String>();
    format!("SELECT ({PRODUCT_ROWS}){additions} AS n")
}

#[derive(Debug, thiserror::Error)]
pub enum PreparationError {
    #[error("Preserving identity and profile preparation failed")]
    Transition(#[source] Box<crate::StoreError>),
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
    #[error("Preparation environment has no valid IANA time zone")]
    InvalidEnvironment,
}

impl From<diesel::result::Error> for PreparationError {
    fn from(value: diesel::result::Error) -> Self {
        Self::Database(value)
    }
}

#[derive(diesel::QueryableByName)]
pub(crate) struct Count {
    #[diesel(sql_type = BigInt)]
    pub n: i64,
}

impl From<crate::StoreError> for PreparationError {
    fn from(value: crate::StoreError) -> Self {
        Self::Transition(Box::new(value))
    }
}

// The forward migration consumes transaction-local assignments. A migration is
// marked applied only after all crypto/profile staging and schema work succeeds.
macro_rules! prepare_migrations {
    ($connection:expr, $source:expr, $postgres:expr, $cipher:expr) => {{
        let pending = $connection
            .pending_migrations($source)
            .map_err(PreparationError::Migration)?;
        for migration in pending {
            let transition = migration.name().version().to_string() == "00000000000009";
            if transition {
                crate::profile_preparation::stage_transition!($connection, $postgres, $cipher);
            }
            $connection
                .run_migration(&*migration)
                .map_err(PreparationError::Migration)?;
            if transition {
                $connection.batch_execute(crate::profile_preparation::DROP_STAGING)?;
            }
        }
        verify_versions(
            diesel::sql_query("SELECT version FROM __diesel_schema_migrations ORDER BY version")
                .load($connection)?,
        )?;
    }};
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
    if versions
        .iter()
        .map(|value| value.version.as_str())
        .eq(VERSIONS)
    {
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

macro_rules! initialize_display {
    ($connection:expr, $postgres:expr, $environment:expr) => {{
        let initialized = diesel::sql_query("SELECT count(*) AS n FROM display_defaults")
            .get_result::<Count>($connection)?.n;
        let pending = diesel::sql_query("SELECT count(*) AS n FROM users WHERE language='' OR time_zone='' OR region=''")
            .get_result::<Count>($connection)?.n;
        if initialized == 0 || pending != 0 {
            let environment = $environment.as_ref()
                .map_err(|_| PreparationError::InvalidEnvironment)?;
            if initialized == 0 {
                diesel::sql_query(if $postgres {
                    "INSERT INTO display_defaults(singleton,language,time_zone,region) VALUES(1,$1,$2,$3)"
                } else {
                    "INSERT INTO display_defaults(singleton,language,time_zone,region) VALUES(1,?,?,?)"
                }).bind::<Text,_>(environment.language()).bind::<Text,_>(environment.time_zone())
                    .bind::<Text,_>(environment.region()).execute($connection)?;
            }
            diesel::sql_query(if $postgres {
                "UPDATE users SET language=CASE WHEN language='' THEN (SELECT language FROM display_defaults WHERE singleton=1) ELSE language END,
                 time_zone=CASE WHEN time_zone='' THEN $1 ELSE time_zone END,
                 region=CASE WHEN region='' THEN (SELECT region FROM display_defaults WHERE singleton=1) ELSE region END
                 WHERE language='' OR time_zone='' OR region=''"
            } else {
                "UPDATE users SET language=CASE WHEN language='' THEN (SELECT language FROM display_defaults WHERE singleton=1) ELSE language END,
                 time_zone=CASE WHEN time_zone='' THEN ? ELSE time_zone END,
                 region=CASE WHEN region='' THEN (SELECT region FROM display_defaults WHERE singleton=1) ELSE region END
                 WHERE language='' OR time_zone='' OR region=''"
            }).bind::<Text,_>(environment.time_zone()).execute($connection)?;
        }
    }};
}

macro_rules! verify_display {
    ($connection:expr) => {{
        let rows = diesel::sql_query(
            "SELECT language,time_zone,region FROM display_defaults WHERE singleton=1",
        )
        .load::<crate::display::DisplayRow>($connection)?;
        let [row] = rows.as_slice() else {
            return Err(PreparationError::Unprepared);
        };
        row.value().map_err(|_| PreparationError::Unprepared)?;
        if diesel::sql_query(
            "SELECT count(*) AS n FROM users WHERE language='' OR time_zone='' OR region=''",
        )
        .get_result::<Count>($connection)?
        .n != 0
        {
            return Err(PreparationError::Unprepared);
        }
    }};
}

/// Synchronous preparation serializes migration, backfill and binding atomically.
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
    let environment = chapchoo_domain::DisplaySettings::from_environment();
    connection.immediate_transaction::<_, PreparationError, _>(|connection| {
        prepare_migrations!(connection, SQLITE_MIGRATIONS, false, cipher);
        if !already_bound {
            if diesel::sql_query(prepared_product_query())
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
        }
        initialize_display!(connection, false, environment);
        crate::profile_catalogue::seed_catalogue!(connection, false);
        crate::profile_catalogue::verify_profiles!(connection, false);
        verify_display!(connection);
        sqlite_binding(connection, cipher)?;
        Ok(())
    })?;
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
    // Match ordinary Store leases: a normal short writer is not schema failure.
    connection.batch_execute("PRAGMA busy_timeout=5000")?;
    if !sqlite_has_binding(&mut connection)? {
        return Err(PreparationError::Unprepared);
    }
    sqlite_binding(&mut connection, cipher)?;
    verify_versions(
        diesel::sql_query("SELECT version FROM __diesel_schema_migrations ORDER BY version")
            .load(&mut connection)?,
    )?;
    verify_display!(&mut connection);
    crate::profile_catalogue::verify_profiles!(&mut connection, false);
    Ok(())
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
    let environment = chapchoo_domain::DisplaySettings::from_environment();
    connection.transaction::<_, PreparationError, _>(|connection| {
        prepare_migrations!(connection, POSTGRES_MIGRATIONS, true, cipher);
        if !already_bound {
            if diesel::sql_query(prepared_product_query())
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
        }
        initialize_display!(connection, true, environment);
        crate::profile_catalogue::seed_catalogue!(connection, true);
        crate::profile_catalogue::verify_profiles!(connection, true);
        verify_display!(connection);
        pg_binding(connection, cipher)?;
        Ok(())
    })?;
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
    verify_display!(&mut connection);
    crate::profile_catalogue::verify_profiles!(&mut connection, true);
    connection.batch_execute("COMMIT")?;
    Ok(())
}
