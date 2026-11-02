use crate::{AcceptanceTime, DomainReason, ServiceError};
use chapchoo_crypto::{BearerSecret, SecretDigest, SystemOwnerVerifier, VerifiedSystemOwner};
use chapchoo_domain::LifecyclePhase;
use chapchoo_persistence::{
    AgentRemoval, AgentSummary, ComponentKeyDraft, ComponentKeyPolicy, ComponentKeyResult,
    ComputerRequest, ConnectorIdentity, ConnectorSessionDraft, ConnectorSessionResult,
    CreateCloudAgent, CreationOutcome, ExecutionDraft, ExecutionReservation, LifecycleRequest,
    ManagedComputer, ManagedResource, ProviderCapabilities, ProviderChoice, ProviderOwner,
    ProviderSource, ResourceReport, Store,
};
use std::{fmt, sync::Arc};

/// Transport-independent application boundary. Only the composition root supplies
/// the configured system-owner verifier; requests cannot assert an owner identity.
#[derive(Clone)]
pub struct ComputersService {
    store: Store,
    system_owner: Option<Arc<SystemOwnerVerifier>>,
}
impl fmt::Debug for ComputersService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ComputersService([REDACTED])")
    }
}
impl ComputersService {
    pub fn new(store: Store, system_owner: Option<SystemOwnerVerifier>) -> Self {
        Self {
            store,
            system_owner: system_owner.map(Arc::new),
        }
    }

    fn system_proof(
        &self,
        provider_id: &str,
        credential: &BearerSecret,
    ) -> Result<VerifiedSystemOwner, ServiceError> {
        self.system_owner
            .as_ref()
            .ok_or_else(|| ServiceError::domain(DomainReason::PermissionDenied))?
            .verify(provider_id, credential)
            .map_err(|_| ServiceError::domain(DomainReason::PermissionDenied))
    }

    pub async fn configure_system_provider(
        &self,
        credential: &BearerSecret,
        provider_id: &str,
        name: &str,
        capabilities: ProviderCapabilities,
        clock: AcceptanceTime,
    ) -> Result<(), ServiceError> {
        let proof = self.system_proof(provider_id, credential)?;
        Ok(self
            .store
            .configure_system_provider(&proof, name, capabilities, clock.now()?)
            .await?)
    }
    pub async fn rotate_system_provider_key(
        &self,
        credential: &BearerSecret,
        provider_id: &str,
        request_id: &str,
        clock: AcceptanceTime,
    ) -> Result<ComponentKeyResult, ServiceError> {
        let proof = self.system_proof(provider_id, credential)?;
        let draft = key_draft(request_id)?;
        Ok(self
            .store
            .rotate_provider_key(
                ProviderOwner::System(&proof),
                provider_id,
                &draft,
                ComponentKeyPolicy::default(),
                clock.now()?,
            )
            .await?)
    }
    pub async fn revoke_system_provider_key(
        &self,
        credential: &BearerSecret,
        provider_id: &str,
        request_id: &str,
        clock: AcceptanceTime,
    ) -> Result<(), ServiceError> {
        let proof = self.system_proof(provider_id, credential)?;
        Ok(self
            .store
            .revoke_provider_key(
                ProviderOwner::System(&proof),
                provider_id,
                request_id,
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
    ) -> Result<ComponentKeyResult, ServiceError> {
        let draft = key_draft(request_id)?;
        Ok(self
            .store
            .rotate_provider_key(
                ProviderOwner::User(actor),
                provider_id,
                &draft,
                ComponentKeyPolicy::default(),
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
    ) -> Result<ProviderChoice, ServiceError> {
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
        request_id: &str,
        provider_id: &str,
        name: &str,
        clock: AcceptanceTime,
    ) -> Result<CreationOutcome, ServiceError> {
        let draft = CreateCloudAgent {
            agent_id: opaque_id()?,
            computer_id: opaque_id()?,
            request_id: request_id.into(),
            provider_id: provider_id.into(),
            name: name.into(),
        };
        Ok(self
            .store
            .create_cloud_agent(actor, &draft, clock.now()?)
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
        after: Option<&str>,
        limit: i64,
        clock: AcceptanceTime,
    ) -> Result<Vec<AgentSummary>, ServiceError> {
        Ok(self.store.agents(actor, after, limit, clock.now()?).await?)
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
            id: opaque_id()?,
            request_id: request_id.into(),
            initial_key: secret()?,
        };
        Ok(self
            .store
            .issue_connector_session(provider_id, credential, execution_id, &draft, clock.now()?)
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
                ComponentKeyPolicy::default(),
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
                ComponentKeyPolicy::default(),
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

    /// Namespace is trusted Provider configuration. Resource names and execution
    /// identity are server-generated and committed before any external effect.
    pub async fn reserve_execution(
        &self,
        source: &ProviderSource<'_>,
        scope: ComputerRequest<'_>,
        namespace: &str,
    ) -> Result<ExecutionReservation, ServiceError> {
        let id = opaque_id()?;
        let draft = ExecutionDraft {
            pod_name: format!("computer-{id}"),
            bootstrap_secret_name: format!("connector-{id}"),
            saved_pvc_name: format!("computer-{}", opaque_id()?),
            id,
            namespace: namespace.into(),
        };
        Ok(self.store.reserve_execution(source, scope, &draft).await?)
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
fn opaque_id() -> Result<String, ServiceError> {
    Ok(secret()?
        .digest()
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}
fn key_draft(request_id: &str) -> Result<ComponentKeyDraft, ServiceError> {
    Ok(ComponentKeyDraft {
        request_id: request_id.into(),
        token: secret()?,
    })
}
