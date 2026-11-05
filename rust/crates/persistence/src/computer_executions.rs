//! Current-connection-authorized managed resource evidence. No external effects run here.

use crate::native_resources::{
    ManagedResource, ResourceEvidence, ResourceReport, ResourceRow, identity_columns,
};
use crate::{
    Store, StoreError,
    component_keys::{authenticate_provider, key_matches},
    computers::valid_id,
    directory::database_now as db_now,
    entity_id::{EntityId, EntityKind, IdentitySql},
    store::{Dialect, operation},
};
use chapchoo_crypto::SecretDigest;
use chapchoo_domain::{
    LifecyclePhase, NativeReservation, NativeScope, ResourcePresence, ResourceRole,
};
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

pub use chapchoo_domain::ManagedResourceKind;

pub(crate) fn validate_native_inventory(
    provider: &crate::computers::ProviderRow,
    resources: &[ManagedResource],
) -> Result<(), StoreError> {
    let substrate = provider.substrate()?;
    if resources.iter().any(|resource| {
        resource.kind().provider_kind() != substrate.kind()
            || !substrate.accepts_scope(resource.native.scope())
    }) {
        return Err(StoreError::Integrity);
    }
    Ok(())
}

#[derive(Debug)]
pub struct ExecutionDraft {
    pub native: ExecutionResources,
}

#[derive(Debug)]
pub enum ExecutionResources {
    Kubernetes {
        namespace: String,
        pod_name: String,
        bootstrap_secret_name: String,
        saved_pvc_name: String,
    },
    Docker {
        daemon: chapchoo_domain::DockerDaemonId,
        container_name: String,
        saved_volume_name: String,
    },
}

impl ExecutionDraft {
    fn scope(&self) -> Result<NativeScope, StoreError> {
        match &self.native {
            ExecutionResources::Kubernetes { namespace, .. } => {
                NativeScope::kubernetes(namespace.clone()).map_err(|_| StoreError::InvalidInput)
            }
            ExecutionResources::Docker { daemon, .. } => {
                Ok(NativeScope::DockerDaemon(daemon.clone()))
            }
        }
    }

