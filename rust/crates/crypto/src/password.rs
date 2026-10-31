use chapchoo_domain::Password;
use std::fmt;
use zeroize::Zeroizing;

use crate::CryptoError;

#[derive(Debug, Clone, Copy)]
pub struct PasswordCost(u32);

impl PasswordCost {
    /// The upper bound is the bcrypt algorithm's maximum, not an ingress value.
    pub fn new(cost: u32) -> Result<Self, CryptoError> {
        if !(10..=31).contains(&cost) {
            return Err(CryptoError::InvalidCost);
        }
        Ok(Self(cost))
    }
}

impl Default for PasswordCost {
    fn default() -> Self {
        Self(12)
    }
}

pub struct PasswordHash(Zeroizing<String>);

impl PasswordHash {
    pub fn parse(encoded: String) -> Result<Self, CryptoError> {
        let encoded = Zeroizing::new(encoded);
        let parts =
            bcrypt::HashParts::from_str(&encoded).map_err(CryptoError::InvalidPasswordHash)?;
        PasswordCost::new(parts.get_cost()).map_err(|_| {
            CryptoError::InvalidPasswordHash(bcrypt::BcryptError::InvalidHash(
                "Work factor outside the approved policy",
            ))
        })?;
        Ok(Self(encoded))
    }

    /// Repository persistence/credential snapshot boundary only.
    pub fn encoded(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for PasswordHash {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("PasswordHash([REDACTED])")
    }
}

use std::str::FromStr;

/// Synchronous CPU operation. Its service caller must own bounded blocking admission.
#[derive(Debug, Default)]
pub struct PasswordHasher {
    cost: PasswordCost,
}

impl PasswordHasher {
    pub fn new(cost: PasswordCost) -> Self {
        Self { cost }
    }

    pub fn hash(&self, password: &Password) -> Result<PasswordHash, CryptoError> {
        let encoded = bcrypt::non_truncating_hash(password.expose(), self.cost.0)
            .map_err(CryptoError::PasswordHash)?;
        PasswordHash::parse(encoded)
    }

    /// Reject inputs outside the accepted representation before bcrypt can
    /// equate a repeated key with embedded NULs. Character classes are a
    /// creation policy, not a second authentication policy.
    pub fn verify(&self, input: &str, hash: &PasswordHash) -> Result<bool, CryptoError> {
        if !(8..=64).contains(&input.len())
            || !input.bytes().all(|value| (b' '..=b'~').contains(&value))
        {
            return Ok(false);
        }
        bcrypt::non_truncating_verify(input, hash.encoded())
            .map_err(CryptoError::InvalidPasswordHash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn full_value_admission_and_secret_safe_formatting() -> Result<(), Box<dyn Error>> {
        let password = Password::parse(format!("Aa1!{}", "x".repeat(60)))?;
        let hasher = PasswordHasher::new(PasswordCost::new(10)?);
        let hash = hasher.hash(&password)?;
        assert!(hasher.verify(password.expose(), &hash)?);
        assert!(!hasher.verify(&format!("{}y", &password.expose()[..63]), &hash)?);
        assert!(!hasher.verify(&format!(" {}", password.expose()), &hash)?);
        let parsed = PasswordHash::parse(hash.encoded().to_owned())?;
        assert!(hasher.verify(password.expose(), &parsed)?);
        assert!(!format!("{hash:?}").contains(hash.encoded()));
        assert!(PasswordCost::new(9).is_err());
        assert!(PasswordCost::new(32).is_err());
        assert!(PasswordHash::parse("invalid fixture".to_owned()).is_err());
        Ok(())
    }

    #[test]
    fn rejects_repeated_nul_keys_and_non_printable_login_input() -> Result<(), Box<dyn Error>> {
        let password = Password::parse("Aa1!xxxx".to_owned())?;
        let hasher = PasswordHasher::new(PasswordCost::new(10)?);
        let hash = hasher.hash(&password)?;
        assert!(hasher.verify(password.expose(), &hash)?);
        for input in [
            "Aa1!xxxx\0Aa1!xxxx",
            "Aa1!xxxx\0Aa1!xxxx\0Aa1!xxxx",
            "Aa1!xxxx\n",
            "Aa1!xxxxé",
            "Aa1!xxx",
        ] {
            assert!(!hasher.verify(input, &hash)?);
        }
        Ok(())
    }
}
