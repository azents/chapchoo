use super::*;
use chapchoo_domain::{ConversationSource, SnapshotId};
use chapchoo_persistence::{ConversationRegionRead, EventChunkRead, EventPayloadKind};
use chapchoo_protocol::conversations::{
    self as capture, connector_capture_client::ConnectorCaptureClient,
};
pub(super) struct Context<'a> {
    pub store: &'a Store,
    pub owner: &'a chapchoo_crypto::SecretDigest,
    pub agent: &'a chapchoo_persistence::AgentSummary,
    pub endpoint: &'a str,
    pub source: capture::CaptureSource,
    pub key: &'a str,
    pub user_key: &'a str,
}
pub(super) async fn exercise(context: Context<'_>) -> Result<(), TestError> {
    let Context {
        store,
        owner,
        agent,
        endpoint,
        source,
        key,
        user_key,
    } = context;
    let now = AcceptanceTime::System.now()?;
    let accepted = super::input_owner::exercise(store, owner, agent).await?;
    let conversation = accepted.conversation_id;
    let mut client = ConnectorCaptureClient::connect(endpoint.to_owned())
        .await?
        .max_decoding_message_size(64 * 1024)
        .max_encoding_message_size(64 * 1024);
    let inspect = capture::CaptureRequest {
        source: Some(source.clone()),
        conversation_id: conversation.clone(),
    };
    let initial = client
        .inspect(authenticated(inspect.clone(), key)?)
        .await?
        .into_inner();
    assert_eq!(initial.status, capture::CaptureStatus::NotStarted as i32);
    assert!(initial.position.is_none());
    super::owner_input::exercise(
        &Context {
            store,
            owner,
            agent,
            endpoint,
            source: source.clone(),
            key,
            user_key,
        },
        &conversation,
    )
    .await?;
    let bind = capture::BindCaptureRequest {
        source: Some(source.clone()),
        conversation_id: conversation.clone(),
        expected_previous_snapshot_id: None,
        snapshot_id: "rpc-source".into(),
        native_session_id: "native-fixture".into(),
    };
    let bound = client
        .bind(authenticated(bind.clone(), key)?)
        .await?
        .into_inner();
    assert_eq!(bound.status, capture::CaptureStatus::Loading as i32);
    let position = capture::SourcePosition {
        ordinal: 1,
        occurrence_id: "actual-capture-occurrence".into(),
    };
    let complete = capture::CompleteCaptureRequest {
        source: Some(source.clone()),
        conversation_id: conversation.clone(),
        snapshot_id: "rpc-source".into(),
        boundary: Some(position.clone()),
    };
    assert_eq!(
        client
            .complete(authenticated(complete.clone(), key)?)
            .await
            .err()
            .ok_or("Unpublished boundary completed")?
            .code(),
        Code::FailedPrecondition
    );
    let text = "한글🦀".repeat(9000);
    let raw=format!(r#"{{"jsonrpc":"2.0","method":"session/update","params":{{"sessionId":"native-fixture","update":{{"sessionUpdate":"agent_message_chunk","messageId":"message","content":{{"type":"text","text":"{text}"}}}}}},"unknown":"retained"}}"#).into_bytes();
    assert!(raw.len() > 64 * 1024);
    for round in 0..2 {
        for (index, chunk) in raw.chunks(32 * 1024).enumerate() {
            let offset = u32::try_from(index * 32 * 1024)?;
            let piece = capture::CapturePiece {
                source: Some(source.clone()),
                conversation_id: conversation.clone(),
                snapshot_id: "rpc-source".into(),
                position: Some(position.clone()),
                transfer_id: format!("transfer-{round}"),
                total_bytes: u32::try_from(raw.len())?,
                offset,
                content: chunk.to_vec(),
            };
            let reply = client
                .append(authenticated(piece.clone(), key)?)
                .await?
                .into_inner();
            let end = index * 32 * 1024 + chunk.len();
            if end == raw.len() {
                assert!(matches!(
                    reply.outcome,
                    Some(capture::capture_piece_result::Outcome::Published(_))
                ));
            } else {
                assert!(
                    matches!(reply.outcome,Some(capture::capture_piece_result::Outcome::Accepted(capture::PieceAccepted {next_offset})) if usize::try_from(next_offset)?==end)
                );
            }
            if index == 0 && round == 0 {
                let partial = client
                    .inspect(authenticated(inspect.clone(), key)?)
                    .await?
                    .into_inner();
                assert!(partial.position.is_none());
                let mut conflict = piece;
                conflict.content[0] ^= 1;
                assert_eq!(
                    client
                        .append(authenticated(conflict, key)?)
                        .await
                        .err()
                        .ok_or("Conflicting transfer accepted")?
                        .code(),
                    Code::Aborted
                );
                assert!(
                    store
                        .current_conversation(owner, &agent.id, now)
                        .await?
                        .ok_or("Missing conversation")?
                        .source
                        .position()
                        .is_none()
                );
            }
        }
    }
    let done = client
        .complete(authenticated(complete, key)?)
        .await?
        .into_inner();
    assert_eq!(done.status, capture::CaptureStatus::Ready as i32);
    let snapshot = SnapshotId::parse("rpc-source".into())?;
    let stored = store
        .current_conversation(owner, &agent.id, now)
        .await?
        .ok_or("Missing stored conversation")?;
    let position = stored.source.position().ok_or("Missing source cursor")?;
    assert_eq!(position.ordinal(), 1);
    let mut retained = Vec::new();
    while retained.len() < raw.len() {
        let chunk = store
            .conversation_event_chunk(
                owner,
                &EventChunkRead {
                    agent_id: &agent.id,
                    conversation_id: &conversation,
                    snapshot: &snapshot,
                    occurrence: position.occurrence(),
                    kind: EventPayloadKind::Raw,
                    offset: u32::try_from(retained.len())?,
                    length: 32768,
                },
                now,
            )
            .await?;
        assert!(!chunk.bytes.is_empty());
        retained.extend(chunk.bytes);
    }
    assert_eq!(retained, raw);
    let records = store
        .conversation_region(
            owner,
            &ConversationRegionRead {
                agent_id: &agent.id,
                conversation_id: &conversation,
                snapshot: &snapshot,
                before: None,
                limit: 25,
            },
            now,
        )
        .await?;
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].native_id, "message");
    super::owner_records::exercise(
        &Context {
            store,
            owner,
            agent,
            endpoint,
            source: source.clone(),
            key,
            user_key,
        },
        &conversation,
        &text,
    )
    .await?;
    super::native_state::exercise(
        &Context {
            store,
            owner,
            agent,
            endpoint,
            source: source.clone(),
            key,
            user_key,
        },
        &conversation,
    )
    .await?;
    super::input_wire::exercise(
        &Context {
            store,
            owner,
            agent,
            endpoint,
            source: source.clone(),
            key,
            user_key,
        },
        &conversation,
    )
    .await?;
    super::answer_wire::exercise(
        &Context {
            store,
            owner,
            agent,
            endpoint,
            source: source.clone(),
            key,
            user_key,
        },
        &conversation,
    )
    .await?;
    assert!(
        store
            .conversation_input(
                owner,
                &agent.id,
                &conversation,
                "capture-fixture-input",
                now
            )
            .await?
            .is_some()
    );
    let replacement = client
        .bind(authenticated(
            capture::BindCaptureRequest {
                expected_previous_snapshot_id: Some("rpc-source".into()),
                snapshot_id: "rpc-replacement".into(),
                ..bind
            },
            key,
        )?)
        .await?
        .into_inner();
    assert_eq!(replacement.status, capture::CaptureStatus::Loading as i32);
    assert!(replacement.position.is_none());
    assert!(
        store
            .conversation_input(
                owner,
                &agent.id,
                &conversation,
                "capture-fixture-input",
                now
            )
            .await?
            .is_none()
    );
    let stale = capture::CapturePiece {
        source: Some(source.clone()),
        conversation_id: conversation.clone(),
        snapshot_id: "rpc-source".into(),
        position: Some(capture::SourcePosition {
            ordinal: 1,
            occurrence_id: "actual-capture-occurrence".into(),
        }),
        transfer_id: "late-old-source".into(),
        total_bytes: 1,
        offset: 0,
        content: vec![b'x'],
    };
    assert_eq!(
        client
            .append(authenticated(stale, key)?)
            .await
            .err()
            .ok_or("Old source accepted")?
            .code(),
        Code::Aborted
    );
    assert!(matches!(
        store
            .current_conversation(owner, &agent.id, now)
            .await?
            .ok_or("Missing conversation")?
            .source,
        ConversationSource::Loading { position: None, .. }
    ));
    Ok(())
}
