//! Explicit application orchestration without persistence handles or transport types.
mod computers;
mod error;
mod mail;
mod password;
mod service;
mod time;

pub use chapchoo_crypto::{
    BearerSecret, EmailCode, PasswordCost, SecretDigest, SystemOwnerVerifier,
};
pub use chapchoo_domain::{
    DisplaySettings, EmailAddress, MembershipRole, Password, TeamKind, TeamRole,
};
pub use chapchoo_persistence::{
    AgentRemoval, AgentSummary, ComponentKeyPolicy, ComponentKeyResult, ComputerRequest,
    ComputerSummary, ConnectorIdentity, ConnectorSessionResult, CreationOutcome,
    ExecutionReservation, GrantKind, GrantSummary, InvitationStatus, InvitationSummary,
    LifecycleRequest, ManagedComputer, ManagedResource, ManagedResourceKind, MemberSummary,
    ProofPreview, ProviderCapabilities, ProviderChoice, ProviderSource, ResourceEvidence,
    ResourceReport, SessionPolicy, SessionResult, TeamPatch, TeamSummary, UserSummary,
};
pub use computers::ComputersService;
pub use error::{DomainReason, InfrastructureReport, ServiceError};
pub use mail::{MailFailure, MailFuture, MailMessage, MailPurpose, Mailer};
pub use password::PasswordWork;
pub use service::{
    AccountsConfig, AccountsService, CodeChallenge, GrantsPage, IssuedGrant, LoginMethods,
};
pub use time::AcceptanceTime;
