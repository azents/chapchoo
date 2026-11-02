use crate::{BearerSecret, CryptoError, SecretDigest};
use std::fmt;

/// A configured verifier for one system owner and an explicit Provider scope.
/// This is unrelated to the installation encryption root and component keys.
pub struct SystemOwnerVerifier {
    owner_id: String,
    providers: Vec<String>,
    verifier: SecretDigest,
}

impl SystemOwnerVerifier {
    pub fn new(
        owner_id: String,
        providers: Vec<String>,
        verifier: SecretDigest,
    ) -> Result<Self, CryptoError> {
        if !valid_identity(&owner_id)
            || providers.is_empty()
            || providers.iter().any(|id| !valid_identity(id))
        {
            return Err(CryptoError::InvalidContext);
        }
        let mut unique = providers;
        unique.sort();
        unique.dedup();
        Ok(Self {
            owner_id,
            providers: unique,
            verifier,
        })
    }

    /// Returns a request-local proof of the configured owner/Provider binding.
    pub fn verify(
        &self,
        provider_id: &str,
        credential: &BearerSecret,
    ) -> Result<VerifiedSystemOwner, CryptoError> {
        let valid_credential = self.verifier.matches(credential);
        let in_scope = self.providers.iter().any(|id| id == provider_id);
        if !valid_credential || !in_scope {
            return Err(CryptoError::InvalidToken);
        }
        Ok(VerifiedSystemOwner {
            owner_id: self.owner_id.clone(),
            provider_id: provider_id.to_owned(),
        })
    }
}

/// Constructible only by verification, never from a caller's asserted owner ID.
pub struct VerifiedSystemOwner {
    owner_id: String,
    provider_id: String,
}

impl VerifiedSystemOwner {
    pub fn owner_id(&self) -> &str {
        &self.owner_id
    }
    pub fn provider_id(&self) -> &str {
        &self.provider_id
    }
}

impl fmt::Debug for SystemOwnerVerifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SystemOwnerVerifier([REDACTED])")
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
    fn only_matching_credential_and_explicit_provider_scope_produce_a_proof()
    -> Result<(), CryptoError> {
        let credential = BearerSecret::generate()?;
        let different = BearerSecret::generate()?;
        let verifier = SystemOwnerVerifier::new(
            "system-owner".into(),
            vec!["provider-a".into()],
            credential.digest(),
        )?;
        let proof = verifier.verify("provider-a", &credential)?;
        assert_eq!(proof.owner_id(), "system-owner");
        assert_eq!(proof.provider_id(), "provider-a");
        assert!(verifier.verify("provider-b", &credential).is_err());
        assert!(verifier.verify("provider-a", &different).is_err());
        assert!(!format!("{verifier:?} {proof:?}").contains(credential.expose()));
        assert_eq!(format!("{proof:?}"), "VerifiedSystemOwner([REDACTED])");
        Ok(())
    }

    #[test]
    fn empty_or_ambiguous_configured_scope_is_rejected() -> Result<(), CryptoError> {
        let digest = BearerSecret::generate()?.digest();
        for providers in [vec![], vec!["*".into()], vec!["provider\nassertion".into()]] {
            assert!(SystemOwnerVerifier::new("owner".into(), providers, digest.clone()).is_err());
        }
        assert!(SystemOwnerVerifier::new(String::new(), vec!["provider".into()], digest).is_err());
        Ok(())
    }
}
