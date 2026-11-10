use super::*;
use chapchoo_domain::SnapshotId;
use sha2::{Digest, Sha256};

#[derive(Clone, PartialEq, Eq)]
pub struct NativeTextTarget {
    pub snapshot: SnapshotId,
    pub native_session: NativeConversationId,
}
#[derive(Clone)]
pub struct NativeInputOperation {
    pub operation_id: String,
    pub target: NativeTextTarget,
    pub text: ConversationText,
}
#[derive(Debug, Clone)]
pub enum NativeInputObservation {
    /// Missing execution-local facts are not evidence of non-delivery.
    Unseen,
    PossibleForwarding,
    Known(NativePromptAcceptance),
}
impl fmt::Debug for NativeTextTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("NativeTextTarget([REDACTED])")
    }
}
impl fmt::Debug for NativeInputOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("NativeInputOperation([REDACTED])")
    }
}
impl NativeInputOperation {
    pub(super) fn validate(&self) -> Result<(), Error> {
        if self.operation_id.is_empty()
            || self.operation_id.len() > 128
            || !self.operation_id.bytes().all(|b| b.is_ascii_graphic())
            || self.text.as_str().len() > MAX_LINE
        {
            return Err(Error::NotAdmitted);
        }
        Ok(())
    }
}
#[derive(Default)]
pub(super) struct InputRegistry {
    entries: BTreeMap<String, Entry>,
}
struct Entry {
    target: NativeTextTarget,
    fingerprint: [u8; 32],
    outcome: Outcome,
}
enum Outcome {
    PossibleForwarding,
    Known(NativePromptAcceptance),
}
impl InputRegistry {
    pub(super) fn release_reported(
        &mut self,
        id: &str,
        expected: &NativePromptAcceptance,
    ) -> Result<(), Error> {
        if self.entries.get(id).is_some_and(
            |entry| !matches!(&entry.outcome, Outcome::Known(result) if result == expected),
        ) {
            return Err(Error::OperationConflict);
        }
        self.entries.remove(id);
        Ok(())
    }
    pub(super) fn observe(
        &self,
        id: &str,
        target: &NativeTextTarget,
    ) -> Result<NativeInputObservation, Error> {
        let Some(entry) = self.entries.get(id) else {
            return Ok(NativeInputObservation::Unseen);
        };
        if &entry.target != target {
            return Err(Error::OperationConflict);
        }
        Ok(match &entry.outcome {
            Outcome::PossibleForwarding => NativeInputObservation::PossibleForwarding,
            Outcome::Known(result) => NativeInputObservation::Known(result.clone()),
        })
    }
    pub(super) fn recover(
        &self,
        operation: &NativeInputOperation,
    ) -> Result<Option<NativePromptAcceptance>, Error> {
        let Some(existing) = self.entries.get(&operation.operation_id) else {
            return Ok(None);
        };
        let fingerprint: [u8; 32] = Sha256::digest(operation.text.as_str().as_bytes()).into();
        if existing.target != operation.target || existing.fingerprint != fingerprint {
            return Err(Error::OperationConflict);
        }
        match &existing.outcome {
            Outcome::PossibleForwarding => Err(Error::DeliveryUncertain),
            Outcome::Known(result) => Ok(Some(result.clone())),
        }
    }
    /// The owning actor retains this fact before publishing any native request.
    /// A cancelled/failed SDK waiter never reconstructs fresh dispatch permission.
    pub(super) fn begin(
        &mut self,
        operation: &NativeInputOperation,
    ) -> Result<Option<NativePromptAcceptance>, Error> {
        operation.validate()?;
        if let Some(result) = self.recover(operation)? {
            return Ok(Some(result));
        }
        let fingerprint = Sha256::digest(operation.text.as_str().as_bytes()).into();
        if self.entries.len() >= 256 {
            return Err(Error::NotAdmitted);
        }
        self.entries.insert(
            operation.operation_id.clone(),
            Entry {
                target: operation.target.clone(),
                fingerprint,
                outcome: Outcome::PossibleForwarding,
            },
        );
        Ok(None)
    }
    pub(super) fn completed(
        &mut self,
        id: &str,
        outcome: NativePromptAcceptance,
    ) -> Result<(), Error> {
        let entry = self.entries.get_mut(id).ok_or(Error::OperationConflict)?;
        if !matches!(entry.outcome, Outcome::PossibleForwarding) {
            return Err(Error::OperationConflict);
        }
        entry.outcome = Outcome::Known(outcome);
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    type TestError = Box<dyn std::error::Error>;
    fn operation(id: &str) -> Result<NativeInputOperation, TestError> {
        Ok(NativeInputOperation {
            operation_id: id.into(),
            target: NativeTextTarget {
                snapshot: SnapshotId::parse("source".into())?,
                native_session: NativeConversationId::parse("native".into())?,
            },
            text: ConversationText::parse("exact \ntext".into())?,
        })
    }
    #[test]
    fn possible_forwarding_survives_waiter_loss_without_fresh_dispatch_permission()
    -> Result<(), TestError> {
        let mut registry = InputRegistry::default();
        let input = operation("operation")?;
        assert!(registry.begin(&input)?.is_none());
        assert!(matches!(
            registry.begin(&input),
            Err(Error::DeliveryUncertain)
        ));
        let mut changed = input.clone();
        changed.text = ConversationText::parse("changed".into())?;
        assert!(matches!(
            registry.begin(&changed),
            Err(Error::OperationConflict)
        ));
        let mut retargeted = input.clone();
        retargeted.target.snapshot = SnapshotId::parse("another".into())?;
        assert!(matches!(
            registry.begin(&retargeted),
            Err(Error::OperationConflict)
        ));
        assert!(matches!(
            registry.observe(&input.operation_id, &input.target)?,
            NativeInputObservation::PossibleForwarding
        ));
        registry.completed(&input.operation_id, NativePromptAcceptance::Steered)?;
        assert!(matches!(
            registry.observe(&input.operation_id, &input.target)?,
            NativeInputObservation::Known(NativePromptAcceptance::Steered)
        ));
        assert!(matches!(
            registry.observe("missing", &input.target)?,
            NativeInputObservation::Unseen
        ));
        assert!(matches!(
            registry.begin(&input)?,
            Some(NativePromptAcceptance::Steered)
        ));
        assert!(
            registry
                .completed(&input.operation_id, NativePromptAcceptance::Failed)
                .is_err()
        );
        assert!(
            registry
                .begin(&operation("identical-but-independent")?)?
                .is_none()
        );
        Ok(())
    }
    #[test]
    fn capacity_never_evicts_admitted_work_or_uncertainty() -> Result<(), TestError> {
        let mut registry = InputRegistry::default();
        let uncertain = operation("uncertain")?;
        registry.begin(&uncertain)?;
        assert!(
            registry
                .release_reported("uncertain", &NativePromptAcceptance::Failed)
                .is_err()
        );
        assert!(matches!(
            registry.observe("uncertain", &uncertain.target)?,
            NativeInputObservation::PossibleForwarding
        ));
        registry.completed("uncertain", NativePromptAcceptance::Failed)?;
        assert!(
            registry
                .release_reported("uncertain", &NativePromptAcceptance::Steered)
                .is_err()
        );
        registry.release_reported("uncertain", &NativePromptAcceptance::Failed)?;
        registry.release_reported("uncertain", &NativePromptAcceptance::Failed)?;
        for index in 0..256 {
            assert!(
                registry
                    .begin(&operation(&format!("operation-{index}"))?)?
                    .is_none()
            );
        }
        assert!(matches!(
            registry.begin(&operation("overflow")?),
            Err(Error::NotAdmitted)
        ));
        assert!(matches!(
            registry.begin(&operation("operation-0")?),
            Err(Error::DeliveryUncertain)
        ));
        registry.completed(
            "operation-0",
            NativePromptAcceptance::Inserted {
                native_message_id: "message".into(),
            },
        )?;
        assert!(
            matches!(registry.begin(&operation("operation-0")?)?,Some(NativePromptAcceptance::Inserted { native_message_id }) if native_message_id=="message")
        );
        registry.release_reported(
            "operation-0",
            &NativePromptAcceptance::Inserted {
                native_message_id: "message".into(),
            },
        )?;
        assert!(
            registry
                .begin(&operation("after-reported-release")?)?
                .is_none()
        );
        assert!(matches!(
            registry.begin(&operation("overflow")?),
            Err(Error::NotAdmitted)
        ));
        Ok(())
    }
}
