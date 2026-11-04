/// A canonical public handle, distinct from an entity ID or access grant.
///
/// This value validates syntax only. Uniqueness and post-creation immutability
/// belong to the accepting repository operation.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Handle(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error(
    "Handle must contain 3 to 16 lowercase ASCII letters, digits or hyphens, with alphanumeric ends"
)]
pub struct InvalidHandle;

impl Handle {
    pub const MIN_LENGTH: usize = 3;
    pub const MAX_LENGTH: usize = 16;

    /// Parse the exact supplied value without trimming, casing or replacement.
    pub fn parse(value: &str) -> Result<Self, InvalidHandle> {
        let bytes = value.as_bytes();
        let alphanumeric = |byte: u8| byte.is_ascii_lowercase() || byte.is_ascii_digit();
        if !(Self::MIN_LENGTH..=Self::MAX_LENGTH).contains(&bytes.len())
            || !bytes.first().copied().is_some_and(alphanumeric)
            || !bytes.last().copied().is_some_and(alphanumeric)
            || !bytes
                .iter()
                .copied()
                .all(|byte| alphanumeric(byte) || byte == b'-')
        {
            return Err(InvalidHandle);
        }
        Ok(Self(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for Handle {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Handle([REDACTED])")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_the_exact_common_contract() -> Result<(), InvalidHandle> {
        for value in [
            "abc",
            "012",
            "a--b",
            "dubu",
            "design-team",
            "a12345678901234z",
        ] {
            assert_eq!(Handle::parse(value)?.as_str(), value);
        }
        Ok(())
    }

    #[test]
    fn rejects_without_rewriting() {
        for value in [
            "",
            "a",
            "my",
            " a-b",
            "a-b ",
            "Dubu",
            "-abc",
            "abc-",
            "a_b",
            "a.b",
            "a/b",
            "a\nb",
            "abc\n",
            "abc\r",
            "a\0b",
            "두부",
            "ｄｕｂｕ",
            "a123456789012345z",
        ] {
            assert_eq!(Handle::parse(value), Err(InvalidHandle), "{value:?}");
        }
    }

    #[test]
    fn debug_does_not_disclose_user_supplied_handle() -> Result<(), InvalidHandle> {
        assert_eq!(
            format!("{:?}", Handle::parse("private-team")?),
            "Handle([REDACTED])"
        );
        Ok(())
    }
}
