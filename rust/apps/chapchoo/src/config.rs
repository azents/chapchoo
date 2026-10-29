use chapchoo_crypto::{BearerSecret, CredentialCipher, RootKey, SecretDigest};
use chapchoo_persistence::{
    SessionPolicy, Store, postgres_ready, prepare_postgres, prepare_sqlite, sqlite_ready,
};
use clap::{Args, Parser, Subcommand, ValueEnum};
use std::{env, fmt, net::SocketAddr, path::PathBuf, sync::Arc};

use crate::AppError;

#[derive(Debug, Parser)]
#[command(
    name = "chapchoo",
    version,
    about = "Chapchoo foreground backend and operator commands"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Prepare the configured database schema and fixed installation-key binding.
    Prepare,
    /// Grant administrator authority to one existing account using local operator access.
    GrantAdmin {
        #[arg(long)]
        email: String,
    },
    /// Run the selected gRPC roles in the foreground.
    Serve(ServeOptions),
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Role {
    General,
    Admin,
    Both,
}

#[derive(Debug, Args)]
pub struct ServeOptions {
    #[arg(long, value_enum, default_value = "both")]
    pub role: Role,
    #[arg(
        long,
        env = "CHAPCHOO_GENERAL_LISTEN",
        default_value = "127.0.0.1:50051"
    )]
    pub general_listen: SocketAddr,
    #[arg(long, env = "CHAPCHOO_ADMIN_LISTEN", default_value = "127.0.0.1:50052")]
    pub admin_listen: SocketAddr,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Required configuration is missing or malformed: {0}")]
    Variable(&'static str),
    #[error("Select exactly one of CHAPCHOO_SQLITE_PATH or CHAPCHOO_POSTGRES_URL")]
    DatabaseSelection,
    #[error("General and Admin must use different listener addresses")]
    SameListener,
    #[error("A selected backend component exited unexpectedly")]
    ComponentExit,
    #[error("Backend startup deadline was exceeded")]
    StartupTimeout,
    #[error("Backend cleanup deadline was exceeded; committed operations are not undone")]
    CleanupTimeout,
}

#[derive(Clone)]
enum Database {
    Sqlite(PathBuf),
    Postgres(String),
}

/// Cloning shares immutable backend-only configuration, never request identity.
#[derive(Clone)]
pub struct BackendConfig {
    database: Database,
    pub cipher: Arc<CredentialCipher>,
    readiness_capacity: Arc<tokio::sync::Semaphore>,
}

impl fmt::Debug for BackendConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("BackendConfig([REDACTED])")
    }
}

fn optional(name: &'static str) -> Result<Option<String>, ConfigError> {
    match env::var(name) {
        Ok(value) if !value.is_empty() => Ok(Some(value)),
        Ok(_) | Err(env::VarError::NotUnicode(_)) => Err(ConfigError::Variable(name)),
        Err(env::VarError::NotPresent) => Ok(None),
    }
}

fn required(name: &'static str) -> Result<String, ConfigError> {
    optional(name)?.ok_or(ConfigError::Variable(name))
}

fn duration(name: &'static str, default: Option<i64>) -> Result<Option<i64>, ConfigError> {
    let days = match optional(name)? {
        Some(value) => Some(
            value
                .parse::<i64>()
                .map_err(|_| ConfigError::Variable(name))?,
        ),
        None => default,
    };
    days.map(|value| {
        value
            .checked_mul(86_400_000)
            .filter(|value| *value > 0)
            .ok_or(ConfigError::Variable(name))
    })
    .transpose()
}

impl BackendConfig {
    pub fn load() -> Result<Self, AppError> {
        let database = match (
            optional("CHAPCHOO_SQLITE_PATH")?,
            optional("CHAPCHOO_POSTGRES_URL")?,
        ) {
            (Some(path), None) => Database::Sqlite(PathBuf::from(path)),
            (None, Some(url)) => {
                let parsed = url::Url::parse(&url)
                    .map_err(|_| ConfigError::Variable("CHAPCHOO_POSTGRES_URL"))?;
                if !matches!(parsed.scheme(), "postgres" | "postgresql") {
                    return Err(ConfigError::Variable("CHAPCHOO_POSTGRES_URL").into());
                }
                Database::Postgres(url)
            }
            _ => return Err(ConfigError::DatabaseSelection.into()),
        };
        let key = RootKey::parse(required("CHAPCHOO_INSTALLATION_KEY")?)
            .map_err(|_| ConfigError::Variable("CHAPCHOO_INSTALLATION_KEY"))?;
        Ok(Self {
            database,
            cipher: Arc::new(CredentialCipher::new(key)?),
            readiness_capacity: Arc::new(tokio::sync::Semaphore::new(2)),
        })
    }

