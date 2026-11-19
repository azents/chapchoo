//! Scoped read for the managed Codex SessionStart hook.
use super::observations::Observations;
use crate::{ContinuationRoute, ContinuationRouteLease, NativeGoalRestore, NativeWorkTarget};
use std::{path::Path, sync::Arc, time::Duration};
use tokio::{net::UnixListener, task::JoinHandle, time::timeout};

pub(super) const SOCKET: &str = "goal.sock";
pub(super) struct Listener {
    lease: Option<ContinuationRouteLease>,
    task: JoinHandle<()>,
}

impl Listener {
    pub(super) fn start(
        launch: &Path,
        observations: Arc<Observations>,
        read: NativeGoalRestore,
    ) -> std::io::Result<Self> {
        let listener = UnixListener::bind(launch.join(SOCKET))?;
        let lease = ContinuationRouteLease::open();
        let lifetime = lease.route();
        let task = tokio::spawn(async move {
            while let Ok((link, _)) = listener.accept().await {
                let Some(work) = observations.restore_work() else {
                    continue;
                };
                let Ok(Ok(context)) = timeout(Duration::from_millis(900), read(work.clone())).await
                else {
                    continue;
                };
                let response = if context.projection == b"[]" {
                    Some(Vec::from(&b"{}"[..]))
                } else {
                    format_context(context.projection)
                };
                if let Some(response) = response
                    && matches!(
                        timeout(Duration::from_millis(100), link.writable()).await,
                        Ok(Ok(()))
                    )
                {
                    publish(&context.route, &lifetime, &observations, &work, || {
                        // A short write is rejected by the relay's JSON decoder.
                        let _ = link.try_write(&response);
                    });
                }
            }
        });
        Ok(Self {
            lease: Some(lease),
            task,
        })
    }
}

impl Drop for Listener {
    fn drop(&mut self) {
        drop(self.lease.take());
        self.task.abort();
    }
}

fn publish(
    route: &ContinuationRoute,
    lifetime: &ContinuationRoute,
    observations: &Observations,
    work: &NativeWorkTarget,
    write: impl FnOnce(),
) {
    let _ = lifetime.publish(|| {
        route.publish(|| {
            observations.publish_restore(work, write);
            Ok(())
        })
    });
}

fn format_context(bytes: Vec<u8>) -> Option<Vec<u8>> {
    if bytes.is_empty() || bytes.len() > 8 * 1024 {
        return None;
    }
    let context = String::from_utf8(bytes).ok()?;
    serde_json::to_vec(&serde_json::json!({
        "hookSpecificOutput": {
            "hookEventName": "SessionStart",
            "additionalContext": format!("Current Conversation Goal context (authoritative):\n{context}")
        }
    }))
    .ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CaptureLog, NativeStorage};
    use chapchoo_domain::NativeConversationId;
    use std::cell::Cell;

    #[test]
    fn ready_reply_cannot_publish_after_route_or_launch_withdrawal()
    -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let capture = CaptureLog::create(&NativeStorage::open(dir.path())?)?;
        let native = NativeConversationId::parse("fixture-root".into())?;
        capture.bind_native(&native)?;
        let observations = Observations::default();
        observations.bind(capture.snapshot()?, native, false)?;
        observations.record_live(
            &capture,
            r#"{"method":"session/update","params":{"sessionId":"fixture-root","update":{"sessionUpdate":"state_update","state":"running"}}}"#,
        )?;
        let work = observations.restore_work().ok_or("No live fixture work")?;
        let route = ContinuationRouteLease::open();
        let lifetime = ContinuationRouteLease::open();
        let context = crate::NativeGoalContext::new(b"[]".to_vec(), route.route());
        let writes = Cell::new(0);
        let write = || writes.set(writes.get() + 1);
        publish(
            &context.route,
            &lifetime.route(),
            &observations,
            &work,
            write,
        );
        assert_eq!(writes.get(), 1);

        drop(route);
        publish(
            &context.route,
            &lifetime.route(),
            &observations,
            &work,
            write,
        );
        let launch = lifetime.route();
        drop(lifetime);
        let replacement = ContinuationRouteLease::open();
        publish(&replacement.route(), &launch, &observations, &work, write);
        assert_eq!(writes.get(), 1);
        Ok(())
    }
}
