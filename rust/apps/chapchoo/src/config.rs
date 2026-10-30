use chapchoo_crypto::{BearerSecret, CredentialCipher, RootKey, SecretDigest};
use chapchoo_persistence::{
    SessionPolicy, Store, postgres_ready, prepare_postgres, prepare_sqlite, sqlite_ready,
};
use clap::{Args, Parser, Subcommand, ValueEnum};
use std::{
    env,
    ffi::OsString,
    fmt,
    net::{IpAddr, SocketAddr},
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::AppError;

#[derive(Debug, Parser)]
#[command(
    name = "chapchoo",
    version,
    about = "Chapchoo foreground backend, bundled web and operator commands"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Prepare the configured database schema and fixed secret-key binding.
    Prepare,
    /// Grant administrator authority to one existing account using local operator access.
    GrantAdmin {
        #[arg(long)]
        email: String,
    },
    /// Run selected backend and bundled web components in the foreground.
    Serve(ServeOptions),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Role {
    /// Select only explicitly requested web components, without database authority.
    None,
    General,
    Admin,
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum WebSelection {
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
    #[arg(long, value_enum)]
    pub web: Option<WebSelection>,
    #[arg(
        long,
        env = "CHAPCHOO_GENERAL_WEB_LISTEN",
        default_value = "127.0.0.1:3003"
    )]
    pub general_web_listen: SocketAddr,
    #[arg(
        long,
        env = "CHAPCHOO_ADMIN_WEB_LISTEN",
        default_value = "127.0.0.1:3004"
    )]
    pub admin_web_listen: SocketAddr,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Required configuration is missing or malformed: {0}")]
    Variable(&'static str),
    #[error("Select exactly one of CHAPCHOO_SQLITE_PATH or CHAPCHOO_POSTGRES_URL")]
    DatabaseSelection,
    #[error("General and Admin must use different listener addresses")]
    SameListener,
    #[error("Select at least one backend or bundled web component")]
    NoComponents,
    #[error("The selected bundled Node runtime or web artifacts are unavailable")]
    WebArtifacts,
    #[error("Bundled web process groups are not supported on this platform")]
    WebPlatform,
    #[error("A selected component exited unexpectedly")]
    ComponentExit,
    #[error("Selected-component startup deadline was exceeded")]
    StartupTimeout,
    #[error("Owned cleanup deadline was exceeded; committed operations are not undone")]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebApp {
    General,
    Admin,
}

pub struct WebConfig {
    pub app: WebApp,
    pub listen: SocketAddr,
    pub public_url: url::Url,
    pub backend_url: url::Url,
    pub node: PathBuf,
    pub script: PathBuf,
    pub directory: PathBuf,
}

impl fmt::Debug for WebConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("WebConfig([REDACTED])")
    }
}

impl WebApp {
    fn public_variable(self) -> &'static str {
        match self {
            Self::General => "CHAPCHOO_GENERAL_WEB_URL",
            Self::Admin => "CHAPCHOO_ADMIN_WEB_URL",
        }
    }

    fn backend_variable(self) -> &'static str {
        match self {
            Self::General => "CHAPCHOO_GENERAL_BACKEND_URL",
            Self::Admin => "CHAPCHOO_ADMIN_BACKEND_URL",
        }
    }
}

fn endpoint(value: &str, name: &'static str) -> Result<url::Url, ConfigError> {
    let url = url::Url::parse(value).map_err(|_| ConfigError::Variable(name))?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(ConfigError::Variable(name));
    }
    Ok(url)
}

