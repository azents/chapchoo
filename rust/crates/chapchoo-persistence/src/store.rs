use chapchoo_crypto::CredentialCipher;
use diesel_async::{
    AsyncConnection, AsyncPgConnection, SimpleAsyncConnection,
    pooled_connection::{AsyncDieselConnectionManager, ManagerConfig, bb8::Pool},
    sync_connection_wrapper::SyncConnectionWrapper,
};
use std::{error::Error, fmt, path::PathBuf, sync::Arc, time::Duration};

use crate::{PreparationError, postgres_ready, sqlite_ready};

pub(crate) type Sqlite = SyncConnectionWrapper<diesel::SqliteConnection>;

#[derive(Clone)]
pub(crate) enum Pools {
    Postgres(Pool<AsyncPgConnection>),
    Sqlite(Pool<Sqlite>),
}

#[derive(Clone)]
pub struct Store {
    pub(crate) pools: Pools,
    pub(crate) cipher: Arc<CredentialCipher>,
}

impl fmt::Debug for Store {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Store([REDACTED])")
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("Storage connection acquisition failed")]
    Pool(#[source] Box<dyn Error + Send + Sync>),
    #[error("Storage operation failed")]
    Database(#[source] diesel::result::Error),
    #[error("Installation is not ready")]
    Preparation(#[from] PreparationError),
    #[error("Storage task failed")]
    Task(#[from] tokio::task::JoinError),
    #[error("Stored credential is invalid")]
    Crypto(#[from] chapchoo_crypto::CryptoError),
    #[error("Authentication required")]
    Unauthenticated,
    #[error("Permission denied")]
    Forbidden,
    #[error("Object not found")]
    NotFound,
    #[error("Conflicting existing resource")]
    Conflict,
    #[error("Operation is not valid in the current state")]
    InvalidTransition,
    #[error("Invalid input")]
    InvalidInput,
    #[error("Invalid or unusable proof")]
    InvalidProof,
    #[error("Email does not match the proof recipient")]
    EmailMismatch,
    #[error("Email is already registered")]
    EmailRegistered,
    #[error("User is already a member")]
    AlreadyMember,
    #[error("Invitation has already been processed")]
    AlreadyProcessed,
    #[error("Ownership transfer is required before leaving")]
    OwnerTransferRequired,
    #[error("Team handle is already in use")]
    HandleConflict,
    #[error("Registration through a signup link is required")]
    RegistrationRequired,
    #[error("Stored account state is inconsistent")]
    Integrity,
}

impl From<diesel::result::Error> for StoreError {
    fn from(error: diesel::result::Error) -> Self {
        if matches!(
            error,
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _
            )
        ) {
            Self::Conflict
        } else {
            Self::Database(error)
        }
    }
}

impl StoreError {
    pub(crate) fn unique_field(
        error: diesel::result::Error,
        postgres_constraint: &str,
        sqlite_column: &str,
        reason: Self,
    ) -> Self {
        if let diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::UniqueViolation,
            info,
        ) = &error
            && (info.constraint_name() == Some(postgres_constraint)
                || info.message() == format!("UNIQUE constraint failed: {sqlite_column}"))
        {
            return reason;
        }
        error.into()
    }

    pub(crate) fn pool(error: impl Error + Send + Sync + 'static) -> Self {
        Self::Pool(Box::new(error))
    }
}

impl Store {
    pub async fn connect_sqlite(
        path: PathBuf,
        cipher: Arc<CredentialCipher>,
    ) -> Result<Self, StoreError> {
        let check_path = path.clone();
        let check_cipher = cipher.clone();
        tokio::task::spawn_blocking(move || sqlite_ready(&check_path, &check_cipher)).await??;
        let path = path.to_str().ok_or(StoreError::Integrity)?.to_owned();
        let mut config = ManagerConfig::<Sqlite>::default();
        config.custom_setup = Box::new(|url| {
            Box::pin(async move {
                let mut connection = Sqlite::establish(url).await?;
                connection
                    .batch_execute("PRAGMA foreign_keys=ON; PRAGMA busy_timeout=5000")
                    .await
                    .map_err(diesel::ConnectionError::CouldntSetupConfiguration)?;
                Ok(connection)
            })
        });
        let pool = Pool::builder()
            .max_size(4)
            .connection_timeout(Duration::from_secs(5))
            .build(AsyncDieselConnectionManager::new_with_config(path, config))
            .await
            .map_err(StoreError::pool)?;
        Ok(Self {
            pools: Pools::Sqlite(pool),
            cipher,
        })
    }

    pub async fn connect_postgres(
        url: String,
        cipher: Arc<CredentialCipher>,
    ) -> Result<Self, StoreError> {
        let check_url = url.clone();
        let check_cipher = cipher.clone();
        tokio::task::spawn_blocking(move || postgres_ready(&check_url, &check_cipher)).await??;
        let pool = Pool::builder()
            .max_size(8)
            .connection_timeout(Duration::from_secs(5))
            .build(AsyncDieselConnectionManager::new(url))
            .await
            .map_err(StoreError::pool)?;
        Ok(Self {
            pools: Pools::Postgres(pool),
            cipher,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Dialect {
    Postgres,
    Sqlite,
}

impl Dialect {
    /// SQL is an internal static template; values always use Diesel bind parameters.
    pub(crate) fn query(self, sql: &str) -> String {
        if matches!(self, Self::Sqlite) {
            return sql.to_owned();
        }
        let mut parameter = 0;
        sql.chars()
            .map(|character| {
                if character == '?' {
                    parameter += 1;
                    format!("${parameter}")
                } else {
                    character.to_string()
                }
            })
            .collect()
    }

    pub(crate) fn user_lock(self) -> &'static str {
        match self {
            Self::Postgres => "SELECT * FROM users WHERE id=? FOR NO KEY UPDATE",
            Self::Sqlite => "SELECT * FROM users WHERE id=?",
        }
    }
}

// Expanding at concrete backend types preserves Diesel's bind/result checks.
// Each complete operation owns its lease inside its future, including cancellation.
macro_rules! operation {
    ($store:expr, |$connection:ident, $dialect:ident| $body:block) => {{
        match &$store.pools {
            $crate::store::Pools::Postgres(pool) => {
                let mut lease = pool.get().await.map_err($crate::StoreError::pool)?;
                diesel_async::AsyncConnection::transaction::<_, $crate::StoreError, _>(
                    &mut *lease,
                    async |$connection| {
                        let $dialect = $crate::store::Dialect::Postgres;
                        $body
                    },
                )
                .await
            }
            $crate::store::Pools::Sqlite(pool) => {
                let mut lease = pool.get().await.map_err($crate::StoreError::pool)?;
                lease
                    .immediate_transaction::<_, $crate::StoreError, _>(async |$connection| {
                        let $dialect = $crate::store::Dialect::Sqlite;
                        $body
                    })
                    .await
            }
        }
    }};
}

pub(crate) use operation;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{prepare_postgres, prepare_sqlite};
    use base64::{Engine, engine::general_purpose::URL_SAFE};
    use chapchoo_crypto::RootKey;
    use diesel::{Connection, connection::SimpleConnection};
    use diesel_async::{RunQueryDsl, async_connection_wrapper::AsyncConnectionWrapper};

    type TestError = Box<dyn Error + Send + Sync>;

    #[derive(Debug, diesel::QueryableByName)]
    struct Count {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        n: i64,
    }

    async fn verify_cancellation(store: Store) -> Result<(), TestError> {
        operation!(store, |connection, dialect| {
            diesel::sql_query(
                dialect.query("CREATE TEMP TABLE cancellation_owner_marker(n BIGINT)"),
            )
            .execute(connection)
            .await?;
            Ok(())
        })?;
        let (sender, receiver) = tokio::sync::oneshot::channel();
        {
            let operation = async {
                operation!(store, |connection, dialect| {
                    diesel::sql_query(dialect.query(
                        "INSERT INTO login_challenges(id,email_key,binding_hash,code_verifier,issued_at,expires_at)
                         VALUES('cancelled','fixture@example.invalid',?,?,0,1000)",
                    ))
                    .bind::<diesel::sql_types::Binary,_>([1_u8;32].as_slice())
                    .bind::<diesel::sql_types::Binary,_>([2_u8;32].as_slice())
                    .execute(connection).await?;
                    sender.send(()).map_err(|_| StoreError::Integrity)?;
                    std::future::pending::<Result<(), StoreError>>().await
                })
            };
            tokio::pin!(operation);
            tokio::select! {
                value=receiver=>{value?;},
                value=&mut operation=>{return Err(format!("Transaction ended before cancellation barrier: {value:?}").into());}
            }
        } // Drops the complete operation future and its owned pool lease.
        let count = operation!(store, |connection, dialect| {
            let marker_query = match dialect {
                Dialect::Postgres => {
                    "SELECT count(*) AS n FROM pg_class WHERE relnamespace=pg_my_temp_schema() AND relname='cancellation_owner_marker'"
                }
                Dialect::Sqlite => {
                    "SELECT count(*) AS n FROM sqlite_temp_master WHERE name='cancellation_owner_marker'"
                }
            };
            let marker = diesel::sql_query(marker_query)
                .get_result::<Count>(connection)
                .await?
                .n;
            assert_eq!(
                marker, 0,
                "Cancelled connection must not return to the pool"
            );
            let count = diesel::sql_query(
                dialect.query("SELECT count(*) AS n FROM login_challenges WHERE id='cancelled'"),
            )
            .get_result::<Count>(connection)
            .await?
            .n;
            diesel::sql_query(dialect.query(
                "INSERT INTO login_challenges(id,email_key,binding_hash,code_verifier,issued_at,expires_at)
                 VALUES('after-cancel','fixture@example.invalid',?,?,0,1000)",
            ))
            .bind::<diesel::sql_types::Binary,_>([3_u8;32].as_slice())
            .bind::<diesel::sql_types::Binary,_>([4_u8;32].as_slice())
            .execute(connection).await?;
            Ok(count)
        })?;
        assert_eq!(count, 0);
        Ok(())
    }

    fn cipher() -> Result<Arc<CredentialCipher>, TestError> {
        Ok(Arc::new(CredentialCipher::new(RootKey::parse(
            URL_SAFE.encode([1; 32]),
        )?)?))
    }

    #[test]
    fn cancelled_sqlite_operation_retires_owned_lease() -> Result<(), TestError> {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("cancellation.sqlite");
        let cipher = cipher()?;
        prepare_sqlite(&path, &cipher)?;
        tokio::runtime::Runtime::new()?.block_on(async {
            let store = Store::connect_sqlite(path, cipher).await?;
            verify_cancellation(store).await
        })
    }

    #[test]
    fn cancelled_postgres_operation_retires_owned_lease() -> Result<(), TestError> {
        let base = std::env::var("CHAPCHOO_TEST_POSTGRES_URL")?;
        let mut control =
            <AsyncConnectionWrapper<AsyncPgConnection> as Connection>::establish(&base)?;
        let suffix = chapchoo_crypto::BearerSecret::generate()?
            .digest()
            .as_bytes()[..8]
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        let schema = format!("cancellation_test_{suffix}");
        SimpleConnection::batch_execute(&mut control, &format!("CREATE SCHEMA {schema}"))?;
        let separator = if base.contains('?') { '&' } else { '?' };
        let url = format!("{base}{separator}options=-csearch_path%3D{schema}");
        let result = (|| -> Result<(), TestError> {
            let cipher = cipher()?;
            prepare_postgres(&url, &cipher)?;
            tokio::runtime::Runtime::new()?.block_on(async {
                let store = Store::connect_postgres(url, cipher).await?;
                verify_cancellation(store).await
            })
        })();
        let cleanup =
            SimpleConnection::batch_execute(&mut control, &format!("DROP SCHEMA {schema} CASCADE"));
        result?;
        cleanup?;
        Ok(())
    }
}
