//! Single-step database admission around externally owned device authorization.

use super::*;
use crate::{Store, models::authenticate, store::operation};
use chapchoo_crypto::{ExternalSecret, ExternalSecretEnvelope};
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, Zeroizing};

const ATTEMPT_LIFETIME_MS: i64 = 900_000;

#[derive(Debug)]
pub struct ReauthenticateTarget {
    pub authentication_id: String,
    pub expected_request_id: String,
}
#[derive(Debug)]
pub struct StartDeviceAuthentication {
    pub scope: AuthenticationScope,
    pub request_id: String,
    pub name: String,
    pub target: Option<ReauthenticateTarget>,
}

pub struct DeviceAuthorizationDraft {
    pub device_auth_id: ExternalSecret,
    pub user_code: String,
    pub interval_ms: i64,
    pub expires_at: i64,
}
impl fmt::Debug for DeviceAuthorizationDraft {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DeviceAuthorizationDraft([REDACTED])")
    }
}
impl Drop for DeviceAuthorizationDraft {
    fn drop(&mut self) {
        self.user_code.zeroize();
    }
}

pub struct DeviceAuthenticationTokens {
    pub secret: ExternalSecret,
    pub account_id: Option<String>,
    pub email: Option<String>,
    pub plan_type: Option<String>,
    pub expires_at: Option<i64>,
}
impl fmt::Debug for DeviceAuthenticationTokens {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DeviceAuthenticationTokens([REDACTED])")
    }
}

/// The code is disclosed only to the initiating, currently authorized interaction.
pub enum DeviceAuthenticationState {
    Starting,
    Pending {
        user_code: String,
        next_check_at: i64,
    },
    Checking,
    Exchanging,
    Registered {
        authentication_id: String,
    },
    Cancelled,
    Expired,
    Failed,
    Superseded,
    Uncertain,
}
pub struct DeviceAuthenticationStatus {
    pub attempt_id: String,
    pub expires_at: i64,
    pub state: DeviceAuthenticationState,
}
impl fmt::Debug for DeviceAuthenticationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DeviceAuthenticationStatus([REDACTED])")
    }
}
impl fmt::Debug for DeviceAuthenticationState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DeviceAuthenticationState([REDACTED])")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StepPhase {
    Starting,
    Checking,
    Exchanging,
}
impl StepPhase {
    fn stored(self) -> &'static str {
        match self {
            Self::Starting => "STARTING",
            Self::Checking => "CHECKING",
            Self::Exchanging => "EXCHANGING",
        }
    }
}

/// Sealed admission evidence, not a cloneable permission to repeat provider I/O.
pub struct DeviceAuthenticationStep {
    attempt: EntityId,
    initiated_by: EntityId,
    owner: Owner,
    step_id: String,
    phase: StepPhase,
    name: String,
}
impl fmt::Debug for DeviceAuthenticationStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DeviceAuthenticationStep([REDACTED])")
    }
}
impl DeviceAuthenticationStep {
    pub fn attempt_id(&self) -> String {
        self.attempt.to_dto(EntityKind::AuthenticationAttempt)
    }
}

#[derive(Debug)]
pub enum StartDeviceAdmission {
    Issue(DeviceAuthenticationStep),
    Current(DeviceAuthenticationStatus),
}
pub struct DeviceCheck {
    pub step: DeviceAuthenticationStep,
    pub device_auth_id: ExternalSecret,
    pub user_code: Zeroizing<String>,
    pub interval_ms: i64,
}
impl fmt::Debug for DeviceCheck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DeviceCheck([REDACTED])")
    }
}
#[derive(Debug)]
pub enum CheckDeviceAdmission {
    Check(DeviceCheck),
    Current(DeviceAuthenticationStatus),
}
#[derive(Debug)]
pub enum ExchangeDeviceAdmission {
    Exchange(DeviceAuthenticationStep),
    Current(DeviceAuthenticationStatus),
}
/// Neither outcome grants another provider dispatch.
#[derive(Debug, Clone, Copy)]
pub enum DeviceStepFailure {
    Failed,
    Uncertain,
}
/// Settlement cannot disclose a code or acquire current catalogue authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceStepSettlement {
    Recorded,
    AlreadySettled,
    Missing,
}
impl DeviceStepFailure {
    fn stored(self) -> &'static str {
        match self {
            Self::Failed => "FAILED",
            Self::Uncertain => "UNCERTAIN",
        }
    }
}

