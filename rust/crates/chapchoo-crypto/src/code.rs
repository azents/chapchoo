use chacha20poly1305::{XNonce, aead::Generate};
use chapchoo_domain::EmailAddress;
use std::fmt;
use zeroize::Zeroizing;

use crate::{BearerSecret, CredentialCipher, CryptoError, token::constant_time_equal};

const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

/// A six-character transient email code, never an ordinary persisted value.
pub struct EmailCode(Zeroizing<String>);

impl EmailCode {
    pub fn generate() -> Result<Self, CryptoError> {
        let mut code = Zeroizing::new(String::with_capacity(6));
        while code.len() < 6 {
            let random = XNonce::try_generate().map_err(CryptoError::Random)?;
            for byte in random.iter() {
                // Rejection sampling avoids modulo bias (252 is a multiple of 36).
                if *byte < 252 {
                    code.push(char::from(ALPHABET[usize::from(*byte) % ALPHABET.len()]));
                    if code.len() == 6 {
                        break;
                    }
                }
            }
        }
        Ok(Self(code))
    }

    pub fn parse(input: String) -> Result<Self, CryptoError> {
        let mut input = Zeroizing::new(input);
        if input.len() != 6 || !input.bytes().all(|v| v.is_ascii_alphanumeric()) {
            return Err(CryptoError::InvalidCode);
        }
        input.make_ascii_uppercase();
        Ok(Self(input))
    }

    pub fn expose(&self) -> &str {
        &self.0
    }

    pub fn verifier(
        &self,
        cipher: &CredentialCipher,
        email: &EmailAddress,
        binding: &BearerSecret,
    ) -> Result<CodeVerifier, CryptoError> {
        cipher.code_verifier(email, binding, self.expose())
    }
}

impl fmt::Debug for EmailCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("EmailCode([REDACTED])")
    }
}

pub struct CodeVerifier(pub(crate) [u8; 32]);

impl CodeVerifier {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn matches(
        &self,
        cipher: &CredentialCipher,
        email: &EmailAddress,
        binding: &BearerSecret,
        submitted: &EmailCode,
    ) -> Result<bool, CryptoError> {
        let candidate = submitted.verifier(cipher, email, binding)?;
        Ok(constant_time_equal(&self.0, &candidate.0))
    }
}

impl fmt::Debug for CodeVerifier {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("CodeVerifier([REDACTED])")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RootKey;
    use base64::{Engine, engine::general_purpose::URL_SAFE};
    use std::error::Error;

    #[test]
    fn verifier_is_email_challenge_and_purpose_bound() -> Result<(), Box<dyn Error>> {
        let cipher = CredentialCipher::new(RootKey::parse(URL_SAFE.encode([1; 32]))?)?;
        let other = CredentialCipher::new(RootKey::parse(URL_SAFE.encode([2; 32]))?)?;
        let email = EmailAddress::parse("Member@example.com")?;
        let binding = BearerSecret::generate()?;
        let code = EmailCode::parse("ab12cd".to_owned())?;
        let verifier = code.verifier(&cipher, &email, &binding)?;
        assert!(verifier.matches(
            &cipher,
            &email,
            &binding,
            &EmailCode::parse("AB12CD".to_owned())?
        )?);
        assert!(!verifier.matches(&other, &email, &binding, &code)?);
        assert!(!verifier.matches(
            &cipher,
            &EmailAddress::parse("other@example.com")?,
            &binding,
            &code
        )?);
        assert!(!verifier.matches(&cipher, &email, &BearerSecret::generate()?, &code)?);
        assert!(!verifier.matches(
            &cipher,
            &email,
            &binding,
            &EmailCode::parse("WRONG1".to_owned())?
        )?);
        assert!(EmailCode::parse("12345 ".to_owned()).is_err());
        for _ in 0..32 {
            let generated = EmailCode::generate()?;
            assert_eq!(generated.expose().len(), 6);
            assert!(generated.expose().bytes().all(|v| ALPHABET.contains(&v)));
            assert_eq!(format!("{generated:?}"), "EmailCode([REDACTED])");
        }
        Ok(())
    }
}
