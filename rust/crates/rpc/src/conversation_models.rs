use chapchoo_domain::{NativeConversationId, SnapshotId};
use chapchoo_protocol::conversations as wire;
use chapchoo_service::{
    ConversationModelChoice, ConversationModelProjection, ConversationModelSaveReceipt,
    ConversationModelTarget,
};
use tonic::Status;

pub(crate) fn target(value: Option<wire::ModelTarget>) -> Result<ConversationModelTarget, Status> {
    let value = value.ok_or_else(|| Status::invalid_argument("Model target is required"))?;
    let native = value
        .native
        .ok_or_else(|| Status::invalid_argument("Native target is required"))?;
    Ok(ConversationModelTarget {
        conversation_id: value.conversation_id,
        snapshot: SnapshotId::parse(native.snapshot_id)
            .map_err(|_| Status::invalid_argument("Captured snapshot is invalid"))?,
        native_session: NativeConversationId::parse(native.native_session_id)
            .map_err(|_| Status::invalid_argument("Native session is invalid"))?,
        launch_operation_id: value.launch_operation_id,
    })
}

fn wire_target(value: ConversationModelTarget) -> wire::ModelTarget {
    wire::ModelTarget {
        conversation_id: value.conversation_id,
        native: Some(wire::NativeTarget {
            snapshot_id: value.snapshot.as_str().into(),
            native_session_id: value.native_session.as_str().into(),
        }),
        launch_operation_id: value.launch_operation_id,
    }
}

fn choice(value: ConversationModelChoice) -> wire::ModelChoice {
    wire::ModelChoice {
        id: value.id,
        label: value.label,
    }
}

pub(crate) fn projection(
    value: Option<ConversationModelProjection>,
) -> wire::OwnerConversationModel {
    value.map_or_else(wire::OwnerConversationModel::default, |model| {
        wire::OwnerConversationModel {
            target: Some(wire_target(model.target)),
            recovery: model.recovery_operation_id.and_then(|operation_id| {
                model.model.clone().map(|value| wire::ModelRecovery {
                    operation_id,
                    model: Some(choice(value)),
                })
            }),
            current: model.model.map(choice),
            saved_request_id: model.saved_request_id,
            options: model.options.map(|items| wire::ModelOptions {
                choices: items.into_iter().map(choice).collect(),
            }),
        }
    })
}

pub(crate) fn receipt(value: ConversationModelSaveReceipt) -> wire::ModelSaveReceipt {
    wire::ModelSaveReceipt {
        request_id: value.request_id,
        model: Some(choice(value.model)),
    }
}
