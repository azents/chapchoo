use super::*;
use chapchoo_domain::{SnapshotId, SourcePosition};
use std::sync::Mutex as SyncMutex;

/// An execution-local target, never reconstructed from a historical protocol ID.
#[derive(Clone, PartialEq, Eq)]
pub struct NativeWorkTarget {
    pub snapshot: SnapshotId,
    pub native_session: NativeConversationId,
    pub work_id: String,
}
impl fmt::Debug for NativeWorkTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("NativeWorkTarget([REDACTED])")
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeForeground {
    Unknown,
    Idle,
    Running,
    RequiresAction,
    OtherActive,
}
#[derive(Debug, Clone)]
pub struct NativeObservation {
    pub target: Option<NativeTextTarget>,
    pub position: Option<SourcePosition>,
    pub foreground: NativeForeground,
    pub work: Option<NativeWorkTarget>,
    pub preparation: Option<crate::NativePreparationObservation>,
    pub authentication_id: Option<String>,
}
#[derive(Default)]
pub(super) struct Observations(SyncMutex<Observed>, tokio::sync::Notify);
#[derive(Default)]
struct Observed {
    position: Option<SourcePosition>,
    source: Option<(SnapshotId, NativeConversationId)>,
    observed_session: Option<NativeConversationId>,
    foreground: Option<NativeForeground>,
    work: Option<String>,
    incoming: BTreeMap<schema::RequestId, SourcePosition>,
}
#[derive(Deserialize)]
struct IncomingRequest {
    id: schema::RequestId,
}
impl Observations {
    pub(super) fn changed(&self) {
        self.1.notify_one();
    }
    #[cfg(test)]
    pub(super) async fn wait_change(&self) {
        self.1.notified().await;
    }
    pub(super) fn captured(&self, line: &str, position: SourcePosition) -> Result<(), Error> {
        #[derive(Deserialize)]
        struct Method {
            method: String,
        }
        let method: Method = serde_json::from_str(line).map_err(|_| Error::Protocol)?;
        if method.method == "session/update" {
            #[derive(Deserialize)]
            struct Notification {
                params: schema::UpdateSessionNotification,
            }
            let update: Notification = serde_json::from_str(line).map_err(|_| Error::Protocol)?;
            return self.update_at(update.params, Some(position));
        }
        if !matches!(
            method.method.as_str(),
            "session/request_permission" | "elicitation/create"
        ) {
            return Ok(());
        }
        let incoming: IncomingRequest = serde_json::from_str(line).map_err(|_| Error::Protocol)?;
        let mut state = self.0.lock().map_err(|_| Error::Protocol)?;
        if state.incoming.len() >= MAX_REQUESTS || state.incoming.contains_key(&incoming.id) {
            return Err(Error::NotAdmitted);
        }
        state.incoming.insert(incoming.id, position);
        Ok(())
    }
    pub(super) fn take_request(&self, id: &schema::RequestId) -> Result<SourcePosition, Error> {
        self.0
            .lock()
            .map_err(|_| Error::Protocol)?
            .incoming
            .remove(id)
            .ok_or(Error::NotAdmitted)
    }
    pub(super) fn bind(
        &self,
        snapshot: SnapshotId,
        native: NativeConversationId,
    ) -> Result<(), Error> {
        let mut state = self.0.lock().map_err(|_| Error::Protocol)?;
        if state
            .source
            .as_ref()
            .is_some_and(|s| s != &(snapshot.clone(), native.clone()))
        {
            return Err(Error::CaptureConflict);
        }
        if state
            .observed_session
            .as_ref()
            .is_some_and(|id| id != &native)
        {
            return Err(Error::CaptureConflict);
        }
        state.source = Some((snapshot, native));
        Ok(())
    }
    #[cfg(test)]
    pub(super) fn update(&self, update: schema::UpdateSessionNotification) -> Result<(), Error> {
        self.update_at(update, None)
    }
    fn update_at(
        &self,
        update: schema::UpdateSessionNotification,
        position: Option<SourcePosition>,
    ) -> Result<(), Error> {
        let schema::SessionUpdate::StateUpdate(update_state) = update.update else {
            return Ok(());
        };
        let mut state = self.0.lock().map_err(|_| Error::Protocol)?;
        if let Some((_, native)) = &state.source
            && native.as_str() != update.session_id.to_string()
        {
            return Err(Error::CaptureConflict);
        }
        let native = NativeConversationId::parse(update.session_id.to_string())
            .map_err(|_| Error::Protocol)?;
        if state
            .observed_session
            .as_ref()
            .is_some_and(|id| id != &native)
        {
            return Err(Error::CaptureConflict);
        }
        state.observed_session = Some(native);
        let foreground = match update_state {
            schema::StateUpdate::Idle(_) => NativeForeground::Idle,
            schema::StateUpdate::Running(_) => NativeForeground::Running,
            schema::StateUpdate::RequiresAction(_) => NativeForeground::RequiresAction,
            _ => NativeForeground::OtherActive,
        };
        if foreground == NativeForeground::Idle {
            state.work = None;
        } else if state.work.is_none() {
            state.work = Some(token("work")?);
        }
        state.foreground = Some(foreground);
        state.position = position;
        Ok(())
    }
    pub(super) fn current(&self) -> Result<NativeObservation, Error> {
        let state = self.0.lock().map_err(|_| Error::Protocol)?;
        Ok(NativeObservation {
            target: state
                .source
                .as_ref()
                .map(|(snapshot, native)| NativeTextTarget {
                    snapshot: snapshot.clone(),
                    native_session: native.clone(),
                }),
            position: state.position.clone(),
            foreground: state.foreground.unwrap_or(NativeForeground::Unknown),
            preparation: None,
            authentication_id: None,
            work: match (&state.source, &state.work) {
                (Some((snapshot, native)), Some(work)) => Some(NativeWorkTarget {
                    snapshot: snapshot.clone(),
                    native_session: native.clone(),
                    work_id: work.clone(),
                }),
                _ => None,
            },
        })
    }
    #[cfg(test)]
    pub(super) fn matches(&self, target: &NativeWorkTarget) -> Result<bool, Error> {
        Ok(self.current()?.work.as_ref() == Some(target))
    }
    pub(super) fn with_current<T>(
        &self,
        target: &NativeWorkTarget,
        effect: impl FnOnce() -> Result<T, Error>,
    ) -> Result<T, Error> {
        let state = self.0.lock().map_err(|_| Error::Protocol)?;
        if state.source.as_ref().is_none_or(|(snapshot, native)| {
            snapshot != &target.snapshot || native != &target.native_session
        }) || state.work.as_ref() != Some(&target.work_id)
        {
            return Err(Error::NotAdmitted);
        }
        effect()
    }
}
pub(super) fn token(prefix: &str) -> Result<String, Error> {
    let digest = chapchoo_crypto::BearerSecret::generate()
        .map_err(|_| Error::Process)?
        .digest();
    Ok(format!(
        "{prefix}-{}",
        digest
            .as_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    type TestError = Box<dyn std::error::Error>;
    fn update(state: schema::StateUpdate) -> schema::UpdateSessionNotification {
        schema::UpdateSessionNotification::new("native", schema::SessionUpdate::StateUpdate(state))
    }
    #[test]
    fn work_targets_expire_on_idle_and_cannot_target_a_later_task() -> Result<(), TestError> {
        let observations = Observations::default();
        observations.bind(
            SnapshotId::parse("snapshot".into())?,
            NativeConversationId::parse("native".into())?,
        )?;
        assert!(observations.current()?.work.is_none());
        observations.update(update(schema::StateUpdate::Running(
            schema::RunningStateUpdate::new(),
        )))?;
        let first = observations.current()?.work.ok_or("First work")?;
        observations.update(update(schema::StateUpdate::RequiresAction(
            schema::RequiresActionStateUpdate::new(),
        )))?;
        assert!(observations.matches(&first)?);
        observations.update(update(schema::StateUpdate::Idle(
            schema::IdleStateUpdate::new(),
        )))?;
        assert!(!observations.matches(&first)?);
        let published = std::cell::Cell::new(false);
        assert!(matches!(
            observations.with_current(&first, || {
                published.set(true);
                Ok(())
            }),
            Err(Error::NotAdmitted)
        ));
        assert!(!published.get());
        observations.update(update(schema::StateUpdate::Running(
            schema::RunningStateUpdate::new(),
        )))?;
        let second = observations.current()?.work.ok_or("Second work")?;
        assert_ne!(first, second);
        assert!(!observations.matches(&first)?);
        observations.with_current(&second, || {
            published.set(true);
            Ok(())
        })?;
        assert!(published.get());
        Ok(())
    }
    #[test]
    fn request_correlation_preserves_json_rpc_id_type_and_occurrence_generation()
    -> Result<(), TestError> {
        let state = Observations::default();
        let first = SourcePosition::new(
            1,
            chapchoo_domain::EventOccurrenceId::parse("first".into())?,
        )?;
        let second = SourcePosition::new(
            2,
            chapchoo_domain::EventOccurrenceId::parse("second".into())?,
        )?;
        state.captured(
            r#"{"id":1,"method":"session/request_permission"}"#,
            first.clone(),
        )?;
        state.captured(
            r#"{"id":"1","method":"session/request_permission"}"#,
            second.clone(),
        )?;
        assert_eq!(state.take_request(&schema::RequestId::Number(1))?, first);
        assert_eq!(
            state.take_request(&schema::RequestId::Str("1".into()))?,
            second
        );
        assert!(state.take_request(&schema::RequestId::Number(1)).is_err());
        let third = SourcePosition::new(
            3,
            chapchoo_domain::EventOccurrenceId::parse("third".into())?,
        )?;
        state.captured(
            r#"{"id":1,"method":"session/request_permission"}"#,
            third.clone(),
        )?;
        assert_eq!(state.take_request(&schema::RequestId::Number(1))?, third);
        assert_ne!(token("request")?, token("request")?);
        Ok(())
    }
}
