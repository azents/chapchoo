//! Internal processing uses the issued input and exact claim, not a token snapshot.
use super::*;
use chapchoo_domain::{InstructionSource, InstructionSourceKind, ProjectInstructions};
use chapchoo_persistence::{
    AgentSummary, ConnectionStartupObservation, ConversationForwardingAdmission,
    ConversationInputAuthority, ConversationInputPreparation, ConversationOperationForwarding,
    ConversationPreparationAdmission, ConversationPreparationFacts, ProjectInstructionEvidence,
    ResolvedRoute,
};

impl ConversationService {
    pub async fn queued_agent(
        &self,
        authority: &ConversationInputAuthority,
    ) -> Result<AgentSummary, ServiceError> {
        Ok(self.store.queued_conversation_agent(authority).await?)
    }
    pub async fn queued_owner_conversation(
        &self,
        authority: &ConversationInputAuthority,
    ) -> Result<Option<ConversationSummary>, ServiceError> {
        Ok(self.store.queued_current_conversation(authority).await?)
    }
    pub async fn queued_begin_setup(
        &self,
        authority: &ConversationInputAuthority,
        route: &ResolvedRoute,
        draft: &ConversationInputPreparation<'_>,
    ) -> Result<ConversationPreparationAdmission, ServiceError> {
        Ok(self
            .store
            .queued_begin_conversation_input_preparation(authority, route, draft)
            .await?)
    }
    pub async fn queued_setup_facts(
        &self,
        authority: &ConversationInputAuthority,
        draft: &ConversationInputPreparation<'_>,
    ) -> Result<Option<ConversationPreparationFacts>, ServiceError> {
        Ok(self
            .store
            .queued_conversation_input_preparation(authority, draft)
            .await?)
    }
    pub async fn queued_connection_startup(
        &self,
        authority: &ConversationInputAuthority,
    ) -> Result<Option<ConnectionStartupObservation>, ServiceError> {
        Ok(self.store.queued_connection_startup(authority).await?)
    }
    pub async fn queued_begin_input(
        &self,
        authority: &ConversationInputAuthority,
        route: &ResolvedRoute,
        draft: &ConversationOperationForwarding<'_>,
    ) -> Result<ConversationForwardingAdmission, ServiceError> {
        Ok(self
            .store
            .queued_begin_conversation_input_forwarding(authority, route, draft)
            .await?)
    }
    pub async fn queued_input_pending(
        &self,
        authority: &ConversationInputAuthority,
        execution: &str,
    ) -> Result<bool, ServiceError> {
        Ok(self
            .store
            .queued_execution_input_pending(authority, execution)
            .await?)
    }
    pub async fn queued_project_instructions(
        &self,
        authority: &ConversationInputAuthority,
    ) -> Result<ProjectInstructions, ServiceError> {
        let text = self
            .store
            .queued_system_prompt(authority)
            .await?
            .ok_or_else(|| ServiceError::domain(DomainReason::InvalidTransition))?;
        super::system_prompts::selected(vec![InstructionSource {
            kind: InstructionSourceKind::Owner,
            text,
        }])
    }
    pub async fn queued_seal_project_instructions(
        &self,
        authority: &ConversationInputAuthority,
        route: &ResolvedRoute,
        draft: &ConversationInputPreparation<'_>,
        instructions: &ProjectInstructions,
        catalog: &[u8],
    ) -> Result<ProjectInstructionEvidence, ServiceError> {
        Ok(self
            .store
            .seal_queued_project_instructions(authority, route, draft, instructions, catalog)
            .await?)
    }
    pub async fn queued_latest_system_prompt(
        &self,
        authority: &ConversationInputAuthority,
    ) -> Result<Option<ProjectInstructionEvidence>, ServiceError> {
        Ok(self.store.queued_latest_system_prompt(authority).await?)
    }
}
