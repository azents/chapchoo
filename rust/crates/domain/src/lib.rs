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

mod computer;
mod computer_resource;
mod display;
mod email;
mod password;
mod role;

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
pub use password::{Password, PasswordPolicyError};
pub use role::{MembershipRole, TeamKind, TeamRole};
