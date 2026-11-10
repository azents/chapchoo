use super::*;
use crate::conversation::input_owner::ConversationInputAcceptance;
use chapchoo_domain::ConversationTarget;
use std::{sync::atomic::AtomicUsize, task::Poll};
type TestError = Box<dyn Error + Send + Sync>;
fn receipt(id: &str) -> ConversationInputAcceptance {
    ConversationInputAcceptance {
        id: id.into(),
        conversation_id: "conversation".into(),
        request_id: id.into(),
        target: ConversationTarget::NotStarted,
        accepted_at: 1,
    }
}
#[tokio::test]
async fn dropped_browser_waiter_keeps_admission_handoff_and_processing_owned()
-> Result<(), TestError> {
    let work = Arc::new(InputWork::new(1)?);
    let count = Arc::new(AtomicUsize::new(0));
    let (entered, entry) = oneshot::channel();
    let (admit, admitted) = oneshot::channel();
    let (finish, finished) = oneshot::channel();
    let owner = work.clone();
    let processed = count.clone();
    let caller = tokio::spawn(async move {
        owner
            .run(1, move |work, sender| async move {
                let guard = work.admission();
                let _ = entered.send(());
                admitted.await.map_err(ServiceError::infrastructure)?;
                work.mark("operation");
                drop(guard);
                let _ = sender.send(Ok(receipt("operation")));
                finished.await.map_err(ServiceError::infrastructure)?;
                processed.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
            .await
    });
    entry.await?;
    caller.abort();
    assert!(caller.await.is_err_and(|error| error.is_cancelled()));
    let mut observed = Box::pin(work.owns_after_admission("operation"));
    std::future::poll_fn(|context| {
        assert!(observed.as_mut().poll(context).is_pending());
        Poll::Ready(())
    })
    .await;
    let (later_ready, later_started) = oneshot::channel();
    let (release_later, later_released) = oneshot::channel();
    let later = work.clone();
    let late = tokio::spawn(async move {
        let guard = later.admission();
        let _ = later_ready.send(());
        later_released.await?;
        drop(guard);
        Ok::<(), TestError>(())
    });
    later_started.await?;
    admit
        .send(())
        .map_err(|_| "Admission owner was abandoned")?;
    assert!(observed.await, "Finite handoff lost its admitted owner");
    assert_eq!(
        work.run::<ConversationInputAcceptance, _, _>(1, |_, _| async { Ok(()) })
            .await
            .err()
            .ok_or("Capacity released before effect")?
            .reason(),
        DomainReason::CapacityExhausted
    );
    release_later
        .send(())
        .map_err(|_| "Later handoff disappeared")?;
    late.await??;
    finish
        .send(())
        .map_err(|_| "Processing was cancelled by browser loss")?;
    work.finish().await?;
    assert_eq!(count.load(Ordering::SeqCst), 1);
    assert!(!work.owns_after_admission("operation").await);
    Ok(())
}
#[tokio::test]
async fn short_acceptance_precedes_processing_and_drain_waits_for_the_effect()
-> Result<(), TestError> {
    let work = Arc::new(InputWork::new(1)?);
    let (release, released) = oneshot::channel();
    let accepted = work
        .run(1, |work, sender| async move {
            work.mark("short");
            let _ = sender.send(Ok(receipt("short")));
            released.await.map_err(ServiceError::infrastructure)?;
            Ok(())
        })
        .await?;
    assert_eq!(accepted.id, "short");
    assert!(work.owns_after_admission("short").await);
    let mut drain = Box::pin(work.finish());
    std::future::poll_fn(|context| {
        assert!(drain.as_mut().poll(context).is_pending());
        Poll::Ready(())
    })
    .await;
    release
        .send(())
        .map_err(|_| "Accepted processing was abandoned")?;
    drain.await?;
    assert_eq!(
        work.run::<ConversationInputAcceptance, _, _>(1, |_, _| async { Ok(()) })
            .await
            .err()
            .ok_or("Closed owner admitted work")?
            .reason(),
        DomainReason::InvalidTransition
    );
    Ok(())
}
#[tokio::test]
async fn retained_byte_budget_is_independent_of_count_and_survives_acceptance()
-> Result<(), TestError> {
    let work = Arc::new(InputWork::new(4)?);
    let mut releases = Vec::new();
    for index in 0..2 {
        let (release, released) = oneshot::channel();
        releases.push(release);
        work.run::<ConversationInputAcceptance, _, _>(
            16 * 1024 * 1024,
            move |work, sender| async move {
                let id = format!("large-{index}");
                work.mark(&id);
                let _ = sender.send(Ok(receipt(&id)));
                released.await.map_err(ServiceError::infrastructure)?;
                Ok(())
            },
        )
        .await?;
    }
    assert_eq!(
        work.run::<ConversationInputAcceptance, _, _>(1, |_, _| async { Ok(()) })
            .await
            .err()
            .ok_or("Byte budget bypassed")?
            .reason(),
        DomainReason::CapacityExhausted
    );
    assert_eq!(
        work.run::<ConversationInputAcceptance, _, _>(16 * 1024 * 1024 + 1, |_, _| async {
            Ok(())
        })
        .await
        .err()
        .ok_or("Input bound bypassed")?
        .reason(),
        DomainReason::InvalidInput
    );
    for release in releases {
        release.send(()).map_err(|_| "Byte owner was dropped")?;
    }
    work.finish().await?;
    Ok(())
}
struct Probe(Arc<AtomicBool>);
impl Drop for Probe {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}
impl fmt::Debug for Probe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Probe([REDACTED])")
    }
}
impl fmt::Display for Probe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Synthetic processing failure")
    }
}
impl Error for Probe {}
#[tokio::test]
async fn cancelled_drain_retains_already_joined_failures_until_reentry() -> Result<(), TestError> {
    let work = Arc::new(InputWork::new(2)?);
    let mut releases = Vec::new();
    let mut dropped = Vec::new();
    for index in 0..2 {
        let (release, released) = oneshot::channel();
        releases.push(release);
        let observed = Arc::new(AtomicBool::new(false));
        dropped.push(observed.clone());
        work.run::<ConversationInputAcceptance, _, _>(1, move |_, sender| async move {
            let _ = sender.send(Ok(receipt(&format!("failure-{index}"))));
            released.await.map_err(ServiceError::infrastructure)?;
            Err(ServiceError::infrastructure(Probe(observed)))
        })
        .await?;
    }
    releases
        .remove(0)
        .send(())
        .map_err(|_| "First task disappeared")?;
    let mut drain = Box::pin(work.finish());
    // Poll until the first completion is authoritatively retained in Book. No
    // sleep or presumed scheduling order establishes the cancellation boundary.
    std::future::poll_fn(|context| {
        assert!(drain.as_mut().poll(context).is_pending());
        if work.book.lock().failures.len() == 1 {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    })
    .await;
    drop(drain);
    assert!(!dropped[0].load(Ordering::SeqCst));
    releases
        .remove(0)
        .send(())
        .map_err(|_| "Second task abandoned after cancelled drain")?;
    let error = work
        .finish()
        .await
        .err()
        .ok_or("Owned failures discarded")?;
    assert!(
        dropped
            .iter()
            .all(|dropped| !dropped.load(Ordering::SeqCst))
    );
    drop(error);
    assert!(dropped.iter().all(|dropped| dropped.load(Ordering::SeqCst)));
    Ok(())
}
