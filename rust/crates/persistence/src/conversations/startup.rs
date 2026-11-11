//! Connection startup is a sealed conversation operation, not an accepted input.
//! Page reads must not call admission.

use super::authority::connector;
use super::{
    ConversationPreparationFacts, ConversationPreparationOutcome, ConversationPreparationState,
    ConversationRow, HarnessSupplySelection, conversation,
};
use crate::{
    Store, StoreError,
    conversations::{
        delivery::dispatch_route,
        preparation::{PreparationRow, startup_attempt_sql},
        supply::current_supply,
    },
    directory::database_now,
    entity_id::{EntityId, EntityKind, IdentitySql},
    store::{Dialect, operation},
};
use chapchoo_crypto::SecretDigest;
use chapchoo_domain::{HarnessSupply, NativeConversationId, SnapshotId};
use diesel::{
    OptionalExtension,
    sql_types::{BigInt, Bool, Nullable, Text},
};
use diesel_async::RunQueryDsl;

const PREPARATION_COLUMNS: &str = "execution_id,supply_kind,executable_path,selection_request_id,preparation_state,ready_snapshot,ready_native_session,failure_kind,fresh_native_allowed,require_retained_source";
const RUNNING_COMPUTER_PAGE: u32 = 100;

#[derive(Clone, PartialEq, Eq)]
pub enum ConnectionStartupContext {
    Initial,
    Existing {
        snapshot: SnapshotId,
        native_session: NativeConversationId,
    },
}
pub enum ConnectionStartupAdmission {
    AuthenticationRequired {
        agent_id: String,
        conversation_id: String,
    },
    Dispatch {
        agent_id: String,
        conversation_id: String,
        operation_id: String,
        computer_request_id: String,
        execution_id: String,
        requested_supply: HarnessSupply,
        fresh_native_allowed: bool,
        require_retained_source: bool,
        context: ConnectionStartupContext,
    },
    Settled {
        agent_id: String,
        conversation_id: String,
        facts: ConversationPreparationFacts,
    },
}
pub struct ConnectionStartupObservation {
    pub operation_id: String,
    pub facts: ConversationPreparationFacts,
}
impl std::fmt::Debug for ConnectionStartupAdmission {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("ConnectionStartupAdmission([REDACTED])")
    }
}
impl std::fmt::Debug for ConnectionStartupContext {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("ConnectionStartupContext([REDACTED])")
    }
}
impl std::fmt::Debug for ConnectionStartupObservation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("ConnectionStartupObservation([REDACTED])")
    }
}

#[derive(diesel::QueryableByName)]
struct AgentBinding {
    #[diesel(sql_type = IdentitySql)]
    agent_id: EntityId,
    #[diesel(sql_type = IdentitySql)]
    owner_user_id: EntityId,
    #[diesel(sql_type = IdentitySql)]
    team_id: EntityId,
}
#[derive(diesel::QueryableByName)]
struct TeamId {
    #[diesel(sql_type = IdentitySql)]
    team_id: EntityId,
}
#[derive(diesel::QueryableByName)]
struct LockedId {
    #[diesel(sql_type = IdentitySql)]
    id: EntityId,
}
#[derive(diesel::QueryableByName)]
struct ComputerGate {
    #[diesel(sql_type = Text)]
    desired_state: String,
    #[diesel(sql_type = Text)]
    current_request_id: String,
}
#[derive(diesel::QueryableByName)]
struct DispatchGate {
    #[diesel(sql_type = Text)]
    desired_state: String,
    #[diesel(sql_type = Nullable<IdentitySql>)]
    current_execution_id: Option<EntityId>,
}
#[derive(diesel::QueryableByName)]
struct CountRow {
    #[diesel(sql_type = BigInt)]
    n: i64,
}
#[derive(diesel::QueryableByName)]
struct OperationHead {
    #[diesel(sql_type = IdentitySql)]
    operation_id: EntityId,
    #[diesel(sql_type = IdentitySql)]
    execution_id: EntityId,
    #[diesel(sql_type = Nullable<IdentitySql>)]
    ready_connection_id: Option<EntityId>,
}
#[derive(diesel::QueryableByName)]
struct AuthRow {
    #[diesel(sql_type = IdentitySql, column_name = authentication_id)]
    _authentication_id: EntityId,
}
#[derive(diesel::QueryableByName)]
struct InputPreparationHead {
    #[diesel(sql_type = Text)]
    preparation_state: String,
    #[diesel(sql_type = Nullable<Text>)]
    ready_snapshot: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    ready_native_session: Option<String>,
}
#[derive(diesel::QueryableByName)]
pub(super) struct StartupIntent {
    #[diesel(sql_type = Nullable<Text>)]
    pub(super) intended_snapshot: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub(super) intended_native_session: Option<String>,
}

