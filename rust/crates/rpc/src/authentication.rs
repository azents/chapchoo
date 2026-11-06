//! Client-only, current-session catalogue and explicit device-advance operations.

use crate::{
    ingress as input,
    mapping::timestamp,
    status::{reject, status},
};
use chapchoo_protocol::authentication as wire;
use chapchoo_service::{self as app, AcceptanceTime, AuthenticationService, DomainReason};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tonic::{Request, Response, Status};

#[derive(Debug, Clone)]
pub struct AuthenticationServices {
    service: Arc<AuthenticationService>,
    admission: Arc<AtomicBool>,
}
impl AuthenticationServices {
    pub fn new(service: Arc<AuthenticationService>) -> Self {
        Self {
            service,
            admission: Arc::new(AtomicBool::new(true)),
        }
    }
    pub fn with_admission(mut self, admission: Arc<AtomicBool>) -> Self {
        self.admission = admission;
        self
    }
    fn admit<T>(&self, request: &Request<T>) -> Result<app::SecretDigest, Status> {
        if !self.admission.load(Ordering::Acquire) {
            return Err(Status::unavailable("Service is not admitting new requests"));
        }
        input::actor(request)
    }
    pub fn server(&self) -> wire::authentications_server::AuthenticationsServer<Self> {
        wire::authentications_server::AuthenticationsServer::new(self.clone())
            .max_decoding_message_size(256 * 1024)
            .max_encoding_message_size(2 * 1024 * 1024)
    }
}
fn scope(value: Option<wire::Scope>) -> Result<app::AuthenticationScope, Status> {
    match value.and_then(|scope| scope.owner) {
        Some(wire::scope::Owner::Personal(())) => Ok(app::AuthenticationScope::Personal),
        Some(wire::scope::Owner::TeamId(team_id)) => {
            input::id(&team_id)?;
            Ok(app::AuthenticationScope::Team { team_id })
        }
        None => Err(reject(DomainReason::InvalidInput)),
    }
}
fn scope_wire(value: app::AuthenticationScope) -> wire::Scope {
    wire::Scope {
        owner: Some(match value {
            app::AuthenticationScope::Personal => wire::scope::Owner::Personal(()),
            app::AuthenticationScope::Team { team_id } => wire::scope::Owner::TeamId(team_id),
        }),
    }
}
fn kind(value: app::AuthenticationKind) -> i32 {
    (match value {
        app::AuthenticationKind::ChatGpt => wire::Kind::Chatgpt,
        app::AuthenticationKind::OpenAiApiKey => wire::Kind::OpenaiApiKey,
    }) as i32
}
fn summary(value: app::AuthenticationSummary) -> Result<wire::Authentication, Status> {
    Ok(wire::Authentication {
        id: value.id,
        scope: Some(scope_wire(value.scope)),
        kind: kind(value.kind),
        name: value.name,
        account_id: value.account_id,
        email: value.email,
        plan_type: value.plan_type,
        expires_at: value.expires_at.map(timestamp).transpose()?,
        current_request_id: value.current_request_id,
        created_at: Some(timestamp(value.created_at)?),
        updated_at: Some(timestamp(value.updated_at)?),
    })
}
fn accepted(value: app::AuthenticationAcceptedResult) -> wire::AcceptedResult {
    use wire::accepted_result::Result;
    wire::AcceptedResult {
        result: Some(match value {
            app::AuthenticationAcceptedResult::Record { authentication_id } => {
                Result::Record(wire::RecordIdentity { authentication_id })
            }
            app::AuthenticationAcceptedResult::Deleted { authentication_id } => {
                Result::Deleted(wire::RecordIdentity { authentication_id })
            }
            app::AuthenticationAcceptedResult::Selection {
                agent_id,
                authentication_id,
            } => Result::Selection(wire::SelectionIdentity {
                agent_id,
                authentication_id,
            }),
            app::AuthenticationAcceptedResult::Device { attempt_id } => {
                Result::Device(wire::DeviceIdentity { attempt_id })
            }
        }),
    }
}
fn operation(value: app::AuthenticationOperation) -> i32 {
    (match value {
        app::AuthenticationOperation::RegisterKey => wire::Operation::RegisterKey,
        app::AuthenticationOperation::ReplaceKey => wire::Operation::ReplaceKey,
        app::AuthenticationOperation::Rename => wire::Operation::Rename,
        app::AuthenticationOperation::Delete => wire::Operation::Delete,
        app::AuthenticationOperation::Select => wire::Operation::Select,
        app::AuthenticationOperation::StartDevice => wire::Operation::StartDevice,
        app::AuthenticationOperation::CancelDevice => wire::Operation::CancelDevice,
    }) as i32
}
fn progress(value: app::AuthenticationProgress) -> Result<wire::DeviceProgress, Status> {
    let (value, failure) = match value {
        app::AuthenticationProgress::Current(value) => (value, None),
        app::AuthenticationProgress::ProviderFailure { status, reason } => (
            status,
            Some(match reason {
                app::DeviceProviderFailure::Rejected => wire::ProviderFailure::Rejected,
                app::DeviceProviderFailure::Unavailable => wire::ProviderFailure::Unavailable,
                app::DeviceProviderFailure::InvalidResponse => {
                    wire::ProviderFailure::InvalidResponse
                }
                app::DeviceProviderFailure::Uncertain => wire::ProviderFailure::Uncertain,
            } as i32),
        ),
    };
    use wire::device_progress::State;
    Ok(wire::DeviceProgress {
        attempt_id: value.attempt_id,
        expires_at: Some(timestamp(value.expires_at)?),
        provider_failure: failure,
        state: Some(match value.state {
            app::DeviceAuthenticationState::Starting => State::Starting(()),
            app::DeviceAuthenticationState::Pending {
                user_code,
                next_check_at,
            } => State::Pending(wire::PendingDevice {
                user_code,
                verification_uri: app::DEVICE_VERIFICATION_URI.into(),
                next_check_at: Some(timestamp(next_check_at)?),
            }),
            app::DeviceAuthenticationState::Checking => State::Checking(()),
            app::DeviceAuthenticationState::Exchanging => State::Exchanging(()),
            app::DeviceAuthenticationState::Registered { authentication_id } => {
                State::Registered(wire::RecordIdentity { authentication_id })
            }
            app::DeviceAuthenticationState::Cancelled => State::Cancelled(()),
            app::DeviceAuthenticationState::Expired => State::Expired(()),
            app::DeviceAuthenticationState::Failed => State::Failed(()),
            app::DeviceAuthenticationState::Superseded => State::Superseded(()),
            app::DeviceAuthenticationState::Uncertain => State::Uncertain(()),
        }),
    })
}
pub(crate) fn selection(
    value: app::AgentAuthenticationSelection,
) -> Result<wire::AgentSelection, Status> {
    Ok(wire::AgentSelection {
        authentication_id: value.authentication_id,
        name: value.name,
        kind: kind(value.kind),
        request_id: value.request_id,
        selected_at: Some(timestamp(value.selected_at)?),
    })
}
fn key(value: Vec<u8>) -> Result<app::ExternalSecret, Status> {
    app::ExternalSecret::new(value).map_err(|_| reject(DomainReason::InvalidInput))
}

