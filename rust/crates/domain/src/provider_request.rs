/// Time-bearing admission metadata, not authentication evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderKeyRequestId {
    value: String,
    created_at_ms: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidProviderKeyRequest {
    #[error("Invalid Provider credential request ID")]
    Invalid,
    #[error("Provider credential request retry period expired")]
    Expired,
    #[error("Provider credential request time is too far in the future")]
    Future,
}

impl ProviderKeyRequestId {
    pub const RETRY_MS: i64 = 900_000;
    pub const FUTURE_TOLERANCE_MS: i64 = 60_000;

    pub fn new(created_at_ms: i64, nonce: &str) -> Result<Self, InvalidProviderKeyRequest> {
        Self::parse(&format!("{created_at_ms}:{nonce}"))
    }

    pub fn parse(value: &str) -> Result<Self, InvalidProviderKeyRequest> {
        let (timestamp, nonce) = value
            .split_once(':')
            .ok_or(InvalidProviderKeyRequest::Invalid)?;
        let created_at_ms = timestamp
            .parse::<i64>()
            .map_err(|_| InvalidProviderKeyRequest::Invalid)?;
        if created_at_ms < 0
            || timestamp != created_at_ms.to_string()
            || created_at_ms.checked_add(Self::RETRY_MS).is_none()
            || nonce.is_empty()
            || nonce.len() > 64
            || !nonce
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
        {
            return Err(InvalidProviderKeyRequest::Invalid);
        }
        Ok(Self {
            value: value.to_owned(),
            created_at_ms,
        })
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }

    pub fn retry_expires_at(&self) -> i64 {
        // Construction checked overflow.
        self.created_at_ms + Self::RETRY_MS
    }

    pub fn admit_at(&self, now_ms: i64) -> Result<(), InvalidProviderKeyRequest> {
        if now_ms < 0 {
            return Err(InvalidProviderKeyRequest::Invalid);
        }
        if self.created_at_ms.saturating_sub(now_ms) > Self::FUTURE_TOLERANCE_MS {
            return Err(InvalidProviderKeyRequest::Future);
        }
        if now_ms >= self.retry_expires_at() {
            return Err(InvalidProviderKeyRequest::Expired);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_and_clock_boundaries() -> Result<(), InvalidProviderKeyRequest> {
        let id = ProviderKeyRequestId::new(1_000_000, "opaque-nonce")?;
        assert_eq!(ProviderKeyRequestId::parse(id.as_str())?, id);
        assert!(id.admit_at(940_000).is_ok());
        assert_eq!(id.admit_at(939_999), Err(InvalidProviderKeyRequest::Future));
        assert!(id.admit_at(1_899_999).is_ok());
        assert_eq!(
            id.admit_at(1_900_000),
            Err(InvalidProviderKeyRequest::Expired)
        );
        assert!(id.admit_at(-1).is_err());
        Ok(())
    }

    #[test]
    fn malformed_and_overflow_ids_reject() {
        for id in [
            "opaque",
            "-1:nonce",
            "01:nonce",
            "+1:nonce",
            "1:",
            "1:a:b",
            "1:a b",
            "9223372036854775807:nonce",
        ] {
            assert!(ProviderKeyRequestId::parse(id).is_err(), "{id}");
        }
        assert!(ProviderKeyRequestId::new(0, &"x".repeat(65)).is_err());
    }
}
