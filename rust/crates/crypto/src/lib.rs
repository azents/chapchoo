//! Fixed-root secret protection, canonical fingerprints and scoped credential verification.
//! Storage, network delivery and credential mutation remain application boundaries.

mod code;
mod error;
mod fingerprint;
mod password;
mod provider;
mod root;
mod system_owner;
mod token;

pub use code::{CodeVerifier, EmailCode};
pub use error::CryptoError;
pub use fingerprint::payload_fingerprint;
pub use password::{PasswordCost, PasswordHash, PasswordHasher};
pub use provider::ProviderBootstrapCredential;
pub use root::{CredentialCipher, InstallationKeyCheck, RecoveryEnvelope, RootKey};
pub use system_owner::{
    BootstrapAuthenticationSource, BootstrapProviderPermit, VerifiedSystemOwner,
};
pub use token::{BearerSecret, SecretDigest};
