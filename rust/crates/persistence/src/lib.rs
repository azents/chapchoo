//! Engine-specific completed storage operations for one installation.

mod accounts;
mod challenges;
mod identity;
mod models;
mod preparation;
mod sessions;
mod store;
mod teams;

pub use accounts::{GrantDraft, GrantKind, GrantSummary, NewAccount, ProofPreview};
pub use challenges::{EmailLoginOutcome, NewChallenge};
pub use models::{CredentialSnapshot, UserSummary, VerifiedCredential};
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
