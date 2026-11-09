use super::*;
use chapchoo_harness::{InteractiveRequest, NativeForeground};
pub(super) async fn publish_live(
    native: NativeHandle,
    channel: Channel,
    origin: captured::CaptureSource,
    key: &BearerSecret,
    stop: &CancellationToken,
    published: PublishedRequests,
) -> Result<(), AppError> {
    if stop.is_cancelled() {
        return Ok(());
    }
    let mut client = ConnectorCaptureClient::new(channel)
        .max_decoding_message_size(64 * 1024)
        .max_encoding_message_size(64 * 1024);
    let Some(current) = client
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
    if current.status != captured::CaptureStatus::Ready as i32 {
        return Ok(());
    }
    let target = captured::NativeTarget {
        snapshot_id: current.snapshot_id.ok_or("Missing source snapshot")?,
        native_session_id: current.native_session_id.ok_or("Missing native session")?,
    };
    match native.observe().await {
        Ok(observed) => {
            if let Some(position) = observed.position {
                let foreground = match observed.foreground {
                    NativeForeground::Idle => captured::Foreground::Idle,
                    NativeForeground::Running => captured::Foreground::Running,
                    NativeForeground::RequiresAction => captured::Foreground::RequiresAction,
                    NativeForeground::OtherActive => captured::Foreground::OtherActive,
                    NativeForeground::Unknown => return Ok(()),
                };
                if let Some(work) = &observed.work
                    && (work.snapshot.as_str() != target.snapshot_id
                        || work.native_session.as_str() != target.native_session_id)
                {
                    return Err("Native observed source changed".into());
                }
                client
                    .report_native(request(
                        captured::NativeStateReport {
                            source: Some(origin.clone()),
                            conversation_id: current.conversation_id.clone(),
                            target: Some(target.clone()),
                            position: Some(super::position(&position)),
                            foreground: foreground as i32,
                            work_id: observed.work.map(|work| work.work_id),
                        },
                        key,
                    )?)
                    .await?;
            }
        }
        Err(chapchoo_harness::Error::NotAdmitted) => {}
        Err(error) => return Err(error.into()),
    }
    if stop.is_cancelled() {
        return Ok(());
    }
    let mut active = BTreeMap::new();
    for pending in native.pending_requests().await? {
        let (token, position, kind) = match pending {
            InteractiveRequest::Permission { id, occurrence, .. } => {
                (id, occurrence, captured::RequestKind::Permission)
            }
            InteractiveRequest::Form { id, occurrence, .. } => {
                (id, occurrence, captured::RequestKind::Form)
            }
        };
        if active.len() >= 32 {
            return Err("Pending native requests exceeded their bounded profile".into());
        }
        active.insert(
            token.clone(),
            captured::LiveRequestReport {
                source: Some(origin.clone()),
                conversation_id: current.conversation_id.clone(),
                target: Some(target.clone()),
                position: Some(super::position(&position)),
                kind: kind as i32,
                responder_token: token,
                open: true,
            },
        );
    }
    let previous = published.lock().await.clone();
    for (token, report) in previous {
        if stop.is_cancelled() {
            return Ok(());
        }
        if report.conversation_id == current.conversation_id
            && report.target.as_ref() == Some(&target)
            && !active.contains_key(&token)
        {
            let closed = captured::LiveRequestReport {
                source: Some(origin.clone()),
                open: false,
                ..report
            };
            client.report_request(request(closed, key)?).await?;
        }
    }
    for report in active.values() {
        if stop.is_cancelled() {
            return Ok(());
        }
        client.report_request(request(report.clone(), key)?).await?;
    }
    *published.lock().await = active;
    Ok(())
}
