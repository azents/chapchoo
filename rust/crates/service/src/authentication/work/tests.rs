use super::*;
use std::sync::atomic::AtomicBool;

type TestError = Box<dyn Error + Send + Sync>;
#[derive(Debug)]
struct Probe(Arc<AtomicBool>);
impl fmt::Display for Probe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Owned failure probe")
    }
}
impl Error for Probe {}
impl Drop for Probe {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ownership_observation_waits_for_the_existing_admission_batch_only() -> Result<(), TestError>
{
    for dispatched in [true, false] {
        let work = Arc::new(AuthenticationWork::new(1)?);
        let (visible, committed) = oneshot::channel();
        let (handoff, handed_off) = oneshot::channel();
        let (finish, finished) = oneshot::channel();
        let worker = work.clone();
        let caller = tokio::spawn(async move {
            worker
                .run(move |work| async move {
                    let admission = work.admission();
                    visible
                        .send(())
                        .map_err(|_| ServiceError::domain(DomainReason::Internal))?;
                    handed_off.await.map_err(ServiceError::infrastructure)?;
                    if dispatched {
                        work.mark("visible-attempt".into(), "visible-step");
                    }
                    drop(admission);
                    finished.await.map_err(ServiceError::infrastructure)?;
                    Ok(())
                })
                .await
        });
        committed.await?;
        let mut observed =
            Box::pin(work.owns_after_admission("visible-attempt", Some("visible-step")));
        std::future::poll_fn(|context| {
            assert!(observed.as_mut().poll(context).is_pending());
            std::task::Poll::Ready(())
        })
        .await;
        // Later admissions cannot explain the row already observed; they must
        // not extend the reader's finite handoff barrier.
        let (later_ready, later_started) = oneshot::channel();
        let (release_later, later_released) = oneshot::channel();
        let later = work.clone();
        let later = tokio::spawn(async move {
            let guard = later.admission();
            later_ready.send(()).map_err(|_| "Observer disappeared")?;
            later_released.await?;
            drop(guard);
            Ok::<_, TestError>(())
        });
        later_started.await?;
        handoff.send(()).map_err(|_| "Admission was abandoned")?;
        assert_eq!(observed.await, dispatched);
        release_later
            .send(())
            .map_err(|_| "Later admission was abandoned")?;
        later.await??;
        assert!(
            !work
                .owns_after_admission("visible-attempt", Some("another-step"))
                .await
        );
        finish.send(()).map_err(|_| "Effect owner was abandoned")?;
        caller.await??;
        work.finish().await?;
        assert!(
            !work
                .owns_after_admission("visible-attempt", Some("visible-step"))
                .await
        );
    }
    Ok(())
}

#[tokio::test]
async fn cancelled_finish_retains_joined_primary_and_cleanup_causes_until_reentry()
-> Result<(), TestError> {
    let work = Arc::new(AuthenticationWork::new(2)?);
    let primary_drop = Arc::new(AtomicBool::new(false));
    let cleanup_drop = Arc::new(AtomicBool::new(false));
    let (entered_one, ready_one) = oneshot::channel();
    let (release_one, waiting_one) = oneshot::channel();
    let (entered_two, ready_two) = oneshot::channel();
    let (release_two, waiting_two) = oneshot::channel();
    let first = work.clone();
    let primary = primary_drop.clone();
    let cleanup = cleanup_drop.clone();
    let caller_one = tokio::spawn(async move {
        first
            .run(move |_| async move {
                entered_one
                    .send(())
                    .map_err(|_| ServiceError::domain(DomainReason::Internal))?;
                waiting_one.await.map_err(ServiceError::infrastructure)?;
                Err::<(), ServiceError>(combined(
                    ServiceError::infrastructure(Probe(primary)),
                    ServiceError::infrastructure(Probe(cleanup)),
                ))
            })
            .await
    });
    ready_one.await?;
    caller_one.abort();
    assert!(caller_one.await.is_err_and(|error| error.is_cancelled()));
    let second = work.clone();
    let caller_two = tokio::spawn(async move {
        second
            .run(move |_| async move {
                entered_two
                    .send(())
                    .map_err(|_| ServiceError::domain(DomainReason::Internal))?;
                waiting_two.await.map_err(ServiceError::infrastructure)?;
                Ok(())
            })
            .await
    });
    ready_two.await?;
    caller_two.abort();
    assert!(caller_two.await.is_err_and(|error| error.is_cancelled()));
    assert_eq!(work.capacity.available_permits(), 0);
    release_one
        .send(())
        .map_err(|_| "First owned call disappeared")?;
    let joined = work.collected.notified();
    let mut finish = Box::pin(work.finish());
    tokio::select! {
        biased;
        result=&mut finish=>return Err(format!("Finish completed before the second call settled: {result:?}").into()),
        _=joined=>{},
    }
    assert_eq!(work.book.lock().failures.len(), 1);
    drop(finish);
    assert!(!primary_drop.load(Ordering::SeqCst));
    assert!(!cleanup_drop.load(Ordering::SeqCst));
    release_two
        .send(())
        .map_err(|_| "Second owned call disappeared")?;
    let failure = work.finish().await.err().ok_or("Lost retained failure")?;
    assert_eq!(work.capacity.available_permits(), 2);
    assert!(!primary_drop.load(Ordering::SeqCst));
    assert!(!cleanup_drop.load(Ordering::SeqCst));
    drop(failure);
    assert!(primary_drop.load(Ordering::SeqCst));
    assert!(cleanup_drop.load(Ordering::SeqCst));
    assert!(
        work.run(|_| async { Ok(()) })
            .await
            .is_err_and(|error| error.reason() == DomainReason::InvalidTransition)
    );
    Ok(())
}
