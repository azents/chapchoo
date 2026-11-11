use crate::{AcceptanceTime, DomainReason, ServiceError};
use chapchoo_crypto::{BearerSecret, SecretDigest, VerifiedSystemOwner};
use chapchoo_domain::{LifecyclePhase, NativeScope, ProviderSubstrate};
use chapchoo_persistence::{
    AgentRemoval, AgentSummary, ComponentKeyDraft, ComponentKeyPolicy, ComponentKeyResult,
    ComponentRoute, ComputerRequest, ConnectionOwner, ConnectorIdentity, ConnectorSessionDraft,
    ConnectorSessionResult, ConnectorSource, ControlRegistration, CreateCloudAgent,
    CreationOutcome, DirectoryPolicy, ExecutionDraft, ExecutionReservation, ExecutionResources,
    LifecycleRequest, ManagedComputer, ManagedResource, ProviderCapabilities, ProviderChoice,
    ProviderCredentialResult, ProviderIdentity, ProviderOwner, ProviderSource, ResolvedRoute,
    ResourceReport, Store,
};
use std::fmt;
use url::Url;

/// Captured before local stream preparation. Only the service constructs this
/// admission context; a client cannot assert the expected owner or Connection ID.
pub struct PreparedComponentConnection {
    target: ConnectionTarget,
    expected: Option<ConnectionOwner>,
    owner: ConnectionOwner,
    credential: SecretDigest,
}
#[derive(Debug)]
enum ConnectionTarget {
    Provider(String),
    Connector(String),
}
impl PreparedComponentConnection {
    pub fn owner(&self) -> &ConnectionOwner {
        &self.owner
    }
}
impl fmt::Debug for PreparedComponentConnection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PreparedComponentConnection([REDACTED])")
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ComputerPolicy {
    pub directory: DirectoryPolicy,
    pub keys: ComponentKeyPolicy,
    pub provider_keys: ComponentKeyPolicy,
    pub heartbeat_ms: u32,
    pub start_delayed_ms: i64,
}
impl Default for ComputerPolicy {
    fn default() -> Self {
        Self {
            directory: DirectoryPolicy::default(),
            keys: ComponentKeyPolicy::default(),
            provider_keys: ComponentKeyPolicy {
                validity_ms: 900_000,
                proactive_after_ms: 300_000,
                overlap_ms: 300_000,
            },
            heartbeat_ms: 20_000,
            start_delayed_ms: 300_000,
        }
    }
}
impl ComputerPolicy {
    pub fn validate(self) -> Result<Self, ServiceError> {
        if self.heartbeat_ms == 0
            || i64::from(self.heartbeat_ms) >= self.directory.control_validity_ms
            || i64::from(self.heartbeat_ms) >= self.directory.component_validity_ms
            || self.directory.control_validity_ms > i64::from(u32::MAX)
            || self.directory.component_validity_ms > i64::from(u32::MAX)
            || self.keys.validity_ms <= 0
            || self.keys.proactive_after_ms <= 0
            || self.keys.proactive_after_ms >= self.keys.validity_ms
            || !(0..=300_000).contains(&self.keys.overlap_ms)
            || self.provider_keys.validity_ms != 900_000
            || self.provider_keys.proactive_after_ms != 300_000
            || !(0..=300_000).contains(&self.provider_keys.overlap_ms)
            || self.start_delayed_ms <= 0
        {
            return Err(ServiceError::domain(DomainReason::InvalidInput));
        }
        Ok(self)
    }
}
/// Transport-independent application boundary. Authentication adapters supply verified
/// bootstrap grants; requests cannot assert an owner identity.
#[derive(Clone)]
pub struct ComputersService {
    store: Store,
    policy: ComputerPolicy,
}
impl fmt::Debug for ComputersService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ComputersService([REDACTED])")
    }
}
impl ComputersService {
    pub fn new(store: Store) -> Self {
        Self {
            store,
            policy: ComputerPolicy::default(),
        }
    }
    pub fn with_policy(mut self, policy: ComputerPolicy) -> Result<Self, ServiceError> {
        self.policy = policy.validate()?;
        Ok(self)
    }
    pub fn policy(&self) -> ComputerPolicy {
        self.policy
    }
    pub async fn register_control(
        &self,
        endpoint: &str,
    ) -> Result<ControlRegistration, ServiceError> {
        let parsed =
            Url::parse(endpoint).map_err(|_| ServiceError::domain(DomainReason::InvalidInput))?;
        if parsed.scheme() != "http"
            || parsed.host_str().is_none()
            || parsed.username() != ""
            || parsed.password().is_some()
            || parsed.query().is_some()
            || parsed.fragment().is_some()
            || parsed.path() != "/"
            || parsed.host_str().is_some_and(|host| {
                host.trim_matches(['[', ']'])
                    .parse::<std::net::IpAddr>()
                    .is_ok_and(|ip| ip.is_unspecified())
            })
        {
            return Err(ServiceError::domain(DomainReason::InvalidInput));
        }
        Ok(self
            .store
            .register_control(parsed.as_str(), self.policy.directory)
            .await?)
    }
    pub async fn renew_control(&self, id: &str) -> Result<ControlRegistration, ServiceError> {
        Ok(self.store.renew_control(id, self.policy.directory).await?)
    }
    pub async fn drain_control(&self, id: &str) -> Result<(), ServiceError> {
        Ok(self.store.drain_control(id).await?)
    }
    pub async fn withdraw_drained_control_connections(
        &self,
        id: &str,
    ) -> Result<usize, ServiceError> {
        Ok(self.store.withdraw_drained_control_connections(id).await?)
    }
    pub async fn prepare_provider_connection(
        &self,
        provider_id: &str,
        credential: &SecretDigest,
        control_id: &str,
    ) -> Result<PreparedComponentConnection, ServiceError> {
        let prepared = self
            .store
            .prepare_provider_connection(provider_id, credential, control_id)
            .await?;
        Ok(PreparedComponentConnection {
            target: ConnectionTarget::Provider(provider_id.into()),
            expected: prepared.expected,
            owner: prepared.owner,
            credential: credential.clone(),
        })
    }
    pub async fn prepare_connector_connection(
        &self,
        session_id: &str,
        credential: &SecretDigest,
        control_id: &str,
    ) -> Result<PreparedComponentConnection, ServiceError> {
        let prepared = self
            .store
            .prepare_connector_connection(session_id, credential, control_id)
            .await?;
        Ok(PreparedComponentConnection {
            target: ConnectionTarget::Connector(session_id.into()),
            expected: prepared.expected,
            owner: prepared.owner,
            credential: credential.clone(),
        })
    }
    /// Call only after the bounded local stream/registry is ready. A losing
    /// attempt is explicit conflict, never a loop that steals the winner.
    pub async fn admit_component_connection(
        &self,
        prepared: &PreparedComponentConnection,
    ) -> Result<ComponentRoute, ServiceError> {
        let policy = self.policy.directory;
        Ok(match &prepared.target {
            ConnectionTarget::Provider(id) => {
                self.store
                    .admit_provider_connection(
                        id,
                        &prepared.credential,
                        prepared.expected.as_ref(),
                        &prepared.owner,
                        policy,
                    )
                    .await?
            }
            ConnectionTarget::Connector(id) => {
                self.store
                    .admit_connector_connection(
                        id,
                        &prepared.credential,
                        prepared.expected.as_ref(),
                        &prepared.owner,
                        policy,
                    )
                    .await?
            }
        })
    }
    pub async fn renew_component_connection(
        &self,
        prepared: &PreparedComponentConnection,
    ) -> Result<ComponentRoute, ServiceError> {
        let policy = self.policy.directory;
        Ok(match &prepared.target {
            ConnectionTarget::Provider(id) => {
                self.store
                    .renew_provider_connection(id, &prepared.credential, &prepared.owner, policy)
                    .await?
            }
            ConnectionTarget::Connector(id) => {
                self.store
                    .renew_connector_connection(id, &prepared.credential, &prepared.owner, policy)
                    .await?
            }
        })
    }
    pub async fn verify_component_connection(
        &self,
        prepared: &PreparedComponentConnection,
    ) -> Result<(), ServiceError> {
        match &prepared.target {
            ConnectionTarget::Provider(id) => {
                self.store
                    .verify_provider_connection(&ProviderSource {
                        provider_id: id,
                        control_id: &prepared.owner.control_id,
                        connection_id: &prepared.owner.connection_id,
                        credential: &prepared.credential,
                    })
                    .await?
            }
            ConnectionTarget::Connector(id) => {
                self.store
                    .verify_connector_connection(&ConnectorSource {
                        session_id: id,
                        control_id: &prepared.owner.control_id,
                        connection_id: &prepared.owner.connection_id,
                        credential: &prepared.credential,
                    })
                    .await?;
            }
        }
        Ok(())
    }
    pub async fn close_component_connection(
        &self,
        prepared: &PreparedComponentConnection,
    ) -> Result<bool, ServiceError> {
        Ok(match &prepared.target {
            ConnectionTarget::Provider(id) => {
                self.store
                    .close_provider_connection(id, &prepared.credential, &prepared.owner)
                    .await?
            }
            ConnectionTarget::Connector(id) => {
                self.store
                    .close_connector_connection(id, &prepared.credential, &prepared.owner)
                    .await?
            }
        })
    }
    pub async fn resolve_provider_route(
        &self,
        provider_id: &str,
    ) -> Result<Option<ResolvedRoute>, ServiceError> {
        Ok(self.store.resolve_provider_route(provider_id).await?)
    }
    pub async fn verify_provider_source(
        &self,
        source: &ProviderSource<'_>,
    ) -> Result<(), ServiceError> {
        Ok(self.store.verify_provider_connection(source).await?)
    }
    pub async fn verify_connector_source(
        &self,
        source: &ConnectorSource<'_>,
    ) -> Result<ConnectorIdentity, ServiceError> {
        Ok(self.store.verify_connector_connection(source).await?)
    }
    pub async fn resolve_connector_route(
        &self,
        computer_id: &str,
    ) -> Result<Option<ResolvedRoute>, ServiceError> {
        Ok(self.store.resolve_connector_route(computer_id).await?)
    }
    pub async fn running_computer_ids(
        &self,
        after: Option<&str>,
        limit: u32,
    ) -> Result<Vec<String>, ServiceError> {
        Ok(self.store.running_computer_ids(after, limit).await?)
    }

