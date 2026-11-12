use super::*;
mod lifetime;
mod panic;
use crate::DomainReason;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::{Semaphore, mpsc};

type TestError = Box<dyn std::error::Error + Send + Sync>;
struct Fixture {
    allowed: AtomicBool,
    released: Semaphore,
    started: mpsc::Sender<(String, Vec<u8>)>,
    effects: AtomicUsize,
    cancelled: AtomicUsize,
}
impl Fixture {
    fn new() -> (Arc<Self>, mpsc::Receiver<(String, Vec<u8>)>) {
        let (started, receiver) = mpsc::channel(256);
        (
            Arc::new(Self {
                allowed: AtomicBool::new(true),
                released: Semaphore::new(0),
                started,
                effects: AtomicUsize::new(0),
                cancelled: AtomicUsize::new(0),
            }),
            receiver,
        )
    }
}
impl ServerToolBackend for Fixture {
    fn authorize<'a>(&'a self, _: &'a ServerToolScope) -> ServerToolFuture<'a, ()> {
        Box::pin(async {
            if self.allowed.load(Ordering::Acquire) {
                Ok(())
            } else {
                Err(crate::ServiceError::domain(DomainReason::PermissionDenied))
            }
        })
    }
    fn catalog(&self) -> Result<Vec<u8>, ServiceError> {
        Ok(br#"{"tools":[]}"#.to_vec())
    }
    fn validate(&self, name: &str, _: &[u8]) -> Result<(), ServerToolValidation> {
        match name {
            "block" | "oversize" => Ok(()),
            "invalid" => Err(ServerToolValidation::InvalidArguments),
            _ => Err(ServerToolValidation::UnknownTool),
        }
    }
    fn execute<'a>(
        &'a self,
        request: &'a ServerToolRequest,
        cancel: ToolCancellation,
    ) -> ServerToolFuture<'a, Vec<u8>> {
        Box::pin(async move {
            let _ = self
                .started
                .send((request.scope.agent_id.clone(), request.arguments.clone()))
                .await;
            if request.arguments == b"panic" {
                panic_fixture();
            }
            let permit = self
                .released
                .acquire()
                .await
                .map_err(ServiceError::infrastructure)?;
            permit.forget();
            self.effects.fetch_add(1, Ordering::AcqRel);
            if cancel.is_cancelled() {
                self.cancelled.fetch_add(1, Ordering::AcqRel);
            }
            if request.name == "oversize" {
                Ok(vec![0; MAX_TOOL_PAYLOAD + 1])
            } else {
                Ok(br#"{"content":[{"type":"text","text":"done"}],"structuredContent":{"exact":9223372036854775807},"isError":false}"#.to_vec())
            }
        })
    }
}
#[expect(
    clippy::panic,
    reason = "Inject an actual worker unwind to verify owned failure cleanup"
)]
fn panic_fixture() -> ! {
    panic!("Synthetic tool worker failure");
}

fn request(agent: &str, label: &str) -> ServerToolRequest {
    ServerToolRequest {
        scope: ServerToolScope {
            session_id: "session".into(),
            control_id: "control".into(),
            connection_id: "connection".into(),
            credential: SecretDigest::from_bytes([0; 32]),
            agent_id: agent.into(),
            conversation_id: "conversation".into(),
        },
        name: "block".into(),
        arguments: label.as_bytes().to_vec(),
    }
}
async fn running(
    owner: &ServerTools,
    agent: &str,
    label: &str,
) -> Result<ToolInvocation, TestError> {
    let mut call = owner.submit(request(agent, label))?;
    assert!(matches!(
        call.next().await,
        Some(ServerToolObservation::Started)
    ));
    Ok(call)
}

#[tokio::test]
async fn direct_result_preserves_representation_and_explicit_cancel_does_not_recall()
-> Result<(), TestError> {
    let (backend, mut started) = Fixture::new();
    let owner = ServerTools::new(backend.clone());
    let mut call = running(&owner, "A", "first").await?;
    started.recv().await.ok_or("Missing execution")?;
    call.control().cancel();
    backend.released.add_permits(1);
    let Some(ServerToolObservation::Final(ServerToolOutcome::Result(bytes))) = call.next().await
    else {
        return Err("Missing actual result".into());
    };
    assert!(String::from_utf8(bytes)?.contains("9223372036854775807"));
    assert!(call.next().await.is_none());
    owner.finish().await?;
    assert_eq!(backend.effects.load(Ordering::Acquire), 1);
    assert_eq!(backend.cancelled.load(Ordering::Acquire), 1);
    Ok(())
}

#[tokio::test]
async fn waiting_is_agent_bounded_and_cancelled_entries_cannot_execute() -> Result<(), TestError> {
    let (backend, _) = Fixture::new();
    let owner = ServerTools::new(backend.clone());
    let mut active = Vec::new();
    for _ in 0..16 {
        active.push(running(&owner, "A", "active").await?);
    }
    let mut queued = Vec::new();
    for _ in 0..8 {
        let mut call = owner.submit(request("A", "waiting"))?;
        assert!(matches!(
            call.next().await,
            Some(ServerToolObservation::Waiting)
        ));
        queued.push(call);
    }
    let mut rejected = owner.submit(request("A", "overflow"))?;
    assert!(matches!(
        rejected.next().await,
        Some(ServerToolObservation::Final(
            ServerToolOutcome::NotExecuted(ServerToolRejection::Capacity)
        ))
    ));
    for call in &mut queued {
        call.control().cancel();
        assert!(matches!(
            call.next().await,
            Some(ServerToolObservation::Final(
                ServerToolOutcome::NotExecuted(ServerToolRejection::Cancelled)
            ))
        ));
    }
    owner.close();
    backend.released.add_permits(16);
    for call in &mut active {
        assert!(matches!(
            call.next().await,
            Some(ServerToolObservation::Final(ServerToolOutcome::Result(_)))
        ));
    }
    owner.finish().await?;
    assert_eq!(backend.effects.load(Ordering::Acquire), 16);
    Ok(())
}