#[derive(Serialize, Deserialize)]
struct DeviceMaterial {
    name: String,
    device_auth_id: String,
    user_code: String,
}
impl Drop for DeviceMaterial {
    fn drop(&mut self) {
        self.device_auth_id.zeroize();
        self.user_code.zeroize();
    }
}

#[derive(diesel::QueryableByName)]
struct AttemptRow {
    #[diesel(sql_type=IdentitySql)]
    id: EntityId,
    #[diesel(sql_type=IdentitySql)]
    initiated_by: EntityId,
    #[diesel(sql_type=Nullable<IdentitySql>)]
    owner_user_id: Option<EntityId>,
    #[diesel(sql_type=Nullable<IdentitySql>)]
    owner_team_id: Option<EntityId>,
    #[diesel(sql_type=Text)]
    request_id: String,
    #[diesel(sql_type=Nullable<IdentitySql>)]
    target_id: Option<EntityId>,
    #[diesel(sql_type=Nullable<Text>)]
    expected_target_request_id: Option<String>,
    #[diesel(sql_type=Text)]
    phase: String,
    #[diesel(sql_type=Nullable<Text>)]
    step_id: Option<String>,
    #[diesel(sql_type=Nullable<Binary>)]
    pending_nonce: Option<Vec<u8>>,
    #[diesel(sql_type=Nullable<Binary>)]
    pending_ciphertext: Option<Vec<u8>>,
    #[diesel(sql_type=Nullable<BigInt>)]
    interval_ms: Option<i64>,
    #[diesel(sql_type=Nullable<BigInt>)]
    next_check_at: Option<i64>,
    #[diesel(sql_type=BigInt)]
    expires_at: i64,
    #[diesel(sql_type=Nullable<IdentitySql>)]
    result_authentication_id: Option<EntityId>,
}
impl AttemptRow {
    fn owner(&self) -> Owner {
        Owner {
            user: self.owner_user_id,
            team: self.owner_team_id,
        }
    }
    fn matches(&self, step: &DeviceAuthenticationStep) -> bool {
        self.id == step.attempt
            && self.initiated_by == step.initiated_by
            && self.owner() == step.owner
            && self.phase == step.phase.stored()
            && self.step_id.as_deref() == Some(step.step_id.as_str())
    }
    fn material(&self, store: &Store) -> Result<DeviceMaterial, StoreError> {
        let owner = self.owner().key()?;
        let dto = self.id.to_dto(EntityKind::AuthenticationAttempt);
        let nonce = self
            .pending_nonce
            .as_ref()
            .ok_or(StoreError::Integrity)?
            .as_slice()
            .try_into()
            .map_err(|_| StoreError::Integrity)?;
        let envelope = ExternalSecretEnvelope::from_parts(
            nonce,
            self.pending_ciphertext
                .clone()
                .ok_or(StoreError::Integrity)?,
        )?;
        let secret = store.cipher.recover_external(
            &context(&owner, &dto, ExternalSecretKind::PendingDevice)?,
            &envelope,
        )?;
        serde_json::from_slice(secret.expose()).map_err(|_| StoreError::Integrity)
    }
    fn status(&self, store: &Store, now: i64) -> Result<DeviceAuthenticationStatus, StoreError> {
        let state = match self.phase.as_str() {
            // A deadline fences all acceptance even before lazy terminal collection.
            "STARTING" | "PENDING" | "CHECKING" | "EXCHANGING" if self.expires_at <= now => {
                DeviceAuthenticationState::Expired
            }
            "STARTING" => DeviceAuthenticationState::Starting,
            "PENDING" => DeviceAuthenticationState::Pending {
                user_code: self.material(store)?.user_code.clone(),
                next_check_at: self.next_check_at.ok_or(StoreError::Integrity)?,
            },
            "CHECKING" => DeviceAuthenticationState::Checking,
            "EXCHANGING" => DeviceAuthenticationState::Exchanging,
            "REGISTERED" => DeviceAuthenticationState::Registered {
                authentication_id: self
                    .result_authentication_id
                    .ok_or(StoreError::Integrity)?
                    .to_dto(EntityKind::Authentication),
            },
            "CANCELLED" => DeviceAuthenticationState::Cancelled,
            "EXPIRED" => DeviceAuthenticationState::Expired,
            "FAILED" => DeviceAuthenticationState::Failed,
            "SUPERSEDED" => DeviceAuthenticationState::Superseded,
            "UNCERTAIN" => DeviceAuthenticationState::Uncertain,
            _ => return Err(StoreError::Integrity),
        };
        Ok(DeviceAuthenticationStatus {
            attempt_id: self.id.to_dto(EntityKind::AuthenticationAttempt),
            expires_at: self.expires_at,
            state,
        })
    }
}

