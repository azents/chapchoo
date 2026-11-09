use super::{input_work::InputWork, *};
use chapchoo_crypto::SecretDigest;
use chapchoo_persistence::{
    AcceptConversationStop, ConversationStopAdmission, ConversationStopReceipt,
    ConversationWorkTarget,
};
use std::{future::Future, pin::Pin, sync::Arc};
pub struct AcceptedConversationStop {
    pub actor: SecretDigest,
    pub agent_id: String,
    pub receipt: ConversationStopReceipt,
}
pub type ConversationStopFuture =
    Pin<Box<dyn Future<Output = Result<(), ServiceError>> + Send + 'static>>;
pub trait ConversationStopProcessor: Send + Sync {
    fn stop(&self, stop: AcceptedConversationStop) -> ConversationStopFuture;
}
#[derive(Clone)]
pub struct ConversationStopService {
    store: Store,
    processor: Arc<dyn ConversationStopProcessor>,
    work: Arc<InputWork>,
}
impl fmt::Debug for ConversationStopService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ConversationStopService([REDACTED])")
    }
}
impl fmt::Debug for AcceptedConversationStop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("AcceptedConversationStop([REDACTED])")
    }
}
impl ConversationStopService {
    pub fn new(
        store: Store,
        processor: Arc<dyn ConversationStopProcessor>,
    ) -> Result<Self, ServiceError> {
        Ok(Self {
            store,
            processor,
            work: Arc::new(InputWork::new(16)?),
        })
    }
    pub async fn accept(
        &self,
        actor: SecretDigest,
        agent_id: String,
        conversation_id: String,
        request_id: String,
        target: ConversationWorkTarget,
        clock: crate::AcceptanceTime,
    ) -> Result<ConversationStopReceipt, ServiceError> {
        let store = self.store.clone();
        let processor = self.processor.clone();
        self.work
            .run(1, move |owner, sender| async move {
                let result: Result<ConversationStopAdmission, ServiceError> = async {
                    Ok(store
                        .accept_conversation_stop(
                            &actor,
                            &AcceptConversationStop {
                                agent_id: &agent_id,
                                conversation_id: &conversation_id,
                                request_id: &request_id,
                                target: &target,
                            },
                            clock.now()?,
                        )
                        .await?)
                }
                .await;
                let accepted = match result {
                    Ok(accepted) => accepted,
                    Err(error) => {
                        return match sender.send(Err(error)) {
                            Err(Err(error)) => Err(error),
                            _ => Ok(()),
                        };
                    }
                };
                let receipt = match accepted {
                    ConversationStopAdmission::Existing(receipt) => {
                        let _ = sender.send(Ok(receipt));
                        return Ok(());
                    }
                    ConversationStopAdmission::Accepted(receipt) => receipt,
                };
                owner.mark(&receipt.id);
                let _ = sender.send(Ok(receipt.clone()));
                processor
                    .stop(AcceptedConversationStop {
                        actor,
                        agent_id,
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
    ) -> Result<Option<ConversationStopReceipt>, ServiceError> {
        self.work.reap()?;
        Ok(self
            .store
            .conversation_stop(actor, agent, conversation, request, clock.now()?)
            .await?)
    }
    pub fn close(&self) {
        self.work.close();
    }
    pub async fn finish(&self) -> Result<(), ServiceError> {
        self.work.finish().await
    }
}