/// What the current startup row permits. Database writes stay with the caller.
#[derive(Clone, PartialEq, Eq, Debug)]
pub(super) enum StartupRecovery {
    Redispatch(ConnectionStartupContext),
    Settle,
    RetirePrevious,
    ConsiderNew,
    Wait,
}

pub(super) fn recover_startup(
    latest: Option<&ConversationPreparationFacts>,
    latest_is_current_execution: bool,
    ready_on_current_connection: bool,
    source_state: &str,
    snapshot: Option<&str>,
    native: Option<&str>,
    intended: Option<(&str, &str)>,
) -> Result<StartupRecovery, StoreError> {
    let Some(facts) = latest else {
        return Ok(StartupRecovery::ConsiderNew);
    };
    if matches!(
        facts.state,
        ConversationPreparationState::PossibleForwarding
    ) {
        if !latest_is_current_execution {
            return Ok(StartupRecovery::RetirePrevious);
        }
        return Ok(
            match sealed_context(facts, source_state, snapshot, native, intended)? {
                Some(context) => StartupRecovery::Redispatch(context),
                None => StartupRecovery::Wait,
            },
        );
    }
    if let ConversationPreparationState::Known(ConversationPreparationOutcome::SessionReady(target)) =
        &facts.state
        && source_state == "READY"
    {
        let matches_source = snapshot == Some(target.snapshot.as_str())
            && native == Some(target.native_session.as_str());
        if !matches_source {
            return Err(StoreError::Integrity);
        }
        if latest_is_current_execution && ready_on_current_connection {
            return Ok(StartupRecovery::Settle);
        }
        return Ok(StartupRecovery::ConsiderNew);
    }
    if let ConversationPreparationState::Known(ConversationPreparationOutcome::Failed(failure)) =
        &facts.state
        && !retryable(failure)
        && grant_context(source_state, snapshot, native, false)?.is_none()
    {
        return Ok(StartupRecovery::Settle);
    }
    Ok(StartupRecovery::ConsiderNew)
}

fn retryable(failure: &super::ConversationPreparationFailure) -> bool {
    matches!(
        failure,
        super::ConversationPreparationFailure::NotAdmitted
            | super::ConversationPreparationFailure::BeforeNativeLaunch
    )
}

/// Redispatch uses the sealed intent. A later source observation must not turn
/// an initial grant into a different resume target, or a retained grant into
/// whatever source is current now.
fn sealed_context(
    facts: &ConversationPreparationFacts,
    source_state: &str,
    snapshot: Option<&str>,
    native: Option<&str>,
    intended: Option<(&str, &str)>,
) -> Result<Option<ConnectionStartupContext>, StoreError> {
    if facts.fresh_native_allowed {
        if intended.is_some() {
            return Err(StoreError::Integrity);
        }
        return Ok(Some(ConnectionStartupContext::Initial));
    }
    let Some((intended_snapshot, intended_native)) = intended else {
        return Err(StoreError::Integrity);
    };
    let matches = snapshot == Some(intended_snapshot) && native == Some(intended_native);
    if !matches {
        return Err(StoreError::Integrity);
    }
    if source_state == "READY" {
        return Ok(Some(existing_context(
            Some(intended_snapshot),
            Some(intended_native),
        )?));
    }
    // LOADING with the sealed identity is the same operation in progress.
    Ok(None)
}

fn grant_context(
    source_state: &str,
    snapshot: Option<&str>,
    native: Option<&str>,
    fresh: bool,
) -> Result<Option<ConnectionStartupContext>, StoreError> {
    match (source_state, snapshot, native, fresh) {
        ("NOT_STARTED", None, None, true) => Ok(Some(ConnectionStartupContext::Initial)),
        ("READY", Some(snapshot), Some(native), _) => {
            Ok(Some(existing_context(Some(snapshot), Some(native))?))
        }
        _ => Ok(None),
    }
}