macro_rules! attempt_row {
    ($connection:expr,$dialect:expr,$id:expr,$lock:expr) => {{
        let sql = "SELECT * FROM external_auth_attempts WHERE id=?";
        let sql = if $lock && matches!($dialect, Dialect::Postgres) {
            format!("{sql} FOR UPDATE")
        } else {
            sql.to_owned()
        };
        diesel::sql_query($dialect.query(&sql))
            .bind::<IdentitySql, _>($id)
            .get_result::<AttemptRow>($connection)
            .await
            .optional()?
            .ok_or(StoreError::NotFound)?
    }};
}
// Discover scope, then lock Team/authentication/attempt in the shared order.
macro_rules! admitted_attempt {
    ($connection:expr,$dialect:expr,$id:expr,$user:expr) => {{
        let discovered = attempt_row!($connection, $dialect, $id, false);
        if discovered.initiated_by != EntityId::from_dto(EntityKind::User, $user)? {
            return Err(StoreError::NotFound);
        }
        let owner = admit_scope!(
            $connection,
            $dialect,
            &discovered.owner().scope()?,
            $user,
            true
        );
        if owner != discovered.owner() {
            return Err(StoreError::NotFound);
        }
        let current = attempt_row!($connection, $dialect, $id, false);
        let target = if let Some(target) = current.target_id {
            Some(lock_record!($connection, $dialect, target, owner))
        } else {
            None
        };
        (attempt_row!($connection, $dialect, $id, true), target)
    }};
}
macro_rules! terminal {
    ($connection:expr,$dialect:expr,$row:expr,$phase:expr,$now:expr) => {{
        diesel::sql_query($dialect.query("UPDATE external_auth_attempts SET phase=?,step_id=NULL,pending_nonce=NULL,pending_ciphertext=NULL,expires_at=?,updated_at=? WHERE id=?"))
            .bind::<Text,_>($phase).bind::<BigInt,_>($row.expires_at).bind::<BigInt,_>($now).bind::<IdentitySql,_>($row.id).execute($connection).await?;
        diesel::sql_query($dialect.query("UPDATE external_authentications SET active_attempt_id=NULL WHERE active_attempt_id=?"))
            .bind::<IdentitySql,_>($row.id).execute($connection).await?;
        $row.phase=$phase.to_owned(); $row.step_id=None; $row.pending_nonce=None; $row.pending_ciphertext=None;
    }};
}
macro_rules! acceptance_fence {
    ($connection:expr,$dialect:expr,$row:expr,$target:expr,$now:expr) => {{
        if matches!(
            $row.phase.as_str(),
            "STARTING" | "PENDING" | "CHECKING" | "EXCHANGING"
        ) {
            if $row.expires_at <= $now {
                terminal!($connection, $dialect, $row, "EXPIRED", $now);
            } else if let Some(target) = &$target {
                #[derive(diesel::QueryableByName)]
                struct Active {
                    #[diesel(sql_type=Nullable<IdentitySql>)]
                    active_attempt_id: Option<EntityId>,
                }
                let active = diesel::sql_query(
                    $dialect
                        .query("SELECT active_attempt_id FROM external_authentications WHERE id=?"),
                )
                .bind::<IdentitySql, _>(target.id)
                .get_result::<Active>($connection)
                .await?;
                if target.current_request_id.as_str()
                    != $row
                        .expected_target_request_id
                        .as_deref()
                        .ok_or(StoreError::Integrity)?
                    || active.active_attempt_id != Some($row.id)
                {
                    terminal!($connection, $dialect, $row, "SUPERSEDED", $now);
                }
            }
        }
    }};
}

