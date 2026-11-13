//! In-memory input-receipt fences, scoped to one actual native launch.
use super::*;
use chapchoo_harness::NativeAuthenticationBinding;
use std::{collections::BTreeMap, sync::Weak};
use tokio::sync::watch;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Outcome {
    Pending,
    Recorded,
    Failed,
}
#[derive(Default)]
struct Pending {
    closed: bool,
    inputs: BTreeMap<String, watch::Sender<Outcome>>,
}
pub(in crate::connector) struct Owner {
    binding: NativeAuthenticationBinding,
    pending: Mutex<Pending>,
}
#[derive(Clone, Default)]
pub(in crate::connector) struct Ordering(Arc<Mutex<Option<Weak<Owner>>>>);
pub(in crate::connector) struct Ticket {
    owner: Arc<Owner>,
    id: String,
    recorded: bool,
}
pub(in crate::connector) struct Wait(Vec<watch::Receiver<Outcome>>);
fn same_owner(one: &NativeAuthenticationBinding, two: &NativeAuthenticationBinding) -> bool {
    one.agent_id == two.agent_id
        && one.conversation_id == two.conversation_id
        && one.operation_id == two.operation_id
        && one.execution_id == two.execution_id
        && one.authentication_id == two.authentication_id
}
impl Ordering {
    pub(in crate::connector) fn open(&self, binding: NativeAuthenticationBinding) -> Arc<Owner> {
        let owner = Arc::new(Owner {
            binding,
            pending: Mutex::new(Pending::default()),
        });
        if let Some(previous) = lock(&self.0)
            .replace(Arc::downgrade(&owner))
            .and_then(|owner| owner.upgrade())
        {
            previous.close();
        }
        owner
    }
    pub(in crate::connector) fn begin(
        &self,
        binding: &NativeAuthenticationBinding,
        id: &str,
    ) -> Result<Ticket, AppError> {
        let owner = lock(&self.0)
            .as_ref()
            .and_then(Weak::upgrade)
            .ok_or("Native publication owner is absent")?;
        if !same_owner(&owner.binding, binding) {
            return Err("Native publication owner changed".into());
        }
        {
            let mut pending = lock(&owner.pending);
            if pending.closed || pending.inputs.len() >= 32 || pending.inputs.contains_key(id) {
                return Err("Native publication ordering is unavailable".into());
            }
            pending
                .inputs
                .insert(id.into(), watch::channel(Outcome::Pending).0);
        }
        Ok(Ticket {
            owner,
            id: id.into(),
            recorded: false,
        })
    }
}
impl Owner {
    #[cfg(test)]
    pub(super) fn waiting(&self) -> bool {
        lock(&self.pending)
            .inputs
            .values()
            .any(|sender| sender.receiver_count() > 0)
    }
    pub(in crate::connector) fn snapshot(&self) -> Result<Wait, AppError> {
        let pending = lock(&self.pending);
        if pending.closed {
            return Err("Input receipt publication failed; tool call was not submitted".into());
        }
        Ok(Wait(
            pending
                .inputs
                .values()
                .map(watch::Sender::subscribe)
                .collect(),
        ))
    }
    pub(super) fn close(&self) {
        let mut pending = lock(&self.pending);
        pending.closed = true;
        for sender in pending.inputs.values() {
            sender.send_replace(Outcome::Failed);
        }
    }
}
impl Wait {
    pub(in crate::connector) async fn ready(self) -> Result<(), AppError> {
        for mut receiver in self.0 {
            loop {
                let outcome = *receiver.borrow_and_update();
                match outcome {
                    Outcome::Recorded => break,
                    Outcome::Failed => {
                        return Err(
                            "Input receipt publication failed; tool call was not submitted".into(),
                        );
                    }
                    Outcome::Pending => receiver
                        .changed()
                        .await
                        .map_err(|_| "Input receipt owner lost")?,
                }
            }
        }
        Ok(())
    }
}
impl Ticket {
    pub(in crate::connector) fn recorded(&mut self) {
        let mut pending = lock(&self.owner.pending);
        if let Some(sender) = pending.inputs.remove(&self.id) {
            sender.send_replace(if pending.closed {
                Outcome::Failed
            } else {
                Outcome::Recorded
            });
        }
        self.recorded = true;
    }
}
impl Drop for Ticket {
    fn drop(&mut self) {
        if !self.recorded
            && let Some(sender) = lock(&self.owner.pending).inputs.remove(&self.id)
        {
            sender.send_replace(Outcome::Failed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        future::Future,
        pin::pin,
        task::{Context, Waker},
    };
    fn binding(id: &str) -> NativeAuthenticationBinding {
        NativeAuthenticationBinding {
            agent_id: "agent".into(),
            conversation_id: "conversation".into(),
            operation_id: id.into(),
            execution_id: "execution".into(),
            authentication_id: "authentication".into(),
            source: [1; 24],
        }
    }
    #[tokio::test]
    async fn publication_waits_for_exact_ack_without_waiting_for_future_inputs()
    -> Result<(), AppError> {
        let ordering = Ordering::default();
        let binding = binding("first");
        let owner = ordering.open(binding.clone());
        let mut first = ordering.begin(&binding, "input")?;
        let mut waiting = pin!(owner.snapshot()?.ready());
        assert!(
            waiting
                .as_mut()
                .poll(&mut Context::from_waker(Waker::noop()))
                .is_pending()
        );
        let _later = ordering.begin(&binding, "later-input")?;
        first.recorded();
        waiting.await?;
        assert!(!owner.waiting());
        Ok(())
    }
    #[tokio::test]
    async fn failed_or_retired_owner_fails_closed_and_old_ack_cannot_open_successor()
    -> Result<(), AppError> {
        let ordering = Ordering::default();
        let old_binding = binding("old");
        let old = ordering.open(old_binding.clone());
        let mut old_ticket = ordering.begin(&old_binding, "same-id")?;
        let old_wait = old.snapshot()?;
        let next_binding = binding("next");
        let next = ordering.open(next_binding.clone());
        let next_ticket = ordering.begin(&next_binding, "same-id")?;
        let mut next_wait = pin!(next.snapshot()?.ready());
        old_ticket.recorded();
        assert!(old_wait.ready().await.is_err());
        assert!(ordering.begin(&old_binding, "stale").is_err());
        assert!(
            next_wait
                .as_mut()
                .poll(&mut Context::from_waker(Waker::noop()))
                .is_pending()
        );
        drop(next_ticket);
        assert!(next_wait.await.is_err());
        let mut retry = ordering.begin(&next_binding, "manual-retry")?;
        let retry_wait = next.snapshot()?;
        retry.recorded();
        retry_wait.ready().await?;
        next.close();
        assert!(next.snapshot().is_err());
        Ok(())
    }
}
