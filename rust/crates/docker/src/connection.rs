//! Resolve once from the executing account's CLI configuration or one complete
//! explicit source. This module performs no daemon IO and never changes context.
use crate::Error;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    fmt,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};
use zeroize::Zeroizing;

const FILE_LIMIT: u64 = 1024 * 1024;
mod proxy;

/// Captured configuration inputs, injectable without mutating process globals.
#[derive(Clone)]
pub struct Environment {
    pub config_dir: PathBuf,
    pub host: Option<String>,
    pub context: Option<String>,
    pub tls: bool,
    pub tls_verify: bool,
    pub cert_dir: Option<PathBuf>,
    pub api_version: Option<String>,
}
impl Environment {
    pub fn capture() -> Result<Self, Error> {
        fn text(name: &str) -> Result<Option<String>, Error> {
            std::env::var_os(name)
                .filter(|value| !value.is_empty())
                .map(|value| value.into_string().map_err(|_| Error::Configuration))
                .transpose()
        }
        let config_dir = match std::env::var_os("DOCKER_CONFIG").filter(|v| !v.is_empty()) {
            Some(directory) => PathBuf::from(directory),
            None => PathBuf::from(
                std::env::var_os("HOME")
                    .filter(|v| !v.is_empty())
                    .ok_or(Error::Configuration)?,
            )
            .join(".docker"),
        };
        Ok(Self {
            config_dir,
            host: text("DOCKER_HOST")?,
            context: text("DOCKER_CONTEXT")?,
            api_version: text("DOCKER_API_VERSION")?,
            tls: text("DOCKER_TLS")?.is_some(),
            tls_verify: text("DOCKER_TLS_VERIFY")?.is_some(),
            cert_dir: std::env::var_os("DOCKER_CERT_PATH")
                .filter(|v| !v.is_empty())
                .map(PathBuf::from),
        })
    }
}
impl fmt::Debug for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DockerEnvironment([REDACTED])")
    }
}

#[derive(Clone)]
pub enum Endpoint {
    Unix(PathBuf),
    Tcp {
        host: String,
        port: u16,
    },
    Ssh {
        host: String,
        user: Option<String>,
        port: Option<u16>,
        socket: Option<String>,
    },
}
impl fmt::Debug for Endpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DockerEndpoint([REDACTED])")
    }
}

/// Material is captured at startup, not reread on stream/key renewal.
#[derive(Clone)]
pub struct Tls {
    pub skip_verify: bool,
    pub ca: Option<Zeroizing<Vec<u8>>>,
    pub certificate: Option<Zeroizing<Vec<u8>>>,
    pub key: Option<Zeroizing<Vec<u8>>>,
}
impl fmt::Debug for Tls {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DockerTls([REDACTED])")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApiVersion {
    pub(crate) major: usize,
    pub(crate) minor: usize,
}
impl ApiVersion {
    pub fn parse(value: &str) -> Result<Self, Error> {
        let (major, minor) = value.split_once('.').ok_or(Error::Configuration)?;
        if value.len() > 16
            || major.is_empty()
            || minor.is_empty()
            || !major
                .bytes()
                .chain(minor.bytes())
                .all(|byte| byte.is_ascii_digit())
        {
            return Err(Error::Configuration);
        }
        let major = major.parse().map_err(|_| Error::Configuration)?;
        let minor = minor.parse().map_err(|_| Error::Configuration)?;
        if major == 0 {
            return Err(Error::Configuration);
        }
        Ok(Self { major, minor })
    }
}

#[derive(Debug, Clone)]
pub struct Selection {
    pub endpoint: Endpoint,
    pub tls: Option<Tls>,
    pub api_version: Option<ApiVersion>,
}
impl Selection {
    pub fn from_process(explicit: Option<Explicit>) -> Result<Self, Error> {
        let selected = match explicit {
            Some(explicit) => Self::explicit(explicit),
            None => resolve(&Environment::capture()?, None),
        }?;
        proxy::check(selected)
    }
    fn explicit(explicit: Explicit) -> Result<Self, Error> {
        validate_tls(&explicit.host, explicit.tls)?
            .with_api_version(explicit.api_version.as_deref())
    }
    fn with_api_version(mut self, value: Option<&str>) -> Result<Self, Error> {
        self.api_version = value.map(ApiVersion::parse).transpose()?;
        Ok(self)
    }
}

