use crate::CryptoError;
use std::fmt;

/// Request-local proof supplied by a trusted, authenticated bootstrap adapter.
pub struct VerifiedSystemOwner {
    owner_id: String,
    provider_id: String,
    source: BootstrapAuthenticationSource,
}

#[derive(Debug, Clone)]
pub enum BootstrapAuthenticationSource {
    Internal,
    SecretKey,
    ServiceAccount {
        namespace: String,
        service_account: String,
        audience: String,
    },
}

/// Captured only from server bootstrap configuration, never a wire request.
#[derive(Debug, Clone)]
pub struct BootstrapProviderPermit {
    provider_id: String,
}
impl BootstrapProviderPermit {
    pub fn new(provider_id: String) -> Result<Self, CryptoError> {
        if !valid_identity(&provider_id) {
            return Err(CryptoError::InvalidContext);
        }
        Ok(Self { provider_id })
    }

    /// Called by the configured adapter after its evidence has been verified.
    pub fn authenticated(&self) -> VerifiedSystemOwner {
        self.authenticated_with_source(BootstrapAuthenticationSource::Internal)
    }

    pub fn authenticated_with_source(
        &self,
        source: BootstrapAuthenticationSource,
    ) -> VerifiedSystemOwner {
        VerifiedSystemOwner {
            owner_id: "installation".to_owned(),
            provider_id: self.provider_id.clone(),
            source,
        }
    }
}

impl VerifiedSystemOwner {
    pub fn owner_id(&self) -> &str {
        &self.owner_id
    }
    pub fn provider_id(&self) -> &str {
        &self.provider_id
    }
    pub fn authentication_fingerprint(&self) -> [u8; 32] {
        match &self.source {
            BootstrapAuthenticationSource::Internal => {
                crate::payload_fingerprint("provider-auth-internal", &[])
            }
            BootstrapAuthenticationSource::SecretKey => {
                crate::payload_fingerprint("provider-auth-secret-key", &[])
            }
            BootstrapAuthenticationSource::ServiceAccount {
                namespace,
                service_account,
                audience,
            } => crate::payload_fingerprint(
                "provider-auth-service-account",
                &[
                    namespace.as_bytes(),
                    service_account.as_bytes(),
                    audience.as_bytes(),
                ],
            ),
        }
    }
}

impl fmt::Debug for VerifiedSystemOwner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("VerifiedSystemOwner([REDACTED])")
    }
}

fn valid_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captured_permit_has_one_valid_identity_and_redacted_proof() -> Result<(), CryptoError> {
        let permit = BootstrapProviderPermit::new("provider-a".into())?;
        let proof = permit.authenticated();
        assert_eq!(proof.provider_id(), "provider-a");
        assert_eq!(proof.owner_id(), "installation");
        assert_eq!(format!("{proof:?}"), "VerifiedSystemOwner([REDACTED])");
        for invalid in ["", "*", "provider\n"] {
            assert!(BootstrapProviderPermit::new(invalid.to_owned()).is_err());
        }
        Ok(())
    }
}
