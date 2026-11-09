//! Integrated Store/service admission with a deliberately non-native processor.
use super::*;
use chapchoo_domain::{ConversationTarget, ConversationText};
use chapchoo_persistence::AcceptConversationInput;
use chapchoo_service::{
    AcceptedConversationInput, ConversationInputAcceptance, ConversationInputFuture,
    ConversationInputOwnership, ConversationInputProcessor, ConversationInputService,
};
use std::sync::atomic::AtomicUsize;
struct Processor {
    calls: Arc<AtomicUsize>,
    entered: Arc<tokio::sync::Semaphore>,
    release: Arc<tokio::sync::Semaphore>,
}
impl ConversationInputProcessor for Processor {
    fn process(&self, input: AcceptedConversationInput) -> ConversationInputFuture {
        let entered = self.entered.clone();
        let release = self.release.clone();
        let calls = self.calls.clone();
        Box::pin(async move {
            assert_eq!(
                input.receipt.text.as_str(),
                "Synthetic fixture input; never forwarded to a model"
            );
            assert_eq!(input.receipt.target, ConversationTarget::NotStarted);
            calls.fetch_add(1, Ordering::SeqCst);
            entered.add_permits(1);
            release
                .acquire()
                .await
                .map_err(chapchoo_service::ServiceError::infrastructure)?
                .forget();
            Ok(())
        })
    }
}
pub(super) async fn exercise(
    store: &Store,
    owner: &chapchoo_crypto::SecretDigest,
    agent: &chapchoo_persistence::AgentSummary,
) -> Result<ConversationInputAcceptance, TestError> {
    let calls = Arc::new(AtomicUsize::new(0));
    let entered = Arc::new(tokio::sync::Semaphore::new(0));
    let release = Arc::new(tokio::sync::Semaphore::new(0));
    let processor = Arc::new(Processor {
        calls: calls.clone(),
        entered: entered.clone(),
        release: release.clone(),
    });
    let service = ConversationInputService::new(store.clone(), processor.clone(), 4)?;
    let draft = AcceptConversationInput {
        agent_id: agent.id.clone(),
        conversation_id: None,
        request_id: "capture-fixture-input".into(),
        target: ConversationTarget::NotStarted,
        text: ConversationText::parse(
            "Synthetic fixture input; never forwarded to a model".into(),
        )?,
    };
    let accepted = service
        .accept(owner.clone(), draft.clone(), AcceptanceTime::System)
        .await?;
    // Acceptance returned while the processing owner is held at an explicit gate.
    entered.acquire().await?.forget();
    let status = service
        .input(
            owner,
            &agent.id,
            &accepted.conversation_id,
            &draft.request_id,
            AcceptanceTime::System,
        )
        .await?
        .ok_or("Missing accepted operation")?;
    assert_eq!(status.ownership, ConversationInputOwnership::ProcessingHere);
    let duplicate = service
        .accept(owner.clone(), draft.clone(), AcceptanceTime::System)
        .await?;
    assert_eq!(accepted.id, duplicate.id);
    let mut changed = draft.clone();
    changed.text = ConversationText::parse("changed accepted text".into())?;
    assert_eq!(
        service
            .accept(owner.clone(), changed, AcceptanceTime::System)
            .await
            .err()
            .ok_or("Changed payload accepted")?
            .reason(),
        chapchoo_service::DomainReason::Conflict
    );
    // A fresh server owner recovers persisted acceptance, not another process call.
    let recovered = ConversationInputService::new(store.clone(), processor, 4)?;
    assert_eq!(
        recovered
            .accept(owner.clone(), draft, AcceptanceTime::System)
            .await?
            .id,
        accepted.id
    );
    let observed = recovered
        .input(
            owner,
            &agent.id,
            &accepted.conversation_id,
            &accepted.request_id,
            AcceptanceTime::System,
        )
        .await?
        .ok_or("Missing recovered operation")?;
    assert_eq!(
        observed.ownership,
        ConversationInputOwnership::NotObservedHere
    );
    release.add_permits(4);
    service.finish().await?;
    recovered.finish().await?;
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    Ok(accepted)
}
