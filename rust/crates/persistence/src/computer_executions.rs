//! Current-connection-authorized managed resource evidence. No external effects run here.

use crate::{
    Store, StoreError,
    component_keys::{authenticate_provider, key_matches},
    computers::valid_id,
    directory::database_now as db_now,
    store::{Dialect, operation},
};
use chapchoo_crypto::SecretDigest;
use chapchoo_domain::LifecyclePhase;
use diesel::{
    OptionalExtension,
    sql_types::{BigInt, Bool, Nullable, Text},
};
use diesel_async::RunQueryDsl;
use std::fmt;

/// The Control supplies its own instance/connection identity, not client assertions.
pub struct ProviderSource<'a> {
    pub provider_id: &'a str,
    pub control_id: &'a str,
    pub connection_id: &'a str,
    pub credential: &'a SecretDigest,
}
impl fmt::Debug for ProviderSource<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ProviderSource([REDACTED])")
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ComputerRequest<'a> {
    pub computer_id: &'a str,
    pub request_id: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManagedResourceKind {
    Pod,
    Pvc,
    BootstrapSecret,
}
impl ManagedResourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pod => "POD",
            Self::Pvc => "PVC",
            Self::BootstrapSecret => "BOOTSTRAP_SECRET",
        }
    }
}

#[derive(Debug)]
pub struct ExecutionDraft {
    pub id: String,
    pub namespace: String,
    pub pod_name: String,
    pub bootstrap_secret_name: String,
    pub saved_pvc_name: String,
}

#[derive(Debug, Clone, diesel::QueryableByName)]
pub struct ManagedResource {
    #[diesel(sql_type=Text)]
    pub resource_kind: String,
    #[diesel(sql_type=Text)]
    pub namespace: String,
    #[diesel(sql_type=Text)]
    pub name: String,
    #[diesel(sql_type=Nullable<Text>)]
    pub resource_uid: Option<String>,
    #[diesel(sql_type=Nullable<Text>)]
    pub execution_id: Option<String>,
    #[diesel(sql_type=Text)]
    pub presence: String,
    #[diesel(sql_type=Nullable<BigInt>)]
    pub observed_at: Option<i64>,
    #[diesel(sql_type=Nullable<BigInt>)]
    pub absence_confirmed_at: Option<i64>,
    #[diesel(sql_type=Nullable<Text>)]
    pub source_connection_id: Option<String>,
}

#[derive(Debug)]
pub struct ExecutionReservation {
    pub execution_id: String,
    pub initial_allocation: bool,
    pub resources: Vec<ManagedResource>,
}

/// Absence is a substrate retirement/quiescence conclusion, never a bare GET 404.
/// For an unobserved UID, all possibly accepted creates must have finished and
/// the exact candidate must be definitively absent before this evidence is sent.
#[derive(Debug, Clone, Copy)]
pub enum ResourceEvidence<'a> {
    OwnedPresent { uid: &'a str },
    DefinitivelyAbsent { expected_uid: Option<&'a str> },
    Unknown,
}

#[derive(Debug, Clone, Copy)]
pub struct ResourceReport<'a> {
    pub kind: ManagedResourceKind,
    pub namespace: &'a str,
    pub name: &'a str,
    pub execution_id: Option<&'a str>,
    pub evidence: ResourceEvidence<'a>,
}

#[derive(Debug, Clone, diesel::QueryableByName)]
pub struct ManagedComputer {
    #[diesel(sql_type=Text)]
    pub id: String,
    #[diesel(sql_type=Text)]
    pub current_request_id: String,
    #[diesel(sql_type=Text)]
    pub current_action: String,
    #[diesel(sql_type=Text)]
    pub current_phase: String,
    #[diesel(sql_type=Text)]
    pub desired_state: String,
    #[diesel(sql_type=Nullable<Text>)]
    pub current_execution_id: Option<String>,
    #[diesel(sql_type=Nullable<Text>)]
    pub expected_old_execution_id: Option<String>,
    #[diesel(sql_type=Text)]
    pub saved_environment: String,
    #[diesel(sql_type=Nullable<Text>)]
    pub environment_source_connection_id: Option<String>,
    #[diesel(sql_type=Nullable<BigInt>)]
    pub agent_removed_at: Option<i64>,
    #[diesel(sql_type=Nullable<BigInt>)]
    pub cleanup_completed_at: Option<i64>,
}

#[derive(diesel::QueryableByName)]
struct Execution {
    #[diesel(sql_type=Bool)]
    initial_allocation: bool,
    #[diesel(sql_type=Nullable<BigInt>)]
    running_confirmed_at: Option<i64>,
    #[diesel(sql_type=Nullable<BigInt>)]
    retired_at: Option<i64>,
}
#[derive(diesel::QueryableByName)]
struct Count {
    #[diesel(sql_type=BigInt)]
    n: i64,
}

