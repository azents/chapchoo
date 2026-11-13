//! Short product acceptance and process-owned input processing are separate lifetimes.
use super::{input_work::InputWork, *};
use crate::AcceptanceTime;
use chapchoo_crypto::SecretDigest;
use chapchoo_domain::ConversationTarget;
use chapchoo_persistence::{
    AcceptConversationInput, ConversationInputAdmission, ConversationInputQueueState,
    ConversationInputReceipt, RetryConversationInput,
};
use std::{future::Future, pin::Pin, sync::Arc};

#[derive(Clone)]
pub struct ConversationInputAcceptance {
    pub id: String,
    pub conversation_id: String,
    pub request_id: String,
    pub target: ConversationTarget,
    pub accepted_at: i64,
}
impl From<&ConversationInputReceipt> for ConversationInputAcceptance {
    fn from(receipt: &ConversationInputReceipt) -> Self {
        Self {
            id: receipt.id.clone(),
            conversation_id: receipt.conversation_id.clone(),
            request_id: receipt.request_id.clone(),
            target: receipt.target.clone(),
            accepted_at: receipt.accepted_at,
        }
    }
}
impl From<chapchoo_persistence::ConversationInputReference> for ConversationInputAcceptance {
    fn from(receipt: chapchoo_persistence::ConversationInputReference) -> Self {
        Self {
            id: receipt.id,
            conversation_id: receipt.conversation_id,
            request_id: receipt.request_id,
            target: receipt.target,
            accepted_at: receipt.accepted_at,
        }
    }
}
pub struct AcceptedConversationInput {
    pub actor: SecretDigest,
    pub agent_id: String,
    pub receipt: ConversationInputReceipt,
}
enum Submission {
    New(AcceptConversationInput),
    Retry(RetryConversationInput),
}
impl Submission {
    fn agent_id(&self) -> &str {
        match self {
            Self::New(value) => &value.agent_id,
            Self::Retry(value) => &value.agent_id,
        }
    }
}
pub type ConversationInputFuture =
    Pin<Box<dyn Future<Output = Result<(), ServiceError>> + Send + 'static>>;
