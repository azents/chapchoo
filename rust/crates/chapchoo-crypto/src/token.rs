use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chacha20poly1305::{
    XChaCha20Poly1305,
    aead::{Generate, Key},
};
use sha2::{Digest, Sha256};
use std::fmt;
use subtle::ConstantTimeEq;
use zeroize::Zeroizing;

use crate::CryptoError;

/// A generated 256-bit bearer value, used for tokens and one-shot proofs.
pub struct BearerSecret(Zeroizing<String>);

impl BearerSecret {
    pub fn generate() -> Result<Self, CryptoError> {
        let bytes = Key::<XChaCha20Poly1305>::try_generate().map_err(CryptoError::Random)?;
        let bytes = Zeroizing::new(bytes.to_vec());
        Ok(Self(Zeroizing::new(URL_SAFE_NO_PAD.encode(&bytes))))
    }

    pub fn parse(input: String) -> Result<Self, CryptoError> {
        let input = Zeroizing::new(input);
        if input.len() != 43 {
            return Err(CryptoError::InvalidToken);
        }
        let decoded = Zeroizing::new(
            URL_SAFE_NO_PAD
                .decode(input.as_bytes())
                .map_err(|_| CryptoError::InvalidToken)?,
        );
        if decoded.len() != 32 {
            return Err(CryptoError::InvalidToken);
        }
        Ok(Self(input))
    }

    /// Issuance/authentication boundaries only. Never log or serialize implicitly.
    pub fn expose(&self) -> &str {
        &self.0
    }

    pub fn digest(&self) -> SecretDigest {
        SecretDigest(Sha256::digest(self.0.as_bytes()).into())
    }
}

impl fmt::Debug for BearerSecret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("BearerSecret([REDACTED])")
    }
}

/// Matching material, not an authentication bearer. Debug is still redacted.
#[derive(Clone, PartialEq, Eq)]
pub struct SecretDigest(pub(crate) [u8; 32]);

impl SecretDigest {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn matches(&self, token: &BearerSecret) -> bool {
        constant_time_equal(&self.0, token.digest().as_bytes())
    }
}

impl fmt::Debug for SecretDigest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SecretDigest([REDACTED])")
    }
}

pub(crate) fn constant_time_equal(left: &[u8; 32], right: &[u8; 32]) -> bool {
    bool::from(left.ct_eq(right))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_are_full_entropy_canonical_and_redacted() -> Result<(), CryptoError> {
        let token = BearerSecret::generate()?;
        let other = BearerSecret::generate()?;
        assert_eq!(token.expose().len(), 43);
        assert_ne!(token.expose(), other.expose());
        assert!(token.digest().matches(&token));
        assert!(!token.digest().matches(&other));
        let parsed = BearerSecret::parse(token.expose().to_owned())?;
        assert!(token.digest().matches(&parsed));
        assert!(!format!("{token:?}").contains(token.expose()));
        assert_eq!(format!("{:?}", token.digest()), "SecretDigest([REDACTED])");
        Ok(())
    }

    #[test]
    fn malformed_tokens_cannot_enter_storage() {
        for value in ["", "secret", "A".repeat(44).as_str(), &"é".repeat(43)] {
            assert!(BearerSecret::parse(value.to_owned()).is_err());
        }
    }
}
