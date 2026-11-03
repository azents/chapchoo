//! Engine-specific completed storage operations for one installation.

mod accounts;
mod agent_cleanup;
mod challenges;
mod component_keys;
mod computer_executions;
mod computers;
mod connector_keys;
mod delivery;
mod directory;
mod display;
mod identity;
mod models;
mod native_resources;
mod preparation;
mod sessions;
mod store;
mod teams;

pub use accounts::{GrantDraft, GrantKind, GrantSummary, NewAccount, ProofPreview};
pub use challenges::{EmailLoginOutcome, NewChallenge};
pub use component_keys::{
    ComponentKeyDraft, ComponentKeyPolicy, ComponentKeyResult, ProviderOwner,
};
pub use computer_executions::{
    ComputerRequest, ExecutionDraft, ExecutionReservation, ExecutionResources, ManagedComputer,
    ManagedResourceKind, NativeRetirementAdmission, ProviderSource,
};
pub use computers::{
    AgentRemoval, AgentSummary, ComputerInventorySummary, ComputerSummary, CreateCloudAgent,
    CreationOutcome, LifecycleRequest, ProviderCapabilities, ProviderChoice, ProviderIdentity,
    ProviderInventorySummary, ResourceCursor,
};
pub use connector_keys::{
    ConnectorIdentity, ConnectorSessionDraft, ConnectorSessionResult, ConnectorSource,
};
pub use delivery::{DeliveryComputer, DeliveryExecution, DeliverySnapshot};
pub use directory::{
    ComponentRoute, ConnectionOwner, ControlRegistration, DirectoryPolicy, ResolvedRoute,
};
pub use models::{CredentialSnapshot, UserSummary, VerifiedCredential};
pub use native_resources::{ManagedResource, ResourceEvidence, ResourceReport};
pub use preparation::{
    POSTGRES_MIGRATIONS, PreparationError, SQLITE_MIGRATIONS, postgres_ready, prepare_postgres,
    prepare_sqlite, sqlite_ready,
};
pub use sessions::{NewSession, SessionPolicy, SessionResult};
pub use store::{Store, StoreError};
pub use teams::{
    CreateTeam, InvitationResult, InvitationStatus, InvitationSummary, InviteMember, MemberSummary,
    TeamPatch, TeamSignupGrant, TeamSummary,
};
