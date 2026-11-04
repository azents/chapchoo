//! Provider evidence is reread for every exchange; no operational key can renew itself.
use crate::{AppError, config::ConfigError};
use chapchoo_protocol::control::{self as wire, provider_authentication_request::Evidence};
use std::{fmt, path::PathBuf, sync::Arc};
use tokio::io::AsyncReadExt;
use tonic::Status;
use zeroize::Zeroizing;

#[derive(Debug, Clone, Copy)]
pub enum Method {
    ServiceAccount,
    SecretKey,
    RandomToken,
}
#[derive(Clone)]
pub enum ProviderAuth {
    File {
        method: Method,
        path: PathBuf,
    },
    Internal,
    #[cfg(test)]
    Fixture(Arc<Zeroizing<String>>),
}
impl fmt::Debug for ProviderAuth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ProviderAuth([REDACTED])")
    }
}
impl ProviderAuth {
    pub fn from_process() -> Result<Self, AppError> {
        let method = match std::env::var("CHAPCHOO_PROVIDER_AUTH_METHOD").as_deref() {
            Ok("service-account") => Method::ServiceAccount,
            Ok("secret-key") => Method::SecretKey,
            Ok("random-token") => Method::RandomToken,
            _ => return Err(ConfigError::Variable("CHAPCHOO_PROVIDER_AUTH_METHOD").into()),
        };
        let path = PathBuf::from(
            std::env::var_os("CHAPCHOO_PROVIDER_AUTH_FILE")
                .ok_or(ConfigError::Variable("CHAPCHOO_PROVIDER_AUTH_FILE"))?,
        );
        if !path.is_absolute() {
            return Err(ConfigError::Variable("CHAPCHOO_PROVIDER_AUTH_FILE").into());
        }
        Ok(Self::File { method, path })
    }
    pub async fn evidence(&self) -> Result<Option<Evidence>, AppError> {
        let (method, token) = match self {
            Self::Internal => return Ok(None),
            Self::File { method, path } => {
                let metadata = tokio::fs::metadata(path).await?;
                if !metadata.is_file() {
                    return Err("Provider authentication source must be a regular file".into());
                }
                let file = tokio::fs::File::open(path).await?;
                let mut content = Zeroizing::new(String::new());
                file.take(16_385).read_to_string(&mut content).await?;
                if content.len() > 16_384 {
                    return Err("Provider authentication file is too large".into());
                }
                (
                    *method,
                    Zeroizing::new(content.trim_end_matches(['\n', '\r']).to_owned()),
                )
            }
            #[cfg(test)]
            Self::Fixture(value) => (Method::SecretKey, Zeroizing::new(value.as_str().to_owned())),
        };
        if token.is_empty() || token.len() > 16_384 {
            return Err("Provider authentication file is invalid".into());
        }
        Ok(Some(match method {
            Method::ServiceAccount => Evidence::ServiceAccountToken(token.to_string()),
            Method::SecretKey => Evidence::BootstrapCredential(token.to_string()),
            Method::RandomToken => Evidence::RegistrationToken(token.to_string()),
        }))
    }
    #[cfg(test)]
    pub fn fixture(value: String) -> Self {
        Self::Fixture(Arc::new(Zeroizing::new(value)))
    }
}

#[derive(Debug)]
struct KubernetesReviewer(chapchoo_kubernetes::authentication::ServiceAccountReviewer);
#[tonic::async_trait]
impl chapchoo_rpc::ServiceAccountReviewer for KubernetesReviewer {
    async fn review(
        &self,
        token: &str,
        audience: &str,
    ) -> Result<chapchoo_rpc::ReviewedServiceAccount, Status> {
        match self.0.review(token, audience).await {
            Ok(Some((username, audiences))) => Ok(chapchoo_rpc::ReviewedServiceAccount {
                username,
                audiences,
            }),
            Ok(None) => Err(Status::permission_denied(
                "ServiceAccount authentication denied",
            )),
            Err(_) => Err(Status::unavailable(
                "ServiceAccount verification unavailable",
            )),
        }
    }
}
pub fn authenticator(
    config: chapchoo_rpc::ProviderBootstrapConfig,
    cipher: Arc<chapchoo_crypto::CredentialCipher>,
) -> Result<Arc<chapchoo_rpc::ProviderAuthenticator>, AppError> {
    let reviewer: Option<Arc<dyn chapchoo_rpc::ServiceAccountReviewer>> =
        if config.service_accounts.is_empty() {
            None
        } else {
            Some(Arc::new(KubernetesReviewer(
                chapchoo_kubernetes::authentication::ServiceAccountReviewer::in_cluster()?,
            )))
        };
    Ok(Arc::new(chapchoo_rpc::ProviderAuthenticator::new(
        config, cipher, reviewer,
    )?))
}

