//! Setup uncertainty exists before a source is READY and is not input delivery.
use super::*;

pub struct ConversationInputPreparation<'a> {
    pub agent_id: &'a str,
    pub conversation_id: &'a str,
    pub operation_id: &'a str,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversationPreparationFailure {
    NotAdmitted,
    /// Explicit failure before native launch, not a generic transport/session error.
    BeforeNativeLaunch,
    NativeRejected,
    SourceUnavailable,
    RetirementUnknown,
}
impl ConversationPreparationFailure {
    fn database(self) -> &'static str {
        match self {
            Self::NotAdmitted => "NOT_ADMITTED",
            Self::BeforeNativeLaunch => "BEFORE_NATIVE_LAUNCH",
            Self::NativeRejected => "NATIVE_REJECTED",
            Self::SourceUnavailable => "SOURCE_UNAVAILABLE",
            Self::RetirementUnknown => "RETIREMENT_UNKNOWN",
        }
    }
    fn parse(value: &str) -> Result<Self, StoreError> {
        match value {
            "NOT_ADMITTED" => Ok(Self::NotAdmitted),
            "BEFORE_NATIVE_LAUNCH" => Ok(Self::BeforeNativeLaunch),
            "NATIVE_REJECTED" => Ok(Self::NativeRejected),
            "SOURCE_UNAVAILABLE" => Ok(Self::SourceUnavailable),
            "RETIREMENT_UNKNOWN" => Ok(Self::RetirementUnknown),
            _ => Err(StoreError::Integrity),
        }
    }
}
#[derive(Clone, PartialEq, Eq)]
pub enum ConversationPreparationOutcome {
    /// A setup result, not current liveness, captured-source completion or input consumption.
    SessionReady(ConversationDeliveryTarget),
    Failed(ConversationPreparationFailure),
}
#[derive(Clone, PartialEq, Eq)]
pub enum ConversationPreparationState {
    PossibleForwarding,
    Known(ConversationPreparationOutcome),
}
pub struct ConversationPreparationFacts {
    pub execution_id: String,
    pub requested_supply: HarnessSupply,
    pub selection_request_id: Option<String>,
    pub fresh_native_allowed: bool,
    pub state: ConversationPreparationState,
}
pub enum ConversationPreparationAdmission {
    Granted {
        grant: ConversationForwardingGrant,
        requested_supply: HarnessSupply,
        selection_request_id: Option<String>,
        fresh_native_allowed: bool,
    },
    Existing(ConversationPreparationFacts),
}
macro_rules! safe_debug {($($name:ty),+)=>{$(impl fmt::Debug for $name {
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result {f.write_str(concat!(stringify!($name),"([REDACTED])"))}
})+};}
safe_debug!(
    ConversationInputPreparation<'_>,
    ConversationPreparationOutcome,
    ConversationPreparationState,
    ConversationPreparationFacts,
    ConversationPreparationAdmission
);

