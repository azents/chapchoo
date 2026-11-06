//! Current account authority over backend authentication records and retained results.

use chapchoo_crypto::{
    ExternalSecretContext, ExternalSecretKind, ExternalSecretOwner, SecretDigest,
};
use diesel::{
    OptionalExtension,
    sql_types::{BigInt, Binary, Nullable, Text},
};
use diesel_async::RunQueryDsl;
use std::fmt;

use crate::{
    StoreError,
    entity_id::{EntityId, EntityKind, IdentitySql},
    store::Dialect,
};

/// Latest supported credential expiry, including millisecond precision at the
/// end of year 9999. Accepted metadata must remain representable over RPC.
pub const MAX_AUTHENTICATION_EXPIRY_MILLIS: i64 = 253_402_300_799_999;

/// Personal settings are always bound to the authenticated User, never a supplied User ID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthenticationScope {
    Personal,
    Team { team_id: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthenticationKind {
    ChatGpt,
    OpenAiApiKey,
}
impl AuthenticationKind {
    pub(crate) fn parse(value: &str) -> Result<Self, StoreError> {
        match value {
            "CHATGPT" => Ok(Self::ChatGpt),
            "OPENAI_API_KEY" => Ok(Self::OpenAiApiKey),
            _ => Err(StoreError::Integrity),
        }
    }
}

/// Metadata is secret-free; registration is not proof of runtime applicability.
#[derive(Clone)]
pub struct AuthenticationSummary {
    pub id: String,
    pub scope: AuthenticationScope,
    pub kind: AuthenticationKind,
    pub name: String,
    pub account_id: Option<String>,
    pub email: Option<String>,
    pub plan_type: Option<String>,
    pub expires_at: Option<i64>,
    pub current_request_id: String,
    pub created_at: i64,
    pub updated_at: i64,
}
impl fmt::Debug for AuthenticationSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("AuthenticationSummary([REDACTED])")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthenticationOperation {
    RegisterKey,
    ReplaceKey,
    Rename,
    Delete,
    Select,
    StartDevice,
    CancelDevice,
}
impl AuthenticationOperation {
    fn stored(self) -> &'static str {
        match self {
            Self::RegisterKey => "REGISTER_KEY",
            Self::ReplaceKey => "REPLACE_KEY",
            Self::Rename => "RENAME",
            Self::Delete => "DELETE",
            Self::Select => "SELECT",
            Self::StartDevice => "START_DEVICE",
            Self::CancelDevice => "CANCEL_DEVICE",
        }
    }
    fn parse(value: &str) -> Result<Self, StoreError> {
        match value {
            "REGISTER_KEY" => Ok(Self::RegisterKey),
            "REPLACE_KEY" => Ok(Self::ReplaceKey),
            "RENAME" => Ok(Self::Rename),
            "DELETE" => Ok(Self::Delete),
            "SELECT" => Ok(Self::Select),
            "START_DEVICE" => Ok(Self::StartDevice),
            "CANCEL_DEVICE" => Ok(Self::CancelDevice),
            _ => Err(StoreError::Integrity),
        }
    }
    fn needs_management(self) -> bool {
        self != Self::Select
    }
}

/// Historical accepted identities, independent of whether the catalogue/Agent survives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthenticationAcceptedResult {
    Record {
        authentication_id: String,
    },
    Deleted {
        authentication_id: String,
    },
    Selection {
        agent_id: String,
        authentication_id: String,
    },
    Device {
        attempt_id: String,
    },
}

#[derive(Debug, Clone)]
pub struct AuthenticationRequestResult {
    pub operation: AuthenticationOperation,
    pub accepted: AuthenticationAcceptedResult,
    pub accepted_at: i64,
}

