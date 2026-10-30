use base64::{Engine, engine::general_purpose::URL_SAFE};
use chacha20poly1305::{
    XChaCha20Poly1305, XNonce,
    aead::{Aead, Generate, KeyInit, Payload},
};
use hkdf::Hkdf;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::fmt;
use zeroize::Zeroizing;

use crate::{BearerSecret, CryptoError, SecretDigest, code::CodeVerifier};

type HmacSha256 = Hmac<Sha256>;
const ENCRYPTION: &[u8] = b"chapchoo/credential-encryption";
const CODES: &[u8] = b"chapchoo/email-code-verifier";
const BINDING: &[u8] = b"chapchoo/installation-key-check";
const BINDING_CONTEXT: &[u8] = b"chapchoo/fixed-installation-root";

/// Operator-provided root. No replacement, generation or implicit formatting API.
pub struct RootKey(Zeroizing<[u8; 32]>);

impl RootKey {
    pub fn parse(encoded: String) -> Result<Self, CryptoError> {
        let encoded = Zeroizing::new(encoded);
        let decoded = Zeroizing::new(
            URL_SAFE
                .decode(encoded.as_bytes())
                .map_err(|_| CryptoError::InvalidKey)?,
        );
        let bytes: [u8; 32] = decoded
            .as_slice()
            .try_into()
            .map_err(|_| CryptoError::InvalidKey)?;
        Ok(Self(Zeroizing::new(bytes)))
    }
}

impl fmt::Debug for RootKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RootKey([REDACTED])")
    }
}

/// One common immutable cryptographic capability for the backend installation.
pub struct CredentialCipher {
    encryption: Zeroizing<[u8; 32]>,
    codes: Zeroizing<[u8; 32]>,
    binding: Zeroizing<[u8; 32]>,
}

impl CredentialCipher {
    pub fn new(root: RootKey) -> Result<Self, CryptoError> {
        let hkdf = Hkdf::<Sha256>::new(None, root.0.as_slice());
        let derive = |label: &[u8]| -> Result<Zeroizing<[u8; 32]>, CryptoError> {
            let mut key = Zeroizing::new([0; 32]);
            hkdf.expand(label, key.as_mut())
                .map_err(|_| CryptoError::Protection)?;
            Ok(key)
        };
        Ok(Self {
            encryption: derive(ENCRYPTION)?,
            codes: derive(CODES)?,
            binding: derive(BINDING)?,
        })
    }

    pub fn installation_check(&self) -> Result<InstallationKeyCheck, CryptoError> {
        let mut mac = <HmacSha256 as Mac>::new_from_slice(self.binding.as_slice())
            .map_err(|_| CryptoError::Protection)?;
        mac.update(BINDING_CONTEXT);
        Ok(InstallationKeyCheck(mac.finalize().into_bytes().into()))
    }

    pub fn verify_installation(&self, stored: &InstallationKeyCheck) -> Result<(), CryptoError> {
        let mut mac = <HmacSha256 as Mac>::new_from_slice(self.binding.as_slice())
            .map_err(|_| CryptoError::Protection)?;
        mac.update(BINDING_CONTEXT);
        mac.verify_slice(&stored.0)
            .map_err(|_| CryptoError::KeyMismatch)
    }

    pub fn protect_latest(
        &self,
        session_id: &str,
        token: &BearerSecret,
    ) -> Result<RecoveryEnvelope, CryptoError> {
        let aad = recovery_context(session_id, &token.digest())?;
        let cipher = XChaCha20Poly1305::new_from_slice(self.encryption.as_slice())
            .map_err(|_| CryptoError::Protection)?;
        let nonce = XNonce::try_generate().map_err(CryptoError::Random)?;
        let ciphertext = cipher
            .encrypt(
                &nonce,
                Payload {
                    msg: token.expose().as_bytes(),
                    aad: &aad,
                },
            )
            .map_err(|_| CryptoError::Protection)?;
        Ok(RecoveryEnvelope {
            nonce: nonce.into(),
            ciphertext,
        })
    }

    /// Caller must first establish current session validity and previous-token grace.
    /// This primitive authenticates storage; it is not a session authorization API.
    pub fn recover_latest(
        &self,
        session_id: &str,
        current_hash: &SecretDigest,
        envelope: &RecoveryEnvelope,
    ) -> Result<BearerSecret, CryptoError> {
        let aad = recovery_context(session_id, current_hash)?;
        let cipher = XChaCha20Poly1305::new_from_slice(self.encryption.as_slice())
            .map_err(|_| CryptoError::Protection)?;
        let nonce = XNonce::from(envelope.nonce);
        let plaintext = Zeroizing::new(
            cipher
                .decrypt(
                    &nonce,
                    Payload {
                        msg: &envelope.ciphertext,
                        aad: &aad,
                    },
                )
                .map_err(|_| CryptoError::InvalidCiphertext)?,
        );
        let text = std::str::from_utf8(&plaintext).map_err(|_| CryptoError::InvalidCiphertext)?;
        let token =
            BearerSecret::parse(text.to_owned()).map_err(|_| CryptoError::InvalidCiphertext)?;
        if !current_hash.matches(&token) {
            return Err(CryptoError::InvalidCiphertext);
        }
        Ok(token)
    }

