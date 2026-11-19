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
/// A normal terminal occurrence from the current live root.
/// Report consumers must still authorize the source and deduplicate.
#[derive(Clone, PartialEq, Eq)]
pub struct NativeNormalEnding {
    pub work: NativeWorkTarget,
    pub position: SourcePosition,
}
impl fmt::Debug for NativeNormalEnding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("NativeNormalEnding([REDACTED])")
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
    pub applied_supply: Option<HarnessSupply>,
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
    ending_evidence_enabled: bool,
    live_running: bool,
    stop_published: bool,
    normal_ending: Option<NativeNormalEnding>,
    incoming: BTreeMap<schema::RequestId, SourcePosition>,
    activity: super::activity::ActivityTracker,
}
#[derive(Deserialize)]
struct IncomingRequest {
    id: schema::RequestId,
}
impl Observations {
    pub(super) fn record_live(&self, source: &crate::CaptureLog, line: &str) -> Result<(), Error> {
        let envelope: SessionEnvelope = serde_json::from_str(line).map_err(|_| Error::Protocol)?;
        let session = envelope.params.session_id.ok_or(Error::Protocol)?;
        if source
            .native_session()?
            .as_ref()
            .is_some_and(|root| root.as_str() != session)
        {
            let method: IncomingEnvelope =
                serde_json::from_str(line).map_err(|_| Error::Protocol)?;
            let mut state = self.0.lock().map_err(|_| Error::Protocol)?;
            if !state.activity.contains_child(&session) {
                state.activity.mark_uncertain();
            } else if method.method.as_deref() == Some("session/update") {
                // Incomplete work evidence blocks refresh, not the lifetime of
                // the already-started native owner.
                let _ = state.activity.observe(line);
            } else {
                state.activity.child_request(&session);
            }
            // Child updates are live work evidence, not root journal records.
            // Child requests get an explicit unsupported SDK response.
            return Ok(());
        }
        // Capture and transition share Stop's single ordering boundary.
        let mut state = self.0.lock().map_err(|_| Error::Protocol)?;
        let position = source.append(line, None)?;
        Self::apply_captured(&mut state, line, position, true)
    }
    pub(super) fn accepts_root_request(&self, session: &schema::SessionId) -> bool {
        self.0.lock().is_ok_and(|state| {
            state
                .source
                .as_ref()
                .is_some_and(|(_, root)| root.as_str() == session.to_string())
        })
    }
    pub(super) fn changed(&self) {
        self.1.notify_one();
    }
    #[cfg(test)]
    pub(super) async fn wait_change(&self) {
        self.1.notified().await;
    }
    #[cfg(test)]
    pub(super) fn captured(&self, line: &str, position: SourcePosition) -> Result<(), Error> {
        let mut state = self.0.lock().map_err(|_| Error::Protocol)?;
        Self::apply_captured(&mut state, line, position, false)
    }
    fn apply_captured(
        state: &mut Observed,
        line: &str,
        position: SourcePosition,
        live: bool,
    ) -> Result<(), Error> {
        #[derive(Deserialize)]
        struct Method {
            method: String,
        }
        let method: Method = serde_json::from_str(line).map_err(|_| Error::Protocol)?;
        if method.method == "session/update" {
            let _ = state.activity.observe(line);
            #[derive(Deserialize)]
            struct UpdateKind {
                params: UpdateParams,
            }
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct UpdateParams {
                session_id: String,
                update: Kind,
            }
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Kind {
                session_update: String,
            }
            let kind: UpdateKind = serde_json::from_str(line).map_err(|_| Error::Protocol)?;
            if kind.params.update.session_update != "state_update" {
                return Ok(());
            }
            if state
                .source
                .as_ref()
                .is_some_and(|(_, root)| root.as_str() != kind.params.session_id)
            {
                return Ok(());
            }
            #[derive(Deserialize)]
            struct Notification {
                params: schema::UpdateSessionNotification,
            }
            let update: Notification = serde_json::from_str(line).map_err(|_| Error::Protocol)?;
            return Self::update_at(state, update.params, Some(position), live);
        }
        if !matches!(
            method.method.as_str(),
            "session/request_permission" | "elicitation/create"
        ) {
            return Ok(());
        }
        let incoming: IncomingRequest = serde_json::from_str(line).map_err(|_| Error::Protocol)?;
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
        empty_scope: bool,
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
        state.activity.bind(native.as_str(), empty_scope)?;
        state.source = Some((snapshot, native));
        Ok(())
    }
    /// Session setup and catalogue verification must finish before any
    /// Running observation can qualify a subsequent ending.
    pub(super) fn enable_normal_endings(&self) -> Result<(), Error> {
        let mut state = self.0.lock().map_err(|_| Error::Protocol)?;
        if state.source.is_none() || state.ending_evidence_enabled {
            return Err(Error::NotAdmitted);
        }
        state.ending_evidence_enabled = true;
        Ok(())
    }
    pub(super) fn quiescent(&self) -> bool {
        self.0.lock().is_ok_and(|state| {
            state
                .source
                .as_ref()
                .is_some_and(|(_, native)| state.activity.is_quiescent(native.as_str()))
        })
    }
    #[cfg(test)]
    pub(super) fn update(&self, update: schema::UpdateSessionNotification) -> Result<(), Error> {
        let mut state = self.0.lock().map_err(|_| Error::Protocol)?;
        Self::update_at(&mut state, update, None, false)
    }
    fn update_at(
        state: &mut Observed,
        update: schema::UpdateSessionNotification,
        position: Option<SourcePosition>,
        live: bool,
    ) -> Result<(), Error> {
        let schema::SessionUpdate::StateUpdate(update_state) = update.update else {
            return Ok(());
        };
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
        let normal = matches!(&update_state, schema::StateUpdate::Idle(idle)
            if matches!(&idle.stop_reason, Some(schema::StopReason::EndTurn)));
        let foreground = match update_state {
            schema::StateUpdate::Idle(_) => NativeForeground::Idle,
            schema::StateUpdate::Running(_) => NativeForeground::Running,
            schema::StateUpdate::RequiresAction(_) => NativeForeground::RequiresAction,
            _ => NativeForeground::OtherActive,
        };
        if foreground == NativeForeground::Idle {
            if live
                && state.ending_evidence_enabled
                && normal
                && state.live_running
                && !state.stop_published
                && let (Some((snapshot, native)), Some(work_id), Some(position)) =
                    (&state.source, &state.work, &position)
            {
                state.normal_ending = Some(NativeNormalEnding {
                    work: NativeWorkTarget {
                        snapshot: snapshot.clone(),
                        native_session: native.clone(),
                        work_id: work_id.clone(),
                    },
                    position: position.clone(),
                });
            }
            state.work = None;
            state.live_running = false;
            state.stop_published = false;
        } else {
            if state.work.is_none() {
                state.work = Some(token("work")?);
                state.normal_ending = None;
            }
            if live && state.ending_evidence_enabled && foreground == NativeForeground::Running {
                state.live_running = true;
            } else if foreground == NativeForeground::OtherActive {
                state.live_running = false;
            }
        }
        state.foreground = Some(foreground);
        state.position = position;
        Ok(())
    }
    pub(super) fn normal_ending(&self) -> Result<Option<NativeNormalEnding>, Error> {
        let state = self.0.lock().map_err(|_| Error::Protocol)?;
        Ok(state.normal_ending.clone())
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
            applied_supply: None,
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
    pub(super) fn publish_stop(
        &self,
        target: &NativeWorkTarget,
        publish: impl FnOnce() -> Result<(), Error>,
    ) -> Result<(), Error> {
        let mut state = self.0.lock().map_err(|_| Error::Protocol)?;
        if state.source.as_ref().is_none_or(|(snapshot, native)| {
            snapshot != &target.snapshot || native != &target.native_session
        }) || state.work.as_ref() != Some(&target.work_id)
        {
            return Err(Error::NotAdmitted);
        }
        publish()?;
        state.stop_published = true;
        Ok(())
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
    fn state_line(session: &str, state: &str, reason: Option<&str>) -> String {
        let mut update = serde_json::json!({"sessionUpdate":"state_update","state":state});
        if let Some(reason) = reason {
            update["stopReason"] = reason.into();
        }
        serde_json::json!({"method":"session/update","params":{"sessionId":session,"update":update}})
            .to_string()
    }
    fn fixture() -> Result<(tempfile::TempDir, crate::CaptureLog, Observations), TestError> {
        let root = tempfile::tempdir()?;
        let capture = crate::CaptureLog::create(&NativeStorage::open(root.path())?)?;
        let native = NativeConversationId::parse("native".into())?;
        capture.bind_native(&native)?;
        let observations = Observations::default();
        observations.bind(capture.snapshot()?, native, false)?;
        Ok((root, capture, observations))
    }
    #[test]
    fn normal_ending_reuses_exact_live_work_and_occurrence_until_next_work() -> Result<(), TestError>
    {
        let (_root, capture, observations) = fixture()?;
        observations.enable_normal_endings()?;
        for reason in [None, Some("end_turn")] {
            observations.record_live(&capture, &state_line("native", "idle", reason))?;
            assert!(observations.normal_ending()?.is_none());
        }
        let running = state_line("native", "running", None);
        let ended = state_line("native", "idle", Some("end_turn"));
        observations.record_live(&capture, &running)?;
        let work = observations.current()?.work.ok_or("running work")?;
        observations.record_live(&capture, &state_line("child", "idle", Some("end_turn")))?;
        assert!(observations.normal_ending()?.is_none());
        observations.record_live(&capture, &ended)?;
        let ending = observations.normal_ending()?.ok_or("qualified ending")?;
        assert_eq!(ending.work, work);
        assert_eq!(
            Some(ending.position.clone()),
            observations.current()?.position
        );
        assert!(observations.current()?.work.is_none());
        observations.record_live(&capture, &ended)?;
        assert_ne!(
            Some(ending.position.clone()),
            observations.current()?.position
        );
        assert_eq!(observations.normal_ending()?, Some(ending));
        observations.record_live(&capture, &running)?;
        assert!(observations.normal_ending()?.is_none());
        Ok(())
    }

    #[test]
    fn stop_and_abnormal_endings_do_not_qualify_or_poison_later_work() -> Result<(), TestError> {
        let (_root, capture, observations) = fixture()?;
        observations.enable_normal_endings()?;
        let running = state_line("native", "running", None);
        let ended = state_line("native", "idle", Some("end_turn"));
        observations.record_live(&capture, &running)?;
        let stopped = observations.current()?.work.ok_or("running work")?;
        std::thread::scope(|scope| -> Result<(), TestError> {
            let (stop_has_target, wait_for_stop) = std::sync::mpsc::channel();
            let (end_started, wait_for_end) = std::sync::mpsc::channel();
            let stop = scope.spawn(|| {
                observations.publish_stop(&stopped, move || {
                    stop_has_target.send(()).map_err(|_| Error::Process)?;
                    wait_for_end
                        .recv_timeout(Duration::from_secs(5))
                        .map_err(|_| Error::Process)
                })
            });
            wait_for_stop.recv_timeout(Duration::from_secs(5))?;
            end_started.send(())?;
            observations.record_live(&capture, &ended)?;
            stop.join().map_err(|_| "stop thread panicked")??;
            Ok(())
        })?;
        assert!(observations.normal_ending()?.is_none());
        for reason in [Some("cancelled"), Some("error"), None] {
            observations.record_live(&capture, &running)?;
            observations.record_live(&capture, &state_line("native", "idle", reason))?;
            assert!(observations.normal_ending()?.is_none());
        }
        observations.record_live(&capture, &running)?;
        assert!(matches!(
            observations.publish_stop(&stopped, || Ok(())),
            Err(Error::NotAdmitted)
        ));
        observations.record_live(&capture, &ended)?;
        let later = observations
            .normal_ending()?
            .ok_or("later qualified ending")?;
        assert_ne!(later.work, stopped);
        assert!(matches!(
            observations.publish_stop(&later.work, || Ok(())),
            Err(Error::NotAdmitted)
        ));
        assert_eq!(observations.normal_ending()?, Some(later));
        Ok(())
    }

    #[test]
    fn historical_capture_cannot_mint_or_prime_live_ending_evidence() -> Result<(), TestError> {
        let (_root, capture, observations) = fixture()?;
        observations.enable_normal_endings()?;
        let running = state_line("native", "running", None);
        let ended = state_line("native", "idle", Some("end_turn"));
        let historic_start = capture.append(&running, None)?;
        let historic_end = capture.append(&ended, None)?;
        observations.captured(&running, historic_start.clone())?;
        observations.captured(&ended, historic_end.clone())?;
        assert!(observations.normal_ending()?.is_none());
        observations.captured(&running, historic_start)?;
        observations.record_live(&capture, &ended)?;
        assert!(observations.normal_ending()?.is_none());
        observations.record_live(&capture, &running)?;
        observations.captured(&ended, historic_end)?;
        assert!(observations.normal_ending()?.is_none());
        Ok(())
    }

    #[test]
    fn work_targets_expire_on_idle_and_cannot_target_a_later_task() -> Result<(), TestError> {
        let observations = Observations::default();
        observations.bind(
            SnapshotId::parse("snapshot".into())?,
            NativeConversationId::parse("native".into())?,
            true,
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
            observations.publish_stop(&first, || {
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
        observations.publish_stop(&second, || {
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