pub fn configuration(
    provider: &str,
    name: &str,
    kind: wire::ProviderKind,
    daemon: Option<String>,
) -> wire::ProviderConfiguration {
    wire::ProviderConfiguration {
        provider_id: provider.into(),
        name: name.into(),
        kind: kind as i32,
        docker_daemon_id: daemon,
        capabilities: Some(wire::ProviderCapabilities {
            create: true,
            start: true,
            stop: true,
            restart: true,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{Engine, engine::general_purpose::URL_SAFE};

    #[tokio::test]
    async fn authentication_file_is_reread_and_bounded() -> Result<(), AppError> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("evidence");
        let source = ProviderAuth::File {
            method: Method::ServiceAccount,
            path: path.clone(),
        };
        tokio::fs::write(&path, "first-token\n").await?;
        assert!(
            matches!(source.evidence().await?,Some(Evidence::ServiceAccountToken(value)) if value=="first-token")
        );
        tokio::fs::write(&path, "rotated-token").await?;
        assert!(
            matches!(source.evidence().await?,Some(Evidence::ServiceAccountToken(value)) if value=="rotated-token")
        );
        tokio::fs::write(&path, "x".repeat(16_385)).await?;
        assert!(source.evidence().await.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn private_trust_exchange_is_identity_bound_and_external_adapter_denies()
    -> Result<(), AppError> {
        let dir = tempfile::tempdir()?;
        let cipher = Arc::new(chapchoo_crypto::CredentialCipher::new(
            chapchoo_crypto::RootKey::parse(URL_SAFE.encode([84; 32]))?,
        )?);
        let path = dir.path().join("standalone.sqlite");
        chapchoo_persistence::prepare_sqlite(&path, &cipher)?;
        let store = chapchoo_persistence::Store::connect_sqlite(path, cipher.clone()).await?;
        let service = Arc::new(chapchoo_service::ComputersService::new(store));
        let control = service.register_control("http://chapchoo.memory").await?;
        let connections = chapchoo_rpc::ComponentConnectionServices::new(
            service.clone(),
            control.id.clone(),
            Arc::new(std::sync::atomic::AtomicBool::new(true)),
        );
        let config = chapchoo_rpc::ProviderBootstrapConfig {
            trust_provider_ids: vec!["local".into()],
            ..Default::default()
        };
        let auth = chapchoo_rpc::ComponentAuthenticationServices::new(
            service.clone(),
            control.id.clone(),
            connections.clone(),
        )
        .with_provider_authenticator(authenticator(config, cipher)?);
        let (channel, incoming) = chapchoo_rpc::memory_transport();
        let stop = tokio_util::sync::CancellationToken::new();
        let shutdown = stop.clone();
        let server = auth.internal_provider_server("local".into());
        let task = tokio::spawn(async move {
            tonic::transport::Server::builder()
                .add_service(server)
                .serve_with_incoming_shutdown(incoming, shutdown.cancelled_owned())
                .await
        });
        let primary = async {
            let mut client =
                wire::provider_authentication_client::ProviderAuthenticationClient::new(channel);
            let key = client
                .exchange(wire::ProviderAuthenticationRequest {
                    configuration: Some(configuration(
                        "local",
                        "Local",
                        wire::ProviderKind::Kubernetes,
                        None,
                    )),
                    request_id: format!(
                        "{}:local-exchange",
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)?
                            .as_millis()
                    ),
                    evidence: None,
                })
                .await?
                .into_inner();
            assert_eq!(
                key.expires_at.as_ref().ok_or("No expiry")?.seconds
                    - key.issued_at.as_ref().ok_or("No issue time")?.seconds,
                900
            );
            let wrong = client
                .exchange(wire::ProviderAuthenticationRequest {
                    configuration: Some(configuration(
                        "foreign",
                        "Local",
                        wire::ProviderKind::Kubernetes,
                        None,
                    )),
                    request_id: "foreign".into(),
                    evidence: None,
                })
                .await;
            assert_eq!(
                wrong.err().ok_or("Foreign Trust accepted")?.code(),
                tonic::Code::PermissionDenied
            );
            Ok::<(), AppError>(())
        }
        .await;
        stop.cancel();
        let cleanup = task.await?.map_err(Into::into);
        crate::completion::complete(primary, cleanup)?;
        let (channel, incoming) = chapchoo_rpc::memory_transport();
        let external = auth.provider_server();
        let stop = tokio_util::sync::CancellationToken::new();
        let shutdown = stop.clone();
        let task = tokio::spawn(async move {
            tonic::transport::Server::builder()
                .add_service(external)
                .serve_with_incoming_shutdown(incoming, shutdown.cancelled_owned())
                .await
        });
        let mut client =
            wire::provider_authentication_client::ProviderAuthenticationClient::new(channel);
        let denied = client
            .exchange(wire::ProviderAuthenticationRequest {
                configuration: Some(configuration(
                    "local",
                    "Local",
                    wire::ProviderKind::Kubernetes,
                    None,
                )),
                request_id: "external".into(),
                evidence: None,
            })
            .await;
        stop.cancel();
        task.await??;
        assert_eq!(
            denied.err().ok_or("External Trust accepted")?.code(),
            tonic::Code::PermissionDenied
        );
        service.drain_control(&control.id).await?;
        connections.drain().await?;
        Ok(())
    }
}
