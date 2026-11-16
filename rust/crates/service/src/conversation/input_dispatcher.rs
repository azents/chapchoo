//! Bounded rediscovery owns readiness waiting independently of an HTTP caller.
use super::{input_consumer::Consumer, input_owner::*, input_work::InputWork};
use crate::{DomainReason, ServiceError};
use chapchoo_persistence::{ConversationInputCandidate, Store};
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::{Mutex, watch},
    task::JoinHandle,
};

pub(super) struct Dispatcher {
    stop: watch::Sender<bool>,
    task: Mutex<Option<JoinHandle<()>>>,
}
#[derive(Default)]
struct Scan {
    after: Option<String>,
    through: Option<String>,
}
impl Scan {
    fn advance(&mut self, page: &[ConversationInputCandidate]) {
        let last = page.last().map(|candidate| &candidate.id);
        if page.len() < 32 || last == self.through.as_ref() {
            *self = Self::default();
        } else {
            self.after = last.cloned();
        }
    }
}
impl Dispatcher {
    pub(super) fn new(
        store: Store,
        processor: Arc<dyn ConversationInputProcessor>,
        work: Arc<InputWork>,
        consumer: Arc<Consumer>,
    ) -> Result<Self, ServiceError> {
        let handle = tokio::runtime::Handle::try_current().map_err(ServiceError::infrastructure)?;
        let (stop, mut stopped) = watch::channel(false);
        let task = handle.spawn(async move {
            let mut cadence = tokio::time::interval(Duration::from_secs(1));
            cadence.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            let mut scan = Scan::default();
            loop {
                tokio::select! {
                    changed=stopped.changed()=>{if changed.is_err() || *stopped.borrow(){break}},
                    _=cadence.tick()=>{}
                }
                if *stopped.borrow() {
                    break;
                }
                if scan.through.is_none() {
                    match store.waiting_conversation_input_frontier().await {
                        Ok(frontier) => scan.through = frontier,
                        Err(error) => {
                            consumer.record_failure(error.into());
                            continue;
                        }
                    }
                }
                let Some(through) = scan.through.as_deref() else {
                    continue;
                };
                let page = match store
                    .waiting_conversation_inputs(scan.after.as_deref(), through)
                    .await
                {
                    Ok(page) => page,
                    Err(error) => {
                        consumer.record_failure(error.into());
                        continue;
                    }
                };
                scan.advance(&page);
                visit_page(
                    page,
                    &stopped,
                    |candidate| {
                        let (store, processor) = (&store, &processor);
                        async move { wake(store, processor, &candidate).await }
                    },
                    |candidate| {
                        let (store, processor, work, consumer) =
                            (&store, &processor, &work, &consumer);
                        async move { deliver(store, processor, work, consumer, candidate).await }
                    },
                    |error| consumer.record_failure(error),
                )
                .await;
            }
        });
        Ok(Self {
            stop,
            task: Mutex::new(Some(task)),
        })
    }
    pub(super) fn close(&self) {
        let _ = self.stop.send(true);
    }
    pub(super) async fn finish(&self) -> Result<(), ServiceError> {
        self.close();
        let mut task = self.task.lock().await;
        if let Some(handle) = task.as_mut() {
            let result = handle.await.map_err(ServiceError::infrastructure);
            *task = None;
            result?;
        }
        Ok(())
    }
}
/// Metadata wake remains independent of body-delivery admission for the whole page.
async fn visit_page<W, D, WF, DF>(
    page: Vec<ConversationInputCandidate>,
    stopped: &watch::Receiver<bool>,
    wake: W,
    deliver: D,
    record_failure: impl Fn(ServiceError),
) where
    W: Fn(ConversationInputCandidate) -> WF,
    D: Fn(ConversationInputCandidate) -> DF,
    WF: std::future::Future<Output = Result<(), ServiceError>> + Send,
    DF: std::future::Future<Output = Result<(), ServiceError>> + Send,
{
    let mut delivery_available = true;
    for candidate in page {
        if *stopped.borrow() {
            break;
        }
        if let Err(error) = wake(candidate.clone()).await
            && error.report().is_some()
        {
            record_failure(error);
        }
        if !delivery_available {
            continue;
        }
        match deliver(candidate).await {
            Ok(()) => {}
            Err(error) if error.reason() == DomainReason::CapacityExhausted => {
                delivery_available = false
            }
            Err(error) if error.report().is_none() => {}
            Err(error) => record_failure(error),
        }
    }
}
async fn deliver(
    store: &Store,
    processor: &Arc<dyn ConversationInputProcessor>,
    work: &Arc<InputWork>,
    consumer: &Arc<Consumer>,
    candidate: ConversationInputCandidate,
) -> Result<(), ServiceError> {
    let store = store.clone();
    let processor = processor.clone();
    let consumer = consumer.clone();
    work.run(16 * 1024 * 1024, move |owner, sender| async move {
        let claim = store
            .claim_conversation_input(&candidate, consumer.policy())
            .await;
        let claim = match claim {
            Ok(Some(claim)) => claim,
            Ok(None) => {
                let _ = sender.send(Ok(()));
                return Ok(());
            }
            Err(error) => {
                let _ = sender.send(Err(error.into()));
                return Ok(());
            }
        };
        owner.mark(&claim.receipt.id);
        let _ = sender.send(Ok(()));
        consumer
            .run(
                store,
                claim.receipt.conversation_id.clone(),
                claim.receipt.id.clone(),
                claim.claim_id.clone(),
                processor.process(AcceptedConversationInput {
                    agent_id: claim.agent_id,
                    receipt: claim.receipt,
                    claim_id: claim.claim_id,
                }),
            )
            .await
    })
    .await
}
async fn wake(
    store: &Store,
    processor: &Arc<dyn ConversationInputProcessor>,
    candidate: &ConversationInputCandidate,
) -> Result<(), ServiceError> {
    let Some(wake) = store.conversation_input_wake(candidate).await? else {
        return Ok(());
    };
    let id = wake.input_id.clone();
    let predecessor = wake.predecessor.clone();
    processor.wake(wake).await?;
    store
        .settle_conversation_input_wake(&id, &predecessor)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidates(start: usize, end: usize) -> Vec<ConversationInputCandidate> {
        (start..end)
            .map(|index| ConversationInputCandidate {
                id: format!("{index:04}"),
                agent_id: "agent".into(),
                conversation_id: "conversation".into(),
            })
            .collect()
    }

    #[test]
    fn full_frontier_page_wraps_despite_continuous_arrivals() {
        let mut scan = Scan {
            after: None,
            through: Some("0063".into()),
        };
        scan.advance(&candidates(0, 32));
        assert_eq!(scan.after.as_deref(), Some("0031"));
        assert_eq!(scan.through.as_deref(), Some("0063"));
        // The Store's frozen bound excludes arrivals 64..96, leaving a full
        // final page. It must wrap without waiting for a short page.
        scan.advance(&candidates(32, 64));
        assert!(scan.after.is_none());
        assert!(scan.through.is_none());
        scan.through = Some("0095".into());
        assert!(
            scan.after.is_none(),
            "Next cycle revisits earlier readiness waiting"
        );
        scan.advance(&candidates(0, 32));
        assert_eq!(scan.after.as_deref(), Some("0031"));
    }

    #[tokio::test]
    async fn saturated_delivery_still_examines_every_wake_in_the_page() {
        let (_stop, stopped) = watch::channel(false);
        let woken = parking_lot::Mutex::new(Vec::new());
        let delivered = parking_lot::Mutex::new(Vec::new());
        let failures = parking_lot::Mutex::new(Vec::new());
        visit_page(
            candidates(0, 32),
            &stopped,
            |candidate| {
                woken.lock().push(candidate.id);
                std::future::ready(Ok(()))
            },
            |candidate| {
                delivered.lock().push(candidate.id);
                std::future::ready(Err(ServiceError::domain(DomainReason::CapacityExhausted)))
            },
            |error| failures.lock().push(error),
        )
        .await;
        assert_eq!(
            *woken.lock(),
            (0..32)
                .map(|index| format!("{index:04}"))
                .collect::<Vec<_>>()
        );
        assert_eq!(*delivered.lock(), vec!["0000"]);
        assert!(
            failures.lock().is_empty(),
            "Saturation is not an infrastructure failure"
        );
    }

    #[tokio::test]
    async fn failed_dispatcher_join_is_settled_once() {
        let (stop, _stopped) = watch::channel(false);
        let injected_failure = true;
        let dispatcher = Dispatcher {
            stop,
            task: Mutex::new(Some(tokio::spawn(async move {
                assert!(!injected_failure, "Injected dispatcher panic");
            }))),
        };
        assert!(dispatcher.finish().await.is_err());
        assert!(dispatcher.task.lock().await.is_none());
        assert!(dispatcher.finish().await.is_ok());
    }
}
