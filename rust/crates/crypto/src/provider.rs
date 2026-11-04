//! Purpose-separated, identity-bound bootstrap proof. The Provider never gets a root key.
use crate::{CredentialCipher, CryptoError};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chacha20poly1305::{
    XChaCha20Poly1305, XNonce,
    aead::{Aead, Generate, KeyInit, Payload},
};
use std::fmt;
use zeroize::Zeroizing;

const PURPOSE: &[u8] = b"chapchoo/provider-bootstrap";
const SALT_BYTES: usize = 32;
const NONCE_BYTES: usize = 24;
const TAG_BYTES: usize = 16;
const MAX_ID: usize = 128;

/// A reusable secret proof, not a communication key or a root-key container.
pub struct ProviderBootstrapCredential(Zeroizing<String>);
impl ProviderBootstrapCredential {
    pub fn parse(encoded: String) -> Result<Self, CryptoError> {
        let encoded = Zeroizing::new(encoded);
        if encoded.len() > 272 {
            return Err(CryptoError::InvalidToken);
        }
        let bytes = Zeroizing::new(
            URL_SAFE_NO_PAD
                .decode(encoded.as_bytes())
                .map_err(|_| CryptoError::InvalidToken)?,
        );
        if !(NONCE_BYTES + 1 + SALT_BYTES + TAG_BYTES
            ..=NONCE_BYTES + MAX_ID + SALT_BYTES + TAG_BYTES)
            .contains(&bytes.len())
        {
            return Err(CryptoError::InvalidToken);
        }
        Ok(Self(encoded))
    }
    pub fn expose(&self) -> &str {
        &self.0
    }
}
impl fmt::Debug for ProviderBootstrapCredential {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ProviderBootstrapCredential([REDACTED])")
    }
}

impl CredentialCipher {
    pub fn issue_provider_bootstrap(
        &self,
        provider_id: &str,
    ) -> Result<ProviderBootstrapCredential, CryptoError> {
        validate_id(provider_id)?;
        let mut plaintext = Zeroizing::new(provider_id.as_bytes().to_vec());
        let mut salt = Zeroizing::new([0; SALT_BYTES]);
        getrandom::fill(salt.as_mut()).map_err(CryptoError::Random)?;
        plaintext.extend_from_slice(salt.as_slice());
        let cipher = XChaCha20Poly1305::new_from_slice(self.provider_bootstrap.as_slice())
            .map_err(|_| CryptoError::Protection)?;
        let nonce = XNonce::try_generate().map_err(CryptoError::Random)?;
        let ciphertext = cipher
            .encrypt(
                &nonce,
                Payload {
                    msg: &plaintext,
                    aad: PURPOSE,
                },
            )
            .map_err(|_| CryptoError::Protection)?;
        let mut bytes = Zeroizing::new(nonce.to_vec());
        bytes.extend_from_slice(&ciphertext);
        ProviderBootstrapCredential::parse(URL_SAFE_NO_PAD.encode(bytes.as_slice()))
    }

    /// Cryptographic validity is insufficient: caller must also enforce bootstrap permits.
    pub fn verify_provider_bootstrap(
        &self,
        provider_id: &str,
        credential: &ProviderBootstrapCredential,
    ) -> Result<(), CryptoError> {
        validate_id(provider_id)?;
        let bytes = Zeroizing::new(
            URL_SAFE_NO_PAD
                .decode(credential.expose())
                .map_err(|_| CryptoError::InvalidToken)?,
        );
        let (nonce, ciphertext) = bytes.split_at(NONCE_BYTES);
        let nonce: [u8; NONCE_BYTES] = nonce.try_into().map_err(|_| CryptoError::InvalidToken)?;
        let cipher = XChaCha20Poly1305::new_from_slice(self.provider_bootstrap.as_slice())
            .map_err(|_| CryptoError::Protection)?;
        let plaintext = Zeroizing::new(
            cipher
                .decrypt(
                    &XNonce::from(nonce),
                    Payload {
                        msg: ciphertext,
                        aad: PURPOSE,
                    },
                )
                .map_err(|_| CryptoError::InvalidToken)?,
        );
        if plaintext.len() != provider_id.len() + SALT_BYTES
            || plaintext[..provider_id.len()] != *provider_id.as_bytes()
        {
            return Err(CryptoError::InvalidToken);
        }
        Ok(())
    }
}

fn validate_id(id: &str) -> Result<(), CryptoError> {
    if id.is_empty()
        || id.len() > MAX_ID
        || !id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b':'))
    {
        return Err(CryptoError::InvalidContext);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RootKey;

    fn fixture_cipher(byte: u8) -> Result<CredentialCipher, CryptoError> {
        CredentialCipher::new(RootKey::parse(
            base64::engine::general_purpose::URL_SAFE.encode([byte; 32]),
        )?)
    }

    #[test]
    fn proof_binds_identity_root_purpose_and_redacts() -> Result<(), CryptoError> {
        let cipher = fixture_cipher(1)?;
        let proof = cipher.issue_provider_bootstrap("provider-a")?;
        cipher.verify_provider_bootstrap("provider-a", &proof)?;
        assert!(
            cipher
                .verify_provider_bootstrap("provider-b", &proof)
                .is_err()
        );
        assert!(
            fixture_cipher(2)?
                .verify_provider_bootstrap("provider-a", &proof)
                .is_err()
        );
        assert_eq!(
            format!("{proof:?}"),
            "ProviderBootstrapCredential([REDACTED])"
        );
        let another = cipher.issue_provider_bootstrap("provider-a")?;
        assert_ne!(proof.expose(), another.expose());
        // Issuance is not a revocation registry.
        cipher.verify_provider_bootstrap("provider-a", &proof)?;
        let mut bytes = URL_SAFE_NO_PAD
            .decode(proof.expose())
            .map_err(|_| CryptoError::InvalidToken)?;
        if let Some(byte) = bytes.last_mut() {
            *byte ^= 1;
        }
        let tampered = ProviderBootstrapCredential::parse(URL_SAFE_NO_PAD.encode(bytes))?;
        assert!(
            cipher
                .verify_provider_bootstrap("provider-a", &tampered)
                .is_err()
        );
        for id in ["", "*", "provider\n", &"a".repeat(129)] {
            assert!(cipher.issue_provider_bootstrap(id).is_err());
        }
        for input in ["bad", &"a".repeat(273)] {
            assert!(ProviderBootstrapCredential::parse(input.to_owned()).is_err());
        }
        Ok(())
    }
}
