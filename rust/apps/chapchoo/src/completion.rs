//! Combine owned completion without replacing a primary failure with cleanup.
use crate::AppError;
use std::{error::Error, fmt};

pub fn complete<T>(
    primary: Result<T, AppError>,
    cleanup: Result<(), AppError>,
) -> Result<T, AppError> {
    match (primary, cleanup) {
        (Err(primary), Err(cleanup)) => Err(Box::new(FailedCleanup { primary, cleanup })),
        (Err(error), Ok(())) | (Ok(_), Err(error)) => Err(error),
        (Ok(value), Ok(())) => Ok(value),
    }
}

/// Both native causes stay private; only the primary participates in source traversal.
struct FailedCleanup {
    primary: AppError,
    cleanup: AppError,
}

impl fmt::Debug for FailedCleanup {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("FailedCleanup([REDACTED])")
    }
}

impl fmt::Display for FailedCleanup {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Selected-component failure was followed by owned cleanup failure")
    }
}

impl Error for FailedCleanup {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let _ = &self.cleanup;
        Some(self.primary.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn failure(message: &'static str) -> AppError {
        std::io::Error::other(message).into()
    }

    #[test]
    fn completion_preserves_values_and_each_single_failure() -> Result<(), AppError> {
        assert_eq!(complete(Ok(7), Ok(()))?, 7);
        assert_eq!(
            complete::<()>(Err(failure("primary")), Ok(()))
                .err()
                .ok_or("Missing primary")?
                .to_string(),
            "primary"
        );
        assert_eq!(
            complete(Ok(7), Err(failure("cleanup")))
                .err()
                .ok_or("Missing cleanup")?
                .to_string(),
            "cleanup"
        );
        Ok(())
    }

    #[test]
    fn combined_failure_retains_both_causes_without_disclosing_them() -> Result<(), AppError> {
        let error = complete::<()>(
            Err(failure("private primary")),
            Err(failure("private cleanup")),
        )
        .err()
        .ok_or("Missing combined failure")?;
        let combined = error
            .downcast_ref::<FailedCleanup>()
            .ok_or("Missing owned causes")?;
        assert_eq!(combined.primary.to_string(), "private primary");
        assert_eq!(combined.cleanup.to_string(), "private cleanup");
        assert_eq!(
            error.source().ok_or("Missing primary source")?.to_string(),
            "private primary"
        );
        assert!(!error.to_string().contains("private"));
        assert_eq!(format!("{error:?}"), "FailedCleanup([REDACTED])");
        Ok(())
    }
}
