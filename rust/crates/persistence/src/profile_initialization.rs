//! One-time initialization rules for persisted profiles and handles.
//!
//! These rules run only in explicit preparation. Normal resource creation accepts
//! the client's final handle and never calls this migration-only generator.

use crate::StoreError;
use chapchoo_crypto::payload_fingerprint;
use chapchoo_domain::{AgentPalette, Handle};
use std::collections::BTreeSet;

#[derive(Debug)]
pub(crate) struct BuiltinProfileImage {
    pub name: &'static str,
    pub asset_key: &'static str,
}

/// Stable initial backfill order, independent of catalogue presentation order.
pub(crate) const INITIAL_IMAGES: [BuiltinProfileImage; 6] = [
    BuiltinProfileImage {
        name: "retriever",
        asset_key: "01-retriever",
    },
    BuiltinProfileImage {
        name: "bichon",
        asset_key: "02-bichon",
    },
    BuiltinProfileImage {
        name: "corgi",
        asset_key: "03-corgi",
    },
    BuiltinProfileImage {
        name: "dachshund",
        asset_key: "04-dachshund",
    },
    BuiltinProfileImage {
        name: "husky",
        asset_key: "05-husky",
    },
    BuiltinProfileImage {
        name: "border-collie",
        asset_key: "06-border-collie",
    },
];

fn hash_index(domain: &str, original_id: &str) -> usize {
    let digest = payload_fingerprint(domain, &[original_id.as_bytes()]);
    // Interpret the complete digest as a big-endian integer modulo six.
    digest.iter().fold(0_usize, |value, byte| {
        (value * 256 + usize::from(*byte)) % INITIAL_IMAGES.len()
    })
}

pub(crate) fn initial_image(original_id: &str) -> &'static BuiltinProfileImage {
    &INITIAL_IMAGES[hash_index("agent-image", original_id)]
}

pub(crate) fn initial_palette(original_id: &str) -> AgentPalette {
    AgentPalette::INITIAL[hash_index("agent-color", original_id)]
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum HandleResource {
    Agent,
    Team,
}

impl HandleResource {
    fn fallback(self) -> &'static str {
        match self {
            Self::Agent => "agent",
            Self::Team => "team",
        }
    }
}

fn automatic_base(value: &str) -> String {
    let ascii = any_ascii::any_ascii(value);
    let mut result = String::new();
    let mut separator = false;
    for byte in ascii.bytes() {
        if byte.is_ascii_alphanumeric() {
            if separator && !result.is_empty() {
                result.push('-');
            }
            result.push(char::from(byte.to_ascii_lowercase()));
            separator = false;
        } else {
            separator = true;
        }
    }
    result
}

fn random_suffix() -> Result<String, StoreError> {
    const ALPHABET: &[u8; 36] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut suffix = String::new();
    while suffix.len() < 5 {
        let mut bytes = [0_u8; 16];
        getrandom::fill(&mut bytes).map_err(chapchoo_crypto::CryptoError::Random)?;
        for byte in bytes {
            if byte < 252 {
                suffix.push(char::from(ALPHABET[usize::from(byte) % ALPHABET.len()]));
                if suffix.len() == 5 {
                    return Ok(suffix);
                }
            }
        }
    }
    Ok(suffix)
}

/// Reserve already-valid live handles before assigning any transformed value.
/// The caller chooses stable-ID order and whether a deleted Team is in scope.
#[derive(Debug)]
pub(crate) struct HandleInitialization {
    used: BTreeSet<String>,
}

impl HandleInitialization {
    pub(crate) fn new(existing: impl IntoIterator<Item = Handle>) -> Self {
        Self {
            used: existing
                .into_iter()
                .map(|handle| handle.as_str().to_owned())
                .collect(),
        }
    }

    pub(crate) fn assign(
        &mut self,
        resource: HandleResource,
        source: &str,
    ) -> Result<Handle, StoreError> {
        self.assign_with(resource, source, random_suffix)
    }

