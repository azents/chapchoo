//! Explicit application orchestration without persistence handles or transport types.
mod authentication;
mod computers;
mod error;
mod mail;
mod password;
mod service;
mod time;

pub use authentication::{
    AuthenticationProgress, AuthenticationService, DEVICE_VERIFICATION_URI, DeviceApproval,
    DeviceAuthenticationProvider, DevicePollOutcome, DeviceProviderFailure, DeviceProviderFuture,
};
pub use chapchoo_crypto::{BearerSecret, EmailCode, ExternalSecret, PasswordCost, SecretDigest};
pub use chapchoo_domain::{
    DisplaySettings, DockerDaemonId, EmailAddress, MembershipRole, Password, ProviderKind,
    ProviderSubstrate, TeamKind, TeamRole,
};
pub use chapchoo_persistence::{
    AgentAuthenticationSelection, AgentCreationResult, AgentProfileImage, AgentRemoval,
    AgentSummary, AuthenticationAcceptedResult, AuthenticationKind, AuthenticationOperation,
    AuthenticationRequestResult, AuthenticationScope, AuthenticationSummary, ComponentKeyPolicy,
    ComponentKeyResult, ComponentRoute, ComputerInventorySummary, ComputerRequest, ComputerSummary,
    ConnectionOwner, ConnectorIdentity, ConnectorSessionResult, ConnectorSource,
    ControlRegistration, CreateAgentProfile, CreateCloudAgent, CreationOutcome,
    DeleteAuthentication, DeleteAuthenticationOutcome, DeliveryComputer, DeliveryExecution,
    DeliverySnapshot, DeviceAuthenticationState, DeviceAuthenticationStatus,
    DeviceAuthenticationTokens, DeviceAuthorizationDraft, DirectoryPolicy, ExecutionReservation,
    ExpectedAgentAuthentication, GrantKind, GrantSummary, InvitationStatus, InvitationSummary,
    LifecycleRequest, MAX_AUTHENTICATION_EXPIRY_MILLIS, ManagedComputer, ManagedResource,
    ManagedResourceKind, MemberSummary, NativeRetirementAdmission, ProofPreview,
    ProviderCapabilities, ProviderChoice, ProviderCredentialResult, ProviderIdentity,
    ProviderInventorySummary, ProviderSource, ReauthenticateTarget, RegisterApiKey,
    RenameAuthentication, ReplaceApiKey, ResolvedRoute, ResourceCursor, ResourceEvidence,
    ResourceReport, SelectAgentAuthentication, SessionPolicy, SessionResult,
    StartDeviceAuthentication, TeamPatch, TeamSummary, UpdateAgentProfile, UserSummary,
};
pub use computers::{ComputerPolicy, ComputersService, PreparedComponentConnection};
pub use error::{DomainReason, InfrastructureReport, ServiceError};
pub use mail::{MailFailure, MailFuture, MailMessage, MailPurpose, Mailer};
pub use password::PasswordWork;
pub use service::{
    AccountsConfig, AccountsService, CodeChallenge, GrantsPage, IssuedGrant, LoginMethods,
};
pub use time::AcceptanceTime;
