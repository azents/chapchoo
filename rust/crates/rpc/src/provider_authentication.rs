//! Bootstrap evidence adapters. Only captured server policy selects trust and verifiers.
#[cfg(test)]
mod tests;
use chapchoo_crypto::{
    BearerSecret, BootstrapAuthenticationSource, BootstrapProviderPermit, CredentialCipher,
    ProviderBootstrapCredential, VerifiedSystemOwner,
};
use chapchoo_protocol::control::provider_authentication_request::Evidence;
use serde::Deserialize;
use std::{collections::BTreeSet, fmt, sync::Arc, time::Duration};
use tokio::sync::Semaphore;
use tonic::Status;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProviderBootstrapConfig {
    #[serde(default)]
    pub service_accounts: Vec<ServiceAccountBinding>,
    #[serde(default)]
    pub secret_key_provider_ids: Vec<String>,
    #[serde(default)]
    pub trust_provider_ids: Vec<String>,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ServiceAccountBinding {
    pub provider_id: String,
    pub namespace: String,
    pub service_account: String,
    pub audience: String,
}
#[derive(Debug)]
pub struct ReviewedServiceAccount {
    pub username: String,
    pub audiences: Vec<String>,
}
#[tonic::async_trait]
pub trait ServiceAccountReviewer: Send + Sync {
    async fn review(&self, token: &str, audience: &str) -> Result<ReviewedServiceAccount, Status>;
}
pub(crate) enum AuthenticatedProvider {
    Bootstrap(VerifiedSystemOwner),
    Registration(BearerSecret),
}
pub struct ProviderAuthenticator {
    config: ProviderBootstrapConfig,
    cipher: Arc<CredentialCipher>,
    reviewer: Option<Arc<dyn ServiceAccountReviewer>>,
    capacity: Semaphore,
}
impl fmt::Debug for ProviderAuthenticator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ProviderAuthenticator([REDACTED])")
    }
}
impl ProviderAuthenticator {
    pub fn new(
        config: ProviderBootstrapConfig,
        cipher: Arc<CredentialCipher>,
        reviewer: Option<Arc<dyn ServiceAccountReviewer>>,
    ) -> Result<Self, Status> {
        let mut ids = BTreeSet::new();
        if config.service_accounts.len() > 64
            || config.secret_key_provider_ids.len() > 64
            || config.trust_provider_ids.len() > 64
        {
            return Err(Status::invalid_argument("Bootstrap policy is too large"));
        }
        for binding in &config.service_accounts {
            BootstrapProviderPermit::new(binding.provider_id.clone())
                .map_err(|_| Status::invalid_argument("Invalid bootstrap identity"))?;
            if !ids.insert(binding.provider_id.clone())
                || !dns_name(&binding.namespace)
                || !dns_name(&binding.service_account)
                || binding.audience.is_empty()
                || binding.audience.len() > 256
                || binding.audience.chars().any(char::is_whitespace)
            {
                return Err(Status::invalid_argument(
                    "Invalid ServiceAccount bootstrap binding",
                ));
            }
        }
        if !config.service_accounts.is_empty() && reviewer.is_none() {
            return Err(Status::failed_precondition(
                "ServiceAccount verification is required",
            ));
        }
        for list in [&config.secret_key_provider_ids, &config.trust_provider_ids] {
            let mut unique = BTreeSet::new();
            for id in list {
                BootstrapProviderPermit::new(id.clone())
                    .map_err(|_| Status::invalid_argument("Invalid bootstrap identity"))?;
                if !unique.insert(id) {
                    return Err(Status::invalid_argument("Duplicate bootstrap identity"));
                }
            }
        }
        Ok(Self {
            config,
            cipher,
            reviewer,
            capacity: Semaphore::new(16),
        })
    }

    pub(crate) async fn authenticate(
        &self,
        provider: &str,
        evidence: Option<Evidence>,
        internal: Option<&str>,
    ) -> Result<AuthenticatedProvider, Status> {
        let permit = BootstrapProviderPermit::new(provider.to_owned()).map_err(|_| denied())?;
        match evidence {
            Some(Evidence::ServiceAccountToken(token)) => {
                let binding = self
                    .config
                    .service_accounts
                    .iter()
                    .find(|b| b.provider_id == provider)
                    .ok_or_else(denied)?;
                if token.is_empty() || token.len() > 16_384 {
                    return Err(denied());
                }
                let _capacity = self.capacity.try_acquire().map_err(|_| {
                    Status::resource_exhausted("Authentication verification is busy")
                })?;
                let reviewer = self.reviewer.as_ref().ok_or_else(denied)?;
                let reviewed = tokio::time::timeout(
                    Duration::from_secs(5),
                    reviewer.review(&token, &binding.audience),
                )
                .await
                .map_err(|_| Status::unavailable("Authentication verification unavailable"))??;
                let expected = format!(
                    "system:serviceaccount:{}:{}",
                    binding.namespace, binding.service_account
                );
                if reviewed.username != expected || !reviewed.audiences.contains(&binding.audience)
                {
                    return Err(denied());
                }
                Ok(AuthenticatedProvider::Bootstrap(
                    permit.authenticated_with_source(
                        BootstrapAuthenticationSource::ServiceAccount {
                            namespace: binding.namespace.clone(),
                            service_account: binding.service_account.clone(),
                            audience: binding.audience.clone(),
                        },
                    ),
                ))
            }
            Some(Evidence::BootstrapCredential(token)) => {
                if !self
                    .config
                    .secret_key_provider_ids
                    .iter()
                    .any(|id| id == provider)
                {
                    return Err(denied());
                }
                let credential = ProviderBootstrapCredential::parse(token).map_err(|_| denied())?;
                self.cipher
                    .verify_provider_bootstrap(provider, &credential)
                    .map_err(|_| denied())?;
                Ok(AuthenticatedProvider::Bootstrap(
                    permit.authenticated_with_source(BootstrapAuthenticationSource::SecretKey),
                ))
            }
            Some(Evidence::RegistrationToken(token)) => Ok(AuthenticatedProvider::Registration(
                BearerSecret::parse(token).map_err(|_| denied())?,
            )),
            None if internal == Some(provider)
                && self
                    .config
                    .trust_provider_ids
                    .iter()
                    .any(|id| id == provider) =>
            {
                Ok(AuthenticatedProvider::Bootstrap(permit.authenticated()))
            }
            None => Err(denied()),
        }
    }
}
fn denied() -> Status {
    Status::permission_denied("Provider authentication denied")
}
fn dns_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 253
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'-' | b'.'))
        && value
            .bytes()
            .next()
            .is_some_and(|b| b.is_ascii_alphanumeric())
        && value
            .bytes()
            .last()
            .is_some_and(|b| b.is_ascii_alphanumeric())
}
