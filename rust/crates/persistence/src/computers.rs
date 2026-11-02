use chapchoo_crypto::{SecretDigest, VerifiedSystemOwner, payload_fingerprint};
use chapchoo_domain::{
    ActionAvailability, ComputerActionContext, ComputerKind, ConnectionStatus, DesiredState,
    LifecycleAction, LifecyclePhase, LifecycleProgress, ObservedPower, PowerCapabilities,
    SavedEnvironment,
};
use diesel::{
    OptionalExtension,
    sql_types::{BigInt, Binary, Bool, Nullable, Text},
};
use diesel_async::RunQueryDsl;
use std::{fmt, str::FromStr};

use crate::{
    Store, StoreError,
    models::authenticate,
    store::{Dialect, operation},
};

#[derive(Clone, diesel::QueryableByName)]
pub(crate) struct ProviderRow {
    #[diesel(sql_type = Text)]
    pub id: String,
    #[diesel(sql_type = Text)]
    pub owner_kind: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub system_owner_id: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub user_owner_id: Option<String>,
    #[diesel(sql_type = Text)]
    pub name: String,
    #[diesel(sql_type = Bool)]
    pub can_create: bool,
    #[diesel(sql_type = Bool)]
    pub can_start: bool,
    #[diesel(sql_type = Bool)]
    pub can_stop: bool,
    #[diesel(sql_type = Bool)]
    pub can_restart: bool,
}
impl fmt::Debug for ProviderRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ProviderRow([REDACTED])")
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ProviderCapabilities {
    pub create: bool,
    pub start: bool,
    pub stop: bool,
    pub restart: bool,
}

#[derive(Clone)]
pub struct ProviderChoice {
    pub id: String,
    pub name: String,
}
impl fmt::Debug for ProviderChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ProviderChoice([REDACTED])")
    }
}

pub struct CreateCloudAgent {
    pub agent_id: String,
    pub computer_id: String,
    pub request_id: String,
    pub provider_id: String,
    pub name: String,
}
impl fmt::Debug for CreateCloudAgent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CreateCloudAgent([REDACTED])")
    }
}

#[derive(Clone)]
pub struct ComputerSummary {
    pub id: String,
    pub owner_user_id: String,
    pub kind: ComputerKind,
    pub provider_id: Option<String>,
    pub desired: DesiredState,
    pub current_request_id: String,
    pub progress: LifecycleProgress,
    pub power: ObservedPower,
    pub last_known_power: ObservedPower,
    pub management: ConnectionStatus,
    pub power_observed_at: Option<i64>,
    pub saved_environment: SavedEnvironment,
}
impl fmt::Debug for ComputerSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ComputerSummary([REDACTED])")
    }
}

#[derive(Clone)]
pub struct AgentSummary {
    pub id: String,
    pub name: String,
    pub computer: ComputerSummary,
}
impl fmt::Debug for AgentSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("AgentSummary([REDACTED])")
    }
}

#[derive(Debug)]
pub enum CreationOutcome {
    Present(AgentSummary),
    Removed { assigned_agent_id: String },
}

pub struct LifecycleRequest {
    pub request_id: String,
    pub expected_request_id: String,
    pub action: LifecycleAction,
}
impl fmt::Debug for LifecycleRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("LifecycleRequest([REDACTED])")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentRemoval {
    Removed,
    AlreadyRemoved,
}

#[derive(diesel::QueryableByName)]
struct CreationReceiptRow {
    #[diesel(sql_type = Binary)]
    payload_hash: Vec<u8>,
    #[diesel(sql_type = Text)]
    assigned_agent_id: String,
}