#[derive(diesel::QueryableByName)]
struct PreparationRow {
    #[diesel(sql_type=IdentitySql)]
    execution_id: EntityId,
    #[diesel(sql_type=Text)]
    supply_kind: String,
    #[diesel(sql_type=Nullable<Text>)]
    executable_path: Option<String>,
    #[diesel(sql_type=Nullable<Text>)]
    selection_request_id: Option<String>,
    #[diesel(sql_type=Text)]
    preparation_state: String,
    #[diesel(sql_type=Nullable<Text>)]
    ready_snapshot: Option<String>,
    #[diesel(sql_type=Nullable<Text>)]
    ready_native_session: Option<String>,
    #[diesel(sql_type=Nullable<Text>)]
    failure_kind: Option<String>,
    #[diesel(sql_type=diesel::sql_types::Bool)]
    fresh_native_allowed: bool,
}
impl PreparationRow {
    fn state(&self) -> Result<ConversationPreparationState, StoreError> {
        use ConversationPreparationOutcome as Outcome;
        use ConversationPreparationState as State;
        Ok(
            match (
                self.preparation_state.as_str(),
                &self.ready_snapshot,
                &self.ready_native_session,
                &self.failure_kind,
            ) {
                ("POSSIBLE", None, None, None) => State::PossibleForwarding,
                ("READY", Some(snapshot), Some(native), None) => {
                    State::Known(Outcome::SessionReady(ConversationDeliveryTarget {
                        snapshot: SnapshotId::parse(snapshot.clone())
                            .map_err(|_| StoreError::Integrity)?,
                        native_session: NativeConversationId::parse(native.clone())
                            .map_err(|_| StoreError::Integrity)?,
                    }))
                }
                ("FAILED", None, None, Some(kind)) => State::Known(Outcome::Failed(
                    ConversationPreparationFailure::parse(kind)?,
                )),
                _ => return Err(StoreError::Integrity),
            },
        )
    }
    fn facts(&self) -> Result<ConversationPreparationFacts, StoreError> {
        let supply = match (self.supply_kind.as_str(), &self.executable_path) {
            ("MANAGED", None) => HarnessSupply::Managed,
            ("EXISTING", Some(path)) => HarnessSupply::Existing(
                ExistingCodexPath::parse(path.clone()).map_err(|_| StoreError::Integrity)?,
            ),
            _ => return Err(StoreError::Integrity),
        };
        Ok(ConversationPreparationFacts {
            execution_id: self.execution_id.to_dto(EntityKind::Execution),
            requested_supply: supply,
            selection_request_id: self.selection_request_id.clone(),
            fresh_native_allowed: self.fresh_native_allowed,
            state: self.state()?,
        })
    }
}
#[derive(diesel::QueryableByName)]
struct InputTarget {
    #[diesel(sql_type=Nullable<Text>)]
    target_snapshot: Option<String>,
}
#[derive(diesel::QueryableByName)]
struct PriorStartup {
    #[diesel(sql_type=BigInt)]
    n: i64,
}
macro_rules! input {
    ($connection:expr,$dialect:expr,$conversation:expr,$operation:expr) => {{
        diesel::sql_query($dialect.query(
            "SELECT target_snapshot FROM conversation_inputs WHERE conversation_id=? AND id=?",
        ))
        .bind::<IdentitySql, _>($conversation)
        .bind::<IdentitySql, _>($operation)
        .get_result::<InputTarget>($connection)
        .await
        .optional()?
        .ok_or(StoreError::NotFound)?
    }};
}
macro_rules! preparation {
    ($connection:expr,$dialect:expr,$operation:expr) => {{
        diesel::sql_query(
            $dialect.query("SELECT * FROM conversation_input_preparation WHERE operation_id=?"),
        )
        .bind::<IdentitySql, _>($operation)
        .get_result::<PreparationRow>($connection)
        .await
        .optional()?
    }};
}
fn ids(
    draft: &ConversationInputPreparation<'_>,
) -> Result<(EntityId, EntityId, EntityId), StoreError> {
    Ok((
        EntityId::from_dto(EntityKind::Agent, draft.agent_id)?,
        EntityId::from_dto(EntityKind::Conversation, draft.conversation_id)?,
        EntityId::from_dto(EntityKind::ConversationOperation, draft.operation_id)?,
    ))
}
impl Store {
    /// Native dequeue validates the sealed server intent again. A Control call
    /// alone cannot create setup permission or change its saved supply choice.
    pub async fn check_conversation_input_preparation(
        &self,
        source: &crate::ConnectorSource<'_>,
        draft: &ConversationInputPreparation<'_>,
        supply: &HarnessSupply,
        fresh: Option<bool>,
        target: Option<&ConversationDeliveryTarget>,
    ) -> Result<(), StoreError> {
        let (agent, id, operation) = ids(draft)?;
        operation!(self, |connection, dialect| {
            let row = super::records::source_conversation!(connection, dialect, source, id);
            let route = connector!(connection, dialect, source);
            if row.agent_id != agent {
                return Err(StoreError::NotFound);
            }
            let input = input!(connection, dialect, id, operation);
            let previous =
                preparation!(connection, dialect, operation).ok_or(StoreError::NotFound)?;
            if previous.execution_id != route._execution_id
                || &previous.facts()?.requested_supply != supply
                || !matches!(
                    previous.state()?,
                    ConversationPreparationState::PossibleForwarding
                )
            {
                return Err(StoreError::Conflict);
            }
            match (input.target_snapshot.as_deref(), fresh, target) {
                (None, Some(allowed), None)
                    if allowed == previous.fresh_native_allowed
                        && row.source_snapshot.is_none() => {}
                (Some(snapshot), None, Some(target))
                    if snapshot == target.snapshot.as_str()
                        && row.source_snapshot.as_deref() == Some(snapshot)
                        && row.native_session_id.as_deref()
                            == Some(target.native_session.as_str())
                        && row.source_state == "READY" => {}
                _ => return Err(StoreError::InvalidTransition),
            }
            Ok(())
        })
    }
    /// Capture saved supply and possible setup forwarding atomically. No source
    /// READY/native message identity is fabricated to obtain this separate grant.
    pub async fn begin_conversation_input_preparation(
        &self,
        actor: &SecretDigest,
        expected: &crate::ResolvedRoute,
        draft: &ConversationInputPreparation<'_>,
        now: i64,
    ) -> Result<ConversationPreparationAdmission, StoreError> {
        let (agent, id, operation) = ids(draft)?;
        operation!(self, |connection, dialect| {
            let owner = owned_agent!(connection, dialect, actor, agent, now);
            let (route, session_id) =
                super::delivery::dispatch_route!(connection, dialect, expected, owner.computer_id);
            let row = conversation!(connection, dialect, id);
            if row.agent_id != agent {
                return Err(StoreError::NotFound);
            }
            let input = input!(connection, dialect, id, operation);
            if let Some(previous) = preparation!(connection, dialect, operation) {
                if previous.execution_id != route._execution_id {
                    return Err(StoreError::Conflict);
                }
                return Ok(ConversationPreparationAdmission::Existing(
                    previous.facts()?,
                ));
            }
            let target = input
                .target_snapshot
                .map(SnapshotId::parse)
                .transpose()
                .map_err(|_| StoreError::Integrity)?;
            let valid = match (&row.source()?, &target) {
                (ConversationSource::NotStarted, None) => true,
                (ConversationSource::Ready { snapshot, .. }, Some(target)) => snapshot == target,
                _ => false,
            };
            if !valid {
                return Err(StoreError::InvalidTransition);
            }
            let prior=diesel::sql_query(dialect.query("SELECT CAST(count(*) AS BIGINT) AS n FROM conversation_input_preparation p JOIN conversation_inputs i ON i.id=p.operation_id WHERE i.conversation_id=? AND NOT (p.preparation_state='FAILED' AND (p.failure_kind IN ('NOT_ADMITTED','BEFORE_NATIVE_LAUNCH','NATIVE_REJECTED') OR (p.failure_kind='SOURCE_UNAVAILABLE' AND p.fresh_native_allowed=FALSE)))"))
                .bind::<IdentitySql,_>(id).get_result::<PriorStartup>(connection).await?;
            // Seal positive fresh-context permission once, rather than deriving
            // it later from a missing local owner or lost transport result.
            // A settled explicit rejection is not a native context. A failed
            // resume-only lookup cannot create one either. Unresolved starts,
            // retirement uncertainty and session-ready outcomes remain barriers.
            let fresh_native_allowed =
                target.is_none() && row.native_session_id.is_none() && prior.n == 0;
            let selection = super::supply::current_supply!(connection, dialect, agent);
            let selection_request_id = selection.saved.map(|receipt| receipt.request_id);
            let (kind, path) = match &selection.supply {
                HarnessSupply::Managed => ("MANAGED", None),
                HarnessSupply::Existing(path) => ("EXISTING", Some(path.as_str())),
            };
            diesel::sql_query(dialect.query("INSERT INTO conversation_input_preparation(operation_id,execution_id,supply_kind,executable_path,selection_request_id,fresh_native_allowed,preparation_state) VALUES(?,?,?,?,?,?,'POSSIBLE')"))
                .bind::<IdentitySql,_>(operation).bind::<IdentitySql,_>(route._execution_id).bind::<Text,_>(kind).bind::<Nullable<Text>,_>(path)
                .bind::<Nullable<Text>,_>(selection_request_id.as_deref()).bind::<diesel::sql_types::Bool,_>(fresh_native_allowed).execute(connection).await?;
            Ok(ConversationPreparationAdmission::Granted {
                grant: ConversationForwardingGrant {
                    execution_id: route._execution_id.to_dto(EntityKind::Execution),
                    connector_session_id: session_id,
                },
                requested_supply: selection.supply,
                selection_request_id,
                fresh_native_allowed,
            })
        })
    }
    /// Publish only an actual setup observation. Even READY grants no text replay
    /// and says nothing about later scope liveness or model work completion.
    pub async fn complete_conversation_input_preparation(
        &self,
        source: &crate::ConnectorSource<'_>,
        draft: &ConversationInputPreparation<'_>,
        outcome: &ConversationPreparationOutcome,
    ) -> Result<(), StoreError> {
        let (agent, id, operation) = ids(draft)?;
        let (state, snapshot, native, failure) = match outcome {
            ConversationPreparationOutcome::SessionReady(target) => (
                "READY",
                Some(target.snapshot.as_str()),
                Some(target.native_session.as_str()),
                None,
            ),
            ConversationPreparationOutcome::Failed(kind) => {
                ("FAILED", None, None, Some(kind.database()))
            }
        };
        operation!(self, |connection, dialect| {
            let row = super::records::source_conversation!(connection, dialect, source, id);
            let route = connector!(connection, dialect, source);
            if row.agent_id != agent {
                return Err(StoreError::NotFound);
            }
            let _ = input!(connection, dialect, id, operation);
            let previous = preparation!(connection, dialect, operation)
                .ok_or(StoreError::InvalidTransition)?;
            if previous.execution_id != route._execution_id {
                return Err(StoreError::Conflict);
            }
            if let ConversationPreparationOutcome::SessionReady(target) = outcome
                && (row
                    .source_snapshot
                    .as_deref()
                    .is_some_and(|snapshot| snapshot != target.snapshot.as_str())
                    || row
                        .native_session_id
                        .as_deref()
                        .is_some_and(|native| native != target.native_session.as_str()))
            {
                return Err(StoreError::Conflict);
            }
            match previous.state()? {
                ConversationPreparationState::Known(previous) if &previous == outcome => {
                    return Ok(());
                }
                ConversationPreparationState::Known(_) => return Err(StoreError::Conflict),
                ConversationPreparationState::PossibleForwarding => {}
            }
            diesel::sql_query(dialect.query("UPDATE conversation_input_preparation SET preparation_state=?,ready_snapshot=?,ready_native_session=?,failure_kind=? WHERE operation_id=?"))
                .bind::<Text,_>(state).bind::<Nullable<Text>,_>(snapshot).bind::<Nullable<Text>,_>(native).bind::<Nullable<Text>,_>(failure).bind::<IdentitySql,_>(operation).execute(connection).await?;
            Ok(())
        })
    }
    /// Missing preparation facts do not authorize setup; this is a private read.
    pub async fn conversation_input_preparation(
        &self,
        actor: &SecretDigest,
        draft: &ConversationInputPreparation<'_>,
        now: i64,
    ) -> Result<Option<ConversationPreparationFacts>, StoreError> {
        let (agent, id, operation) = ids(draft)?;
        operation!(self, |connection, dialect| {
            let _ = owned_agent!(connection, dialect, actor, agent, now);
            let row = conversation!(connection, dialect, id);
            if row.agent_id != agent {
                return Err(StoreError::NotFound);
            }
            let _ = input!(connection, dialect, id, operation);
            preparation!(connection, dialect, operation)
                .map(|row| row.facts())
                .transpose()
        })
    }
}
