//! Message-only reads and receipt writes retain their completed Store boundary.
use super::ConversationService;
use crate::ServiceError;
use chapchoo_crypto::SecretDigest;
use chapchoo_persistence::{
    ConnectorSource, ConversationMessageChunk, ConversationMessageRead, ConversationMessageReceipt,
    ConversationMessageRegion, ConversationMessageScope, ConversationMessageWindow,
    PublishConversationMessage,
};

impl ConversationService {
    pub async fn check_tool_scope(
        &self,
        source: &ConnectorSource<'_>,
        scope: &ConversationMessageScope<'_>,
    ) -> Result<(), ServiceError> {
        Ok(self
            .store
            .check_conversation_tool_scope(source, scope)
            .await?)
    }

    pub async fn messages(
        &self,
        actor: &SecretDigest,
        scope: &ConversationMessageScope<'_>,
        window: ConversationMessageWindow,
        limit: i64,
        now: i64,
    ) -> Result<ConversationMessageRegion, ServiceError> {
        Ok(self
            .store
            .conversation_messages(actor, scope, window, limit, now)
            .await?)
    }

    pub async fn message_chunk(
        &self,
        actor: &SecretDigest,
        read: &ConversationMessageRead<'_>,
        now: i64,
    ) -> Result<ConversationMessageChunk, ServiceError> {
        Ok(self
            .store
            .conversation_message_chunk(actor, read, now)
            .await?)
    }

    pub async fn publish_message(
        &self,
        source: &ConnectorSource<'_>,
        draft: &PublishConversationMessage<'_>,
    ) -> Result<ConversationMessageReceipt, ServiceError> {
        Ok(self
            .store
            .publish_conversation_message(source, draft)
            .await?)
    }
}