    fn reservations(&self, scope: &NativeScope) -> Result<Vec<NativeReservation>, StoreError> {
        let names = match &self.native {
            ExecutionResources::Kubernetes {
                pod_name,
                bootstrap_secret_name,
                saved_pvc_name,
                ..
            } => vec![
                (ManagedResourceKind::Pod, pod_name),
                (ManagedResourceKind::BootstrapSecret, bootstrap_secret_name),
                (ManagedResourceKind::Pvc, saved_pvc_name),
            ],
            ExecutionResources::Docker {
                container_name,
                saved_volume_name,
                ..
            } => vec![
                (ManagedResourceKind::DockerContainer, container_name),
                (ManagedResourceKind::DockerVolume, saved_volume_name),
            ],
        };
        names
            .into_iter()
            .map(|(kind, name)| {
                NativeReservation::new(kind, scope.clone(), name.clone())
                    .map_err(|_| StoreError::InvalidInput)
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct ExecutionReservation {
    pub execution_id: String,
    pub initial_allocation: bool,
    pub resources: Vec<ManagedResource>,
}

/// Exact server phase admission, not a native effect receipt. The Control must
/// retain matching live source/call/native correlation before forwarding.
#[derive(Debug, Clone)]
pub struct NativeRetirementAdmission {
    provider_id: String,
    control_id: String,
    connection_id: String,
    computer_id: String,
    request_id: String,
    previous_phase: String,
    previous_expected_old: Option<String>,
    admitted_execution: Option<String>,
}

impl NativeRetirementAdmission {
    pub fn previous_phase(&self) -> &str {
        &self.previous_phase
    }
}

#[derive(Debug, Clone)]
pub struct ManagedComputer {
    pub id: String,
    pub current_request_id: String,
    pub current_action: String,
    pub current_phase: String,
    pub prepare_admitted_at: Option<i64>,
    pub desired_state: String,
    pub current_execution_id: Option<String>,
    pub expected_old_execution_id: Option<String>,
    pub saved_environment: String,
    pub environment_source_connection_id: Option<String>,
    pub agent_removed_at: Option<i64>,
    pub cleanup_completed_at: Option<i64>,
}
#[derive(diesel::QueryableByName)]
pub(crate) struct ManagedComputerRow {
    #[diesel(sql_type=IdentitySql)]
    id: EntityId,
    #[diesel(sql_type=Text)]
    current_request_id: String,
    #[diesel(sql_type=Text)]
    current_action: String,
    #[diesel(sql_type=Text)]
    current_phase: String,
    #[diesel(sql_type=Nullable<BigInt>)]
    prepare_admitted_at: Option<i64>,
    #[diesel(sql_type=Text)]
    desired_state: String,
    #[diesel(sql_type=Nullable<IdentitySql>)]
    current_execution_id: Option<EntityId>,
    #[diesel(sql_type=Nullable<IdentitySql>)]
    expected_old_execution_id: Option<EntityId>,
    #[diesel(sql_type=Text)]
    saved_environment: String,
    #[diesel(sql_type=Nullable<IdentitySql>)]
    environment_source_connection_id: Option<EntityId>,
    #[diesel(sql_type=Nullable<BigInt>)]
    agent_removed_at: Option<i64>,
    #[diesel(sql_type=Nullable<BigInt>)]
    cleanup_completed_at: Option<i64>,
}
impl From<ManagedComputerRow> for ManagedComputer {
    fn from(row: ManagedComputerRow) -> Self {
        Self {
            id: row.id.to_dto(EntityKind::Computer),
            current_request_id: row.current_request_id,
            current_action: row.current_action,
            current_phase: row.current_phase,
            prepare_admitted_at: row.prepare_admitted_at,
            desired_state: row.desired_state,
            current_execution_id: row
                .current_execution_id
                .map(|id| id.to_dto(EntityKind::Execution)),
            expected_old_execution_id: row
                .expected_old_execution_id
                .map(|id| id.to_dto(EntityKind::Execution)),
            saved_environment: row.saved_environment,
            environment_source_connection_id: row
                .environment_source_connection_id
                .map(|id| id.to_dto(EntityKind::Connection)),
            agent_removed_at: row.agent_removed_at,
            cleanup_completed_at: row.cleanup_completed_at,
        }
    }
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
            .bind::<$crate::entity_id::IdentitySql, _>($crate::entity_id::EntityId::from_dto(
                $crate::entity_id::EntityKind::Provider,
                source.provider_id,
            )?)
            .get_result::<$crate::directory::ComponentRouteRow>($connection)
            .await
            .map($crate::ComponentRoute::from)
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
            .bind::<$crate::entity_id::IdentitySql, _>($crate::entity_id::EntityId::from_dto(
                $crate::entity_id::EntityKind::ControlInstance,
                source.control_id,
            )?)
            .get_result::<$crate::directory::ControlRow>($connection)
            .await
            .map($crate::ControlRegistration::from)
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
            .bind::<IdentitySql, _>(EntityId::from_dto(
                EntityKind::Computer,
                $scope.computer_id,
            )?)
            .bind::<IdentitySql, _>(EntityId::from_dto(
                EntityKind::Provider,
                $source.provider_id,
            )?)
            .get_result::<ManagedComputerRow>($connection)
            .await
            .map(ManagedComputer::from)
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
    ($connection:expr,$dialect:expr,$computer:expr) => {{
        diesel::sql_query($dialect.query("SELECT * FROM computer_resources WHERE computer_id=? ORDER BY resource_kind,native_scope,name"))
            .bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::Computer,$computer)?).load::<ResourceRow>($connection).await?
            .into_iter().map(ManagedResource::try_from).collect::<Result<Vec<_>,StoreError>>()?
    }};
}
macro_rules! revoke_execution {
    ($connection:expr,$dialect:expr,$execution:expr,$now:expr) => {{
        diesel::sql_query($dialect.query("DELETE FROM connector_connections WHERE session_id IN (SELECT id FROM connector_sessions WHERE execution_id=?)"))
            .bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::Execution,$execution)?).execute($connection).await?;
        diesel::sql_query($dialect.query("UPDATE connector_sessions SET revoked_at=COALESCE(revoked_at,?),initial_nonce=NULL,initial_ciphertext=NULL,current_hash=NULL,previous_hash=NULL,previous_valid_until=NULL,issued_at=NULL,expires_at=NULL,current_request_id=NULL,recovery_nonce=NULL,recovery_ciphertext=NULL WHERE execution_id=?"))
            .bind::<BigInt,_>($now).bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::Execution,$execution)?).execute($connection).await?;
    }};
}