fn existing_context(
    snapshot: Option<&str>,
    native: Option<&str>,
) -> Result<ConnectionStartupContext, StoreError> {
    let (Some(snapshot), Some(native)) = (snapshot, native) else {
        return Err(StoreError::Integrity);
    };
    Ok(ConnectionStartupContext::Existing {
        snapshot: SnapshotId::parse(snapshot.to_owned()).map_err(|_| StoreError::Integrity)?,
        native_session: NativeConversationId::parse(native.to_owned())
            .map_err(|_| StoreError::Integrity)?,
    })
}

pub(super) fn startup_covers_input(
    facts: &ConversationPreparationFacts,
    latest_is_current_execution: bool,
    ready_on_current_connection: bool,
    source_state: &str,
    snapshot: Option<&str>,
    native: Option<&str>,
    input_target_snapshot: Option<&str>,
) -> bool {
    if !latest_is_current_execution {
        return false;
    }
    if input_target_snapshot.is_some_and(|target| snapshot != Some(target)) {
        return false;
    }
    match &facts.state {
        ConversationPreparationState::PossibleForwarding => true,
        ConversationPreparationState::Known(ConversationPreparationOutcome::SessionReady(
            target,
        )) => {
            ready_on_current_connection
                && source_state == "READY"
                && snapshot == Some(target.snapshot.as_str())
                && native == Some(target.native_session.as_str())
        }
        ConversationPreparationState::Known(ConversationPreparationOutcome::Failed(_)) => false,
    }
}

macro_rules! latest_startup_head {
    ($connection:expr, $dialect:expr, $conversation:expr) => {
        diesel::sql_query($dialect.query(
            "SELECT operation_id,execution_id,ready_connection_id FROM conversation_startups WHERE conversation_id=? ORDER BY admitted_at DESC, operation_id DESC LIMIT 1",
        ))
        .bind::<IdentitySql, _>($conversation)
        .get_result::<OperationHead>($connection)
        .await
        .optional()?
    };
}

macro_rules! startup_facts {
    ($connection:expr, $dialect:expr, $operation:expr) => {
        diesel::sql_query($dialect.query(&format!(
            "SELECT {PREPARATION_COLUMNS} FROM conversation_startups WHERE operation_id=?"
        )))
        .bind::<IdentitySql, _>($operation)
        .get_result::<PreparationRow>($connection)
        .await?
        .facts()?
    };
}

