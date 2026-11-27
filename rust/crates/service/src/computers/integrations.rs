use super::ComputersService;
use crate::{
    AcceptanceTime, AgentIntegrationChange, AgentIntegrationReceipt, AgentMcpPage,
    AgentMcpTestOutcome, McpTestEdit, ServiceError,
};
use chapchoo_crypto::SecretDigest;

impl ComputersService {
    pub async fn agent_integrations(
        &self,
        actor: &SecretDigest,
        agent: &str,
        after: Option<&str>,
        clock: AcceptanceTime,
    ) -> Result<AgentMcpPage, ServiceError> {
        Ok(self
            .store
            .agent_mcp_integrations(actor, agent, after, clock.now()?)
            .await?)
    }

    pub async fn change_agent_integration(
        &self,
        actor: &SecretDigest,
        agent: &str,
        request: &str,
        change: &AgentIntegrationChange,
        clock: AcceptanceTime,
    ) -> Result<AgentIntegrationReceipt, ServiceError> {
        Ok(self
            .store
            .change_agent_integration(actor, agent, request, change, clock.now()?)
            .await?)
    }

    pub async fn agent_integration_result(
        &self,
        actor: &SecretDigest,
        agent: &str,
        request: &str,
        clock: AcceptanceTime,
    ) -> Result<Option<AgentIntegrationReceipt>, ServiceError> {
        Ok(self
            .store
            .agent_integration_result(actor, agent, request, clock.now()?)
            .await?)
    }

    pub async fn test_mcp_connection(
        &self,
        actor: &SecretDigest,
        agent: &str,
        edit: McpTestEdit<'_>,
        clock: AcceptanceTime,
    ) -> Result<AgentMcpTestOutcome, ServiceError> {
        self.mcp_test.test(actor, agent, edit, clock).await
    }
}