    /// Trusted peer handlers supply either a current user session or a separately
    /// verified live reconciliation owner, never an asserted user ID.
    pub async fn delivery_snapshot(
        &self,
        actor: Option<&SecretDigest>,
        scope: ComputerRequest<'_>,
        execution: Option<&str>,
    ) -> Result<chapchoo_persistence::DeliverySnapshot, ServiceError> {
        Ok(self
            .store
            .delivery_snapshot(actor, scope, execution)
            .await?)
    }

    pub async fn verify_delivery_bootstrap(
        &self,
        scope: ComputerRequest<'_>,
        execution: &str,
        session: &str,
        key: &SecretDigest,
    ) -> Result<(), ServiceError> {
        Ok(self
            .store
            .verify_delivery_bootstrap(scope, execution, session, key)
            .await?)
    }

    pub async fn provider_delivery_snapshot(
        &self,
        source: &ProviderSource<'_>,
        scope: ComputerRequest<'_>,
        execution: Option<&str>,
    ) -> Result<chapchoo_persistence::DeliverySnapshot, ServiceError> {
        Ok(self
            .store
            .provider_delivery_snapshot(source, scope, execution)
            .await?)
    }

    pub async fn provider_inventory(
        &self,
        actor: &SecretDigest,
        after: Option<&str>,
        limit: i64,
        clock: AcceptanceTime,
    ) -> Result<Vec<chapchoo_persistence::ProviderInventorySummary>, ServiceError> {
        Ok(self
            .store
            .provider_inventory(actor, after, limit, clock.now()?)
            .await?)
    }
    pub async fn computer_inventory(
        &self,
        actor: &SecretDigest,
        provider: &str,
        after: Option<&str>,
        limit: i64,
        clock: AcceptanceTime,
    ) -> Result<Vec<chapchoo_persistence::ComputerInventorySummary>, ServiceError> {
        Ok(self
            .store
            .computer_inventory(actor, provider, after, limit, clock.now()?)
            .await?)
    }
    pub async fn resource_inventory(
        &self,
        actor: &SecretDigest,
        computer: &str,
        after: Option<&chapchoo_persistence::ResourceCursor>,
        limit: i64,
        clock: AcceptanceTime,
    ) -> Result<Vec<ManagedResource>, ServiceError> {
        Ok(self
            .store
            .resource_inventory(actor, computer, after, limit, clock.now()?)
            .await?)
    }