    fn assign_with(
        &mut self,
        resource: HandleResource,
        source: &str,
        mut random: impl FnMut() -> Result<String, StoreError>,
    ) -> Result<Handle, StoreError> {
        let converted = automatic_base(source);
        let base = if converted.is_empty() {
            resource.fallback()
        } else {
            converted.as_str()
        };
        let candidate = base[..base.len().min(Handle::MAX_LENGTH)].trim_end_matches('-');
        if !converted.is_empty()
            && let Ok(handle) = Handle::parse(candidate)
            && self.used.insert(candidate.to_owned())
        {
            return Ok(handle);
        }
        // An exceptionally unlucky namespace must fail preparation atomically,
        // not enter an unbounded loop or discard another resource.
        for _ in 0..1024 {
            let suffix = random()?;
            if suffix.len() != 5
                || !suffix
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
            {
                return Err(StoreError::Integrity);
            }
            let head =
                base[..base.len().min(Handle::MAX_LENGTH - suffix.len() - 1)].trim_end_matches('-');
            let value = format!("{head}-{suffix}");
            let handle = Handle::parse(&value).map_err(|_| StoreError::Integrity)?;
            if self.used.insert(value) {
                return Ok(handle);
            }
        }
        Err(StoreError::Conflict)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_suffix() -> Result<String, StoreError> {
        Ok("k7m2q".to_owned())
    }

    #[test]
    fn initialization_matches_client_candidate_examples() -> Result<(), StoreError> {
        for (resource, name, expected) in [
            (HandleResource::Agent, "두부", "dubu"),
            (HandleResource::Team, "디자인 팀", "dijain-tim"),
            (HandleResource::Agent, "개발 도우미", "gaebal-doumi"),
            (HandleResource::Agent, "AI", "ai-k7m2q"),
            (HandleResource::Agent, "!!!", "agent-k7m2q"),
            (HandleResource::Team, "!!!", "team-k7m2q"),
            (
                HandleResource::Team,
                "abcdefghijklmnop-more",
                "abcdefghijklmnop",
            ),
            (
                HandleResource::Team,
                "abcdefghijklmno-more",
                "abcdefghijklmno",
            ),
        ] {
            let mut handles = HandleInitialization::new([]);
            let handle = handles.assign_with(resource, name, fixture_suffix)?;
            assert_eq!(handle.as_str(), expected);
        }
        Ok(())
    }

    #[test]
    fn existing_canonical_values_win_and_collisions_preserve_the_base() -> Result<(), StoreError> {
        let existing = Handle::parse("abcdefghijklmnop").map_err(|_| StoreError::Integrity)?;
        let mut handles = HandleInitialization::new([existing]);
        let handle = handles.assign_with(
            HandleResource::Team,
            "abcdefghijklmnop-more",
            fixture_suffix,
        )?;
        assert_eq!(handle.as_str(), "abcdefghij-k7m2q");
        Ok(())
    }

    #[test]
    fn bounded_exhaustion_and_random_failure_do_not_accept_duplicates() -> Result<(), StoreError> {
        let existing = Handle::parse("team-k7m2q").map_err(|_| StoreError::Integrity)?;
        let mut handles = HandleInitialization::new([existing]);
        assert!(matches!(
            handles.assign_with(HandleResource::Team, "!!!", fixture_suffix),
            Err(StoreError::Conflict)
        ));
        assert!(matches!(
            handles.assign_with(HandleResource::Agent, "!!!", || Err(StoreError::Integrity)),
            Err(StoreError::Integrity)
        ));
        Ok(())
    }

    #[test]
    fn saved_defaults_are_deterministic_and_domain_separated() {
        // Golden migration assignments protect the owned domain/order/encoding
        // contract, not the SHA-256 dependency's implementation.
        for (original_id, image, palette) in [
            ("old-agent", "border-collie", AgentPalette::Lilac),
            ("another-agent", "bichon", AgentPalette::Teal),
            ("0000000000000000", "corgi", AgentPalette::Lilac),
        ] {
            assert_eq!(initial_image(original_id).name, image);
            assert_eq!(initial_palette(original_id), palette);
        }
        let combinations: BTreeSet<_> = (0..100)
            .map(|i| {
                let id = format!("old-{i}");
                (initial_image(&id).asset_key, initial_palette(&id).as_str())
            })
            .collect();
        // Character and color are not accidentally the same modulo-six draw.
        assert!(combinations.len() > 6);
    }
}
