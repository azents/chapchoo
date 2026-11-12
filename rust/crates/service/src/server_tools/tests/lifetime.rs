use super::*;

#[tokio::test]
async fn revoked_waiting_call_does_not_reuse_admission_authority_at_dispatch()
-> Result<(), TestError> {
    let (backend, _) = Fixture::new();
    let owner = ServerTools::new(backend.clone());
    let mut active = Vec::new();
    for _ in 0..16 {
        active.push(running(&owner, "A", "active").await?);
    }
    let mut queued = owner.submit(request("B", "revoked"))?;
    assert!(matches!(
        queued.next().await,
        Some(ServerToolObservation::Waiting)
    ));
    backend.allowed.store(false, Ordering::Release);
    backend.released.add_permits(1);
    assert!(matches!(
        queued.next().await,
        Some(ServerToolObservation::Final(
            ServerToolOutcome::NotExecuted(ServerToolRejection::Authority)
        ))
    ));
    owner.close();
    drop(active);
    backend.released.add_permits(16);
    owner.finish().await?;
    assert_eq!(backend.effects.load(Ordering::Acquire), 16);
    Ok(())
}

#[tokio::test]
async fn global_waiting_capacity_and_encoded_payload_are_hard_bounds() -> Result<(), TestError> {
    let (backend, _) = Fixture::new();
    let owner = ServerTools::new(backend.clone());
    let mut active = Vec::new();
    for _ in 0..16 {
        active.push(running(&owner, "active", "active").await?);
    }
    let mut queued = Vec::new();
    for agent in 0..16 {
        for _ in 0..8 {
            let mut call = owner.submit(request(&format!("agent-{agent}"), "waiting"))?;
            assert!(matches!(
                call.next().await,
                Some(ServerToolObservation::Waiting)
            ));
            queued.push(call);
        }
    }
    let error = owner
        .submit(request("overflow", "extra"))
        .err()
        .ok_or("Accepted overflow")?;
    assert_eq!(error.reason(), DomainReason::CapacityExhausted);
    let mut oversize = request("A", "large");
    oversize.arguments = vec![0; MAX_TOOL_PAYLOAD + 1];
    let error = owner
        .submit(oversize)
        .err()
        .ok_or("Accepted oversize arguments")?;
    assert_eq!(error.reason(), DomainReason::InvalidInput);
    owner.close();
    for call in &mut queued {
        assert!(matches!(
            call.next().await,
            Some(ServerToolObservation::Final(
                ServerToolOutcome::NotExecuted(ServerToolRejection::Closing)
            ))
        ));
    }
    drop(active);
    backend.released.add_permits(16);
    owner.finish().await?;
    assert_eq!(backend.effects.load(Ordering::Acquire), 16);
    assert!(owner.submit(request("A", "closed")).is_err());
    Ok(())
}

#[tokio::test]
async fn lost_observer_does_not_cancel_started_work_or_release_its_execution_slot()
-> Result<(), TestError> {
    let (backend, _) = Fixture::new();
    let owner = ServerTools::new(backend.clone());
    let call = running(&owner, "A", "lost").await?;
    drop(call);
    assert_eq!(owner.executing(), 1);
    owner.close();
    backend.released.add_permits(1);
    owner.finish().await?;
    assert_eq!(backend.effects.load(Ordering::Acquire), 1);
    assert_eq!(backend.cancelled.load(Ordering::Acquire), 0);
    assert_eq!(owner.executing(), 0);
    Ok(())
}

#[tokio::test]
async fn cancelled_drain_can_reenter_without_dropping_physical_work() -> Result<(), TestError> {
    let (backend, _) = Fixture::new();
    let owner = ServerTools::new(backend.clone());
    let mut call = running(&owner, "A", "retained").await?;
    {
        let drain = owner.finish();
        tokio::pin!(drain);
        std::future::poll_fn(|cx| {
            assert!(matches!(drain.as_mut().poll(cx), std::task::Poll::Pending));
            std::task::Poll::Ready(())
        })
        .await;
    }
    backend.released.add_permits(1);
    assert!(matches!(
        call.next().await,
        Some(ServerToolObservation::Final(ServerToolOutcome::Result(_)))
    ));
    owner.finish().await?;
    assert_eq!(backend.effects.load(Ordering::Acquire), 1);
    Ok(())
}
