//! Restricted-network peer boundary. Never register this on the component listener.
use crate::{ComponentConnectionServices, status::status};
use chapchoo_protocol::control as wire;
use chapchoo_service::{
    BearerSecret, ComputerRequest, ComputersService, DeliverySnapshot, ManagedResource,
    SecretDigest,
};
use std::{sync::Arc, time::Duration};
use tokio::time::Instant;
use tonic::{Request, Response, Status};

const MAX_BUDGET: Duration = Duration::from_secs(10);
#[derive(Debug, Clone)]
pub struct PeerServices {
    service: Arc<ComputersService>,
    streams: ComponentConnectionServices,
    pending: Arc<tokio::sync::Semaphore>,
}
impl PeerServices {
    pub fn new(service: Arc<ComputersService>, streams: ComponentConnectionServices) -> Self {
        Self {
            service,
            streams,
            pending: Arc::new(tokio::sync::Semaphore::new(256)),
        }
    }
    pub fn server(&self) -> wire::peer_relay_server::PeerRelayServer<Self> {
        wire::peer_relay_server::PeerRelayServer::new(self.clone())
            .max_decoding_message_size(64 * 1024)
            .max_encoding_message_size(64 * 1024)
    }
    async fn authorize(
        &self,
        authority: Option<&wire::PeerAuthority>,
        context: &wire::ComputerContext,
    ) -> Result<DeliverySnapshot, Status> {
        let authority = authority
            .and_then(|a| a.context.as_ref())
            .ok_or_else(|| Status::permission_denied("Peer authority is required"))?;
        let actor = match authority {
            wire::peer_authority::Context::User(user) => Some(SecretDigest::from_bytes(
                user.session_credential_digest
                    .as_slice()
                    .try_into()
                    .map_err(|_| Status::unauthenticated("Session context is invalid"))?,
            )),
            wire::peer_authority::Context::Reconciliation(reconciliation) => {
                if reconciliation.computer.as_ref() != Some(context) {
                    return Err(Status::permission_denied(
                        "Reconciliation context does not match",
                    ));
                }
                None
            }
        };
        let snapshot = self
            .service
            .delivery_snapshot(
                actor.as_ref(),
                scope(context),
                (!context.execution_id.is_empty()).then_some(context.execution_id.as_str()),
            )
            .await
            .map_err(status)?;
        if let wire::peer_authority::Context::Reconciliation(reconciliation) = authority {
            let expected = reconciliation
                .owner
                .as_ref()
                .ok_or_else(|| Status::permission_denied("Reconciliation owner is required"))?;
            // Connector ownership takes precedence. The Provider's owner is the
            // management authority only while no eligible Connector exists.
            let route = match self
                .service
                .resolve_connector_route(&context.computer_id)
                .await
                .map_err(status)?
            {
                Some(route) => Some(route),
                None => self
                    .service
                    .resolve_provider_route(&snapshot.computer.provider_id)
                    .await
                    .map_err(status)?,
            }
            .ok_or_else(|| Status::unavailable("Reconciliation owner is unavailable"))?;
            if route.component.control_id != expected.instance_id
                || route.component.connection_id != expected.connection_id
            {
                return Err(Status::permission_denied(
                    "Reconciliation owner is no longer current",
                ));
            }
        }
        Ok(snapshot)
    }
    async fn provider(
        &self,
        invocation: wire::ProviderInvocation,
        deadline: Instant,
    ) -> Result<wire::ProviderInvocationResult, Status> {
        let command = invocation
            .command
            .ok_or_else(|| Status::invalid_argument("Provider command is required"))?;
        let context = command
            .computer
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("Computer context is required"))?;
        validate_call(command.identity.as_ref(), context)?;
        let snapshot = self
            .authorize(invocation.authority.as_ref(), context)
            .await?;
        if snapshot.computer.provider_id != invocation.provider_id {
            return Err(Status::permission_denied("Provider target does not match"));
        }
        validate_provider(&command, &snapshot)?;
        validate_bootstrap(&self.service, &command).await?;
        let owner = invocation
            .owner
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("Expected owner is required"))?;
        self.streams
            .invoke_provider(&invocation.provider_id, owner, command, deadline)
            .await
    }
    async fn connector(
        &self,
        invocation: wire::ConnectorInvocation,
        deadline: Instant,
    ) -> Result<wire::ConnectorInvocationResult, Status> {
        let command = invocation
            .command
            .ok_or_else(|| Status::invalid_argument("Connector command is required"))?;
        let context = command
            .computer
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("Computer context is required"))?;
        validate_call(command.identity.as_ref(), context)?;
        let snapshot = self
            .authorize(invocation.authority.as_ref(), context)
            .await?;
        if invocation.computer_id != context.computer_id
            || context.execution_id.is_empty()
            || snapshot.computer.current_execution_id.as_deref()
                != Some(context.execution_id.as_str())
            || snapshot
                .execution
                .as_ref()
                .is_none_or(|e| e.retired_at.is_some())
            || command.observe.is_none()
        {
            return Err(Status::failed_precondition(
                "Connector command does not match current execution",
            ));
        }
        let owner = invocation
            .owner
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("Expected owner is required"))?;
        self.streams
            .invoke_connector(&invocation.computer_id, owner, command, deadline)
            .await
    }
}
pub(crate) async fn validate_bootstrap(
    service: &ComputersService,
    command: &wire::ProviderCommand,
) -> Result<(), Status> {
    if let Some(wire::provider_command::Operation::Prepare(prepare)) = &command.operation {
        let context = command
            .computer
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("Computer context is required"))?;
        let key = BearerSecret::parse(prepare.connector_initial_key.clone())
            .map_err(|_| Status::invalid_argument("Bootstrap key is invalid"))?;
        service
            .verify_delivery_bootstrap(
                scope(context),
                &context.execution_id,
                &prepare.connector_session_id,
                &key.digest(),
            )
            .await
            .map_err(status)?;
    }
    Ok(())
}
pub(crate) fn scope(context: &wire::ComputerContext) -> ComputerRequest<'_> {
    ComputerRequest {
        computer_id: &context.computer_id,
        request_id: &context.request_id,
    }
}
pub(crate) fn validate_call(
    identity: Option<&wire::CallIdentity>,
    context: &wire::ComputerContext,
) -> Result<(), Status> {
    let identity = identity.ok_or_else(|| Status::invalid_argument("Call identity is required"))?;
    if identity.request_id != context.request_id
        || identity.call_id.is_empty()
        || identity.call_id.len() > 128
        || identity.connection_id.is_empty()
    {
        return Err(Status::invalid_argument("Call identity does not match"));
    }
    Ok(())
}
fn resource_matches(resource: &wire::ManagedResource, saved: &ManagedResource) -> bool {
    crate::native_resources::decode(resource).is_ok_and(|decoded| {
        saved.absence_confirmed_at.is_some() == decoded.absence_confirmed
            && saved.native() == &decoded.native
            && saved.identity() == decoded.identity.as_ref()
            && saved.execution_id == decoded.execution_id
    })
}
pub(crate) fn validate_provider(
    command: &wire::ProviderCommand,
    snapshot: &DeliverySnapshot,
) -> Result<(), Status> {
    let computer = &snapshot.computer;
    let context = command
        .computer
        .as_ref()
        .ok_or_else(|| Status::invalid_argument("Computer context is required"))?;
    let invalid = || {
        Status::failed_precondition(
            "Provider command does not match current intent or reserved resources",
        )
    };
    let check = |resources: &[wire::ManagedResource]| -> Result<(), Status> {
        if resources.is_empty() || resources.len() > 32 {
            return Err(invalid());
        }
        for (i, resource) in resources.iter().enumerate() {
            if !snapshot
                .resources
                .iter()
                .any(|saved| resource_matches(resource, saved))
                || resources[..i].contains(resource)
            {
                return Err(invalid());
            }
        }
        Ok(())
    };
    match command.operation.as_ref().ok_or_else(invalid)? {
        wire::provider_command::Operation::Observe(observe) => check(&observe.resources),
        wire::provider_command::Operation::Prepare(prepare) => {
            let execution = snapshot.execution.as_ref().ok_or_else(invalid)?;
            if computer.desired_state != "RUNNING"
                || computer.current_phase != "STARTING"
                || computer.current_execution_id.as_deref() != Some(&context.execution_id)
                || execution.retired_at.is_some()
                || execution.initial_allocation != prepare.initial_allocation
                || (computer.current_action == "RESTART"
                    && computer.expected_old_execution_id.as_deref() == Some(&context.execution_id))
            {
                return Err(invalid());
            }
            let resources = match snapshot.substrate.kind() {
                chapchoo_domain::ProviderKind::Kubernetes if prepare.docker.is_none() => vec![
                    prepare.saved_pvc.clone().ok_or_else(invalid)?,
                    prepare.bootstrap_secret.clone().ok_or_else(invalid)?,
                    prepare.pod.clone().ok_or_else(invalid)?,
                ],
                chapchoo_domain::ProviderKind::Docker
                    if prepare.saved_pvc.is_none()
                        && prepare.bootstrap_secret.is_none()
                        && prepare.pod.is_none() =>
                {
                    let docker = prepare.docker.as_ref().ok_or_else(invalid)?;
                    vec![
                        docker.saved_volume.clone().ok_or_else(invalid)?,
                        docker.container.clone().ok_or_else(invalid)?,
                    ]
                }
                _ => return Err(invalid()),
            };
            let expected = match snapshot.substrate.kind() {
                chapchoo_domain::ProviderKind::Kubernetes => vec![
                    wire::ResourceKind::Pvc,
                    wire::ResourceKind::BootstrapSecret,
                    wire::ResourceKind::Pod,
                ],
                chapchoo_domain::ProviderKind::Docker => vec![
                    wire::ResourceKind::DockerVolume,
                    wire::ResourceKind::DockerContainer,
                ],
            };
            if resources
                .iter()
                .zip(expected)
                .any(|(resource, kind)| resource.kind != kind as i32)
                || resources[0].execution_id.is_some()
                || resources[1..]
                    .iter()
                    .any(|r| r.execution_id.as_deref() != Some(&context.execution_id))
                || (!prepare.initial_allocation
                    && crate::native_resources::decode(&resources[0])?
                        .identity
                        .is_none())
            {
                return Err(invalid());
            }
            check(&resources)
        }
        wire::provider_command::Operation::Retire(retire) => {
            if !matches!(computer.desired_state.as_str(), "STOPPED" | "DELETING")
                && !(computer.desired_state == "RUNNING"
                    && computer.current_phase == "UNCERTAIN"
                    && retire.require_terminal)
                && !(computer.current_action == "RESTART"
                    && computer.expected_old_execution_id.as_deref() == Some(&context.execution_id))
            {
                return Err(invalid());
            }
            let (resource, expected_kind) = match snapshot.substrate.kind() {
                chapchoo_domain::ProviderKind::Kubernetes
                    if retire.docker_container.is_none() && retire.saved_volume.is_none() =>
                {
                    (
                        retire.pod.as_ref().ok_or_else(invalid)?,
                        wire::ResourceKind::Pod,
                    )
                }
                chapchoo_domain::ProviderKind::Docker
                    if retire.pod.is_none() && computer.current_phase == "UNCERTAIN" =>
                {
                    let saved = retire.saved_volume.as_ref().ok_or_else(invalid)?;
                    if saved.kind != wire::ResourceKind::DockerVolume as i32
                        || saved.execution_id.is_some()
                    {
                        return Err(invalid());
                    }
                    check(std::slice::from_ref(saved))?;
                    (
                        retire.docker_container.as_ref().ok_or_else(invalid)?,
                        wire::ResourceKind::DockerContainer,
                    )
                }
                _ => return Err(invalid()),
            };
            if resource.kind != expected_kind as i32
                || resource.execution_id.as_deref() != Some(&context.execution_id)
            {
                return Err(invalid());
            }
            check(std::slice::from_ref(resource))
        }
        wire::provider_command::Operation::Cleanup(cleanup) => {
            if computer.desired_state != "DELETING" {
                return Err(invalid());
            }
            check(&cleanup.resources)
        }
    }
}
fn deadline(remaining: u32) -> Result<Instant, Status> {
    if remaining == 0 {
        return Err(Status::deadline_exceeded("Peer deadline expired"));
    }
    Ok(Instant::now() + Duration::from_millis(u64::from(remaining)).min(MAX_BUDGET))
}
#[tonic::async_trait]
impl wire::peer_relay_server::PeerRelay for PeerServices {
    async fn converge_provider(
        &self,
        request: Request<wire::ProviderConvergence>,
    ) -> Result<Response<wire::ProviderInvocationResult>, Status> {
        let _permit = self
            .pending
            .try_acquire()
            .map_err(|_| Status::resource_exhausted("Peer pending capacity is exhausted"))?;
        let value = request.into_inner();
        let deadline = deadline(value.remaining_ms)?;
        let context = value
            .computer
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("Computer context is required"))?;
        if !matches!(
            value.authority.as_ref().and_then(|a| a.context.as_ref()),
            Some(wire::peer_authority::Context::Reconciliation(_))
        ) {
            return Err(Status::permission_denied(
                "Current reconciliation authority is required",
            ));
        }
        let snapshot = self.authorize(value.authority.as_ref(), context).await?;
        if snapshot.computer.provider_id != value.provider_id {
            return Err(Status::permission_denied("Provider target does not match"));
        }
        let owner = value
            .owner
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("Expected owner is required"))?;
        tokio::time::timeout_at(
            deadline,
            self.streams
                .converge_provider(&value.provider_id, owner, context, deadline),
        )
        .await
        .map_err(|_| Status::deadline_exceeded("Reconciliation deadline expired"))?
        .map(Response::new)
    }
    async fn invoke_provider(
        &self,
        request: Request<wire::ProviderInvocation>,
    ) -> Result<Response<wire::ProviderInvocationResult>, Status> {
        let _permit = self
            .pending
            .try_acquire()
            .map_err(|_| Status::resource_exhausted("Peer pending capacity is exhausted"))?;
        let invocation = request.into_inner();
        let deadline = deadline(invocation.command.as_ref().map_or(0, |c| c.remaining_ms))?;
        tokio::time::timeout_at(deadline, self.provider(invocation, deadline))
            .await
            .map_err(|_| Status::deadline_exceeded("Peer deadline expired"))?
            .map(Response::new)
    }
    async fn invoke_connector(
        &self,
        request: Request<wire::ConnectorInvocation>,
    ) -> Result<Response<wire::ConnectorInvocationResult>, Status> {
        let _permit = self
            .pending
            .try_acquire()
            .map_err(|_| Status::resource_exhausted("Peer pending capacity is exhausted"))?;
        let invocation = request.into_inner();
        let deadline = deadline(invocation.command.as_ref().map_or(0, |c| c.remaining_ms))?;
        tokio::time::timeout_at(deadline, self.connector(invocation, deadline))
            .await
            .map_err(|_| Status::deadline_exceeded("Peer deadline expired"))?
            .map(Response::new)
    }
}
