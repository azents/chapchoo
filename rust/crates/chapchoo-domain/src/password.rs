use std::fmt;
use zeroize::Zeroizing;

const SPECIAL: &[u8] = b"!@#$%^&*()_+-=[]{};':\"\\|,.<>/?";

/// A policy-valid password. Deliberately has no Display/serialization API.
pub struct Password(Zeroizing<String>);

impl Password {
    pub fn parse(input: String) -> Result<Self, PasswordPolicyError> {
        let input = Zeroizing::new(input);
        if !(8..=64).contains(&input.len()) {
            return Err(PasswordPolicyError::Length);
        }
        if !input.bytes().all(|value| (b' '..=b'~').contains(&value)) {
            return Err(PasswordPolicyError::Characters);
        }
        let classes = [
            input.bytes().any(|value| value.is_ascii_uppercase()),
            input.bytes().any(|value| value.is_ascii_lowercase()),
            input.bytes().any(|value| value.is_ascii_digit()),
            input.bytes().any(|value| SPECIAL.contains(&value)),
        ];
        if classes.into_iter().filter(|found| *found).count() < 3 {
            return Err(PasswordPolicyError::Classes);
        }
        Ok(Self(input))
    }

    /// Used only at a password-hashing boundary; callers must not log this value.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Password {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Password([REDACTED])")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum PasswordPolicyError {
    #[error("Password must contain 8 through 64 characters")]
    Length,
    #[error("Password must contain only printable ASCII characters")]
    Characters,
    #[error("Password must use at least three character classes")]
    Classes,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enforces_boundaries_and_all_three_class_combinations() {
        for value in ["ABcd1234", "ABcd!!!!", "AB1234!!", "ab1234!!", "Aa1!xxxx"] {
            assert!(Password::parse(value.to_owned()).is_ok(), "{value}");
        }
        for length in [8, 64] {
            assert!(Password::parse(format!("Aa1{}", "x".repeat(length - 3))).is_ok());
        }
        for length in [7, 65] {
            assert_eq!(
                Password::parse(format!("Aa1{}", "x".repeat(length - 3))).err(),
                Some(PasswordPolicyError::Length)
            );
        }
        for value in ["Abcdefgh", "1234!!!!", "Aa      ", "Aa1\nxxxx", "Aa1éxxxx"] {
            assert!(Password::parse(value.to_owned()).is_err());
        }
    }

    #[test]
    fn never_trims_or_exposes_through_debug() -> Result<(), PasswordPolicyError> {
        let value = Password::parse(" Aa1xxxx ".to_owned())?;
        assert_eq!(value.expose(), " Aa1xxxx ");
        assert_eq!(format!("{value:?}"), "Password([REDACTED])");
        Ok(())
    }

    #[test]
    fn every_character_follows_the_printable_ascii_boundary() {
        for byte in 0_u8..=127 {
            let candidate = format!("Aa1!xxx{}", char::from(byte));
            assert_eq!(
                Password::parse(candidate).is_ok(),
                (b' '..=b'~').contains(&byte)
            );
        }
        // Printable characters outside the special alphabet count only as length.
        assert!(Password::parse("Aa      ".to_owned()).is_err());
        assert!(Password::parse("Aa1     ".to_owned()).is_ok());
    }
}
