//! Private projection reads never dispatch native work or reveal raw envelopes.
use super::*;
use chapchoo_crypto::SecretDigest;
use chapchoo_persistence::{
    ConversationFieldChunkRead, ConversationFieldMetadata, ConversationFieldRead,
    ConversationRecordMetadata, ConversationRegionRead, EventPayloadChunk,
};
impl ConversationService {
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
