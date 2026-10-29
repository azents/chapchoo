//! Per-call authenticated gRPC boundary over application services.
mod adapter;
mod ingress;
mod mapping;
mod status;

pub use adapter::RpcService;
pub use status::DOMAIN_REASON_METADATA;