    pub async fn exchange_bootstrap_provider(
        &self,
        proof: &VerifiedSystemOwner,
        name: &str,
        capabilities: ProviderCapabilities,
        substrate: &ProviderSubstrate,
        request_id: &str,
        clock: AcceptanceTime,
    ) -> Result<ProviderCredentialResult, ServiceError> {
        let draft = key_draft(request_id)?;
        Ok(self
            .store
            .rotate_provider_key(
                ProviderOwner::Bootstrap {
                    proof,
                    name,
                    capabilities,
                    substrate,
                },
                proof.provider_id(),
                &draft,
                self.policy.provider_keys,
                clock.now()?,
            )
            .await?)
    }
    pub async fn exchange_registered_provider(
        &self,
        provider_id: &str,
        credential: &SecretDigest,
        substrate: &ProviderSubstrate,
        request_id: &str,
        clock: AcceptanceTime,
    ) -> Result<ProviderCredentialResult, ServiceError> {
        let draft = key_draft(request_id)?;
        Ok(self
            .store
            .rotate_provider_key(
                ProviderOwner::Registration {
                    credential,
                    substrate,
                },
                provider_id,
                &draft,
                self.policy.provider_keys,
                clock.now()?,
            )
            .await?)
    }

    pub async fn rotate_user_provider_key(
        &self,
        actor: &SecretDigest,
        provider_id: &str,
        request_id: &str,
        clock: AcceptanceTime,
    ) -> Result<ProviderCredentialResult, ServiceError> {
        let draft = key_draft(request_id)?;
        Ok(self
            .store
            .rotate_provider_key(
                ProviderOwner::User(actor),
                provider_id,
                &draft,
                self.policy.provider_keys,
                clock.now()?,
            )
            .await?)
    }
    pub async fn revoke_user_provider_key(
        &self,
        actor: &SecretDigest,
        provider_id: &str,
        request_id: &str,
        clock: AcceptanceTime,
    ) -> Result<(), ServiceError> {
        Ok(self
            .store
            .revoke_provider_key(
                ProviderOwner::User(actor),
                provider_id,
                request_id,
                clock.now()?,
            )
            .await?)
    }
    pub async fn provider_identity(
        &self,
        provider_id: &str,
        credential: &SecretDigest,
        clock: AcceptanceTime,
    ) -> Result<ProviderIdentity, ServiceError> {
        Ok(self
            .store
            .provider_key_identity(provider_id, credential, clock.now()?)
            .await?)
    }
    pub async fn available_providers(
        &self,
        actor: &SecretDigest,
        after: Option<&str>,
        limit: i64,
        clock: AcceptanceTime,
    ) -> Result<Vec<ProviderChoice>, ServiceError> {
        Ok(self
            .store
            .available_providers(actor, after, limit, clock.now()?)
            .await?)
    }
    pub async fn create_cloud_agent(
        &self,
        actor: &SecretDigest,
        draft: &CreateCloudAgent,
        clock: AcceptanceTime,
    ) -> Result<CreationOutcome, ServiceError> {
        Ok(self
            .store
            .create_cloud_agent(actor, draft, clock.now()?)
            .await?)
    }
    pub async fn agent_handle_available(
        &self,
        actor: &SecretDigest,
        team_id: &str,
        candidate: &str,
        clock: AcceptanceTime,
    ) -> Result<bool, ServiceError> {
        Ok(self
            .store
            .agent_handle_available(actor, team_id, candidate, clock.now()?)
            .await?)
    }

