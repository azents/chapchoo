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
    assert!(
        store
            .conversation_input_queue_state(&accepted.conversation_id, &accepted.id)
            .await?
            == chapchoo_persistence::ConversationInputQueueState::Error,
        "A processor that returns without delivery must close its attempt"
    );
    exercise_terminal(store, owner, agent).await?;
    exercise_retry(store, owner, agent).await?;
    Ok(accepted)
}

async fn exercise_retry(
    store: &Store,
    owner: &chapchoo_crypto::SecretDigest,
    agent: &chapchoo_persistence::AgentSummary,
) -> Result<(), TestError> {
    let calls = Arc::new(AtomicUsize::new(0));
    let entered = Arc::new(tokio::sync::Semaphore::new(0));
    let release = Arc::new(tokio::sync::Semaphore::new(0));
    let service = ConversationInputService::new(
        store.clone(),
        Arc::new(Processor {
            calls: calls.clone(),
            entered: entered.clone(),
            release: release.clone(),
        }),
        4,
    )?;
    let first = service
        .accept(
            owner.clone(),
            AcceptConversationInput {
                agent_id: agent.id.clone(),
                conversation_id: None,
                request_id: "retry-original".into(),
                target: ConversationTarget::NotStarted,
                text: ConversationText::parse(
                    "Synthetic fixture input; never forwarded to a model".into(),
                )?,
            },
            AcceptanceTime::System,
        )
        .await?;
    entered.acquire().await?.forget();
    release.add_permits(1);
    wait_failed(store, &first).await?;
    let draft = chapchoo_persistence::RetryConversationInput {
        agent_id: agent.id.clone(),
        conversation_id: first.conversation_id.clone(),
        original_input_id: first.id.clone(),
        original_request_id: first.request_id.clone(),
        target: ConversationTarget::NotStarted,
        request_id: "retry-fresh".into(),
    };
    let next = service
        .retry(owner.clone(), draft.clone(), AcceptanceTime::System)
        .await?;
    entered.acquire().await?.forget();
    assert_ne!(next.id, first.id);
    assert_eq!(
        service
            .retry(owner.clone(), draft, AcceptanceTime::System)
            .await?
            .id,
        next.id
    );
    assert_eq!(
        calls.load(Ordering::SeqCst),
        2,
        "Existing replacement must not dispatch"
    );
    store
        .fail_conversation_input(&first.conversation_id, &first.id)
        .await?;
    assert_eq!(
        store
            .conversation_input_queue_state(&next.conversation_id, &next.id)
            .await?,
        chapchoo_persistence::ConversationInputQueueState::Pending
    );
    release.add_permits(1);
    service.finish().await?;
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    Ok(())
}
async fn wait_failed(store: &Store, input: &ConversationInputAcceptance) -> Result<(), TestError> {
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        loop {
            if store
                .conversation_input_queue_state(&input.conversation_id, &input.id)
                .await?
                == chapchoo_persistence::ConversationInputQueueState::Error
            {
                return Ok::<_, TestError>(());
            }
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
    })
    .await??;
    Ok(())
}

struct TerminalProcessor(bool);
impl ConversationInputProcessor for TerminalProcessor {
    fn process(&self, _: AcceptedConversationInput) -> ConversationInputFuture {
        let panic = self.0;
        Box::pin(async move {
            assert!(!panic, "Injected consumer panic");
            Err(chapchoo_service::ServiceError::domain(
                chapchoo_service::DomainReason::InvalidTransition,
            ))
        })
    }
}
async fn exercise_terminal(
    store: &Store,
    owner: &chapchoo_crypto::SecretDigest,
    agent: &chapchoo_persistence::AgentSummary,
) -> Result<(), TestError> {
    for panic in [false, true] {
        let policy = chapchoo_service::ComputerPolicy {
            heartbeat_ms: 5,
            directory: chapchoo_persistence::DirectoryPolicy {
                component_validity_ms: 40,
                ..Default::default()
            },
            ..Default::default()
        };
        let service = ConversationInputService::new_with_policy(
            store.clone(),
            Arc::new(TerminalProcessor(panic)),
            4,
            policy,
        )?;
        let accepted = service
            .accept(
                owner.clone(),
                AcceptConversationInput {
                    agent_id: agent.id.clone(),
                    conversation_id: None,
                    request_id: format!("terminal-{panic}"),
                    target: ConversationTarget::NotStarted,
                    text: ConversationText::parse("Synthetic terminal consumer".into())?,
                },
                AcceptanceTime::System,
            )
            .await?;
        // Poll the authoritative state. Timeout bounds a hang; it does not
        // establish the terminal transition or hide the retained processor error.
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            loop {
                if store
                    .conversation_input_queue_state(&accepted.conversation_id, &accepted.id)
                    .await?
                    == chapchoo_persistence::ConversationInputQueueState::Error
                {
                    return Ok::<_, TestError>(());
                }
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            }
        })
        .await??;
        assert!(
            service.finish().await.is_err(),
            "Retain the original failure"
        );
    }
    Ok(())
}
