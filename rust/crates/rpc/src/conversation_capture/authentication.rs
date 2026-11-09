//! Separate protected credential traffic; never session capture or ordinary DTOs.
use super::*;
use chapchoo_service::{
    AcceptanceTime, AppliedExecutionAuthentication, AuthenticationCredentialSource,
    AuthenticationKind, AuthenticationRefreshState, ExecutionAuthenticationConsumer,
    ExecutionAuthenticationDescriptor, ExecutionAuthenticationProgress,
};
fn consumer(
    value: wire::ExecutionAuthenticationReference,
    credential: SecretDigest,
) -> Result<ExecutionAuthenticationConsumer, Status> {
    let source = value
        .source
        .ok_or_else(|| Status::invalid_argument("Execution source is required"))?;
    Ok(ExecutionAuthenticationConsumer {
        session_id: source.session_id,
        control_id: source.control_id,
        connection_id: source.connection_id,
        credential,
        agent_id: value.agent_id,
        conversation_id: value.conversation_id,
        operation_id: value.operation_id,
    })
}
fn credential_source(bytes: Vec<u8>) -> Result<AuthenticationCredentialSource, Status> {
    Ok(AuthenticationCredentialSource::from_bytes(
        bytes
            .as_slice()
            .try_into()
            .map_err(|_| Status::invalid_argument("Credential source is invalid"))?,
    ))
}
fn applied(value: AppliedExecutionAuthentication) -> wire::AppliedAuthentication {
    wire::AppliedAuthentication {
        operation_id: value.operation_id,
        execution_id: value.execution_id,
        authentication_id: value.authentication_id,
    }
}
fn descriptor(
    value: ExecutionAuthenticationDescriptor,
) -> Result<wire::ExecutionCredentialDescriptor, Status> {
    Ok(wire::ExecutionCredentialDescriptor {
        applied: Some(applied(value.applied)),
        credential_source: value.source.as_bytes().to_vec(),
        kind: match value.kind {
            AuthenticationKind::OpenAiApiKey => wire::ExecutionCredentialKind::ApiKey,
            AuthenticationKind::ChatGpt => wire::ExecutionCredentialKind::ChatgptTokens,
        } as i32,
        account_id: value.account_id,
        expires_at_millis: value.expires_at,
        total_bytes: u32::try_from(value.total_bytes)
            .map_err(|_| Status::internal("Native credential size is invalid"))?,
    })
}
pub(super) async fn binding(
    service: &ConversationCaptureServices,
    request: Request<wire::ExecutionAuthenticationReference>,
) -> Result<Response<wire::OptionalAppliedAuthentication>, Status> {
    service.admit()?;
    let credential = actor(&request)?;
    let target = consumer(request.into_inner(), credential)?;
    let binding = service
        .authentication
        .binding(&target)
        .await
        .map_err(status)?;
    Ok(Response::new(wire::OptionalAppliedAuthentication {
        applied: binding.map(applied),
    }))
}
pub(super) async fn begin(
    service: &ConversationCaptureServices,
    request: Request<wire::BeginAuthenticationRequest>,
) -> Result<Response<wire::AppliedAuthentication>, Status> {
    service.admit()?;
    let credential = actor(&request)?;
    let value = request.into_inner();
    let target = consumer(
        value
            .target
            .ok_or_else(|| Status::invalid_argument("Application target is required"))?,
        credential,
    )?;
    Ok(Response::new(applied(
        service
            .authentication
            .begin(&target, value.expected_previous_operation_id.as_deref())
            .await
            .map_err(status)?,
    )))
}
pub(super) async fn acquire(
    service: &ConversationCaptureServices,
    request: Request<wire::ExecutionAuthenticationReference>,
) -> Result<Response<wire::ExecutionAuthenticationProgress>, Status> {
    service.admit()?;
    let credential = actor(&request)?;
    let target = consumer(request.into_inner(), credential)?;
    let state = match service
        .authentication
        .acquire(target, AcceptanceTime::System)
        .await
        .map_err(status)?
    {
        ExecutionAuthenticationProgress::Current(value) => {
            wire::execution_authentication_progress::State::Current(descriptor(*value)?)
        }
        ExecutionAuthenticationProgress::Pending { attempt_id } => {
            wire::execution_authentication_progress::State::Pending(wire::AuthenticationPending {
                attempt_id,
            })
        }
        ExecutionAuthenticationProgress::Settled { attempt_id, state } => {
            wire::execution_authentication_progress::State::Settled(wire::AuthenticationSettled {
                attempt_id,
                state: match state {
                    AuthenticationRefreshState::Possible => {
                        wire::AuthenticationRefreshObservation::Possible
                    }
                    AuthenticationRefreshState::NotDispatched => {
                        wire::AuthenticationRefreshObservation::NotDispatched
                    }
                    AuthenticationRefreshState::Rejected => {
                        wire::AuthenticationRefreshObservation::Rejected
                    }
                    AuthenticationRefreshState::Uncertain => {
                        wire::AuthenticationRefreshObservation::Uncertain
                    }
                    AuthenticationRefreshState::Updated => {
                        wire::AuthenticationRefreshObservation::Updated
                    }
                } as i32,
            })
        }
    };
    Ok(Response::new(wire::ExecutionAuthenticationProgress {
        state: Some(state),
    }))
}
pub(super) async fn read(
    service: &ConversationCaptureServices,
    request: Request<wire::AuthenticationReadRequest>,
) -> Result<Response<wire::AuthenticationPiece>, Status> {
    service.admit()?;
    let credential = actor(&request)?;
    let value = request.into_inner();
    let target = consumer(
        value
            .target
            .ok_or_else(|| Status::invalid_argument("Application target is required"))?,
        credential,
    )?;
    let source = credential_source(value.credential_source)?;
    let chunk = service
        .authentication
        .read(&target, &source, value.offset as usize, 32 * 1024)
        .await
        .map_err(status)?;
    Ok(Response::new(wire::AuthenticationPiece {
        descriptor: Some(descriptor(chunk.descriptor)?),
        offset: value.offset,
        content: chunk.content.expose().to_vec(),
    }))
}
pub(super) async fn check(
    service: &ConversationCaptureServices,
    request: Request<wire::AuthenticationHandoffCheck>,
) -> Result<Response<()>, Status> {
    service.admit()?;
    let credential = actor(&request)?;
    let value = request.into_inner();
    let target = consumer(
        value
            .target
            .ok_or_else(|| Status::invalid_argument("Application target is required"))?,
        credential,
    )?;
    service
        .authentication
        .check(&target, &credential_source(value.credential_source)?)
        .await
        .map_err(status)?;
    Ok(Response::new(()))
}