    pub async fn agent_in_team(
        &self,
        actor: &SecretDigest,
        team_id: &str,
        handle: &str,
        clock: AcceptanceTime,
    ) -> Result<AgentSummary, ServiceError> {
        Ok(self
            .store
            .agent_in_team(actor, team_id, handle, clock.now()?)
            .await?)
    }
    pub async fn update_agent_profile(
        &self,
        actor: &SecretDigest,
        agent_id: &str,
        profile: &crate::UpdateAgentProfile,
        clock: AcceptanceTime,
    ) -> Result<AgentSummary, ServiceError> {
        Ok(self
            .store
            .update_agent_profile(actor, agent_id, profile, clock.now()?)
            .await?)
    }
    pub async fn agent_creation_result(
        &self,
        actor: &SecretDigest,
        request_id: &str,
        clock: AcceptanceTime,
    ) -> Result<Option<crate::AgentCreationResult>, ServiceError> {
        Ok(self
            .store
            .agent_creation_result(actor, request_id, clock.now()?)
            .await?)
    }
    pub async fn agent_profile_images(
        &self,
        actor: &SecretDigest,
        after: Option<&str>,
        limit: i64,
        clock: AcceptanceTime,
    ) -> Result<Vec<crate::AgentProfileImage>, ServiceError> {
        Ok(self
            .store
            .agent_profile_images(actor, after, limit, clock.now()?)
            .await?)
    }

