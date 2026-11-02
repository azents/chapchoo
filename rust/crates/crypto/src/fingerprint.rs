use sha2::{Digest, Sha256};

/// Canonical field boundaries prevent ambiguous concatenations across operations.
/// Inputs are typed, validated operation fields rather than serialized raw JSON.
pub fn payload_fingerprint(domain: &str, fields: &[&[u8]]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"chapchoo/payload-fingerprint\0");
    for field in std::iter::once(domain.as_bytes()).chain(fields.iter().copied()) {
        hash.update((field.len() as u64).to_be_bytes());
        hash.update(field);
    }
    hash.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operation_and_field_boundaries_are_part_of_payload_identity() {
        assert_eq!(
            payload_fingerprint("create", &[b"a", b"bc"]),
            payload_fingerprint("create", &[b"a", b"bc"])
        );
        assert_ne!(
            payload_fingerprint("create", &[b"a", b"bc"]),
            payload_fingerprint("create", &[b"ab", b"c"])
        );
        assert_ne!(
            payload_fingerprint("create", &[b"a"]),
            payload_fingerprint("restart", &[b"a"])
        );
        assert_ne!(
            payload_fingerprint("create", &[]),
            payload_fingerprint("create", &[b""])
        );
    }
}
