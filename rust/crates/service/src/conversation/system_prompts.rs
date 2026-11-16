use super::*;
use chapchoo_crypto::SecretDigest;
use chapchoo_domain::{InstructionSource, InstructionSourceKind, ProjectInstructions};
use chapchoo_persistence::{
    ConversationInputPreparation, ProjectInstructionEvidence, ProjectInstructionHistory,
    ResolvedRoute,
};

impl ConversationService {
    pub async fn reject_unsealed_preparation(
        &self,
        route: &ResolvedRoute,
        draft: &ConversationInputPreparation<'_>,
    ) -> Result<bool, ServiceError> {
        Ok(self.store.reject_unsealed_preparation(route, draft).await?)
    }
    pub async fn admit_project_refresh(
        &self,
        route: &ResolvedRoute,
        agent: &str,
        conversation: &str,
        instructions: &ProjectInstructions,
        tool_catalog: &[u8],
        now: i64,
    ) -> Result<Option<chapchoo_persistence::ConnectionStartupAdmission>, ServiceError> {
        Ok(self
            .store
            .admit_project_refresh(route, agent, conversation, instructions, tool_catalog, now)
            .await?)
    }
    pub async fn project_instructions_for_owner(
        &self,
        actor: &SecretDigest,
        agent: &str,
        now: i64,
    ) -> Result<ProjectInstructions, ServiceError> {
        selected(vec![InstructionSource {
            kind: InstructionSourceKind::Owner,
            text: self.store.system_prompt(actor, agent, now).await?,
        }])
    }
    pub async fn project_instructions_for_startup(
        &self,
        route: &ResolvedRoute,
        agent: &str,
        conversation: &str,
    ) -> Result<ProjectInstructions, ServiceError> {
        let owner = self
            .store
            .project_instructions_for_startup(route, agent, conversation)
            .await?;
        selected(owner.sources)
    }
    pub async fn seal_startup_project_instructions(
        &self,
        route: &ResolvedRoute,
        draft: &ConversationInputPreparation<'_>,
        instructions: &ProjectInstructions,
        tool_catalog: &[u8],
        now: i64,
    ) -> Result<ProjectInstructionEvidence, ServiceError> {
        Ok(self
            .store
            .seal_startup_project_instructions(route, draft, instructions, tool_catalog, now)
            .await?)
    }
    pub async fn seal_project_instructions(
        &self,
        actor: &SecretDigest,
        route: &ResolvedRoute,
        draft: &ConversationInputPreparation<'_>,
        instructions: &ProjectInstructions,
        tool_catalog: &[u8],
        now: i64,
    ) -> Result<ProjectInstructionEvidence, ServiceError> {
        Ok(self
            .store
            .seal_project_instructions(actor, route, draft, instructions, tool_catalog, now)
            .await?)
    }
    pub async fn project_instructions(
        &self,
        source: &chapchoo_persistence::ConnectorSource<'_>,
        agent: &str,
        conversation: &str,
        operation: &str,
    ) -> Result<ProjectInstructionEvidence, ServiceError> {
        Ok(self
            .store
            .project_instructions(source, agent, conversation, operation)
            .await?)
    }
    pub async fn instruction_evidence(
        &self,
        actor: &SecretDigest,
        agent: &str,
        conversation: &str,
        operation: &str,
        now: i64,
    ) -> Result<ProjectInstructionEvidence, ServiceError> {
        Ok(self
            .store
            .instruction_evidence(actor, agent, conversation, operation, now)
            .await?)
    }
    pub async fn instruction_history(
        &self,
        actor: &SecretDigest,
        agent: &str,
        conversation: &str,
        before_operation_id: Option<&str>,
        now: i64,
    ) -> Result<ProjectInstructionHistory, ServiceError> {
        Ok(self
            .store
            .instruction_history(actor, agent, conversation, before_operation_id, now)
            .await?)
    }
}

fn selected(owner_sources: Vec<InstructionSource>) -> Result<ProjectInstructions, ServiceError> {
    let mut sources = vec![
        InstructionSource {
            kind: InstructionSourceKind::Product,
            text: "Work in the Agent default project and preserve existing user files.".into(),
        },
        InstructionSource {
            kind: InstructionSourceKind::Function,
            text: "Use send_message to send text to the user. Ordinary model output is not delivered to the conversation.".into(),
        },
    ];
    sources.extend(owner_sources);
    ProjectInstructions::new(sources)
        .map_err(|_| ServiceError::domain(crate::DomainReason::InvalidInput))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_sources_preserve_owner_bytes_and_clear_only_owner_content()
    -> Result<(), Box<dyn std::error::Error>> {
        let owner = "  Exact owner 🙂\n\n";
        let selected = selected(vec![InstructionSource {
            kind: InstructionSourceKind::Owner,
            text: owner.into(),
        }])?;
        assert_eq!(selected.sources.len(), 3);
        assert_eq!(selected.sources[0].kind, InstructionSourceKind::Product);
        assert_eq!(selected.sources[1].kind, InstructionSourceKind::Function);
        assert_eq!(selected.sources[2].text, owner);
        let cleared = super::selected(vec![InstructionSource {
            kind: InstructionSourceKind::Owner,
            text: String::new(),
        }])?;
        assert_eq!(cleared.sources[..2], selected.sources[..2]);
        assert!(!cleared.document().contains("Owner instructions"));
        Ok(())
    }
}