fn computer_lock(dialect: Dialect) -> &'static str {
    match dialect {
        Dialect::Postgres => {
            "SELECT * FROM computers WHERE id=? AND provider_id=? FOR NO KEY UPDATE"
        }
        Dialect::Sqlite => "SELECT * FROM computers WHERE id=? AND provider_id=?",
    }
}
fn valid_object(value: &str, limit: usize) -> bool {
    !value.is_empty()
        && value.len() <= limit
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'-' | b'.'))
        && value
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphanumeric)
        && value
            .as_bytes()
            .last()
            .is_some_and(u8::is_ascii_alphanumeric)
}
fn validate_source(source: &ProviderSource<'_>) -> Result<(), StoreError> {
    if [source.provider_id, source.control_id, source.connection_id]
        .into_iter()
        .all(valid_id)
    {
        Ok(())
    } else {
        Err(StoreError::InvalidInput)
    }
}
fn validate_scope(scope: ComputerRequest<'_>) -> Result<(), StoreError> {
    if valid_id(scope.computer_id) && valid_id(scope.request_id) {
        Ok(())
    } else {
        Err(StoreError::InvalidInput)
    }
}

// Lock order: Provider -> route -> Control (shared) -> Computer. The shared
// Control lock fences draining/expiry renewal without serializing its Providers.
macro_rules! current_source {
    ($connection:expr,$dialect:expr,$source:expr) => {{
        let source = $source;
        let now = $crate::directory::database_now!($connection, $dialect);
        let _provider = authenticate_provider!(
            $connection,
            $dialect,
            source.provider_id,
            source.credential,
            now
        );
        let route_query = match $dialect {
            Dialect::Postgres => {
                "SELECT * FROM provider_connections WHERE provider_id=? FOR NO KEY UPDATE"
            }
            Dialect::Sqlite => "SELECT * FROM provider_connections WHERE provider_id=?",
        };
        let route = diesel::sql_query($dialect.query(route_query))
            .bind::<Text, _>(source.provider_id)
            .get_result::<$crate::ComponentRoute>($connection)
            .await
            .optional()?
            .ok_or(StoreError::Unauthenticated)?;
        if route.control_id != source.control_id
            || route.connection_id != source.connection_id
            || !key_matches(Some(&route.credential_hash), source.credential)?
        {
            return Err(StoreError::Unauthenticated);
        }
        let control_query = match $dialect {
            Dialect::Postgres => "SELECT * FROM control_instances WHERE id=? FOR SHARE",
            Dialect::Sqlite => "SELECT * FROM control_instances WHERE id=?",
        };
        let control = diesel::sql_query($dialect.query(control_query))
            .bind::<Text, _>(source.control_id)
            .get_result::<$crate::ControlRegistration>($connection)
            .await
            .optional()?
            .ok_or(StoreError::Unauthenticated)?;
        let now = $crate::directory::database_now!($connection, $dialect);
        let provider = authenticate_provider!(
            $connection,
            $dialect,
            source.provider_id,
            source.credential,
            now
        );
        if route.expires_at <= now || control.expires_at <= now || control.draining {
            return Err(StoreError::Unauthenticated);
        }
        (provider, now)
    }};
}
pub(crate) use current_source;
macro_rules! current_computer {
    ($connection:expr,$dialect:expr,$source:expr,$scope:expr) => {{
        let row = diesel::sql_query($dialect.query(computer_lock($dialect)))
            .bind::<Text, _>($scope.computer_id)
            .bind::<Text, _>($source.provider_id)
            .get_result::<ManagedComputer>($connection)
            .await
            .optional()?
            .ok_or(StoreError::NotFound)?;
        if row.current_request_id != $scope.request_id {
            return Err(StoreError::Conflict);
        }
        // Locks may have waited. Recheck time-dependent authority at admission.
        let _ = current_source!($connection, $dialect, $source);
        row
    }};
}
macro_rules! inventory {
    ($connection:expr,$dialect:expr,$computer:expr) => {
        diesel::sql_query($dialect.query("SELECT * FROM computer_resources WHERE computer_id=? ORDER BY resource_kind,namespace,name"))
            .bind::<Text,_>($computer).load::<ManagedResource>($connection).await?
    };
}
macro_rules! revoke_execution {
    ($connection:expr,$dialect:expr,$execution:expr,$now:expr) => {{
        diesel::sql_query($dialect.query("DELETE FROM connector_connections WHERE session_id IN (SELECT id FROM connector_sessions WHERE execution_id=?)"))
            .bind::<Text,_>($execution).execute($connection).await?;
        diesel::sql_query($dialect.query("UPDATE connector_sessions SET revoked_at=COALESCE(revoked_at,?),initial_nonce=NULL,initial_ciphertext=NULL,current_hash=NULL,previous_hash=NULL,previous_valid_until=NULL,issued_at=NULL,expires_at=NULL,current_request_id=NULL,recovery_nonce=NULL,recovery_ciphertext=NULL WHERE execution_id=?"))
            .bind::<BigInt,_>($now).bind::<Text,_>($execution).execute($connection).await?;
    }};
}

