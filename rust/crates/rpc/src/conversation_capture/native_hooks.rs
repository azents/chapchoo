use super::*;
use chapchoo_service::{ConversationContinuationDecision, ConversationNormalEnding};

fn occurrence(
    target: Option<wire::NativeTarget>,
    ending: wire::NativeNormalEnding,
) -> Result<ConversationNormalEnding, Status> {
    Ok(ConversationNormalEnding {
        target: observations::target(target)?,
        position: position(
            ending
                .position
                .ok_or_else(|| Status::invalid_argument("Ending occurrence is required"))?,
        )?,
        work_id: ending.work_id,
        launch_operation_id: ending.launch_operation_id,
    })
}

pub(super) async fn report(
    service: &ConversationCaptureServices,
    request: Request<wire::NativeHookReport>,
) -> Result<Response<wire::NativeHookOutcome>, Status> {
    service.admit()?;
    let credential = actor(&request)?;
    let value = request.into_inner();
    let origin = value
        .source
        .as_ref()
        .ok_or_else(|| Status::invalid_argument("Native source is required"))?;
    let Some(wire::native_hook_report::Event::NormalEnding(ending)) = value.event else {
        return Err(Status::invalid_argument(
            "Supported native hook event is required",
        ));
    };
    let ending = occurrence(value.target, ending)?;
    let decision = service
        .service
        .handle_normal_ending(
            &source(origin, &credential),
            &value.conversation_id,
            &ending,
        )
        .await
        .map_err(status)?;
    let outcome = match decision {
        ConversationContinuationDecision::NoAction => {
            wire::native_hook_outcome::Outcome::NoAction(())
        }
        ConversationContinuationDecision::Admitted { operation_id } => {
            wire::native_hook_outcome::Outcome::ContinuationOperationId(operation_id)
        }
    };
    Ok(Response::new(wire::NativeHookOutcome {
        outcome: Some(outcome),
    }))
}

pub(super) async fn begin(
    service: &ConversationCaptureServices,
    request: Request<wire::ContinuationHandoff>,
) -> Result<Response<wire::ContinuationGrant>, Status> {
    service.admit()?;
    let credential = actor(&request)?;
    let value = request.into_inner();
    let origin = value
        .source
        .as_ref()
        .ok_or_else(|| Status::invalid_argument("Native source is required"))?;
    let ending = occurrence(
        value.target,
        value
            .ending
            .ok_or_else(|| Status::invalid_argument("Ending occurrence is required"))?,
    )?;
    let context = service
        .service
        .begin_continuation(
            &source(origin, &credential),
            &value.conversation_id,
            &value.operation_id,
            &ending,
            value.guards_model_start,
        )
        .await
        .map_err(status)?
        .map(|projection| wire::GoalContext { projection });
    Ok(Response::new(wire::ContinuationGrant { context }))
}

pub(super) async fn close(
    service: &ConversationCaptureServices,
    request: Request<wire::ContinuationHandoff>,
) -> Result<Response<()>, Status> {
    service.admit()?;
    let credential = actor(&request)?;
    let value = request.into_inner();
    let origin = value
        .source
        .as_ref()
        .ok_or_else(|| Status::invalid_argument("Native source is required"))?;
    let ending = occurrence(
        value.target,
        value
            .ending
            .ok_or_else(|| Status::invalid_argument("Ending occurrence is required"))?,
    )?;
    service
        .service
        .close_continuation(
            &source(origin, &credential),
            &value.conversation_id,
            &value.operation_id,
            &ending,
        )
        .await
        .map_err(status)?;
    Ok(Response::new(()))
}

pub(super) async fn restore_context(
    service: &ConversationCaptureServices,
    request: Request<wire::GoalContextRequest>,
) -> Result<Response<wire::GoalContext>, Status> {
    service.admit()?;
    let credential = actor(&request)?;
    let value = request.into_inner();
    let origin = value
        .source
        .as_ref()
        .ok_or_else(|| Status::invalid_argument("Native source is required"))?;
    let target = observations::target(value.target)?;
    let projection = service
        .service
        .read_goal_context(
            &source(origin, &credential),
            &value.conversation_id,
            &target,
        )
        .await
        .map_err(status)?;
    Ok(Response::new(wire::GoalContext { projection }))
}