    pub(crate) fn code_verifier(
        &self,
        email: &chapchoo_domain::EmailAddress,
        binding: &BearerSecret,
        code: &str,
    ) -> Result<CodeVerifier, CryptoError> {
        let mut mac = <HmacSha256 as Mac>::new_from_slice(self.codes.as_slice())
            .map_err(|_| CryptoError::Protection)?;
        for part in [
            email.lookup_key().as_bytes(),
            binding.expose().as_bytes(),
            code.as_bytes(),
        ] {
            mac.update(&(part.len() as u64).to_be_bytes());
            mac.update(part);
        }
        Ok(CodeVerifier(mac.finalize().into_bytes().into()))
    }
}

impl fmt::Debug for CredentialCipher {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("CredentialCipher([REDACTED])")
    }
}

#[derive(Clone)]
pub struct InstallationKeyCheck([u8; 32]);

impl InstallationKeyCheck {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Debug for InstallationKeyCheck {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("InstallationKeyCheck([REDACTED])")
    }
}

/// Fixed construction: separate nonce and authenticated ciphertext; no key version.
pub struct RecoveryEnvelope {
    nonce: [u8; 24],
    ciphertext: Vec<u8>,
}

impl RecoveryEnvelope {
    pub fn from_parts(nonce: [u8; 24], ciphertext: Vec<u8>) -> Result<Self, CryptoError> {
        // A canonical 43-byte token plus the full 16-byte authentication tag.
        if ciphertext.len() != 59 {
            return Err(CryptoError::InvalidCiphertext);
        }
        Ok(Self { nonce, ciphertext })
    }

    pub fn nonce(&self) -> &[u8; 24] {
        &self.nonce
    }

    pub fn ciphertext(&self) -> &[u8] {
        &self.ciphertext
    }
}

impl fmt::Debug for RecoveryEnvelope {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RecoveryEnvelope([REDACTED])")
    }
}

fn recovery_context(session_id: &str, digest: &SecretDigest) -> Result<Vec<u8>, CryptoError> {
    if session_id.is_empty() {
        return Err(CryptoError::InvalidContext);
    }
    let mut aad = b"chapchoo/session-recovery\0".to_vec();
    aad.extend_from_slice(&(session_id.len() as u64).to_be_bytes());
    aad.extend_from_slice(session_id.as_bytes());
    aad.extend_from_slice(digest.as_bytes());
    Ok(aad)
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) fn fixture_cipher(byte: u8) -> Result<CredentialCipher, CryptoError> {
        CredentialCipher::new(RootKey::parse(URL_SAFE.encode([byte; 32]))?)
    }

    #[test]
    fn fixed_key_check_and_redaction() -> Result<(), CryptoError> {
        let original = fixture_cipher(1)?;
        let same = fixture_cipher(1)?;
        let different = fixture_cipher(2)?;
        let check = original.installation_check()?;
        same.verify_installation(&check)?;
        assert!(matches!(
            different.verify_installation(&check),
            Err(CryptoError::KeyMismatch)
        ));
        assert!(RootKey::parse(URL_SAFE.encode([0; 31])).is_err());
        assert!(RootKey::parse("not a key".to_owned()).is_err());
        assert_eq!(format!("{original:?}"), "CredentialCipher([REDACTED])");
        Ok(())
    }

    #[test]
    fn recovery_authenticates_ciphertext_nonce_key_and_context() -> Result<(), CryptoError> {
        let cipher = fixture_cipher(1)?;
        let token = BearerSecret::generate()?;
        let hash = token.digest();
        let envelope = cipher.protect_latest("session-a", &token)?;
        assert!(hash.matches(&cipher.recover_latest("session-a", &hash, &envelope)?));
        assert_ne!(
            envelope.nonce(),
            cipher.protect_latest("session-a", &token)?.nonce()
        );
        assert!(
            cipher
                .recover_latest("session-b", &hash, &envelope)
                .is_err()
        );
        assert!(
            cipher
                .recover_latest("session-a", &BearerSecret::generate()?.digest(), &envelope)
                .is_err()
        );
        assert!(
            fixture_cipher(2)?
                .recover_latest("session-a", &hash, &envelope)
                .is_err()
        );
        let mut damaged = envelope.ciphertext().to_vec();
        damaged[0] ^= 1;
        let damaged = RecoveryEnvelope::from_parts(*envelope.nonce(), damaged)?;
        assert!(cipher.recover_latest("session-a", &hash, &damaged).is_err());
        let mut nonce = *envelope.nonce();
        nonce[0] ^= 1;
        let damaged = RecoveryEnvelope::from_parts(nonce, envelope.ciphertext().to_vec())?;
        assert!(cipher.recover_latest("session-a", &hash, &damaged).is_err());
        assert!(RecoveryEnvelope::from_parts([0; 24], vec![0; 8]).is_err());
        assert!(cipher.protect_latest("", &token).is_err());
        Ok(())
    }
}