impl Store {
    /// The exact live Delete manifest is checked separately by Control. This
    /// transaction proves no current/live execution or unresolved container can
    /// precede that manifest's never-submitted saved-volume suffix.
    pub async fn verify_native_cleanup_suffix(
        &self,
        source: &ProviderSource<'_>,
        scope: ComputerRequest<'_>,
    ) -> Result<(), StoreError> {
        validate_source(source)?;
        validate_scope(scope)?;
        operation!(self, |connection, dialect| {
            let (provider, _) = current_source!(connection, dialect, source);
            let current = current_computer!(connection, dialect, source, scope);
            if provider.substrate()?.kind() != chapchoo_domain::ProviderKind::Docker
                || current.current_phase != "UNCERTAIN"
                || current.desired_state != "DELETING"
                || current.current_execution_id.is_some()
                || current.cleanup_completed_at.is_some()
            {
                return Err(StoreError::InvalidTransition);
            }
            let live = diesel::sql_query(dialect.query("SELECT count(*) AS n FROM computer_executions WHERE computer_id=? AND retired_at IS NULL"))
                .bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::Computer,scope.computer_id)?).get_result::<Count>(connection).await?;
            if live.n != 0 {
                return Err(StoreError::InvalidTransition);
            }
            let resources = inventory!(connection, dialect, scope.computer_id);
            validate_native_inventory(&provider, &resources)?;
            if resources.iter().any(|r| {
                r.kind() == ManagedResourceKind::DockerContainer
                    && (r.presence != ResourcePresence::Absent
                        || r.absence_confirmed_at.is_none()
                        || r.source_connection_id.as_deref() != Some(source.connection_id))
            }) {
                return Err(StoreError::InvalidTransition);
            }
            Ok(())
        })
    }

    /// `terminal_execution` belongs to Control's exact current-source terminal
    /// observation path, never to a caller-supplied require_terminal flag alone.
    pub async fn claim_native_retirement(
        &self,
        source: &ProviderSource<'_>,
        scope: ComputerRequest<'_>,
        terminal_execution: Option<&str>,
    ) -> Result<Option<NativeRetirementAdmission>, StoreError> {
        validate_source(source)?;
        validate_scope(scope)?;
        operation!(self, |connection, dialect| {
            let (provider, _) = current_source!(connection, dialect, source);
            let current = current_computer!(connection, dialect, source, scope);
            if provider.substrate()?.kind() != chapchoo_domain::ProviderKind::Docker
                || current.cleanup_completed_at.is_some()
            {
                return Err(StoreError::InvalidTransition);
            }
            if current.current_phase == "UNCERTAIN" {
                return Ok(None);
            }
            let terminal = terminal_execution.is_some();
            let allowed = if let Some(execution) = terminal_execution {
                current.desired_state == "RUNNING"
                    && current.current_execution_id.as_deref() == Some(execution)
                    && current.current_phase == "COMPLETED"
                    && current.prepare_admitted_at.is_some()
            } else {
                (current.current_phase == "RETIRING"
                    && current.current_execution_id.is_some()
                    && matches!(current.current_action.as_str(), "STOP" | "RESTART"))
                    || (current.current_phase == "DELETING"
                        && current.desired_state == "DELETING"
                        && current.current_action == "DELETE")
            };
            if !allowed {
                return Err(StoreError::InvalidTransition);
            }
            let admitted_execution = current.current_execution_id.clone();
            let changed = diesel::sql_query(dialect.query(
                "UPDATE computers SET current_phase='UNCERTAIN',expected_old_execution_id=CASE WHEN ? THEN current_execution_id ELSE expected_old_execution_id END WHERE id=? AND current_request_id=? AND current_phase=?",
            ))
                .bind::<Bool,_>(terminal).bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::Computer,scope.computer_id)?)
                .bind::<Text,_>(scope.request_id).bind::<Text,_>(&current.current_phase)
                .execute(connection).await?;
            if changed != 1 {
                return Err(StoreError::Conflict);
            }
            Ok(Some(NativeRetirementAdmission {
                provider_id: source.provider_id.into(),
                control_id: source.control_id.into(),
                connection_id: source.connection_id.into(),
                computer_id: scope.computer_id.into(),
                request_id: scope.request_id.into(),
                previous_phase: current.current_phase,
                previous_expected_old: current.expected_old_execution_id,
                admitted_execution,
            }))
        })
    }

    /// The Control must verify and serialize the exact live manifest's known
    /// NotStarted result before using this release. No DB call ledger exists.
    pub async fn release_unstarted_native_retirement(
        &self,
        source: &ProviderSource<'_>,
        admission: &NativeRetirementAdmission,
    ) -> Result<(), StoreError> {
        validate_source(source)?;
        if source.provider_id != admission.provider_id
            || source.control_id != admission.control_id
            || source.connection_id != admission.connection_id
        {
            return Err(StoreError::Conflict);
        }
        let scope = ComputerRequest {
            computer_id: &admission.computer_id,
            request_id: &admission.request_id,
        };
        operation!(self, |connection, dialect| {
            let (provider, _) = current_source!(connection, dialect, source);
            let current = current_computer!(connection, dialect, source, scope);
            if provider.substrate()?.kind() != chapchoo_domain::ProviderKind::Docker
                || current.current_phase != "UNCERTAIN"
                || current.current_execution_id != admission.admitted_execution
                || current.cleanup_completed_at.is_some()
            {
                return Err(StoreError::Conflict);
            }
            diesel::sql_query(dialect.query(
                "UPDATE computers SET current_phase=?,expected_old_execution_id=? WHERE id=? AND current_request_id=? AND current_phase='UNCERTAIN'",
            )).bind::<Text,_>(&admission.previous_phase)
                .bind::<Nullable<IdentitySql>,_>(admission.previous_expected_old.as_deref().map(|id|EntityId::from_dto(EntityKind::Execution,id)).transpose()?)
                .bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::Computer,scope.computer_id)?).bind::<Text,_>(scope.request_id)
                .execute(connection).await?;
            Ok(())
        })
    }

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
                .bind::<BigInt,_>(now).bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::Computer,scope.computer_id)?).bind::<Text,_>(scope.request_id).bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::Execution,execution)?).execute(connection).await?==1)
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
                .bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::Computer,scope.computer_id)?).bind::<Text,_>(scope.request_id).bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::Execution,execution)?).execute(connection).await?;
            Ok(())
        })
    }
    /// The exact current Provider has ACKed prior resource allocations and
    /// confirms that no execution-object submission may have been accepted.
    pub async fn report_preparation_blocked(
        &self,
        source: &ProviderSource<'_>,
        scope: ComputerRequest<'_>,
        execution: &str,
    ) -> Result<(), StoreError> {
        validate_source(source)?;
        validate_scope(scope)?;
        operation!(self, |connection, dialect| {
            let (provider, _) = current_source!(connection, dialect, source);
            let current = current_computer!(connection, dialect, source, scope);
            if current.desired_state != "RUNNING"
                || current.current_execution_id.as_deref() != Some(execution)
                || current.current_phase != "STARTING"
            {
                return Err(StoreError::Conflict);
            }
            let resources = inventory!(connection, dialect, scope.computer_id);
            validate_native_inventory(&provider, &resources)?;
            let kind = provider.substrate()?.kind().execution_resource_kind();
            if !resources.iter().any(|r| {
                r.kind() == kind
                    && r.execution_id.as_deref() == Some(execution)
                    && r.identity.is_none()
            }) {
                return Err(StoreError::InvalidTransition);
            }
            diesel::sql_query(dialect.query("UPDATE computers SET current_phase='BLOCKED',prepare_admitted_at=NULL WHERE id=? AND current_request_id=? AND current_execution_id=?"))
                .bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::Computer,scope.computer_id)?).bind::<Text,_>(scope.request_id).bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::Execution,execution)?).execute(connection).await?;
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
                    .bind::<IdentitySql, _>(EntityId::from_dto(
                        EntityKind::Computer,
                        &identity.computer_id,
                    )?)
                    .get_result::<ManagedComputerRow>(connection)
                    .await
                    .map(ManagedComputer::from)?,
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
            let (provider, _) = current_source!(connection, dialect, source);
            let current = current_computer!(connection, dialect, source, scope);
            if execution_id.is_some_and(|id| {
                current.current_execution_id.as_deref() != Some(id)
                    && current.expected_old_execution_id.as_deref() != Some(id)
            }) {
                return Err(StoreError::Conflict);
            }
            let computer = diesel::sql_query(dialect.query("SELECT * FROM computers WHERE id=?"))
                .bind::<IdentitySql, _>(EntityId::from_dto(
                    EntityKind::Computer,
                    scope.computer_id,
                )?)
                .get_result::<crate::delivery::DeliveryComputerRow>(connection)
                .await
                .map(crate::DeliveryComputer::from)?;
            if computer.cleanup_completed_at.is_some() {
                return Err(StoreError::Conflict);
            }
            let execution =
                if let Some(id) = execution_id {
                    Some(
                        diesel::sql_query(dialect.query(
                            "SELECT * FROM computer_executions WHERE id=? AND computer_id=?",
                        ))
                        .bind::<IdentitySql, _>(EntityId::from_dto(EntityKind::Execution, id)?)
                        .bind::<IdentitySql, _>(EntityId::from_dto(
                            EntityKind::Computer,
                            scope.computer_id,
                        )?)
                        .get_result::<crate::delivery::DeliveryExecutionRow>(connection)
                        .await
                        .map(crate::DeliveryExecution::from)
                        .optional()?
                        .ok_or(StoreError::NotFound)?,
                    )
                } else {
                    None
                };
            let resources = inventory!(connection, dialect, scope.computer_id);
            validate_native_inventory(&provider, &resources)?;
            Ok(crate::DeliverySnapshot {
                computer,
                execution,
                resources,
                substrate: provider.substrate()?,
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
        let native_scope = draft.scope()?;
        let reservations = draft.reservations(&native_scope)?;
        operation!(self, |connection, dialect| {
            let (provider, _) = current_source!(connection, dialect, source);
            let current = current_computer!(connection, dialect, source, scope);
            let now = db_now!(connection, dialect);
            let substrate = provider.substrate()?;
            if !substrate.accepts_scope(&native_scope) {
                return Err(StoreError::Conflict);
            }
            if current.desired_state != "RUNNING"
                || current.current_phase == "RETIRING"
                || (matches!(current.current_action.as_str(), "STOP" | "RESTART")
                    && current.current_execution_id.is_some()
                    && current.current_execution_id == current.expected_old_execution_id)
            {
                return Err(StoreError::InvalidTransition);
            }
            if let Some(execution_id) = current.current_execution_id {
                let execution = diesel::sql_query(
                    dialect.query("SELECT * FROM computer_executions WHERE id=? AND computer_id=?"),
                )
                .bind::<IdentitySql, _>(EntityId::from_dto(EntityKind::Execution, &execution_id)?)
                .bind::<IdentitySql, _>(EntityId::from_dto(
                    EntityKind::Computer,
                    scope.computer_id,
                )?)
                .get_result::<Execution>(connection)
                .await?;
                if execution.retired_at.is_some() {
                    return Err(StoreError::Integrity);
                }
                let resources = inventory!(connection, dialect, scope.computer_id);
                validate_native_inventory(&provider, &resources)?;
                return Ok(ExecutionReservation {
                    execution_id,
                    initial_allocation: execution.initial_allocation,
                    resources,
                });
            }
            let resources = inventory!(connection, dialect, scope.computer_id);
            validate_native_inventory(&provider, &resources)?;
            let saved = resources
                .iter()
                .find(|r| r.kind().role() == ResourceRole::SavedEnvironment);
            if saved.is_some_and(|r| r.native.scope() != &native_scope) {
                return Err(StoreError::InvalidTransition);
            }
            let allocate_storage = current.current_action == "CREATE"
                && (resources.is_empty()
                    || saved.is_some_and(|r| {
                        r.identity.is_none()
                            && r.presence == ResourcePresence::Absent
                            && r.source_connection_id.as_deref() == Some(source.connection_id)
                    }));
            let prior_running = diesel::sql_query(dialect.query("SELECT count(*) AS n FROM computer_executions WHERE computer_id=? AND running_confirmed_at IS NOT NULL"))
                .bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::Computer,scope.computer_id)?).get_result::<Count>(connection).await?;
            let initial = current.current_action == "CREATE" && prior_running.n == 0;
            if (initial && !provider.can_create) || (!initial && !provider.can_start) {
                return Err(StoreError::InvalidTransition);
            }
            if !allocate_storage
                && !saved.is_some_and(|r| {
                    r.identity.is_some()
                        && r.presence == ResourcePresence::Present
                        && r.source_connection_id.as_deref() == Some(source.connection_id)
                })
            {
                return Err(StoreError::InvalidTransition);
            }
            if let Some(old) = current.expected_old_execution_id {
                let execution = diesel::sql_query(
                    dialect.query("SELECT * FROM computer_executions WHERE id=? AND computer_id=?"),
                )
                .bind::<IdentitySql, _>(EntityId::from_dto(EntityKind::Execution, &old)?)
                .bind::<IdentitySql, _>(EntityId::from_dto(
                    EntityKind::Computer,
                    scope.computer_id,
                )?)
                .get_result::<Execution>(connection)
                .await?;
                if execution.retired_at.is_none() {
                    return Err(StoreError::InvalidTransition);
                }
            }
            let assigned = EntityId::generate(now)?;
            let assigned_dto = assigned.to_dto(EntityKind::Execution);
            diesel::sql_query(
                dialect.query(
                    "INSERT INTO computer_executions(id,computer_id,initial_allocation,created_at) VALUES(?,?,?,?)",
                ),
            )
            .bind::<IdentitySql, _>(assigned)
            .bind::<IdentitySql, _>(EntityId::from_dto(EntityKind::Computer,scope.computer_id)?)
            .bind::<Bool, _>(initial)
            .bind::<BigInt, _>(now)
            .execute(connection)
            .await?;
            for native in &reservations {
                let saved_role = native.kind().role() == ResourceRole::SavedEnvironment;
                if saved_role && saved.is_some() {
                    continue;
                }
                let execution = (!saved_role).then_some(assigned);
                let allocation_execution = (!saved_role).then_some(assigned_dto.as_str());
                diesel::sql_query(dialect.query("INSERT INTO computer_resources(computer_id,provider_id,resource_kind,native_scope_kind,native_scope,name,execution_id,presence,allocation_computer_id,allocation_provider_id,allocation_execution_id) VALUES(?,?,?,?,?,?,?,'UNKNOWN',?,?,?)"))
                    .bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::Computer,scope.computer_id)?).bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::Provider,source.provider_id)?)
                    .bind::<Text,_>(native.kind().as_str()).bind::<Text,_>(native.scope().kind_str()).bind::<Text,_>(native.scope().as_str()).bind::<Text,_>(native.name())
                    .bind::<Nullable<IdentitySql>,_>(execution).bind::<Text,_>(scope.computer_id).bind::<Text,_>(source.provider_id).bind::<Nullable<Text>,_>(allocation_execution).execute(connection).await?;
            }
            diesel::sql_query(dialect.query("UPDATE computers SET current_execution_id=?,current_phase='STARTING',observed_power='UNKNOWN',power_source_connection_id=NULL,power_source_execution_id=NULL,prepare_admitted_at=NULL,initial_start_dispatch_at=NULL WHERE id=?"))
                .bind::<IdentitySql,_>(assigned).bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::Computer,scope.computer_id)?).execute(connection).await?;
            Ok(ExecutionReservation {
                execution_id: assigned_dto,
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
        if report.execution_id.is_some_and(|v| !valid_id(v)) {
            return Err(StoreError::InvalidInput);
        }
        operation!(self, |connection, dialect| {
            let (provider, _) = current_source!(connection, dialect, source);
            let current = current_computer!(connection, dialect, source, scope);
            let now = db_now!(connection, dialect);
            let substrate = provider.substrate()?;
            if !substrate.accepts_scope(report.native.scope())
                || substrate.kind() != report.native.kind().provider_kind()
            {
                return Err(StoreError::Conflict);
            }
            if current.cleanup_completed_at.is_some() {
                return Err(StoreError::Conflict);
            }
            let resource: ManagedResource = diesel::sql_query(dialect.query("SELECT * FROM computer_resources WHERE computer_id=? AND resource_kind=? AND native_scope_kind=? AND native_scope=? AND name=?"))
                .bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::Computer,scope.computer_id)?).bind::<Text,_>(report.native.kind().as_str()).bind::<Text,_>(report.native.scope().kind_str()).bind::<Text,_>(report.native.scope().as_str()).bind::<Text,_>(report.native.name()).get_result::<ResourceRow>(connection).await.optional()?.ok_or(StoreError::NotFound)?.try_into()?;
            if resource.execution_id.as_deref() != report.execution_id {
                return Err(StoreError::Conflict);
            }
            let (presence, identity) = match report.evidence {
                ResourceEvidence::OwnedPresent { identity } => {
                    if !identity.matches_kind(report.native.kind()) {
                        return Err(StoreError::InvalidInput);
                    }
                    if resource
                        .identity
                        .as_ref()
                        .is_some_and(|old| old != identity)
                        || (resource.absence_confirmed_at.is_some() && resource.identity.is_some())
                    {
                        return Err(StoreError::Conflict);
                    }
                    if let Some(execution) = &resource.execution_id {
                        let state = diesel::sql_query(dialect.query(
                            "SELECT * FROM computer_executions WHERE id=? AND computer_id=?",
                        ))
                        .bind::<IdentitySql, _>(EntityId::from_dto(
                            EntityKind::Execution,
                            execution,
                        )?)
                        .bind::<IdentitySql, _>(EntityId::from_dto(
                            EntityKind::Computer,
                            scope.computer_id,
                        )?)
                        .get_result::<Execution>(connection)
                        .await?;
                        if state.retired_at.is_some() {
                            return Err(StoreError::InvalidTransition);
                        }
                    }
                    ("PRESENT", Some(identity))
                }
                ResourceEvidence::DefinitivelyAbsent { expected_identity } => {
                    if resource.identity.as_ref() != expected_identity {
                        return Err(StoreError::Conflict);
                    }
                    ("ABSENT", expected_identity)
                }
                ResourceEvidence::Unknown => ("UNKNOWN", resource.identity.as_ref()),
            };
            let columns = identity_columns(identity)?;
            diesel::sql_query(dialect.query("UPDATE computer_resources SET resource_uid=COALESCE(resource_uid,?),docker_container_id=COALESCE(docker_container_id,?),docker_volume_metadata=COALESCE(docker_volume_metadata,?),presence=?,observed_at=?,source_connection_id=?,absence_confirmed_at=CASE WHEN ? THEN NULL ELSE COALESCE(absence_confirmed_at,?) END WHERE computer_id=? AND resource_kind=? AND native_scope_kind=? AND native_scope=? AND name=?"))
                .bind::<Nullable<Text>,_>(columns.uid).bind::<Nullable<Text>,_>(columns.container_id).bind::<Nullable<Text>,_>(columns.volume_metadata)
                .bind::<Text,_>(presence).bind::<BigInt,_>(now).bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::Connection,source.connection_id)?)
                .bind::<Bool,_>(resource.identity.is_none() && presence=="PRESENT")
                .bind::<Nullable<BigInt>,_>(if presence == "ABSENT" { Some(now) } else { None })
                .bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::Computer,scope.computer_id)?).bind::<Text,_>(report.native.kind().as_str()).bind::<Text,_>(report.native.scope().kind_str()).bind::<Text,_>(report.native.scope().as_str()).bind::<Text,_>(report.native.name()).execute(connection).await?;
            if report.native.kind().role() == ResourceRole::SavedEnvironment {
                let saved = match presence {
                    "PRESENT" => "VERIFIED",
                    "ABSENT" => "MISSING",
                    _ => "UNKNOWN",
                };
                diesel::sql_query(dialect.query("UPDATE computers SET saved_environment=?,environment_source_connection_id=?,environment_observed_at=? WHERE id=?"))
                    .bind::<Text,_>(saved).bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::Connection,source.connection_id)?).bind::<BigInt,_>(now).bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::Computer,scope.computer_id)?).execute(connection).await?;
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
                || (matches!(current.current_action.as_str(), "STOP" | "RESTART")
                    && current.expected_old_execution_id.as_deref() == Some(execution_id))
            {
                return Err(StoreError::InvalidTransition);
            }
            let (provider, _) = current_source!(connection, dialect, source);
            let execution = diesel::sql_query(
                dialect.query("SELECT * FROM computer_executions WHERE id=? AND computer_id=?"),
            )
            .bind::<IdentitySql, _>(EntityId::from_dto(EntityKind::Execution, execution_id)?)
            .bind::<IdentitySql, _>(EntityId::from_dto(EntityKind::Computer, scope.computer_id)?)
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
                .bind::<BigInt,_>(now).bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::Computer,scope.computer_id)?).execute(connection).await?;
            Ok(())
        })
    }

    /// Provider asserts actual native running state, not mere object existence.
    pub async fn report_execution_running(
        &self,
        source: &ProviderSource<'_>,
        scope: ComputerRequest<'_>,
        execution_id: &str,
    ) -> Result<(), StoreError> {
        validate_source(source)?;
        validate_scope(scope)?;
        operation!(self, |connection, dialect| {
            let (provider, _) = current_source!(connection, dialect, source);
            let current = current_computer!(connection, dialect, source, scope);
            let now = db_now!(connection, dialect);
            if current.desired_state != "RUNNING"
                || current.current_phase == "RETIRING"
                || current.current_execution_id.as_deref() != Some(execution_id)
                || (matches!(current.current_action.as_str(), "STOP" | "RESTART")
                    && current.expected_old_execution_id.as_deref() == Some(execution_id))
            {
                return Err(StoreError::InvalidTransition);
            }
            if provider.substrate()?.kind() == chapchoo_domain::ProviderKind::Docker
                && current.current_phase == "UNCERTAIN"
                && current.expected_old_execution_id.as_deref() == Some(execution_id)
            {
                return Err(StoreError::InvalidTransition);
            }
            let resources = inventory!(connection, dialect, scope.computer_id);
            validate_native_inventory(&provider, &resources)?;
            for &kind in provider.substrate()?.kind().required_resource_kinds() {
                if !resources.iter().any(|r| {
                    r.kind() == kind
                        && (kind.role() == ResourceRole::SavedEnvironment
                            || r.execution_id.as_deref() == Some(execution_id))
                        && r.identity.is_some()
                        && r.presence == ResourcePresence::Present
                        && r.source_connection_id.as_deref() == Some(source.connection_id)
                }) {
                    return Err(StoreError::InvalidTransition);
                }
            }
            diesel::sql_query(dialect.query("UPDATE computer_executions SET running_confirmed_at=COALESCE(running_confirmed_at,?) WHERE id=? AND computer_id=?"))
                .bind::<BigInt,_>(now).bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::Execution,execution_id)?).bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::Computer,scope.computer_id)?).execute(connection).await?;
            diesel::sql_query(dialect.query("UPDATE computers SET current_phase='COMPLETED',observed_power='RUNNING',power_observed_at=?,power_source_connection_id=?,power_source_execution_id=? WHERE id=?"))
                .bind::<BigInt,_>(now).bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::Connection,source.connection_id)?).bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::Execution,execution_id)?).bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::Computer,scope.computer_id)?).execute(connection).await?;
            Ok(())
        })
    }

    /// Clears the exact current candidate only after definitive native retirement.
    /// Pending/unknown native effects cannot cross this boundary.
    pub async fn retire_execution(
        &self,
        source: &ProviderSource<'_>,
        scope: ComputerRequest<'_>,
        execution_id: &str,
    ) -> Result<(), StoreError> {
        validate_source(source)?;
        validate_scope(scope)?;
        operation!(self, |connection, dialect| {
            let (provider, _) = current_source!(connection, dialect, source);
            let current = current_computer!(connection, dialect, source, scope);
            let now = db_now!(connection, dialect);
            let execution_kind = provider.substrate()?.kind().execution_resource_kind();
            if current.current_execution_id.as_deref() != Some(execution_id) {
                if current.current_execution_id.is_none()
                    && current.expected_old_execution_id.as_deref() == Some(execution_id)
                {
                    let previous =
                        diesel::sql_query(dialect.query(
                            "SELECT * FROM computer_executions WHERE id=? AND computer_id=?",
                        ))
                        .bind::<IdentitySql, _>(EntityId::from_dto(
                            EntityKind::Execution,
                            execution_id,
                        )?)
                        .bind::<IdentitySql, _>(EntityId::from_dto(
                            EntityKind::Computer,
                            scope.computer_id,
                        )?)
                        .get_result::<Execution>(connection)
                        .await?;
                    if previous.retired_at.is_some() {
                        if current.cleanup_completed_at.is_some() {
                            return Ok(());
                        }
                        let resources = inventory!(connection, dialect, scope.computer_id);
                        validate_native_inventory(&provider, &resources)?;
                        if !resources.iter().any(|r| {
                            r.kind() == execution_kind
                                && r.execution_id.as_deref() == Some(execution_id)
                                && r.presence == ResourcePresence::Absent
                                && r.source_connection_id.as_deref() == Some(source.connection_id)
                        }) {
                            return Err(StoreError::InvalidTransition);
                        }
                        // Reconnect must freshly establish stopped power. This
                        // replay changes no lifecycle phase or dispatch interval.
                        diesel::sql_query(dialect.query("UPDATE computers SET observed_power='STOPPED',power_observed_at=?,power_source_connection_id=?,power_source_execution_id=NULL WHERE id=?"))
                            .bind::<BigInt,_>(now).bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::Connection,source.connection_id)?)
                            .bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::Computer,scope.computer_id)?).execute(connection).await?;
                        return Ok(());
                    }
                }
                return Err(StoreError::Conflict);
            }
            let resources = inventory!(connection, dialect, scope.computer_id);
            validate_native_inventory(&provider, &resources)?;
            if !resources.iter().any(|r| {
                r.kind() == execution_kind
                    && r.execution_id.as_deref() == Some(execution_id)
                    && r.presence == ResourcePresence::Absent
                    && r.source_connection_id.as_deref() == Some(source.connection_id)
            }) {
                return Err(StoreError::InvalidTransition);
            }
            revoke_execution!(connection, dialect, execution_id, now);
            let execution = diesel::sql_query(
                dialect.query("SELECT * FROM computer_executions WHERE id=? AND computer_id=?"),
            )
            .bind::<IdentitySql, _>(EntityId::from_dto(EntityKind::Execution, execution_id)?)
            .bind::<IdentitySql, _>(EntityId::from_dto(EntityKind::Computer, scope.computer_id)?)
            .get_result::<Execution>(connection)
            .await?;
            diesel::sql_query(dialect.query("UPDATE computer_executions SET retired_at=COALESCE(retired_at,?) WHERE id=? AND computer_id=?"))
                .bind::<BigInt,_>(now).bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::Execution,execution_id)?).bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::Computer,scope.computer_id)?).execute(connection).await?;
            let phase = match current.desired_state.as_str() {
                "DELETING" if current.current_phase == "UNCERTAIN" => "UNCERTAIN",
                "DELETING" => "DELETING",
                "STOPPED" => "COMPLETED",
                "RUNNING" => "STARTING",
                _ => return Err(StoreError::Integrity),
            };
            diesel::sql_query(dialect.query("UPDATE computers SET current_execution_id=NULL,expected_old_execution_id=?,current_phase=?,initial_start_dispatch_at=CASE WHEN ? THEN NULL ELSE initial_start_dispatch_at END,observed_power='STOPPED',power_observed_at=?,power_source_connection_id=?,power_source_execution_id=NULL WHERE id=?"))
                .bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::Execution,execution_id)?).bind::<Text,_>(phase)
                .bind::<Bool,_>(execution.running_confirmed_at.is_some() && current.desired_state=="RUNNING")
                .bind::<BigInt,_>(now).bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::Connection,source.connection_id)?).bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::Computer,scope.computer_id)?).execute(connection).await?;
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
            let (provider, _) = current_source!(connection, dialect, source);
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
                if current.current_phase == "UNCERTAIN"
                    && (current.desired_state == "DELETING"
                        || provider.substrate()?.kind() == chapchoo_domain::ProviderKind::Docker)
                {
                    "UNCERTAIN"
                } else {
                    phase.as_str()
                },
            )
            .bind::<IdentitySql, _>(EntityId::from_dto(EntityKind::Computer, scope.computer_id)?)
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
                    "SELECT * FROM computers WHERE provider_id=? AND (? IS NULL OR id>?) ORDER BY id LIMIT ?",
                ),
            )
            .bind::<IdentitySql, _>(EntityId::from_dto(EntityKind::Provider,source.provider_id)?)
            .bind::<Nullable<IdentitySql>, _>(after.map(|id|EntityId::from_dto(EntityKind::Computer,id)).transpose()?)
            .bind::<Nullable<IdentitySql>, _>(after.map(|id|EntityId::from_dto(EntityKind::Computer,id)).transpose()?)
            .bind::<BigInt, _>(limit)
            .load::<ManagedComputerRow>(connection)
            .await?.into_iter().map(ManagedComputer::from).collect())
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
            let (provider, _) = current_source!(connection, dialect, source);
            let _ = current_computer!(connection, dialect, source, scope);
            let resources = inventory!(connection, dialect, scope.computer_id);
            validate_native_inventory(&provider, &resources)?;
            Ok(resources)
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
            let (provider, _) = current_source!(connection, dialect, source);
            let current = current_computer!(connection, dialect, source, scope);
            let now = db_now!(connection, dialect);
            if current.desired_state != "DELETING" || current.current_execution_id.is_some() {
                return Err(StoreError::InvalidTransition);
            }
            if current.cleanup_completed_at.is_some() {
                return Ok(());
            }
            let live = diesel::sql_query(dialect.query("SELECT count(*) AS n FROM computer_executions WHERE computer_id=? AND retired_at IS NULL"))
                .bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::Computer,scope.computer_id)?).get_result::<Count>(connection).await?;
            if live.n != 0 {
                return Err(StoreError::InvalidTransition);
            }
            let resources = inventory!(connection, dialect, scope.computer_id);
            validate_native_inventory(&provider, &resources)?;
            if resources.iter().any(|r| {
                r.presence != ResourcePresence::Absent
                    || r.source_connection_id.as_deref() != Some(source.connection_id)
            }) {
                return Err(StoreError::InvalidTransition);
            }
            diesel::sql_query(dialect.query("UPDATE computers SET removed_agent_id=COALESCE(removed_agent_id,(SELECT id FROM agents WHERE computer_id=computers.id)),agent_removed_at=COALESCE(agent_removed_at,?),cleanup_completed_at=?,current_phase='COMPLETED' WHERE id=?"))
                .bind::<BigInt,_>(now).bind::<BigInt,_>(now).bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::Computer,scope.computer_id)?).execute(connection).await?;
            diesel::sql_query(dialect.query("DELETE FROM agents WHERE computer_id=?"))
                .bind::<IdentitySql, _>(EntityId::from_dto(
                    EntityKind::Computer,
                    scope.computer_id,
                )?)
                .execute(connection)
                .await?;
            Ok(())
        })?;
        self.collect_deleted_teams().await
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
        let collected = operation!(self, |connection, dialect| {
            let (_, now) = current_source!(connection, dialect, source);
            let threshold = now.checked_sub(86_400_000).ok_or(StoreError::Integrity)?;
            let candidates = diesel::sql_query(dialect.query("SELECT * FROM computers WHERE provider_id=? AND desired_state='DELETING' AND agent_removed_at IS NOT NULL AND cleanup_completed_at<=? AND current_execution_id IS NULL AND NOT EXISTS(SELECT 1 FROM computer_resources r WHERE r.computer_id=computers.id AND r.presence<>'ABSENT') AND NOT EXISTS(SELECT 1 FROM computer_executions e WHERE e.computer_id=computers.id AND e.retired_at IS NULL) ORDER BY id LIMIT ?"))
                .bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::Provider,source.provider_id)?).bind::<BigInt,_>(threshold).bind::<BigInt,_>(limit).load::<ManagedComputerRow>(connection).await?;
            let mut count = 0;
            for row in candidates {
                count += diesel::sql_query(
                    dialect.query("DELETE FROM computers WHERE id=? AND provider_id=?"),
                )
                .bind::<IdentitySql, _>(row.id)
                .bind::<IdentitySql, _>(EntityId::from_dto(
                    EntityKind::Provider,
                    source.provider_id,
                )?)
                .execute(connection)
                .await?;
            }
            Ok(count)
        })?;
        self.collect_deleted_teams().await?;
        Ok(collected)
    }
}
