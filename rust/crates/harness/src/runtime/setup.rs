//! Setup is execution-owned and idempotent independently of Control call IDs.
use super::*;
#[derive(Clone, PartialEq, Eq)]
pub enum NativeSetupIntent {
    Initial { allow_fresh: bool },
    Existing(NativeTextTarget),
}
#[derive(Clone, PartialEq, Eq)]
pub struct NativeSetupOperation {
    pub operation_id: String,
    pub supply: HarnessSupply,
    pub intent: NativeSetupIntent,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeSetupFailure {
    NotAdmitted,
    SourceUnavailable,
    BeforeNativeLaunch,
    NativeRejected,
    RetirementUnknown,
}
#[derive(Clone, PartialEq, Eq)]
pub enum NativeSetupObservation {
    Unseen,
    PossibleForwarding,
    Ready(NativeTextTarget),
    Failed(NativeSetupFailure),
}
macro_rules! safe_debug {($($name:ty),+)=>{$(impl fmt::Debug for $name {fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result {f.write_str(concat!(stringify!($name),"([REDACTED])"))}})+};}
safe_debug!(
    NativeSetupIntent,
    NativeSetupOperation,
    NativeSetupObservation
);
#[derive(Default)]
pub(super) struct SetupRegistry {
    entries: BTreeMap<String, (NativeSetupOperation, NativeSetupObservation)>,
}
impl SetupRegistry {
    pub(super) fn release_reported(
        &mut self,
        id: &str,
        expected: &NativeSetupObservation,
    ) -> Result<(), Error> {
        valid_id(id)?;
        if !matches!(
            expected,
            NativeSetupObservation::Ready(_) | NativeSetupObservation::Failed(_)
        ) {
            return Err(Error::OperationConflict);
        }
        if self
            .entries
            .get(id)
            .is_some_and(|(_, outcome)| outcome != expected)
        {
            return Err(Error::OperationConflict);
        }
        self.entries.remove(id);
        Ok(())
    }
    pub(super) fn begin(
        &mut self,
        operation: &NativeSetupOperation,
    ) -> Result<Option<NativeSetupObservation>, Error> {
        valid_id(&operation.operation_id)?;
        if let Some((previous, outcome)) = self.entries.get(&operation.operation_id) {
            if previous != operation {
                return Err(Error::OperationConflict);
            }
            return Ok(Some(outcome.clone()));
        }
        if self.entries.len() >= 256 {
            return Err(Error::NotAdmitted);
        }
        self.entries.insert(
            operation.operation_id.clone(),
            (
                operation.clone(),
                NativeSetupObservation::PossibleForwarding,
            ),
        );
        Ok(None)
    }
    pub(super) fn completed(
        &mut self,
        id: &str,
        outcome: NativeSetupObservation,
    ) -> Result<(), Error> {
        let (_, current) = self.entries.get_mut(id).ok_or(Error::OperationConflict)?;
        *current = outcome;
        Ok(())
    }
    pub(super) fn observe(&self, id: &str) -> Result<NativeSetupObservation, Error> {
        valid_id(id)?;
        Ok(self
            .entries
            .get(id)
            .map_or(NativeSetupObservation::Unseen, |(_, outcome)| {
                outcome.clone()
            }))
    }
}
fn valid_id(id: &str) -> Result<(), Error> {
    if id.is_empty() || id.len() > 128 || !id.bytes().all(|byte| byte.is_ascii_graphic()) {
        return Err(Error::NotAdmitted);
    }
    Ok(())
}
impl State {
    pub(super) async fn setup(
        &mut self,
        operation: NativeSetupOperation,
        authentication: Option<Arc<dyn NativeAuthenticationProvider>>,
    ) -> Result<NativeSetupObservation, Error> {
        if let Some(previous) = self.setups.begin(&operation)? {
            return Ok(previous);
        }
        let outcome = match self.setup_once(&operation, authentication).await {
            Ok(target) => NativeSetupObservation::Ready(target),
            Err(error) => NativeSetupObservation::Failed(match error {
                Error::NotAdmitted | Error::UnsupportedRuntime | Error::Authentication => {
                    NativeSetupFailure::NotAdmitted
                }
                Error::Storage
                | Error::CaptureUnavailable
                | Error::CaptureCorrupt
                | Error::CaptureConflict => NativeSetupFailure::SourceUnavailable,
                Error::Transient | Error::Integrity | Error::Incompatible => {
                    NativeSetupFailure::BeforeNativeLaunch
                }
                Error::ProtocolRejected { .. } => NativeSetupFailure::NativeRejected,
                Error::RetirementUnknown | Error::StartAndRetirement { .. } => {
                    NativeSetupFailure::RetirementUnknown
                }
                _ => {
                    return Err(Error::DeliveryUncertain);
                }
            }),
        };
        self.setups
            .completed(&operation.operation_id, outcome.clone())?;
        Ok(outcome)
    }
    async fn setup_once(
        &mut self,
        operation: &NativeSetupOperation,
        authentication: Option<Arc<dyn NativeAuthenticationProvider>>,
    ) -> Result<NativeTextTarget, Error> {
        let retained = self.captured_source().await?;
        let source = match retained {
            Some(source) => source,
            None => {
                if !matches!(
                    operation.intent,
                    NativeSetupIntent::Initial { allow_fresh: true }
                ) {
                    return Err(Error::CaptureUnavailable);
                }
                let storage = self.storage.clone();
                let source =
                    tokio::task::spawn_blocking(move || crate::CaptureLog::create(&storage))
                        .await
                        .map_err(|_| Error::Process)??;
                self.retained_capture = Some(source.clone());
                source
            }
        };
        let snapshot = source.snapshot()?;
        let native = source.native_session()?;
        let context = match (&operation.intent, native) {
            (NativeSetupIntent::Existing(expected), Some(session))
                if expected.snapshot == snapshot && expected.native_session == session =>
            {
                NativeContext::Resume {
                    session,
                    replay: CaptureReplay::AlreadyCaptured,
                }
            }
            (NativeSetupIntent::Initial { .. }, Some(session)) => NativeContext::Resume {
                session,
                replay: CaptureReplay::AlreadyCaptured,
            },
            (NativeSetupIntent::Initial { allow_fresh: true }, None) if source.is_empty()? => {
                NativeContext::New
            }
            _ => return Err(Error::CaptureUnavailable),
        };
        let native_session = self
            .start(operation.supply.clone(), context, source, authentication)
            .await?;
        Ok(NativeTextTarget {
            snapshot,
            native_session,
        })
    }
}
impl NativeHandle {
    /// Queue admission returns before preparation; later lookup reads actor-owned facts.
    pub fn admit_setup(
        &self,
        operation: NativeSetupOperation,
    ) -> Result<tokio::sync::oneshot::Receiver<Result<NativeSetupObservation, Error>>, Error> {
        self.admit_setup_with_authentication(operation, None)
    }
    pub fn admit_setup_with_authentication(
        &self,
        operation: NativeSetupOperation,
        authentication: Option<Arc<dyn NativeAuthenticationProvider>>,
    ) -> Result<tokio::sync::oneshot::Receiver<Result<NativeSetupObservation, Error>>, Error> {
        valid_id(&operation.operation_id)?;
        let (result, rx) = oneshot::channel();
        self.sender
            .try_send(Command::Setup(operation, authentication, result))
            .map_err(|_| Error::NotAdmitted)?;
        Ok(rx)
    }
    pub async fn setup_observation(&self, id: String) -> Result<NativeSetupObservation, Error> {
        valid_id(&id)?;
        let (result, rx) = oneshot::channel();
        self.sender
            .try_send(Command::SetupObservation(id, result))
            .map_err(|_| Error::NotAdmitted)?;
        rx.await.map_err(|_| Error::Process)?
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_the_exact_known_setup_result_can_release_retained_capacity() -> Result<(), Error> {
        let mut registry = SetupRegistry::default();
        let operation = NativeSetupOperation {
            operation_id: "operation".into(),
            supply: HarnessSupply::Managed,
            intent: NativeSetupIntent::Initial { allow_fresh: true },
        };
        registry.begin(&operation)?;
        let rejected = NativeSetupObservation::Failed(NativeSetupFailure::NotAdmitted);
        assert!(registry.release_reported("operation", &rejected).is_err());
        registry.completed("operation", rejected.clone())?;
        assert!(
            registry
                .release_reported(
                    "operation",
                    &NativeSetupObservation::Failed(NativeSetupFailure::NativeRejected)
                )
                .is_err()
        );
        registry.release_reported("operation", &rejected)?;
        registry.release_reported("operation", &rejected)?;
        assert!(matches!(
            registry.observe("operation")?,
            NativeSetupObservation::Unseen
        ));
        Ok(())
    }
    type TestError = Box<dyn std::error::Error>;
    fn operation(id: &str) -> NativeSetupOperation {
        NativeSetupOperation {
            operation_id: id.into(),
            supply: HarnessSupply::Managed,
            intent: NativeSetupIntent::Initial { allow_fresh: true },
        }
    }
    #[test]
    fn duplicate_setup_recovers_original_evidence_and_changed_intent_conflicts()
    -> Result<(), TestError> {
        let mut registry = SetupRegistry::default();
        let first = operation("operation");
        assert!(registry.begin(&first)?.is_none());
        assert!(matches!(
            registry.begin(&first)?,
            Some(NativeSetupObservation::PossibleForwarding)
        ));
        let changed = NativeSetupOperation {
            intent: NativeSetupIntent::Initial { allow_fresh: false },
            ..first.clone()
        };
        assert!(matches!(
            registry.begin(&changed),
            Err(Error::OperationConflict)
        ));
        registry.completed(
            &first.operation_id,
            NativeSetupObservation::Failed(NativeSetupFailure::SourceUnavailable),
        )?;
        assert!(matches!(
            registry.begin(&first)?,
            Some(NativeSetupObservation::Failed(
                NativeSetupFailure::SourceUnavailable
            ))
        ));
        assert!(matches!(
            registry.observe("missing")?,
            NativeSetupObservation::Unseen
        ));
        Ok(())
    }
    #[tokio::test]
    async fn missing_resume_source_does_not_create_a_fresh_context() -> Result<(), TestError> {
        let saved = tempfile::tempdir()?;
        let storage = NativeStorage::open(saved.path())?;
        let owner = NativeOwner::new(storage.clone())?;
        let handle = owner.handle();
        let operation = NativeSetupOperation {
            intent: NativeSetupIntent::Initial { allow_fresh: false },
            ..operation("resume-only")
        };
        let rx = handle.admit_setup(operation.clone())?;
        drop(rx);
        assert!(matches!(
            handle
                .setup_observation(operation.operation_id.clone())
                .await?,
            NativeSetupObservation::Failed(NativeSetupFailure::SourceUnavailable)
        ));
        assert!(!storage.capture_root().join("current").exists());
        assert!(matches!(
            handle.admit_setup(operation)?.await??,
            NativeSetupObservation::Failed(NativeSetupFailure::SourceUnavailable)
        ));
        assert_eq!(owner.finish().await?, Retirement::NoNativeStarted);
        Ok(())
    }
}