#[tokio::test]
async fn waiting_dispatch_is_round_robin_by_agent_and_fifo_inside_each_agent()
-> Result<(), TestError> {
    let (backend, mut starts) = Fixture::new();
    let owner = ServerTools::new(backend.clone());
    let mut active = Vec::new();
    for _ in 0..16 {
        active.push(running(&owner, "active", "active").await?);
        starts.recv().await.ok_or("Missing active call")?;
    }
    let mut queued = Vec::new();
    for (agent, label) in [("A", "A1"), ("A", "A2"), ("B", "B1"), ("B", "B2")] {
        let mut call = owner.submit(request(agent, label))?;
        assert!(matches!(
            call.next().await,
            Some(ServerToolObservation::Waiting)
        ));
        queued.push(call);
    }
    for expected in ["A1", "B1", "A2", "B2"] {
        backend.released.add_permits(1);
        let (_, label) = starts.recv().await.ok_or("Missing fair dispatch")?;
        assert_eq!(label, expected.as_bytes());
    }
    owner.close();
    backend.released.add_permits(20);
    drop(active);
    drop(queued);
    owner.finish().await?;
    assert_eq!(backend.effects.load(Ordering::Acquire), 20);
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn deadlines_retire_waiting_but_retain_started_owners_without_fake_cancellation()
-> Result<(), TestError> {
    let (backend, _) = Fixture::new();
    let owner = ServerTools::new(backend.clone());
    let mut active = Vec::new();
    for _ in 0..16 {
        active.push(running(&owner, "A", "active").await?);
    }
    let mut waiting = owner.submit(request("B", "expire"))?;
    assert!(matches!(
        waiting.next().await,
        Some(ServerToolObservation::Waiting)
    ));
    tokio::time::advance(std::time::Duration::from_secs(10)).await;
    assert!(matches!(
        waiting.next().await,
        Some(ServerToolObservation::Final(
            ServerToolOutcome::NotExecuted(ServerToolRejection::QueueExpired)
        ))
    ));
    tokio::time::advance(std::time::Duration::from_secs(40)).await;
    for call in &mut active {
        assert!(matches!(
            call.next().await,
            Some(ServerToolObservation::Final(ServerToolOutcome::Uncertain))
        ));
    }
    assert_eq!(owner.executing(), 16);
    assert_eq!(backend.effects.load(Ordering::Acquire), 0);
    owner.close();
    backend.released.add_permits(16);
    owner.finish().await?;
    assert_eq!(backend.effects.load(Ordering::Acquire), 16);
    assert_eq!(backend.cancelled.load(Ordering::Acquire), 0);
    Ok(())
}

#[tokio::test]
async fn stale_authority_denies_admission_and_protected_disclosure() -> Result<(), TestError> {
    let (backend, _) = Fixture::new();
    let owner = ServerTools::new(backend.clone());
    let mut call = running(&owner, "A", "accepted").await?;
    backend.allowed.store(false, Ordering::Release);
    backend.released.add_permits(1);
    assert!(matches!(
        call.next().await,
        Some(ServerToolObservation::Final(ServerToolOutcome::Uncertain))
    ));
    let mut denied = owner.submit(request("A", "denied"))?;
    assert!(matches!(
        denied.next().await,
        Some(ServerToolObservation::Final(
            ServerToolOutcome::NotExecuted(ServerToolRejection::Authority)
        ))
    ));
    owner.finish().await?;
    assert_eq!(backend.effects.load(Ordering::Acquire), 1);
    Ok(())
}

#[tokio::test]
async fn oversize_result_after_effect_is_uncertain_and_validation_is_not_a_started_call()
-> Result<(), TestError> {
    let (backend, _) = Fixture::new();
    let owner = ServerTools::new(backend.clone());
    let mut input = request("A", "oversize");
    input.name = "oversize".into();
    let mut call = owner.submit(input)?;
    assert!(matches!(
        call.next().await,
        Some(ServerToolObservation::Started)
    ));
    backend.released.add_permits(1);
    assert!(matches!(
        call.next().await,
        Some(ServerToolObservation::Final(
            ServerToolOutcome::UncertainRepresentation
        ))
    ));
    for (name, recognized) in [("unknown", false), ("invalid", true)] {
        let mut input = request("A", "invalid");
        input.name = name.into();
        let mut call = owner.submit(input)?;
        let final_event = call.next().await;
        if recognized {
            assert!(matches!(
                final_event,
                Some(ServerToolObservation::Final(ServerToolOutcome::Result(_)))
            ));
        } else {
            assert!(matches!(
                final_event,
                Some(ServerToolObservation::Final(
                    ServerToolOutcome::NotExecuted(ServerToolRejection::UnknownTool)
                ))
            ));
        }
    }
    owner.finish().await?;
    assert_eq!(backend.effects.load(Ordering::Acquire), 1);
    Ok(())
}
