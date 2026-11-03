//! Nested fixture completion must keep every owned cause diagnosable.
mod support;

type Error = Box<dyn std::error::Error + Send + Sync>;

#[tokio::test]
async fn nested_completion_preserves_inner_primary_and_cleanup_diagnostics() -> Result<(), Error> {
    let failure = support::scope(async |outer| {
        outer.server(async { Err::<(), Error>("Outer server failure".into()) });
        support::scope(async |inner| {
            inner.server(async { Err::<(), Error>("Inner server failure".into()) });
            Err::<(), Error>("Inner exercise failure".into())
        })
        .await
    })
    .await
    .err()
    .ok_or("Expected nested fixture failure")?;
    let retained = format!("{failure:?}");
    for cause in [
        "Inner exercise failure",
        "Inner server failure",
        "Outer server failure",
    ] {
        assert!(retained.contains(cause), "{retained}");
    }
    Ok(())
}