    pub async fn agent(
        &self,
        actor: &SecretDigest,
        agent_id: &str,
        clock: AcceptanceTime,
    ) -> Result<AgentSummary, ServiceError> {
        Ok(self.store.agent(actor, agent_id, clock.now()?).await?)
    }
    pub async fn agents(
        &self,
        actor: &SecretDigest,
        team_id: &str,
        after: Option<&str>,
        limit: i64,
        clock: AcceptanceTime,
    ) -> Result<Vec<AgentSummary>, ServiceError> {
        Ok(self
            .store
            .agents(actor, team_id, after, limit, clock.now()?)
            .await?)
    }
    pub async fn retry_preparation(
        &self,
        actor: &SecretDigest,
        agent_id: &str,
        request_id: &str,
        expected_request_id: &str,
        clock: AcceptanceTime,
    ) -> Result<AgentSummary, ServiceError> {
        Ok(self
            .store
            .retry_preparation(
                actor,
                agent_id,
                request_id,
                expected_request_id,
                clock.now()?,
            )
            .await?)
    }
    pub async fn request_lifecycle(
        &self,
        actor: &SecretDigest,
        agent_id: &str,
        request: &LifecycleRequest,
        clock: AcceptanceTime,
    ) -> Result<AgentSummary, ServiceError> {
        Ok(self
            .store
            .request_lifecycle(actor, agent_id, request, clock.now()?)
            .await?)
    }
    pub async fn force_remove_agent(
        &self,
        actor: &SecretDigest,
        agent_id: &str,
        deletion_request_id: &str,
        clock: AcceptanceTime,
    ) -> Result<AgentRemoval, ServiceError> {
        Ok(self
            .store
            .force_remove_agent(actor, agent_id, deletion_request_id, clock.now()?)
            .await?)
    }
    pub async fn issue_connector_session(
        &self,
        provider_id: &str,
        credential: &SecretDigest,
        execution_id: &str,
        request_id: &str,
        clock: AcceptanceTime,
    ) -> Result<ConnectorSessionResult, ServiceError> {
        let draft = ConnectorSessionDraft {
            request_id: request_id.into(),
            initial_key: secret()?,
        };
        Ok(self
            .store
            .issue_connector_session(provider_id, credential, execution_id, &draft, clock.now()?)
            .await?)
    }
    pub async fn issue_connector_session_from_source(
        &self,
        source: &ProviderSource<'_>,
        execution_id: &str,
        request_id: &str,
        clock: AcceptanceTime,
    ) -> Result<ConnectorSessionResult, ServiceError> {
        let draft = ConnectorSessionDraft {
            request_id: request_id.into(),
            initial_key: secret()?,
        };
        Ok(self
            .store
            .issue_connector_session_from_source(source, execution_id, &draft, clock.now()?)
            .await?)
    }
    pub async fn revoke_connector_session_from_source(
        &self,
        source: &ProviderSource<'_>,
        session_id: &str,
        request_id: &str,
        clock: AcceptanceTime,
    ) -> Result<(), ServiceError> {
        Ok(self
            .store
            .revoke_connector_session_from_source(source, session_id, request_id, clock.now()?)
            .await?)
    }
    pub async fn exchange_connector_key(
        &self,
        session_id: &str,
        initial_key: &SecretDigest,
        request_id: &str,
        clock: AcceptanceTime,
    ) -> Result<ComponentKeyResult, ServiceError> {
        let draft = key_draft(request_id)?;
        Ok(self
            .store
            .exchange_connector_key(
                session_id,
                initial_key,
                &draft,
                self.policy.keys,
                clock.now()?,
            )
            .await?)
    }
    pub async fn refresh_connector_key(
        &self,
        session_id: &str,
        credential: &SecretDigest,
        request_id: &str,
        clock: AcceptanceTime,
    ) -> Result<ComponentKeyResult, ServiceError> {
        let draft = key_draft(request_id)?;
        Ok(self
            .store
            .refresh_connector_key(
                session_id,
                credential,
                &draft,
                self.policy.keys,
                clock.now()?,
            )
            .await?)
    }
    pub async fn revoke_connector_session(
        &self,
        provider_id: &str,
        credential: &SecretDigest,
        session_id: &str,
        request_id: &str,
        clock: AcceptanceTime,
    ) -> Result<(), ServiceError> {
        Ok(self
            .store
            .revoke_connector_session(
                provider_id,
                credential,
                session_id,
                request_id,
                clock.now()?,
            )
            .await?)
    }
    pub async fn connector_identity(
        &self,
        session_id: &str,
        credential: &SecretDigest,
        clock: AcceptanceTime,
    ) -> Result<ConnectorIdentity, ServiceError> {
        Ok(self
            .store
            .connector_key_identity(session_id, credential, clock.now()?)
            .await?)
    }