    pub async fn prepare(&self) -> Result<(), AppError> {
        let config = self.clone();
        tokio::task::spawn_blocking(move || match &config.database {
            Database::Sqlite(path) => prepare_sqlite(path, &config.cipher),
            Database::Postgres(url) => prepare_postgres(url, &config.cipher),
        })
        .await??;
        Ok(())
    }

    pub async fn ready(&self) -> Result<(), AppError> {
        let permit = self.readiness_capacity.clone().try_acquire_owned()?;
        let config = self.clone();
        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            match &config.database {
                Database::Sqlite(path) => sqlite_ready(path, &config.cipher),
                Database::Postgres(url) => postgres_ready(url, &config.cipher),
            }
        })
        .await??;
        Ok(())
    }

    pub async fn connect(&self) -> Result<Store, AppError> {
        Ok(match &self.database {
            Database::Sqlite(path) => {
                Store::connect_sqlite(path.clone(), self.cipher.clone()).await?
            }
            Database::Postgres(url) => {
                Store::connect_postgres(url.clone(), self.cipher.clone()).await?
            }
        })
    }
}

pub struct ApplicationConfig {
    pub accounts: chapchoo_service::AccountsConfig,
    pub password_capacity: usize,
    pub mail: Option<MailConfig>,
}

pub struct MailConfig {
    pub sender: String,
    pub region: String,
    pub endpoint: Option<String>,
}

impl ApplicationConfig {
    pub fn load() -> Result<Self, AppError> {
        let setup_proof: Option<SecretDigest> = optional("CHAPCHOO_SETUP_PROOF")?
            .map(|value| BearerSecret::parse(value).map(|proof| proof.digest()))
            .transpose()
            .map_err(|_| ConfigError::Variable("CHAPCHOO_SETUP_PROOF"))?;
        let password_capacity = optional("CHAPCHOO_PASSWORD_WORKERS")?
            .map(|value| value.parse::<usize>())
            .transpose()
            .map_err(|_| ConfigError::Variable("CHAPCHOO_PASSWORD_WORKERS"))?
            .unwrap_or(4);
        if !(1..=256).contains(&password_capacity) {
            return Err(ConfigError::Variable("CHAPCHOO_PASSWORD_WORKERS").into());
        }
        let sender = optional("CHAPCHOO_SES_SENDER")?;
        let region = optional("CHAPCHOO_SES_REGION")?;
        let endpoint = optional("CHAPCHOO_SES_ENDPOINT")?;
        let mail = match (sender, region, endpoint) {
            (None, None, None) => None,
            (Some(sender), Some(region), endpoint) => {
                chapchoo_domain::EmailAddress::parse(&sender)
                    .map_err(|_| ConfigError::Variable("CHAPCHOO_SES_SENDER"))?;
                if let Some(endpoint) = &endpoint {
                    let parsed = url::Url::parse(endpoint)
                        .map_err(|_| ConfigError::Variable("CHAPCHOO_SES_ENDPOINT"))?;
                    if !matches!(parsed.scheme(), "http" | "https")
                        || parsed.host_str().is_none()
                        || !parsed.username().is_empty()
                        || parsed.password().is_some()
                    {
                        return Err(ConfigError::Variable("CHAPCHOO_SES_ENDPOINT").into());
                    }
                }
                Some(MailConfig {
                    sender,
                    region,
                    endpoint,
                })
            }
            _ => {
                return Err(
                    ConfigError::Variable("CHAPCHOO_SES_SENDER / CHAPCHOO_SES_REGION").into(),
                );
            }
        };
        let validity_ms = duration("CHAPCHOO_SESSION_VALIDITY_DAYS", Some(180))?
            .ok_or(ConfigError::Variable("CHAPCHOO_SESSION_VALIDITY_DAYS"))?;
        Ok(Self {
            accounts: chapchoo_service::AccountsConfig {
                general_web_url: optional("CHAPCHOO_GENERAL_WEB_URL")?
                    .unwrap_or_else(|| "http://localhost:3000".to_owned()),
                setup_proof,
                session_policy: SessionPolicy {
                    validity_ms,
                    absolute_lifetime_ms: duration("CHAPCHOO_SESSION_ABSOLUTE_DAYS", None)?,
                    ..SessionPolicy::default()
                },
            },
            password_capacity,
            mail,
        })
    }
}

impl fmt::Debug for ApplicationConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ApplicationConfig([REDACTED])")
    }
}

impl fmt::Debug for MailConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("MailConfig([REDACTED])")
    }
}
