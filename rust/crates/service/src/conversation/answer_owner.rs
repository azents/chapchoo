//! Typed answer acceptance is owned beyond a cancelled browser waiter.
use super::{input_work::InputWork, *};
use chapchoo_crypto::SecretDigest;
use chapchoo_persistence::{
    AcceptConversationAnswer, ConversationAnswer, ConversationAnswerAdmission,
    ConversationAnswerDelivery, ConversationAnswerReceipt, ConversationDeliveryTarget,
};
use std::{future::Future, pin::Pin, sync::Arc};

pub struct ConversationAnswerDraft {
    pub agent_id: String,
    pub conversation_id: String,
    pub request_id: String,
    pub target: ConversationDeliveryTarget,
    pub responder_token: String,
    pub answer: ConversationAnswer,
}
#[derive(Clone)]
pub struct ConversationAnswerAcceptance {
    pub id: String,
    pub conversation_id: String,
    pub request_id: String,
    pub responder_token: String,
    pub occurrence: SourcePosition,
    pub accepted_at: i64,
    pub delivery: ConversationAnswerDelivery,
}
impl From<&ConversationAnswerReceipt> for ConversationAnswerAcceptance {
    fn from(value: &ConversationAnswerReceipt) -> Self {
        Self {
            id: value.id.clone(),
            conversation_id: value.conversation_id.clone(),
            request_id: value.request_id.clone(),
            responder_token: value.responder_token.clone(),
            occurrence: value.occurrence.clone(),
            accepted_at: value.accepted_at,
            delivery: value.delivery,
        }
    }
}
impl From<chapchoo_persistence::ConversationAnswerReference> for ConversationAnswerAcceptance {
    fn from(value: chapchoo_persistence::ConversationAnswerReference) -> Self {
        Self {
            id: value.id,
            conversation_id: value.conversation_id,
            request_id: value.request_id,
            responder_token: value.responder_token,
            occurrence: value.occurrence,
            accepted_at: value.accepted_at,
            delivery: value.delivery,
        }
    }
}
pub struct AcceptedConversationAnswer {
    pub actor: SecretDigest,
    pub agent_id: String,
    pub target: ConversationDeliveryTarget,
    pub receipt: ConversationAnswerReceipt,
}
pub type ConversationAnswerFuture =
    Pin<Box<dyn Future<Output = Result<(), ServiceError>> + Send + 'static>>;
pub trait ConversationAnswerProcessor: Send + Sync {
    fn answer(&self, answer: AcceptedConversationAnswer) -> ConversationAnswerFuture;
}
#[derive(Clone)]
pub struct ConversationAnswerService {
    service: Arc<ConversationService>,
    processor: Arc<dyn ConversationAnswerProcessor>,
    work: Arc<InputWork>,
}
macro_rules! safe_debug {($($name:ty),+)=>{$(impl fmt::Debug for $name {
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result {f.write_str(concat!(stringify!($name),"([REDACTED])"))}
})+};}
safe_debug!(
    ConversationAnswerDraft,
    ConversationAnswerAcceptance,
    AcceptedConversationAnswer,
    ConversationAnswerService
);
impl ConversationAnswerService {
    pub fn new(
        service: Arc<ConversationService>,
        processor: Arc<dyn ConversationAnswerProcessor>,
    ) -> Result<Self, ServiceError> {
        Ok(Self {
            service,
            processor,
            work: Arc::new(InputWork::new(16)?),
        })
    }
    pub async fn accept(
        &self,
        actor: SecretDigest,
        draft: ConversationAnswerDraft,
        clock: crate::AcceptanceTime,
    ) -> Result<ConversationAnswerAcceptance, ServiceError> {
        let bytes = serde_json::to_vec(&draft.answer)
            .map_err(|_| ServiceError::domain(DomainReason::InvalidInput))?;
        let size = bytes.len();
        drop(bytes);
        let service = self.service.clone();
        let processor = self.processor.clone();
        self.work
            .run(size, move |owner, sender| async move {
                let admission = owner.admission();
                let accepted = service
                    .accept_answer(
                        &actor,
                        &AcceptConversationAnswer {
                            agent_id: &draft.agent_id,
                            conversation_id: &draft.conversation_id,
                            request_id: &draft.request_id,
                            target: &draft.target,
                            responder_token: &draft.responder_token,
                            answer: &draft.answer,
                        },
                        clock.now()?,
                    )
                    .await;
                let receipt = match accepted {
                    Ok(ConversationAnswerAdmission::Existing(receipt)) => {
                        drop(admission);
                        let _ = sender.send(Ok((&receipt).into()));
                        return Ok(());
                    }
                    Ok(ConversationAnswerAdmission::Accepted(receipt)) => receipt,
                    Err(error) => {
                        drop(admission);
                        return match sender.send(Err(error)) {
                            Err(Err(error)) => Err(error),
                            _ => Ok(()),
                        };
                    }
                };
                owner.mark(&receipt.id);
                drop(admission);
                let _ = sender.send(Ok((&receipt).into()));
                let agent_id = draft.agent_id;
                let target = draft.target;
                drop(draft.answer);
                processor
                    .answer(AcceptedConversationAnswer {
                        actor,
                        agent_id,
                        target,
                        receipt,
                    })
                    .await
            })
            .await
    }
    pub async fn current(
        &self,
        actor: &SecretDigest,
        agent: &str,
        conversation: &str,
        request: &str,
        clock: crate::AcceptanceTime,
    ) -> Result<Option<ConversationAnswerAcceptance>, ServiceError> {
        self.work.reap()?;
        Ok(self
            .service
            .store
            .conversation_answer_reference(actor, agent, conversation, request, clock.now()?)
            .await?
            .map(Into::into))
    }
    pub fn close(&self) {
        self.work.close();
    }
    pub async fn finish(&self) -> Result<(), ServiceError> {
        self.work.finish().await
    }
}
