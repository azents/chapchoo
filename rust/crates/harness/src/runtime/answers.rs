//! Exact single-use responses retain uncertainty independently of server links.
use super::*;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeAnswerObservation {
    PossibleForwarding,
    ResponderAccepted,
    Rejected,
}
pub struct NativeAnswerOperation {
    pub operation_id: String,
    pub target: NativeTextTarget,
    pub occurrence: chapchoo_domain::SourcePosition,
    pub response: InteractiveResponse,
}
impl fmt::Debug for NativeAnswerOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("NativeAnswerOperation([REDACTED])")
    }
}
impl NativeAnswerOperation {
    pub fn permission(
        operation_id: String,
        target: NativeTextTarget,
        occurrence: chapchoo_domain::SourcePosition,
        responder_token: String,
        body: &[u8],
    ) -> Result<Self, Error> {
        if body.len() > 16 * 1024 * 1024 {
            return Err(Error::NotAdmitted);
        }
        Ok(Self {
            operation_id,
            target,
            occurrence,
            response: InteractiveResponse::Permission {
                id: responder_token,
                response: serde_json::from_slice(body).map_err(|_| Error::NotAdmitted)?,
            },
        })
    }
    pub fn form(
        operation_id: String,
        target: NativeTextTarget,
        occurrence: chapchoo_domain::SourcePosition,
        responder_token: String,
        body: &[u8],
    ) -> Result<Self, Error> {
        if body.len() > 16 * 1024 * 1024 {
            return Err(Error::NotAdmitted);
        }
        Ok(Self {
            operation_id,
            target,
            occurrence,
            response: InteractiveResponse::Form {
                id: responder_token,
                response: serde_json::from_slice(body).map_err(|_| Error::NotAdmitted)?,
            },
        })
    }
    fn fingerprint(&self) -> Result<([u8; 32], u32), Error> {
        if self.operation_id.is_empty()
            || self.operation_id.len() > 128
            || !self
                .operation_id
                .bytes()
                .all(|byte| byte.is_ascii_graphic())
        {
            return Err(Error::NotAdmitted);
        }
        let (token, kind, bytes) = match &self.response {
            InteractiveResponse::Permission { id, response } => (
                id,
                "permission",
                serde_json::to_vec(response).map_err(|_| Error::NotAdmitted)?,
            ),
            InteractiveResponse::Form { id, response } => (
                id,
                "form",
                serde_json::to_vec(response).map_err(|_| Error::NotAdmitted)?,
            ),
        };
        let length = u32::try_from(bytes.len()).map_err(|_| Error::NotAdmitted)?;
        if length == 0
            || length > 16 * 1024 * 1024
            || token.is_empty()
            || token.len() > 128
            || !token.bytes().all(|byte| byte.is_ascii_graphic())
        {
            return Err(Error::NotAdmitted);
        }
        let mut digest = Sha256::new();
        for part in [
            self.target.snapshot.as_str().as_bytes(),
            self.target.native_session.as_str().as_bytes(),
            self.occurrence.occurrence().as_str().as_bytes(),
            &self.occurrence.ordinal().to_be_bytes(),
            token.as_bytes(),
            kind.as_bytes(),
            &bytes,
        ] {
            digest.update(
                u64::try_from(part.len())
                    .map_err(|_| Error::NotAdmitted)?
                    .to_be_bytes(),
            );
            digest.update(part);
        }
        Ok((digest.finalize().into(), length))
    }
}
#[derive(Default)]
pub(super) struct AnswerRegistry {
    entries: BTreeMap<String, ([u8; 32], NativeAnswerObservation)>,
}
impl AnswerRegistry {
    pub(super) fn release_reported(
        &mut self,
        id: &str,
        expected: NativeAnswerObservation,
    ) -> Result<(), Error> {
        if expected == NativeAnswerObservation::PossibleForwarding
            || self
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
        fingerprint: [u8; 32],
    ) -> Result<Option<NativeAnswerObservation>, Error> {
        if let Some((previous, observation)) = self.entries.get(id) {
            if *previous != fingerprint {
                return Err(Error::OperationConflict);
            }
            return Ok(Some(*observation));
        }
        if self.entries.len() >= 256 {
            return Err(Error::NotAdmitted);
        }
        self.entries.insert(
            id.into(),
            (fingerprint, NativeAnswerObservation::PossibleForwarding),
        );
        Ok(None)
    }
}
impl State {
    pub(super) async fn answer(
        &mut self,
        operation: NativeAnswerOperation,
    ) -> Result<NativeAnswerObservation, Error> {
        let (fingerprint, _) = operation.fingerprint()?;
        if let Some(previous) = self.answers.begin(&operation.operation_id, fingerprint)? {
            return Ok(previous);
        }
        let result = self.exact_answer(&operation).await;
        let observed = match result {
            Ok(()) => NativeAnswerObservation::ResponderAccepted,
            Err(Error::NotAdmitted) => NativeAnswerObservation::Rejected,
            // Sending through the SDK may have started; generic errors are uncertain.
            Err(error) => return Err(error),
        };
        self.answers
            .entries
            .get_mut(&operation.operation_id)
            .ok_or(Error::OperationConflict)?
            .1 = observed;
        Ok(observed)
    }
    async fn exact_answer(&self, operation: &NativeAnswerOperation) -> Result<(), Error> {
        let live = self.live.as_ref().ok_or(Error::NotAdmitted)?;
        let session = live.session.as_ref().ok_or(Error::NotAdmitted)?;
        if self.blocked
            || live.capture.snapshot()? != operation.target.snapshot
            || session.session_id().to_string() != operation.target.native_session.as_str()
        {
            return Err(Error::NotAdmitted);
        }
        let token = match &operation.response {
            InteractiveResponse::Permission { id, .. } | InteractiveResponse::Form { id, .. } => id,
        };
        // The actor cannot replace its native owner across this check and respond.
        // Incoming cancellation may close the responder, which respond rechecks.
        {
            let pending = live.requests.lock().await;
            let (occurrence, _) = pending.get(token).ok_or(Error::NotAdmitted)?;
            if occurrence != &operation.occurrence {
                return Err(Error::NotAdmitted);
            }
        }
        self.respond(operation.response.clone()).await
    }
}
impl NativeHandle {
    pub fn admit_answer(
        &self,
        operation: NativeAnswerOperation,
    ) -> Result<oneshot::Receiver<Result<NativeAnswerObservation, Error>>, Error> {
        let (_, length) = operation.fingerprint()?;
        let permit = self
            .payload_budget
            .clone()
            .try_acquire_many_owned(length)
            .map_err(|_| Error::NotAdmitted)?;
        let (result, rx) = oneshot::channel();
        self.sender
            .try_send(Command::Answer(operation, permit, result))
            .map_err(|_| Error::NotAdmitted)?;
        Ok(rx)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn response_fingerprint_binds_source_token_kind_and_original_values()
    -> Result<(), Box<dyn std::error::Error>> {
        let target = NativeTextTarget {
            snapshot: chapchoo_domain::SnapshotId::parse("snapshot".into())?,
            native_session: NativeConversationId::parse("native".into())?,
        };
        let occurrence = chapchoo_domain::SourcePosition::new(
            1,
            chapchoo_domain::EventOccurrenceId::parse("request".into())?,
        )?;
        let make = |token: &str, body: &[u8]| {
            NativeAnswerOperation::form(
                "operation".into(),
                target.clone(),
                occurrence.clone(),
                token.into(),
                body,
            )
        };
        let original = make(
            "token",
            br#"{"action":"accept","content":{"text":"","enabled":false}}"#,
        )?;
        let reordered = make(
            "token",
            br#"{"content":{"enabled":false,"text":""},"action":"accept"}"#,
        )?;
        assert_eq!(original.fingerprint()?, reordered.fingerprint()?);
        for changed in [
            make(
                "other-token",
                br#"{"action":"accept","content":{"text":"","enabled":false}}"#,
            )?,
            make(
                "token",
                br#"{"action":"accept","content":{"text":"","enabled":true}}"#,
            )?,
            make("token", br#"{"action":"decline"}"#)?,
            NativeAnswerOperation {
                target: NativeTextTarget {
                    snapshot: chapchoo_domain::SnapshotId::parse("replacement".into())?,
                    ..target
                },
                ..reordered
            },
        ] {
            assert_ne!(original.fingerprint()?.0, changed.fingerprint()?.0);
        }
        Ok(())
    }
    #[tokio::test]
    async fn missing_native_owner_rejects_duplicates_without_starting_native()
    -> Result<(), Box<dyn std::error::Error>> {
        let temporary = tempfile::tempdir()?;
        let storage = NativeStorage::open(temporary.path())?;
        let owner = NativeOwner::new(storage)?;
        let make = || -> Result<NativeAnswerOperation, Error> {
            NativeAnswerOperation::permission(
                "answer".into(),
                NativeTextTarget {
                    snapshot: chapchoo_domain::SnapshotId::parse("snapshot".into())
                        .map_err(|_| Error::NotAdmitted)?,
                    native_session: NativeConversationId::parse("native".into())
                        .map_err(|_| Error::NotAdmitted)?,
                },
                chapchoo_domain::SourcePosition::new(
                    1,
                    chapchoo_domain::EventOccurrenceId::parse("occurrence".into())
                        .map_err(|_| Error::NotAdmitted)?,
                )
                .map_err(|_| Error::NotAdmitted)?,
                "token".into(),
                &serde_json::to_vec(&schema::RequestPermissionResponse::new(
                    schema::RequestPermissionOutcome::Cancelled(
                        schema::CancelledPermissionOutcome::new(),
                    ),
                ))
                .map_err(|_| Error::NotAdmitted)?,
            )
        };
        for _ in 0..2 {
            assert_eq!(
                owner.handle().admit_answer(make()?)?.await??,
                NativeAnswerObservation::Rejected
            );
        }
        assert!(owner.capture_reader().current_source().await?.is_none());
        assert_eq!(owner.finish().await?, Retirement::NoNativeStarted);
        Ok(())
    }
    #[test]
    fn retained_response_cannot_be_changed_or_replayed_from_uncertainty() -> Result<(), Error> {
        let mut registry = AnswerRegistry::default();
        assert_eq!(registry.begin("one", [1; 32])?, None);
        assert!(
            registry
                .release_reported("one", NativeAnswerObservation::Rejected)
                .is_err()
        );
        assert_eq!(
            registry.begin("one", [1; 32])?,
            Some(NativeAnswerObservation::PossibleForwarding)
        );
        assert!(matches!(
            registry.begin("one", [2; 32]),
            Err(Error::OperationConflict)
        ));
        registry
            .entries
            .get_mut("one")
            .ok_or(Error::OperationConflict)?
            .1 = NativeAnswerObservation::ResponderAccepted;
        assert_eq!(
            registry.begin("one", [1; 32])?,
            Some(NativeAnswerObservation::ResponderAccepted)
        );
        assert!(
            registry
                .release_reported("one", NativeAnswerObservation::Rejected)
                .is_err()
        );
        for index in 1..256 {
            registry.begin(&format!("operation-{index}"), [0; 32])?;
        }
        assert!(matches!(
            registry.begin("overflow", [0; 32]),
            Err(Error::NotAdmitted)
        ));
        assert_eq!(
            registry.begin("one", [1; 32])?,
            Some(NativeAnswerObservation::ResponderAccepted)
        );
        registry.release_reported("one", NativeAnswerObservation::ResponderAccepted)?;
        assert!(registry.begin("new-operation", [0; 32])?.is_none());
        Ok(())
    }
}