macro_rules! admit_agent {
    ($connection:expr, $dialect:expr, $agent:expr, $execution:expr, $request:expr, $now:expr, $connection_id:expr) => {{
        let agent = $agent;
        let execution_id = $execution;
        let request_id = $request;
        let admitted_at = $now;
        let connection_id = $connection_id;
        'agent: {
            let current = diesel::sql_query($dialect.query(
                "SELECT c.* FROM conversations c JOIN agent_conversations a ON a.conversation_id=c.id AND a.agent_id=c.agent_id WHERE a.agent_id=?",
            ))
            .bind::<IdentitySql, _>(agent.agent_id)
            .get_result::<super::ConversationRow>($connection)
            .await
            .optional()?;
            let row = match current {
                Some(row) => row,
                None => {
                    let id = EntityId::generate(admitted_at)?;
                    diesel::sql_query($dialect.query(
                        "INSERT INTO conversations(id,agent_id,source_state,created_at) VALUES(?,?,'NOT_STARTED',?)",
                    ))
                    .bind::<IdentitySql, _>(id)
                    .bind::<IdentitySql, _>(agent.agent_id)
                    .bind::<BigInt, _>(admitted_at)
                    .execute($connection)
                    .await?;
                    diesel::sql_query($dialect.query(
                        "INSERT INTO agent_conversations(agent_id,conversation_id) VALUES(?,?)",
                    ))
                    .bind::<IdentitySql, _>(agent.agent_id)
                    .bind::<IdentitySql, _>(id)
                    .execute($connection)
                    .await?;
                    conversation!($connection, $dialect, id)
                }
            };
            if row.agent_id != agent.agent_id {
                break 'agent Err(StoreError::Integrity);
            }
            let saved = diesel::sql_query($dialect.query(
                "SELECT s.authentication_id FROM agent_authentication_selections s JOIN external_authentications a ON a.id=s.authentication_id WHERE s.agent_id=? AND (a.owner_user_id=? OR a.owner_team_id=?)",
            ))
            .bind::<IdentitySql, _>(agent.agent_id)
            .bind::<IdentitySql, _>(agent.owner_user_id)
            .bind::<IdentitySql, _>(agent.team_id)
            .load::<AuthRow>($connection)
            .await?;
            if saved.len() > 1 {
                break 'agent Err(StoreError::Integrity);
            }
            let agent_id = agent.agent_id.to_dto(EntityKind::Agent);
            let conversation_id = row.id.to_dto(EntityKind::Conversation);
            if saved.is_empty() {
                break 'agent Ok(Some(ConnectionStartupAdmission::AuthenticationRequired {
                    agent_id,
                    conversation_id,
                }));
            }
            let mut retired = false;
            let recovered = loop {
                let Some(head) = latest_startup_head!($connection, $dialect, row.id) else {
                    break None;
                };
                let facts = startup_facts!($connection, $dialect, head.operation_id);
                let intent = diesel::sql_query($dialect.query(
                    "SELECT intended_snapshot,intended_native_session FROM conversation_startups WHERE operation_id=?",
                ))
                .bind::<IdentitySql, _>(head.operation_id)
                .get_result::<StartupIntent>($connection)
                .await?;
                let intended = match (
                    intent.intended_snapshot.as_deref(),
                    intent.intended_native_session.as_deref(),
                ) {
                    (Some(snapshot), Some(native)) => Some((snapshot, native)),
                    (None, None) => None,
                    _ => break 'agent Err(StoreError::Integrity),
                };
                let action = recover_startup(
                    Some(&facts),
                    head.execution_id == *execution_id,
                    head.ready_connection_id == Some(connection_id),
                    &row.source_state,
                    row.source_snapshot.as_deref(),
                    row.native_session_id.as_deref(),
                    intended,
                )?;
                if !matches!(action, StartupRecovery::RetirePrevious) {
                    break Some((head, facts, action));
                }
                if retired {
                    break 'agent Err(StoreError::Conflict);
                }
                let changed = diesel::sql_query($dialect.query(
                    "UPDATE conversation_startups SET preparation_state='FAILED',failure_kind='RETIREMENT_UNKNOWN' WHERE operation_id=? AND preparation_state='POSSIBLE'",
                ))
                .bind::<IdentitySql, _>(head.operation_id)
                .execute($connection)
                .await?;
                if changed != 1 {
                    break 'agent Err(StoreError::Conflict);
                }
                retired = true;
            };
            if let Some((head, facts, action)) = recovered {
                match action {
                    StartupRecovery::Settle => {
                        break 'agent Ok(Some(ConnectionStartupAdmission::Settled {
                            agent_id,
                            conversation_id,
                            facts,
                        }));
                    }
                    StartupRecovery::Redispatch(context) => {
                        break 'agent Ok(Some(dispatch_admission(
                            agent_id,
                            conversation_id,
                            head.operation_id,
                            request_id,
                            facts,
                            context,
                        )));
                    }
                    StartupRecovery::Wait => break 'agent Ok(None),
                    StartupRecovery::RetirePrevious => break 'agent Err(StoreError::Conflict),
                    StartupRecovery::ConsiderNew => {}
                }
            }
            let input_preparation = diesel::sql_query($dialect.query(
                "SELECT p.preparation_state,p.ready_snapshot,p.ready_native_session FROM conversation_input_preparation p JOIN conversation_inputs i ON i.id=p.operation_id WHERE i.conversation_id=? AND p.execution_id=? AND p.preparation_state IN ('POSSIBLE','READY')",
            ))
            .bind::<IdentitySql, _>(row.id)
            .bind::<IdentitySql, _>(*execution_id)
            .load::<InputPreparationHead>($connection)
            .await?;
            if input_preparation
                .iter()
                .any(|row| row.preparation_state == "POSSIBLE")
            {
                break 'agent Ok(None);
            }
            if input_preparation.iter().any(|preparation| preparation.preparation_state == "READY") {
                let matches_source = input_preparation.iter().any(|preparation| {
                    preparation.preparation_state == "READY"
                        && row.source_state == "READY"
                        && row.source_snapshot.as_deref() == preparation.ready_snapshot.as_deref()
                        && row.native_session_id.as_deref()
                            == preparation.ready_native_session.as_deref()
                });
                if !matches_source {
                    break 'agent Err(StoreError::Integrity);
                }
            }
            let prior = diesel::sql_query($dialect.query(&startup_attempt_sql(
                "NOT (preparation_state='FAILED' AND (failure_kind IN ('NOT_ADMITTED','BEFORE_NATIVE_LAUNCH','NATIVE_REJECTED') OR (failure_kind='SOURCE_UNAVAILABLE' AND fresh_native_allowed=FALSE)))",
            )))
            .bind::<IdentitySql, _>(row.id)
            .bind::<IdentitySql, _>(row.id)
            .get_result::<CountRow>($connection)
            .await?;
            let rejected = diesel::sql_query($dialect.query(&startup_attempt_sql(
                "preparation_state='FAILED' AND failure_kind='NATIVE_REJECTED'",
            )))
            .bind::<IdentitySql, _>(row.id)
            .bind::<IdentitySql, _>(row.id)
            .get_result::<CountRow>($connection)
            .await?;
            let fresh_native_allowed = row.source_snapshot.is_none()
                && row.native_session_id.is_none()
                && prior.n == 0;
            let require_retained_source = rejected.n > 0;
            let Some(context) = grant_context(
                &row.source_state,
                row.source_snapshot.as_deref(),
                row.native_session_id.as_deref(),
                fresh_native_allowed,
            )?
            else {
                break 'agent Ok(None);
            };
            let selection = current_supply!($connection, $dialect, agent.agent_id);
            let selection_request_id = selection.saved.map(|receipt| receipt.request_id);
            let (kind, path) = match &selection.supply {
                HarnessSupply::Managed => ("MANAGED", None),
                HarnessSupply::Existing(path) => ("EXISTING", Some(path.as_str())),
            };
            let operation_id = EntityId::generate(admitted_at)?;
            let (intended_snapshot, intended_native) = match &context {
                ConnectionStartupContext::Initial => (None, None),
                ConnectionStartupContext::Existing {
                    snapshot,
                    native_session,
                } => (Some(snapshot.as_str()), Some(native_session.as_str())),
            };
            diesel::sql_query($dialect.query(
                "INSERT INTO conversation_startups(operation_id,conversation_id,execution_id,supply_kind,executable_path,selection_request_id,fresh_native_allowed,require_retained_source,intended_snapshot,intended_native_session,preparation_state,admitted_at) VALUES(?,?,?,?,?,?,?,?,?,?,'POSSIBLE',?)",
            ))
            .bind::<IdentitySql, _>(operation_id)
            .bind::<IdentitySql, _>(row.id)
            .bind::<IdentitySql, _>(*execution_id)
            .bind::<Text, _>(kind)
            .bind::<Nullable<Text>, _>(path)
            .bind::<Nullable<Text>, _>(selection_request_id.as_deref())
            .bind::<Bool, _>(fresh_native_allowed)
            .bind::<Bool, _>(require_retained_source)
            .bind::<Nullable<Text>, _>(intended_snapshot)
            .bind::<Nullable<Text>, _>(intended_native)
            .bind::<BigInt, _>(admitted_at)
            .execute($connection)
            .await?;
            break 'agent Ok(Some(ConnectionStartupAdmission::Dispatch {
                agent_id,
                conversation_id,
                operation_id: operation_id.to_dto(EntityKind::ConversationOperation),
                computer_request_id: request_id.to_owned(),
                execution_id: execution_id.to_dto(EntityKind::Execution),
                requested_supply: selection.supply,
                fresh_native_allowed,
                require_retained_source,
                context,
            }));
        }
    }};
}

