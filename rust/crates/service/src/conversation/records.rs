//! Private projection reads never dispatch native work or reveal raw envelopes.
use super::*;
use chapchoo_crypto::SecretDigest;
use chapchoo_persistence::{
    ConversationFieldChunkRead, ConversationFieldMetadata, ConversationFieldRead,
    ConversationRecordMetadata, ConversationRegionRead, EventPayloadChunk,
};
impl ConversationService {
    pub async fn closed_inputs(
        &self,
        actor: &SecretDigest,
        scope: &chapchoo_persistence::ConversationInputScope<'_>,
        known: &[String],
        now: i64,
    ) -> Result<Vec<String>, ServiceError> {
        Ok(self
            .store
            .closed_conversation_inputs(actor, scope, known, now)
            .await?)
    }
    pub async fn latest_plan(
        &self,
        actor: &SecretDigest,
        read: &ConversationRegionRead<'_>,
        now: i64,
    ) -> Result<Vec<ConversationRecordMetadata>, ServiceError> {
        Ok(self
            .store
            .conversation_latest_plan(actor, read, now)
            .await?)
    }
    pub async fn inputs(
        &self,
        actor: &SecretDigest,
        scope: &chapchoo_persistence::ConversationInputScope<'_>,
        before: Option<&chapchoo_persistence::ConversationInputAnchor>,
        limit: i64,
        now: i64,
    ) -> Result<Vec<chapchoo_persistence::RetainedConversationInput>, ServiceError> {
        Ok(self
            .store
            .conversation_inputs(actor, scope, before, limit, now)
            .await?)
    }
    pub async fn latest_input(
        &self,
        actor: &SecretDigest,
        scope: &chapchoo_persistence::ConversationInputScope<'_>,
        now: i64,
    ) -> Result<Option<chapchoo_persistence::RetainedConversationInput>, ServiceError> {
        Ok(self
            .store
            .latest_conversation_input(actor, scope, now)
            .await?)
    }
    pub async fn input_text(
        &self,
        actor: &SecretDigest,
        scope: &chapchoo_persistence::ConversationInputScope<'_>,
        request: &str,
        offset: u32,
        length: u32,
        now: i64,
    ) -> Result<EventPayloadChunk, ServiceError> {
        Ok(self
            .store
            .conversation_input_text(actor, scope, request, offset, length, now)
            .await?)
    }
    pub async fn delete_input(
        &self,
        actor: &SecretDigest,
        scope: &chapchoo_persistence::ConversationInputScope<'_>,
        request: &str,
        now: i64,
    ) -> Result<(), ServiceError> {
        Ok(self
            .store
            .delete_failed_conversation_input(actor, scope, request, now)
            .await?)
    }
    pub async fn records(
        &self,
        actor: &SecretDigest,
        read: &ConversationRegionRead<'_>,
        now: i64,
    ) -> Result<Vec<ConversationRecordMetadata>, ServiceError> {
        Ok(self.store.conversation_region(actor, read, now).await?)
    }
    pub async fn fields(
        &self,
        actor: &SecretDigest,
        read: &ConversationFieldRead<'_>,
        now: i64,
    ) -> Result<Vec<ConversationFieldMetadata>, ServiceError> {
        Ok(self.store.conversation_fields(actor, read, now).await?)
    }
    pub async fn field_chunk(
        &self,
        actor: &SecretDigest,
        read: &ConversationFieldChunkRead<'_>,
        now: i64,
    ) -> Result<EventPayloadChunk, ServiceError> {
        Ok(self
            .store
            .conversation_field_chunk(actor, read, now)
            .await?)
    }
}