#[derive(Debug)]
pub enum DeleteAuthenticationOutcome {
    Accepted(AuthenticationAcceptedResult),
    InUse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Owner {
    user: Option<EntityId>,
    team: Option<EntityId>,
}
impl Owner {
    fn personal(user: EntityId) -> Self {
        Self {
            user: Some(user),
            team: None,
        }
    }
    fn team(team: EntityId) -> Self {
        Self {
            user: None,
            team: Some(team),
        }
    }
    fn scope(self) -> Result<AuthenticationScope, StoreError> {
        match (self.user, self.team) {
            (Some(_), None) => Ok(AuthenticationScope::Personal),
            (None, Some(id)) => Ok(AuthenticationScope::Team {
                team_id: id.to_dto(EntityKind::Team),
            }),
            _ => Err(StoreError::Integrity),
        }
    }
    fn key(self) -> Result<OwnerContext, StoreError> {
        match (self.user, self.team) {
            (Some(id), None) => Ok(OwnerContext::User(id.to_dto(EntityKind::User))),
            (None, Some(id)) => Ok(OwnerContext::Team(id.to_dto(EntityKind::Team))),
            _ => Err(StoreError::Integrity),
        }
    }
}

enum OwnerContext {
    User(String),
    Team(String),
}

fn context<'a>(
    owner: &'a OwnerContext,
    object: &'a str,
    kind: ExternalSecretKind,
) -> Result<ExternalSecretContext<'a>, StoreError> {
    Ok(ExternalSecretContext::new(
        match owner {
            OwnerContext::User(id) => ExternalSecretOwner::User(id),
            OwnerContext::Team(id) => ExternalSecretOwner::Team(id),
        },
        object,
        kind,
    )?)
}

fn request_id(value: &str) -> Result<(), StoreError> {
    if !crate::computers::valid_id(value) {
        return Err(StoreError::InvalidInput);
    }
    Ok(())
}
fn name(value: &str) -> Result<(), StoreError> {
    if value.trim().is_empty()
        || value.chars().count() > 256
        || value.len() > 1024
        || value.chars().any(char::is_control)
    {
        return Err(StoreError::InvalidInput);
    }
    Ok(())
}

#[derive(diesel::QueryableByName)]
struct ScopeTeamRow {
    #[diesel(sql_type=IdentitySql)]
    owner_user_id: EntityId,
}
#[derive(diesel::QueryableByName)]
struct ScopeRoleRow {
    #[diesel(sql_type=Text)]
    role: String,
}

// User lock is already held by authenticate!. Team mutations share this Team lock.
macro_rules! admit_scope {
    ($connection:expr,$dialect:expr,$scope:expr,$user:expr,$manage:expr) => {{
        match $scope {
            AuthenticationScope::Personal => Owner::personal(EntityId::from_dto(EntityKind::User, $user)?),
            AuthenticationScope::Team { team_id } => {
                let team_id = EntityId::from_dto(EntityKind::Team, team_id)?;
                let sql = match $dialect {
                    Dialect::Postgres => "SELECT owner_user_id FROM teams WHERE id=? AND deletion_requested_at IS NULL FOR UPDATE",
                    Dialect::Sqlite => "SELECT owner_user_id FROM teams WHERE id=? AND deletion_requested_at IS NULL",
                };
                let team = diesel::sql_query($dialect.query(sql)).bind::<IdentitySql,_>(team_id)
                    .get_result::<ScopeTeamRow>($connection).await.optional()?.ok_or(StoreError::NotFound)?;
                let caller = EntityId::from_dto(EntityKind::User, $user)?;
                let membership = diesel::sql_query($dialect.query("SELECT CAST(role AS TEXT) AS role FROM memberships WHERE team_id=? AND user_id=?"))
                    .bind::<IdentitySql,_>(team_id).bind::<IdentitySql,_>(caller)
                    .get_result::<ScopeRoleRow>($connection).await.optional()?.ok_or(StoreError::NotFound)?;
                let manager = match membership.role.as_str() {
                    "MANAGER" => true,
                    "MEMBER" => team.owner_user_id == caller,
                    _ => return Err(StoreError::Integrity),
                };
                if $manage && !manager { return Err(StoreError::Forbidden); }
                Owner::team(team_id)
            }
        }
    }};
}