impl Store {
    /// One bounded page of desired-running Computers. Route and authentication
    /// are rechecked by admission; this read does not start a harness.
    pub async fn running_computer_ids(
        &self,
        after: Option<&str>,
        limit: u32,
    ) -> Result<Vec<String>, StoreError> {
        let after = after
            .map(|id| EntityId::from_dto(EntityKind::Computer, id))
            .transpose()?;
        let limit = i64::from(limit.clamp(1, RUNNING_COMPUTER_PAGE));
        operation!(self, |connection, dialect| {
            let rows = if let Some(after) = after {
                diesel::sql_query(dialect.query(
                    "SELECT id FROM computers WHERE desired_state='RUNNING' AND current_execution_id IS NOT NULL AND id>? ORDER BY id LIMIT ?",
                ))
                .bind::<IdentitySql, _>(after)
                .bind::<BigInt, _>(limit)
                .load::<LockedId>(connection)
                .await?
            } else {
                diesel::sql_query(dialect.query(
                    "SELECT id FROM computers WHERE desired_state='RUNNING' AND current_execution_id IS NOT NULL ORDER BY id LIMIT ?",
                ))
                .bind::<BigInt, _>(limit)
                .load::<LockedId>(connection)
                .await?
            };
            Ok(rows
                .into_iter()
                .map(|row| row.id.to_dto(EntityKind::Computer))
                .collect())
        })
    }

