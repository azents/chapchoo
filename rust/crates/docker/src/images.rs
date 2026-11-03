//! Public-image cache policy and existing network prerequisites. No registry
//! credential loading, architecture emulation or operator-network mutation.
use crate::{Docker, Error, ImagePolicy, volumes::not_found};
use bollard::query_parameters::{CreateImageOptionsBuilder, InspectNetworkOptions};
use futures_util::TryStreamExt;
use std::time::Duration;

#[derive(Debug)]
pub struct PreparedImage {
    pub(crate) configured: String,
    pub(crate) digest: String,
    pub(crate) daemon: chapchoo_domain::DockerDaemonId,
}
pub(crate) fn image_digest(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}
impl Docker {
    pub async fn prepare_image(&self) -> Result<PreparedImage, Error> {
        self.verify_daemon().await?;
        let exists = match tokio::time::timeout(
            Duration::from_secs(10),
            self.sdk.inspect_image(&self.config.image),
        )
        .await
        .map_err(|_| Error::Transport)?
        {
            Ok(_) => true,
            Err(error) if not_found(&error) => false,
            Err(error) => return Err(error.into()),
        };
        if self.config.image_policy == ImagePolicy::Always
            || (!exists && self.config.image_policy == ImagePolicy::IfNotPresent)
        {
            let sdk = self.sdk.clone().with_timeout(Duration::from_secs(300));
            let mut stream = sdk.create_image(
                Some(
                    CreateImageOptionsBuilder::default()
                        .from_image(&self.config.image)
                        .platform(&format!("linux/{}", self.architecture))
                        .build(),
                ),
                None,
                None,
            );
            tokio::time::timeout(Duration::from_secs(300), async {
                while let Some(value) = stream.try_next().await? {
                    if value.error_detail.is_some() {
                        return Err(Error::Uncertain);
                    }
                }
                Ok::<_, Error>(())
            })
            .await
            .map_err(|_| Error::Uncertain)??;
        } else if !exists {
            return Err(Error::Configuration);
        }
        let image = tokio::time::timeout(
            Duration::from_secs(10),
            self.sdk.inspect_image(&self.config.image),
        )
        .await
        .map_err(|_| Error::Transport)??;
        let architecture = match image.architecture.as_deref() {
            Some("amd64" | "x86_64") => "amd64",
            Some("arm64" | "aarch64") => "arm64",
            _ => return Err(Error::WorkloadPolicy),
        };
        if image.os.as_deref() != Some("linux")
            || architecture != self.architecture
            || image.id.as_deref().is_none_or(|value| !image_digest(value))
            || image
                .config
                .as_ref()
                .and_then(|value| value.volumes.as_ref())
                .is_some_and(|values| !values.is_empty())
        {
            return Err(Error::WorkloadPolicy);
        }
        let network = self.config.network.as_deref().unwrap_or("bridge");
        let actual = tokio::time::timeout(
            Duration::from_secs(10),
            self.sdk
                .inspect_network(network, None::<InspectNetworkOptions>),
        )
        .await
        .map_err(|_| Error::Transport)??;
        if actual.name.as_deref() != Some(network)
            || matches!(actual.driver.as_deref(), None | Some("host" | "null"))
            || actual.internal == Some(true)
            || actual.config_only == Some(true)
        {
            return Err(Error::Configuration);
        }
        Ok(PreparedImage {
            configured: self.config.image.clone(),
            digest: image.id.ok_or(Error::WorkloadPolicy)?,
            daemon: self.daemon.clone(),
        })
    }
}
