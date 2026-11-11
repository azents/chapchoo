//! Typed native normalization and private conversation orchestration.
mod answer_owner;
mod answers;
pub use answer_owner::{
    AcceptedConversationAnswer, ConversationAnswerAcceptance, ConversationAnswerDraft,
    ConversationAnswerFuture, ConversationAnswerProcessor, ConversationAnswerService,
};
mod input_owner;
mod input_work;
mod records;
mod stop_owner;
pub use stop_owner::{
    AcceptedConversationStop, ConversationStopFuture, ConversationStopProcessor,
    ConversationStopService,
};
mod normalize;
use crate::{DomainReason, ServiceError};
pub use answers::{
    EncodedNativeAnswer, NativeAnswerError, encode_native_answer, validate_conversation_answer,
};
use chapchoo_domain::{SnapshotId, SourcePosition};
use chapchoo_persistence::{ConnectorSource, ConversationSummary, Store};
pub use input_owner::{
    AcceptedConversationInput, ConversationInputAcceptance, ConversationInputFuture,
    ConversationInputOwnership, ConversationInputProcessor, ConversationInputService,
    ConversationInputStatus,
};
pub use normalize::{NativeNormalizationError, NormalizedCapture, normalize_captured_event};
use std::fmt;

#[derive(Clone)]
pub struct ConversationService {
    store: Store,
}
impl fmt::Debug for ConversationService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ConversationService")
    }
}
impl ConversationService {
    /// Owner read only: selecting or inspecting authentication never acquires native credentials.
    pub async fn saved_authentication_id(
        &self,
        actor: &chapchoo_crypto::SecretDigest,
        agent: &str,
        now: i64,
    ) -> Result<Option<String>, ServiceError> {
        Ok(self
            .store
            .agent(actor, agent, now)
            .await?
            .authentication
            .map(|selection| selection.authentication_id))
    }
    pub async fn request_answer(
        &self,
        actor: &chapchoo_crypto::SecretDigest,
        agent: &str,
        conversation: &str,
        snapshot: &SnapshotId,
        occurrence: &chapchoo_domain::EventOccurrenceId,
        now: i64,
    ) -> Result<Option<ConversationAnswerAcceptance>, ServiceError> {
        Ok(self
            .store
            .conversation_request_answer(actor, agent, conversation, snapshot, occurrence, now)
            .await?
            .map(Into::into))
    }
    pub async fn retained_answer(
        &self,
        actor: &chapchoo_crypto::SecretDigest,
        agent: &str,
        conversation: &str,
        snapshot: &SnapshotId,
        operation: &str,
        now: i64,
    ) -> Result<chapchoo_persistence::ConversationAnswer, ServiceError> {
        Ok(self
            .store
            .conversation_retained_answer(actor, agent, conversation, snapshot, operation, now)
            .await?)
    }
    pub async fn live_requests(
        &self,
        actor: &chapchoo_crypto::SecretDigest,
        agent: &str,
        conversation: &str,
        target: &chapchoo_persistence::ConversationDeliveryTarget,
        now: i64,
    ) -> Result<Vec<chapchoo_persistence::ConversationLiveRequestReference>, ServiceError> {
        Ok(self
            .store
            .conversation_live_requests(actor, agent, conversation, target, now)
            .await?)
    }
    pub async fn begin_answer(
        &self,
        actor: &chapchoo_crypto::SecretDigest,
        route: &chapchoo_persistence::ResolvedRoute,
        draft: &chapchoo_persistence::ConversationOperationForwarding<'_>,
        now: i64,
    ) -> Result<chapchoo_persistence::ConversationAnswerForwardingAdmission, ServiceError> {
        Ok(self
            .store
            .begin_conversation_answer_forwarding(actor, route, draft, now)
            .await?)
    }
    pub async fn answer_chunk(
        &self,
        source: &ConnectorSource<'_>,
        draft: &chapchoo_persistence::ConversationOperationForwarding<'_>,
        offset: u32,
    ) -> Result<chapchoo_persistence::ConversationAnswerChunk, ServiceError> {
        Ok(self
            .store
            .connector_answer_chunk(source, draft, offset)
            .await?)
    }
    pub async fn report_answer(
        &self,
        source: &ConnectorSource<'_>,
        draft: &chapchoo_persistence::ConversationOperationForwarding<'_>,
        result: chapchoo_persistence::ConversationAnswerResult,
    ) -> Result<(), ServiceError> {
        Ok(self
            .store
            .complete_conversation_answer_forwarding(source, draft, result)
            .await?)
    }
    pub async fn native_observation(
        &self,
        actor: &chapchoo_crypto::SecretDigest,
        agent: &str,
        conversation: &str,
        now: i64,
    ) -> Result<Option<chapchoo_persistence::ConversationNativeObservation>, ServiceError> {
        Ok(self
            .store
            .conversation_native_observation(actor, agent, conversation, now)
            .await?)
    }
    pub async fn saved_supply(
        &self,
        actor: &chapchoo_crypto::SecretDigest,
        agent: &str,
        now: i64,
    ) -> Result<chapchoo_persistence::HarnessSupplySelection, ServiceError> {
        Ok(self.store.harness_supply(actor, agent, now).await?)
    }
    pub async fn save_supply(
        &self,
        actor: &chapchoo_crypto::SecretDigest,
        draft: &chapchoo_persistence::SaveHarnessSupply,
        now: i64,
    ) -> Result<chapchoo_persistence::HarnessSupplyReceipt, ServiceError> {
        Ok(self.store.save_harness_supply(actor, draft, now).await?)
    }
    pub async fn check_stop(
        &self,
        source: &ConnectorSource<'_>,
        conversation: &str,
        operation: &str,
        target: &chapchoo_persistence::ConversationWorkTarget,
    ) -> Result<(), ServiceError> {
        Ok(self
            .store
            .check_conversation_stop(source, conversation, operation, target)
            .await?)
    }
    pub async fn report_stop(
        &self,
        source: &ConnectorSource<'_>,
        conversation: &str,
        operation: &str,
        target: &chapchoo_persistence::ConversationWorkTarget,
        published: bool,
    ) -> Result<(), ServiceError> {
        Ok(self
            .store
            .complete_conversation_stop(source, conversation, operation, target, published)
            .await?)
    }
    pub async fn observe_native(
        &self,
        source: &ConnectorSource<'_>,
        conversation: &str,
        observation: &chapchoo_persistence::ConversationNativeObservation,
    ) -> Result<(), ServiceError> {
        Ok(self
            .store
            .observe_conversation_native(source, conversation, observation)
            .await?)
    }
    pub async fn observe_request(
        &self,
        source: &ConnectorSource<'_>,
        conversation: &str,
        target: &chapchoo_persistence::ConversationDeliveryTarget,
        request: &chapchoo_persistence::LiveConversationRequest,
    ) -> Result<(), ServiceError> {
        Ok(self
            .store
            .observe_conversation_request(source, conversation, target, request)
            .await?)
    }
    pub async fn begin_setup(
        &self,
        actor: &chapchoo_crypto::SecretDigest,
        route: &chapchoo_persistence::ResolvedRoute,
        draft: &chapchoo_persistence::ConversationInputPreparation<'_>,
        now: i64,
    ) -> Result<chapchoo_persistence::ConversationPreparationAdmission, ServiceError> {
        Ok(self
            .store
            .begin_conversation_input_preparation(actor, route, draft, now)
            .await?)
    }
    pub async fn connection_startup(
        &self,
        actor: &chapchoo_crypto::SecretDigest,
        agent_id: &str,
        now: i64,
    ) -> Result<Option<chapchoo_persistence::ConnectionStartupObservation>, ServiceError> {
        Ok(self.store.connection_startup(actor, agent_id, now).await?)
    }
    pub async fn admit_connection_startup(
        &self,
        computer_id: &str,
        route: &chapchoo_persistence::ResolvedRoute,
    ) -> Result<Vec<chapchoo_persistence::ConnectionStartupAdmission>, ServiceError> {
        Ok(self
            .store
            .admit_connection_startup(computer_id, route)
            .await?)
    }
    pub async fn connection_startup_may_dispatch(
        &self,
        computer_id: &str,
        route: &chapchoo_persistence::ResolvedRoute,
        operation_id: &str,
    ) -> Result<bool, ServiceError> {
        Ok(self
            .store
            .connection_startup_may_dispatch(computer_id, route, operation_id)
            .await?)
    }
    pub async fn setup_facts(
        &self,
        actor: &chapchoo_crypto::SecretDigest,
        draft: &chapchoo_persistence::ConversationInputPreparation<'_>,
        now: i64,
    ) -> Result<Option<chapchoo_persistence::ConversationPreparationFacts>, ServiceError> {
        Ok(self
            .store
            .conversation_input_preparation(actor, draft, now)
            .await?)
    }
    pub async fn owner_conversation(
        &self,
        actor: &chapchoo_crypto::SecretDigest,
        agent_id: &str,
        now: i64,
    ) -> Result<Option<ConversationSummary>, ServiceError> {
        Ok(self
            .store
            .current_conversation(actor, agent_id, now)
            .await?)
    }
    pub async fn begin_input(
        &self,
        actor: &chapchoo_crypto::SecretDigest,
        route: &chapchoo_persistence::ResolvedRoute,
        draft: &chapchoo_persistence::ConversationOperationForwarding<'_>,
        now: i64,
    ) -> Result<chapchoo_persistence::ConversationForwardingAdmission, ServiceError> {
        Ok(self
            .store
            .begin_conversation_input_forwarding(actor, route, draft, now)
            .await?)
    }
    pub async fn input_chunk(
        &self,
        source: &ConnectorSource<'_>,
        draft: &chapchoo_persistence::ConversationOperationForwarding<'_>,
        offset: u32,
    ) -> Result<chapchoo_persistence::ConversationInputChunk, ServiceError> {
        Ok(self
            .store
            .connector_input_chunk(source, draft, offset)
            .await?)
    }
    pub async fn report_input(
        &self,
        source: &ConnectorSource<'_>,
        draft: &chapchoo_persistence::ConversationOperationForwarding<'_>,
        outcome: &chapchoo_persistence::ConversationInputOutcome,
    ) -> Result<(), ServiceError> {
        Ok(self
            .store
            .complete_conversation_input_forwarding(source, draft, outcome)
            .await?)
    }
    pub async fn check_setup(
        &self,
        source: &ConnectorSource<'_>,
        draft: &chapchoo_persistence::ConversationInputPreparation<'_>,
        supply: &chapchoo_domain::HarnessSupply,
        fresh: Option<(bool, bool)>,
        target: Option<&chapchoo_persistence::ConversationDeliveryTarget>,
    ) -> Result<(), ServiceError> {
        Ok(self
            .store
            .check_conversation_input_preparation(source, draft, supply, fresh, target)
            .await?)
    }
    pub async fn report_setup(
        &self,
        source: &ConnectorSource<'_>,
        draft: &chapchoo_persistence::ConversationInputPreparation<'_>,
        outcome: &chapchoo_persistence::ConversationPreparationOutcome,
    ) -> Result<(), ServiceError> {
        Ok(self
            .store
            .complete_conversation_input_preparation(source, draft, outcome)
            .await?)
    }
    pub fn new(store: Store) -> Self {
        Self { store }
    }
    pub async fn current_captured_source(
        &self,
        source: &ConnectorSource<'_>,
    ) -> Result<Option<ConversationSummary>, ServiceError> {
        Ok(self.store.connector_current_conversation(source).await?)
    }
    pub async fn captured_source(
        &self,
        source: &ConnectorSource<'_>,
        conversation_id: &str,
    ) -> Result<ConversationSummary, ServiceError> {
        Ok(self
            .store
            .connector_conversation(source, conversation_id)
            .await?)
    }
    pub async fn bind_capture(
        &self,
        source: &ConnectorSource<'_>,
        binding: &chapchoo_persistence::SourceBinding,
    ) -> Result<ConversationSummary, ServiceError> {
        Ok(self.store.bind_conversation_source(source, binding).await?)
    }
    pub async fn complete_capture(
        &self,
        source: &ConnectorSource<'_>,
        conversation_id: &str,
        snapshot: &SnapshotId,
        boundary: Option<&SourcePosition>,
    ) -> Result<ConversationSummary, ServiceError> {
        Ok(self
            .store
            .complete_conversation_source(source, conversation_id, snapshot, boundary)
            .await?)
    }
    /// Pure decoding occurs outside DB authority locks; the completed Store
    /// publication revalidates the exact current source before all writes.
    pub async fn ingest(
        &self,
        source: &ConnectorSource<'_>,
        conversation_id: &str,
        snapshot: &SnapshotId,
        position: &SourcePosition,
        raw: &[u8],
    ) -> Result<ConversationSummary, ServiceError> {
        let current = self
            .store
            .connector_conversation(source, conversation_id)
            .await?;
        if current.source.snapshot() != Some(snapshot) {
            return Err(ServiceError::domain(DomainReason::Conflict));
        }
        let native = current
            .native_session_id
            .ok_or_else(|| ServiceError::domain(DomainReason::InvalidTransition))?;
        let normalized = normalize_captured_event(snapshot, position, &native, raw)
            .map_err(|_| ServiceError::domain(DomainReason::InvalidInput))?;
        Ok(self
            .store
            .ingest_conversation_records(
                source,
                conversation_id,
                &normalized.pair,
                &normalized.records,
            )
            .await?)
    }
}
