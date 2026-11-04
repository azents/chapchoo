//! Explicit application orchestration without persistence handles or transport types.
mod computers;
mod error;
mod mail;
mod password;
mod service;
mod time;

pub use chapchoo_crypto::{BearerSecret, EmailCode, PasswordCost, SecretDigest};
pub use chapchoo_domain::{
    DisplaySettings, DockerDaemonId, EmailAddress, MembershipRole, Password, ProviderKind,
    ProviderSubstrate, TeamKind, TeamRole,
};
pub use chapchoo_persistence::{
    AgentCreationResult, AgentProfileImage, AgentRemoval, AgentSummary, ComponentKeyPolicy,
    ComponentKeyResult, ComponentRoute, ComputerInventorySummary, ComputerRequest, ComputerSummary,
    ConnectionOwner, ConnectorIdentity, ConnectorSessionResult, ConnectorSource,
    ControlRegistration, CreateAgentProfile, CreateCloudAgent, CreationOutcome, DeliveryComputer,
    DeliveryExecution, DeliverySnapshot, DirectoryPolicy, ExecutionReservation, GrantKind,
    GrantSummary, InvitationStatus, InvitationSummary, LifecycleRequest, ManagedComputer,
    ManagedResource, ManagedResourceKind, MemberSummary, NativeRetirementAdmission, ProofPreview,
    ProviderCapabilities, ProviderChoice, ProviderCredentialResult, ProviderIdentity,
    ProviderInventorySummary, ProviderSource, ResolvedRoute, ResourceCursor, ResourceEvidence,
    ResourceReport, SessionPolicy, SessionResult, TeamPatch, TeamSummary, UpdateAgentProfile,
    UserSummary,
};
pub use computers::{ComputerPolicy, ComputersService, PreparedComponentConnection};
pub use error::{DomainReason, InfrastructureReport, ServiceError};
pub use mail::{MailFailure, MailFuture, MailMessage, MailPurpose, Mailer};
pub use password::PasswordWork;
pub use service::{
    AccountsConfig, AccountsService, CodeChallenge, GrantsPage, IssuedGrant, LoginMethods,
};
pub use time::AcceptanceTime;