impl Store {
    pub async fn start_external_device_authentication(
        &self,
        actor: &SecretDigest,
        draft: &StartDeviceAuthentication,
        now: i64,
    ) -> Result<StartDeviceAdmission, StoreError> {
        request_id(&draft.request_id)?;
        name(&draft.name)?;
        if let Some(target) = &draft.target {
            EntityId::from_dto(EntityKind::Authentication, &target.authentication_id)?;
            request_id(&target.expected_request_id)?;
        }
        let (user, owner) = self.external_scope(actor, &draft.scope, true, now).await?;
        let owner_key = owner.key()?;
        let target_id = draft
            .target
            .as_ref()
            .map(|target| EntityId::from_dto(EntityKind::Authentication, &target.authentication_id))
            .transpose()?;
        let expected = draft
            .target
            .as_ref()
            .map(|target| target.expected_request_id.as_str());
        let operation = AuthenticationOperation::StartDevice;
        let fingerprint = self.cipher.external_request_fingerprint(
            &context(&owner_key, &user, ExternalSecretKind::PendingDevice)?,
            operation.stored(),
            &[
                draft.request_id.as_bytes(),
                draft.name.as_bytes(),
                draft
                    .target
                    .as_ref()
                    .map_or("", |t| t.authentication_id.as_str())
                    .as_bytes(),
                expected.unwrap_or("").as_bytes(),
            ],
        )?;
        let id = EntityId::generate(now)?;
        let step_id = EntityId::generate(now)?.to_dto(EntityKind::AuthenticationAttempt);
        let expires = now
            .checked_add(ATTEMPT_LIFETIME_MS)
            .ok_or(StoreError::InvalidInput)?;
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            let owner = admit_scope!(connection, dialect, &draft.scope, &user.id, true);
            if let Some(receipt) = receipt!(connection, dialect, &user.id, &draft.request_id) {
                let AuthenticationAcceptedResult::Device { attempt_id } =
                    receipt.replay(owner, operation, &fingerprint)?
                else {
                    return Err(StoreError::Integrity);
                };
                let row = attempt_row!(
                    connection,
                    dialect,
                    EntityId::from_dto(EntityKind::AuthenticationAttempt, &attempt_id)?,
                    false
                );
                if row.owner() != owner
                    || row.initiated_by != EntityId::from_dto(EntityKind::User, &user.id)?
                {
                    return Err(StoreError::Integrity);
                }
                return Ok(StartDeviceAdmission::Current(row.status(self, now)?));
            }
            if let Some(target_id) = target_id {
                let target = lock_record!(connection, dialect, target_id, owner);
                if target.kind != "CHATGPT" || Some(target.current_request_id.as_str()) != expected
                {
                    return Err(StoreError::InvalidTransition);
                }
                // Exact target identity explicitly supersedes its former active attempt.
                diesel::sql_query(dialect.query("UPDATE external_auth_attempts SET phase='SUPERSEDED',step_id=NULL,pending_nonce=NULL,pending_ciphertext=NULL,updated_at=? WHERE id IN (SELECT active_attempt_id FROM external_authentications WHERE id=?) AND phase IN ('STARTING','PENDING','CHECKING','EXCHANGING','UNCERTAIN')"))
                    .bind::<BigInt,_>(now).bind::<IdentitySql,_>(target_id).execute(connection).await?;
            }
            diesel::sql_query(dialect.query("INSERT INTO external_auth_attempts(id,initiated_by,owner_user_id,owner_team_id,request_id,target_id,expected_target_request_id,phase,step_id,expires_at,created_at,updated_at) VALUES(?,?,?,?,?,?,?,'STARTING',?,?,?,?)"))
                .bind::<IdentitySql,_>(id).bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::User,&user.id)?)
                .bind::<Nullable<IdentitySql>,_>(owner.user).bind::<Nullable<IdentitySql>,_>(owner.team)
                .bind::<Text,_>(&draft.request_id).bind::<Nullable<IdentitySql>,_>(target_id).bind::<Nullable<Text>,_>(expected)
                .bind::<Text,_>(&step_id).bind::<BigInt,_>(expires).bind::<BigInt,_>(now).bind::<BigInt,_>(now).execute(connection).await?;
            if let Some(target_id) = target_id {
                diesel::sql_query(
                    dialect.query(
                        "UPDATE external_authentications SET active_attempt_id=? WHERE id=?",
                    ),
                )
                .bind::<IdentitySql, _>(id)
                .bind::<IdentitySql, _>(target_id)
                .execute(connection)
                .await?;
            }
            save_receipt!(
                connection,
                dialect,
                &user.id,
                owner,
                &draft.request_id,
                operation,
                &fingerprint,
                None::<EntityId>,
                None::<EntityId>,
                Some(id),
                now
            );
            Ok(StartDeviceAdmission::Issue(DeviceAuthenticationStep {
                attempt: id,
                initiated_by: EntityId::from_dto(EntityKind::User, &user.id)?,
                owner,
                step_id: step_id.clone(),
                phase: StepPhase::Starting,
                name: draft.name.clone(),
            }))
        })
    }

    /// Read-only: observing an in-flight step never grants another dispatch.
    pub async fn external_device_authentication_status(
        &self,
        actor: &SecretDigest,
        attempt: &str,
        now: i64,
    ) -> Result<DeviceAuthenticationStatus, StoreError> {
        let id = EntityId::from_dto(EntityKind::AuthenticationAttempt, attempt)?;
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            let row = attempt_row!(connection, dialect, id, false);
            if row.initiated_by != EntityId::from_dto(EntityKind::User, &user.id)? {
                return Err(StoreError::NotFound);
            }
            let owner = admit_scope!(connection, dialect, &row.owner().scope()?, &user.id, true);
            if owner != row.owner() {
                return Err(StoreError::NotFound);
            }
            // Re-read after the Team lock, which may have waited for cancellation.
            attempt_row!(connection, dialect, id, false).status(self, now)
        })
    }

    pub async fn accept_external_device_authorization(
        &self,
        actor: &SecretDigest,
        step: &DeviceAuthenticationStep,
        issued: &DeviceAuthorizationDraft,
        now: i64,
    ) -> Result<DeviceAuthenticationStatus, StoreError> {
        if step.phase != StepPhase::Starting
            || !(1000..=900000).contains(&issued.interval_ms)
            || issued.expires_at < 0
            || issued.expires_at
                > now
                    .checked_add(ATTEMPT_LIFETIME_MS)
                    .ok_or(StoreError::InvalidInput)?
        {
            return Err(StoreError::InvalidInput);
        }
        let device_id = std::str::from_utf8(issued.device_auth_id.expose())
            .map_err(|_| StoreError::InvalidInput)?;
        if device_id.is_empty()
            || device_id.len() > 4096
            || device_id.chars().any(char::is_control)
            || issued.user_code.is_empty()
            || issued.user_code.len() > 128
            || issued.user_code.chars().any(char::is_control)
        {
            return Err(StoreError::InvalidInput);
        }
        let material = DeviceMaterial {
            name: step.name.clone(),
            device_auth_id: device_id.to_owned(),
            user_code: issued.user_code.clone(),
        };
        let secret =
            ExternalSecret::new(serde_json::to_vec(&material).map_err(|_| StoreError::Integrity)?)?;
        let owner = step.owner.key()?;
        let dto = step.attempt_id();
        let protected = self.cipher.protect_external(
            &context(&owner, &dto, ExternalSecretKind::PendingDevice)?,
            &secret,
        )?;
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            let (mut row, target) = admitted_attempt!(connection, dialect, step.attempt, &user.id);
            acceptance_fence!(connection, dialect, row, target, now);
            if !row.matches(step) {
                return row.status(self, now);
            }
            let expires = row.expires_at.min(issued.expires_at);
            row.expires_at = expires;
            if expires <= now {
                terminal!(connection, dialect, row, "EXPIRED", now);
                return row.status(self, now);
            }
            let due = now
                .checked_add(issued.interval_ms)
                .ok_or(StoreError::InvalidInput)?;
            diesel::sql_query(dialect.query("UPDATE external_auth_attempts SET phase='PENDING',step_id=NULL,pending_nonce=?,pending_ciphertext=?,interval_ms=?,next_check_at=?,expires_at=?,updated_at=? WHERE id=?"))
                .bind::<Binary,_>(protected.nonce().as_slice()).bind::<Binary,_>(protected.ciphertext()).bind::<BigInt,_>(issued.interval_ms)
                .bind::<BigInt,_>(due).bind::<BigInt,_>(expires).bind::<BigInt,_>(now).bind::<IdentitySql,_>(row.id).execute(connection).await?;
            row.phase = "PENDING".to_owned();
            row.step_id = None;
            row.pending_nonce = Some(protected.nonce().to_vec());
            row.pending_ciphertext = Some(protected.ciphertext().to_vec());
            row.interval_ms = Some(issued.interval_ms);
            row.next_check_at = Some(due);
            row.expires_at = expires;
            row.status(self, now)
        })
    }

    pub async fn check_external_device_authentication(
        &self,
        actor: &SecretDigest,
        attempt: &str,
        now: i64,
    ) -> Result<CheckDeviceAdmission, StoreError> {
        let id = EntityId::from_dto(EntityKind::AuthenticationAttempt, attempt)?;
        let step_id = EntityId::generate(now)?.to_dto(EntityKind::AuthenticationAttempt);
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            let (mut row, target) = admitted_attempt!(connection, dialect, id, &user.id);
            acceptance_fence!(connection, dialect, row, target, now);
            if row.phase != "PENDING" || row.next_check_at.ok_or(StoreError::Integrity)? > now {
                return Ok(CheckDeviceAdmission::Current(row.status(self, now)?));
            }
            let material = row.material(self)?;
            diesel::sql_query(dialect.query("UPDATE external_auth_attempts SET phase='CHECKING',step_id=?,updated_at=? WHERE id=? AND phase='PENDING'"))
                .bind::<Text,_>(&step_id).bind::<BigInt,_>(now).bind::<IdentitySql,_>(id).execute(connection).await?;
            Ok(CheckDeviceAdmission::Check(DeviceCheck {
                step: DeviceAuthenticationStep {
                    attempt: id,
                    initiated_by: row.initiated_by,
                    owner: row.owner(),
                    step_id: step_id.clone(),
                    phase: StepPhase::Checking,
                    name: material.name.clone(),
                },
                device_auth_id: ExternalSecret::new(material.device_auth_id.as_bytes().to_vec())?,
                user_code: Zeroizing::new(material.user_code.clone()),
                interval_ms: row.interval_ms.ok_or(StoreError::Integrity)?,
            }))
        })
    }

    pub async fn accept_external_device_pending(
        &self,
        actor: &SecretDigest,
        step: &DeviceAuthenticationStep,
        interval_ms: i64,
        now: i64,
    ) -> Result<DeviceAuthenticationStatus, StoreError> {
        if step.phase != StepPhase::Checking || !(1000..=900000).contains(&interval_ms) {
            return Err(StoreError::InvalidInput);
        }
        let due = now
            .checked_add(interval_ms)
            .ok_or(StoreError::InvalidInput)?;
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            let (mut row, target) = admitted_attempt!(connection, dialect, step.attempt, &user.id);
            acceptance_fence!(connection, dialect, row, target, now);
            if !row.matches(step) {
                return row.status(self, now);
            }
            diesel::sql_query(dialect.query("UPDATE external_auth_attempts SET phase='PENDING',step_id=NULL,interval_ms=?,next_check_at=?,updated_at=? WHERE id=?"))
                .bind::<BigInt,_>(interval_ms).bind::<BigInt,_>(due).bind::<BigInt,_>(now).bind::<IdentitySql,_>(row.id).execute(connection).await?;
            row.phase = "PENDING".to_owned();
            row.step_id = None;
            row.interval_ms = Some(interval_ms);
            row.next_check_at = Some(due);
            row.status(self, now)
        })
    }

    pub async fn exchange_external_device_authentication(
        &self,
        actor: &SecretDigest,
        step: &DeviceAuthenticationStep,
        now: i64,
    ) -> Result<ExchangeDeviceAdmission, StoreError> {
        if step.phase != StepPhase::Checking {
            return Err(StoreError::InvalidInput);
        }
        let next = EntityId::generate(now)?.to_dto(EntityKind::AuthenticationAttempt);
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            let (mut row, target) = admitted_attempt!(connection, dialect, step.attempt, &user.id);
            acceptance_fence!(connection, dialect, row, target, now);
            if !row.matches(step) {
                return Ok(ExchangeDeviceAdmission::Current(row.status(self, now)?));
            }
            diesel::sql_query(dialect.query("UPDATE external_auth_attempts SET phase='EXCHANGING',step_id=?,updated_at=? WHERE id=?"))
                .bind::<Text,_>(&next).bind::<BigInt,_>(now).bind::<IdentitySql,_>(row.id).execute(connection).await?;
            Ok(ExchangeDeviceAdmission::Exchange(
                DeviceAuthenticationStep {
                    attempt: row.id,
                    initiated_by: row.initiated_by,
                    owner: row.owner(),
                    step_id: next.clone(),
                    phase: StepPhase::Exchanging,
                    name: step.name.clone(),
                },
            ))
        })
    }

    pub async fn complete_external_device_authentication(
        &self,
        actor: &SecretDigest,
        step: &DeviceAuthenticationStep,
        tokens: &DeviceAuthenticationTokens,
        now: i64,
    ) -> Result<DeviceAuthenticationStatus, StoreError> {
        if step.phase != StepPhase::Exchanging {
            return Err(StoreError::InvalidInput);
        }
        if tokens.expires_at.is_some_and(|expiry| expiry < 0) {
            return Err(StoreError::InvalidInput);
        }
        for value in [&tokens.account_id, &tokens.email, &tokens.plan_type]
            .into_iter()
            .flatten()
        {
            if value.is_empty() || value.len() > 4096 || value.chars().any(char::is_control) {
                return Err(StoreError::InvalidInput);
            }
        }
        // Discover a same-record target before preparing encryption, without
        // holding a transaction during cryptographic preparation.
        let target = operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            let (row, _) = admitted_attempt!(connection, dialect, step.attempt, &user.id);
            Ok(row.target_id)
        })?;
        let result_id = target.map_or_else(|| EntityId::generate(now), Ok)?;
        let dto = result_id.to_dto(EntityKind::Authentication);
        let owner = step.owner.key()?;
        let protected = self.cipher.protect_external(
            &context(&owner, &dto, ExternalSecretKind::ChatGpt)?,
            &tokens.secret,
        )?;
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            let (mut row, target) = admitted_attempt!(connection, dialect, step.attempt, &user.id);
            acceptance_fence!(connection, dialect, row, target, now);
            if !row.matches(step) {
                return row.status(self, now);
            }
            if let Some(target) = &target {
                if target.id != result_id
                    || target.kind != "CHATGPT"
                    || target.account_id.is_none()
                    || tokens.account_id.is_none()
                    || target.account_id != tokens.account_id
                {
                    terminal!(connection, dialect, row, "FAILED", now);
                    return row.status(self, now);
                }
                diesel::sql_query(dialect.query("UPDATE external_authentications SET nonce=?,ciphertext=?,account_id=?,email=?,plan_type=?,expires_at=?,current_request_id=?,active_attempt_id=NULL,updated_at=? WHERE id=?"))
                    .bind::<Binary,_>(protected.nonce().as_slice()).bind::<Binary,_>(protected.ciphertext())
                    .bind::<Nullable<Text>,_>(tokens.account_id.as_deref()).bind::<Nullable<Text>,_>(tokens.email.as_deref()).bind::<Nullable<Text>,_>(tokens.plan_type.as_deref())
                    .bind::<Nullable<BigInt>,_>(tokens.expires_at).bind::<Text,_>(&row.request_id).bind::<BigInt,_>(now).bind::<IdentitySql,_>(result_id).execute(connection).await?;
            } else {
                let owner = row.owner();
                diesel::sql_query(dialect.query("INSERT INTO external_authentications(id,owner_user_id,owner_team_id,kind,name,account_id,email,plan_type,nonce,ciphertext,expires_at,current_request_id,created_at,updated_at) VALUES(?,?,?,'CHATGPT',?,?,?,?,?,?,?,?,?,?)"))
                    .bind::<IdentitySql,_>(result_id).bind::<Nullable<IdentitySql>,_>(owner.user).bind::<Nullable<IdentitySql>,_>(owner.team)
                    .bind::<Text,_>(&step.name).bind::<Nullable<Text>,_>(tokens.account_id.as_deref()).bind::<Nullable<Text>,_>(tokens.email.as_deref()).bind::<Nullable<Text>,_>(tokens.plan_type.as_deref())
                    .bind::<Binary,_>(protected.nonce().as_slice()).bind::<Binary,_>(protected.ciphertext()).bind::<Nullable<BigInt>,_>(tokens.expires_at)
                    .bind::<Text,_>(&row.request_id).bind::<BigInt,_>(now).bind::<BigInt,_>(now).execute(connection).await?;
            }
            diesel::sql_query(dialect.query("UPDATE external_auth_attempts SET phase='REGISTERED',step_id=NULL,pending_nonce=NULL,pending_ciphertext=NULL,result_authentication_id=?,updated_at=? WHERE id=?"))
                .bind::<IdentitySql,_>(result_id).bind::<BigInt,_>(now).bind::<IdentitySql,_>(row.id).execute(connection).await?;
            row.phase = "REGISTERED".to_owned();
            row.result_authentication_id = Some(result_id);
            row.step_id = None;
            row.pending_nonce = None;
            row.pending_ciphertext = None;
            row.status(self, now)
        })
    }

    pub async fn cancel_external_device_authentication(
        &self,
        actor: &SecretDigest,
        attempt: &str,
        request: &str,
        now: i64,
    ) -> Result<DeviceAuthenticationStatus, StoreError> {
        request_id(request)?;
        let id = EntityId::from_dto(EntityKind::AuthenticationAttempt, attempt)?;
        let (user, expected_owner) = operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            let (row, _) = admitted_attempt!(connection, dialect, id, &user.id);
            Ok((user.id, row.owner()))
        })?;
        let owner_key = expected_owner.key()?;
        let operation = AuthenticationOperation::CancelDevice;
        let fingerprint = self.cipher.external_request_fingerprint(
            &context(&owner_key, &user, ExternalSecretKind::PendingDevice)?,
            operation.stored(),
            &[request.as_bytes(), attempt.as_bytes()],
        )?;
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            let (mut row, target) = admitted_attempt!(connection, dialect, id, &user.id);
            let owner = row.owner();
            if owner != expected_owner {
                return Err(StoreError::Integrity);
            }
            if let Some(receipt) = receipt!(connection, dialect, &user.id, request) {
                receipt.replay(owner, operation, &fingerprint)?;
                return row.status(self, now);
            }
            acceptance_fence!(connection, dialect, row, target, now);
            if matches!(
                row.phase.as_str(),
                "STARTING" | "PENDING" | "CHECKING" | "EXCHANGING" | "UNCERTAIN"
            ) {
                terminal!(connection, dialect, row, "CANCELLED", now);
            }
            save_receipt!(
                connection,
                dialect,
                &user.id,
                owner,
                request,
                operation,
                &fingerprint,
                None::<EntityId>,
                None::<EntityId>,
                Some(id),
                now
            );
            row.status(self, now)
        })
    }

    /// Narrow settlement for an already-admitted step. It may only clear secret
    /// material and stop progression; it cannot publish credentials or retry work.
    pub async fn fail_external_device_step(
        &self,
        step: &DeviceAuthenticationStep,
        failure: DeviceStepFailure,
        now: i64,
    ) -> Result<DeviceStepSettlement, StoreError> {
        operation!(self, |connection, dialect| {
            let sql = match dialect {
                Dialect::Postgres => "SELECT id AS user_id FROM users WHERE id=? FOR NO KEY UPDATE",
                Dialect::Sqlite => "SELECT id AS user_id FROM users WHERE id=?",
            };
            let user = diesel::sql_query(dialect.query(sql))
                .bind::<IdentitySql, _>(step.initiated_by)
                .get_result::<crate::models::UserId>(connection)
                .await
                .optional()?;
            if user.is_none() {
                return Ok(DeviceStepSettlement::Missing);
            }
            if let Some(team) = step.owner.team {
                let sql = match dialect {
                    Dialect::Postgres => "SELECT owner_user_id FROM teams WHERE id=? FOR UPDATE",
                    Dialect::Sqlite => "SELECT owner_user_id FROM teams WHERE id=?",
                };
                if diesel::sql_query(dialect.query(sql))
                    .bind::<IdentitySql, _>(team)
                    .get_result::<ScopeTeamRow>(connection)
                    .await
                    .optional()?
                    .is_none()
                {
                    return Ok(DeviceStepSettlement::Missing);
                }
            }
            let Some(initial) =
                diesel::sql_query(dialect.query("SELECT * FROM external_auth_attempts WHERE id=?"))
                    .bind::<IdentitySql, _>(step.attempt)
                    .get_result::<AttemptRow>(connection)
                    .await
                    .optional()?
            else {
                return Ok(DeviceStepSettlement::Missing);
            };
            if initial.owner() != step.owner || initial.initiated_by != step.initiated_by {
                return Err(StoreError::Integrity);
            }
            if let Some(target) = initial.target_id {
                let _ = lock_record!(connection, dialect, target, step.owner);
            }
            let mut row = attempt_row!(connection, dialect, step.attempt, true);
            if row.matches(step) {
                terminal!(connection, dialect, row, failure.stored(), now);
                return Ok(DeviceStepSettlement::Recorded);
            }
            Ok(DeviceStepSettlement::AlreadySettled)
        })
    }
}