    /// Admit startup for every eligible Agent on the current route.
    /// An empty result means this Computer is not currently admissible.
    /// This is not a user or Connector RPC.
    pub async fn admit_connection_startup(
        &self,
        computer_id: &str,
        route: &crate::ResolvedRoute,
    ) -> Result<Vec<ConnectionStartupAdmission>, StoreError> {
        let computer = EntityId::from_dto(EntityKind::Computer, computer_id)?;
        operation!(self, |connection, dialect| {
            let teams =
                diesel::sql_query(dialect.query(
                    "SELECT DISTINCT team_id FROM agents WHERE computer_id=? ORDER BY team_id",
                ))
                .bind::<IdentitySql, _>(computer)
                .load::<TeamId>(connection)
                .await?;
            let team_lock = match dialect {
                Dialect::Postgres => "SELECT id FROM teams WHERE id=? FOR NO KEY UPDATE",
                Dialect::Sqlite => "SELECT id FROM teams WHERE id=?",
            };
            for team in &teams {
                diesel::sql_query(dialect.query(team_lock))
                    .bind::<IdentitySql, _>(team.team_id)
                    .get_result::<LockedId>(connection)
                    .await
                    .optional()?;
            }
            let connected = diesel::sql_query(dialect.query(
                "SELECT session_id AS id FROM connector_connections WHERE computer_id=? AND control_id=? AND connection_id=?",
            ))
            .bind::<IdentitySql, _>(computer)
            .bind::<IdentitySql, _>(EntityId::from_dto(
                EntityKind::ControlInstance,
                &route.component.control_id,
            )?)
            .bind::<IdentitySql, _>(EntityId::from_dto(
                EntityKind::Connection,
                &route.component.connection_id,
            )?)
            .get_result::<LockedId>(connection)
            .await
            .optional()?;
            if connected.is_none() {
                return Ok(Vec::new());
            }
            let (execution, _session) = dispatch_route!(connection, dialect, route, computer);
            let gate = diesel::sql_query(
                dialect.query("SELECT desired_state,current_request_id FROM computers WHERE id=?"),
            )
            .bind::<IdentitySql, _>(computer)
            .get_result::<ComputerGate>(connection)
            .await
            .optional()?;
            let Some(gate) = gate else {
                return Ok(Vec::new());
            };
            if gate.desired_state != "RUNNING" {
                return Ok(Vec::new());
            }
            let agents = diesel::sql_query(dialect.query(
                "SELECT a.id AS agent_id,a.owner_user_id,a.team_id FROM agents a JOIN teams t ON t.id=a.team_id AND t.deletion_requested_at IS NULL JOIN memberships m ON m.team_id=a.team_id AND m.user_id=a.owner_user_id WHERE a.computer_id=? ORDER BY a.id",
            ))
            .bind::<IdentitySql, _>(computer)
            .load::<AgentBinding>(connection)
            .await?;
            if agents
                .iter()
                .any(|agent| !teams.iter().any(|team| team.team_id == agent.team_id))
            {
                return Err(StoreError::Conflict);
            }
            let now = database_now!(connection, dialect);
            let mut admissions = Vec::new();
            for agent in &agents {
                if let Some(admission) = admit_agent!(
                    connection,
                    dialect,
                    agent,
                    &execution._execution_id,
                    &gate.current_request_id,
                    now,
                    EntityId::from_dto(EntityKind::Connection, &route.component.connection_id)?
                )? {
                    admissions.push(admission);
                }
            }
            Ok(admissions)
        })
    }

