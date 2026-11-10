//! Current native work is an explicit captured observation, not reconstructed authority.
use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConversationPreparationFreshness {
    Current,
    TransientLookupFailure,
}
/// Nonsecret facts selected by one physical native start, not saved settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConversationNativePreparation {
    pub freshness: ConversationPreparationFreshness,
    pub node: String,
    pub adapter: String,
    pub codex: String,
}
impl ConversationNativePreparation {
    fn validate(&self) -> Result<(), StoreError> {
        for version in [&self.node, &self.adapter, &self.codex] {
            if version.is_empty() || version.len() > 256 || version.chars().any(char::is_control) {
                return Err(StoreError::InvalidInput);
            }
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversationForeground {
    Idle,
    Running,
    RequiresAction,
    OtherActive,
}
impl ConversationForeground {
    fn database(self) -> &'static str {
        match self {
            Self::Idle => "IDLE",
            Self::Running => "RUNNING",
            Self::RequiresAction => "REQUIRES_ACTION",
            Self::OtherActive => "OTHER_ACTIVE",
        }
    }
    fn parse(value: &str) -> Result<Self, StoreError> {
        match value {
            "IDLE" => Ok(Self::Idle),
            "RUNNING" => Ok(Self::Running),
            "REQUIRES_ACTION" => Ok(Self::RequiresAction),
            "OTHER_ACTIVE" => Ok(Self::OtherActive),
            _ => Err(StoreError::Integrity),
        }
    }
}
#[derive(Clone, PartialEq, Eq)]
pub struct ConversationNativeObservation {
    pub target: ConversationDeliveryTarget,
    pub position: SourcePosition,
    pub foreground: ConversationForeground,
    pub work_id: Option<String>,
    pub preparation: Option<ConversationNativePreparation>,
    pub authentication_id: Option<String>,
    /// A later physical start may reuse a retained capture; freezing belongs to the native owner.
    pub applied_supply: Option<HarnessSupply>,
}
impl fmt::Debug for ConversationNativeObservation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ConversationNativeObservation([REDACTED])")
    }
}
#[derive(diesel::QueryableByName)]
pub(super) struct NativeRow {
    #[diesel(sql_type=IdentitySql)]
    pub(super) execution_id: EntityId,
    #[diesel(sql_type=Text)]
    source_snapshot: String,
    #[diesel(sql_type=Text)]
    native_session_id: String,
    #[diesel(sql_type=BigInt)]
    ordinal: i64,
    #[diesel(sql_type=Text)]
    occurrence_id: String,
    #[diesel(sql_type=Text)]
    foreground: String,
    #[diesel(sql_type=Nullable<Text>)]
    work_id: Option<String>,
    #[diesel(sql_type=Nullable<Binary>)]
    preparation_payload: Option<Vec<u8>>,
    #[diesel(sql_type=Nullable<IdentitySql>)]
    authentication_id: Option<EntityId>,
    #[diesel(sql_type=Nullable<Text>)]
    applied_supply_kind: Option<String>,
    #[diesel(sql_type=Nullable<Text>)]
    applied_executable_path: Option<String>,
}
impl NativeRow {
    pub(super) fn observation(&self) -> Result<ConversationNativeObservation, StoreError> {
        Ok(ConversationNativeObservation {
            target: ConversationDeliveryTarget {
                snapshot: SnapshotId::parse(self.source_snapshot.clone())
                    .map_err(|_| StoreError::Integrity)?,
                native_session: NativeConversationId::parse(self.native_session_id.clone())
                    .map_err(|_| StoreError::Integrity)?,
            },
            position: SourcePosition::new(
                self.ordinal,
                EventOccurrenceId::parse(self.occurrence_id.clone())
                    .map_err(|_| StoreError::Integrity)?,
            )
            .map_err(|_| StoreError::Integrity)?,
            foreground: ConversationForeground::parse(&self.foreground)?,
            work_id: self.work_id.clone(),
            preparation: self
                .preparation_payload
                .as_deref()
                .map(|bytes| {
                    let value: ConversationNativePreparation =
                        serde_json::from_slice(bytes).map_err(|_| StoreError::Integrity)?;
                    value.validate().map_err(|_| StoreError::Integrity)?;
                    Ok::<_, StoreError>(value)
                })
                .transpose()?,
            authentication_id: self
                .authentication_id
                .map(|id| id.to_dto(EntityKind::Authentication)),
            applied_supply: match (
                self.applied_supply_kind.as_deref(),
                &self.applied_executable_path,
            ) {
                (None, None) => None,
                (Some("MANAGED"), None) => Some(HarnessSupply::Managed),
                (Some("EXISTING"), Some(path)) => Some(HarnessSupply::Existing(
                    ExistingCodexPath::parse(path.clone()).map_err(|_| StoreError::Integrity)?,
                )),
                _ => return Err(StoreError::Integrity),
            },
        })
    }
}
#[derive(diesel::QueryableByName)]
struct Pair {
    #[diesel(sql_type=BigInt)]
    ordinal: i64,
}
macro_rules! native_state {
    ($connection:expr,$dialect:expr,$conversation:expr) => {{
        diesel::sql_query(
            $dialect.query("SELECT n.*,d.preparation_payload,d.authentication_id,s.supply_kind AS applied_supply_kind,s.executable_path AS applied_executable_path FROM conversation_native_state n LEFT JOIN conversation_native_details d ON d.conversation_id=n.conversation_id LEFT JOIN conversation_applied_supply s ON s.conversation_id=n.conversation_id WHERE n.conversation_id=?"),
        )
        .bind::<IdentitySql, _>($conversation)
        .get_result::<$crate::conversations::native_state::NativeRow>($connection)
        .await
        .optional()?
    }};
}
pub(super) use native_state;
impl Store {
    pub async fn observe_conversation_native(
        &self,
        source: &crate::ConnectorSource<'_>,
        conversation_id: &str,
        observation: &ConversationNativeObservation,
    ) -> Result<(), StoreError> {
        if let Some(preparation) = &observation.preparation {
            preparation.validate()?;
        }
        let authentication = observation
            .authentication_id
            .as_deref()
            .map(|id| EntityId::from_dto(EntityKind::Authentication, id))
            .transpose()?;
        if observation.foreground == ConversationForeground::Idle {
            if observation.work_id.is_some() {
                return Err(StoreError::InvalidInput);
            }
        } else {
            request_id(
                observation
                    .work_id
                    .as_deref()
                    .ok_or(StoreError::InvalidInput)?,
            )?;
        }
        let id = EntityId::from_dto(EntityKind::Conversation, conversation_id)?;
        operation!(self, |connection, dialect| {
            let row = super::records::source_conversation!(connection, dialect, source, id);
            let route = connector!(connection, dialect, source);
            super::delivery::current(&row, row.agent_id, &observation.target)?;
            let pair=diesel::sql_query(dialect.query("SELECT ordinal FROM conversation_event_pairs WHERE conversation_id=? AND occurrence_id=?"))
                .bind::<IdentitySql,_>(id).bind::<Text,_>(observation.position.occurrence().as_str()).get_result::<Pair>(connection).await.optional()?;
            if pair.is_none_or(|pair| pair.ordinal != observation.position.ordinal()) {
                return Err(StoreError::InvalidTransition);
            }
            let mut preparation = observation.preparation.clone();
            let mut authentication = authentication;
            let mut supply = observation.applied_supply.clone();
            if let Some(previous) = native_state!(connection, dialect, id) {
                let old = previous.observation()?;
                if previous.execution_id == route._execution_id && old.target == observation.target
                {
                    if old.position.ordinal() > observation.position.ordinal() {
                        return Ok(());
                    }
                    if old.position.ordinal() == observation.position.ordinal()
                        && (old.position != observation.position
                            || old.foreground != observation.foreground
                            || old.work_id != observation.work_id
                            || old
                                .preparation
                                .as_ref()
                                .zip(preparation.as_ref())
                                .is_some_and(|(a, b)| a != b)
                            || previous
                                .authentication_id
                                .zip(authentication)
                                .is_some_and(|(a, b)| a != b)
                            || old
                                .applied_supply
                                .as_ref()
                                .zip(supply.as_ref())
                                .is_some_and(|(a, b)| a != b))
                    {
                        return Err(StoreError::Conflict);
                    }
                    preparation = preparation.or(old.preparation);
                    authentication = authentication.or(previous.authentication_id);
                    supply = supply.or(old.applied_supply);
                }
            }
            diesel::sql_query(dialect.query("INSERT INTO conversation_native_state(conversation_id,execution_id,source_snapshot,native_session_id,ordinal,occurrence_id,foreground,work_id) VALUES(?,?,?,?,?,?,?,?) ON CONFLICT(conversation_id) DO UPDATE SET execution_id=excluded.execution_id,source_snapshot=excluded.source_snapshot,native_session_id=excluded.native_session_id,ordinal=excluded.ordinal,occurrence_id=excluded.occurrence_id,foreground=excluded.foreground,work_id=excluded.work_id"))
                .bind::<IdentitySql,_>(id).bind::<IdentitySql,_>(route._execution_id).bind::<Text,_>(observation.target.snapshot.as_str()).bind::<Text,_>(observation.target.native_session.as_str())
                .bind::<BigInt,_>(observation.position.ordinal()).bind::<Text,_>(observation.position.occurrence().as_str()).bind::<Text,_>(observation.foreground.database()).bind::<Nullable<Text>,_>(observation.work_id.as_deref()).execute(connection).await?;
            if preparation.is_some() || authentication.is_some() {
                let payload = preparation
                    .as_ref()
                    .map(serde_json::to_vec)
                    .transpose()
                    .map_err(|_| StoreError::Integrity)?;
                diesel::sql_query(dialect.query("INSERT INTO conversation_native_details(conversation_id,preparation_payload,authentication_id) VALUES(?,?,?) ON CONFLICT(conversation_id) DO UPDATE SET preparation_payload=excluded.preparation_payload,authentication_id=excluded.authentication_id"))
                    .bind::<IdentitySql,_>(id).bind::<Nullable<Binary>,_>(payload).bind::<Nullable<IdentitySql>,_>(authentication).execute(connection).await?;
            } else {
                diesel::sql_query(
                    dialect
                        .query("DELETE FROM conversation_native_details WHERE conversation_id=?"),
                )
                .bind::<IdentitySql, _>(id)
                .execute(connection)
                .await?;
            }
            if let Some(supply) = &supply {
                let (kind, path) = match supply {
                    HarnessSupply::Managed => ("MANAGED", None),
                    HarnessSupply::Existing(path) => ("EXISTING", Some(path.as_str())),
                };
                diesel::sql_query(dialect.query("INSERT INTO conversation_applied_supply(conversation_id,supply_kind,executable_path) VALUES(?,?,?) ON CONFLICT(conversation_id) DO UPDATE SET supply_kind=excluded.supply_kind,executable_path=excluded.executable_path"))
                    .bind::<IdentitySql,_>(id).bind::<Text,_>(kind).bind::<Nullable<Text>,_>(path).execute(connection).await?;
            } else {
                diesel::sql_query(
                    dialect
                        .query("DELETE FROM conversation_applied_supply WHERE conversation_id=?"),
                )
                .bind::<IdentitySql, _>(id)
                .execute(connection)
                .await?;
            }
            Ok(())
        })
    }
    /// An observed state is not a guarantee of present process liveness. A Stop
    /// still carries this exact work identity and checks the actual native owner.
    pub async fn conversation_native_observation(
        &self,
        actor: &SecretDigest,
        agent_id: &str,
        conversation_id: &str,
        now: i64,
    ) -> Result<Option<ConversationNativeObservation>, StoreError> {
        let agent = EntityId::from_dto(EntityKind::Agent, agent_id)?;
        let id = EntityId::from_dto(EntityKind::Conversation, conversation_id)?;
        operation!(self, |connection, dialect| {
            let owner = owned_agent!(connection, dialect, actor, agent, now);
            let row = conversation!(connection, dialect, id);
            if row.agent_id != agent {
                return Err(StoreError::NotFound);
            }
            #[derive(diesel::QueryableByName)]
            struct Execution {
                #[diesel(sql_type=Nullable<IdentitySql>)]
                current_execution_id: Option<EntityId>,
            }
            let execution = diesel::sql_query(
                dialect.query("SELECT current_execution_id FROM computers WHERE id=?"),
            )
            .bind::<IdentitySql, _>(owner.computer_id)
            .get_result::<Execution>(connection)
            .await?;
            let Some(observed) = native_state!(connection, dialect, id) else {
                return Ok(None);
            };
            if execution.current_execution_id != Some(observed.execution_id) {
                return Ok(None);
            }
            Ok(Some(observed.observation()?))
        })
    }
}