#[derive(diesel::QueryableByName)]
pub(crate) struct AgentComputerRow {
    #[diesel(sql_type = Text)]
    agent_id: String,
    #[diesel(sql_type = Text)]
    name: String,
    #[diesel(sql_type = Text)]
    id: String,
    #[diesel(sql_type = Text)]
    owner_user_id: String,
    #[diesel(sql_type = Text)]
    kind: String,
    #[diesel(sql_type = Nullable<Text>)]
    provider_id: Option<String>,
    #[diesel(sql_type = Text)]
    desired_state: String,
    #[diesel(sql_type = Text)]
    current_request_id: String,
    #[diesel(sql_type = Binary)]
    current_payload_hash: Vec<u8>,
    #[diesel(sql_type = Nullable<Text>)]
    current_execution_id: Option<String>,
    #[diesel(sql_type = Text)]
    current_action: String,
    #[diesel(sql_type = Text)]
    current_phase: String,
    #[diesel(sql_type = Nullable<BigInt>)]
    initial_start_dispatch_at: Option<i64>,
    #[diesel(sql_type = Text)]
    observed_power: String,
    #[diesel(sql_type = Nullable<BigInt>)]
    power_observed_at: Option<i64>,
    #[diesel(sql_type = Nullable<Text>)]
    power_source_connection_id: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    power_source_execution_id: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    environment_source_connection_id: Option<String>,
    #[diesel(sql_type = Text)]
    saved_environment: String,
    #[diesel(sql_type = Nullable<Text>)]
    provider_connection_id: Option<String>,
    #[diesel(sql_type = Nullable<BigInt>)]
    provider_connection_expires_at: Option<i64>,
    #[diesel(sql_type = Nullable<BigInt>)]
    control_expires_at: Option<i64>,
    #[diesel(sql_type = Nullable<Bool>)]
    control_draining: Option<bool>,
    #[diesel(sql_type = Nullable<BigInt>)]
    management_key_valid_until: Option<i64>,
}

const AGENT_QUERY: &str = "SELECT a.id AS agent_id,a.name,c.id,c.owner_user_id,c.kind,c.provider_id,c.desired_state,c.current_request_id,c.current_payload_hash,c.current_execution_id,c.current_action,c.current_phase,c.initial_start_dispatch_at,c.observed_power,c.power_observed_at,c.power_source_connection_id,c.power_source_execution_id,c.environment_source_connection_id,c.saved_environment,pc.connection_id AS provider_connection_id,pc.expires_at AS provider_connection_expires_at,ci.expires_at AS control_expires_at,ci.draining AS control_draining,CASE WHEN pk.revoked_at IS NOT NULL THEN NULL WHEN pc.credential_hash=pk.current_hash THEN pk.expires_at WHEN pc.credential_hash=pk.previous_hash THEN CASE WHEN pk.expires_at<pk.previous_valid_until THEN pk.expires_at ELSE pk.previous_valid_until END ELSE NULL END AS management_key_valid_until FROM agents a JOIN computers c ON c.id=a.computer_id LEFT JOIN provider_connections pc ON pc.provider_id=c.provider_id LEFT JOIN control_instances ci ON ci.id=pc.control_id LEFT JOIN provider_keys pk ON pk.provider_id=c.provider_id";

impl AgentComputerRow {
    fn management_active(&self, now: i64) -> bool {
        self.provider_connection_id.is_some()
            && self
                .provider_connection_expires_at
                .is_some_and(|expiry| expiry > now)
            && self.control_expires_at.is_some_and(|expiry| expiry > now)
            && self.control_draining == Some(false)
            && self
                .management_key_valid_until
                .is_some_and(|expiry| expiry > now)
    }

    fn current_power(&self, now: i64) -> Result<ObservedPower, StoreError> {
        let raw =
            ObservedPower::from_str(&self.observed_power).map_err(|_| StoreError::Integrity)?;
        if !self.management_active(now)
            || self.power_observed_at.is_none()
            || self.power_source_connection_id != self.provider_connection_id
            || self.power_source_execution_id != self.current_execution_id
        {
            return Ok(ObservedPower::Unknown);
        }
        match (raw, self.current_execution_id.is_some()) {
            (ObservedPower::Running, false) | (ObservedPower::Stopped, true) => {
                Ok(ObservedPower::Unknown)
            }
            _ => Ok(raw),
        }
    }

