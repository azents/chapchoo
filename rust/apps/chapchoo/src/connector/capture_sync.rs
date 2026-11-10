//! Coalesced captured-copy synchronization. It never starts or replays native work.
mod live;
use super::*;
use chapchoo_domain::{EventOccurrenceId, SnapshotId, SourcePosition};
use chapchoo_harness::{CaptureLog, CapturePass, NativeHandle};
use chapchoo_protocol::conversations::{
    self as captured, connector_capture_client::ConnectorCaptureClient,
};
use live::publish_live;
use std::{collections::BTreeMap, sync::Arc};
use tokio::task::JoinSet;
type PublishedRequests = Arc<tokio::sync::Mutex<BTreeMap<String, captured::LiveRequestReport>>>;

pub(super) struct CaptureSync {
    native: NativeHandle,
    published: PublishedRequests,
    tasks: JoinSet<Result<(), AppError>>,
    stop: CancellationToken,
}
impl CaptureSync {
    pub(super) fn new(native: NativeHandle) -> Self {
        Self {
            native,
            published: Arc::default(),
            tasks: JoinSet::new(),
            stop: CancellationToken::new(),
        }
    }
    pub(super) fn schedule(
        &mut self,
        channel: Channel,
        origin: captured::CaptureSource,
        key: &BearerSecret,
    ) -> Result<(), AppError> {
        while let Some(result) = self.tasks.try_join_next() {
            match result? {
                Ok(()) => {}
                Err(error) => {
                    tracing::warn!(error=?error,"Captured source synchronization failed; retained observations remain available")
                }
            }
        }
        if !self.tasks.is_empty() {
            return Ok(());
        }
        let native = self.native.clone();
        let published = self.published.clone();
        let key = BearerSecret::parse(key.expose().to_owned())?;
        let stop = self.stop.clone();
        self.tasks.spawn(async move {
            // Cooperative stop preserves ownership of any started blocking
            // source read. finish() awaits this task through physical completion.
            if stop.is_cancelled() {
                return Ok(());
            }
            let Some(source) = native.capture_reader().current_source().await? else {
                return Ok(());
            };
            synchronize(source, channel.clone(), origin.clone(), &key, &stop).await?;
            publish_live(native, channel, origin, &key, &stop, published).await
        });
        Ok(())
    }
    pub(super) async fn finish(mut self) -> Result<(), AppError> {
        self.stop.cancel();
        let mut result = Ok(());
        while let Some(completed) = self.tasks.join_next().await {
            // Ordinary link/source failures are read-transfer failures, not a
            // reason to stop native work. Join failures belong to this owner.
            match completed {
                Ok(Err(error)) => {
                    tracing::warn!(error=?error,"Captured source synchronization ended without completing its pass")
                }
                Ok(Ok(())) => {}
                Err(error) => result = crate::completion::complete(result, Err(error.into())),
            }
        }
        result
    }
}
fn position(value: &SourcePosition) -> captured::SourcePosition {
    captured::SourcePosition {
        ordinal: value.ordinal(),
        occurrence_id: value.occurrence().as_str().into(),
    }
}
fn domain_position(value: captured::SourcePosition) -> Result<SourcePosition, AppError> {
    Ok(SourcePosition::new(
        value.ordinal,
        EventOccurrenceId::parse(value.occurrence_id)?,
    )?)
}
async fn next(
    mut pass: CapturePass,
) -> Result<(CapturePass, Option<chapchoo_harness::CapturedEvent>), AppError> {
    // The closure owns the pass/read permit even if its async waiter is lost.
    let (pass, event) = tokio::task::spawn_blocking(move || {
        let event = pass.next_event();
        (pass, event)
    })
    .await?;
    Ok((pass, event?))
}
/// One finite source pass is complete independently of later append. On response
/// loss, another pass asks the durable cursor again and resends observations only.
async fn synchronize(
    source: CaptureLog,
    channel: Channel,
    origin: captured::CaptureSource,
    key: &BearerSecret,
    stop: &CancellationToken,
) -> Result<(), AppError> {
    if stop.is_cancelled() {
        return Ok(());
    }
    let Some(local) = source.current_boundary()? else {
        return Ok(());
    };
    let mut client = ConnectorCaptureClient::new(channel)
        .max_decoding_message_size(64 * 1024)
        .max_encoding_message_size(64 * 1024);
    let Some(mut remote) = client
        .current(request(
            captured::CurrentCaptureRequest {
                source: Some(origin.clone()),
            },
            key,
        )?)
        .await?
        .into_inner()
        .conversation
    else {
        return Ok(());
    };
    if stop.is_cancelled() {
        return Ok(());
    }
    if remote.snapshot_id.as_deref() != Some(local.snapshot.as_str()) {
        remote = client
            .bind(request(
                captured::BindCaptureRequest {
                    source: Some(origin.clone()),
                    conversation_id: remote.conversation_id,
                    expected_previous_snapshot_id: remote.snapshot_id,
                    snapshot_id: local.snapshot.as_str().into(),
                    native_session_id: local.native_session.as_str().into(),
                },
                key,
            )?)
            .await?
            .into_inner();
    }
    if remote.native_session_id.as_deref() != Some(local.native_session.as_str()) {
        return Err("Captured native session does not match server state".into());
    }
    let after = remote.position.map(domain_position).transpose()?;
    if remote.status == captured::CaptureStatus::Ready as i32 && after == local.through {
        return Ok(());
    }
    let snapshot = SnapshotId::parse(
        remote
            .snapshot_id
            .ok_or("Captured source binding is missing")?,
    )?;
    let begin_snapshot = snapshot.clone();
    if stop.is_cancelled() {
        return Ok(());
    }
    let mut pass =
        tokio::task::spawn_blocking(move || source.begin(&begin_snapshot, after.as_ref()))
            .await??;
    loop {
        if stop.is_cancelled() {
            return Ok(());
        }
        let (returned, event) = next(pass).await?;
        pass = returned;
        let Some(event) = event else {
            break;
        };
        let total = u32::try_from(event.raw.len())?;
        if total == 0 || total > 16 * 1024 * 1024 {
            return Err("Captured event exceeds the supported transfer budget".into());
        }
        let transfer = mutation_id()?;
        for (index, piece) in event.raw.as_bytes().chunks(32 * 1024).enumerate() {
            if stop.is_cancelled() {
                return Ok(());
            }
            let offset = u32::try_from(index * 32 * 1024)?;
            let end = offset + u32::try_from(piece.len())?;
            let result = client
                .append(request(
                    captured::CapturePiece {
                        source: Some(origin.clone()),
                        conversation_id: remote.conversation_id.clone(),
                        snapshot_id: snapshot.as_str().into(),
                        position: Some(position(&event.position)),
                        transfer_id: transfer.clone(),
                        total_bytes: total,
                        offset,
                        content: piece.into(),
                    },
                    key,
                )?)
                .await?
                .into_inner();
            match result.outcome {
                Some(captured::capture_piece_result::Outcome::Accepted(accepted))
                    if end < total && accepted.next_offset == end => {}
                Some(captured::capture_piece_result::Outcome::Published(published))
                    if end == total
                        && published.position.as_ref() == Some(&position(&event.position)) => {}
                _ => {
                    return Err(
                        "Captured transfer acknowledgement does not match its exact piece".into(),
                    );
                }
            }
        }
    }
    if stop.is_cancelled() {
        return Ok(());
    }
    let completed = client
        .complete(request(
            captured::CompleteCaptureRequest {
                source: Some(origin),
                conversation_id: remote.conversation_id.clone(),
                snapshot_id: snapshot.as_str().into(),
                boundary: pass.boundary.through.as_ref().map(position),
            },
            key,
        )?)
        .await?
        .into_inner();
    if completed.conversation_id != remote.conversation_id
        || completed.snapshot_id.as_deref() != Some(snapshot.as_str())
        || completed.status != captured::CaptureStatus::Ready as i32
    {
        return Err("Captured pass completion does not match its exact source".into());
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::{future::Future, task::Poll};
    #[tokio::test]
    async fn shutdown_waits_for_the_actual_blocking_source_task() -> Result<(), AppError> {
        let saved = tempfile::tempdir()?;
        let native = chapchoo_harness::NativeOwner::new(chapchoo_harness::NativeStorage::open(
            saved.path(),
        )?)?;
        let mut sync = CaptureSync::new(native.handle());
        let (entered, entry) = tokio::sync::oneshot::channel();
        let (release, released) = tokio::sync::oneshot::channel();
        sync.tasks.spawn(async move {
            tokio::task::spawn_blocking(move || {
                let _ = entered.send(());
                released
                    .blocking_recv()
                    .map_err(|_| std::io::Error::other("Source read was abandoned"))
            })
            .await??;
            Ok(())
        });
        entry.await?;
        let stopped = sync.stop.clone();
        let mut finishing = Box::pin(sync.finish());
        std::future::poll_fn(|context| {
            assert!(
                finishing.as_mut().poll(context).is_pending(),
                "Shutdown abandoned a started blocking source read"
            );
            Poll::Ready(())
        })
        .await;
        assert!(stopped.is_cancelled());
        release
            .send(())
            .map_err(|_| "Blocking source read was abandoned")?;
        finishing.await?;
        assert_eq!(
            native.finish().await?,
            chapchoo_harness::Retirement::NoNativeStarted
        );
        Ok(())
    }
}
