//! Native daemon identity and trusted workload policy. Authorization and product
//! persistence are supplied by the current authenticated Control connection.
use crate::{Error, connection::Selection, transport::Transport};
use bollard::{Docker as Engine, models::SystemInfo};
use chapchoo_domain::DockerDaemonId;
use std::{
    fmt,
    path::{Component, Path},
    time::Duration,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImagePolicy {
    Always,
    IfNotPresent,
    Never,
}
#[derive(Clone)]
pub struct Config {
    pub image: String,
    pub image_policy: ImagePolicy,
    pub saved_mount: String,
    pub network: Option<String>,
    pub nano_cpus: i64,
    pub memory_bytes: i64,
}
impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DockerConfig([REDACTED])")
    }
}
impl Config {
    pub fn validate(&self) -> Result<(), Error> {
        let mount = Path::new(&self.saved_mount);
        if self.image.is_empty()
            || self.image.len() > 1024
            || self
                .image
                .chars()
                .any(|c| c.is_whitespace() || c.is_control())
            || !mount.is_absolute()
            || mount == Path::new("/")
            || mount
                .components()
                .any(|c| matches!(c, Component::ParentDir | Component::CurDir))
            || self.saved_mount.chars().any(char::is_control)
            || self.saved_mount.ends_with('/')
            || self
                .saved_mount
                .split('/')
                .any(|part| matches!(part, "." | ".."))
            || self.nano_cpus <= 0
            || self.memory_bytes <= 0
            || self.network.as_ref().is_some_and(|value| {
                value.is_empty()
                    || value.len() > 255
                    || value == "host"
                    || value.starts_with("container:")
                    || value.chars().any(|c| c.is_whitespace() || c.is_control())
            })
        {
            return Err(Error::Configuration);
        }
        Ok(())
    }
}

pub struct Docker {
    pub(crate) sdk: Engine,
    pub(crate) transport: Transport,
    pub(crate) provider: String,
    pub(crate) daemon: DockerDaemonId,
    pub(crate) architecture: String,
    pub(crate) config: Config,
    pub(crate) progress: crate::containers::Progress,
}
impl fmt::Debug for Docker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DockerAdapter([REDACTED])")
    }
}

fn architecture(info: &SystemInfo) -> Result<&'static str, Error> {
    match info.architecture.as_deref() {
        Some("x86_64" | "amd64") => Ok("amd64"),
        Some("aarch64" | "arm64") => Ok("arm64"),
        _ => Err(Error::WorkloadPolicy),
    }
}
fn identity(info: &SystemInfo) -> Result<DockerDaemonId, Error> {
    DockerDaemonId::parse(info.id.clone().ok_or(Error::DaemonIdentity)?)
        .map_err(|_| Error::DaemonIdentity)
}
fn workload_policy(info: &SystemInfo) -> Result<(), Error> {
    if info.os_type.as_deref() != Some("linux")
        || info.memory_limit != Some(true)
        || info.cpu_cfs_period != Some(true)
        || info.cpu_cfs_quota != Some(true)
        || !info.security_options.as_ref().is_some_and(|values| {
            values.iter().any(|value| {
                let mut fields = value.split(',');
                fields.next() == Some("name=seccomp")
                    && !fields
                        .any(|field| matches!(field, "profile=unconfined" | "profile=disabled"))
            })
        })
    {
        return Err(Error::WorkloadPolicy);
    }
    architecture(info)?;
    Ok(())
}
impl Docker {
    pub async fn connect(
        provider: String,
        selected: Selection,
        config: Config,
    ) -> Result<Self, Error> {
        config.validate()?;
        if provider.is_empty()
            || provider.len() > 128
            || provider
                .chars()
                .any(|c| c.is_control() || c.is_whitespace())
        {
            return Err(Error::Configuration);
        }
        let negotiate = selected.api_version.is_none();
        let transport = Transport::new(selected).await?;
        let sdk = transport.sdk()?.with_timeout(Duration::from_secs(10));
        let sdk = if negotiate {
            tokio::time::timeout(Duration::from_secs(10), sdk.negotiate_version())
                .await
                .map_err(|_| Error::Transport)??
        } else {
            sdk
        };
        let info = tokio::time::timeout(Duration::from_secs(10), sdk.info())
            .await
            .map_err(|_| Error::Transport)??;
        let daemon = identity(&info)?;
        workload_policy(&info)?;
        let architecture = architecture(&info)?.to_owned();
        Ok(Self {
            sdk,
            transport,
            provider,
            daemon,
            architecture,
            config,
            progress: Default::default(),
        })
    }

    pub fn daemon_id(&self) -> &DockerDaemonId {
        &self.daemon
    }
    pub fn provider_id(&self) -> &str {
        &self.provider
    }
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Comparison evidence, not an atomic daemon/effect attestation. Operators
    /// must supply one stable trusted endpoint for this process lifetime.
    pub async fn verify_daemon(&self) -> Result<(), Error> {
        let info = tokio::time::timeout(Duration::from_secs(10), self.sdk.info())
            .await
            .map_err(|_| Error::Transport)??;
        if identity(&info)? != self.daemon || architecture(&info)? != self.architecture {
            return Err(Error::DaemonIdentity);
        }
        workload_policy(&info)
    }

    pub async fn drain(&self) -> Result<(), Error> {
        self.transport.drain().await
    }
}

#[cfg(test)]
mod tests;
