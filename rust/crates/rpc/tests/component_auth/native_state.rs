use super::*;
use chapchoo_domain::{NativeConversationId, SnapshotId};
use chapchoo_persistence::{
    AcceptConversationStop, ConversationDeliveryTarget, ConversationStopAdmission,
    ConversationStopState, ConversationWorkTarget,
};
use chapchoo_protocol::conversations::{
    self as c, connector_capture_client::ConnectorCaptureClient,
};
pub(super) async fn exercise(
    context: &super::capture::Context<'_>,
    conversation: &str,
) -> Result<(), TestError> {
    let mut client = ConnectorCaptureClient::connect(context.endpoint.to_owned()).await?;
    let target = c::NativeTarget {
        snapshot_id: "rpc-source".into(),
        native_session_id: "native-fixture".into(),
    };
    let position = c::SourcePosition {
        ordinal: 2,
        occurrence_id: "state-running".into(),
    };
    let context_request = c::GoalContextRequest {
        source: Some(context.source.clone()),
        conversation_id: conversation.into(),
        target: Some(target.clone()),
    };
    assert_eq!(
        client
            .read_goal_context(authenticated(context_request.clone(), context.key)?)
            .await?
            .into_inner()
            .projection,
        b"[]",
    );
    assert_eq!(
        client
            .read_goal_context(authenticated(context_request.clone(), context.user_key)?)
            .await
            .err()
            .ok_or("Owner credential read component-only Goal context")?
            .code(),
        Code::Unauthenticated,
    );
    let stale_context = c::GoalContextRequest {
        target: Some(c::NativeTarget {
            snapshot_id: "stale-goal-source".into(),
            ..target.clone()
        }),
        ..context_request
    };
    assert_eq!(
        client
            .read_goal_context(authenticated(stale_context, context.key)?)
            .await
            .err()
            .ok_or("Stale source read Goal context")?
            .code(),
        Code::AlreadyExists,
    );
    let hook = c::NativeHookReport {
        source: Some(context.source.clone()),
        conversation_id: conversation.into(),
        target: Some(target.clone()),
        event: None,
    };
    assert_eq!(
        client
            .report_native_hook(authenticated(hook.clone(), context.key)?)
            .await
            .err()
            .ok_or("Unsupported hook event accepted")?
            .code(),
        Code::InvalidArgument,
    );
    let observed = c::NativeStateReport {
        source: Some(context.source.clone()),
        conversation_id: conversation.into(),
        target: Some(target.clone()),
        position: Some(position.clone()),
        foreground: c::Foreground::Running as i32,
        work_id: Some("work-live".into()),
        preparation: Some(c::NativePreparation {
            freshness: c::PreparationFreshness::TransientLookupFailure as i32,
            node: "24.0.0".into(),
            adapter: "2.6.0".into(),
            codex: "0.165.0".into(),
        }),
        authentication_id: None,
        applied_supply: Some(c::HarnessSupply {
            choice: Some(c::harness_supply::Choice::Managed(())),
        }),
    };
    assert_eq!(
        client
            .report_native(authenticated(observed.clone(), context.key)?)
            .await
            .err()
            .ok_or("Uncaptured state accepted")?
            .code(),
        Code::FailedPrecondition
    );
    let raw=br#"{"method":"session/update","params":{"sessionId":"native-fixture","update":{"sessionUpdate":"state_update","state":"running"}}}"#;
    client
        .append(authenticated(
            c::CapturePiece {
                source: Some(context.source.clone()),
                conversation_id: conversation.into(),
                snapshot_id: "rpc-source".into(),
                position: Some(position),
                transfer_id: "native-state".into(),
                total_bytes: u32::try_from(raw.len())?,
                offset: 0,
                content: raw.to_vec(),
            },
            context.key,
        )?)
        .await?;
    client
        .report_native(authenticated(observed.clone(), context.key)?)
        .await?;
    client
        .report_native(authenticated(observed, context.key)?)
        .await?;
    let native = context
        .store
        .conversation_native_observation(
            context.owner,
            &context.agent.id,
            conversation,
            AcceptanceTime::System.now()?,
        )
        .await?
        .ok_or("Missing native work observation")?;
    assert_eq!(native.work_id.as_deref(), Some("work-live"));
    assert_eq!(
        native
            .preparation
            .as_ref()
            .ok_or("Missing preparation")?
            .adapter,
        "2.6.0"
    );
    assert_eq!(
        native.applied_supply,
        Some(chapchoo_domain::HarnessSupply::Managed)
    );
    let mut owner =
        c::conversations_client::ConversationsClient::connect(context.endpoint.to_owned()).await?;
    let current = owner
        .current(authenticated(
            c::OwnerConversationRequest {
                agent_id: context.agent.id.clone(),
            },
            context.user_key,
        )?)
        .await?
        .into_inner();
    assert_eq!(
        current.saved_authentication_id.as_deref(),
        context
            .agent
            .authentication
            .as_ref()
            .map(|saved| saved.authentication_id.as_str())
    );
    let preparation = current
        .observed_native
        .as_ref()
        .and_then(|observed| observed.preparation.as_ref())
        .ok_or("Missing owner preparation")?;
    assert_eq!(
        preparation.freshness,
        c::PreparationFreshness::TransientLookupFailure as i32
    );
    assert_eq!(preparation.node, "24.0.0");
    assert_eq!(preparation.adapter, "2.6.0");
    assert_eq!(preparation.codex, "0.165.0");
    assert_eq!(
        current
            .observed_native
            .ok_or("Missing owner native observation")?
            .work_id
            .as_deref(),
        Some("work-live")
    );
    let supply = owner
        .get_supply(authenticated(
            c::OwnerConversationRequest {
                agent_id: context.agent.id.clone(),
            },
            context.user_key,
        )?)
        .await?
        .into_inner();
    assert!(matches!(
        supply.supply.and_then(|supply| supply.choice),
        Some(c::harness_supply::Choice::Managed(()))
    ));
    assert_eq!(
        owner
            .current(authenticated(
                c::OwnerConversationRequest {
                    agent_id: context.agent.id.clone()
                },
                context.key
            )?)
            .await
            .err()
            .ok_or("Connector credential gained owner admission")?
            .code(),
        Code::Unauthenticated
    );
    let work = ConversationWorkTarget {
        source: ConversationDeliveryTarget {
            snapshot: SnapshotId::parse("rpc-source".into())?,
            native_session: NativeConversationId::parse("native-fixture".into())?,
        },
        work_id: "work-live".into(),
    };
    let accepted = context
        .store
        .accept_conversation_stop(
            context.owner,
            &AcceptConversationStop {
                agent_id: &context.agent.id,
                conversation_id: conversation,
                request_id: "stop-rpc",
                target: &work,
            },
            AcceptanceTime::System.now()?,
        )
        .await?;
    let ConversationStopAdmission::Accepted(receipt) = accepted else {
        return Err("Expected new Stop".into());
    };
    let command = c::StopCommand {
        conversation_id: conversation.into(),
        operation_id: receipt.id,
        target: Some(c::WorkTarget {
            source: Some(target),
            work_id: "work-live".into(),
        }),
    };
    client
        .check_stop(authenticated(
            c::StopCheck {
                source: Some(context.source.clone()),
                command: Some(command.clone()),
            },
            context.key,
        )?)
        .await?;
    client
        .report_stop(authenticated(
            c::StopReport {
                source: Some(context.source.clone()),
                command: Some(command.clone()),
                cancel_published: true,
            },
            context.key,
        )?)
        .await?;
    client
        .report_stop(authenticated(
            c::StopReport {
                source: Some(context.source.clone()),
                command: Some(command.clone()),
                cancel_published: true,
            },
            context.key,
        )?)
        .await?;
    assert_eq!(
        context
            .store
            .conversation_stop(
                context.owner,
                &context.agent.id,
                conversation,
                "stop-rpc",
                AcceptanceTime::System.now()?
            )
            .await?
            .ok_or("Missing Stop")?
            .state,
        ConversationStopState::CancelPublished
    );
    let receipt = owner
        .stop_result(authenticated(
            c::OwnerStopResultRequest {
                agent_id: context.agent.id.clone(),
                conversation_id: conversation.into(),
                request_id: "stop-rpc".into(),
            },
            context.user_key,
        )?)
        .await?
        .into_inner()
        .receipt
        .ok_or("Missing owner Stop result")?;
    assert_eq!(receipt.state, c::StopState::CancelPublished as i32);
    assert_eq!(
        client
            .report_stop(authenticated(
                c::StopReport {
                    source: Some(context.source.clone()),
                    command: Some(command),
                    cancel_published: false
                },
                context.key
            )?)
            .await
            .err()
            .ok_or("Changed Stop outcome accepted")?
            .code(),
        Code::AlreadyExists
    );
    Ok(())
}

