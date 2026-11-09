use super::*;
use chapchoo_harness::{
    NativeAnswerObservation, NativePromptAcceptance, NativeReportedOperation, NativeStopObservation,
};
impl Publication {
    pub(super) fn record(&mut self, pending: Pending, result: Result<(), tonic::Status>) {
        match result {
            Ok(()) => self.settled.push(pending),
            // A refused report is not a durable acknowledgement. Drop only
            // the report owner; native facts still fence later stale dispatch.
            Err(error)
                if matches!(
                    error.code(),
                    tonic::Code::NotFound
                        | tonic::Code::AlreadyExists
                        | tonic::Code::Aborted
                        | tonic::Code::FailedPrecondition
                ) => {}
            Err(_) => self.failed.push(pending),
        }
    }
}
impl Report {
    pub(super) fn native_report(&self) -> Result<NativeReportedOperation, AppError> {
        Ok(match self {
            Self::Input(report) => NativeReportedOperation::Input {
                id: report
                    .command
                    .as_ref()
                    .ok_or("Input report command is absent")?
                    .operation_id
                    .clone(),
                result: match report
                    .result
                    .as_ref()
                    .ok_or("Input report result is absent")?
                {
                    c::input_report::Result::InsertedNativeMessageId(id) => {
                        NativePromptAcceptance::Inserted {
                            native_message_id: id.clone(),
                        }
                    }
                    c::input_report::Result::Steered(()) => NativePromptAcceptance::Steered,
                    c::input_report::Result::Failed(()) => NativePromptAcceptance::Failed,
                },
            },
            Self::Stop(report) => NativeReportedOperation::Stop {
                id: report
                    .command
                    .as_ref()
                    .ok_or("Stop report command is absent")?
                    .operation_id
                    .clone(),
                result: if report.cancel_published {
                    NativeStopObservation::CancelPublished
                } else {
                    NativeStopObservation::Rejected
                },
            },
            Self::Answer(report) => NativeReportedOperation::Answer {
                id: report
                    .command
                    .as_ref()
                    .ok_or("Answer report command is absent")?
                    .operation_id
                    .clone(),
                result: if report.responder_accepted {
                    NativeAnswerObservation::ResponderAccepted
                } else {
                    NativeAnswerObservation::Rejected
                },
            },
            Self::Setup(report) => NativeReportedOperation::Setup {
                id: report.operation_id.clone(),
                result: match report
                    .result
                    .as_ref()
                    .ok_or("Setup report result is absent")?
                {
                    c::setup_report::Result::Ready(target) => {
                        NativeSetupObservation::Ready(NativeTextTarget {
                            snapshot: SnapshotId::parse(target.snapshot_id.clone())?,
                            native_session: NativeConversationId::parse(
                                target.native_session_id.clone(),
                            )?,
                        })
                    }
                    c::setup_report::Result::Failed(failure) => {
                        NativeSetupObservation::Failed(match c::SetupFailure::try_from(*failure)? {
                            c::SetupFailure::NotAdmitted => NativeSetupFailure::NotAdmitted,
                            c::SetupFailure::BeforeNativeLaunch => {
                                NativeSetupFailure::BeforeNativeLaunch
                            }
                            c::SetupFailure::NativeRejected => NativeSetupFailure::NativeRejected,
                            c::SetupFailure::SourceUnavailable => {
                                NativeSetupFailure::SourceUnavailable
                            }
                            c::SetupFailure::RetirementUnknown => {
                                NativeSetupFailure::RetirementUnknown
                            }
                            c::SetupFailure::Unspecified => {
                                return Err("Known setup report result is absent".into());
                            }
                        })
                    }
                },
            },
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn refused_publication_never_releases_known_native_facts() -> Result<(), AppError> {
        let temporary = tempfile::tempdir()?;
        let native = chapchoo_harness::NativeOwner::new(chapchoo_harness::NativeStorage::open(
            temporary.path(),
        )?)?;
        let mut operations = Operations::new(native.handle());
        assert!(operations.stop(c::StopCommand {
            conversation_id: "conversation".into(),
            operation_id: "operation".into(),
            target: Some(c::WorkTarget {
                source: Some(c::NativeTarget {
                    snapshot_id: "snapshot".into(),
                    native_session_id: "native".into(),
                }),
                work_id: "work".into()
            }),
        })?);
        let pending = operations
            .tasks
            .join_next()
            .await
            .ok_or("Missing native task")??
            .ok_or("Missing known native result")?;
        let mut publication = Publication::default();
        publication.record(
            pending,
            Err(tonic::Status::failed_precondition(
                "Fixture route withdrawal",
            )),
        );
        assert!(publication.settled.is_empty());
        assert!(publication.failed.is_empty());
        let changed = chapchoo_harness::NativeWorkTarget {
            snapshot: SnapshotId::parse("snapshot".into())?,
            native_session: NativeConversationId::parse("native".into())?,
            work_id: "later-work".into(),
        };
        assert!(matches!(
            native
                .handle()
                .admit_stop("operation".into(), changed)?
                .await?,
            Err(chapchoo_harness::Error::OperationConflict)
        ));
        operations.finish().await?;
        native.finish().await?;
        Ok(())
    }
}
