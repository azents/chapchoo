use super::*;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeStopObservation {
    Unseen,
    PossibleForwarding,
    CancelPublished,
    Rejected,
}
#[derive(Default)]
pub(super) struct StopRegistry {
    entries: BTreeMap<String, (NativeWorkTarget, NativeStopObservation)>,
}
impl StopRegistry {
    pub(super) fn release_reported(
        &mut self,
        id: &str,
        expected: NativeStopObservation,
    ) -> Result<(), Error> {
        if !matches!(
            expected,
            NativeStopObservation::CancelPublished | NativeStopObservation::Rejected
        ) || self
            .entries
            .get(id)
            .is_some_and(|(_, outcome)| *outcome != expected)
        {
            return Err(Error::OperationConflict);
        }
        self.entries.remove(id);
        Ok(())
    }
    fn begin(
        &mut self,
        id: &str,
        target: &NativeWorkTarget,
    ) -> Result<Option<NativeStopObservation>, Error> {
        if id.is_empty() || id.len() > 128 || !id.bytes().all(|byte| byte.is_ascii_graphic()) {
            return Err(Error::NotAdmitted);
        }
        if let Some((previous, result)) = self.entries.get(id) {
            if previous != target {
                return Err(Error::OperationConflict);
            }
            return Ok(Some(*result));
        }
        if self.entries.len() >= 256 {
            return Err(Error::NotAdmitted);
        }
        self.entries.insert(
            id.into(),
            (target.clone(), NativeStopObservation::PossibleForwarding),
        );
        Ok(None)
    }
}
impl State {
    pub(super) fn stop_operation(
        &mut self,
        id: &str,
        target: &NativeWorkTarget,
    ) -> Result<NativeStopObservation, Error> {
        if let Some(previous) = self.stops.begin(id, target)? {
            return Ok(previous);
        }
        let result = match self.stop(target) {
            Ok(()) => NativeStopObservation::CancelPublished,
            Err(Error::NotAdmitted) => NativeStopObservation::Rejected,
            Err(error) => return Err(error),
        };
        self.stops
            .entries
            .get_mut(id)
            .ok_or(Error::OperationConflict)?
            .1 = result;
        Ok(result)
    }
}
impl NativeHandle {
    pub fn admit_stop(
        &self,
        id: String,
        target: NativeWorkTarget,
    ) -> Result<oneshot::Receiver<Result<NativeStopObservation, Error>>, Error> {
        if id.is_empty() || id.len() > 128 || !id.bytes().all(|byte| byte.is_ascii_graphic()) {
            return Err(Error::NotAdmitted);
        }
        let (result, rx) = oneshot::channel();
        self.sender
            .try_send(Command::StopOperation(id, target, result))
            .map_err(|_| Error::NotAdmitted)?;
        Ok(rx)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stop_operation_identity_never_retargets_an_uncertain_cancel()
    -> Result<(), Box<dyn std::error::Error>> {
        let target = NativeWorkTarget {
            snapshot: chapchoo_domain::SnapshotId::parse("snapshot".into())?,
            native_session: NativeConversationId::parse("native".into())?,
            work_id: "work".into(),
        };
        let mut registry = StopRegistry::default();
        assert!(registry.begin("operation", &target)?.is_none());
        assert!(
            registry
                .release_reported("operation", NativeStopObservation::Rejected)
                .is_err()
        );
        assert_eq!(
            registry.begin("operation", &target)?,
            Some(NativeStopObservation::PossibleForwarding)
        );
        let later = NativeWorkTarget {
            work_id: "later-work".into(),
            ..target
        };
        assert!(matches!(
            registry.begin("operation", &later),
            Err(Error::OperationConflict)
        ));
        registry
            .entries
            .get_mut("operation")
            .ok_or(Error::OperationConflict)?
            .1 = NativeStopObservation::Rejected;
        assert!(
            registry
                .release_reported("operation", NativeStopObservation::CancelPublished)
                .is_err()
        );
        registry.release_reported("operation", NativeStopObservation::Rejected)?;
        assert!(registry.begin("new-operation", &later)?.is_none());
        Ok(())
    }
}
