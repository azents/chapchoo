use super::*;

#[tokio::test]
async fn unexpected_worker_unwind_closes_admission_and_is_retained_at_drain()
-> Result<(), TestError> {
    let (backend, _) = Fixture::new();
    let owner = ServerTools::new(backend.clone());
    let call = running(&owner, "A", "panic").await?;
    drop(call);
    let failure = owner.finish().await.err().ok_or("Worker panic was lost")?;
    assert!(failure.report().is_some());
    assert!(owner.submit(request("A", "after failure")).is_err());
    assert_eq!(backend.effects.load(Ordering::Acquire), 0);
    Ok(())
}
