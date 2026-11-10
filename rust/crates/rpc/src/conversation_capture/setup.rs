use super::*;
use chapchoo_domain::{ExistingCodexPath, HarnessSupply};
use chapchoo_service::{
    ConversationDeliveryTarget, ConversationInputPreparation,
    ConversationPreparationFailure as Failure, ConversationPreparationOutcome as Outcome,
};
fn target(value: wire::NativeTarget) -> Result<ConversationDeliveryTarget, Status> {
    Ok(ConversationDeliveryTarget {
        snapshot: snapshot(value.snapshot_id)?,
        native_session: NativeConversationId::parse(value.native_session_id)
            .map_err(|_| Status::invalid_argument("Native target is invalid"))?,
    })
}
pub(super) async fn check(
    service: &ConversationCaptureServices,
    request: Request<wire::SetupCheck>,
) -> Result<Response<()>, Status> {
    service.admit()?;
    let credential = actor(&request)?;
    let value = request.into_inner();
    let origin = value
        .source
        .ok_or_else(|| Status::invalid_argument("Setup source is required"))?;
    let command = value
        .command
        .ok_or_else(|| Status::invalid_argument("Setup intent is required"))?;
    let supply = match command.supply.and_then(|supply| supply.choice) {
        Some(wire::harness_supply::Choice::Managed(())) => HarnessSupply::Managed,
        Some(wire::harness_supply::Choice::ExistingCodexPath(path)) => HarnessSupply::Existing(
            ExistingCodexPath::parse(path)
                .map_err(|_| Status::invalid_argument("Supply path is invalid"))?,
        ),
        None => return Err(Status::invalid_argument("Requested supply is required")),
    };
    let (fresh, existing) = match command.context {
        Some(wire::setup_command::Context::Initial(value)) => (
            Some((value.allow_fresh, value.require_retained_source)),
            None,
        ),
        Some(wire::setup_command::Context::Existing(value)) => (None, Some(target(value)?)),
        None => return Err(Status::invalid_argument("Setup context intent is required")),
    };
    service
        .service
        .check_setup(
            &source(&origin, &credential),
            &ConversationInputPreparation {
                agent_id: &command.agent_id,
                conversation_id: &command.conversation_id,
                operation_id: &command.operation_id,
            },
            &supply,
            fresh,
            existing.as_ref(),
        )
        .await
        .map_err(status)?;
    Ok(Response::new(()))
}
pub(super) async fn report(
    service: &ConversationCaptureServices,
    request: Request<wire::SetupReport>,
) -> Result<Response<()>, Status> {
    service.admit()?;
    let credential = actor(&request)?;
    let value = request.into_inner();
    let origin = value
        .source
        .ok_or_else(|| Status::invalid_argument("Setup source is required"))?;
    let outcome = match value.result {
        Some(wire::setup_report::Result::Ready(value)) => Outcome::SessionReady(target(value)?),
        Some(wire::setup_report::Result::Failed(value)) => {
            Outcome::Failed(match wire::SetupFailure::try_from(value) {
                Ok(wire::SetupFailure::NotAdmitted) => Failure::NotAdmitted,
                Ok(wire::SetupFailure::BeforeNativeLaunch) => Failure::BeforeNativeLaunch,
                Ok(wire::SetupFailure::NativeRejected) => Failure::NativeRejected,
                Ok(wire::SetupFailure::SourceUnavailable) => Failure::SourceUnavailable,
                Ok(wire::SetupFailure::RetirementUnknown) => Failure::RetirementUnknown,
                _ => {
                    return Err(Status::invalid_argument(
                        "Setup failure evidence is invalid",
                    ));
                }
            })
        }
        None => return Err(Status::invalid_argument("Setup outcome is required")),
    };
    service
        .service
        .report_setup(
            &source(&origin, &credential),
            &ConversationInputPreparation {
                agent_id: &value.agent_id,
                conversation_id: &value.conversation_id,
                operation_id: &value.operation_id,
            },
            &outcome,
        )
        .await
        .map_err(status)?;
    Ok(Response::new(()))
}
