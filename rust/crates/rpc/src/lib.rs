//! Per-call authenticated gRPC boundary over application services.
mod adapters;
mod authentication;
mod component_auth;
mod computers;
mod connection_startup;
mod control_calls;
mod control_streams;
mod conversation_capture;
mod conversation_processor;
mod conversations;
mod delivery;
mod groups;
mod ingress;
mod mapping;
mod memory;
mod native_resources;
mod peer;
mod provider_authentication;
mod reports;
mod server_tools;
mod status;
#[cfg(test)]
#[path = "../tests/support/mod.rs"]
pub(crate) mod test_support;

pub use adapters::AccountServices;
pub use authentication::AuthenticationServices;
pub use component_auth::ComponentAuthenticationServices;
pub use computers::ComputerServices;
pub use connection_startup::reconcile_connection_startups;
pub use control_streams::ComponentConnectionServices;
pub use conversation_capture::ConversationCaptureServices;
pub use conversation_processor::ConversationControlProcessor;
pub use conversations::ConversationServices;
pub use delivery::PeerDelivery;
pub use groups::{ServiceGroup, ServiceGroups};
pub use memory::{MemoryIncoming, memory_transport};
pub use peer::PeerServices;
pub use provider_authentication::{
    ProviderAuthenticator, ProviderBootstrapConfig, ReviewedServiceAccount, ServiceAccountReviewer,
};
pub use server_tools::ServerToolServices;
pub use status::DOMAIN_REASON_METADATA;