pub(super) async fn exercise_ending(
    context: &super::capture::Context<'_>,
    conversation: &str,
    launch_operation_id: &str,
) -> Result<(), TestError> {
    let mut client = ConnectorCaptureClient::connect(context.endpoint.to_owned()).await?;
    let running_position = c::SourcePosition {
        ordinal: 1,
        occurrence_id: "state-running".into(),
    };
    let running = br#"{"method":"session/update","params":{"sessionId":"native-fixture","update":{"sessionUpdate":"state_update","state":"running"}}}"#;
    client
        .append(authenticated(
            c::CapturePiece {
                source: Some(context.source.clone()),
                conversation_id: conversation.into(),
                snapshot_id: "rpc-replacement".into(),
                position: Some(running_position.clone()),
                transfer_id: "ending-fixture-running".into(),
                total_bytes: u32::try_from(running.len())?,
                offset: 0,
                content: running.to_vec(),
            },
            context.key,
        )?)
        .await?;
    let position = c::SourcePosition {
        ordinal: 2,
        occurrence_id: "state-ending".into(),
    };
    let hook = c::NativeHookReport {
        source: Some(context.source.clone()),
        conversation_id: conversation.into(),
        target: Some(c::NativeTarget {
            snapshot_id: "rpc-replacement".into(),
            native_session_id: "native-fixture".into(),
        }),
        event: Some(c::native_hook_report::Event::NormalEnding(
            c::NativeNormalEnding {
                position: Some(position.clone()),
                work_id: "rpc-ending-work".into(),
                launch_operation_id: launch_operation_id.into(),
            },
        )),
    };
    let raw = br#"{"method":"session/update","params":{"sessionId":"native-fixture","update":{"sessionUpdate":"state_update","state":"idle","stopReason":"end_turn"}}}"#;
    let mut mismatched = hook.clone();
    if let Some(c::native_hook_report::Event::NormalEnding(ending)) = mismatched.event.as_mut() {
        ending.position = Some(running_position);
    }
    assert_eq!(
        client
            .report_native_hook(authenticated(mismatched, context.key)?)
            .await
            .err()
            .ok_or("Captured Running update became a normal ending")?
            .code(),
        Code::FailedPrecondition,
    );
    assert_eq!(
        client
            .report_native_hook(authenticated(hook.clone(), context.key)?)
            .await
            .err()
            .ok_or("Unpublished ending accepted")?
            .code(),
        Code::FailedPrecondition,
    );
    client
        .append(authenticated(
            c::CapturePiece {
                source: hook.source.clone(),
                conversation_id: hook.conversation_id.clone(),
                snapshot_id: "rpc-replacement".into(),
                position: Some(position),
                transfer_id: "native-ending".into(),
                total_bytes: u32::try_from(raw.len())?,
                offset: 0,
                content: raw.to_vec(),
            },
            context.key,
        )?)
        .await?;
    let mut stale = hook.clone();
    stale
        .source
        .as_mut()
        .ok_or("Fixture lacks source")?
        .connection_id = "retired-link".into();
    assert!(
        client
            .report_native_hook(authenticated(stale, context.key)?)
            .await
            .is_err(),
        "Stale connection reported an ending",
    );
    assert_eq!(
        client
            .report_native_hook(authenticated(hook.clone(), context.user_key)?)
            .await
            .err()
            .ok_or("Owner credential reported an ending")?
            .code(),
        Code::Unauthenticated,
    );
    for _ in 0..2 {
        assert!(matches!(
            client
                .report_native_hook(authenticated(hook.clone(), context.key)?)
                .await?
                .into_inner()
                .outcome,
            Some(c::native_hook_outcome::Outcome::NoAction(()))
        ));
    }
    Ok(())
}
