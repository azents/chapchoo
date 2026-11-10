//! One invocation lease spans native result, durable report and ordered release.
use super::*;
use std::{collections::BTreeSet, sync::Mutex};
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Kind {
    Setup,
    Input,
    Stop,
    Answer,
}
type Key = (Kind, String);
pub(super) type Active = Arc<Mutex<BTreeSet<Key>>>;
struct Lease {
    key: Key,
    active: Active,
}
impl Drop for Lease {
    fn drop(&mut self) {
        if let Ok(mut active) = self.active.lock() {
            active.remove(&self.key);
        }
    }
}
pub(super) struct OperationPermit {
    _slot: OwnedSemaphorePermit,
    _lease: Lease,
}
impl Operations {
    pub(super) fn admit(&self, kind: Kind, id: &str) -> Result<Option<OperationPermit>, AppError> {
        if id.is_empty() || id.len() > 128 || !id.bytes().all(|byte| byte.is_ascii_graphic()) {
            return Err("Native operation identity is invalid".into());
        }
        let mut active = self
            .active
            .lock()
            .map_err(|_| "Native operation admission is unavailable")?;
        let key = (kind, id.into());
        if active.contains(&key) {
            return Ok(None);
        }
        let slot = match self.slots.clone().try_acquire_owned() {
            Ok(slot) => slot,
            Err(_) => return Ok(None),
        };
        active.insert(key.clone());
        Ok(Some(OperationPermit {
            _slot: slot,
            _lease: Lease {
                key,
                active: self.active.clone(),
            },
        }))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn settled_reports_release_capacity_only_after_the_command_loop_orders_native_release()
    -> Result<(), AppError> {
        let temporary = tempfile::tempdir()?;
        let native = chapchoo_harness::NativeOwner::new(chapchoo_harness::NativeStorage::open(
            temporary.path(),
        )?)?;
        let mut operations = Operations::new(native.handle());
        let channel = tonic::transport::Endpoint::from_static("http://127.0.0.1:1").connect_lazy();
        let key = BearerSecret::generate()?;
        for index in 0..300 {
            let id = format!("stop-{index}");
            let command = c::StopCommand {
                conversation_id: "conversation".into(),
                operation_id: id.clone(),
                target: Some(c::WorkTarget {
                    source: Some(c::NativeTarget {
                        snapshot_id: "source".into(),
                        native_session_id: "native".into(),
                    }),
                    work_id: "work".into(),
                }),
            };
            assert!(operations.stop(command.clone())?);
            let pending = operations
                .tasks
                .join_next()
                .await
                .ok_or("Missing native waiter")??
                .ok_or("Expected exact native rejection")?;
            // Simulate a completed authenticated report. A command whose server
            // check was already pending must still be denied by this lease.
            operations.settled.push(pending);
            assert!(!operations.stop(command.clone())?);
            operations.publish(channel.clone(), c::CaptureSource::default(), &key)?;
            assert!(!operations.stop(command)?);
            assert!(
                operations
                    .releasing
                    .join_next()
                    .await
                    .ok_or("Missing release waiter")??
                    .is_none()
            );
            assert!(operations.admit(Kind::Stop, &id)?.is_some());
        }
        operations.finish().await?;
        native.finish().await?;
        Ok(())
    }
    #[tokio::test]
    async fn lease_holds_duplicate_admission_until_release_and_keeps_kinds_distinct()
    -> Result<(), AppError> {
        let temporary = tempfile::tempdir()?;
        let native = chapchoo_harness::NativeOwner::new(chapchoo_harness::NativeStorage::open(
            temporary.path(),
        )?)?;
        let operations = Operations::new(native.handle());
        let original = operations
            .admit(Kind::Input, "same-operation")?
            .ok_or("Not admitted")?;
        assert!(operations.admit(Kind::Input, "same-operation")?.is_none());
        let setup = operations
            .admit(Kind::Setup, "same-operation")?
            .ok_or("Different kind blocked")?;
        drop(setup);
        assert!(operations.admit(Kind::Input, "same-operation")?.is_none());
        drop(original);
        assert!(operations.admit(Kind::Input, "same-operation")?.is_some());
        operations.finish().await?;
        native.finish().await?;
        Ok(())
    }
}
