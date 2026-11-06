//! Purpose-bound protection for external authentication, never caller authorization.

use crate::{CredentialCipher, CryptoError, SecretDigest};
use chacha20poly1305::{
    XChaCha20Poly1305, XNonce,
    aead::{Aead, Generate, KeyInit, Payload},
};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::fmt;
use zeroize::Zeroizing;

const MAX_SECRET_BYTES: usize = 65_536;
const TAG_BYTES: usize = 16;

/// Bounded secret bytes; never implicitly serialized, displayed or cloned.
pub struct ExternalSecret(Zeroizing<Vec<u8>>);

impl ExternalSecret {
    pub fn new(bytes: Vec<u8>) -> Result<Self, CryptoError> {
        let bytes = Zeroizing::new(bytes);
        if bytes.is_empty() || bytes.len() > MAX_SECRET_BYTES {
            return Err(CryptoError::InvalidToken);
        }
        Ok(Self(bytes))
    }

    /// Only an authorized acquisition/storage boundary may expose these bytes.
    pub fn expose(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Debug for ExternalSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ExternalSecret([REDACTED])")
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ExternalSecretOwner<'a> {
    User(&'a str),
    Team(&'a str),
}

#[derive(Debug, Clone, Copy)]
pub enum ExternalSecretKind {
    ChatGpt,
    OpenAiApiKey,
    PendingDevice,
}

/// Stable record/attempt identity and owning scope, not a caller permission.
pub struct ExternalSecretContext<'a> {
    owner: ExternalSecretOwner<'a>,
    object_id: &'a str,
    kind: ExternalSecretKind,
}

impl<'a> ExternalSecretContext<'a> {
    pub fn new(
        owner: ExternalSecretOwner<'a>,
        object_id: &'a str,
        kind: ExternalSecretKind,
    ) -> Result<Self, CryptoError> {
        let owner_id = match owner {
            ExternalSecretOwner::User(id) | ExternalSecretOwner::Team(id) => id,
        };
        for id in [owner_id, object_id] {
            if id.is_empty()
                || id.len() > 128
                || !id
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"-_".contains(&b))
            {
                return Err(CryptoError::InvalidContext);
            }
        }
        Ok(Self {
            owner,
            object_id,
            kind,
        })
    }

    fn associated_data(&self) -> Vec<u8> {
        let (scope, id) = match self.owner {
            ExternalSecretOwner::User(id) => (b"user".as_slice(), id),
            ExternalSecretOwner::Team(id) => (b"team".as_slice(), id),
        };
        let kind = match self.kind {
            ExternalSecretKind::ChatGpt => b"chatgpt".as_slice(),
            ExternalSecretKind::OpenAiApiKey => b"openai-api-key".as_slice(),
            ExternalSecretKind::PendingDevice => b"pending-device".as_slice(),
        };
        let mut bytes = b"chapchoo/external-secret\0".to_vec();
        for part in [scope, id.as_bytes(), self.object_id.as_bytes(), kind] {
            bytes.extend_from_slice(&(part.len() as u64).to_be_bytes());
            bytes.extend_from_slice(part);
        }
        bytes
    }

    fn key<'b>(&self, cipher: &'b CredentialCipher) -> &'b [u8] {
        match self.kind {
            ExternalSecretKind::PendingDevice => cipher.device_attempt.as_slice(),
            ExternalSecretKind::ChatGpt | ExternalSecretKind::OpenAiApiKey => {
                cipher.external_auth.as_slice()
            }
        }
    }
}

impl fmt::Debug for ExternalSecretContext<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ExternalSecretContext([REDACTED])")
    }
}

/// Authenticated ciphertext only, independent of the fixed session-bearer envelope.
pub struct ExternalSecretEnvelope {
    nonce: [u8; 24],
    ciphertext: Vec<u8>,
}

