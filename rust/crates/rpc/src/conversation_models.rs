use chapchoo_domain::{
    ConversationModelCapabilities, ConversationModelTuning, ConversationPermissionMode,
    ConversationSpeed, NativeConversationId, SnapshotId,
};
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
        capabilities: value.capabilities.map(|options| wire::ModelCapabilities {
            efforts: options.efforts,
            speeds: options.speeds.into_iter().map(wire_speed).collect(),
        }),
    }
}

fn speed(value: i32) -> Result<ConversationSpeed, Status> {
    match wire::ModelSpeed::try_from(value) {
        Ok(wire::ModelSpeed::Standard) => Ok(ConversationSpeed::Standard),
        Ok(wire::ModelSpeed::Fast) => Ok(ConversationSpeed::Fast),
        Ok(wire::ModelSpeed::Ultrafast) => Ok(ConversationSpeed::Ultrafast),
        _ => Err(Status::invalid_argument("Unknown response speed")),
    }
}

fn wire_speed(value: ConversationSpeed) -> i32 {
    (match value {
        ConversationSpeed::Standard => wire::ModelSpeed::Standard,
        ConversationSpeed::Fast => wire::ModelSpeed::Fast,
        ConversationSpeed::Ultrafast => wire::ModelSpeed::Ultrafast,
    }) as i32
}

pub(crate) fn from_wire_choice(
    value: wire::ModelChoice,
) -> Result<ConversationModelChoice, Status> {
    Ok(ConversationModelChoice {
        id: value.id,
        label: value.label,
        capabilities: value
            .capabilities
            .map(|options| {
                let speeds = options
                    .speeds
                    .into_iter()
                    .map(speed)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok::<_, Status>(ConversationModelCapabilities {
                    efforts: options.efforts,
                    speeds,
                })
            })
            .transpose()?,
    })
}

pub(crate) fn tuning(value: Option<wire::ModelTuning>) -> Result<ConversationModelTuning, Status> {
    let value = value.unwrap_or_default();
    Ok(ConversationModelTuning {
        effort: value.effort,
        speed: value.speed.map(speed).transpose()?,
        permission_mode: value.permission_mode.map(permission_mode).transpose()?,
    })
}

pub(crate) fn wire_tuning(value: ConversationModelTuning) -> Option<wire::ModelTuning> {
    (!value.is_inherited()).then(|| wire::ModelTuning {
        effort: value.effort,
        speed: value.speed.map(wire_speed),
        permission_mode: value.permission_mode.map(wire_permission_mode),
    })
}

pub(crate) fn permission_mode(value: i32) -> Result<ConversationPermissionMode, Status> {
    match wire::PermissionMode::try_from(value) {
        Ok(wire::PermissionMode::ApproveForMe) => Ok(ConversationPermissionMode::ApproveForMe),
        Ok(wire::PermissionMode::FullAccess) => Ok(ConversationPermissionMode::FullAccess),
        Ok(wire::PermissionMode::AskForApproval) => Ok(ConversationPermissionMode::AskForApproval),
        _ => Err(Status::invalid_argument("Unknown permission mode")),
    }
}

fn wire_permission_mode(value: ConversationPermissionMode) -> i32 {
    (match value {
        ConversationPermissionMode::ApproveForMe => wire::PermissionMode::ApproveForMe,
        ConversationPermissionMode::FullAccess => wire::PermissionMode::FullAccess,
        ConversationPermissionMode::AskForApproval => wire::PermissionMode::AskForApproval,
    }) as i32
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
            saved_tuning: wire_tuning(model.tuning),
            options: model.options.map(|items| wire::ModelOptions {
                choices: items.into_iter().map(choice).collect(),
                permission_modes: model
                    .permission_modes
                    .into_iter()
                    .map(wire_permission_mode)
                    .collect(),
            }),
        }
    })
}

pub(crate) fn receipt(value: ConversationModelSaveReceipt) -> wire::ModelSaveReceipt {
    wire::ModelSaveReceipt {
        request_id: value.request_id,
        model: value.model.map(choice),
        tuning: wire_tuning(value.tuning),
    }
}
