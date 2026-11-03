mod support;

use std::{
    error::Error,
    fmt,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

type TestError = Box<dyn Error + Send + Sync>;

#[derive(Debug)]
struct Cause(&'static str, Arc<AtomicUsize>);

impl fmt::Display for Cause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}
impl Error for Cause {}
impl Drop for Cause {
    fn drop(&mut self) {
        self.1.fetch_add(1, Ordering::SeqCst);
    }
}

#[tokio::test]
async fn primary_failure_still_finishes_every_server_and_pending_call() -> Result<(), TestError> {
    let dropped = Arc::new(AtomicUsize::new(0));
    let primary: TestError = Box::new(Cause("Primary exercise failure", dropped.clone()));
    let server: TestError = Box::new(Cause("Server cleanup failure", dropped.clone()));
    let (release, released) = tokio::sync::oneshot::channel();
    let (completed, completion) = tokio::sync::oneshot::channel();
    let mut scoped = Box::pin(support::scope(async |fixture| {
        let stopped = fixture.shutdown();
        fixture.server(async move {
            stopped.await?;
            Err::<(), TestError>(server)
        });
        let call = fixture.call(async move {
            released.await?;
            completed.send(()).map_err(|_| "Completion receiver lost")?;
            Ok::<_, TestError>(())
        });
        drop(call);
        Err::<(), TestError>(primary)
    }));
    std::future::poll_fn(|context| {
        assert!(scoped.as_mut().poll(context).is_pending());
        std::task::Poll::Ready(())
    })
    .await;
    release.send(()).map_err(|_| "Owned call was abandoned")?;
    let failure = scoped.await.err().ok_or("Missing fixture failure")?;
    completion.await?;
    assert_eq!(dropped.load(Ordering::SeqCst), 0);
    assert_eq!(
        failure.source().ok_or("Missing primary cause")?.to_string(),
        "Primary exercise failure"
    );
    assert!(format!("{failure:?}").contains("Server cleanup failure"));
    drop(failure);
    assert_eq!(dropped.load(Ordering::SeqCst), 2);
    Ok(())
}

#[tokio::test]
async fn completed_call_failure_is_retained_after_its_receiver_is_abandoned()
-> Result<(), TestError> {
    let dropped = Arc::new(AtomicUsize::new(0));
    let cause: TestError = Box::new(Cause("Native call failure", dropped.clone()));
    let failure = support::scope(async |fixture| {
        let call = fixture.call(async move { Err::<(), TestError>(cause) });
        // Channel completion is not the typed failure's owner.
        assert!(call.await.is_err());
        Ok(())
    })
    .await
    .err()
    .ok_or("Completed call failure was discarded")?;
    assert!(failure.downcast_ref::<Cause>().is_some());
    assert_eq!(dropped.load(Ordering::SeqCst), 0);
    drop(failure);
    assert_eq!(dropped.load(Ordering::SeqCst), 1);
    Ok(())
}

#[tokio::test]
async fn explicitly_cancelled_call_does_not_hide_an_independent_server_failure()
-> Result<(), TestError> {
    let dropped = Arc::new(AtomicUsize::new(0));
    let cause: TestError = Box::new(Cause("Independent server failure", dropped.clone()));
    let failure = support::scope(async |fixture| {
        let call = fixture.call(std::future::pending::<Result<(), TestError>>());
        fixture.abort(&call);
        assert!(call.await.is_err());
        fixture.server(async move { Err::<(), TestError>(cause) });
        Ok(())
    })
    .await
    .err()
    .ok_or("Independent server failure was hidden")?;
    assert!(failure.downcast_ref::<Cause>().is_some());
    drop(failure);
    assert_eq!(dropped.load(Ordering::SeqCst), 1);
    Ok(())
}

#[tokio::test]
async fn required_shutdown_signal_failure_is_retained_with_the_exercise_failure()
-> Result<(), TestError> {
    let failure = support::scope(async |fixture| {
        drop(fixture.required_shutdown("Required listener ended too early"));
        Err::<(), TestError>("Primary exercise failure".into())
    })
    .await
    .err()
    .ok_or("Missing shutdown failure")?;
    assert_eq!(
        failure
            .source()
            .ok_or("Missing primary source")?
            .to_string(),
        "Primary exercise failure"
    );
    assert!(format!("{failure:?}").contains("Required listener ended too early"));
    Ok(())
}

#[tokio::test]
async fn shutdown_deadline_keeps_completed_causes_and_waits_for_the_later_owner()
-> Result<(), TestError> {
    let (release, released) = tokio::sync::oneshot::channel();
    let mut scoped = Box::pin(support::scope(async |fixture| {
        fixture.shutdown_budget(std::time::Duration::ZERO);
        fixture.server(async { Err::<(), TestError>("Completed server failure".into()) });
        fixture.server(async move {
            released.await?;
            Ok::<_, TestError>(())
        });
        Ok(())
    }));
    std::future::poll_fn(|context| {
        assert!(scoped.as_mut().poll(context).is_pending());
        std::task::Poll::Ready(())
    })
    .await;
    // Elapsed time is this test's contract. No work is released until the
    // expired timeout has been explicitly polled through to completion drain.
    tokio::time::sleep(std::time::Duration::from_millis(1)).await;
    std::future::poll_fn(|context| {
        assert!(scoped.as_mut().poll(context).is_pending());
        std::task::Poll::Ready(())
    })
    .await;
    release
        .send(())
        .map_err(|_| "Deadline abandoned the later server")?;
    let failure = scoped
        .await
        .err()
        .ok_or("Missing shutdown deadline failure")?;
    let retained = format!("{failure:?}");
    assert!(retained.contains("Completed server failure"));
    assert!(retained.contains("shutdown deadline exceeded"));
    Ok(())
}
