//! Scoped component credential adapters. Peer methods are not registered here.
use crate::{
    ingress as input,
    mapping::timestamp,
    status::{reject, status},
};
use chapchoo_protocol::control as wire;
use chapchoo_service::{
    AcceptanceTime, ComponentKeyResult, ComputersService, ConnectorSource, DomainReason,
    ProviderSource,
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tonic::{Request, Response, Status};

#[derive(Debug, Clone)]
struct Context {
    service: Arc<ComputersService>,
    control_id: String,
    admission: Arc<AtomicBool>,
    connections: crate::ComponentConnectionServices,
}
impl Context {
    fn admit(&self) -> Result<(), Status> {
        if self.admission.load(Ordering::Acquire) {
            Ok(())
        } else {
            Err(Status::unavailable("Service is not admitting new requests"))
        }
    }
}
#[derive(Debug, Clone)]
pub struct ComponentAuthenticationServices {
    context: Context,
}
impl ComponentAuthenticationServices {
    pub fn new(
        service: Arc<ComputersService>,
        control_id: String,
        connections: crate::ComponentConnectionServices,
    ) -> Self {
        Self {
            context: Context {
                service,
                control_id,
                admission: Arc::new(AtomicBool::new(true)),
                connections,
            },
        }
    }
    pub fn with_admission(mut self, admission: Arc<AtomicBool>) -> Self {
        self.context.admission = admission;
        self
    }
    pub fn provider_server(
        &self,
    ) -> wire::provider_authentication_server::ProviderAuthenticationServer<
        ProviderAuthenticationAdapter,
    > {
        wire::provider_authentication_server::ProviderAuthenticationServer::new(
            ProviderAuthenticationAdapter {
                context: self.context.clone(),
            },
        )
        .max_decoding_message_size(64 * 1024)
    }
    pub fn connector_server(
        &self,
    ) -> wire::connector_authentication_server::ConnectorAuthenticationServer<
        ConnectorAuthenticationAdapter,
    > {
        wire::connector_authentication_server::ConnectorAuthenticationServer::new(
            ConnectorAuthenticationAdapter {
                context: self.context.clone(),
            },
        )
        .max_decoding_message_size(64 * 1024)
    }
}
#[derive(Debug, Clone)]
pub struct ProviderAuthenticationAdapter {
    context: Context,
}
#[derive(Debug, Clone)]
pub struct ConnectorAuthenticationAdapter {
    context: Context,
}
fn key(value: ComponentKeyResult) -> Result<wire::ComponentKey, Status> {
    Ok(wire::ComponentKey {
        token: value.token.expose().to_owned(),
        issued_at: Some(timestamp(value.issued_at)?),
        expires_at: Some(timestamp(value.expires_at)?),
        proactive_after: Some(timestamp(value.proactive_after)?),
    })
}
fn owner_authority(value: i32) -> Result<wire::OwnerAuthority, Status> {
    match wire::OwnerAuthority::try_from(value) {
        Ok(wire::OwnerAuthority::System) => Ok(wire::OwnerAuthority::System),
        Ok(wire::OwnerAuthority::User) => Ok(wire::OwnerAuthority::User),
        _ => Err(reject(DomainReason::InvalidInput)),
    }
}
#[tonic::async_trait]
impl wire::provider_authentication_server::ProviderAuthentication
    for ProviderAuthenticationAdapter
{
    #[tracing::instrument(skip_all, fields(rpc = "report_provider_evidence"))]
    async fn report(&self, request: Request<wire::ProviderReport>) -> Result<Response<()>, Status> {
        self.context.admit()?;
        let credential = input::actor(&request)?;
        let value = request.into_inner();
        if value.resources.len() > 32 {
            return Err(Status::resource_exhausted("Resource batch is too large"));
        }
        let identity = value
            .source
            .ok_or_else(|| Status::invalid_argument("Provider source is required"))?;
        let computer = value
            .computer
            .ok_or_else(|| Status::invalid_argument("Computer context is required"))?;
        let source = ProviderSource {
            provider_id: &identity.provider_id,
            control_id: &identity.instance_id,
            connection_id: &identity.connection_id,
            credential: &credential,
        };
        self.context
            .service
            .verify_provider_source(&source)
            .await
            .map_err(status)?;
        for resource in value.resources {
            crate::reports::resource(&self.context.service, &source, &computer, resource).await?;
        }
        crate::reports::execution(&self.context.service, &source, &computer, value.execution)
            .await?;
        if value.cleanup_complete {
            self.context
                .service
                .confirm_computer_cleanup(&source, crate::peer::scope(&computer))
                .await
                .map_err(status)?;
        }
        Ok(Response::new(()))
    }
    #[tracing::instrument(skip_all, fields(rpc = "configure_system_provider"))]
    async fn configure_system_provider(
        &self,
        request: Request<wire::ProviderConfiguration>,
    ) -> Result<Response<()>, Status> {
        self.context.admit()?;
        let credential = input::authorization(&request)?;
        let value = request.into_inner();
        let capabilities = value
            .capabilities
            .ok_or_else(|| Status::invalid_argument("Provider capabilities are required"))?;
        let substrate = match (
            wire::ProviderKind::try_from(value.kind),
            value.docker_daemon_id,
        ) {
            (Ok(wire::ProviderKind::Kubernetes), None) => {
                chapchoo_service::ProviderSubstrate::Kubernetes
            }
            (Ok(wire::ProviderKind::Docker), Some(daemon)) => {
                chapchoo_service::ProviderSubstrate::Docker(
                    chapchoo_service::DockerDaemonId::parse(daemon).map_err(|_| {
                        Status::invalid_argument("Docker daemon identity is invalid")
                    })?,
                )
            }
            _ => {
                return Err(Status::invalid_argument(
                    "Provider substrate and binding are invalid",
                ));
            }
        };
        self.context
            .service
            .configure_system_provider(
                &credential,
                &value.provider_id,
                &value.name,
                chapchoo_service::ProviderCapabilities {
                    create: capabilities.create,
                    start: capabilities.start,
                    stop: capabilities.stop,
                    restart: capabilities.restart,
                },
                &substrate,
                AcceptanceTime::System,
            )
            .await
            .map_err(status)?;
        Ok(Response::new(()))
    }
    #[tracing::instrument(skip_all, fields(rpc = "check_provider_command"))]
    async fn check_command(
        &self,
        request: Request<wire::ProviderCommandCheck>,
    ) -> Result<Response<()>, Status> {
        self.context.admit()?;
        let credential = input::actor(&request)?;
        let value = request.into_inner();
        let source = value
            .source
            .ok_or_else(|| Status::invalid_argument("Provider source is required"))?;
        let command = value
            .command
            .ok_or_else(|| Status::invalid_argument("Provider command is required"))?;
        let context = command
            .computer
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("Computer context is required"))?;
        crate::peer::validate_call(command.identity.as_ref(), context)?;
        if command
            .identity
            .as_ref()
            .is_none_or(|identity| identity.connection_id != source.connection_id)
        {
            return Err(Status::permission_denied("Command source does not match"));
        }
        let source = ProviderSource {
            provider_id: &source.provider_id,
            control_id: &source.instance_id,
            connection_id: &source.connection_id,
            credential: &credential,
        };
        let snapshot = self
            .context
            .service
            .provider_delivery_snapshot(
                &source,
                crate::peer::scope(context),
                (!context.execution_id.is_empty()).then_some(context.execution_id.as_str()),
            )
            .await
            .map_err(status)?;
        crate::peer::validate_provider(&command, &snapshot)?;
        self.context
            .connections
            .verify_native_dequeue(&source, &command)?;
        if snapshot.substrate.kind() == chapchoo_domain::ProviderKind::Docker
            && matches!(
                command.operation,
                Some(wire::provider_command::Operation::Cleanup(_))
            )
        {
            self.context
                .service
                .verify_native_cleanup_suffix(&source, crate::peer::scope(context))
                .await
                .map_err(status)?;
        }
        crate::peer::validate_bootstrap(&self.context.service, &command).await?;
        if matches!(
            command.operation,
            Some(wire::provider_command::Operation::Prepare(_))
        ) {
            self.context
                .service
                .mark_start_dispatched(&source, crate::peer::scope(context), &context.execution_id)
                .await
                .map_err(status)?;
        }
        Ok(Response::new(()))
    }
    #[tracing::instrument(skip_all, fields(rpc = "check_provider_connection"))]
    async fn check_connection(
        &self,
        request: Request<wire::ProviderConnectionCheck>,
    ) -> Result<Response<()>, Status> {
        self.context.admit()?;
        let credential = input::actor(&request)?;
        let value = request.into_inner();
        self.context
            .service
            .verify_provider_source(&ProviderSource {
                provider_id: &value.provider_id,
                control_id: &value.instance_id,
                connection_id: &value.connection_id,
                credential: &credential,
            })
            .await
            .map_err(status)?;
        Ok(Response::new(()))
    }
    #[tracing::instrument(skip_all, fields(rpc = "rotate_provider_key"))]
    async fn rotate_key(
        &self,
        request: Request<wire::ProviderKeyRequest>,
    ) -> Result<Response<wire::ComponentKey>, Status> {
        self.context.admit()?;
        let credential = input::authorization(&request)?;
        let value = request.into_inner();
        let result = match owner_authority(value.authority)? {
            wire::OwnerAuthority::System => {
                self.context
                    .service
                    .rotate_system_provider_key(
                        &credential,
                        &value.provider_id,
                        &value.request_id,
                        AcceptanceTime::System,
                    )
                    .await
            }
            wire::OwnerAuthority::User => {
                self.context
                    .service
                    .rotate_user_provider_key(
                        &credential.digest(),
                        &value.provider_id,
                        &value.request_id,
                        AcceptanceTime::System,
                    )
                    .await
            }
            wire::OwnerAuthority::Unspecified => return Err(reject(DomainReason::InvalidInput)),
        }
        .map_err(status)?;
        Ok(Response::new(key(result)?))
    }
    #[tracing::instrument(skip_all, fields(rpc = "revoke_provider_key"))]
    async fn revoke_key(
        &self,
        request: Request<wire::ProviderKeyRequest>,
    ) -> Result<Response<()>, Status> {
        self.context.admit()?;
        let credential = input::authorization(&request)?;
        let value = request.into_inner();
        match owner_authority(value.authority)? {
            wire::OwnerAuthority::System => {
                self.context
                    .service
                    .revoke_system_provider_key(
                        &credential,
                        &value.provider_id,
                        &value.request_id,
                        AcceptanceTime::System,
                    )
                    .await
            }
            wire::OwnerAuthority::User => {
                self.context
                    .service
                    .revoke_user_provider_key(
                        &credential.digest(),
                        &value.provider_id,
                        &value.request_id,
                        AcceptanceTime::System,
                    )
                    .await
            }
            wire::OwnerAuthority::Unspecified => return Err(reject(DomainReason::InvalidInput)),
        }
        .map_err(status)?;
        Ok(Response::new(()))
    }
    #[tracing::instrument(skip_all, fields(rpc = "issue_connector_session"))]
    async fn issue_connector_session(
        &self,
        request: Request<wire::ConnectorSessionRequest>,
    ) -> Result<Response<wire::ConnectorSession>, Status> {
        self.context.admit()?;
        let credential = input::actor(&request)?;
        let value = request.into_inner();
        let result = self
            .context
            .service
            .issue_connector_session_from_source(
                &ProviderSource {
                    provider_id: &value.provider_id,
                    control_id: &self.context.control_id,
                    connection_id: &value.connection_id,
                    credential: &credential,
                },
                &value.execution_id,
                &value.request_id,
                AcceptanceTime::System,
            )
            .await
            .map_err(status)?;
        Ok(Response::new(wire::ConnectorSession {
            session_id: result.id,
            execution_id: result.execution_id,
            accepted_request_id: result.accepted_request_id,
            initial_key: result.initial_key.expose().to_owned(),
        }))
    }
    #[tracing::instrument(skip_all, fields(rpc = "revoke_connector_session"))]
    async fn revoke_connector_session(
        &self,
        request: Request<wire::RevokeConnectorSessionRequest>,
    ) -> Result<Response<()>, Status> {
        self.context.admit()?;
        let credential = input::actor(&request)?;
        let value = request.into_inner();
        self.context
            .service
            .revoke_connector_session_from_source(
                &ProviderSource {
                    provider_id: &value.provider_id,
                    control_id: &self.context.control_id,
                    connection_id: &value.connection_id,
                    credential: &credential,
                },
                &value.session_id,
                &value.request_id,
                AcceptanceTime::System,
            )
            .await
            .map_err(status)?;
        Ok(Response::new(()))
    }
}
#[tonic::async_trait]
impl wire::connector_authentication_server::ConnectorAuthentication
    for ConnectorAuthenticationAdapter
{
    #[tracing::instrument(skip_all, fields(rpc = "check_connector_command"))]
    async fn check_command(
        &self,
        request: Request<wire::ConnectorCommandCheck>,
    ) -> Result<Response<()>, Status> {
        self.context.admit()?;
        let credential = input::actor(&request)?;
        let value = request.into_inner();
        let source = value
            .source
            .ok_or_else(|| Status::invalid_argument("Connector source is required"))?;
        let command = value
            .command
            .ok_or_else(|| Status::invalid_argument("Connector command is required"))?;
        let context = command
            .computer
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("Computer context is required"))?;
        crate::peer::validate_call(command.identity.as_ref(), context)?;
        if command
            .identity
            .as_ref()
            .is_none_or(|identity| identity.connection_id != source.connection_id)
            || command.observe.is_none()
        {
            return Err(Status::permission_denied("Command source does not match"));
        }
        let snapshot = self
            .context
            .service
            .delivery_snapshot(
                None,
                crate::peer::scope(context),
                Some(&context.execution_id),
            )
            .await
            .map_err(status)?;
        let identity = self
            .context
            .service
            .verify_connector_source(&ConnectorSource {
                session_id: &source.session_id,
                control_id: &source.instance_id,
                connection_id: &source.connection_id,
                credential: &credential,
            })
            .await
            .map_err(status)?;
        if identity.computer_id != context.computer_id
            || identity.execution_id != context.execution_id
            || snapshot.computer.current_execution_id.as_deref() != Some(&context.execution_id)
        {
            return Err(Status::permission_denied(
                "Connector execution does not match",
            ));
        }
        Ok(Response::new(()))
    }
    #[tracing::instrument(skip_all, fields(rpc = "check_connector_connection"))]
    async fn check_connection(
        &self,
        request: Request<wire::ConnectorConnectionCheck>,
    ) -> Result<Response<()>, Status> {
        self.context.admit()?;
        let credential = input::actor(&request)?;
        let value = request.into_inner();
        self.context
            .service
            .verify_connector_source(&ConnectorSource {
                session_id: &value.session_id,
                control_id: &value.instance_id,
                connection_id: &value.connection_id,
                credential: &credential,
            })
            .await
            .map_err(status)?;
        Ok(Response::new(()))
    }
    #[tracing::instrument(skip_all, fields(rpc = "exchange_connector_key"))]
    async fn exchange(
        &self,
        request: Request<wire::ConnectorKeyRequest>,
    ) -> Result<Response<wire::ComponentKey>, Status> {
        self.context.admit()?;
        let credential = input::actor(&request)?;
        let value = request.into_inner();
        let result = self
            .context
            .service
            .exchange_connector_key(
                &value.session_id,
                &credential,
                &value.request_id,
                AcceptanceTime::System,
            )
            .await
            .map_err(status)?;
        Ok(Response::new(key(result)?))
    }
    #[tracing::instrument(skip_all, fields(rpc = "refresh_connector_key"))]
    async fn refresh(
        &self,
        request: Request<wire::ConnectorKeyRequest>,
    ) -> Result<Response<wire::ComponentKey>, Status> {
        self.context.admit()?;
        let credential = input::actor(&request)?;
        let value = request.into_inner();
        let result = self
            .context
            .service
            .refresh_connector_key(
                &value.session_id,
                &credential,
                &value.request_id,
                AcceptanceTime::System,
            )
            .await
            .map_err(status)?;
        Ok(Response::new(key(result)?))
    }
}