impl Store {
    /// One conservative prepare admission per accepted lifecycle request. A
    /// lost dispatch/ACK does not clear this marker or authorize create replay.
    pub async fn claim_execution_dispatch(
        &self,
        source: &ProviderSource<'_>,
        scope: ComputerRequest<'_>,
        execution: &str,
    ) -> Result<bool, StoreError> {
        validate_source(source)?;
        validate_scope(scope)?;
        operation!(self, |connection, dialect| {
            let _ = current_source!(connection, dialect, source);
            let current = current_computer!(connection, dialect, source, scope);
            if current.desired_state != "RUNNING"
                || current.current_phase != "STARTING"
                || current.current_execution_id.as_deref() != Some(execution)
            {
                return Err(StoreError::InvalidTransition);
            }
            let now = db_now!(connection, dialect);
            Ok(diesel::sql_query(dialect.query("UPDATE computers SET prepare_admitted_at=? WHERE id=? AND current_request_id=? AND current_execution_id=? AND prepare_admitted_at IS NULL"))
                .bind::<BigInt,_>(now).bind::<Text,_>(scope.computer_id).bind::<Text,_>(scope.request_id).bind::<Text,_>(execution).execute(connection).await?==1)
        })
    }
    /// Clear only a claim whose call was definitely rejected before enqueue.
    /// Possible forwarding, cancellation and lost outcomes retain the claim.
    pub async fn release_undelivered_prepare(
        &self,
        source: &ProviderSource<'_>,
        scope: ComputerRequest<'_>,
        execution: &str,
    ) -> Result<(), StoreError> {
        validate_source(source)?;
        validate_scope(scope)?;
        operation!(self, |connection, dialect| {
            let _ = current_source!(connection, dialect, source);
            let _ = current_computer!(connection, dialect, source, scope);
            diesel::sql_query(dialect.query("UPDATE computers SET prepare_admitted_at=NULL WHERE id=? AND current_request_id=? AND current_execution_id=? AND current_phase='STARTING' AND initial_start_dispatch_at IS NULL"))
                .bind::<Text,_>(scope.computer_id).bind::<Text,_>(scope.request_id).bind::<Text,_>(execution).execute(connection).await?;
            Ok(())
        })
    }
    /// The exact current Provider has ACKed prior resource allocations and
    /// confirms that no Pod submission may have been accepted.
    pub async fn report_preparation_blocked(
        &self,
        source: &ProviderSource<'_>,
        scope: ComputerRequest<'_>,
        execution: &str,
    ) -> Result<(), StoreError> {
        validate_source(source)?;
        validate_scope(scope)?;
        operation!(self, |connection, dialect| {
            let _ = current_source!(connection, dialect, source);
            let current = current_computer!(connection, dialect, source, scope);
            if current.desired_state != "RUNNING"
                || current.current_execution_id.as_deref() != Some(execution)
                || current.current_phase != "STARTING"
            {
                return Err(StoreError::Conflict);
            }
            let resources = inventory!(connection, dialect, scope.computer_id);
            if !resources.iter().any(|r| {
                r.resource_kind == "POD"
                    && r.execution_id.as_deref() == Some(execution)
                    && r.resource_uid.is_none()
            }) {
                return Err(StoreError::InvalidTransition);
            }
            diesel::sql_query(dialect.query("UPDATE computers SET current_phase='BLOCKED',prepare_admitted_at=NULL WHERE id=? AND current_request_id=? AND current_execution_id=?"))
                .bind::<Text,_>(scope.computer_id).bind::<Text,_>(scope.request_id).bind::<Text,_>(execution).execute(connection).await?;
            Ok(())
        })
    }
    pub async fn connector_managed_computer(
        &self,
        source: &crate::ConnectorSource<'_>,
    ) -> Result<ManagedComputer, StoreError> {
        let identity = self.verify_connector_connection(source).await?;
        operation!(self, |connection, dialect| {
            Ok(
                diesel::sql_query(dialect.query("SELECT * FROM computers WHERE id=?"))
                    .bind::<Text, _>(&identity.computer_id)
                    .get_result::<ManagedComputer>(connection)
                    .await?,
            )
        })
    }
    /// Component dequeue authority is captured with the exact Provider source and
    /// Computer request under the same short transaction.
    pub async fn provider_delivery_snapshot(
        &self,
        source: &ProviderSource<'_>,
        scope: ComputerRequest<'_>,
        execution_id: Option<&str>,
    ) -> Result<crate::DeliverySnapshot, StoreError> {
        validate_source(source)?;
        validate_scope(scope)?;
        operation!(self, |connection, dialect| {
            let _ = current_source!(connection, dialect, source);
            let current = current_computer!(connection, dialect, source, scope);
            if execution_id.is_some_and(|id| {
                current.current_execution_id.as_deref() != Some(id)
                    && current.expected_old_execution_id.as_deref() != Some(id)
            }) {
                return Err(StoreError::Conflict);
            }
            let computer = diesel::sql_query(dialect.query("SELECT * FROM computers WHERE id=?"))
                .bind::<Text, _>(scope.computer_id)
                .get_result::<crate::DeliveryComputer>(connection)
                .await?;
            if computer.cleanup_completed_at.is_some() {
                return Err(StoreError::Conflict);
            }
            let execution =
                if let Some(id) = execution_id {
                    Some(
                        diesel::sql_query(dialect.query(
                            "SELECT * FROM computer_executions WHERE id=? AND computer_id=?",
                        ))
                        .bind::<Text, _>(id)
                        .bind::<Text, _>(scope.computer_id)
                        .get_result::<crate::DeliveryExecution>(connection)
                        .await
                        .optional()?
                        .ok_or(StoreError::NotFound)?,
                    )
                } else {
                    None
                };
            Ok(crate::DeliverySnapshot {
                computer,
                execution,
                resources: inventory!(connection, dialect, scope.computer_id),
            })
        })
    }
    /// Admission for a frame or dequeue rechecks authority without renewing the
    /// heartbeat lease. A previous successful check is never a cached permit.
    pub async fn verify_provider_connection(
        &self,
        source: &ProviderSource<'_>,
    ) -> Result<(), StoreError> {
        validate_source(source)?;
        operation!(self, |connection, dialect| {
            let _ = current_source!(connection, dialect, source);
            Ok(())
        })
    }
    /// Reserve stable identities before creating a PVC, Secret or Pod. Recovery
    /// returns the original candidate even when a new draft was prepared.
    pub async fn reserve_execution(
        &self,
        source: &ProviderSource<'_>,
        scope: ComputerRequest<'_>,
        draft: &ExecutionDraft,
    ) -> Result<ExecutionReservation, StoreError> {
        validate_source(source)?;
        validate_scope(scope)?;
        if !valid_id(&draft.id)
            || !valid_object(&draft.namespace, 63)
            || [
                &draft.pod_name,
                &draft.bootstrap_secret_name,
                &draft.saved_pvc_name,
            ]
            .into_iter()
            .any(|v| !valid_object(v, 253))
        {
            return Err(StoreError::InvalidInput);
        }
        operation!(self, |connection, dialect| {
            let (provider, _) = current_source!(connection, dialect, source);
            let current = current_computer!(connection, dialect, source, scope);
            let now = db_now!(connection, dialect);
            if current.desired_state != "RUNNING"
                || current.current_phase == "RETIRING"
                || (current.current_action == "RESTART"
                    && current.current_execution_id.is_some()
                    && current.current_execution_id == current.expected_old_execution_id)
            {
                return Err(StoreError::InvalidTransition);
            }
            if let Some(execution_id) = current.current_execution_id {
                let execution = diesel::sql_query(
                    dialect.query("SELECT * FROM computer_executions WHERE id=? AND computer_id=?"),
                )
                .bind::<Text, _>(&execution_id)
                .bind::<Text, _>(scope.computer_id)
                .get_result::<Execution>(connection)
                .await?;
                if execution.retired_at.is_some() {
                    return Err(StoreError::Integrity);
                }
                return Ok(ExecutionReservation {
                    execution_id,
                    initial_allocation: execution.initial_allocation,
                    resources: inventory!(connection, dialect, scope.computer_id),
                });
            }
            let resources = inventory!(connection, dialect, scope.computer_id);
            let pvc = resources.iter().find(|r| r.resource_kind == "PVC");
            if pvc.is_some_and(|r| r.namespace != draft.namespace) {
                return Err(StoreError::InvalidTransition);
            }
            let allocate_storage = current.current_action == "CREATE"
                && (resources.is_empty()
                    || pvc.is_some_and(|r| {
                        r.resource_uid.is_none()
                            && r.presence == "ABSENT"
                            && r.source_connection_id.as_deref() == Some(source.connection_id)
                    }));
            let prior_running = diesel::sql_query(dialect.query("SELECT count(*) AS n FROM computer_executions WHERE computer_id=? AND running_confirmed_at IS NOT NULL"))
                .bind::<Text,_>(scope.computer_id).get_result::<Count>(connection).await?;
            let initial = current.current_action == "CREATE" && prior_running.n == 0;
            if (initial && !provider.can_create) || (!initial && !provider.can_start) {
                return Err(StoreError::InvalidTransition);
            }
            if !allocate_storage
                && !pvc.is_some_and(|r| {
                    r.resource_uid.is_some()
                        && r.presence == "PRESENT"
                        && r.source_connection_id.as_deref() == Some(source.connection_id)
                })
            {
                return Err(StoreError::InvalidTransition);
            }
            if let Some(old) = current.expected_old_execution_id {
                let execution = diesel::sql_query(
                    dialect.query("SELECT * FROM computer_executions WHERE id=? AND computer_id=?"),
                )
                .bind::<Text, _>(&old)
                .bind::<Text, _>(scope.computer_id)
                .get_result::<Execution>(connection)
                .await?;
                if execution.retired_at.is_none() {
                    return Err(StoreError::InvalidTransition);
                }
            }
            diesel::sql_query(
                dialect.query(
                    "INSERT INTO computer_executions(id,computer_id,initial_allocation,created_at) VALUES(?,?,?,?)",
                ),
            )
            .bind::<Text, _>(&draft.id)
            .bind::<Text, _>(scope.computer_id)
            .bind::<Bool, _>(initial)
            .bind::<BigInt, _>(now)
            .execute(connection)
            .await?;
            for (kind, name, execution) in [
                (
                    ManagedResourceKind::Pod,
                    draft.pod_name.as_str(),
                    Some(draft.id.as_str()),
                ),
                (
                    ManagedResourceKind::BootstrapSecret,
                    draft.bootstrap_secret_name.as_str(),
                    Some(draft.id.as_str()),
                ),
                (
                    ManagedResourceKind::Pvc,
                    draft.saved_pvc_name.as_str(),
                    None,
                ),
            ] {
                if kind == ManagedResourceKind::Pvc && pvc.is_some() {
                    continue;
                }
                diesel::sql_query(dialect.query("INSERT INTO computer_resources(computer_id,provider_id,resource_kind,namespace,name,execution_id,presence) VALUES(?,?,?,?,?,?,'UNKNOWN')"))
                    .bind::<Text,_>(scope.computer_id).bind::<Text,_>(source.provider_id).bind::<Text,_>(kind.as_str()).bind::<Text,_>(&draft.namespace).bind::<Text,_>(name).bind::<Nullable<Text>,_>(execution).execute(connection).await?;
            }
            diesel::sql_query(dialect.query("UPDATE computers SET current_execution_id=?,current_phase='STARTING',observed_power='UNKNOWN',power_source_connection_id=NULL,power_source_execution_id=NULL,prepare_admitted_at=NULL,initial_start_dispatch_at=NULL WHERE id=?"))
                .bind::<Text,_>(&draft.id).bind::<Text,_>(scope.computer_id).execute(connection).await?;
            Ok(ExecutionReservation {
                execution_id: draft.id.clone(),
                initial_allocation: initial,
                resources: inventory!(connection, dialect, scope.computer_id),
            })
        })
    }

