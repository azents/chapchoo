use std::fmt;

/// One whole-address case-insensitive lookup identity with preserved spelling.
#[derive(Clone, PartialEq, Eq)]
pub struct EmailAddress {
    delivery: String,
    lookup: String,
}

impl EmailAddress {
    pub fn parse(input: &str) -> Result<Self, InvalidEmail> {
        let delivery = input.trim();
        email_address::EmailAddress::parse_with_options(
            delivery,
            email_address::Options::default().without_display_text(),
        )
        .map_err(|_| InvalidEmail)?;
        Ok(Self {
            delivery: delivery.to_owned(),
            lookup: delivery.to_lowercase(),
        })
    }

    pub fn delivery(&self) -> &str {
        &self.delivery
    }

    pub fn lookup_key(&self) -> &str {
        &self.lookup
    }
}

impl fmt::Debug for EmailAddress {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("EmailAddress([REDACTED])")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("A valid email address is required")]
pub struct InvalidEmail;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_and_normalizes_without_alias_merging() -> Result<(), InvalidEmail> {
        let email = EmailAddress::parse(" \tUser.Name+Tag@Example.COM\n")?;
        assert_eq!(email.delivery(), "User.Name+Tag@Example.COM");
        assert_eq!(email.lookup_key(), "user.name+tag@example.com");
        assert_eq!(
            email.lookup_key(),
            EmailAddress::parse("USER.NAME+TAG@example.com")?.lookup_key()
        );
        assert_ne!(
            email.lookup_key(),
            EmailAddress::parse("username@example.com")?.lookup_key()
        );
        assert!(!format!("{email:?}").contains("User"));
        Ok(())
    }

    #[test]
    fn rejects_non_address_inputs_without_echoing_them() {
        for value in ["invalid", "Name <a@example.com>"] {
            assert!(EmailAddress::parse(value).is_err());
        }
    }
}
