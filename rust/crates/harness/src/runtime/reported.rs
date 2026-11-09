//! Known-result release requires the caller's durable report and exclusive dispatch lease.
use super::*;
pub enum NativeReportedOperation {
    Setup {
        id: String,
        result: NativeSetupObservation,
    },
    Input {
        id: String,
        result: NativePromptAcceptance,
    },
    Stop {
        id: String,
        result: NativeStopObservation,
    },
    Answer {
        id: String,
        result: NativeAnswerObservation,
    },
}
impl fmt::Debug for NativeReportedOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("NativeReportedOperation([REDACTED])")
    }
}
impl State {
    pub(super) fn release_reported(
        &mut self,
        reported: NativeReportedOperation,
    ) -> Result<(), Error> {
        match reported {
            NativeReportedOperation::Setup { id, result } => {
                self.setups.release_reported(&id, &result)
            }
            NativeReportedOperation::Input { id, result } => {
                self.inputs.release_reported(&id, &result)
            }
            NativeReportedOperation::Stop { id, result } => {
                self.stops.release_reported(&id, result)
            }
            NativeReportedOperation::Answer { id, result } => {
                self.answers.release_reported(&id, result)
            }
        }
    }
}
impl NativeHandle {
    /// The Connector must retain its exclusive operation lease through this
    /// acknowledgement. It calls only after successful durable publication.
    /// A rejected report is not an acknowledgement. This releases no uncertain
    /// fact and never grants native dispatch permission.
    pub fn admit_reported(
        &self,
        reported: NativeReportedOperation,
    ) -> Result<oneshot::Receiver<Result<(), Error>>, Error> {
        let (result, rx) = oneshot::channel();
        self.sender
            .try_send(Command::Reported(reported, result))
            .map_err(|_| Error::NotAdmitted)?;
        Ok(rx)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn reported_known_answers_reuse_capacity_beyond_the_retention_bound()
    -> Result<(), Box<dyn std::error::Error>> {
        let temporary = tempfile::tempdir()?;
        let owner = NativeOwner::new(NativeStorage::open(temporary.path())?)?;
        let handle = owner.handle();
        for index in 0..300 {
            let id = format!("answer-{index}");
            let operation = NativeAnswerOperation::permission(
                id.clone(),
                NativeTextTarget {
                    snapshot: chapchoo_domain::SnapshotId::parse("source".into())?,
                    native_session: NativeConversationId::parse("native".into())?,
                },
                chapchoo_domain::SourcePosition::new(
                    1,
                    chapchoo_domain::EventOccurrenceId::parse("request".into())?,
                )?,
                "responder".into(),
                &serde_json::to_vec(&schema::RequestPermissionResponse::new(
                    schema::RequestPermissionOutcome::Cancelled(
                        schema::CancelledPermissionOutcome::new(),
                    ),
                ))?,
            )?;
            assert_eq!(
                handle.admit_answer(operation)?.await??,
                NativeAnswerObservation::Rejected
            );
            handle
                .admit_reported(NativeReportedOperation::Answer {
                    id: id.clone(),
                    result: NativeAnswerObservation::Rejected,
                })?
                .await??;
            handle
                .admit_reported(NativeReportedOperation::Answer {
                    id,
                    result: NativeAnswerObservation::Rejected,
                })?
                .await??;
        }
        assert!(handle.capture_reader().current_source().await?.is_none());
        assert_eq!(owner.finish().await?, Retirement::NoNativeStarted);
        Ok(())
    }
}
