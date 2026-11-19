//! Message-only reads and receipt writes retain their completed Store boundary.
use super::ConversationService;
use crate::ServiceError;
use chapchoo_crypto::SecretDigest;
use chapchoo_domain::SendAction;
use chapchoo_persistence::{
    ConnectorSource, ConversationMessageChunk, ConversationMessageRead, ConversationMessageReceipt,
    ConversationMessageRegion, ConversationMessageScope, ConversationMessageWindow,
    ConversationWorkSummary,
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

    pub async fn send_work(
        &self,
        source: &ConnectorSource<'_>,
        scope: &ConversationMessageScope<'_>,
        action: &SendAction,
    ) -> Result<Option<ConversationMessageReceipt>, ServiceError> {
        Ok(self
            .store
            .send_conversation_work(source, scope, action)
            .await?)
    }

    pub async fn work_summary(
        &self,
        actor: &SecretDigest,
        scope: &ConversationMessageScope<'_>,
        now: i64,
    ) -> Result<Option<ConversationWorkSummary>, ServiceError> {
        Ok(self
            .store
            .conversation_work_summary(actor, scope, now)
            .await?)
    }
}