/// Complete override: no ambient context, TLS material or host is borrowed.
#[derive(Clone)]
pub struct Explicit {
    pub host: String,
    pub tls: Option<Tls>,
    pub api_version: Option<String>,
}
impl fmt::Debug for Explicit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DockerExplicit([REDACTED])")
    }
}

impl Explicit {
    pub fn from_files(
        host: String,
        tls: bool,
        skip_verify: bool,
        ca: Option<&Path>,
        certificate: Option<&Path>,
        key: Option<&Path>,
        api_version: Option<String>,
    ) -> Result<Self, Error> {
        if !tls && (skip_verify || ca.is_some() || certificate.is_some() || key.is_some()) {
            return Err(Error::Configuration);
        }
        let material = if tls {
            Some(Tls {
                skip_verify,
                ca: ca
                    .map(|path| read(path)?.ok_or(Error::Configuration))
                    .transpose()?,
                certificate: certificate
                    .map(|path| read(path)?.ok_or(Error::Configuration))
                    .transpose()?,
                key: key
                    .map(|path| read(path)?.ok_or(Error::Configuration))
                    .transpose()?,
            })
        } else {
            None
        };
        Ok(Self {
            host,
            tls: material,
            api_version,
        })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CliConfig {
    #[serde(default)]
    current_context: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Context {
    name: String,
    endpoints: ContextEndpoints,
}
#[derive(Deserialize)]
struct ContextEndpoints {
    docker: ContextEndpoint,
}
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ContextEndpoint {
    host: String,
    #[serde(default, rename = "SkipTLSVerify")]
    skip_tls_verify: bool,
}

fn read(path: &Path) -> Result<Option<Zeroizing<Vec<u8>>>, Error> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(Error::Configuration),
    };
    let mut bytes = Zeroizing::new(Vec::new());
    file.take(FILE_LIMIT + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| Error::Configuration)?;
    if bytes.len() as u64 > FILE_LIMIT {
        return Err(Error::Configuration);
    }
    Ok(Some(bytes))
}

fn decode(value: &str) -> Result<String, Error> {
    let decoded = percent_encoding::percent_decode_str(value)
        .decode_utf8()
        .map_err(|_| Error::Configuration)?
        .into_owned();
    if decoded.chars().any(char::is_control) {
        return Err(Error::Configuration);
    }
    Ok(decoded)
}

fn host(url: &url::Url) -> Result<String, Error> {
    let host = match url.host().ok_or(Error::Configuration)? {
        url::Host::Ipv6(address) => address.to_string(),
        value => value.to_string(),
    };
    if host.is_empty() || host.starts_with('-') {
        return Err(Error::Configuration);
    }
    Ok(host)
}

fn endpoint(value: &str) -> Result<Endpoint, Error> {
    let url = url::Url::parse(value).map_err(|_| Error::Configuration)?;
    if url.password().is_some() || url.query().is_some() || url.fragment().is_some() {
        return Err(Error::Configuration);
    }
    match url.scheme() {
        "unix"
            if url.host_str().is_none()
                && url.username().is_empty()
                && url.path().starts_with('/')
                && url.path() != "/" =>
        {
            // The URL path is encoded; transport must not target a literal %20.
            let path = url
                .to_file_path()
                .or_else(|_| {
                    let file =
                        url::Url::parse(&value.replacen("unix:", "file:", 1)).map_err(|_| ())?;
                    file.to_file_path()
                })
                .map_err(|_| Error::Configuration)?;
            Ok(Endpoint::Unix(path))
        }
        "tcp" if url.username().is_empty() && matches!(url.path(), "" | "/") => Ok(Endpoint::Tcp {
            host: host(&url)?,
            port: url.port().unwrap_or(2375),
        }),
        "ssh" => {
            let host = host(&url)?;
            let socket = match url.path() {
                "" | "/" => None,
                path if path.starts_with('/') && !path.chars().any(char::is_control) => {
                    Some(decode(path)?)
                }
                _ => return Err(Error::Configuration),
            };
            Ok(Endpoint::Ssh {
                host,
                user: (!url.username().is_empty())
                    .then(|| decode(url.username()))
                    .transpose()?,
                port: url.port(),
                socket,
            })
        }
        "unix" | "tcp" => Err(Error::Configuration),
        _ => Err(Error::UnsupportedTransport),
    }
}

fn validate_tls(value: &str, tls: Option<Tls>) -> Result<Selection, Error> {
    let mut endpoint = endpoint(value)?;
    if let Some(tls) = &tls
        && tls.certificate.is_some() != tls.key.is_some()
    {
        return Err(Error::Configuration);
    }
    // CLI socket and SSH helpers carry their own transport. Context TLS files
    // cannot silently turn either stream into a different TCP target.
    if let Endpoint::Tcp { port, .. } = &mut endpoint
        && tls.is_some()
        && url::Url::parse(value)
            .map_err(|_| Error::Configuration)?
            .port()
            .is_none()
    {
        *port = 2376;
    }
    let tls = if matches!(endpoint, Endpoint::Tcp { .. }) {
        tls
    } else {
        None
    };
    Ok(Selection {
        endpoint,
        tls,
        api_version: None,
    })
}

/// Match CLI source selection; all selected-file failures are terminal.
pub fn resolve(env: &Environment, explicit: Option<Explicit>) -> Result<Selection, Error> {
    if let Some(explicit) = explicit {
        return Selection::explicit(explicit);
    }
    // Docker CLI's resolveContextName checks DOCKER_HOST before DOCKER_CONTEXT.
    // Explicit --context is a different input; the Provider's explicit override
    // above is coherent and bypasses both ambient sources.
    let context = if env.host.as_ref().is_some_and(|v| !v.is_empty()) {
        "default".to_owned()
    } else if let Some(context) = env.context.as_ref().filter(|v| !v.is_empty()) {
        context.clone()
    } else {
        read(&env.config_dir.join("config.json"))?
            .map(|bytes| {
                serde_json::from_slice::<CliConfig>(&bytes).map_err(|_| Error::Configuration)
            })
            .transpose()?
            .map(|config| config.current_context)
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| "default".into())
    };
    if context != "default" {
        let digest = format!("{:x}", Sha256::digest(context.as_bytes()));
        let bytes = read(
            &env.config_dir
                .join("contexts/meta")
                .join(&digest)
                .join("meta.json"),
        )?
        .ok_or(Error::Configuration)?;
        let metadata: Context = serde_json::from_slice(&bytes).map_err(|_| Error::Configuration)?;
        if metadata.name != context {
            return Err(Error::Configuration);
        }
        let selected = &metadata.endpoints.docker;
        let tls_dir = env
            .config_dir
            .join("contexts/tls")
            .join(&digest)
            .join("docker");
        let ca = read(&tls_dir.join("ca.pem"))?;
        let certificate = read(&tls_dir.join("cert.pem"))?;
        let key = read(&tls_dir.join("key.pem"))?;
        let tls =
            if selected.skip_tls_verify || ca.is_some() || certificate.is_some() || key.is_some() {
                Some(Tls {
                    skip_verify: selected.skip_tls_verify,
                    ca,
                    certificate,
                    key,
                })
            } else {
                None
            };
        return validate_tls(&selected.host, tls)?.with_api_version(env.api_version.as_deref());
    }
    let tls = if env.tls || env.tls_verify {
        let directory = env.cert_dir.as_ref().unwrap_or(&env.config_dir);
        // Like the CLI, TLS defaults require the selected CA; absent optional
        // client certificate/key files do not borrow another context's material.
        Some(Tls {
            skip_verify: !env.tls_verify,
            ca: Some(read(&directory.join("ca.pem"))?.ok_or(Error::Configuration)?),
            certificate: read(&directory.join("cert.pem"))?,
            key: read(&directory.join("key.pem"))?,
        })
    } else {
        None
    };
    let default = if tls.is_some() {
        "tcp://localhost:2376"
    } else {
        "unix:///var/run/docker.sock"
    };
    validate_tls(
        env.host
            .as_deref()
            .filter(|v| !v.is_empty())
            .unwrap_or(default),
        tls,
    )?
    .with_api_version(env.api_version.as_deref())
}

#[cfg(test)]
mod tests;
