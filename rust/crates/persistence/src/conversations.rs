//! Private conversation storage; no native effects or credential payloads belong here.

mod answers;
pub(crate) mod authority;
mod delivery;
mod input_reads;
mod inputs;
mod native_state;
mod preparation;
mod projection;
mod records;
mod startup;
mod stops;
mod supply;

use crate::{
    Store, StoreError,
    entity_id::{EntityId, EntityKind, IdentitySql},
    store::{Dialect, operation},
};
use chapchoo_crypto::SecretDigest;
use chapchoo_domain::{
    ConversationSource, ConversationTarget, ConversationText, EventOccurrenceId, ExistingCodexPath,
    HarnessSupply, NativeConversationId, SnapshotId, SourcePosition,
};
use diesel::{
    OptionalExtension,
    sql_types::{BigInt, Binary, Nullable, Text},
};
use diesel_async::RunQueryDsl;
use std::fmt;

pub use answers::{
    AcceptConversationAnswer, ConversationAnswer, ConversationAnswerAdmission,
    ConversationAnswerChunk, ConversationAnswerDelivery, ConversationAnswerForwardingAdmission,
    ConversationAnswerQuestion, ConversationAnswerReceipt, ConversationAnswerReference,
    ConversationAnswerResult, ConversationAnswerValue, ConversationFormAnswer,
    ConversationLiveRequestReference, ConversationRequestKind, LiveConversationRequest,
};
use authority::{connector, owned_agent};
pub use delivery::{
    ConversationDeliveryTarget, ConversationForwardingAdmission, ConversationForwardingGrant,
    ConversationInputChunk, ConversationInputDelivery, ConversationInputOutcome,
    ConversationOperationForwarding,
};
pub use input_reads::{ConversationInputAnchor, ConversationInputScope, RetainedConversationInput};
pub use inputs::{
    AcceptConversationInput, ConversationInputAdmission, ConversationInputReceipt,
    ConversationInputReference,
};
pub use native_state::{
    ConversationForeground, ConversationNativeObservation, ConversationNativePreparation,
    ConversationPreparationFreshness,
};
pub use preparation::{
    ConversationInputPreparation, ConversationPreparationAdmission, ConversationPreparationFacts,
    ConversationPreparationFailure, ConversationPreparationOutcome, ConversationPreparationState,
};
pub use projection::{
    ConversationFieldChange, ConversationFieldChunkRead, ConversationFieldMetadata,
    ConversationFieldRead, ConversationFieldUpdate, ConversationRecordAnchor,
    ConversationRecordField, ConversationRecordKind, ConversationRecordMetadata,
    ConversationRecordUpdate, ConversationRegionRead,
};
pub use records::{
    CapturedEventPair, EventChunkRead, EventMetadataRead, EventPairMetadata, EventPayloadChunk,
    EventPayloadKind, SourceBinding,
};
pub use startup::{
    ConnectionStartupAdmission, ConnectionStartupContext, ConnectionStartupObservation,
};
pub use stops::{
    AcceptConversationStop, ConversationStopAdmission, ConversationStopReceipt,
    ConversationStopState, ConversationWorkTarget,
};
pub use supply::{HarnessSupplyReceipt, HarnessSupplySelection, SaveHarnessSupply};

pub struct ConversationSummary {
    pub id: String,
    pub agent_id: String,
    pub native_session_id: Option<NativeConversationId>,
    pub source: ConversationSource,
    pub created_at: i64,
}
impl fmt::Debug for ConversationSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ConversationSummary([REDACTED])")
    }
}

#[derive(diesel::QueryableByName)]
pub(super) struct ConversationRow {
    #[diesel(sql_type=IdentitySql)]
    id: EntityId,
    #[diesel(sql_type=IdentitySql)]
    agent_id: EntityId,
    #[diesel(sql_type=Nullable<Text>)]
    native_session_id: Option<String>,
    #[diesel(sql_type=Nullable<Text>)]
    source_snapshot: Option<String>,
    #[diesel(sql_type=Text)]
    source_state: String,
    #[diesel(sql_type=BigInt)]
    source_ordinal: i64,
    #[diesel(sql_type=Nullable<Text>)]
    source_event_id: Option<String>,
    #[diesel(sql_type=BigInt)]
    created_at: i64,
}
impl ConversationRow {
    fn source(&self) -> Result<ConversationSource, StoreError> {
        let position = match (self.source_ordinal, &self.source_event_id) {
            (0, None) => None,
            (ordinal, Some(id)) => Some(
                SourcePosition::new(
                    ordinal,
                    EventOccurrenceId::parse(id.clone()).map_err(|_| StoreError::Integrity)?,
                )
                .map_err(|_| StoreError::Integrity)?,
            ),
            _ => return Err(StoreError::Integrity),
        };
        match (&self.source_snapshot, self.source_state.as_str()) {
            (None, "NOT_STARTED") if position.is_none() && self.native_session_id.is_none() => {
                Ok(ConversationSource::NotStarted)
            }
            (Some(snapshot), state @ ("LOADING" | "READY")) if self.native_session_id.is_some() => {
                let snapshot =
                    SnapshotId::parse(snapshot.clone()).map_err(|_| StoreError::Integrity)?;
                if state == "LOADING" {
                    Ok(ConversationSource::Loading { snapshot, position })
                } else {
                    Ok(ConversationSource::Ready { snapshot, position })
                }
            }
            _ => Err(StoreError::Integrity),
        }
    }
    fn summary(&self) -> Result<ConversationSummary, StoreError> {
        Ok(ConversationSummary {
            id: self.id.to_dto(EntityKind::Conversation),
            agent_id: self.agent_id.to_dto(EntityKind::Agent),
            native_session_id: self
                .native_session_id
                .clone()
                .map(NativeConversationId::parse)
                .transpose()
                .map_err(|_| StoreError::Integrity)?,
            source: self.source()?,
            created_at: self.created_at,
        })
    }
}

fn request_id(value: &str) -> Result<(), StoreError> {
    if value.is_empty() || value.len() > 128 || !value.bytes().all(|b| b.is_ascii_graphic()) {
        return Err(StoreError::InvalidInput);
    }
    Ok(())
}

macro_rules! conversation {
    ($connection:expr, $dialect:expr, $id:expr) => {{
        diesel::sql_query($dialect.query("SELECT * FROM conversations WHERE id=?"))
            .bind::<IdentitySql, _>($id)
            .get_result::<ConversationRow>($connection)
            .await
            .optional()?
            .ok_or(StoreError::NotFound)?
    }};
}
use conversation;

impl Store {
    /// Reading does not create a conversation or start a Computer/harness.
    pub async fn current_conversation(
        &self,
        actor: &SecretDigest,
        agent_id: &str,
        now: i64,
    ) -> Result<Option<ConversationSummary>, StoreError> {
        let agent = EntityId::from_dto(EntityKind::Agent, agent_id)?;
        operation!(self, |connection, dialect| {
            let _ = owned_agent!(connection, dialect, actor, agent, now);
            diesel::sql_query(dialect.query("SELECT c.* FROM conversations c JOIN agent_conversations a ON a.conversation_id=c.id AND a.agent_id=c.agent_id WHERE a.agent_id=?"))
                .bind::<IdentitySql,_>(agent).get_result::<ConversationRow>(connection)
                .await.optional()?.map(|row| row.summary()).transpose()
        })
    }
}