impl ExternalSecretEnvelope {
    pub fn from_parts(nonce: [u8; 24], ciphertext: Vec<u8>) -> Result<Self, CryptoError> {
        if !(TAG_BYTES + 1..=MAX_SECRET_BYTES + TAG_BYTES).contains(&ciphertext.len()) {
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

impl fmt::Debug for ExternalSecretEnvelope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ExternalSecretEnvelope([REDACTED])")
    }
}

impl CredentialCipher {
    pub fn protect_external(
        &self,
        context: &ExternalSecretContext<'_>,
        secret: &ExternalSecret,
    ) -> Result<ExternalSecretEnvelope, CryptoError> {
        let cipher = XChaCha20Poly1305::new_from_slice(context.key(self))
            .map_err(|_| CryptoError::Protection)?;
        let nonce = XNonce::try_generate().map_err(CryptoError::Random)?;
        let ciphertext = cipher
            .encrypt(
                &nonce,
                Payload {
                    msg: secret.expose(),
                    aad: &context.associated_data(),
                },
            )
            .map_err(|_| CryptoError::Protection)?;
        ExternalSecretEnvelope::from_parts(nonce.into(), ciphertext)
    }

    /// Authenticate stored context; callers must independently revalidate authority.
    pub fn recover_external(
        &self,
        context: &ExternalSecretContext<'_>,
        envelope: &ExternalSecretEnvelope,
    ) -> Result<ExternalSecret, CryptoError> {
        let cipher = XChaCha20Poly1305::new_from_slice(context.key(self))
            .map_err(|_| CryptoError::Protection)?;
        let nonce = XNonce::from(envelope.nonce);
        let plaintext = cipher
            .decrypt(
                &nonce,
                Payload {
                    msg: &envelope.ciphertext,
                    aad: &context.associated_data(),
                },
            )
            .map_err(|_| CryptoError::InvalidCiphertext)?;
        ExternalSecret::new(plaintext).map_err(|_| CryptoError::InvalidCiphertext)
    }

