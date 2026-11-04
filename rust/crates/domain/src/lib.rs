//! Protocol- and persistence-independent account values.
//!
//! Credential contents cannot be accessed without the explicit boundary method:
//!
//! ```compile_fail
//! let password = chapchoo_domain::Password::parse("Aa1!xxxx".to_owned()).unwrap();
//! let plaintext = password.0;
//! ```
//!
//! Neither persistence nor protocol adapters are dependencies of this crate:
//!
//! ```compile_fail
//! use diesel::Connection;
//! use tonic::Request;
//! ```

mod agent_profile;
mod computer;
mod computer_resource;
mod display;
mod email;
mod handle;
mod password;
mod provider_request;
mod role;

pub use agent_profile::{AgentPalette, InvalidAgentPalette};
pub use computer::{
    ActionAvailability, ActionBlockReason, ComputerActionContext, ComputerKind, ConnectionStatus,
    DesiredState, InvalidComputerValue, LifecycleAction, LifecyclePhase, LifecycleProgress,
    ObservedPower, PowerCapabilities, ResourcePresence, RestorationBlockReason,
    RestorationEvidence, SavedEnvironment,
};
pub use computer_resource::{
    DockerContainerId, DockerDaemonId, DockerVolumeIdentity, InvalidNativeResource, KubernetesUid,
    ManagedResourceKind, NativeAllocation, NativeIdentity, NativeReservation, NativeScope,
    ProviderKind, ProviderSubstrate, ResourceRole,
};
pub use display::{DisplayPatch, DisplaySettings, InvalidDisplaySettings};
pub use email::{EmailAddress, InvalidEmail};
pub use handle::{Handle, InvalidHandle};
pub use password::{Password, PasswordPolicyError};
pub use provider_request::{InvalidProviderKeyRequest, ProviderKeyRequestId};
pub use role::{MembershipRole, TeamKind, TeamRole};