    pub async fn report_resource(
        &self,
        source: &ProviderSource<'_>,
        scope: ComputerRequest<'_>,
        report: ResourceReport<'_>,
    ) -> Result<(), StoreError> {
        validate_source(source)?;
        validate_scope(scope)?;
        if !valid_object(report.namespace, 63)
            || !valid_object(report.name, 253)
            || report.execution_id.is_some_and(|v| !valid_id(v))
        {
            return Err(StoreError::InvalidInput);
        }
        operation!(self, |connection, dialect| {
            let _ = current_source!(connection, dialect, source);
            let current = current_computer!(connection, dialect, source, scope);
            let now = db_now!(connection, dialect);
            if current.cleanup_completed_at.is_some() {
                return Err(StoreError::Conflict);
            }
            let resource = diesel::sql_query(dialect.query("SELECT * FROM computer_resources WHERE computer_id=? AND resource_kind=? AND namespace=? AND name=?"))
                .bind::<Text,_>(scope.computer_id).bind::<Text,_>(report.kind.as_str()).bind::<Text,_>(report.namespace).bind::<Text,_>(report.name).get_result::<ManagedResource>(connection).await.optional()?.ok_or(StoreError::NotFound)?;
            if resource.execution_id.as_deref() != report.execution_id {
                return Err(StoreError::Conflict);
            }
            let (presence, uid) = match report.evidence {
                ResourceEvidence::OwnedPresent { uid } => {
                    if !valid_id(uid) {
                        return Err(StoreError::InvalidInput);
                    }
                    if current.cleanup_completed_at.is_some()
                        || resource
                            .resource_uid
                            .as_deref()
                            .is_some_and(|old| old != uid)
                        || (resource.absence_confirmed_at.is_some()
                            && resource.resource_uid.is_some())
                    {
                        return Err(StoreError::Conflict);
                    }
                    if let Some(execution) = &resource.execution_id {
                        let state = diesel::sql_query(dialect.query(
                            "SELECT * FROM computer_executions WHERE id=? AND computer_id=?",
                        ))
                        .bind::<Text, _>(execution)
                        .bind::<Text, _>(scope.computer_id)
                        .get_result::<Execution>(connection)
                        .await?;
                        if state.retired_at.is_some() {
                            return Err(StoreError::InvalidTransition);
                        }
                    }
                    ("PRESENT", Some(uid))
                }
                ResourceEvidence::DefinitivelyAbsent { expected_uid } => {
                    if resource.resource_uid.as_deref() != expected_uid {
                        return Err(StoreError::Conflict);
                    }
                    ("ABSENT", expected_uid)
                }
                ResourceEvidence::Unknown => ("UNKNOWN", resource.resource_uid.as_deref()),
            };
            diesel::sql_query(dialect.query("UPDATE computer_resources SET resource_uid=?,presence=?,observed_at=?,source_connection_id=?,absence_confirmed_at=CASE WHEN ? THEN NULL ELSE COALESCE(absence_confirmed_at,?) END WHERE computer_id=? AND resource_kind=? AND namespace=? AND name=?"))
                .bind::<Nullable<Text>,_>(uid).bind::<Text,_>(presence).bind::<BigInt,_>(now).bind::<Text,_>(source.connection_id)
                .bind::<Bool,_>(resource.resource_uid.is_none() && presence=="PRESENT")
                .bind::<Nullable<BigInt>,_>(if presence == "ABSENT" { Some(now) } else { None })
                .bind::<Text,_>(scope.computer_id).bind::<Text,_>(report.kind.as_str()).bind::<Text,_>(report.namespace).bind::<Text,_>(report.name).execute(connection).await?;
            if report.kind == ManagedResourceKind::Pvc {
                let saved = match presence {
                    "PRESENT" => "VERIFIED",
                    "ABSENT" => "MISSING",
                    _ => "UNKNOWN",
                };
                diesel::sql_query(dialect.query("UPDATE computers SET saved_environment=?,environment_source_connection_id=?,environment_observed_at=? WHERE id=?"))
                    .bind::<Text,_>(saved).bind::<Text,_>(source.connection_id).bind::<BigInt,_>(now).bind::<Text,_>(scope.computer_id).execute(connection).await?;
            }
            Ok(())
        })
    }