#[derive(diesel::QueryableByName)]
struct AuthenticationRow {
    #[diesel(sql_type=IdentitySql)]
    id: EntityId,
    #[diesel(sql_type=Nullable<IdentitySql>)]
    owner_user_id: Option<EntityId>,
    #[diesel(sql_type=Nullable<IdentitySql>)]
    owner_team_id: Option<EntityId>,
    #[diesel(sql_type=Text)]
    kind: String,
    #[diesel(sql_type=Text)]
    name: String,
    #[diesel(sql_type=Nullable<Text>)]
    account_id: Option<String>,
    #[diesel(sql_type=Nullable<Text>)]
    email: Option<String>,
    #[diesel(sql_type=Nullable<Text>)]
    plan_type: Option<String>,
    #[diesel(sql_type=Nullable<BigInt>)]
    expires_at: Option<i64>,
    #[diesel(sql_type=Text)]
    current_request_id: String,
    #[diesel(sql_type=BigInt)]
    created_at: i64,
    #[diesel(sql_type=BigInt)]
    updated_at: i64,
}
const AUTH_COLUMNS: &str = "SELECT id,owner_user_id,owner_team_id,kind,name,account_id,email,plan_type,expires_at,current_request_id,created_at,updated_at FROM external_authentications";
impl AuthenticationRow {
    fn owner(&self) -> Owner {
        Owner {
            user: self.owner_user_id,
            team: self.owner_team_id,
        }
    }
    fn summary(self) -> Result<AuthenticationSummary, StoreError> {
        Ok(AuthenticationSummary {
            scope: self.owner().scope()?,
            kind: AuthenticationKind::parse(&self.kind)?,
            id: self.id.to_dto(EntityKind::Authentication),
            name: self.name,
            account_id: self.account_id,
            email: self.email,
            plan_type: self.plan_type,
            expires_at: self.expires_at,
            current_request_id: self.current_request_id,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}
macro_rules! lock_record {
    ($connection:expr,$dialect:expr,$id:expr,$owner:expr) => {{
        let sql = match $dialect {
            Dialect::Postgres => format!("{AUTH_COLUMNS} WHERE id=? FOR UPDATE"),
            Dialect::Sqlite => format!("{AUTH_COLUMNS} WHERE id=?"),
        };
        let row = diesel::sql_query($dialect.query(&sql))
            .bind::<IdentitySql, _>($id)
            .get_result::<AuthenticationRow>($connection)
            .await
            .optional()?
            .ok_or(StoreError::NotFound)?;
        if row.owner() != $owner {
            return Err(StoreError::NotFound);
        }
        row
    }};
}

#[derive(diesel::QueryableByName)]
struct ReceiptRow {
    #[diesel(sql_type=Text)]
    operation: String,
    #[diesel(sql_type=Binary)]
    fingerprint: Vec<u8>,
    #[diesel(sql_type=Nullable<IdentitySql>)]
    owner_user_id: Option<EntityId>,
    #[diesel(sql_type=Nullable<IdentitySql>)]
    owner_team_id: Option<EntityId>,
    #[diesel(sql_type=Nullable<IdentitySql>)]
    assigned_authentication_id: Option<EntityId>,
    #[diesel(sql_type=Nullable<IdentitySql>)]
    agent_id: Option<EntityId>,
    #[diesel(sql_type=Nullable<IdentitySql>)]
    attempt_id: Option<EntityId>,
    #[diesel(sql_type=BigInt)]
    accepted_at: i64,
}
impl ReceiptRow {
    fn owner(&self) -> Owner {
        Owner {
            user: self.owner_user_id,
            team: self.owner_team_id,
        }
    }
    fn result(&self) -> Result<AuthenticationRequestResult, StoreError> {
        let operation = AuthenticationOperation::parse(&self.operation)?;
        let accepted = match (
            operation,
            self.assigned_authentication_id,
            self.agent_id,
            self.attempt_id,
        ) {
            (
                AuthenticationOperation::RegisterKey
                | AuthenticationOperation::ReplaceKey
                | AuthenticationOperation::Rename,
                Some(id),
                None,
                None,
            ) => AuthenticationAcceptedResult::Record {
                authentication_id: id.to_dto(EntityKind::Authentication),
            },
            (AuthenticationOperation::Delete, Some(id), None, None) => {
                AuthenticationAcceptedResult::Deleted {
                    authentication_id: id.to_dto(EntityKind::Authentication),
                }
            }
            (AuthenticationOperation::Select, Some(id), Some(agent), None) => {
                AuthenticationAcceptedResult::Selection {
                    agent_id: agent.to_dto(EntityKind::Agent),
                    authentication_id: id.to_dto(EntityKind::Authentication),
                }
            }
            (
                AuthenticationOperation::StartDevice | AuthenticationOperation::CancelDevice,
                _,
                None,
                Some(attempt),
            ) => AuthenticationAcceptedResult::Device {
                attempt_id: attempt.to_dto(EntityKind::AuthenticationAttempt),
            },
            _ => return Err(StoreError::Integrity),
        };
        Ok(AuthenticationRequestResult {
            operation,
            accepted,
            accepted_at: self.accepted_at,
        })
    }
    fn replay(
        &self,
        owner: Owner,
        operation: AuthenticationOperation,
        fingerprint: &SecretDigest,
    ) -> Result<AuthenticationAcceptedResult, StoreError> {
        let stored = SecretDigest::from_bytes(
            self.fingerprint
                .as_slice()
                .try_into()
                .map_err(|_| StoreError::Integrity)?,
        );
        if self.owner() != owner
            || self.operation != operation.stored()
            || !stored.matches_digest(fingerprint)
        {
            return Err(StoreError::Conflict);
        }
        Ok(self.result()?.accepted)
    }
}
macro_rules! receipt {
    ($connection:expr,$dialect:expr,$user:expr,$request:expr) => {{
        diesel::sql_query($dialect.query("SELECT operation,fingerprint,owner_user_id,owner_team_id,assigned_authentication_id,agent_id,attempt_id,accepted_at FROM external_auth_requests WHERE actor_user_id=? AND request_id=?"))
            .bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::User,$user)?)
            .bind::<Text,_>($request).get_result::<ReceiptRow>($connection).await.optional()?
    }};
}
macro_rules! save_receipt {
    ($connection:expr,$dialect:expr,$user:expr,$owner:expr,$request:expr,$operation:expr,$fingerprint:expr,$record:expr,$agent:expr,$attempt:expr,$now:expr) => {{
        diesel::sql_query($dialect.query("INSERT INTO external_auth_requests(actor_user_id,request_id,operation,fingerprint,assigned_authentication_id,agent_id,attempt_id,accepted_at,owner_user_id,owner_team_id) VALUES(?,?,?,?,?,?,?,?,?,?)"))
            .bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::User,$user)?)
            .bind::<Text,_>($request).bind::<Text,_>($operation.stored())
            .bind::<Binary,_>($fingerprint.as_bytes().as_slice())
            .bind::<Nullable<IdentitySql>,_>($record).bind::<Nullable<IdentitySql>,_>($agent)
            .bind::<Nullable<IdentitySql>,_>($attempt).bind::<BigInt,_>($now)
            .bind::<Nullable<IdentitySql>,_>($owner.user).bind::<Nullable<IdentitySql>,_>($owner.team)
            .execute($connection).await?;
    }};
}

mod catalogue;
mod cleanup;
mod device;
mod selection;
pub use catalogue::{DeleteAuthentication, RegisterApiKey, RenameAuthentication, ReplaceApiKey};
pub(crate) use cleanup::{AuthenticationLock, detach_team_authentications};
pub use device::{
    CheckDeviceAdmission, DeviceAuthenticationState, DeviceAuthenticationStatus,
    DeviceAuthenticationStep, DeviceAuthenticationTokens, DeviceAuthorizationDraft, DeviceCheck,
    DeviceStepFailure, DeviceStepSettlement, ExchangeDeviceAdmission, ReauthenticateTarget,
    StartDeviceAdmission, StartDeviceAuthentication,
};
pub use selection::{
    AgentAuthenticationSelection, ExpectedAgentAuthentication, SelectAgentAuthentication,
};