#[tonic::async_trait]
impl wire::authentications_server::Authentications for AuthenticationServices {
    async fn list(
        &self,
        request: Request<wire::ListRequest>,
    ) -> Result<Response<wire::AuthenticationList>, Status> {
        let actor = self.admit(&request)?;
        let value = request.into_inner();
        let scope = scope(value.scope)?;
        let limit = if value.limit == 0 { 50 } else { value.limit };
        let items = self
            .service
            .list(&actor, &scope, value.after.as_deref(), limit, input::now()?)
            .await
            .map_err(status)?;
        let next_after = if usize::try_from(limit).is_ok_and(|limit| items.len() == limit) {
            items.last().map(|item| item.id.clone())
        } else {
            None
        };
        Ok(Response::new(wire::AuthenticationList {
            items: items.into_iter().map(summary).collect::<Result<_, _>>()?,
            next_after,
        }))
    }
    async fn get(
        &self,
        request: Request<wire::RecordRequest>,
    ) -> Result<Response<wire::Authentication>, Status> {
        let actor = self.admit(&request)?;
        let value = request.into_inner();
        let scope = scope(value.scope)?;
        let item = self
            .service
            .get(&actor, &scope, &value.authentication_id, input::now()?)
            .await
            .map_err(status)?;
        Ok(Response::new(summary(item)?))
    }
    async fn register_key(
        &self,
        request: Request<wire::RegisterKeyRequest>,
    ) -> Result<Response<wire::AcceptedResult>, Status> {
        let actor = self.admit(&request)?;
        let value = request.into_inner();
        let key = key(value.key)?;
        let draft = app::RegisterApiKey {
            scope: scope(value.scope)?,
            request_id: value.request_id,
            name: value.name,
            key,
        };
        Ok(Response::new(accepted(
            self.service
                .register_key(&actor, &draft, input::now()?)
                .await
                .map_err(status)?,
        )))
    }
    async fn replace_key(
        &self,
        request: Request<wire::ReplaceKeyRequest>,
    ) -> Result<Response<wire::AcceptedResult>, Status> {
        let actor = self.admit(&request)?;
        let value = request.into_inner();
        let key = key(value.key)?;
        let draft = app::ReplaceApiKey {
            scope: scope(value.scope)?,
            request_id: value.request_id,
            authentication_id: value.authentication_id,
            expected_request_id: value.expected_request_id,
            key,
        };
        Ok(Response::new(accepted(
            self.service
                .replace_key(&actor, &draft, input::now()?)
                .await
                .map_err(status)?,
        )))
    }
    async fn rename(
        &self,
        request: Request<wire::RenameRequest>,
    ) -> Result<Response<wire::AcceptedResult>, Status> {
        let actor = self.admit(&request)?;
        let value = request.into_inner();
        let draft = app::RenameAuthentication {
            scope: scope(value.scope)?,
            request_id: value.request_id,
            authentication_id: value.authentication_id,
            expected_request_id: value.expected_request_id,
            name: value.name,
        };
        Ok(Response::new(accepted(
            self.service
                .rename(&actor, &draft, input::now()?)
                .await
                .map_err(status)?,
        )))
    }
    async fn delete(
        &self,
        request: Request<wire::DeleteRequest>,
    ) -> Result<Response<wire::DeleteResult>, Status> {
        let actor = self.admit(&request)?;
        let value = request.into_inner();
        let draft = app::DeleteAuthentication {
            scope: scope(value.scope)?,
            request_id: value.request_id,
            authentication_id: value.authentication_id,
            expected_request_id: value.expected_request_id,
        };
        let outcome = match self
            .service
            .delete(&actor, &draft, input::now()?)
            .await
            .map_err(status)?
        {
            app::DeleteAuthenticationOutcome::Accepted(result) => {
                wire::delete_result::Outcome::Accepted(accepted(result))
            }
            app::DeleteAuthenticationOutcome::InUse => wire::delete_result::Outcome::InUse(()),
        };
        Ok(Response::new(wire::DeleteResult {
            outcome: Some(outcome),
        }))
    }
    async fn request_result(
        &self,
        request: Request<wire::RequestResultRequest>,
    ) -> Result<Response<wire::HistoricalResult>, Status> {
        let actor = self.admit(&request)?;
        let result = self
            .service
            .request_result(&actor, &request.into_inner().request_id, input::now()?)
            .await
            .map_err(status)?;
        let result = match result {
            Some(result) => wire::HistoricalResult {
                operation: operation(result.operation),
                accepted: Some(accepted(result.accepted)),
                accepted_at: Some(timestamp(result.accepted_at)?),
            },
            None => wire::HistoricalResult::default(),
        };
        Ok(Response::new(result))
    }
    async fn start_device(
        &self,
        request: Request<wire::StartDeviceRequest>,
    ) -> Result<Response<wire::DeviceProgress>, Status> {
        let actor = self.admit(&request)?;
        let value = request.into_inner();
        let draft = app::StartDeviceAuthentication {
            scope: scope(value.scope)?,
            request_id: value.request_id,
            name: value.name,
            target: value.target.map(|value| app::ReauthenticateTarget {
                authentication_id: value.authentication_id,
                expected_request_id: value.expected_request_id,
            }),
        };
        Ok(Response::new(progress(
            self.service
                .start_device(&actor, draft, AcceptanceTime::System)
                .await
                .map_err(status)?,
        )?))
    }
    async fn device_status(
        &self,
        request: Request<wire::DeviceRequest>,
    ) -> Result<Response<wire::DeviceProgress>, Status> {
        let actor = self.admit(&request)?;
        Ok(Response::new(progress(
            self.service
                .device_status(&actor, &request.into_inner().attempt_id, input::now()?)
                .await
                .map_err(status)?,
        )?))
    }
    async fn poll_device(
        &self,
        request: Request<wire::DeviceRequest>,
    ) -> Result<Response<wire::DeviceProgress>, Status> {
        let actor = self.admit(&request)?;
        Ok(Response::new(progress(
            self.service
                .poll_device(
                    &actor,
                    request.into_inner().attempt_id,
                    AcceptanceTime::System,
                )
                .await
                .map_err(status)?,
        )?))
    }
    async fn cancel_device(
        &self,
        request: Request<wire::CancelDeviceRequest>,
    ) -> Result<Response<wire::DeviceProgress>, Status> {
        let actor = self.admit(&request)?;
        let value = request.into_inner();
        Ok(Response::new(progress(
            self.service
                .cancel_device(&actor, &value.attempt_id, &value.request_id, input::now()?)
                .await
                .map_err(status)?,
        )?))
    }
    async fn select_agent(
        &self,
        request: Request<wire::SelectAgentRequest>,
    ) -> Result<Response<wire::AcceptedResult>, Status> {
        let actor = self.admit(&request)?;
        let value = request.into_inner();
        let draft = app::SelectAgentAuthentication {
            request_id: value.request_id,
            agent_id: value.agent_id,
            authentication_id: value.authentication_id,
            expected: value
                .expected
                .map(|value| app::ExpectedAgentAuthentication {
                    authentication_id: value.authentication_id,
                    request_id: value.request_id,
                }),
        };
        Ok(Response::new(accepted(
            self.service
                .select_agent(&actor, &draft, input::now()?)
                .await
                .map_err(status)?,
        )))
    }
}
