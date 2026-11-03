use std::{error::Error, fmt};

type TestError = Box<dyn Error + Send + Sync>;

struct CompletionFailure {
    primary: TestError,
    secondary: TestError,
}

impl fmt::Debug for CompletionFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CompletionFailure")
            .field("primary", &self.primary.to_string())
            .field("secondary", &self.secondary.to_string())
            .finish()
    }
}

impl fmt::Display for CompletionFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}; additionally: {}", self.primary, self.secondary)
    }
}

impl Error for CompletionFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.primary.as_ref())
    }
}

/// Both operations must already have completed before their errors propagate.
pub fn combine<T, U>(
    primary: Result<T, TestError>,
    secondary: Result<U, TestError>,
) -> Result<(T, U), TestError> {
    match (primary, secondary) {
        (Ok(primary), Ok(secondary)) => Ok((primary, secondary)),
        (Err(primary), Err(secondary)) => Err(Box::new(CompletionFailure { primary, secondary })),
        (Err(error), Ok(_)) | (Ok(_), Err(error)) => Err(error),
    }
}