    fn summary(self, now: i64) -> Result<AgentSummary, StoreError> {
        let parsed = |error| {
            let _ = error;
            StoreError::Integrity
        };
        let saved_environment = match self.saved_environment.as_str() {
            "VERIFIED" => SavedEnvironment::Verified,
            "MISSING" => SavedEnvironment::Missing,
            "UNKNOWN" => SavedEnvironment::Unknown,
            _ => return Err(StoreError::Integrity),
        };
        let management = if self.management_active(now) {
            ConnectionStatus::Connected
        } else {
            ConnectionStatus::Disconnected
        };
        let power = self.current_power(now)?;
        Ok(AgentSummary {
            id: self.agent_id,
            name: self.name,
            computer: ComputerSummary {
                id: self.id,
                owner_user_id: self.owner_user_id,
                kind: ComputerKind::from_str(&self.kind).map_err(parsed)?,
                provider_id: self.provider_id,
                desired: DesiredState::from_str(&self.desired_state).map_err(parsed)?,
                current_request_id: self.current_request_id,
                progress: LifecycleProgress {
                    action: LifecycleAction::from_str(&self.current_action).map_err(parsed)?,
                    phase: LifecyclePhase::from_str(&self.current_phase).map_err(parsed)?,
                    initial_start_dispatch_ms: self.initial_start_dispatch_at,
                },
                power,
                last_known_power: ObservedPower::from_str(&self.observed_power).map_err(parsed)?,
                management,
                power_observed_at: self.power_observed_at,
                saved_environment,
            },
        })
    }
}

pub(crate) fn provider_lock(dialect: Dialect) -> &'static str {
    match dialect {
        Dialect::Postgres => "SELECT * FROM computer_providers WHERE id=? FOR NO KEY UPDATE",
        Dialect::Sqlite => "SELECT * FROM computer_providers WHERE id=?",
    }
}

pub(crate) fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b':'))
}
fn valid_name(value: &str) -> bool {
    !value.trim().is_empty() && value.chars().count() <= 256 && !value.chars().any(char::is_control)
}

macro_rules! available_provider {
    ($connection:expr, $dialect:expr, $provider:expr, $owner:expr, $now:expr) => {{
        let q = $dialect.query("SELECT p.* FROM computer_providers p JOIN provider_keys k ON k.provider_id=p.id JOIN provider_connections pc ON pc.provider_id=p.id JOIN control_instances ci ON ci.id=pc.control_id WHERE p.id=? AND (p.owner_kind='SYSTEM' OR p.user_owner_id=?) AND p.can_create=TRUE AND k.revoked_at IS NULL AND k.expires_at>? AND pc.expires_at>? AND ci.expires_at>? AND ci.draining=FALSE AND (pc.credential_hash=k.current_hash OR (pc.credential_hash=k.previous_hash AND k.previous_valid_until>?))");
        diesel::sql_query(q).bind::<Text,_>($provider).bind::<Text,_>($owner)
            .bind::<BigInt,_>($now).bind::<BigInt,_>($now).bind::<BigInt,_>($now).bind::<BigInt,_>($now)
            .get_result::<ProviderRow>($connection).await.optional()?.ok_or(StoreError::InvalidTransition)?
    }};
}

impl Store {
    pub async fn configure_system_provider(
        &self,
        proof: &VerifiedSystemOwner,
        name: &str,
        capabilities: ProviderCapabilities,
        now: i64,
    ) -> Result<(), StoreError> {
        if !valid_name(name) {
            return Err(StoreError::InvalidInput);
        }
        operation!(self, |connection, dialect| {
            let existing = diesel::sql_query(dialect.query(provider_lock(dialect)))
                .bind::<Text, _>(proof.provider_id())
                .get_result::<ProviderRow>(connection)
                .await
                .optional()?;
            if let Some(existing) = existing {
                if existing.owner_kind != "SYSTEM"
                    || existing.system_owner_id.as_deref() != Some(proof.owner_id())
                {
                    return Err(StoreError::Forbidden);
                }
                diesel::sql_query(dialect.query("UPDATE computer_providers SET name=?,can_create=?,can_start=?,can_stop=?,can_restart=? WHERE id=?"))
                    .bind::<Text,_>(name).bind::<Bool,_>(capabilities.create).bind::<Bool,_>(capabilities.start).bind::<Bool,_>(capabilities.stop).bind::<Bool,_>(capabilities.restart).bind::<Text,_>(proof.provider_id()).execute(connection).await?;
            } else {
                diesel::sql_query(dialect.query("INSERT INTO computer_providers(id,owner_kind,system_owner_id,name,provider_kind,can_create,can_start,can_stop,can_restart,created_at) VALUES(?,'SYSTEM',?,?,'KUBERNETES',?,?,?,?,?)"))
                    .bind::<Text,_>(proof.provider_id()).bind::<Text,_>(proof.owner_id()).bind::<Text,_>(name)
                    .bind::<Bool,_>(capabilities.create).bind::<Bool,_>(capabilities.start).bind::<Bool,_>(capabilities.stop).bind::<Bool,_>(capabilities.restart).bind::<BigInt,_>(now).execute(connection).await?;
            }
            Ok(())
        })
    }

