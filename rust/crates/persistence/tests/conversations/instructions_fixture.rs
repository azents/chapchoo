use super::*;
use chapchoo_domain::{InstructionSource, InstructionSourceKind, ProjectInstructions};
use chapchoo_persistence::{
    ConversationInputPreparation, ConversationOperationForwarding,
    ConversationPreparationAdmission, ConversationPreparationOutcome, ResolvedRoute,
};

/// Establish actual preparation and applied evidence before a new input effect.
pub(super) async fn apply(
    store: &Store,
    actor: &chapchoo_crypto::SecretDigest,
    route: &ResolvedRoute,
    source: &ConnectorSource<'_>,
    operation: &ConversationOperationForwarding<'_>,
    now: i64,
) -> Result<(), StoreError> {
    let draft = ConversationInputPreparation {
        agent_id: operation.agent_id,
        conversation_id: operation.conversation_id,
        operation_id: operation.operation_id,
    };
    let admission = store
        .begin_conversation_input_preparation(actor, route, &draft, now)
        .await?;
    if matches!(
        admission,
        ConversationPreparationAdmission::ObservedStartup { .. }
    ) {
        return Ok(());
    }
    let owner = store.system_prompt(actor, operation.agent_id, now).await?;
    let instructions = ProjectInstructions::new(vec![
        InstructionSource {
            kind: InstructionSourceKind::Product,
            text: "Preserve existing user files.".into(),
        },
        InstructionSource {
            kind: InstructionSourceKind::Owner,
            text: owner,
        },
    ])
    .map_err(|_| StoreError::InvalidInput)?;
    store
        .seal_project_instructions(actor, route, &draft, &instructions, br#"{"tools":[]}"#, now)
        .await?;
    store
        .complete_conversation_input_preparation(
            source,
            &draft,
            &ConversationPreparationOutcome::SessionReady(operation.target.clone()),
        )
        .await
}