    /// Owner projection of the newest connection startup. Absence is not a failure.
    pub async fn connection_startup(
        &self,
        actor: &chapchoo_crypto::SecretDigest,
        agent_id: &str,
        now: i64,
    ) -> Result<Option<ConnectionStartupObservation>, StoreError> {
        let agent = EntityId::from_dto(EntityKind::Agent, agent_id)?;
        operation!(self, |connection, dialect| {
            let _ = super::authority::owned_agent!(connection, dialect, actor, agent, now);
            let current = diesel::sql_query(dialect.query(
                "SELECT c.id FROM conversations c JOIN agent_conversations a ON a.conversation_id=c.id AND a.agent_id=c.agent_id WHERE a.agent_id=?",
            ))
            .bind::<IdentitySql, _>(agent)
            .get_result::<LockedId>(connection)
            .await
            .optional()?;
            let Some(current) = current else {
                return Ok(None);
            };
            let Some(head) = latest_startup_head!(connection, dialect, current.id) else {
                return Ok(None);
            };
            let facts = startup_facts!(connection, dialect, head.operation_id);
            Ok(Some(ConnectionStartupObservation {
                operation_id: head.operation_id.to_dto(EntityKind::ConversationOperation),
                facts,
            }))
        })
    }

    /// Recheck the route and desired running state before an effect.
    /// Withdrawal holds the same grant: it cannot prove an earlier dispatch
    /// did not reach the native owner.
    pub async fn connection_startup_may_dispatch(
        &self,
        computer_id: &str,
        route: &crate::ResolvedRoute,
        operation_id: &str,
    ) -> Result<bool, StoreError> {
        let computer = EntityId::from_dto(EntityKind::Computer, computer_id)?;
        let operation = EntityId::from_dto(EntityKind::ConversationOperation, operation_id)?;
        operation!(self, |connection, dialect| {
            let gate_sql = match dialect {
                Dialect::Postgres => {
                    "SELECT desired_state,current_execution_id FROM computers WHERE id=? FOR NO KEY UPDATE"
                }
                Dialect::Sqlite => {
                    "SELECT desired_state,current_execution_id FROM computers WHERE id=?"
                }
            };
            let gate = diesel::sql_query(dialect.query(gate_sql))
                .bind::<IdentitySql, _>(computer)
                .get_result::<DispatchGate>(connection)
                .await
                .optional()?;
            let Some(gate) = gate else {
                return Ok(false);
            };
            let owned = diesel::sql_query(dialect.query(
                "SELECT s.execution_id AS id FROM conversation_startups s JOIN conversations c ON c.id=s.conversation_id JOIN agents a ON a.id=c.agent_id WHERE s.operation_id=? AND a.computer_id=? AND s.preparation_state='POSSIBLE'",
            ))
            .bind::<IdentitySql, _>(operation)
            .bind::<IdentitySql, _>(computer)
            .get_result::<LockedId>(connection)
            .await
            .optional()?;
            let Some(owned) = owned else {
                return Ok(false);
            };
            if gate.desired_state != "RUNNING" {
                return Ok(false);
            }
            let connected = diesel::sql_query(dialect.query(
                "SELECT session_id AS id FROM connector_connections WHERE computer_id=? AND control_id=? AND connection_id=?",
            ))
            .bind::<IdentitySql, _>(computer)
            .bind::<IdentitySql, _>(EntityId::from_dto(
                EntityKind::ControlInstance,
                &route.component.control_id,
            )?)
            .bind::<IdentitySql, _>(EntityId::from_dto(
                EntityKind::Connection,
                &route.component.connection_id,
            )?)
            .get_result::<LockedId>(connection)
            .await
            .optional()?;
            if connected.is_none() {
                return Ok(false);
            }
            let (execution, _session) = dispatch_route!(connection, dialect, route, computer);
            Ok(owned.id == execution._execution_id
                && gate.current_execution_id == Some(execution._execution_id))
        })
    }
}

