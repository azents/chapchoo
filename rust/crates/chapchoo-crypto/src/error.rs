/// Typed failures deliberately exclude rejected secret material.
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Invalid installation encryption key")]
    InvalidKey,
    #[error("Secure random generation failed")]
    Random(#[source] getrandom::Error),
    #[error("Invalid bearer credential")]
    InvalidToken,
    #[error("Invalid authentication code")]
    InvalidCode,
    #[error("Invalid password work factor")]
    InvalidCost,
    #[error("Password hashing failed")]
    PasswordHash(#[source] bcrypt::BcryptError),
    #[error("Stored password verifier is invalid")]
    InvalidPasswordHash(#[source] bcrypt::BcryptError),
    #[error("Credential protection failed")]
    Protection,
    #[error("Protected credential is invalid")]
    InvalidCiphertext,
    #[error("Credential context is invalid")]
    InvalidContext,
    #[error("Installation encryption key does not match")]
    KeyMismatch,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn infrastructure_causes_survive_sanitized_display() {
        let error = CryptoError::PasswordHash(bcrypt::BcryptError::InvalidHash(
            "Static dependency failure",
        ));
        assert_eq!(error.to_string(), "Password hashing failed");
        assert!(error.source().is_some());
    }
}