/// The composition supplies exact Control routing. Processing records durable
/// preparation/forwarding/result evidence before its native boundaries and
/// revalidates current authority there. It never waits for model completion.
pub trait ConversationInputProcessor: Send + Sync {
    fn process(&self, input: AcceptedConversationInput) -> ConversationInputFuture;
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversationInputOwnership {
    ProcessingHere,
    /// Missing local ownership is not proof of non-delivery and grants no replay.
    NotObservedHere,
}
pub struct ConversationInputStatus {
    pub acceptance: ConversationInputAcceptance,
    pub ownership: ConversationInputOwnership,
    pub delivery: Option<chapchoo_persistence::ConversationInputDelivery>,
    pub failed: bool,
}
#[derive(Clone)]
pub struct ConversationInputService {
    store: Store,
    processor: Arc<dyn ConversationInputProcessor>,
    work: Arc<InputWork>,
    consumer: Arc<super::input_consumer::Consumer>,
}
macro_rules! safe_debug {($($name:ty),+)=>{$(impl fmt::Debug for $name {
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result {f.write_str(concat!(stringify!($name),"([REDACTED])"))}
})+};}
safe_debug!(
    ConversationInputAcceptance,
    AcceptedConversationInput,
    ConversationInputStatus,
    ConversationInputService
);
impl ConversationInputService {
    pub fn new(
        store: Store,
        processor: Arc<dyn ConversationInputProcessor>,
        capacity: usize,
    ) -> Result<Self, ServiceError> {
        Self::new_with_policy(store, processor, capacity, crate::ComputerPolicy::default())
    }
    pub fn new_with_policy(
        store: Store,
        processor: Arc<dyn ConversationInputProcessor>,
        capacity: usize,
        policy: crate::ComputerPolicy,
    ) -> Result<Self, ServiceError> {
        let work = Arc::new(InputWork::new(capacity)?);
        let consumer = Arc::new(super::input_consumer::Consumer::new(store.clone(), policy)?);
        Ok(Self {
            store,
            processor,
            work,
            consumer,
        })
    }
    /// Capacity and task ownership precede database acceptance. The response
    /// contains a small reference, never the full submitted body or native result.
    pub async fn accept(
        &self,
        actor: SecretDigest,
        draft: AcceptConversationInput,
        clock: AcceptanceTime,
    ) -> Result<ConversationInputAcceptance, ServiceError> {
        let bytes = draft.text.as_str().len();
        self.enqueue(actor, Submission::New(draft), bytes, clock)
            .await
    }
    pub async fn retry(
        &self,
        actor: SecretDigest,
        draft: RetryConversationInput,
        clock: AcceptanceTime,
    ) -> Result<ConversationInputAcceptance, ServiceError> {
        // Reserve the maximum retained body before loading it from storage.
        self.enqueue(actor, Submission::Retry(draft), 16 * 1024 * 1024, clock)
            .await
    }
    async fn enqueue(
        &self,
        actor: SecretDigest,
        draft: Submission,
        bytes: usize,
        clock: AcceptanceTime,
    ) -> Result<ConversationInputAcceptance, ServiceError> {
        let store = self.store.clone();
        let processor = self.processor.clone();
        self.consumer.check()?;
        let consumer = self.consumer.clone();
        self.work
            .run(bytes, move |owner, sender| async move {
                let admission = owner.admission();
                let result: Result<ConversationInputAdmission, ServiceError> = async {
                    Ok(match &draft {
                        Submission::New(value) => {
                            store
                                .admit_conversation_input_for_consumer(
                                    &actor,
                                    value,
                                    clock.now()?,
                                    consumer.policy(),
                                )
                                .await?
                        }
                        Submission::Retry(value) => {
                            store
                                .retry_conversation_input_for_consumer(
                                    &actor,
                                    value,
                                    clock.now()?,
                                    consumer.policy(),
                                )
                                .await?
                        }
                    })
                }
                .await;
                let accepted = match result {
                    Ok(accepted) => accepted,
                    Err(error) => {
                        drop(admission);
                        return match sender.send(Err(error)) {
                            Err(Err(error)) => Err(error),
                            _ => Ok(()),
                        };
                    }
                };
                let receipt = match accepted {
                    ConversationInputAdmission::Existing(receipt) => {
                        drop(admission);
                        let _ = sender.send(Ok((&receipt).into()));
                        return Ok(());
                    }
                    ConversationInputAdmission::Accepted(receipt) => receipt,
                };
                owner.mark(&receipt.id);
                drop(admission);
                let agent_id = draft.agent_id().to_owned();
                drop(draft); // Release the original submitted body before processing.
                let _ = sender.send(Ok((&receipt).into()));
                let conversation_id = receipt.conversation_id.clone();
                let id = receipt.id.clone();
                consumer
                    .run(
                        store,
                        conversation_id,
                        id,
                        processor.process(AcceptedConversationInput {
                            actor,
                            agent_id,
                            receipt,
                        }),
                    )
                    .await
            })
            .await
    }
    /// Reads wait only for already-existing admission handoffs, then report
    /// ownership without dispatching or inventing native outcome evidence.
    pub async fn input(
        &self,
        actor: &SecretDigest,
        agent_id: &str,
        conversation_id: &str,
        request_id: &str,
        clock: AcceptanceTime,
    ) -> Result<Option<ConversationInputStatus>, ServiceError> {
        self.work.reap()?;
        self.consumer.check()?;
        let receipt = self
            .store
            .conversation_input_reference(
                actor,
                agent_id,
                conversation_id,
                request_id,
                clock.now()?,
            )
            .await?;
        let Some(receipt) = receipt else {
            return Ok(None);
        };
        let ownership = if self.work.owns_after_admission(&receipt.id).await {
            ConversationInputOwnership::ProcessingHere
        } else {
            ConversationInputOwnership::NotObservedHere
        };
        let delivery = match self
            .store
            .conversation_input_delivery(
                actor,
                agent_id,
                conversation_id,
                &receipt.id,
                clock.now()?,
            )
            .await
        {
            Ok(value) => value,
            Err(chapchoo_persistence::StoreError::NotFound) => {
                if self
                    .store
                    .conversation_input_reference(
                        actor,
                        agent_id,
                        conversation_id,
                        request_id,
                        clock.now()?,
                    )
                    .await?
                    .is_none_or(|current| current.id != receipt.id)
                {
                    return Ok(None);
                }
                return Err(chapchoo_persistence::StoreError::NotFound.into());
            }
            Err(error) => return Err(error.into()),
        };
        Ok(Some(ConversationInputStatus {
            failed: self
                .store
                .conversation_input_queue_state(conversation_id, &receipt.id)
                .await?
                == ConversationInputQueueState::Error,
            acceptance: receipt.into(),
            ownership,
            delivery,
        }))
    }
    pub fn close(&self) {
        self.work.close();
    }
    pub async fn finish(&self) -> Result<(), ServiceError> {
        let work = self.work.finish().await;
        let consumer = self.consumer.finish().await;
        super::input_consumer::combine(work, consumer)
    }
}
