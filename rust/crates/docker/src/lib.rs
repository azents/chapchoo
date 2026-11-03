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
    CreationRejected,
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
    pub fn definitely_rejected(&self) -> bool {
        matches!(self, Self::CreationRejected)
    }
    /// Only the container preparation boundary uses this classification, after
    /// previous allocations have settled and their report ACKs were received.
    pub fn container_creation_retry_safe(&self) -> bool {
        matches!(self, Self::CreationRejected | Self::ContainerCapacity)
    }
}
