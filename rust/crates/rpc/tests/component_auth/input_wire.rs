use super::*;
use chapchoo_domain::{ConversationTarget, ConversationText, NativeConversationId, SnapshotId};
use chapchoo_persistence::{
    AcceptConversationInput, ConversationDeliveryTarget, ConversationForwardingAdmission,
    ConversationOperationForwarding,
};
use chapchoo_protocol::conversations::{
    self as c, connector_capture_client::ConnectorCaptureClient,
};
pub(super) async fn exercise(
    context: &super::capture::Context<'_>,
    conversation: &str,
) -> Result<(), TestError> {
    let text = "한글🦀 original input\n".repeat(5000);
    let now = AcceptanceTime::System.now()?;
    let snapshot = SnapshotId::parse("rpc-source".into())?;
    let receipt = context
        .store
        .accept_conversation_input(
            context.owner,
            &AcceptConversationInput {
                agent_id: context.agent.id.clone(),
                conversation_id: Some(conversation.into()),
                request_id: "large-input-wire".into(),
                target: ConversationTarget::Snapshot(snapshot.clone()),
                text: ConversationText::parse(text.clone())?,
            },
            now,
        )
        .await?;
    let target = ConversationDeliveryTarget {
        snapshot,
        native_session: NativeConversationId::parse("native-fixture".into())?,
    };
    let draft = ConversationOperationForwarding {
        agent_id: &context.agent.id,
        conversation_id: conversation,
        operation_id: &receipt.id,
        target: &target,
    };
    let command = c::InputCommand {
        agent_id: context.agent.id.clone(),
        conversation_id: conversation.into(),
        operation_id: receipt.id.clone(),
        target: Some(c::NativeTarget {
            snapshot_id: "rpc-source".into(),
            native_session_id: "native-fixture".into(),
        }),
    };
    let mut client = ConnectorCaptureClient::connect(context.endpoint.to_owned())
        .await?
        .max_decoding_message_size(64 * 1024)
        .max_encoding_message_size(64 * 1024);
    assert_eq!(
        client
            .pull_input(authenticated(
                c::InputChunkRequest {
                    source: Some(context.source.clone()),
                    command: Some(command.clone()),
                    offset: 0
                },
                context.key
            )?)
            .await
            .err()
            .ok_or("Unsealed input body leaked")?
            .code(),
        Code::FailedPrecondition
    );
    let route = context
        .store
        .resolve_connector_route(&context.agent.computer.id)
        .await?
        .ok_or("Missing route")?;
    assert!(matches!(
        context
            .store
            .begin_conversation_input_forwarding(context.owner, &route, &draft, now)
            .await?,
        ConversationForwardingAdmission::Granted(_)
    ));
    let mut bytes = Vec::new();
    loop {
        let offset = u32::try_from(bytes.len())?;
        let chunk = client
            .pull_input(authenticated(
                c::InputChunkRequest {
                    source: Some(context.source.clone()),
                    command: Some(command.clone()),
                    offset,
                },
                context.key,
            )?)
            .await?
            .into_inner();
        assert_eq!(chunk.offset, offset);
        assert_eq!(chunk.total_bytes as usize, text.len());
        assert!(chunk.content.len() <= 32768);
        if chunk.content.is_empty() {
            break;
        }
        bytes.extend(chunk.content);
    }
    assert_eq!(bytes, text.as_bytes());
    let report = c::InputReport {
        source: Some(context.source.clone()),
        command: Some(command.clone()),
        result: Some(c::input_report::Result::Steered(())),
    };
    client
        .report_input(authenticated(report.clone(), context.key)?)
        .await?;
    client
        .report_input(authenticated(report, context.key)?)
        .await?;
    assert!(
        context
            .store
            .conversation_input_reference(
                context.owner,
                &context.agent.id,
                conversation,
                &receipt.request_id,
                now
            )
            .await?
            .is_none()
    );
    // After the pending operation is gone, another successful report only
    // closes the Connector's report owner. No delivery receipt is recreated.
    client
        .report_input(authenticated(
            c::InputReport {
                source: Some(context.source.clone()),
                command: Some(command.clone()),
                result: Some(c::input_report::Result::InsertedNativeMessageId(
                    "native-message".into(),
                )),
            },
            context.key,
        )?)
        .await?;
    assert_eq!(
        client
            .pull_input(authenticated(
                c::InputChunkRequest {
                    source: Some(context.source.clone()),
                    command: Some(command),
                    offset: 0,
                },
                context.key
            )?)
            .await
            .err()
            .ok_or("Consumed input body remained readable")?
            .code(),
        Code::FailedPrecondition
    );
    let setup_input = context
        .store
        .accept_conversation_input(
            context.owner,
            &AcceptConversationInput {
                agent_id: context.agent.id.clone(),
                conversation_id: Some(conversation.into()),
                request_id: "owner-setup-failure".into(),
                target: ConversationTarget::Snapshot(target.snapshot.clone()),
                text: ConversationText::parse("Synthetic setup status fixture".into())?,
            },
            now,
        )
        .await?;
    assert!(matches!(
        context
            .store
            .begin_conversation_input_preparation(
                context.owner,
                &route,
                &chapchoo_persistence::ConversationInputPreparation {
                    agent_id: &context.agent.id,
                    conversation_id: conversation,
                    operation_id: &setup_input.id,
                },
                now
            )
            .await?,
        chapchoo_persistence::ConversationPreparationAdmission::Granted { .. }
    ));
    client
        .report_setup(authenticated(
            c::SetupReport {
                source: Some(context.source.clone()),
                agent_id: context.agent.id.clone(),
                conversation_id: conversation.into(),
                operation_id: setup_input.id,
                result: Some(c::setup_report::Result::Failed(
                    c::SetupFailure::BeforeNativeLaunch as i32,
                )),
            },
            context.key,
        )?)
        .await?;
    let mut owner =
        c::conversations_client::ConversationsClient::connect(context.endpoint.to_owned()).await?;
    let status = owner
        .input_result(authenticated(
            c::OwnerInputResultRequest {
                agent_id: context.agent.id.clone(),
                conversation_id: conversation.into(),
                request_id: setup_input.request_id,
            },
            context.user_key,
        )?)
        .await?
        .into_inner();
    assert!(status.delivery.is_none());
    assert!(
        matches!(status.preparation.and_then(|preparation| preparation.observation),
        Some(c::input_preparation::Observation::Failed(failure)) if failure == c::SetupFailure::BeforeNativeLaunch as i32)
    );
    Ok(())
}
