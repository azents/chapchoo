//! Per-call authenticated gRPC boundary over application services.
mod adapters;
mod ingress;
mod mapping;
mod status;

pub use adapters::AccountServices;
pub use status::DOMAIN_REASON_METADATA;
