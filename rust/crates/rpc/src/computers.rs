//! Owner-scoped Client operations and redacted read-only installation inventory.
use crate::{ingress as input, mapping::timestamp, status::status};
use chapchoo_domain as domain;
use chapchoo_protocol::computers as wire;
use chapchoo_service::{
    AcceptanceTime, AgentSummary, ComputerSummary, ComputersService, CreationOutcome,
    LifecycleRequest, ResourceCursor,
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tonic::{Request, Response, Status};

#[derive(Debug, Clone)]
pub struct ComputerServices {
    service: Arc<ComputersService>,
    admission: Arc<AtomicBool>,
}
impl ComputerServices {
    pub fn new(service: Arc<ComputersService>) -> Self {
        Self {
            service,
            admission: Arc::new(AtomicBool::new(true)),
        }
    }
    pub fn with_admission(mut self, admission: Arc<AtomicBool>) -> Self {
        self.admission = admission;
        self
    }
    fn admit(&self) -> Result<(), Status> {
        if self.admission.load(Ordering::Acquire) {
            Ok(())
        } else {
            Err(Status::unavailable("Service is not admitting new requests"))
        }
    }
    pub fn agents_server(&self) -> wire::agents_server::AgentsServer<AgentsAdapter> {
        wire::agents_server::AgentsServer::new(AgentsAdapter(self.clone()))
            .max_decoding_message_size(64 * 1024)
            .max_encoding_message_size(64 * 1024)
    }
    pub fn computers_server(&self) -> wire::computers_server::ComputersServer<ComputersAdapter> {
        wire::computers_server::ComputersServer::new(ComputersAdapter(self.clone()))
            .max_decoding_message_size(64 * 1024)
            .max_encoding_message_size(64 * 1024)
    }
    pub fn catalog_server(
        &self,
    ) -> wire::provider_catalog_server::ProviderCatalogServer<CatalogAdapter> {
        wire::provider_catalog_server::ProviderCatalogServer::new(CatalogAdapter(self.clone()))
            .max_decoding_message_size(64 * 1024)
            .max_encoding_message_size(64 * 1024)
    }
    pub fn inventory_server(
        &self,
    ) -> wire::provider_inventory_server::ProviderInventoryServer<InventoryAdapter> {
        wire::provider_inventory_server::ProviderInventoryServer::new(InventoryAdapter(
            self.clone(),
        ))
        .max_decoding_message_size(64 * 1024)
        .max_encoding_message_size(64 * 1024)
    }
    async fn computer(&self, value: ComputerSummary) -> Result<wire::Computer, Status> {
        let connector = if self
            .service
            .resolve_connector_route(&value.id)
            .await
            .map_err(status)?
            .is_some()
        {
            wire::Connectivity::Connected
        } else {
            wire::Connectivity::Disconnected
        };
        let actions = [
            domain::LifecycleAction::Start,
            domain::LifecycleAction::Stop,
            domain::LifecycleAction::Restart,
        ]
        .into_iter()
        .map(|operation| {
            let availability = value.action_availability(operation);
            wire::ActionAvailability {
                action: action(operation) as i32,
                available: availability == domain::ActionAvailability::Available,
                blocked_reason: match availability {
                    domain::ActionAvailability::Available => None,
                    domain::ActionAvailability::Blocked(reason) => {
                        Some(block_reason(reason) as i32)
                    }
                },
            }
        })
        .chain(std::iter::once(wire::ActionAvailability {
            action: wire::Action::Delete as i32,
            available: value.desired != domain::DesiredState::Deleting,
            blocked_reason: (value.desired == domain::DesiredState::Deleting)
                .then_some(wire::BlockReason::Deleting as i32),
        }))
        .collect();
        Ok(wire::Computer {
            id: value.id,
            kind: kind(value.kind) as i32,
            provider_id: value.provider_id,
            desired: desired(value.desired) as i32,
            request_id: value.current_request_id,
            action: action(value.progress.action) as i32,
            phase: phase(value.progress.phase) as i32,
            power: power(value.power) as i32,
            last_known_power: power(value.last_known_power) as i32,
            management: connectivity(value.management) as i32,
            connector: connector as i32,
            power_observed_at: value.power_observed_at.map(timestamp).transpose()?,
            saved_environment: saved(value.saved_environment) as i32,
            actions,
            preparation_retry_available: value.preparation_retry_available,
            force_remove_available: value.desired == domain::DesiredState::Deleting,
            start_delayed: value
                .progress
                .start_delayed_after(input::now()?, self.service.policy().start_delayed_ms),
        })
    }
    async fn agent(&self, value: AgentSummary) -> Result<wire::Agent, Status> {
        Ok(wire::Agent {
            id: value.id,
            name: value.name,
            computer: Some(self.computer(value.computer).await?),
        })
    }
}
#[derive(Debug, Clone)]
pub struct AgentsAdapter(ComputerServices);
#[derive(Debug, Clone)]
pub struct ComputersAdapter(ComputerServices);
#[derive(Debug, Clone)]
pub struct CatalogAdapter(ComputerServices);
#[derive(Debug, Clone)]
pub struct InventoryAdapter(ComputerServices);
fn limit(value: u32) -> Result<i64, Status> {
    if value == 0 {
        Ok(20)
    } else if value <= 20 {
        Ok(i64::from(value))
    } else {
        Err(Status::invalid_argument(
            "Page limit must be between one and twenty",
        ))
    }
}
fn id(value: &str) -> Result<(), Status> {
    if !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b':'))
    {
        Ok(())
    } else {
        Err(Status::invalid_argument("Object identity is invalid"))
    }
}
fn page(value: &wire::Page) -> Result<i64, Status> {
    if let Some(after) = &value.after {
        id(after)?;
    }
    limit(value.limit)
}
fn next(items: usize, limit: i64, last: Option<&str>) -> Option<String> {
    (items == limit as usize)
        .then(|| last.map(str::to_owned))
        .flatten()
}
#[tonic::async_trait]
impl wire::agents_server::Agents for AgentsAdapter {
    async fn create(
        &self,
        request: Request<wire::CreateAgentRequest>,
    ) -> Result<Response<wire::CreateAgentResult>, Status> {
        self.0.admit()?;
        let actor = input::actor(&request)?;
        let value = request.into_inner();
        id(&value.request_id)?;
        id(&value.provider_id)?;
        let outcome = match self
            .0
            .service
            .create_cloud_agent(
                &actor,
                &value.request_id,
                &value.provider_id,
                &value.name,
                AcceptanceTime::System,
            )
            .await
            .map_err(status)?
        {
            CreationOutcome::Present(agent) => {
                wire::create_agent_result::Outcome::Agent(self.0.agent(agent).await?)
            }
            CreationOutcome::Removed { assigned_agent_id } => {
                wire::create_agent_result::Outcome::Removed(wire::RemovedCreation {
                    assigned_agent_id,
                })
            }
        };
        Ok(Response::new(wire::CreateAgentResult {
            outcome: Some(outcome),
        }))
    }
    async fn list(
        &self,
        request: Request<wire::Page>,
    ) -> Result<Response<wire::AgentList>, Status> {
        self.0.admit()?;
        let actor = input::actor(&request)?;
        let value = request.into_inner();
        let limit = page(&value)?;
        let items = self
            .0
            .service
            .agents(
                &actor,
                value.after.as_deref(),
                limit,
                AcceptanceTime::System,
            )
            .await
            .map_err(status)?;
        let next_after = next(items.len(), limit, items.last().map(|a| a.id.as_str()));
        let mut result = Vec::with_capacity(items.len());
        for item in items {
            result.push(self.0.agent(item).await?);
        }
        Ok(Response::new(wire::AgentList {
            items: result,
            next_after,
        }))
    }
    async fn get(
        &self,
        request: Request<wire::AgentRequest>,
    ) -> Result<Response<wire::Agent>, Status> {
        self.0.admit()?;
        let actor = input::actor(&request)?;
        let value = request.into_inner();
        id(&value.agent_id)?;
        let agent = self
            .0
            .service
            .agent(&actor, &value.agent_id, AcceptanceTime::System)
            .await
            .map_err(status)?;
        Ok(Response::new(self.0.agent(agent).await?))
    }
    async fn force_remove(
        &self,
        request: Request<wire::ForceRemoveRequest>,
    ) -> Result<Response<()>, Status> {
        self.0.admit()?;
        let actor = input::actor(&request)?;
        let value = request.into_inner();
        id(&value.agent_id)?;
        id(&value.deletion_request_id)?;
        self.0
            .service
            .force_remove_agent(
                &actor,
                &value.agent_id,
                &value.deletion_request_id,
                AcceptanceTime::System,
            )
            .await
            .map_err(status)?;
        Ok(Response::new(()))
    }
}
#[tonic::async_trait]
impl wire::computers_server::Computers for ComputersAdapter {
    async fn get(
        &self,
        request: Request<wire::AgentRequest>,
    ) -> Result<Response<wire::Computer>, Status> {
        self.0.admit()?;
        let actor = input::actor(&request)?;
        let value = request.into_inner();
        id(&value.agent_id)?;
        let agent = self
            .0
            .service
            .agent(&actor, &value.agent_id, AcceptanceTime::System)
            .await
            .map_err(status)?;
        Ok(Response::new(self.0.computer(agent.computer).await?))
    }
    async fn retry_preparation(
        &self,
        request: Request<wire::PreparationRetryRequest>,
    ) -> Result<Response<wire::Agent>, Status> {
        self.0.admit()?;
        let actor = input::actor(&request)?;
        let value = request.into_inner();
        id(&value.agent_id)?;
        id(&value.request_id)?;
        id(&value.expected_request_id)?;
        let agent = self
            .0
            .service
            .retry_preparation(
                &actor,
                &value.agent_id,
                &value.request_id,
                &value.expected_request_id,
                AcceptanceTime::System,
            )
            .await
            .map_err(status)?;
        Ok(Response::new(self.0.agent(agent).await?))
    }
    async fn request_lifecycle(
        &self,
        request: Request<wire::LifecycleRequest>,
    ) -> Result<Response<wire::Agent>, Status> {
        self.0.admit()?;
        let actor = input::actor(&request)?;
        let value = request.into_inner();
        id(&value.agent_id)?;
        id(&value.request_id)?;
        id(&value.expected_request_id)?;
        let operation = match wire::Action::try_from(value.action) {
            Ok(wire::Action::Start) => domain::LifecycleAction::Start,
            Ok(wire::Action::Stop) => domain::LifecycleAction::Stop,
            Ok(wire::Action::Restart) => domain::LifecycleAction::Restart,
            Ok(wire::Action::Delete) => domain::LifecycleAction::Delete,
            _ => return Err(Status::invalid_argument("Lifecycle action is invalid")),
        };
        let agent = self
            .0
            .service
            .request_lifecycle(
                &actor,
                &value.agent_id,
                &LifecycleRequest {
                    request_id: value.request_id,
                    expected_request_id: value.expected_request_id,
                    action: operation,
                },
                AcceptanceTime::System,
            )
            .await
            .map_err(status)?;
        Ok(Response::new(self.0.agent(agent).await?))
    }
}
#[tonic::async_trait]
impl wire::provider_catalog_server::ProviderCatalog for CatalogAdapter {
    async fn list(
        &self,
        request: Request<wire::Page>,
    ) -> Result<Response<wire::ProviderChoices>, Status> {
        self.0.admit()?;
        let actor = input::actor(&request)?;
        let value = request.into_inner();
        let limit = page(&value)?;
        let items = self
            .0
            .service
            .available_providers(
                &actor,
                value.after.as_deref(),
                limit,
                AcceptanceTime::System,
            )
            .await
            .map_err(status)?;
        let next_after = next(items.len(), limit, items.last().map(|p| p.id.as_str()));
        Ok(Response::new(wire::ProviderChoices {
            items: items
                .into_iter()
                .map(|p| wire::ProviderChoice {
                    id: p.id,
                    name: p.name,
                })
                .collect(),
            next_after,
        }))
    }
}
#[tonic::async_trait]
impl wire::provider_inventory_server::ProviderInventory for InventoryAdapter {
    async fn list_providers(
        &self,
        request: Request<wire::Page>,
    ) -> Result<Response<wire::ProviderList>, Status> {
        self.0.admit()?;
        let actor = input::actor(&request)?;
        let value = request.into_inner();
        let limit = page(&value)?;
        let items = self
            .0
            .service
            .provider_inventory(
                &actor,
                value.after.as_deref(),
                limit,
                AcceptanceTime::System,
            )
            .await
            .map_err(status)?;
        let next_after = next(items.len(), limit, items.last().map(|p| p.id.as_str()));
        let mut result = Vec::with_capacity(items.len());
        for item in items {
            let connected = self
                .0
                .service
                .resolve_provider_route(&item.id)
                .await
                .map_err(status)?
                .is_some();
            result.push(wire::ProviderSummary {
                id: item.id,
                name: item.name,
                capabilities: Some(wire::ProviderCapabilities {
                    create: item.capabilities.create,
                    start: item.capabilities.start,
                    stop: item.capabilities.stop,
                    restart: item.capabilities.restart,
                }),
                management: if connected {
                    wire::Connectivity::Connected
                } else {
                    wire::Connectivity::Disconnected
                } as i32,
            });
        }
        Ok(Response::new(wire::ProviderList {
            items: result,
            next_after,
        }))
    }
    async fn list_computers(
        &self,
        request: Request<wire::InventoryRequest>,
    ) -> Result<Response<wire::ComputerInventory>, Status> {
        self.0.admit()?;
        let actor = input::actor(&request)?;
        let value = request.into_inner();
        id(&value.provider_id)?;
        let pagination = value.page.unwrap_or_default();
        let limit = page(&pagination)?;
        let items = self
            .0
            .service
            .computer_inventory(
                &actor,
                &value.provider_id,
                pagination.after.as_deref(),
                limit,
                AcceptanceTime::System,
            )
            .await
            .map_err(status)?;
        let next_after = next(
            items.len(),
            limit,
            items.last().map(|p| p.computer.id.as_str()),
        );
        let mut result = Vec::with_capacity(items.len());
        for item in items {
            let computer = self.0.computer(item.computer).await?;
            result.push(wire::InventoryComputer {
                id: computer.id,
                kind: computer.kind,
                provider_id: computer.provider_id,
                desired: computer.desired,
                action: computer.action,
                phase: computer.phase,
                power: computer.power,
                last_known_power: computer.last_known_power,
                management: computer.management,
                connector: computer.connector,
                saved_environment: computer.saved_environment,
                residual: item.residual,
                cleanup_completed_at: item.cleanup_completed_at.map(timestamp).transpose()?,
                power_observed_at: computer.power_observed_at,
            });
        }
        Ok(Response::new(wire::ComputerInventory {
            items: result,
            next_after,
        }))
    }
    async fn list_resources(
        &self,
        request: Request<wire::ResourcePage>,
    ) -> Result<Response<wire::ResourceInventory>, Status> {
        self.0.admit()?;
        let actor = input::actor(&request)?;
        let value = request.into_inner();
        id(&value.computer_id)?;
        let limit = limit(value.limit)?;
        let cursor = value.after.map(|c| ResourceCursor {
            kind: c.kind,
            namespace: c.namespace,
            name: c.name,
        });
        let items = self
            .0
            .service
            .resource_inventory(
                &actor,
                &value.computer_id,
                cursor.as_ref(),
                limit,
                AcceptanceTime::System,
            )
            .await
            .map_err(status)?;
        let next_after = if items.len() == limit as usize {
            items.last().map(|r| wire::ResourceCursor {
                kind: r.resource_kind.clone(),
                namespace: r.namespace.clone(),
                name: r.name.clone(),
            })
        } else {
            None
        };
        let items = items
            .into_iter()
            .map(|r| {
                Ok(wire::InventoryResource {
                    identity: Some(wire::ResourceCursor {
                        kind: r.resource_kind,
                        namespace: r.namespace,
                        name: r.name,
                    }),
                    uid: r.resource_uid,
                    execution_id: r.execution_id,
                    presence: r.presence,
                    observed_at: r.observed_at.map(timestamp).transpose()?,
                })
            })
            .collect::<Result<Vec<_>, Status>>()?;
        Ok(Response::new(wire::ResourceInventory { items, next_after }))
    }
}
fn kind(v: domain::ComputerKind) -> wire::ComputerKind {
    match v {
        domain::ComputerKind::Cloud => wire::ComputerKind::Cloud,
        domain::ComputerKind::Local => wire::ComputerKind::Local,
    }
}
fn desired(v: domain::DesiredState) -> wire::DesiredState {
    match v {
        domain::DesiredState::Running => wire::DesiredState::Running,
        domain::DesiredState::Stopped => wire::DesiredState::Stopped,
        domain::DesiredState::Deleting => wire::DesiredState::Deleting,
    }
}
fn power(v: domain::ObservedPower) -> wire::Power {
    match v {
        domain::ObservedPower::Running => wire::Power::Running,
        domain::ObservedPower::Stopped => wire::Power::Stopped,
        domain::ObservedPower::Unknown => wire::Power::Unknown,
    }
}
fn saved(v: domain::SavedEnvironment) -> wire::SavedEnvironment {
    match v {
        domain::SavedEnvironment::Verified => wire::SavedEnvironment::Verified,
        domain::SavedEnvironment::Missing => wire::SavedEnvironment::Missing,
        domain::SavedEnvironment::Unknown => wire::SavedEnvironment::Unknown,
    }
}
fn connectivity(v: domain::ConnectionStatus) -> wire::Connectivity {
    match v {
        domain::ConnectionStatus::Connected => wire::Connectivity::Connected,
        domain::ConnectionStatus::Disconnected => wire::Connectivity::Disconnected,
        domain::ConnectionStatus::Unknown => wire::Connectivity::Unknown,
    }
}
fn action(v: domain::LifecycleAction) -> wire::Action {
    match v {
        domain::LifecycleAction::Create => wire::Action::Create,
        domain::LifecycleAction::Start => wire::Action::Start,
        domain::LifecycleAction::Stop => wire::Action::Stop,
        domain::LifecycleAction::Restart => wire::Action::Restart,
        domain::LifecycleAction::Delete => wire::Action::Delete,
    }
}
fn phase(v: domain::LifecyclePhase) -> wire::Phase {
    match v {
        domain::LifecyclePhase::Accepted => wire::Phase::Accepted,
        domain::LifecyclePhase::Retiring => wire::Phase::Retiring,
        domain::LifecyclePhase::Starting => wire::Phase::Starting,
        domain::LifecyclePhase::Deleting => wire::Phase::Deleting,
        domain::LifecyclePhase::Completed => wire::Phase::Completed,
        domain::LifecyclePhase::Blocked => wire::Phase::Blocked,
        domain::LifecyclePhase::Uncertain => wire::Phase::Uncertain,
    }
}
fn block_reason(v: domain::ActionBlockReason) -> wire::BlockReason {
    match v {
        domain::ActionBlockReason::Unsupported => wire::BlockReason::Unsupported,
        domain::ActionBlockReason::Deleting => wire::BlockReason::Deleting,
        domain::ActionBlockReason::ConflictingLifecycle => wire::BlockReason::ConflictingLifecycle,
        domain::ActionBlockReason::ManagementUnavailable => {
            wire::BlockReason::ManagementUnavailable
        }
        domain::ActionBlockReason::PowerUnknown => wire::BlockReason::PowerUnknown,
        domain::ActionBlockReason::RequiresStopped => wire::BlockReason::RequiresStopped,
        domain::ActionBlockReason::RequiresRunning => wire::BlockReason::RequiresRunning,
        domain::ActionBlockReason::SavedEnvironmentUnverified => {
            wire::BlockReason::SavedEnvironmentUnverified
        }
    }
}
