mod support;
#[path = "support/tasks.rs"]
mod tasks;

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

#[test]
fn exercise_and_schema_cleanup_failures_remain_owned_together() -> Result<(), TestError> {
    let dropped = Arc::new(AtomicUsize::new(0));
    let exercise: TestError = Box::new(Cause("Exercise failed", dropped.clone()));
    let cleanup: TestError = Box::new(Cause("Schema cleanup failed", dropped.clone()));
    let failure = support::combine::<(), ()>(Err(exercise), Err(cleanup))
        .err()
        .ok_or("Both failures were discarded")?;
    assert_eq!(dropped.load(Ordering::SeqCst), 0);
    assert_eq!(
        failure
            .source()
            .ok_or("Missing primary source")?
            .to_string(),
        "Exercise failed"
    );
    assert!(failure.to_string().contains("Schema cleanup failed"));
    drop(failure);
    assert_eq!(dropped.load(Ordering::SeqCst), 2);
    Ok(())
}

#[test]
fn completion_keeps_success_values_and_single_typed_causes() -> Result<(), TestError> {
    assert_eq!(support::combine(Ok(7), Ok(9))?, (7, 9));
    let dropped = Arc::new(AtomicUsize::new(0));
    for primary_fails in [true, false] {
        let cause: TestError = Box::new(Cause("Single failure", dropped.clone()));
        let (primary, secondary) = if primary_fails {
            (Err(cause), Ok(()))
        } else {
            (Ok(()), Err(cause))
        };
        let failure = support::combine(primary, secondary)
            .err()
            .ok_or("Missing single cause")?;
        assert!(failure.downcast_ref::<Cause>().is_some());
    }
    assert_eq!(dropped.load(Ordering::SeqCst), 2);
    Ok(())
}

#[tokio::test]
async fn earlier_join_failure_does_not_abandon_the_later_owned_task() -> Result<(), TestError> {
    let first = tokio::spawn(std::future::pending::<Result<(), TestError>>());
    first.abort();
    let (release, released) = tokio::sync::oneshot::channel();
    let (completed, completion) = tokio::sync::oneshot::channel();
    let second = tokio::spawn(async move {
        released.await?;
        completed.send(()).map_err(|_| "Completion receiver lost")?;
        Ok::<_, TestError>(())
    });
    let mut joined = Box::pin(tasks::join_pair(first, second, None));
    std::future::poll_fn(|context| {
        assert!(joined.as_mut().poll(context).is_pending());
        std::task::Poll::Ready(())
    })
    .await;
    release.send(()).map_err(|_| "Later task was abandoned")?;
    assert!(joined.await.is_err());
    completion.await?;
    Ok(())
}

#[tokio::test]
async fn deadline_finishes_started_work_and_keeps_already_collected_causes() -> Result<(), TestError>
{
    let dropped = Arc::new(AtomicUsize::new(0));
    let cause = Cause("Completed database failure", dropped.clone());
    let (first_done, first_ready) = tokio::sync::oneshot::channel();
    let first = tokio::spawn(async move {
        let _ = first_done.send(());
        Err::<(), TestError>(Box::new(cause))
    });
    let (started, ready) = tokio::sync::oneshot::channel();
    let (release, released) = tokio::sync::oneshot::channel();
    let pending_drop = Cause("Pending work", dropped.clone());
    let second = tokio::spawn(async move {
        let _guard = pending_drop;
        let _ = started.send(());
        released.await?;
        Ok::<_, TestError>(())
    });
    first_ready.await?;
    ready.await?;
    let expired = tokio::time::Instant::now()
        .checked_sub(std::time::Duration::from_secs(1))
        .ok_or("Clock cannot represent expired deadline")?;
    let mut joined = Box::pin(tasks::join_pair(first, second, Some(expired)));
    std::future::poll_fn(|context| {
        assert!(joined.as_mut().poll(context).is_pending());
        std::task::Poll::Ready(())
    })
    .await;
    // The expired waiter cannot release ownership of the pending DB task.
    release
        .send(())
        .map_err(|_| "Deadline abandoned started work")?;
    let failure = joined.await.err().ok_or("Missing fixture deadline")?;
    assert!(failure.to_string().contains("deadline exceeded"));
    assert!(failure.to_string().contains("Completed database failure"));
    assert_eq!(
        dropped.load(Ordering::SeqCst),
        1,
        "Started task was not joined"
    );
    drop(failure);
    assert_eq!(dropped.load(Ordering::SeqCst), 2);
    Ok(())
}