    /// Called at first actual start dispatch, not on reservation or reconnect.
    pub async fn mark_start_dispatched(
        &self,
        source: &ProviderSource<'_>,
        scope: ComputerRequest<'_>,
        execution_id: &str,
    ) -> Result<(), StoreError> {
        validate_source(source)?;
        validate_scope(scope)?;
        operation!(self, |connection, dialect| {
            let _ = current_source!(connection, dialect, source);
            let current = current_computer!(connection, dialect, source, scope);
            let now = db_now!(connection, dialect);
            if current.desired_state != "RUNNING"
                || current.current_execution_id.as_deref() != Some(execution_id)
                || current.current_phase == "RETIRING"
                || (current.current_action == "RESTART"
                    && current.expected_old_execution_id.as_deref() == Some(execution_id))
            {
                return Err(StoreError::InvalidTransition);
            }
            let (provider, _) = current_source!(connection, dialect, source);
            let execution = diesel::sql_query(
                dialect.query("SELECT * FROM computer_executions WHERE id=? AND computer_id=?"),
            )
            .bind::<Text, _>(execution_id)
            .bind::<Text, _>(scope.computer_id)
            .get_result::<Execution>(connection)
            .await?;
            if execution.retired_at.is_some() {
                return Err(StoreError::InvalidTransition);
            }
            let initial = execution.initial_allocation;
            if (initial && !provider.can_create) || (!initial && !provider.can_start) {
                return Err(StoreError::InvalidTransition);
            }
            diesel::sql_query(dialect.query("UPDATE computers SET initial_start_dispatch_at=COALESCE(initial_start_dispatch_at,?) WHERE id=?"))
                .bind::<BigInt,_>(now).bind::<Text,_>(scope.computer_id).execute(connection).await?;
            Ok(())
        })
    }