impl WebConfig {
    fn parse(
        app: WebApp,
        listen: SocketAddr,
        public: &str,
        backend: &str,
        root: &Path,
    ) -> Result<Self, ConfigError> {
        let public_url = endpoint(public, app.public_variable())?;
        let backend_url = endpoint(backend, app.backend_variable())?;
        let loopback = match public_url.host() {
            Some(url::Host::Domain("localhost")) => true,
            Some(url::Host::Ipv4(address)) => address.is_loopback(),
            Some(url::Host::Ipv6(address)) => address.is_loopback(),
            _ => false,
        };
        let path = public_url
            .path()
            .strip_suffix('/')
            .unwrap_or(public_url.path());
        if listen.port() == 0
            || (public_url.scheme() == "http" && (!loopback || !listen.ip().is_loopback()))
            || path.contains("//")
            || path.contains([';', '%', '\\'])
        {
            return Err(ConfigError::Variable(app.public_variable()));
        }
        if backend_url.path() != "/" {
            return Err(ConfigError::Variable(app.backend_variable()));
        }
        let (scope, application) = match app {
            WebApp::General => ("general", "chapchoo-web"),
            WebApp::Admin => ("admin", "chapchoo-admin-web"),
        };
        let directory = root.join("web").join(scope).join("apps").join(application);
        Ok(Self {
            app,
            listen,
            public_url,
            backend_url,
            node: root.join("runtime/bin/node"),
            script: directory.join("server.mjs"),
            directory,
        })
    }

    pub fn health_path(&self) -> String {
        let base = self
            .public_url
            .path()
            .strip_suffix('/')
            .unwrap_or(self.public_url.path());
        format!("{base}/health")
    }

    pub fn environment(&self) -> Vec<(OsString, OsString)> {
        web_environment(self, env::vars_os())
    }
}

fn web_environment(
    config: &WebConfig,
    parent: impl IntoIterator<Item = (OsString, OsString)>,
) -> Vec<(OsString, OsString)> {
    // Never inherit backend credentials, Node preload/TLS-disable settings or
    // arbitrary application environment. Additional CA certificates are public
    // trust configuration consumed through Node's supported TLS mechanism.
    let allowed = [
        "PATH",
        "HOME",
        "LANG",
        "LC_ALL",
        "TZ",
        "TMPDIR",
        "NODE_EXTRA_CA_CERTS",
    ];
    let mut environment: Vec<_> = parent
        .into_iter()
        .filter(|(name, _)| name.to_str().is_some_and(|name| allowed.contains(&name)))
        .collect();
    for (name, value) in [
        ("NODE_ENV", "production".to_owned()),
        ("NEXT_TELEMETRY_DISABLED", "1".to_owned()),
        ("TURBO_TELEMETRY_DISABLED", "1".to_owned()),
        ("HOSTNAME", config.listen.ip().to_string()),
        ("PORT", config.listen.port().to_string()),
        ("CHAPCHOO_WEB_HOST", config.listen.ip().to_string()),
        ("CHAPCHOO_PUBLIC_URL", config.public_url.to_string()),
        ("CHAPCHOO_BACKEND_URL", config.backend_url.to_string()),
    ] {
        environment.push((OsString::from(name), OsString::from(value)));
    }
    environment
}

fn client_address(mut listen: SocketAddr) -> SocketAddr {
    if listen.ip().is_unspecified() {
        listen.set_ip(match listen.ip() {
            IpAddr::V4(_) => IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
            IpAddr::V6(_) => IpAddr::V6(std::net::Ipv6Addr::LOCALHOST),
        });
    }
    listen
}

pub fn web_configs(options: &ServeOptions) -> Result<Vec<WebConfig>, AppError> {
    let Some(selection) = options.web else {
        return Ok(Vec::new());
    };
    if !cfg!(unix) {
        return Err(ConfigError::WebPlatform.into());
    }
    let executable = env::current_exe()?;
    let root = executable.parent().ok_or(ConfigError::WebArtifacts)?;
    let mut selected = Vec::new();
    for app in [WebApp::General, WebApp::Admin] {
        let (enabled, listen, public_default, backend_default) = match app {
            WebApp::General => (
                matches!(selection, WebSelection::General | WebSelection::Both),
                options.general_web_listen,
                "http://localhost:3003",
                if matches!(options.role, Role::General | Role::Both) {
                    client_address(options.general_listen)
                } else {
                    "127.0.0.1:50051".parse()?
                },
            ),
            WebApp::Admin => (
                matches!(selection, WebSelection::Admin | WebSelection::Both),
                options.admin_web_listen,
                "http://localhost:3004",
                if matches!(options.role, Role::Admin | Role::Both) {
                    client_address(options.admin_listen)
                } else {
                    "127.0.0.1:50052".parse()?
                },
            ),
        };
        if !enabled {
            continue;
        }
        let public = optional(app.public_variable())?.unwrap_or_else(|| public_default.to_owned());
        let backend = optional(app.backend_variable())?
            .unwrap_or_else(|| format!("http://{backend_default}"));
        let config = WebConfig::parse(app, listen, &public, &backend, root)?;
        if !config.node.is_file() || !config.script.is_file() {
            return Err(ConfigError::WebArtifacts.into());
        }
        selected.push(config);
    }
    Ok(selected)
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
        let key = RootKey::parse(required("CHAPCHOO_SECRET_KEY")?)
            .map_err(|_| ConfigError::Variable("CHAPCHOO_SECRET_KEY"))?;
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
    pub host: String,
    pub port: u16,
    pub security: chapchoo_mail::SmtpSecurity,
    pub credentials: Option<(String, String)>,
}

