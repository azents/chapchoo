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