    /// Keyed, scope-bound canonical input matching; never a public hash of an API key.
    pub fn external_request_fingerprint(
        &self,
        context: &ExternalSecretContext<'_>,
        operation: &str,
        fields: &[&[u8]],
    ) -> Result<SecretDigest, CryptoError> {
        if operation.is_empty()
            || operation.len() > 128
            || fields.len() > 32
            || fields.iter().any(|field| field.len() > MAX_SECRET_BYTES)
        {
            return Err(CryptoError::InvalidContext);
        }
        let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(self.external_request.as_slice())
            .map_err(|_| CryptoError::Protection)?;
        mac.update(b"chapchoo/external-request\0");
        mac.update(&context.associated_data());
        for field in std::iter::once(operation.as_bytes()).chain(fields.iter().copied()) {
            mac.update(&(field.len() as u64).to_be_bytes());
            mac.update(field);
        }
        Ok(SecretDigest::from_bytes(mac.finalize().into_bytes().into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RootKey;
    use base64::{Engine, engine::general_purpose::URL_SAFE};

    fn fixture_cipher(byte: u8) -> Result<CredentialCipher, CryptoError> {
        CredentialCipher::new(RootKey::parse(URL_SAFE.encode([byte; 32]))?)
    }

    #[test]
    fn scoped_protection_binds_owner_object_kind_and_root() -> Result<(), CryptoError> {
        let cipher = fixture_cipher(1)?;
        let original = ExternalSecretContext::new(
            ExternalSecretOwner::User("Uowner"),
            "HArecord",
            ExternalSecretKind::ChatGpt,
        )?;
        let secret = ExternalSecret::new(b"synthetic-oauth-bundle".to_vec())?;
        let envelope = cipher.protect_external(&original, &secret)?;
        assert_eq!(
            cipher.recover_external(&original, &envelope)?.expose(),
            secret.expose()
        );
        for context in [
            ExternalSecretContext::new(
                ExternalSecretOwner::Team("Uowner"),
                "HArecord",
                ExternalSecretKind::ChatGpt,
            )?,
            ExternalSecretContext::new(
                ExternalSecretOwner::User("Uother"),
                "HArecord",
                ExternalSecretKind::ChatGpt,
            )?,
            ExternalSecretContext::new(
                ExternalSecretOwner::User("Uowner"),
                "HAother",
                ExternalSecretKind::ChatGpt,
            )?,
            ExternalSecretContext::new(
                ExternalSecretOwner::User("Uowner"),
                "HArecord",
                ExternalSecretKind::OpenAiApiKey,
            )?,
            ExternalSecretContext::new(
                ExternalSecretOwner::User("Uowner"),
                "HArecord",
                ExternalSecretKind::PendingDevice,
            )?,
        ] {
            assert!(cipher.recover_external(&context, &envelope).is_err());
        }
        assert!(
            fixture_cipher(2)?
                .recover_external(&original, &envelope)
                .is_err()
        );
        let mut tampered = envelope.ciphertext().to_vec();
        tampered[0] ^= 1;
        let damaged = ExternalSecretEnvelope::from_parts(*envelope.nonce(), tampered)?;
        assert!(cipher.recover_external(&original, &damaged).is_err());
        let mut nonce = *envelope.nonce();
        nonce[0] ^= 1;
        let damaged = ExternalSecretEnvelope::from_parts(nonce, envelope.ciphertext().to_vec())?;
        assert!(cipher.recover_external(&original, &damaged).is_err());
        assert_eq!(format!("{secret:?}"), "ExternalSecret([REDACTED])");
        assert_eq!(
            format!("{envelope:?}"),
            "ExternalSecretEnvelope([REDACTED])"
        );
        assert_eq!(format!("{original:?}"), "ExternalSecretContext([REDACTED])");
        Ok(())
    }

    #[test]
    fn bounds_and_keyed_request_matching_preserve_field_boundaries() -> Result<(), CryptoError> {
        assert!(ExternalSecret::new(Vec::new()).is_err());
        assert!(ExternalSecret::new(vec![1; MAX_SECRET_BYTES + 1]).is_err());
        assert!(ExternalSecretEnvelope::from_parts([0; 24], vec![0; TAG_BYTES]).is_err());
        assert!(
            ExternalSecretContext::new(
                ExternalSecretOwner::User(""),
                "id",
                ExternalSecretKind::ChatGpt
            )
            .is_err()
        );
        assert!(
            ExternalSecretContext::new(
                ExternalSecretOwner::User("owner"),
                "id\n",
                ExternalSecretKind::ChatGpt
            )
            .is_err()
        );
        let context = ExternalSecretContext::new(
            ExternalSecretOwner::Team("Towner"),
            "HArecord",
            ExternalSecretKind::OpenAiApiKey,
        )?;
        let cipher = fixture_cipher(1)?;
        let value = ExternalSecret::new(vec![1; MAX_SECRET_BYTES])?;
        let first = cipher.protect_external(&context, &value)?;
        let second = cipher.protect_external(&context, &value)?;
        assert_ne!(first.nonce(), second.nonce());
        assert_eq!(
            cipher.recover_external(&context, &first)?.expose().len(),
            MAX_SECRET_BYTES
        );
        let hash = cipher.external_request_fingerprint(&context, "register", &[b"ab", b"c"])?;
        assert!(hash.matches_digest(&cipher.external_request_fingerprint(
            &context,
            "register",
            &[b"ab", b"c"]
        )?));
        assert!(!hash.matches_digest(&cipher.external_request_fingerprint(
            &context,
            "register",
            &[b"a", b"bc"]
        )?));
        assert!(!hash.matches_digest(&cipher.external_request_fingerprint(
            &context,
            "replace",
            &[b"ab", b"c"]
        )?));
        assert!(
            !hash.matches_digest(&fixture_cipher(2)?.external_request_fingerprint(
                &context,
                "register",
                &[b"ab", b"c"]
            )?)
        );
        Ok(())
    }
}
