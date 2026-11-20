use super::*;
use crate::conversation_models::target;
use chapchoo_service::{
    ConversationModelChoice, ConversationModelObservation, ConversationModelStart,
    ConversationModelStartAdmission, ConversationModelStartOutcome,
};

fn origin(value: Option<&wire::CaptureSource>) -> Result<&wire::CaptureSource, Status> {
    value.ok_or_else(|| Status::invalid_argument("Native source is required"))
}

fn choice(value: wire::ModelChoice) -> ConversationModelChoice {
    ConversationModelChoice {
        id: value.id,
        label: value.label,
    }
}

fn start(
    value: Option<wire::ModelStartTarget>,
    capable: bool,
) -> Result<ConversationModelStart, Status> {
    let value = value.ok_or_else(|| Status::invalid_argument("Model start target is required"))?;
    Ok(ConversationModelStart {
        target: target(value.model_target)?,
        operation_id: value.operation_id,
        capable,
    })
}

pub(super) async fn observe(
    service: &ConversationCaptureServices,
    request: Request<wire::ModelObservationReport>,
) -> Result<Response<()>, Status> {
    service.admit()?;
    let credential = actor(&request)?;
    let value = request.into_inner();
    let origin = origin(value.source.as_ref())?;
    let observation = ConversationModelObservation {
        target: target(value.target)?,
        order: value.order,
        current: value.current.map(choice),
        options: value
            .options
            .map(|items| items.choices.into_iter().map(choice).collect()),
        api_key_source: value
            .api_key_source
            .map(|bytes| bytes.try_into())
            .transpose()
            .map_err(|_| Status::invalid_argument("Invalid applied key source"))?,
    };
    service
        .service
        .report_model(&source(origin, &credential), &observation)
        .await
        .map_err(status)?;
    Ok(Response::new(()))
}

pub(super) async fn reserve(
    service: &ConversationCaptureServices,
    request: Request<wire::ModelStartRequest>,
) -> Result<Response<wire::ModelStartAdmission>, Status> {
    service.admit()?;
    let credential = actor(&request)?;
    let value = request.into_inner();
    let origin = origin(value.source.as_ref())?;
    let draft = start(value.target, value.capable)?;
    let admission = service
        .service
        .reserve_model_start(&source(origin, &credential), &draft)
        .await
        .map_err(status)?;
    let outcome = match admission {
        ConversationModelStartAdmission::Ungoverned => {
            wire::model_start_admission::Outcome::Ungoverned(())
        }
        ConversationModelStartAdmission::Reserved { model_id } => {
            wire::model_start_admission::Outcome::Reserved(wire::ReservedModelStart { model_id })
        }
    };
    Ok(Response::new(wire::ModelStartAdmission {
        outcome: Some(outcome),
    }))
}

pub(super) async fn settle(
    service: &ConversationCaptureServices,
    request: Request<wire::ModelStartReport>,
) -> Result<Response<()>, Status> {
    service.admit()?;
    let credential = actor(&request)?;
    let value = request.into_inner();
    let origin = origin(value.source.as_ref())?;
    let draft = start(value.target, false)?;
    let outcome = match value.outcome {
        Some(wire::model_start_report::Outcome::InsertedMessageId(id)) => {
            ConversationModelStartOutcome::Inserted(
                NativeConversationId::parse(id)
                    .map_err(|_| Status::invalid_argument("Invalid native insertion ID"))?,
            )
        }
        Some(wire::model_start_report::Outcome::NotPublished(())) => {
            ConversationModelStartOutcome::NotPublished
        }
        Some(wire::model_start_report::Outcome::ModelUnavailableBeforeStart(model)) => {
            ConversationModelStartOutcome::ModelUnavailableBeforeStart(model)
        }
        None => return Err(Status::invalid_argument("Model start outcome is required")),
    };
    service
        .service
        .settle_model_start(&source(origin, &credential), &draft, &outcome)
        .await
        .map_err(status)?;
    Ok(Response::new(()))
}
