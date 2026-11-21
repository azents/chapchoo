//! Server-owned conversation model preferences and the pre-start ordering fence.

use super::*;
use crate::ConnectorSource;
use chapchoo_domain::{
    ConversationModelCapabilities, ConversationModelTuning, ConversationPermissionMode,
};
use diesel::sql_types::Bool;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConversationModelChoice {
    pub id: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<ConversationModelCapabilities>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConversationModelTarget {
    pub conversation_id: String,
    pub snapshot: SnapshotId,
    pub native_session: NativeConversationId,
    pub launch_operation_id: String,
}
impl ConversationModelTarget {
    fn ids(&self) -> Result<(EntityId, EntityId), StoreError> {
        Ok((
            EntityId::from_dto(EntityKind::Conversation, &self.conversation_id)?,
            EntityId::from_dto(EntityKind::ConversationOperation, &self.launch_operation_id)?,
        ))
    }
}

/// Qualified against the source and actual applied authentication in Store.
pub struct ConversationModelObservation {
    pub target: ConversationModelTarget,
    pub order: i64,
    pub current: Option<ConversationModelChoice>,
    pub options: Option<Vec<ConversationModelChoice>>,
    /// An opaque continuity witness for an applied API key, never the key.
    pub api_key_source: Option<[u8; 24]>,
    pub permission_modes: Vec<ConversationPermissionMode>,
}

pub struct ConversationModelProjection {
    pub target: ConversationModelTarget,
    pub model: Option<ConversationModelChoice>,
    pub saved_request_id: Option<String>,
    pub options: Option<Vec<ConversationModelChoice>>,
    pub recovery_operation_id: Option<String>,
    pub tuning: ConversationModelTuning,
    pub permission_modes: Vec<ConversationPermissionMode>,
}

pub struct ConversationModelSave {
    pub target: ConversationModelTarget,
    pub request_id: String,
    pub expected_request_id: Option<String>,
    pub model_id: String,
    pub tuning: ConversationModelTuning,
}

pub struct ConversationModelSaveReceipt {
    pub request_id: String,
    pub model: ConversationModelChoice,
    pub accepted_at: i64,
    pub tuning: ConversationModelTuning,
}

pub struct ConversationModelStart {
    pub target: ConversationModelTarget,
    pub operation_id: String,
    /// True only if this owner can apply the model and prove a qualified start.
    pub capable: bool,
}

pub enum ConversationModelStartAdmission {
    Ungoverned,
    Reserved {
        model_id: Option<String>,
        tuning: ConversationModelTuning,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConversationModelStartOutcome {
    Inserted(NativeConversationId),
    NotPublished,
    ModelUnavailableBeforeStart(String),
}

macro_rules! redacted_debug {
    ($($name:ty),+ $(,)?) => {$(
        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(concat!(stringify!($name), "([REDACTED])"))
            }
        }
    )+};
}
redacted_debug!(
    ConversationModelObservation,
    ConversationModelProjection,
    ConversationModelSave,
    ConversationModelSaveReceipt,
    ConversationModelStart,
    ConversationModelStartAdmission,
);

fn valid_model_id(value: &str) -> bool {
    !value.is_empty() && value.len() <= 256 && value.bytes().all(|b| b.is_ascii_graphic())
}

fn valid_choice(value: &ConversationModelChoice) -> bool {
    valid_model_id(&value.id)
        && !value.label.trim().is_empty()
        && value.label.len() <= 256
        && !value.label.chars().any(char::is_control)
        && value.capabilities.as_ref().is_none_or(|options| {
            options.efforts.len() <= 32
                && options.efforts.iter().all(|effort| valid_model_id(effort))
                && options.efforts.iter().collect::<HashSet<_>>().len() == options.efforts.len()
                && options.speeds.len() <= 3
                && options.speeds.iter().collect::<HashSet<_>>().len() == options.speeds.len()
        })
}

// The saved model label is also used for a permission-only receipt. In that
// case it is display evidence, not an instruction to pin the native model.
#[derive(Default, Serialize, Deserialize)]
struct StoredTuning {
    #[serde(flatten)]
    tuning: ConversationModelTuning,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    inherit_model: bool,
}

fn stored_tuning(value: Option<&str>) -> Result<StoredTuning, StoreError> {
    value
        .map(|data| serde_json::from_str(data).map_err(|_| StoreError::Integrity))
        .transpose()
        .map(|value| value.unwrap_or_default())
}

fn tuning_from_storage(value: Option<&str>) -> Result<ConversationModelTuning, StoreError> {
    Ok(stored_tuning(value)?.tuning)
}

fn tuning_payload(tuning: &StoredTuning) -> Result<Option<String>, StoreError> {
    (tuning.inherit_model || !tuning.tuning.is_inherited())
        .then(|| serde_json::to_string(tuning).map_err(|_| StoreError::InvalidInput))
        .transpose()
}

#[cfg(test)]
mod stored_permissions {
    use super::*;

    #[test]
    fn stored_permission_preserves_inheritance_and_reads_existing_tuning() -> Result<(), StoreError>
    {
        let old = stored_tuning(Some(r#"{"effort":"high","speed":"fast"}"#))?;
        assert_eq!(old.tuning.effort.as_deref(), Some("high"));
        assert_eq!(old.tuning.permission_mode, None);
        assert!(!old.inherit_model);
        let chosen = StoredTuning {
            inherit_model: true,
            tuning: ConversationModelTuning {
                permission_mode: Some(ConversationPermissionMode::AskForApproval),
                ..Default::default()
            },
        };
        let saved = tuning_payload(&chosen)?;
        let read = stored_tuning(saved.as_deref())?;
        assert_eq!(read.tuning, chosen.tuning);
        assert!(read.inherit_model);
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum AdvertisedOptions {
    Models(Vec<ConversationModelChoice>),
    WithPermissions {
        models: Vec<ConversationModelChoice>,
        permission_modes: Vec<ConversationPermissionMode>,
    },
}

impl AdvertisedOptions {
    fn split(
        self,
    ) -> (
        Vec<ConversationModelChoice>,
        Vec<ConversationPermissionMode>,
    ) {
        match self {
            Self::Models(models) => (models, Vec::new()),
            Self::WithPermissions {
                models,
                permission_modes,
            } => (models, permission_modes),
        }
    }
}

fn option_payload(
    options: Option<&[ConversationModelChoice]>,
    permission_modes: &[ConversationPermissionMode],
) -> Result<Option<String>, StoreError> {
    if options.is_none() && permission_modes.is_empty() {
        return Ok(None);
    }
    let options = options.unwrap_or_default();
    let mut ids = HashSet::new();
    if options.len() > 128
        || options
            .iter()
            .any(|item| !valid_choice(item) || !ids.insert(item.id.as_str()))
    {
        return Err(StoreError::InvalidInput);
    }
    if permission_modes.len() > 3
        || permission_modes
            .iter()
            .copied()
            .collect::<HashSet<_>>()
            .len()
            != permission_modes.len()
    {
        return Err(StoreError::InvalidInput);
    }
    let json = if permission_modes.is_empty() {
        serde_json::to_string(options)
    } else {
        serde_json::to_string(&AdvertisedOptions::WithPermissions {
            models: options.to_vec(),
            permission_modes: permission_modes.to_vec(),
        })
    }
    .map_err(|_| StoreError::InvalidInput)?;
    if json.len() > 48 * 1024 {
        return Err(StoreError::InvalidInput);
    }
    Ok(Some(json))
}

#[derive(diesel::QueryableByName)]
struct ScopeRow {
    #[diesel(sql_type = IdentitySql)]
    conversation_id: EntityId,
    #[diesel(sql_type = Text)]
    source_snapshot: String,
    #[diesel(sql_type = Text)]
    native_session_id: String,
    #[diesel(sql_type = IdentitySql)]
    launch_operation_id: EntityId,
    #[diesel(sql_type = IdentitySql)]
    authentication_id: EntityId,
    #[diesel(sql_type = Nullable<Binary>)]
    api_key_source: Option<Vec<u8>>,
    #[diesel(sql_type = Nullable<Text>)]
    observed_current_id: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    observed_current_label: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    advertised_options: Option<String>,
    #[diesel(sql_type = BigInt)]
    observation_order: i64,
    #[diesel(sql_type = Nullable<Text>)]
    saved_model_id: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    saved_model_label: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    saved_tuning: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    saved_request_id: Option<String>,
    #[diesel(sql_type = Nullable<IdentitySql>)]
    recovery_operation_id: Option<EntityId>,
}
impl ScopeRow {
    fn matches(&self, target: &ConversationModelTarget, launch: EntityId) -> bool {
        self.source_snapshot == target.snapshot.as_str()
            && self.native_session_id == target.native_session.as_str()
            && self.launch_operation_id == launch
    }

    fn options(&self) -> Result<Option<AdvertisedOptions>, StoreError> {
        self.advertised_options
            .as_ref()
            .map(|value| serde_json::from_str(value).map_err(|_| StoreError::Integrity))
            .transpose()
    }

    fn projection(self) -> Result<ConversationModelProjection, StoreError> {
        let (options, permission_modes) = self
            .options()?
            .map(AdvertisedOptions::split)
            .map_or((None, Vec::new()), |(models, modes)| (Some(models), modes));
        let choice = |id: String, label| ConversationModelChoice {
            capabilities: options
                .as_ref()
                .and_then(|options| options.iter().find(|choice| choice.id == id))
                .and_then(|choice| choice.capabilities.clone()),
            id,
            label,
        };
        let stored = stored_tuning(self.saved_tuning.as_deref())?;
        let model = match if stored.inherit_model {
            (None, None)
        } else {
            (self.saved_model_id, self.saved_model_label)
        } {
            (Some(id), Some(label)) => Some(choice(id, label)),
            (None, None) => match (self.observed_current_id, self.observed_current_label) {
                (Some(id), Some(label)) => Some(choice(id, label)),
                (None, None) => None,
                _ => return Err(StoreError::Integrity),
            },
            _ => return Err(StoreError::Integrity),
        };
        Ok(ConversationModelProjection {
            target: ConversationModelTarget {
                conversation_id: self.conversation_id.to_dto(EntityKind::Conversation),
                snapshot: SnapshotId::parse(self.source_snapshot)
                    .map_err(|_| StoreError::Integrity)?,
                native_session: NativeConversationId::parse(self.native_session_id)
                    .map_err(|_| StoreError::Integrity)?,
                launch_operation_id: self
                    .launch_operation_id
                    .to_dto(EntityKind::ConversationOperation),
            },
            model,
            saved_request_id: self.saved_request_id,
            tuning: stored.tuning,
            options,
            permission_modes,
            recovery_operation_id: self
                .recovery_operation_id
                .map(|value| value.to_dto(EntityKind::ConversationOperation)),
        })
    }
}

#[derive(diesel::QueryableByName)]
struct SaveRow {
    #[diesel(sql_type = IdentitySql)]
    conversation_id: EntityId,
    #[diesel(sql_type = Text)]
    source_snapshot: String,
    #[diesel(sql_type = Text)]
    native_session_id: String,
    #[diesel(sql_type = IdentitySql)]
    launch_operation_id: EntityId,
    #[diesel(sql_type = Nullable<Text>)]
    expected_request_id: Option<String>,
    #[diesel(sql_type = Text)]
    model_id: String,
    #[diesel(sql_type = Text)]
    model_label: String,
    #[diesel(sql_type = Nullable<Text>)]
    tuning: Option<String>,
    #[diesel(sql_type = Text)]
    request_id: String,
    #[diesel(sql_type = BigInt)]
    accepted_at: i64,
}
impl SaveRow {
    fn receipt(self) -> Result<ConversationModelSaveReceipt, StoreError> {
        Ok(ConversationModelSaveReceipt {
            request_id: self.request_id,
            model: ConversationModelChoice {
                id: self.model_id,
                label: self.model_label,
                capabilities: None,
            },
            accepted_at: self.accepted_at,
            tuning: tuning_from_storage(self.tuning.as_deref())?,
        })
    }
}

#[derive(diesel::QueryableByName)]
struct AppliedBinding {
    #[diesel(sql_type = IdentitySql)]
    authentication_id: EntityId,
    #[diesel(sql_type = Text)]
    kind: String,
}

#[derive(diesel::QueryableByName)]
pub(super) struct Present {
    #[diesel(sql_type = Bool)]
    pub(super) present: bool,
}

#[derive(diesel::QueryableByName)]
struct StartRow {
    #[diesel(sql_type = IdentitySql)]
    execution_id: EntityId,
    #[diesel(sql_type = IdentitySql)]
    launch_operation_id: EntityId,
    #[diesel(sql_type = Text)]
    source_snapshot: String,
    #[diesel(sql_type = Text)]
    native_session_id: String,
    #[diesel(sql_type = Nullable<Text>)]
    settlement: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    native_message_id: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    model_id: Option<String>,
}

macro_rules! scope {
    ($cx:expr, $dialect:expr, $conversation:expr) => {
        diesel::sql_query(
            $dialect.query("SELECT * FROM conversation_model_scopes WHERE conversation_id=?"),
        )
        .bind::<IdentitySql, _>($conversation)
        .get_result::<ScopeRow>($cx)
        .await
        .optional()?
    };
}

macro_rules! pending {
    ($cx:expr, $dialect:expr, $conversation:expr) => {
        diesel::sql_query($dialect.query(
            "SELECT EXISTS(SELECT 1 FROM conversation_model_starts WHERE conversation_id=? AND settlement IS NULL) AS present",
        ))
        .bind::<IdentitySql, _>($conversation)
        .get_result::<Present>($cx)
        .await?
        .present
    };
}

macro_rules! receipt {
    ($cx:expr, $dialect:expr, $agent:expr, $request:expr) => {
        diesel::sql_query($dialect.query(
            "SELECT * FROM conversation_model_save_receipts WHERE agent_id=? AND request_id=?",
        ))
        .bind::<IdentitySql, _>($agent)
        .bind::<Text, _>($request)
        .get_result::<SaveRow>($cx)
        .await
        .optional()?
    };
}

macro_rules! applied_binding {
    ($cx:expr, $dialect:expr, $conversation:expr, $computer:expr, $launch:expr, $target:expr) => {
        diesel::sql_query($dialect.query(
            "SELECT a.id AS authentication_id, a.kind \
                FROM conversations v JOIN agent_conversations av ON av.conversation_id=v.id AND av.agent_id=v.agent_id \
                JOIN agents g ON g.id=v.agent_id JOIN memberships m ON m.team_id=g.team_id AND m.user_id=g.owner_user_id \
                JOIN computers c ON c.id=g.computer_id \
                JOIN execution_authentication_applications x ON x.execution_id=c.current_execution_id \
                    AND x.agent_id=g.id AND x.conversation_id=v.id AND x.is_current=TRUE \
                JOIN external_authentications a ON a.id=x.authentication_id \
                WHERE v.id=? AND c.id=? AND x.operation_id=? AND v.source_snapshot=? AND v.native_session_id=? \
                    AND v.source_state='READY'",
        ))
        .bind::<IdentitySql, _>($conversation)
        .bind::<IdentitySql, _>($computer)
        .bind::<IdentitySql, _>($launch)
        .bind::<Text, _>($target.snapshot.as_str())
        .bind::<Text, _>($target.native_session.as_str())
        .get_result::<AppliedBinding>($cx)
        .await
        .optional()?
    };
}

impl Store {
    /// Current scope read. A stored choice can survive process loss but
    /// cannot represent the current model until continuity is proved.
    pub async fn conversation_model(
        &self,
        actor: &SecretDigest,
        agent_id: &str,
        now: i64,
    ) -> Result<Option<ConversationModelProjection>, StoreError> {
        let agent = EntityId::from_dto(EntityKind::Agent, agent_id)?;
        operation!(self, |connection, dialect| {
            let context = owned_agent!(connection, dialect, actor, agent, now);
            let row = diesel::sql_query(dialect.query(
                "SELECT s.* FROM conversation_model_scopes s \
                 JOIN agent_conversations a ON a.conversation_id=s.conversation_id \
                 JOIN conversations v ON v.id=a.conversation_id AND v.agent_id=a.agent_id \
                 WHERE a.agent_id=? AND v.source_snapshot=s.source_snapshot AND v.native_session_id=s.native_session_id \
                    AND v.source_state='READY'",
            ))
            .bind::<IdentitySql, _>(agent)
            .get_result::<ScopeRow>(connection)
            .await
            .optional()?;
            let Some(row) = row else { return Ok(None) };
            let target = row_target(&row)?;
            let live = applied_binding!(
                connection,
                dialect,
                row.conversation_id,
                context.computer_id,
                row.launch_operation_id,
                target
            )
            .is_some_and(|applied| applied.authentication_id == row.authentication_id);
            if !live {
                return Ok(None);
            }
            Ok(Some(row.projection()?))
        })
    }

    pub async fn conversation_model_save_result(
        &self,
        actor: &SecretDigest,
        agent_id: &str,
        id: &str,
        now: i64,
    ) -> Result<Option<ConversationModelSaveReceipt>, StoreError> {
        request_id(id)?;
        let agent = EntityId::from_dto(EntityKind::Agent, agent_id)?;
        operation!(self, |connection, dialect| {
            let _ = owned_agent!(connection, dialect, actor, agent, now);
            receipt!(connection, dialect, agent, id)
                .map(SaveRow::receipt)
                .transpose()
        })
    }

    pub async fn save_conversation_model(
        &self,
        actor: &SecretDigest,
        agent_id: &str,
        draft: &ConversationModelSave,
        now: i64,
    ) -> Result<ConversationModelSaveReceipt, StoreError> {
        request_id(&draft.request_id)?;
        if let Some(expected) = &draft.expected_request_id {
            request_id(expected)?;
        }
        if !valid_model_id(&draft.model_id) {
            return Err(StoreError::InvalidInput);
        }
        let agent = EntityId::from_dto(EntityKind::Agent, agent_id)?;
        let (conversation, launch) = draft.target.ids()?;
        operation!(self, |connection, dialect| {
            let context = owned_agent!(connection, dialect, actor, agent, now);
            if let Some(prior) = receipt!(connection, dialect, agent, &draft.request_id) {
                if prior.conversation_id != conversation
                    || prior.launch_operation_id != launch
                    || prior.source_snapshot != draft.target.snapshot.as_str()
                    || prior.native_session_id != draft.target.native_session.as_str()
                    || prior.expected_request_id != draft.expected_request_id
                    || prior.model_id != draft.model_id
                    || tuning_from_storage(prior.tuning.as_deref())? != draft.tuning
                {
                    return Err(StoreError::Conflict);
                }
                return prior.receipt();
            }
            let owned = conversation!(connection, dialect, conversation);
            if owned.agent_id != agent {
                return Err(StoreError::NotFound);
            }
            let scope = scope!(connection, dialect, conversation)
                .filter(|row| row.matches(&draft.target, launch))
                .ok_or(StoreError::InvalidTransition)?;
            let same_binding = applied_binding!(
                connection,
                dialect,
                conversation,
                context.computer_id,
                launch,
                draft.target
            )
            .is_some_and(|applied| applied.authentication_id == scope.authentication_id);
            if context.desired_state == "DELETING"
                || !same_binding
                || pending!(connection, dialect, conversation)
            {
                return Err(StoreError::InvalidTransition);
            }
            if scope.saved_request_id != draft.expected_request_id {
                return Err(StoreError::Conflict);
            }
            if scope.saved_request_id.is_none() {
                // An older Connector can already hold an ungoverned handoff.
                let preceding = diesel::sql_query(dialect.query(
                    "SELECT (EXISTS(SELECT 1 FROM conversation_input_delivery d JOIN conversation_inputs i ON i.id=d.operation_id
                         JOIN computers c ON c.id=? AND c.current_execution_id=d.execution_id
                         WHERE i.conversation_id=? AND d.delivery_state='POSSIBLE'
                           AND NOT EXISTS(SELECT 1 FROM conversation_model_starts s WHERE s.conversation_id=i.conversation_id AND s.operation_id=d.operation_id AND s.settlement IS NOT NULL))
                       OR EXISTS(SELECT 1 FROM conversation_continuation_inputs q
                         JOIN computers c ON c.id=? AND c.current_execution_id=q.execution_id
                         WHERE q.conversation_id=? AND q.delivery_state='POSSIBLE'
                           AND NOT EXISTS(SELECT 1 FROM conversation_model_starts s WHERE s.conversation_id=q.conversation_id AND s.operation_id=q.id AND s.settlement IS NOT NULL))) AS present",
                ))
                .bind::<IdentitySql,_>(context.computer_id).bind::<IdentitySql,_>(conversation)
                .bind::<IdentitySql,_>(context.computer_id).bind::<IdentitySql,_>(conversation)
                .get_result::<Present>(connection).await?.present;
                if preceding {
                    return Err(StoreError::InvalidTransition);
                }
            }
            let (choices, permission_modes) = scope
                .options()?
                .map(AdvertisedOptions::split)
                .ok_or(StoreError::InvalidTransition)?;
            if draft
                .tuning
                .permission_mode
                .is_some_and(|mode| !permission_modes.contains(&mode))
            {
                return Err(StoreError::InvalidTransition);
            }
            let choice = choices
                .into_iter()
                .find(|item| item.id == draft.model_id)
                .ok_or(StoreError::InvalidTransition)?;
            if draft.tuning.has_model_tuning()
                && !choice
                    .capabilities
                    .as_ref()
                    .is_some_and(|options| options.accepts(&draft.tuning))
            {
                return Err(StoreError::InvalidTransition);
            }
            let previously = stored_tuning(scope.saved_tuning.as_deref())?;
            let inherit_model = (scope.saved_model_id.is_none() || previously.inherit_model)
                && scope.observed_current_id.as_deref() == Some(&draft.model_id)
                && !draft.tuning.has_model_tuning()
                && previously.tuning.permission_mode != draft.tuning.permission_mode;
            let tuning = tuning_payload(&StoredTuning {
                tuning: draft.tuning.clone(),
                inherit_model,
            })?;
            diesel::sql_query(dialect.query("UPDATE conversation_model_scopes SET saved_model_id=?,saved_model_label=?,saved_tuning=?,saved_request_id=?,recovery_operation_id=NULL WHERE conversation_id=?"))
                .bind::<Text, _>(&choice.id).bind::<Text, _>(&choice.label)
                .bind::<Nullable<Text>, _>(&tuning)
                .bind::<Text, _>(&draft.request_id).bind::<IdentitySql, _>(conversation)
                .execute(connection).await?;
            diesel::sql_query(dialect.query("INSERT INTO conversation_model_save_receipts (agent_id,request_id,conversation_id,source_snapshot,native_session_id,launch_operation_id,expected_request_id,model_id,model_label,accepted_at,tuning) VALUES (?,?,?,?,?,?,?,?,?,?,?)"))
                .bind::<IdentitySql, _>(agent).bind::<Text, _>(&draft.request_id)
                .bind::<IdentitySql, _>(conversation).bind::<Text, _>(draft.target.snapshot.as_str())
                .bind::<Text, _>(draft.target.native_session.as_str()).bind::<IdentitySql, _>(launch)
                .bind::<Nullable<Text>, _>(&draft.expected_request_id).bind::<Text, _>(&choice.id)
                .bind::<Text, _>(&choice.label).bind::<BigInt, _>(now)
                .bind::<Nullable<Text>, _>(&tuning)
                .execute(connection).await?;
            Ok(ConversationModelSaveReceipt {
                request_id: draft.request_id.clone(),
                model: ConversationModelChoice {
                    capabilities: None,
                    ..choice
                },
                accepted_at: now,
                tuning: draft.tuning.clone(),
            })
        })
    }

    pub async fn observe_conversation_model(
        &self,
        source: &ConnectorSource<'_>,
        observation: &ConversationModelObservation,
    ) -> Result<(), StoreError> {
        if observation.order < 0
            || observation
                .current
                .as_ref()
                .is_some_and(|value| !valid_choice(value))
        {
            return Err(StoreError::InvalidInput);
        }
        let choices = option_payload(
            observation.options.as_deref(),
            &observation.permission_modes,
        )?;
        let current_id = observation.current.as_ref().map(|value| value.id.as_str());
        let current_label = observation
            .current
            .as_ref()
            .map(|value| value.label.as_str());
        let key_source = observation
            .api_key_source
            .as_ref()
            .map(|value| value.as_slice());
        let (conversation, launch) = observation.target.ids()?;
        operation!(self, |connection, dialect| {
            let ctx = connector!(connection, dialect, source);
            let binding = applied_binding!(
                connection,
                dialect,
                conversation,
                ctx.computer_id,
                launch,
                observation.target
            )
            .ok_or(StoreError::InvalidTransition)?;
            let key_required = match binding.kind.as_str() {
                "OPENAI_API_KEY" => true,
                "CHATGPT" => false,
                _ => return Err(StoreError::Integrity),
            };
            if key_required != key_source.is_some() {
                return Err(StoreError::InvalidInput);
            }
            let old = scope!(connection, dialect, conversation);
            if let Some(old) = &old
                && old.matches(&observation.target, launch)
            {
                if old.authentication_id != binding.authentication_id
                    || old.api_key_source.as_deref() != key_source
                {
                    return Err(StoreError::Conflict);
                }
                if old.observation_order >= observation.order {
                    return Ok(());
                }
            }
            let prior = old.filter(|row| {
                row.source_snapshot == observation.target.snapshot.as_str()
                    && row.native_session_id == observation.target.native_session.as_str()
                    && row.authentication_id == binding.authentication_id
                    && row.api_key_source.as_deref() == key_source
            });
            let recovery = prior
                .as_ref()
                .filter(|row| row.launch_operation_id == launch)
                .and_then(|row| row.recovery_operation_id);
            diesel::sql_query(dialect.query("INSERT INTO conversation_model_scopes (conversation_id,source_snapshot,native_session_id,harness_kind,authentication_id,api_key_source,launch_operation_id,observed_current_id,observed_current_label,advertised_options,observation_order,saved_model_id,saved_model_label,saved_request_id,recovery_operation_id,saved_tuning) VALUES (?,?,?,'CODEX',?,?,?,?,?,?,?,?,?,?,?,?) ON CONFLICT (conversation_id) DO UPDATE SET source_snapshot=excluded.source_snapshot,native_session_id=excluded.native_session_id,harness_kind=excluded.harness_kind,authentication_id=excluded.authentication_id,api_key_source=excluded.api_key_source,launch_operation_id=excluded.launch_operation_id,observed_current_id=excluded.observed_current_id,observed_current_label=excluded.observed_current_label,advertised_options=excluded.advertised_options,observation_order=excluded.observation_order,saved_model_id=excluded.saved_model_id,saved_model_label=excluded.saved_model_label,saved_request_id=excluded.saved_request_id,recovery_operation_id=excluded.recovery_operation_id,saved_tuning=excluded.saved_tuning"))
                .bind::<IdentitySql, _>(conversation).bind::<Text, _>(observation.target.snapshot.as_str())
                .bind::<Text, _>(observation.target.native_session.as_str()).bind::<IdentitySql, _>(binding.authentication_id)
                .bind::<Nullable<Binary>, _>(key_source).bind::<IdentitySql, _>(launch)
                .bind::<Nullable<Text>, _>(current_id).bind::<Nullable<Text>, _>(current_label)
                .bind::<Nullable<Text>, _>(&choices).bind::<BigInt, _>(observation.order)
                .bind::<Nullable<Text>, _>(prior.as_ref().and_then(|row| row.saved_model_id.as_deref()))
                .bind::<Nullable<Text>, _>(prior.as_ref().and_then(|row| row.saved_model_label.as_deref()))
                .bind::<Nullable<Text>, _>(prior.as_ref().and_then(|row| row.saved_request_id.as_deref()))
                .bind::<Nullable<IdentitySql>, _>(recovery)
                .bind::<Nullable<Text>, _>(prior.as_ref().and_then(|row| row.saved_tuning.as_deref()))
                .execute(connection).await?;
            Ok(())
        })
    }
}

fn row_target(row: &ScopeRow) -> Result<ConversationModelTarget, StoreError> {
    Ok(ConversationModelTarget {
        conversation_id: row.conversation_id.to_dto(EntityKind::Conversation),
        snapshot: SnapshotId::parse(row.source_snapshot.clone())
            .map_err(|_| StoreError::Integrity)?,
        native_session: NativeConversationId::parse(row.native_session_id.clone())
            .map_err(|_| StoreError::Integrity)?,
        launch_operation_id: row
            .launch_operation_id
            .to_dto(EntityKind::ConversationOperation),
    })
}

pub(super) mod starts;