    /// Native scope is trusted Provider configuration. Resource names and execution
    /// identity are server-generated and committed before any external effect.
    pub async fn reserve_execution(
        &self,
        source: &ProviderSource<'_>,
        scope: ComputerRequest<'_>,
        native_scope: &NativeScope,
    ) -> Result<ExecutionReservation, ServiceError> {
        let suffix = native_name_suffix()?;
        let saved_name = format!("computer-{}", native_name_suffix()?);
        let native = match native_scope {
            NativeScope::KubernetesNamespace(namespace) => ExecutionResources::Kubernetes {
                namespace: namespace.clone(),
                pod_name: format!("computer-{suffix}"),
                bootstrap_secret_name: format!("connector-{suffix}"),
                saved_pvc_name: saved_name,
            },
            NativeScope::DockerDaemon(daemon) => ExecutionResources::Docker {
                daemon: daemon.clone(),
                container_name: format!("computer-{suffix}"),
                saved_volume_name: saved_name,
            },
        };
        let draft = ExecutionDraft { native };
        Ok(self.store.reserve_execution(source, scope, &draft).await?)
    }
    pub async fn claim_execution_dispatch(
        &self,
        source: &ProviderSource<'_>,
        scope: ComputerRequest<'_>,
        execution: &str,
    ) -> Result<bool, ServiceError> {
        Ok(self
            .store
            .claim_execution_dispatch(source, scope, execution)
            .await?)
    }
    pub async fn verify_native_cleanup_suffix(
        &self,
        source: &ProviderSource<'_>,
        scope: ComputerRequest<'_>,
    ) -> Result<(), ServiceError> {
        Ok(self
            .store
            .verify_native_cleanup_suffix(source, scope)
            .await?)
    }

    pub async fn claim_native_retirement(
        &self,
        source: &ProviderSource<'_>,
        scope: ComputerRequest<'_>,
        terminal_execution: Option<&str>,
    ) -> Result<Option<chapchoo_persistence::NativeRetirementAdmission>, ServiceError> {
        Ok(self
            .store
            .claim_native_retirement(source, scope, terminal_execution)
            .await?)
    }

    pub async fn release_unstarted_native_retirement(
        &self,
        source: &ProviderSource<'_>,
        admission: &chapchoo_persistence::NativeRetirementAdmission,
    ) -> Result<(), ServiceError> {
        Ok(self
            .store
            .release_unstarted_native_retirement(source, admission)
            .await?)
    }

