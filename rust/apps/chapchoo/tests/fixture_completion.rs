//! Regression tests for the app fixture's owned completion boundary.
#[path = "../test_support.rs"]
mod test_support;

use std::{
    error::Error,
    future::Future,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::{sync::oneshot, task::JoinSet};
use tokio_util::sync::CancellationToken;
type TestError = Box<dyn Error + Send + Sync>;

#[tokio::test]
async fn failed_exercise_finishes_workers_and_cleanup_before_stopping_dependencies()
-> Result<(), TestError> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let stopped = Arc::new(AtomicBool::new(false));
    let worker_finished = Arc::new(AtomicBool::new(false));
    let failure = test_support::scope(async |fixture| {
        let shutdown = fixture.shutdown();
        let server_stopped = stopped.clone();
        fixture.server(async move {
            shutdown.await?;
            drop(listener);
            server_stopped.store(true, Ordering::SeqCst);
            Err::<(), TestError>("Independent server failure".into())
        });
        let stop = CancellationToken::new();
        let cancellation = stop.clone();
        let dependency = stopped.clone();
        let finished = worker_finished.clone();
        let _waiter = fixture.worker(stop, async move {
            cancellation.cancelled().await;
            assert!(!dependency.load(Ordering::SeqCst));
            finished.store(true, Ordering::SeqCst);
            Err::<(), TestError>("Independent worker failure".into())
        });
        let dependency = stopped.clone();
        fixture.cleanup(async move {
            assert!(!dependency.load(Ordering::SeqCst));
            test_support::completed(
                Err("Independent cleanup failure".into()),
                Err("Independent second cleanup failure".into()),
            )
        });
        Err("Primary exercise failure".into())
    })
    .await
    .err()
    .ok_or("Expected failure")?;
    assert_eq!(
        failure.source().ok_or("Missing primary")?.to_string(),
        "Primary exercise failure"
    );
    let retained = format!("{failure:?}");
    for cause in [
        "Primary exercise failure",
        "Independent worker failure",
        "Independent cleanup failure",
        "Independent second cleanup failure",
        "Independent server failure",
    ] {
        assert!(retained.contains(cause), "{retained}");
    }
    assert!(worker_finished.load(Ordering::SeqCst));
    assert!(stopped.load(Ordering::SeqCst));
    drop(tokio::net::TcpListener::bind(address).await?);
    // Keep the child-spawn regression after the listener release check. A
    // parallel fork can transiently inherit another test thread's open socket
    // before exec applies close-on-exec, which is not fixture ownership.
    #[cfg(unix)]
    failed_exercise_kills_and_reaps_child_before_server_shutdown().await?;
    Ok(())
}

#[tokio::test]
async fn earlier_request_failure_still_joins_later_work_and_retains_its_cause()
-> Result<(), TestError> {
    let mut tasks = JoinSet::new();
    tasks.spawn(async { Err::<(), TestError>("Earlier request failure".into()) });
    let (started, running) = oneshot::channel();
    let (release, released) = oneshot::channel();
    tasks.spawn(async {
        started.send(()).map_err(|_| "Missing test observer")?;
        released.await?;
        Err::<(), TestError>("Later request failure".into())
    });
    running.await?;
    let mut completion = Box::pin(test_support::requests(&mut tasks));
    std::future::poll_fn(|context| {
        assert!(completion.as_mut().poll(context).is_pending());
        std::task::Poll::Ready(())
    })
    .await;
    release.send(()).map_err(|_| "Later work was dropped")?;
    let failure = completion.await.err().ok_or("Expected request failure")?;
    let causes = format!("{failure:?}");
    assert!(causes.contains("Earlier request failure"));
    assert!(causes.contains("Later request failure"));
    Ok(())
}

#[cfg(unix)]
async fn failed_exercise_kills_and_reaps_child_before_server_shutdown() -> Result<(), TestError> {
    let mut observed = None;
    let (gone, exited) = oneshot::channel();
    let failure = test_support::scope(async |fixture| {
        let mut command = tokio::process::Command::new("sh");
        command
            .arg("-c")
            .arg("read fixture_input")
            .stdin(std::process::Stdio::piped())
            .kill_on_drop(true);
        let child = fixture.child(&mut command)?;
        observed = Some(child.clone());
        let shutdown = fixture.shutdown();
        fixture.server(async move {
            shutdown.await?;
            assert!(
                child.lock().await.try_wait()?.is_some(),
                "Child must already be reaped"
            );
            gone.send(()).map_err(|_| "Missing exit observer")?;
            Ok::<(), TestError>(())
        });
        Err("Exercise returned before child exit".into())
    })
    .await
    .err()
    .ok_or("Expected failure")?;
    assert_eq!(failure.to_string(), "Exercise returned before child exit");
    exited.await?;
    assert!(observed.ok_or("Missing child")?.lock().await.id().is_none());
    Ok(())
}
