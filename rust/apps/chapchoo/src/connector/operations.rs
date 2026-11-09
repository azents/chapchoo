//! Stream-independent native operation waiters and bounded later-result publication.
mod answers;
mod leases;
mod reported;
use super::*;
use chapchoo_domain::{ExistingCodexPath, HarnessSupply, NativeConversationId, SnapshotId};
use chapchoo_harness::{
    NativeHandle, NativeSetupFailure, NativeSetupIntent, NativeSetupObservation,
    NativeSetupOperation, NativeTextTarget,
};
use chapchoo_protocol::conversations::{
    self as c, connector_capture_client::ConnectorCaptureClient,
};
use std::sync::Arc;
use tokio::{
    sync::{OwnedSemaphorePermit, Semaphore},
    task::JoinSet,
};
#[derive(Clone)]
enum Report {
    Answer(c::AnswerReport),
    Stop(c::StopReport),
    Setup(c::SetupReport),
    Input(c::InputReport),
}
type Pending = (Report, leases::OperationPermit);
#[derive(Default)]
struct Publication {
    failed: Vec<Pending>,
    settled: Vec<Pending>,
}
pub(super) struct Operations {
    native: NativeHandle,
    slots: Arc<Semaphore>,
    tasks: JoinSet<Option<Pending>>,
    pending: Vec<Pending>,
    publishing: JoinSet<Publication>,
    releasing: JoinSet<Option<Pending>>,
    settled: Vec<Pending>,
    active: leases::Active,
    payload: Arc<Semaphore>,
    stop: CancellationToken,
    authentication: super::authentication_sync::AuthenticationSync,
}
impl Operations {
    pub(super) fn stop(&mut self, command: c::StopCommand) -> Result<bool, AppError> {
        let permit = match self.admit(leases::Kind::Stop, &command.operation_id)? {
            Some(permit) => permit,
            None => return Ok(false),
        };
        let target = command
            .target
            .as_ref()
            .ok_or("Stop work target is required")?;
        let source = target
            .source
            .as_ref()
            .ok_or("Stop native target is required")?;
        let native = chapchoo_harness::NativeWorkTarget {
            snapshot: SnapshotId::parse(source.snapshot_id.clone())?,
            native_session: NativeConversationId::parse(source.native_session_id.clone())?,
            work_id: target.work_id.clone(),
        };
        let rx = match self.native.admit_stop(command.operation_id.clone(), native) {
            Ok(rx) => rx,
            Err(chapchoo_harness::Error::NotAdmitted) => return Ok(false),
            Err(error) => return Err(error.into()),
        };
        self.tasks.spawn(async move {
            let published = match rx.await {
                Ok(Ok(chapchoo_harness::NativeStopObservation::CancelPublished)) => true,
                Ok(Ok(chapchoo_harness::NativeStopObservation::Rejected)) => false,
                _ => return None,
            };
            Some((
                Report::Stop(c::StopReport {
                    source: None,
                    command: Some(command),
                    cancel_published: published,
                }),
                permit,
            ))
        });
        Ok(true)
    }
    pub(super) fn new(native: NativeHandle) -> Self {
        Self {
            authentication: super::authentication_sync::AuthenticationSync::new(native.clone()),
            native,
            slots: Arc::new(Semaphore::new(32)),
            tasks: JoinSet::new(),
            pending: Vec::new(),
            publishing: JoinSet::new(),
            releasing: JoinSet::new(),
            settled: Vec::new(),
            active: Default::default(),
            payload: Arc::new(Semaphore::new(32 * 1024 * 1024)),
            stop: CancellationToken::new(),
        }
    }
    pub(super) fn setup(
        &mut self,
        command: c::SetupCommand,
        channel: Channel,
        origin: c::CaptureSource,
        key: &BearerSecret,
    ) -> Result<bool, AppError> {
        let permit = match self.admit(leases::Kind::Setup, &command.operation_id)? {
            Some(permit) => permit,
            None => return Ok(false),
        };
        let supply = match command
            .supply
            .as_ref()
            .and_then(|supply| supply.choice.as_ref())
        {
            Some(c::harness_supply::Choice::Managed(())) => HarnessSupply::Managed,
            Some(c::harness_supply::Choice::ExistingCodexPath(path)) => {
                HarnessSupply::Existing(ExistingCodexPath::parse(path.clone())?)
            }
            None => return Err("Setup supply is required".into()),
        };
        let intent = match command.context.as_ref() {
            Some(c::setup_command::Context::Initial(value)) => NativeSetupIntent::Initial {
                allow_fresh: value.allow_fresh,
            },
            Some(c::setup_command::Context::Existing(value)) => {
                NativeSetupIntent::Existing(NativeTextTarget {
                    snapshot: SnapshotId::parse(value.snapshot_id.clone())?,
                    native_session: NativeConversationId::parse(value.native_session_id.clone())?,
                })
            }
            None => return Err("Setup context is required".into()),
        };
        let authentication = Arc::new(super::authentication::Authentication::new(
            channel, origin, key, &command,
        )?);
        let rx = match self.native.admit_setup_with_authentication(
            NativeSetupOperation {
                operation_id: command.operation_id.clone(),
                supply,
                intent,
            },
            Some(authentication),
        ) {
            Ok(rx) => rx,
            Err(chapchoo_harness::Error::NotAdmitted) => return Ok(false),
            Err(error) => return Err(error.into()),
        };
        self.tasks.spawn(async move {
            let result = match rx.await {
                Ok(Ok(NativeSetupObservation::Ready(target))) => {
                    c::setup_report::Result::Ready(c::NativeTarget {
                        snapshot_id: target.snapshot.as_str().into(),
                        native_session_id: target.native_session.as_str().into(),
                    })
                }
                Ok(Ok(NativeSetupObservation::Failed(failure))) => {
                    c::setup_report::Result::Failed(match failure {
                        NativeSetupFailure::NotAdmitted => c::SetupFailure::NotAdmitted,
                        NativeSetupFailure::BeforeNativeLaunch => {
                            c::SetupFailure::BeforeNativeLaunch
                        }
                        NativeSetupFailure::NativeRejected => c::SetupFailure::NativeRejected,
                        NativeSetupFailure::SourceUnavailable => c::SetupFailure::SourceUnavailable,
                        NativeSetupFailure::RetirementUnknown => c::SetupFailure::RetirementUnknown,
                    } as i32)
                }
                // A lost local result is possible delivery, not a known failure.
                _ => return None,
            };
            Some((
                Report::Setup(c::SetupReport {
                    source: None,
                    agent_id: command.agent_id,
                    conversation_id: command.conversation_id,
                    operation_id: command.operation_id,
                    result: Some(result),
                }),
                permit,
            ))
        });
        Ok(true)
    }
    pub(super) fn input(
        &mut self,
        command: c::InputCommand,
        channel: Channel,
        origin: c::CaptureSource,
        key: &BearerSecret,
    ) -> Result<bool, AppError> {
        let permit = match self.admit(leases::Kind::Input, &command.operation_id)? {
            Some(permit) => permit,
            None => return Ok(false),
        };
        let value = command
            .target
            .as_ref()
            .ok_or("Native input target is required")?;
        let target = NativeTextTarget {
            snapshot: SnapshotId::parse(value.snapshot_id.clone())?,
            native_session: NativeConversationId::parse(value.native_session_id.clone())?,
        };
        let key = BearerSecret::parse(key.expose().to_owned())?;
        let native = self.native.clone();
        let payload = self.payload.clone();
        let stop = self.stop.clone();
        self.tasks.spawn(async move {
            let outcome = async {
                use chapchoo_harness::{NativeInputObservation, NativePromptAcceptance};
                let observed = native
                    .input_observation(command.operation_id.clone(), target.clone())
                    .await?;
                if let NativeInputObservation::Known(result) = observed {
                    return Ok::<_, AppError>(result);
                }
                if !matches!(observed, NativeInputObservation::Unseen) {
                    return Err("Native input outcome remains uncertain".into());
                }
                let mut client = ConnectorCaptureClient::new(channel)
                    .max_decoding_message_size(64 * 1024)
                    .max_encoding_message_size(64 * 1024);
                let mut bytes = Vec::new();
                let mut total = None;
                let mut budget = None;
                loop {
                    if stop.is_cancelled() {
                        return Err("Input transfer stopped before native dispatch".into());
                    }
                    let offset = u32::try_from(bytes.len())?;
                    let chunk = client
                        .pull_input(request(
                            c::InputChunkRequest {
                                source: Some(origin.clone()),
                                command: Some(command.clone()),
                                offset,
                            },
                            &key,
                        )?)
                        .await?
                        .into_inner();
                    if chunk.offset != offset
                        || chunk.total_bytes == 0
                        || chunk.total_bytes > 16 * 1024 * 1024
                        || chunk.content.is_empty()
                        || chunk.content.len() > 32768
                        || offset
                            .checked_add(u32::try_from(chunk.content.len())?)
                            .is_none_or(|end| end > chunk.total_bytes)
                        || total.is_some_and(|total| total != chunk.total_bytes)
                    {
                        return Err("Input transfer identity is inconsistent".into());
                    }
                    if total.is_none() {
                        budget = Some(payload.clone().try_acquire_many_owned(chunk.total_bytes)?);
                        bytes.try_reserve_exact(chunk.total_bytes as usize)?;
                        total = Some(chunk.total_bytes);
                    }
                    bytes.extend(chunk.content);
                    if bytes.len() == chunk.total_bytes as usize {
                        break;
                    }
                }
                if stop.is_cancelled() {
                    return Err("Input stopped before native dispatch".into());
                }
                let total = u32::try_from(bytes.len())?;
                let boundary = client
                    .pull_input(request(
                        c::InputChunkRequest {
                            source: Some(origin),
                            command: Some(command.clone()),
                            offset: total,
                        },
                        &key,
                    )?)
                    .await?
                    .into_inner();
                if boundary.offset != total
                    || boundary.total_bytes != total
                    || !boundary.content.is_empty()
                {
                    return Err("Input transfer completion is inconsistent".into());
                }
                let text = chapchoo_domain::ConversationText::parse(String::from_utf8(bytes)?)?;
                let result = native
                    .prompt(chapchoo_harness::NativeInputOperation {
                        operation_id: command.operation_id.clone(),
                        target,
                        text,
                    })
                    .await?;
                drop(budget);
                match result {
                    NativePromptAcceptance::Inserted { .. }
                    | NativePromptAcceptance::Steered
                    | NativePromptAcceptance::Failed => Ok(result),
                }
            }
            .await;
            let result = match outcome {
                Ok(chapchoo_harness::NativePromptAcceptance::Inserted { native_message_id }) => {
                    c::input_report::Result::InsertedNativeMessageId(native_message_id)
                }
                Ok(chapchoo_harness::NativePromptAcceptance::Steered) => {
                    c::input_report::Result::Steered(())
                }
                Ok(chapchoo_harness::NativePromptAcceptance::Failed) => {
                    c::input_report::Result::Failed(())
                }
                // No generic error is evidence of native non-delivery.
                Err(_) => return None,
            };
            Some((
                Report::Input(c::InputReport {
                    source: None,
                    command: Some(command),
                    result: Some(result),
                }),
                permit,
            ))
        });
        Ok(true)
    }
    pub(super) fn publish(
        &mut self,
        channel: Channel,
        origin: c::CaptureSource,
        key: &BearerSecret,
    ) -> Result<(), AppError> {
        self.authentication
            .schedule(channel.clone(), origin.clone(), key)?;
        while let Some(result) = self.tasks.try_join_next() {
            if let Some(report) = result? {
                self.pending.push(report);
            }
        }
        while let Some(result) = self.publishing.try_join_next() {
            let result = result?;
            self.pending.extend(result.failed);
            self.settled.extend(result.settled);
        }
        while let Some(result) = self.releasing.try_join_next() {
            if let Some(pending) = result? {
                self.settled.push(pending);
            }
        }
        // Enqueue release on the command-checking loop, never in the background
        // report task. A check already awaiting its response still sees the held
        // invocation lease before any native release can be enqueued.
        for pending in std::mem::take(&mut self.settled) {
            match self.native.admit_reported(pending.0.native_report()?) {
                Ok(receiver) => {
                    self.releasing.spawn(async move {
                        match receiver.await {
                            Ok(Ok(())) => None,
                            _ => Some(pending),
                        }
                    });
                }
                Err(_) => self.settled.push(pending),
            }
        }
        if !self.publishing.is_empty() || self.pending.is_empty() {
            return Ok(());
        }
        let pending = self.pending.pop().into_iter().collect::<Vec<_>>();
        let key = BearerSecret::parse(key.expose().to_owned())?;
        self.publishing.spawn(async move {
            let mut publication = Publication::default();
            let mut client = ConnectorCaptureClient::new(channel)
                .max_encoding_message_size(64 * 1024)
                .max_decoding_message_size(64 * 1024);
            for (report, permit) in pending {
                let result = match &report {
                    Report::Answer(report) => {
                        let mut report = report.clone();
                        report.source = Some(origin.clone());
                        match request(report, &key) {
                            Ok(request) => client.report_answer(request).await,
                            Err(_) => Err(tonic::Status::unavailable(
                                "Answer report request could not be prepared",
                            )),
                        }
                    }
                    Report::Stop(report) => {
                        let mut report = report.clone();
                        report.source = Some(origin.clone());
                        match request(report, &key) {
                            Ok(request) => client.report_stop(request).await,
                            Err(_) => Err(tonic::Status::unavailable(
                                "Stop report request could not be prepared",
                            )),
                        }
                    }
                    Report::Setup(report) => {
                        let mut report = report.clone();
                        report.source = Some(origin.clone());
                        match request(report, &key) {
                            Ok(request) => client.report_setup(request).await,
                            Err(_) => Err(tonic::Status::unavailable(
                                "Operation report request could not be prepared",
                            )),
                        }
                    }
                    Report::Input(report) => {
                        let mut report = report.clone();
                        report.source = Some(origin.clone());
                        match request(report, &key) {
                            Ok(request) => client.report_input(request).await,
                            Err(_) => Err(tonic::Status::unavailable(
                                "Operation report request could not be prepared",
                            )),
                        }
                    }
                };
                publication.record((report, permit), result.map(|_| ()));
            }
            publication
        });
        Ok(())
    }
    pub(super) async fn finish(mut self) -> Result<(), AppError> {
        self.stop.cancel();
        let mut result = self.authentication.finish().await;
        while let Some(completed) = self.tasks.join_next().await {
            if let Err(error) = completed {
                result = crate::completion::complete(result, Err(error.into()));
            }
        }
        while let Some(completed) = self.publishing.join_next().await {
            if let Err(error) = completed {
                result = crate::completion::complete(result, Err(error.into()));
            }
        }
        while let Some(completed) = self.releasing.join_next().await {
            if let Err(error) = completed {
                result = crate::completion::complete(result, Err(error.into()));
            }
        }
        result
    }
}
