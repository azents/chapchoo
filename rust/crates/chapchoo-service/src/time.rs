use std::time::{SystemTime, UNIX_EPOCH};

use crate::{DomainReason, ServiceError};

/// Read the acceptance instant after expensive work. Production adapters use
/// System; isolated callers can supply an explicit deterministic instant.
#[derive(Debug, Clone, Copy)]
pub enum AcceptanceTime {
    System,
    Fixed(i64),
}

impl AcceptanceTime {
    pub fn now(self) -> Result<i64, ServiceError> {
        match self {
            Self::System => {
                let elapsed = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(ServiceError::infrastructure)?;
                i64::try_from(elapsed.as_millis()).map_err(ServiceError::infrastructure)
            }
            Self::Fixed(value) if value >= 0 => Ok(value),
            Self::Fixed(_) => Err(ServiceError::domain(DomainReason::InvalidInput)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_and_system_times_are_fallible_and_epoch_based() -> Result<(), ServiceError> {
        assert_eq!(AcceptanceTime::Fixed(1234).now()?, 1234);
        assert!(AcceptanceTime::Fixed(-1).now().is_err());
        let before = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(ServiceError::infrastructure)?
            .as_millis();
        let actual = AcceptanceTime::System.now()?;
        let after = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(ServiceError::infrastructure)?
            .as_millis();
        assert!((before..=after).contains(&(actual as u128)));
        Ok(())
    }
}