    /// Provider asserts actual running substrate state, not mere Pod existence.
    pub async fn report_execution_running(
        &self,
        source: &ProviderSource<'_>,
        scope: ComputerRequest<'_>,
        execution_id: &str,
    ) -> Result<(), StoreError> {
        validate_source(source)?;
        validate_scope(scope)?;
        operation!(self, |connection, dialect| {
            let _ = current_source!(connection, dialect, source);
            let current = current_computer!(connection, dialect, source, scope);
            let now = db_now!(connection, dialect);
            if current.desired_state != "RUNNING"
                || current.current_phase == "RETIRING"
                || current.current_execution_id.as_deref() != Some(execution_id)
                || (current.current_action == "RESTART"
                    && current.expected_old_execution_id.as_deref() == Some(execution_id))
            {
                return Err(StoreError::InvalidTransition);
            }
            let resources = inventory!(connection, dialect, scope.computer_id);
            for kind in ["POD", "BOOTSTRAP_SECRET", "PVC"] {
                if !resources.iter().any(|r| {
                    r.resource_kind == kind
                        && (kind == "PVC" || r.execution_id.as_deref() == Some(execution_id))
                        && r.resource_uid.is_some()
                        && r.presence == "PRESENT"
                        && r.source_connection_id.as_deref() == Some(source.connection_id)
                }) {
                    return Err(StoreError::InvalidTransition);
                }
            }
            diesel::sql_query(dialect.query("UPDATE computer_executions SET running_confirmed_at=COALESCE(running_confirmed_at,?) WHERE id=? AND computer_id=?"))
                .bind::<BigInt,_>(now).bind::<Text,_>(execution_id).bind::<Text,_>(scope.computer_id).execute(connection).await?;
            diesel::sql_query(dialect.query("UPDATE computers SET current_phase='COMPLETED',observed_power='RUNNING',power_observed_at=?,power_source_connection_id=?,power_source_execution_id=? WHERE id=?"))
                .bind::<BigInt,_>(now).bind::<Text,_>(source.connection_id).bind::<Text,_>(execution_id).bind::<Text,_>(scope.computer_id).execute(connection).await?;
            Ok(())
        })
    }