    pub async fn available_providers(
        &self,
        actor: &SecretDigest,
        after: Option<&str>,
        limit: i64,
        now: i64,
    ) -> Result<Vec<ProviderChoice>, StoreError> {
        if !(1..=100).contains(&limit) || after.is_some_and(|id| !valid_id(id)) {
            return Err(StoreError::InvalidInput);
        }
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            let q = dialect.query("SELECT p.* FROM computer_providers p JOIN provider_keys k ON k.provider_id=p.id JOIN provider_connections pc ON pc.provider_id=p.id JOIN control_instances ci ON ci.id=pc.control_id WHERE (p.owner_kind='SYSTEM' OR p.user_owner_id=?) AND p.can_create=TRUE AND k.revoked_at IS NULL AND k.expires_at>? AND pc.expires_at>? AND ci.expires_at>? AND ci.draining=FALSE AND (pc.credential_hash=k.current_hash OR (pc.credential_hash=k.previous_hash AND k.previous_valid_until>?)) AND p.id>? ORDER BY p.id LIMIT ?");
            let rows = diesel::sql_query(q)
                .bind::<Text, _>(&user.id)
                .bind::<BigInt, _>(now)
                .bind::<BigInt, _>(now)
                .bind::<BigInt, _>(now)
                .bind::<BigInt, _>(now)
                .bind::<Text, _>(after.unwrap_or(""))
                .bind::<BigInt, _>(limit)
                .load::<ProviderRow>(connection)
                .await?;
            Ok(rows
                .into_iter()
                .map(|row| ProviderChoice {
                    id: row.id,
                    name: row.name,
                })
                .collect())
        })
    }

    pub async fn create_cloud_agent(
        &self,
        actor: &SecretDigest,
        draft: &CreateCloudAgent,
        now: i64,
    ) -> Result<CreationOutcome, StoreError> {
        if ![
            draft.agent_id.as_str(),
            draft.computer_id.as_str(),
            draft.request_id.as_str(),
            draft.provider_id.as_str(),
        ]
        .into_iter()
        .all(valid_id)
            || !valid_name(&draft.name)
        {
            return Err(StoreError::InvalidInput);
        }
        let fingerprint = payload_fingerprint(
            "create-cloud-agent",
            &[draft.provider_id.as_bytes(), draft.name.as_bytes()],
        );
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            if let Some(receipt) = diesel::sql_query(dialect.query("SELECT payload_hash,assigned_agent_id FROM agent_creation_receipts WHERE owner_user_id=? AND request_id=?"))
                .bind::<Text,_>(&user.id).bind::<Text,_>(&draft.request_id).get_result::<CreationReceiptRow>(connection).await.optional()? {
                if receipt.payload_hash != fingerprint { return Err(StoreError::Conflict); }
                let query = format!("{AGENT_QUERY} WHERE a.id=? AND a.owner_user_id=?");
                let current = diesel::sql_query(dialect.query(&query)).bind::<Text,_>(&receipt.assigned_agent_id).bind::<Text,_>(&user.id).get_result::<AgentComputerRow>(connection).await.optional()?;
                return match current {
                    Some(current) => Ok(CreationOutcome::Present(current.summary(now)?)),
                    None => Ok(CreationOutcome::Removed { assigned_agent_id: receipt.assigned_agent_id }),
                };
            }
            let provider = diesel::sql_query(dialect.query(provider_lock(dialect)))
                .bind::<Text, _>(&draft.provider_id)
                .get_result::<ProviderRow>(connection)
                .await
                .optional()?
                .ok_or(StoreError::InvalidTransition)?;
            if !provider.can_create
                || (provider.owner_kind != "SYSTEM"
                    && provider.user_owner_id.as_deref() != Some(user.id.as_str()))
            {
                return Err(StoreError::InvalidTransition);
            }
            let _eligible = available_provider!(connection, dialect, &provider.id, &user.id, now);
            diesel::sql_query(dialect.query("INSERT INTO computers(id,owner_user_id,kind,provider_id,desired_state,current_request_id,current_payload_hash,current_action,current_phase,created_at) VALUES(?,?,'CLOUD',?,'RUNNING',?,?,'CREATE','ACCEPTED',?)"))
                .bind::<Text,_>(&draft.computer_id).bind::<Text,_>(&user.id).bind::<Text,_>(&draft.provider_id).bind::<Text,_>(&draft.request_id).bind::<Binary,_>(fingerprint.as_slice()).bind::<BigInt,_>(now).execute(connection).await?;
            diesel::sql_query(dialect.query("INSERT INTO agents(id,owner_user_id,name,computer_id,computer_kind,created_at) VALUES(?,?,?,?,'CLOUD',?)"))
                .bind::<Text,_>(&draft.agent_id).bind::<Text,_>(&user.id).bind::<Text,_>(&draft.name).bind::<Text,_>(&draft.computer_id).bind::<BigInt,_>(now).execute(connection).await?;
            diesel::sql_query(dialect.query("INSERT INTO agent_creation_receipts(owner_user_id,request_id,payload_hash,assigned_agent_id) VALUES(?,?,?,?)"))
                .bind::<Text,_>(&user.id).bind::<Text,_>(&draft.request_id).bind::<Binary,_>(fingerprint.as_slice()).bind::<Text,_>(&draft.agent_id).execute(connection).await?;
            let query = format!("{AGENT_QUERY} WHERE a.id=? AND a.owner_user_id=?");
            let row = diesel::sql_query(dialect.query(&query))
                .bind::<Text, _>(&draft.agent_id)
                .bind::<Text, _>(&user.id)
                .get_result::<AgentComputerRow>(connection)
                .await?;
            Ok(CreationOutcome::Present(row.summary(now)?))
        })
    }

    pub async fn agent(
        &self,
        actor: &SecretDigest,
        agent_id: &str,
        now: i64,
    ) -> Result<AgentSummary, StoreError> {
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            let query = format!("{AGENT_QUERY} WHERE a.id=? AND a.owner_user_id=?");
            diesel::sql_query(dialect.query(&query))
                .bind::<Text, _>(agent_id)
                .bind::<Text, _>(&user.id)
                .get_result::<AgentComputerRow>(connection)
                .await
                .optional()?
                .ok_or(StoreError::NotFound)?
                .summary(now)
        })
    }

    pub async fn request_lifecycle(
        &self,
        actor: &SecretDigest,
        agent_id: &str,
        request: &LifecycleRequest,
        now: i64,
    ) -> Result<AgentSummary, StoreError> {
        if !valid_id(agent_id)
            || !valid_id(&request.request_id)
            || !valid_id(&request.expected_request_id)
            || request.action == LifecycleAction::Create
        {
            return Err(StoreError::InvalidInput);
        }
        let fingerprint = payload_fingerprint(
            "computer-lifecycle",
            &[
                agent_id.as_bytes(),
                request.action.as_str().as_bytes(),
                request.expected_request_id.as_bytes(),
            ],
        );
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            let query = format!("{AGENT_QUERY} WHERE a.id=? AND a.owner_user_id=?");
            let initial = diesel::sql_query(dialect.query(&query))
                .bind::<Text, _>(agent_id)
                .bind::<Text, _>(&user.id)
                .get_result::<AgentComputerRow>(connection)
                .await
                .optional()?
                .ok_or(StoreError::NotFound)?;
            let provider_id = initial.provider_id.ok_or(StoreError::InvalidTransition)?;
            let provider = diesel::sql_query(dialect.query(provider_lock(dialect)))
                .bind::<Text, _>(&provider_id)
                .get_result::<ProviderRow>(connection)
                .await
                .optional()?
                .ok_or(StoreError::InvalidTransition)?;
            let locked = match dialect {
                Dialect::Postgres => format!("{query} FOR NO KEY UPDATE OF c"),
                Dialect::Sqlite => query.clone(),
            };
            let current = diesel::sql_query(dialect.query(&locked))
                .bind::<Text, _>(agent_id)
                .bind::<Text, _>(&user.id)
                .get_result::<AgentComputerRow>(connection)
                .await
                .optional()?
                .ok_or(StoreError::NotFound)?;
            if current.current_request_id == request.request_id {
                if current.current_payload_hash != fingerprint {
                    return Err(StoreError::Conflict);
                }
                return current.summary(now);
            }
            if current.current_request_id != request.expected_request_id {
                return Err(StoreError::Conflict);
            }
            if current.kind != "CLOUD" || current.desired_state == "DELETING" {
                return Err(StoreError::InvalidTransition);
            }
            let action = LifecycleAction::from_str(&current.current_action)
                .map_err(|_| StoreError::Integrity)?;
            let phase = LifecyclePhase::from_str(&current.current_phase)
                .map_err(|_| StoreError::Integrity)?;
            if request.action != LifecycleAction::Delete {
                let context = ComputerActionContext {
                    desired: DesiredState::from_str(&current.desired_state)
                        .map_err(|_| StoreError::Integrity)?,
                    power: current.current_power(now)?,
                    management: if current.management_active(now) {
                        ConnectionStatus::Connected
                    } else {
                        ConnectionStatus::Disconnected
                    },
                    capabilities: PowerCapabilities {
                        start: provider.can_start,
                        stop: provider.can_stop,
                        restart: provider.can_restart,
                    },
                    lifecycle: Some(LifecycleProgress {
                        action,
                        phase,
                        initial_start_dispatch_ms: current.initial_start_dispatch_at,
                    }),
                };
                if context.power_action(request.action) != ActionAvailability::Available {
                    return Err(StoreError::InvalidTransition);
                }
                if request.action == LifecycleAction::Start {
                    if current.current_execution_id.is_some()
                        || current.saved_environment != "VERIFIED"
                        || current.environment_source_connection_id
                            != current.provider_connection_id
                    {
                        return Err(StoreError::InvalidTransition);
                    }
                } else if current.current_execution_id.is_none() {
                    return Err(StoreError::InvalidTransition);
                }
            }
            let phase = match request.action {
                LifecycleAction::Stop | LifecycleAction::Restart => LifecyclePhase::Retiring,
                LifecycleAction::Delete => LifecyclePhase::Deleting,
                _ => LifecyclePhase::Accepted,
            };
            let changed = diesel::sql_query(dialect.query("UPDATE computers SET desired_state=?,current_request_id=?,current_payload_hash=?,current_action=?,current_phase=?,initial_start_dispatch_at=NULL,expected_old_execution_id=current_execution_id WHERE id=? AND current_request_id=?"))
                .bind::<Text,_>(request.action.desired_state().as_str()).bind::<Text,_>(&request.request_id).bind::<Binary,_>(fingerprint.as_slice())
                .bind::<Text,_>(request.action.as_str()).bind::<Text,_>(phase.as_str()).bind::<Text,_>(&current.id).bind::<Text,_>(&request.expected_request_id).execute(connection).await?;
            if changed != 1 {
                return Err(StoreError::Conflict);
            }
            if request.action == LifecycleAction::Delete {
                diesel::sql_query(dialect.query("UPDATE connector_sessions SET revoked_at=COALESCE(revoked_at,?),initial_nonce=NULL,initial_ciphertext=NULL,current_hash=NULL,previous_hash=NULL,previous_valid_until=NULL,issued_at=NULL,expires_at=NULL,current_request_id=NULL,recovery_nonce=NULL,recovery_ciphertext=NULL WHERE execution_id IN (SELECT id FROM computer_executions WHERE computer_id=?)"))
                    .bind::<BigInt,_>(now).bind::<Text,_>(&current.id).execute(connection).await?;
                diesel::sql_query(
                    dialect.query("DELETE FROM connector_connections WHERE computer_id=?"),
                )
                .bind::<Text, _>(&current.id)
                .execute(connection)
                .await?;
            }
            diesel::sql_query(dialect.query(&query))
                .bind::<Text, _>(agent_id)
                .bind::<Text, _>(&user.id)
                .get_result::<AgentComputerRow>(connection)
                .await?
                .summary(now)
        })
    }

    pub async fn force_remove_agent(
        &self,
        actor: &SecretDigest,
        agent_id: &str,
        expected_deletion_request_id: &str,
        now: i64,
    ) -> Result<AgentRemoval, StoreError> {
        if !valid_id(agent_id) || !valid_id(expected_deletion_request_id) {
            return Err(StoreError::InvalidInput);
        }
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            let query = format!("{AGENT_QUERY} WHERE a.id=? AND a.owner_user_id=?");
            let initial = diesel::sql_query(dialect.query(&query))
                .bind::<Text, _>(agent_id)
                .bind::<Text, _>(&user.id)
                .get_result::<AgentComputerRow>(connection)
                .await
                .optional()?;
            let Some(initial) = initial else {
                let prior = diesel::sql_query(dialect.query("SELECT payload_hash,assigned_agent_id FROM agent_creation_receipts WHERE owner_user_id=? AND assigned_agent_id=?"))
                    .bind::<Text,_>(&user.id).bind::<Text,_>(agent_id).get_result::<CreationReceiptRow>(connection).await.optional()?;
                return if prior.is_some() {
                    Ok(AgentRemoval::AlreadyRemoved)
                } else {
                    Err(StoreError::NotFound)
                };
            };
            let provider_id = initial.provider_id.ok_or(StoreError::InvalidTransition)?;
            diesel::sql_query(dialect.query(provider_lock(dialect)))
                .bind::<Text, _>(&provider_id)
                .get_result::<ProviderRow>(connection)
                .await
                .optional()?
                .ok_or(StoreError::InvalidTransition)?;
            let locked = match dialect {
                Dialect::Postgres => format!("{query} FOR NO KEY UPDATE OF c"),
                Dialect::Sqlite => query,
            };
            let current = diesel::sql_query(dialect.query(&locked))
                .bind::<Text, _>(agent_id)
                .bind::<Text, _>(&user.id)
                .get_result::<AgentComputerRow>(connection)
                .await
                .optional()?;
            let Some(current) = current else {
                return Ok(AgentRemoval::AlreadyRemoved);
            };
            if current.kind != "CLOUD"
                || current.desired_state != "DELETING"
                || current.current_request_id != expected_deletion_request_id
            {
                return Err(StoreError::InvalidTransition);
            }
            diesel::sql_query(dialect.query(
                "UPDATE computers SET agent_removed_at=? WHERE id=? AND desired_state='DELETING'",
            ))
            .bind::<BigInt, _>(now)
            .bind::<Text, _>(&current.id)
            .execute(connection)
            .await?;
            diesel::sql_query(dialect.query("DELETE FROM agents WHERE id=? AND owner_user_id=?"))
                .bind::<Text, _>(agent_id)
                .bind::<Text, _>(&user.id)
                .execute(connection)
                .await?;
            Ok(AgentRemoval::Removed)
        })
    }

    pub async fn agents(
        &self,
        actor: &SecretDigest,
        after: Option<&str>,
        limit: i64,
        now: i64,
    ) -> Result<Vec<AgentSummary>, StoreError> {
        if !(1..=100).contains(&limit) || after.is_some_and(|id| !valid_id(id)) {
            return Err(StoreError::InvalidInput);
        }
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            let query =
                format!("{AGENT_QUERY} WHERE a.owner_user_id=? AND a.id>? ORDER BY a.id LIMIT ?");
            let rows = diesel::sql_query(dialect.query(&query))
                .bind::<Text, _>(&user.id)
                .bind::<Text, _>(after.unwrap_or(""))
                .bind::<BigInt, _>(limit)
                .load::<AgentComputerRow>(connection)
                .await?;
            rows.into_iter().map(|row| row.summary(now)).collect()
        })
    }
}