    pub async fn connector_identity_for_prepared(
        &self,
        prepared: &PreparedComponentConnection,
    ) -> Result<ManagedComputer, ServiceError> {
        let ConnectionTarget::Connector(session) = &prepared.target else {
            return Err(ServiceError::domain(DomainReason::PermissionDenied));
        };
        self.store
            .connector_managed_computer(&ConnectorSource {
                session_id: session,
                control_id: &prepared.owner.control_id,
                connection_id: &prepared.owner.connection_id,
                credential: &prepared.credential,
            })
            .await
            .map_err(Into::into)
    }
    pub async fn release_undelivered_prepare(
        &self,
        source: &ProviderSource<'_>,
        scope: ComputerRequest<'_>,
        execution: &str,
    ) -> Result<(), ServiceError> {
        Ok(self
            .store
            .release_undelivered_prepare(source, scope, execution)
            .await?)
    }
    pub async fn report_preparation_blocked(
        &self,
        source: &ProviderSource<'_>,
        scope: ComputerRequest<'_>,
        execution: &str,
    ) -> Result<(), ServiceError> {
        Ok(self
            .store
            .report_preparation_blocked(source, scope, execution)
            .await?)
    }
    pub async fn connector_managed_computer(
        &self,
        source: &ConnectorSource<'_>,
    ) -> Result<ManagedComputer, ServiceError> {
        Ok(self.store.connector_managed_computer(source).await?)
    }
    pub async fn report_resource(
        &self,
        source: &ProviderSource<'_>,
        scope: ComputerRequest<'_>,
        report: ResourceReport<'_>,
    ) -> Result<(), ServiceError> {
        Ok(self.store.report_resource(source, scope, report).await?)
    }
    pub async fn mark_start_dispatched(
        &self,
        source: &ProviderSource<'_>,
        scope: ComputerRequest<'_>,
        execution_id: &str,
    ) -> Result<(), ServiceError> {
        Ok(self
            .store
            .mark_start_dispatched(source, scope, execution_id)
            .await?)
    }
    pub async fn report_execution_running(
        &self,
        source: &ProviderSource<'_>,
        scope: ComputerRequest<'_>,
        execution_id: &str,
    ) -> Result<(), ServiceError> {
        Ok(self
            .store
            .report_execution_running(source, scope, execution_id)
            .await?)
    }
    pub async fn retire_execution(
        &self,
        source: &ProviderSource<'_>,
        scope: ComputerRequest<'_>,
        execution_id: &str,
    ) -> Result<(), ServiceError> {
        Ok(self
            .store
            .retire_execution(source, scope, execution_id)
            .await?)
    }
    pub async fn report_lifecycle_issue(
        &self,
        source: &ProviderSource<'_>,
        scope: ComputerRequest<'_>,
        execution_id: Option<&str>,
        phase: LifecyclePhase,
    ) -> Result<(), ServiceError> {
        Ok(self
            .store
            .report_lifecycle_issue(source, scope, execution_id, phase)
            .await?)
    }
    pub async fn managed_computers(
        &self,
        source: &ProviderSource<'_>,
        after: Option<&str>,
        limit: i64,
    ) -> Result<Vec<ManagedComputer>, ServiceError> {
        Ok(self.store.managed_computers(source, after, limit).await?)
    }
    pub async fn managed_resources(
        &self,
        source: &ProviderSource<'_>,
        scope: ComputerRequest<'_>,
    ) -> Result<Vec<ManagedResource>, ServiceError> {
        Ok(self.store.managed_resources(source, scope).await?)
    }
    pub async fn confirm_computer_cleanup(
        &self,
        source: &ProviderSource<'_>,
        scope: ComputerRequest<'_>,
    ) -> Result<(), ServiceError> {
        Ok(self.store.confirm_computer_cleanup(source, scope).await?)
    }
    pub async fn collect_completed_computers(
        &self,
        source: &ProviderSource<'_>,
        limit: i64,
    ) -> Result<usize, ServiceError> {
        Ok(self
            .store
            .collect_completed_computers(source, limit)
            .await?)
    }
}
fn secret() -> Result<BearerSecret, ServiceError> {
    BearerSecret::generate().map_err(ServiceError::infrastructure)
}
fn native_name_suffix() -> Result<String, ServiceError> {
    Ok(secret()?
        .digest()
        .as_bytes()
        .iter()
        .take(16)
        .map(|byte| format!("{byte:02x}"))
        .collect())
}
fn key_draft(request_id: &str) -> Result<ComponentKeyDraft, ServiceError> {
    Ok(ComponentKeyDraft {
        request_id: request_id.into(),
        token: secret()?,
    })
}