    /// Clears only the exact current candidate after definitive Pod retirement.
    /// Pending/unknown creates and terminating Pods cannot cross this boundary.
    pub async fn retire_execution(
        &self,
        source: &ProviderSource<'_>,
        scope: ComputerRequest<'_>,
        execution_id: &str,
    ) -> Result<(), StoreError> {
        validate_source(source)?;
        validate_scope(scope)?;
        operation!(self, |connection, dialect| {
            let _ = current_source!(connection, dialect, source);
            let current = current_computer!(connection, dialect, source, scope);
            let now = db_now!(connection, dialect);
            if current.current_execution_id.as_deref() != Some(execution_id) {
                if current.current_execution_id.is_none()
                    && current.expected_old_execution_id.as_deref() == Some(execution_id)
                {
                    let previous =
                        diesel::sql_query(dialect.query(
                            "SELECT * FROM computer_executions WHERE id=? AND computer_id=?",
                        ))
                        .bind::<Text, _>(execution_id)
                        .bind::<Text, _>(scope.computer_id)
                        .get_result::<Execution>(connection)
                        .await?;
                    if previous.retired_at.is_some() {
                        if current.cleanup_completed_at.is_some() {
                            return Ok(());
                        }
                        let resources = inventory!(connection, dialect, scope.computer_id);
                        if !resources.iter().any(|r| {
                            r.resource_kind == "POD"
                                && r.execution_id.as_deref() == Some(execution_id)
                                && r.presence == "ABSENT"
                                && r.source_connection_id.as_deref() == Some(source.connection_id)
                        }) {
                            return Err(StoreError::InvalidTransition);
                        }
                        // Reconnect must freshly establish stopped power. This
                        // replay changes no lifecycle phase or dispatch interval.
                        diesel::sql_query(dialect.query("UPDATE computers SET observed_power='STOPPED',power_observed_at=?,power_source_connection_id=?,power_source_execution_id=NULL WHERE id=?"))
                            .bind::<BigInt,_>(now).bind::<Text,_>(source.connection_id)
                            .bind::<Text,_>(scope.computer_id).execute(connection).await?;
                        return Ok(());
                    }
                }
                return Err(StoreError::Conflict);
            }
            let resources = inventory!(connection, dialect, scope.computer_id);
            if !resources.iter().any(|r| {
                r.resource_kind == "POD"
                    && r.execution_id.as_deref() == Some(execution_id)
                    && r.presence == "ABSENT"
                    && r.source_connection_id.as_deref() == Some(source.connection_id)
            }) {
                return Err(StoreError::InvalidTransition);
            }
            revoke_execution!(connection, dialect, execution_id, now);
            let execution = diesel::sql_query(
                dialect.query("SELECT * FROM computer_executions WHERE id=? AND computer_id=?"),
            )
            .bind::<Text, _>(execution_id)
            .bind::<Text, _>(scope.computer_id)
            .get_result::<Execution>(connection)
            .await?;
            diesel::sql_query(dialect.query("UPDATE computer_executions SET retired_at=COALESCE(retired_at,?) WHERE id=? AND computer_id=?"))
                .bind::<BigInt,_>(now).bind::<Text,_>(execution_id).bind::<Text,_>(scope.computer_id).execute(connection).await?;
            let phase = match current.desired_state.as_str() {
                "DELETING" if current.current_phase == "UNCERTAIN" => "UNCERTAIN",
                "DELETING" => "DELETING",
                "STOPPED" => "COMPLETED",
                "RUNNING" => "STARTING",
                _ => return Err(StoreError::Integrity),
            };
            diesel::sql_query(dialect.query("UPDATE computers SET current_execution_id=NULL,expected_old_execution_id=?,current_phase=?,initial_start_dispatch_at=CASE WHEN ? THEN NULL ELSE initial_start_dispatch_at END,observed_power='STOPPED',power_observed_at=?,power_source_connection_id=?,power_source_execution_id=NULL WHERE id=?"))
                .bind::<Text,_>(execution_id).bind::<Text,_>(phase)
                .bind::<Bool,_>(execution.running_confirmed_at.is_some() && current.desired_state=="RUNNING")
                .bind::<BigInt,_>(now).bind::<Text,_>(source.connection_id).bind::<Text,_>(scope.computer_id).execute(connection).await?;
            Ok(())
        })
    }

    pub async fn report_lifecycle_issue(
        &self,
        source: &ProviderSource<'_>,
        scope: ComputerRequest<'_>,
        execution_id: Option<&str>,
        phase: LifecyclePhase,
    ) -> Result<(), StoreError> {
        validate_source(source)?;
        validate_scope(scope)?;
        if !matches!(phase, LifecyclePhase::Blocked | LifecyclePhase::Uncertain) {
            return Err(StoreError::InvalidInput);
        }
        operation!(self, |connection, dialect| {
            let _ = current_source!(connection, dialect, source);
            let current = current_computer!(connection, dialect, source, scope);
            if current.current_execution_id.as_deref() != execution_id
                || current.cleanup_completed_at.is_some()
                || (current.desired_state == "STOPPED" && current.current_execution_id.is_none())
            {
                return Err(StoreError::Conflict);
            }
            diesel::sql_query(
                dialect.query(
                    "UPDATE computers SET current_phase=?,observed_power='UNKNOWN' WHERE id=?",
                ),
            )
            .bind::<Text, _>(
                if current.desired_state == "DELETING" && current.current_phase == "UNCERTAIN" {
                    "UNCERTAIN"
                } else {
                    phase.as_str()
                },
            )
            .bind::<Text, _>(scope.computer_id)
            .execute(connection)
            .await?;
            Ok(())
        })
    }