impl MailConfig {
    fn load_with(
        read: impl Fn(&'static str) -> Result<Option<String>, ConfigError>,
    ) -> Result<Option<Self>, AppError> {
        let sender = read("CHAPCHOO_SMTP_SENDER")?;
        let host = read("CHAPCHOO_SMTP_HOST")?;
        let port = read("CHAPCHOO_SMTP_PORT")?;
        let tls = read("CHAPCHOO_SMTP_TLS")?;
        let username = read("CHAPCHOO_SMTP_USERNAME")?;
        let password = read("CHAPCHOO_SMTP_PASSWORD")?;
        if [&sender, &host, &port, &tls, &username, &password]
            .iter()
            .all(|value| value.is_none())
        {
            return Ok(None);
        }
        let sender = sender.ok_or(ConfigError::Variable("CHAPCHOO_SMTP_SENDER"))?;
        chapchoo_domain::EmailAddress::parse(&sender)
            .map_err(|_| ConfigError::Variable("CHAPCHOO_SMTP_SENDER"))?;
        let host = host.ok_or(ConfigError::Variable("CHAPCHOO_SMTP_HOST"))?;
        let security = match tls.as_deref().unwrap_or("starttls") {
            "starttls" => chapchoo_mail::SmtpSecurity::StartTls,
            "implicit" => chapchoo_mail::SmtpSecurity::ImplicitTls,
            "loopback-plaintext" => chapchoo_mail::SmtpSecurity::LoopbackPlaintext,
            _ => return Err(ConfigError::Variable("CHAPCHOO_SMTP_TLS").into()),
        };
        let port = match port {
            Some(value) => value
                .parse::<u16>()
                .ok()
                .filter(|port| *port > 0)
                .ok_or(ConfigError::Variable("CHAPCHOO_SMTP_PORT"))?,
            None => match security {
                chapchoo_mail::SmtpSecurity::StartTls => 587,
                chapchoo_mail::SmtpSecurity::ImplicitTls => 465,
                chapchoo_mail::SmtpSecurity::LoopbackPlaintext => 25,
            },
        };
        let credentials = match (username, password) {
            (None, None) => None,
            (Some(username), Some(password)) if !username.is_empty() && !password.is_empty() => {
                Some((username, password))
            }
            _ => {
                return Err(ConfigError::Variable(
                    "CHAPCHOO_SMTP_USERNAME / CHAPCHOO_SMTP_PASSWORD",
                )
                .into());
            }
        };
        let config = Self {
            sender,
            host,
            port,
            security,
            credentials,
        };
        chapchoo_mail::SmtpMailer::new(
            config.sender.clone(),
            config.host.clone(),
            config.port,
            config.security,
            config.credentials.clone(),
        )?;
        Ok(Some(config))
    }
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
        let mail = MailConfig::load_with(optional)?;
        let validity_ms = duration("CHAPCHOO_SESSION_VALIDITY_DAYS", Some(180))?
            .ok_or(ConfigError::Variable("CHAPCHOO_SESSION_VALIDITY_DAYS"))?;
        Ok(Self {
            accounts: chapchoo_service::AccountsConfig {
                general_web_url: optional("CHAPCHOO_GENERAL_WEB_URL")?
                    .unwrap_or_else(|| "http://localhost:3003".to_owned()),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::HashMap, ffi::OsStr};

    fn smtp(values: &[(&'static str, &str)]) -> Result<Option<MailConfig>, AppError> {
        MailConfig::load_with(|name| {
            Ok(values
                .iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| (*value).to_owned()))
        })
    }

    #[test]
    fn smtp_is_optional_secure_and_rejects_partial_or_invalid_configuration() -> Result<(), AppError>
    {
        assert!(smtp(&[])?.is_none());
        let base = [
            ("CHAPCHOO_SMTP_SENDER", "sender@example.invalid"),
            ("CHAPCHOO_SMTP_HOST", "smtp.example.invalid"),
        ];
        let config = smtp(&base)?.ok_or("Expected SMTP config")?;
        assert_eq!(config.port, 587);
        assert_eq!(config.security, chapchoo_mail::SmtpSecurity::StartTls);
        assert!(config.credentials.is_none());
        for (name, value) in [
            ("CHAPCHOO_SMTP_PORT", "0"),
            ("CHAPCHOO_SMTP_PORT", "65536"),
            ("CHAPCHOO_SMTP_PORT", "-1"),
            ("CHAPCHOO_SMTP_TLS", "opportunistic"),
            ("CHAPCHOO_SMTP_TLS", "loopback-plaintext"),
            ("CHAPCHOO_SMTP_USERNAME", "fixture-user"),
            ("CHAPCHOO_SMTP_PASSWORD", "fixture-password"),
        ] {
            let mut values = base.to_vec();
            values.push((name, value));
            assert!(smtp(&values).is_err());
        }
        assert!(smtp(&base[..1]).is_err());
        let mut implicit = base.to_vec();
        implicit.push(("CHAPCHOO_SMTP_TLS", "implicit"));
        assert_eq!(smtp(&implicit)?.ok_or("Expected SMTP config")?.port, 465);
        implicit.extend([
            ("CHAPCHOO_SMTP_USERNAME", "fixture-user"),
            ("CHAPCHOO_SMTP_PASSWORD", "fixture-password"),
        ]);
        let authenticated = smtp(&implicit)?.ok_or("Expected SMTP config")?;
        assert!(authenticated.credentials.is_some());
        assert!(!format!("{authenticated:?}").contains("fixture-password"));
        Ok(())
    }

    fn web(public: &str, listen: &str, backend: &str) -> Result<WebConfig, AppError> {
        Ok(WebConfig::parse(
            WebApp::General,
            listen.parse()?,
            public,
            backend,
            Path::new("/owned-bundle"),
        )?)
    }

    #[test]
    fn web_transport_matches_actual_listener_and_compiled_path_policy() -> Result<(), AppError> {
        for (public, listen) in [
            ("http://localhost:3003", "127.0.0.1:3003"),
            ("http://127.0.0.2:3003/app", "127.0.0.1:3003"),
            ("http://[::1]:3003/app/", "[::1]:3003"),
            ("https://web.example.invalid/app", "0.0.0.0:3003"),
        ] {
            let config = web(public, listen, "https://rpc.example.invalid")?;
            assert!(config.node.ends_with("runtime/bin/node"));
            assert!(
                config
                    .script
                    .ends_with("web/general/apps/chapchoo-web/server.mjs")
            );
        }
        assert_eq!(
            web(
                "http://localhost:3003/app/",
                "127.0.0.1:3003",
                "http://127.0.0.1:50051"
            )?
            .health_path(),
            "/app/health"
        );
        Ok(())
    }

    #[test]
    fn unsafe_web_origins_binds_and_client_endpoints_fail_before_launch() {
        for (public, listen, backend) in [
            (
                "http://example.invalid",
                "127.0.0.1:3003",
                "http://localhost:50051",
            ),
            (
                "http://localhost:3003",
                "0.0.0.0:3003",
                "http://localhost:50051",
            ),
            (
                "http://localhost:3003",
                "127.0.0.1:0",
                "http://localhost:50051",
            ),
            (
                "https://web.example.invalid/a//b",
                "0.0.0.0:3003",
                "http://localhost:50051",
            ),
            (
                "https://web.example.invalid/%2f",
                "0.0.0.0:3003",
                "http://localhost:50051",
            ),
            (
                "https://web.example.invalid/a;b",
                "0.0.0.0:3003",
                "http://localhost:50051",
            ),
            (
                "https://user:secret@web.example.invalid",
                "0.0.0.0:3003",
                "http://localhost:50051",
            ),
            (
                "https://web.example.invalid?key=value",
                "0.0.0.0:3003",
                "http://localhost:50051",
            ),
            (
                "https://web.example.invalid",
                "0.0.0.0:3003",
                "http://rpc.example.invalid/path",
            ),
            (
                "https://web.example.invalid",
                "0.0.0.0:3003",
                "http://user:secret@rpc.example.invalid",
            ),
        ] {
            assert!(web(public, listen, backend).is_err());
        }
    }

    #[test]
    fn web_environment_does_not_inherit_backend_or_preload_authority() -> Result<(), AppError> {
        let config = web(
            "http://localhost:3003",
            "127.0.0.1:3003",
            "http://127.0.0.1:50051",
        )?;
        let parent = [
            ("PATH", "/usr/bin:/bin"),
            ("HOME", "/owned-home"),
            ("NODE_EXTRA_CA_CERTS", "/public-certs/fixture-ca.pem"),
            ("CHAPCHOO_POSTGRES_URL", "database-canary"),
            ("CHAPCHOO_SECRET_KEY", "key-canary"),
            ("CHAPCHOO_SETUP_PROOF", "proof-canary"),
            ("AWS_SECRET_ACCESS_KEY", "aws-canary"),
            ("CHAPCHOO_SMTP_PASSWORD", "smtp-canary"),
            ("NODE_OPTIONS", "--require untrusted-preload"),
            ("NODE_TLS_REJECT_UNAUTHORIZED", "0"),
            ("LD_PRELOAD", "untrusted-native-preload"),
            ("HOSTNAME", "untrusted-inherited-bind"),
        ]
        .map(|(name, value)| (OsString::from(name), OsString::from(value)));
        let environment: HashMap<_, _> = web_environment(&config, parent).into_iter().collect();
        for name in [
            "CHAPCHOO_POSTGRES_URL",
            "CHAPCHOO_SECRET_KEY",
            "CHAPCHOO_SETUP_PROOF",
            "AWS_SECRET_ACCESS_KEY",
            "CHAPCHOO_SMTP_PASSWORD",
            "NODE_OPTIONS",
            "NODE_TLS_REJECT_UNAUTHORIZED",
            "LD_PRELOAD",
        ] {
            assert!(!environment.contains_key(OsStr::new(name)));
        }
        assert_eq!(
            environment.get(OsStr::new("HOSTNAME")),
            Some(&OsString::from("127.0.0.1"))
        );
        assert_eq!(
            environment.get(OsStr::new("PORT")),
            Some(&OsString::from("3003"))
        );
        assert!(environment.contains_key(OsStr::new("NODE_EXTRA_CA_CERTS")));
        Ok(())
    }

    #[test]
    fn cli_retains_backend_selection_and_explicitly_admits_web_only() -> Result<(), AppError> {
        let cli = Cli::try_parse_from(["chapchoo", "serve", "--role", "none", "--web", "general"])?;
        match cli.command {
            Command::Serve(options) => {
                assert_eq!(options.role, Role::None);
                assert_eq!(options.web, Some(WebSelection::General));
            }
            _ => return Err("Expected selected component command".into()),
        }
        assert!(Cli::try_parse_from(["chapchoo", "serve", "--web", "unknown"]).is_err());
        Ok(())
    }

    #[test]
    fn wildcard_backend_clients_use_loopback_without_rewriting_specific_binds()
    -> Result<(), AppError> {
        assert_eq!(
            client_address("0.0.0.0:50051".parse()?),
            "127.0.0.1:50051".parse()?
        );
        assert_eq!(
            client_address("[::]:50051".parse()?),
            "[::1]:50051".parse()?
        );
        assert_eq!(
            client_address("192.0.2.1:50051".parse()?),
            "192.0.2.1:50051".parse()?
        );
        Ok(())
    }
}
