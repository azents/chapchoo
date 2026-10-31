//! Explicit application orchestration without persistence handles or transport types.
mod error;
mod mail;
mod password;
mod service;
mod time;

pub use chapchoo_crypto::{BearerSecret, EmailCode, PasswordCost, SecretDigest};
pub use chapchoo_domain::{
    DisplaySettings, EmailAddress, MembershipRole, Password, TeamKind, TeamRole,
};
pub use chapchoo_persistence::{
    GrantKind, GrantSummary, InvitationStatus, InvitationSummary, MemberSummary, ProofPreview,
    SessionPolicy, SessionResult, TeamPatch, TeamSummary, UserSummary,
};
pub use error::{DomainReason, InfrastructureReport, ServiceError};
pub use mail::{MailFailure, MailFuture, MailMessage, MailPurpose, Mailer};
pub use password::PasswordWork;
pub use service::{
    AccountsConfig, AccountsService, CodeChallenge, GrantsPage, IssuedGrant, LoginMethods,
};
pub use time::AcceptanceTime;