    pub async fn managed_computers(
        &self,
        source: &ProviderSource<'_>,
        after: Option<&str>,
        limit: i64,
    ) -> Result<Vec<ManagedComputer>, StoreError> {
        validate_source(source)?;
        if !(1..=100).contains(&limit) || after.is_some_and(|v| !valid_id(v)) {
            return Err(StoreError::InvalidInput);
        }
        operation!(self, |connection, dialect| {
            let _ = current_source!(connection, dialect, source);
            Ok(diesel::sql_query(
                dialect.query(
                    "SELECT * FROM computers WHERE provider_id=? AND id>? ORDER BY id LIMIT ?",
                ),
            )
            .bind::<Text, _>(source.provider_id)
            .bind::<Text, _>(after.unwrap_or(""))
            .bind::<BigInt, _>(limit)
            .load::<ManagedComputer>(connection)
            .await?)
        })
    }

    pub async fn managed_resources(
        &self,
        source: &ProviderSource<'_>,
        scope: ComputerRequest<'_>,
    ) -> Result<Vec<ManagedResource>, StoreError> {
        validate_source(source)?;
        validate_scope(scope)?;
        operation!(self, |connection, dialect| {
            let _ = current_source!(connection, dialect, source);
            let _ = current_computer!(connection, dialect, source, scope);
            Ok(inventory!(connection, dialect, scope.computer_id))
        })
    }

    pub async fn confirm_computer_cleanup(
        &self,
        source: &ProviderSource<'_>,
        scope: ComputerRequest<'_>,
    ) -> Result<(), StoreError> {
        validate_source(source)?;
        validate_scope(scope)?;
        operation!(self, |connection, dialect| {
            let _ = current_source!(connection, dialect, source);
            let current = current_computer!(connection, dialect, source, scope);
            let now = db_now!(connection, dialect);
            if current.desired_state != "DELETING" || current.current_execution_id.is_some() {
                return Err(StoreError::InvalidTransition);
            }
            if current.cleanup_completed_at.is_some() {
                return Ok(());
            }
            let live = diesel::sql_query(dialect.query("SELECT count(*) AS n FROM computer_executions WHERE computer_id=? AND retired_at IS NULL"))
                .bind::<Text,_>(scope.computer_id).get_result::<Count>(connection).await?;
            if live.n != 0 {
                return Err(StoreError::InvalidTransition);
            }
            if inventory!(connection, dialect, scope.computer_id)
                .iter()
                .any(|r| {
                    r.presence != "ABSENT"
                        || r.source_connection_id.as_deref() != Some(source.connection_id)
                })
            {
                return Err(StoreError::InvalidTransition);
            }
            diesel::sql_query(dialect.query("UPDATE computers SET removed_agent_id=COALESCE(removed_agent_id,(SELECT id FROM agents WHERE computer_id=computers.id)),agent_removed_at=COALESCE(agent_removed_at,?),cleanup_completed_at=?,current_phase='COMPLETED' WHERE id=?"))
                .bind::<BigInt,_>(now).bind::<BigInt,_>(now).bind::<Text,_>(scope.computer_id).execute(connection).await?;
            diesel::sql_query(dialect.query("DELETE FROM agents WHERE computer_id=?"))
                .bind::<Text, _>(scope.computer_id)
                .execute(connection)
                .await?;
            Ok(())
        })
    }

    /// Only confirmed completed inventory ages out; unresolved cleanup has no TTL.
    pub async fn collect_completed_computers(
        &self,
        source: &ProviderSource<'_>,
        limit: i64,
    ) -> Result<usize, StoreError> {
        validate_source(source)?;
        if !(1..=100).contains(&limit) {
            return Err(StoreError::InvalidInput);
        }
        operation!(self, |connection, dialect| {
            let (_, now) = current_source!(connection, dialect, source);
            let threshold = now.checked_sub(86_400_000).ok_or(StoreError::Integrity)?;
            let candidates = diesel::sql_query(dialect.query("SELECT * FROM computers WHERE provider_id=? AND desired_state='DELETING' AND agent_removed_at IS NOT NULL AND cleanup_completed_at<=? AND current_execution_id IS NULL AND NOT EXISTS(SELECT 1 FROM computer_resources r WHERE r.computer_id=computers.id AND r.presence<>'ABSENT') AND NOT EXISTS(SELECT 1 FROM computer_executions e WHERE e.computer_id=computers.id AND e.retired_at IS NULL) ORDER BY id LIMIT ?"))
                .bind::<Text,_>(source.provider_id).bind::<BigInt,_>(threshold).bind::<BigInt,_>(limit).load::<ManagedComputer>(connection).await?;
            let mut count = 0;
            for row in candidates {
                count += diesel::sql_query(
                    dialect.query("DELETE FROM computers WHERE id=? AND provider_id=?"),
                )
                .bind::<Text, _>(&row.id)
                .bind::<Text, _>(source.provider_id)
                .execute(connection)
                .await?;
            }
            Ok(count)
        })
    }
}
