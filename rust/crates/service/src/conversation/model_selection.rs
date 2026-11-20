use super::{ConnectorSource, ConversationService, ServiceError};
use crate::{
    ConversationModelObservation, ConversationModelProjection, ConversationModelSave,
    ConversationModelSaveReceipt, ConversationModelStart, ConversationModelStartAdmission,
    ConversationModelStartOutcome,
};
use chapchoo_crypto::SecretDigest;

impl ConversationService {
    pub async fn model(
        &self,
        actor: &SecretDigest,
        agent: &str,
        now: i64,
    ) -> Result<Option<ConversationModelProjection>, ServiceError> {
        Ok(self.store.conversation_model(actor, agent, now).await?)
    }

    pub async fn save_model(
        &self,
        actor: &SecretDigest,
        agent: &str,
        draft: &ConversationModelSave,
        now: i64,
    ) -> Result<ConversationModelSaveReceipt, ServiceError> {
        Ok(self
            .store
            .save_conversation_model(actor, agent, draft, now)
            .await?)
    }

    pub async fn model_save_result(
        &self,
        actor: &SecretDigest,
        agent: &str,
        request: &str,
        now: i64,
    ) -> Result<Option<ConversationModelSaveReceipt>, ServiceError> {
        Ok(self
            .store
            .conversation_model_save_result(actor, agent, request, now)
            .await?)
    }

    pub async fn report_model(
        &self,
        source: &ConnectorSource<'_>,
        observed: &ConversationModelObservation,
    ) -> Result<(), ServiceError> {
        Ok(self
            .store
            .observe_conversation_model(source, observed)
            .await?)
    }

    pub async fn reserve_model_start(
        &self,
        source: &ConnectorSource<'_>,
        draft: &ConversationModelStart,
    ) -> Result<ConversationModelStartAdmission, ServiceError> {
        Ok(self
            .store
            .reserve_conversation_model_start(source, draft)
            .await?)
    }

    pub async fn settle_model_start(
        &self,
        source: &ConnectorSource<'_>,
        draft: &ConversationModelStart,
        outcome: &ConversationModelStartOutcome,
    ) -> Result<(), ServiceError> {
        Ok(self
            .store
            .settle_conversation_model_start(source, draft, outcome)
            .await?)
    }
}