fn dispatch_admission(
    agent_id: String,
    conversation_id: String,
    operation_id: EntityId,
    computer_request_id: &str,
    facts: ConversationPreparationFacts,
    context: ConnectionStartupContext,
) -> ConnectionStartupAdmission {
    ConnectionStartupAdmission::Dispatch {
        agent_id,
        conversation_id,
        operation_id: operation_id.to_dto(EntityKind::ConversationOperation),
        computer_request_id: computer_request_id.to_owned(),
        execution_id: facts.execution_id.clone(),
        requested_supply: facts.requested_supply.clone(),
        fresh_native_allowed: facts.fresh_native_allowed,
        require_retained_source: facts.require_retained_source,
        context,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conversations::{
        ConversationDeliveryTarget, ConversationPreparationFailure, ConversationPreparationOutcome,
        ConversationPreparationState,
    };

    fn facts(state: ConversationPreparationState, fresh: bool) -> ConversationPreparationFacts {
        ConversationPreparationFacts {
            execution_id: "current".into(),
            requested_supply: HarnessSupply::Managed,
            selection_request_id: None,
            fresh_native_allowed: fresh,
            require_retained_source: false,
            state,
        }
    }

    #[test]
    fn current_initial_grant_redispatches_without_reading_a_moved_source()
    -> Result<(), Box<dyn std::error::Error>> {
        let latest = facts(ConversationPreparationState::PossibleForwarding, true);
        let recovery = recover_startup(
            Some(&latest),
            true,
            true,
            "LOADING",
            Some("snap"),
            Some("native"),
            None,
        )?;
        assert_eq!(
            recovery,
            StartupRecovery::Redispatch(ConnectionStartupContext::Initial)
        );
        Ok(())
    }

    #[test]
    fn previous_execution_possible_retires_without_a_fresh_grant()
    -> Result<(), Box<dyn std::error::Error>> {
        let latest = facts(ConversationPreparationState::PossibleForwarding, true);
        let recovery =
            recover_startup(Some(&latest), false, false, "NOT_STARTED", None, None, None)?;
        assert_eq!(recovery, StartupRecovery::RetirePrevious);
        Ok(())
    }

    #[test]
    fn current_ready_startup_does_not_dispatch_again() -> Result<(), Box<dyn std::error::Error>> {
        let latest = facts(
            ConversationPreparationState::Known(ConversationPreparationOutcome::SessionReady(
                ConversationDeliveryTarget {
                    snapshot: SnapshotId::parse("snap".into())?,
                    native_session: NativeConversationId::parse("native".into())?,
                },
            )),
            false,
        );
        let recovery = recover_startup(
            Some(&latest),
            true,
            true,
            "READY",
            Some("snap"),
            Some("native"),
            None,
        )?;
        assert_eq!(recovery, StartupRecovery::Settle);
        Ok(())
    }

    #[test]
    fn ready_source_on_a_new_execution_can_be_resumed() -> Result<(), Box<dyn std::error::Error>> {
        let latest = facts(
            ConversationPreparationState::Known(ConversationPreparationOutcome::SessionReady(
                ConversationDeliveryTarget {
                    snapshot: SnapshotId::parse("snap".into())?,
                    native_session: NativeConversationId::parse("native".into())?,
                },
            )),
            false,
        );
        let recovery = recover_startup(
            Some(&latest),
            false,
            false,
            "READY",
            Some("snap"),
            Some("native"),
            None,
        )?;
        assert_eq!(recovery, StartupRecovery::ConsiderNew);
        Ok(())
    }

    #[test]
    fn terminal_failure_without_a_retained_source_stays_settled()
    -> Result<(), Box<dyn std::error::Error>> {
        let latest = facts(
            ConversationPreparationState::Known(ConversationPreparationOutcome::Failed(
                ConversationPreparationFailure::RetirementUnknown,
            )),
            true,
        );
        let recovery =
            recover_startup(Some(&latest), false, false, "NOT_STARTED", None, None, None)?;
        assert_eq!(recovery, StartupRecovery::Settle);
        Ok(())
    }

    #[test]
    fn retained_redispatch_uses_the_sealed_target() -> Result<(), Box<dyn std::error::Error>> {
        let latest = facts(ConversationPreparationState::PossibleForwarding, false);
        let recovery = recover_startup(
            Some(&latest),
            true,
            true,
            "READY",
            Some("sealed"),
            Some("native"),
            Some(("sealed", "native")),
        )?;
        assert_eq!(
            recovery,
            StartupRecovery::Redispatch(ConnectionStartupContext::Existing {
                snapshot: SnapshotId::parse("sealed".into())?,
                native_session: NativeConversationId::parse("native".into())?,
            })
        );
        Ok(())
    }

    #[test]
    fn retained_redispatch_rejects_a_moved_source() {
        let latest = facts(ConversationPreparationState::PossibleForwarding, false);
        let recovery = recover_startup(
            Some(&latest),
            true,
            true,
            "READY",
            Some("moved"),
            Some("other"),
            Some(("sealed", "native")),
        );
        assert!(matches!(recovery, Err(StoreError::Integrity)));
    }
}
