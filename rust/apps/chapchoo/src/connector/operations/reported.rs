use super::*;
use chapchoo_harness::{
    NativeAnswerObservation, NativePromptAcceptance, NativeReportedOperation, NativeStopObservation,
};
impl Publication {
    pub(super) fn record(&mut self, mut pending: Pending, result: Result<bool, tonic::Status>) {
        match result {
            Ok(recorded) => {
                if recorded
                    && matches!(&pending.0, Report::Input(report) if matches!(
                        report.result, Some(c::input_report::Result::InsertedNativeMessageId(_) | c::input_report::Result::Steered(()))
                    ))
                    && let Some(ticket) = pending.1.ordering.as_mut()
                {
                    ticket.recorded();
                }
                self.settled.push(pending);
            }
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
                    c::input_report::Result::AttemptError(()) => {
                        return Err("Queue attempt error has no known native result".into());
                    }
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
    async fn input_report_retry_holds_invocation_until_ordered_closure() -> Result<(), AppError> {
        let temporary = tempfile::tempdir()?;
        let native = chapchoo_harness::NativeOwner::new(chapchoo_harness::NativeStorage::open(
            temporary.path(),
        )?)?;
        let mut operations = Operations::new(native.handle());
        let id = "consumed-input";
        let mut permit = operations
            .admit(leases::Kind::Input, id)?
            .ok_or("Missing permit")?;
        let binding = chapchoo_harness::NativeAuthenticationBinding {
            agent_id: "agent".into(),
            conversation_id: "conversation".into(),
            operation_id: "setup".into(),
            execution_id: "execution".into(),
            authentication_id: "authentication".into(),
            source: [1; 24],
        };
        let owner = operations.ordering.open(binding.clone());
        permit.ordering = Some(operations.ordering.begin(&binding, id)?);
        let mut waiting = std::pin::pin!(owner.snapshot()?.ready());
        let report = Report::Input(c::InputReport {
            source: None,
            command: Some(c::InputCommand {
                agent_id: "agent".into(),
                conversation_id: "conversation".into(),
                operation_id: id.into(),
                target: Some(c::NativeTarget {
                    snapshot_id: "source".into(),
                    native_session_id: "native".into(),
                }),
            }),
            result: Some(c::input_report::Result::Steered(())),
        });
        let mut publication = Publication::default();
        publication.record(
            (report, permit),
            Err(tonic::Status::unavailable("Lost ACK")),
        );
        assert!(publication.settled.is_empty());
        assert!(
            std::future::Future::poll(
                waiting.as_mut(),
                &mut std::task::Context::from_waker(std::task::Waker::noop())
            )
            .is_pending()
        );
        assert!(operations.admit(leases::Kind::Input, id)?.is_none());
        let retry = publication.failed.pop().ok_or("Missing report retry")?;
        publication.record(retry, Ok(true));
        waiting.await?;
        operations.settled.extend(publication.settled);
        assert!(operations.admit(leases::Kind::Input, id)?.is_none());
        // Only the command-checking loop can order registry release. An
        // already absent native fact is also an idempotent closure.
        operations.publish(
            tonic::transport::Endpoint::from_static("http://127.0.0.1:1").connect_lazy(),
            c::CaptureSource::default(),
            &BearerSecret::generate()?,
        )?;
        assert!(operations.admit(leases::Kind::Input, id)?.is_none());
        assert!(
            operations
                .releasing
                .join_next()
                .await
                .ok_or("Missing release")??
                .is_none()
        );
        assert!(operations.admit(leases::Kind::Input, id)?.is_some());
        operations.finish().await?;
        native.finish().await?;
        Ok(())
    }

    #[tokio::test]
    async fn deleted_input_closure_does_not_acknowledge_a_user_receipt() -> Result<(), AppError> {
        let temporary = tempfile::tempdir()?;
        let native = chapchoo_harness::NativeOwner::new(chapchoo_harness::NativeStorage::open(
            temporary.path(),
        )?)?;
        let operations = Operations::new(native.handle());
        let binding = chapchoo_harness::NativeAuthenticationBinding {
            agent_id: "agent".into(),
            conversation_id: "conversation".into(),
            operation_id: "setup".into(),
            execution_id: "execution".into(),
            authentication_id: "authentication".into(),
            source: [1; 24],
        };
        let owner = operations.ordering.open(binding.clone());
        let mut permit = operations
            .admit(leases::Kind::Input, "deleted")?
            .ok_or("Missing permit")?;
        permit.ordering = Some(operations.ordering.begin(&binding, "deleted")?);
        let mut waiting = std::pin::pin!(owner.snapshot()?.ready());
        let mut publication = Publication::default();
        publication.record(
            (
                Report::Input(c::InputReport {
                    source: None,
                    command: Some(c::InputCommand {
                        operation_id: "deleted".into(),
                        ..Default::default()
                    }),
                    result: Some(c::input_report::Result::Steered(())),
                }),
                permit,
            ),
            Ok(false),
        );
        assert!(
            std::future::Future::poll(
                waiting.as_mut(),
                &mut std::task::Context::from_waker(std::task::Waker::noop())
            )
            .is_pending(),
            "A closure ACK must not open publication"
        );
        drop(publication);
        assert!(waiting.await.is_err());
        assert!(operations.ordering.begin(&binding, "manual-retry").is_ok());
        operations.finish().await?;
        native.finish().await?;
        Ok(())
    }
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
