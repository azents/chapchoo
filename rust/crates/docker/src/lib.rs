//! Selected-daemon Docker SDK adapter. Product database authority stays in Control.
mod adapter;
pub mod connection;
mod containers;
mod images;
pub mod transport;
mod volumes;
pub use adapter::{Config, Docker, ImagePolicy};
pub use containers::{
    Bootstrap, PreparedContainer, Retirement, StoppedContainer, UpdatedContainer,
};
pub use images::PreparedImage;
pub use volumes::{Observation, PreparedVolume, Presence};

#[derive(thiserror::Error)]
pub enum Error {
    #[error("Selected Docker connection configuration is invalid or unavailable")]
    Configuration,
    #[error("Selected Docker connection mode is not supported")]
    UnsupportedTransport,
    #[error(
        "Selected Docker TCP proxy routing is not supported; no direct connection was attempted"
    )]
    UnsupportedProxy,
    #[error("Selected Docker transport is unavailable; its external outcome may be uncertain")]
    Transport,
    #[error("Selected Docker daemon identity changed or cannot be verified")]
    DaemonIdentity,
    #[error("Selected Docker daemon cannot enforce the required Linux workload policy")]
    WorkloadPolicy,
    #[error("Docker operation failed; its external outcome may be uncertain")]
    Engine(#[from] bollard::errors::Error),
    #[error("Managed Docker resource identity or ownership does not match")]
    Ownership,
    #[error("The original saved environment is missing or unverified")]
    MissingEnvironment,
    #[error("Docker explicitly rejected resource creation")]
    CreationRejected(#[source] bollard::errors::Error),
    #[error("Local Docker correlation capacity is exhausted; container creation was not submitted")]
    ContainerCapacity,
    #[error("The native operation outcome or required absence is not confirmed")]
    Uncertain,
}
impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DockerError([REDACTED])")
    }
}
impl Error {
    /// Static SDK failure category and numeric HTTP status; no native payload.
    pub fn engine_diagnostic(&self) -> Option<(&'static str, Option<u16>)> {
        use bollard::errors::Error as Sdk;
        let source = match self {
            Self::Engine(source) | Self::CreationRejected(source) => source,
            _ => return None,
        };
        Some(match source {
            Sdk::DockerResponseServerError { status_code, .. } => ("server", Some(*status_code)),
            Sdk::JsonDataError { .. } | Sdk::JsonSerdeError { .. } => ("decode", None),
            Sdk::RequestTimeoutError => ("timeout", None),
            Sdk::IOError { .. } => ("io", None),
            Sdk::HyperResponseError { .. } | Sdk::HttpClientError { .. } => ("http", None),
            Sdk::APIVersionParseError { .. } => ("version", None),
            _ => ("other", None),
        })
    }
    pub fn definitely_rejected(&self) -> bool {
        matches!(self, Self::CreationRejected(_))
    }
    /// Only the container preparation boundary uses this classification, after
    /// previous allocations have settled and their report ACKs were received.
    pub fn container_creation_retry_safe(&self) -> bool {
        matches!(self, Self::CreationRejected(_) | Self::ContainerCapacity)
    }
}

#[cfg(test)]
mod diagnostic_tests {
    use super::Error;
    use bollard::errors::Error as Sdk;

    #[test]
    fn engine_diagnostics_publish_only_static_category_and_numeric_status() {
        let response = Error::Engine(Sdk::DockerResponseServerError {
            status_code: 400,
            message: "private-native-response".into(),
        });
        assert_eq!(response.engine_diagnostic(), Some(("server", Some(400))));
        let decode = Error::Engine(Sdk::JsonDataError {
            message: "private-native-payload".into(),
            column: 42,
        });
        assert_eq!(decode.engine_diagnostic(), Some(("decode", None)));
        let io = Error::Engine(Sdk::IOError {
            err: std::io::Error::other("private-endpoint"),
        });
        assert_eq!(io.engine_diagnostic(), Some(("io", None)));
        assert_eq!(Error::WorkloadPolicy.engine_diagnostic(), None);
    }

    #[test]
    fn rejected_creation_retains_private_cause_without_exposing_native_messages()
    -> Result<(), Box<dyn std::error::Error>> {
        let failure = Error::CreationRejected(Sdk::DockerResponseServerError {
            status_code: 403,
            message: "private-native-rejection-canary".into(),
        });
        assert!(failure.definitely_rejected());
        assert!(failure.container_creation_retry_safe());
        assert_eq!(failure.engine_diagnostic(), Some(("server", Some(403))));
        assert!(
            !failure
                .to_string()
                .contains("private-native-rejection-canary")
        );
        assert!(!format!("{failure:?}").contains("private-native-rejection-canary"));
        let cause = std::error::Error::source(&failure).ok_or("Missing original SDK cause")?;
        assert!(matches!(
            cause.downcast_ref::<Sdk>(),
            Some(Sdk::DockerResponseServerError { status_code: 403, message })
                if message == "private-native-rejection-canary"
        ));
        Ok(())
    }
}
