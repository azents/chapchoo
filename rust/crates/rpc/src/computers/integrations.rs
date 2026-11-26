use super::{status, timestamp, wire};
use crate::status::reject;
use chapchoo_service::{
    AgentIntegrationAction, AgentIntegrationChange, AgentIntegrationReceipt, AgentMcpPage,
    AgentMcpTestOutcome, DomainReason, McpAuthEdit, McpBearer, McpDiscoveryFailure, McpEndpoint,
    McpSettingsDraft,
};
use tonic::Status;

pub(super) fn connection(
    value: Option<wire::AgentMcpConnectionDraft>,
) -> Result<(McpEndpoint, McpAuthEdit), Status> {
    use wire::agent_mcp_connection_draft::Authentication;
    let value = value.ok_or_else(invalid)?;
    let auth = match value.authentication.ok_or_else(invalid)? {
        Authentication::NoAuth(()) => McpAuthEdit::NoAuth,
        Authentication::KeepBearer(()) => McpAuthEdit::KeepBearer,
        Authentication::ReplaceBearer(token) => McpAuthEdit::ReplaceBearer(
            McpBearer::parse(token).map_err(|error| status(error.into()))?,
        ),
    };
    let endpoint = McpEndpoint::parse(&value.endpoint).map_err(|error| status(error.into()))?;
    Ok((endpoint, auth))
}

fn invalid() -> Status {
    reject(DomainReason::InvalidInput)
}

fn draft(value: wire::AgentMcpDraft) -> Result<McpSettingsDraft, Status> {
    let (endpoint, auth) = connection(value.connection)?;
    Ok(McpSettingsDraft {
        display_name: value.display_name,
        endpoint,
        auth,
    })
}

pub(super) fn change(
    value: Option<wire::agent_integration_change_request::Change>,
) -> Result<AgentIntegrationChange, Status> {
    use wire::agent_integration_change_request::Change;
    Ok(match value.ok_or_else(invalid)? {
        Change::CreateMcp(value) => AgentIntegrationChange::CreateMcp(draft(value)?),
        Change::UpdateMcp(value) => AgentIntegrationChange::UpdateMcp {
            id: value.integration_id,
            draft: draft(value.draft.ok_or_else(invalid)?)?,
        },
        Change::DeleteIntegrationId(id) => AgentIntegrationChange::Delete { id },
    })
}

pub(super) fn list(page: AgentMcpPage) -> wire::AgentIntegrationList {
    wire::AgentIntegrationList {
        items: page
            .items
            .into_iter()
            .map(|item| wire::AgentIntegration {
                id: item.id,
                display_name: item.display_name,
                settings: Some(wire::agent_integration::Settings::Mcp(
                    wire::AgentMcpSettings {
                        endpoint: item.endpoint.url().to_string(),
                        has_bearer: item.has_bearer,
                    },
                )),
            })
            .collect(),
        next_after: page.next_after,
    }
}

pub(super) fn receipt(
    value: AgentIntegrationReceipt,
) -> Result<wire::AgentIntegrationReceipt, Status> {
    let action = match value.action {
        AgentIntegrationAction::Create => wire::AgentIntegrationAction::Create,
        AgentIntegrationAction::Update => wire::AgentIntegrationAction::Update,
        AgentIntegrationAction::Delete => wire::AgentIntegrationAction::Delete,
    };
    Ok(wire::AgentIntegrationReceipt {
        integration_id: value.id,
        action: action.into(),
        accepted_at: Some(timestamp(value.accepted_at)?),
    })
}

pub(super) fn test_result(
    value: AgentMcpTestOutcome,
) -> Result<wire::TestMcpConnectionResult, Status> {
    use wire::{McpConnectionTestFailure as Failure, test_mcp_connection_result::Outcome};
    let outcome = match value {
        AgentMcpTestOutcome::Listed { tool_count } => Outcome::ListedToolCount(
            u32::try_from(tool_count).map_err(|_| Status::internal("Invalid tool count"))?,
        ),
        AgentMcpTestOutcome::Failed(reason) => Outcome::Failure(
            match reason {
                McpDiscoveryFailure::DestinationDenied => Failure::DestinationDenied,
                McpDiscoveryFailure::Busy => Failure::Busy,
                McpDiscoveryFailure::Unavailable => Failure::Unavailable,
                McpDiscoveryFailure::Authentication => Failure::Authentication,
                McpDiscoveryFailure::UnsupportedVersion => Failure::UnsupportedVersion,
                McpDiscoveryFailure::MissingDiscovery => Failure::MissingDiscovery,
                McpDiscoveryFailure::MissingToolsCapability => Failure::MissingToolsCapability,
                McpDiscoveryFailure::InvalidResponse => Failure::InvalidResponse,
                McpDiscoveryFailure::IncompleteCatalogue => Failure::IncompleteCatalogue,
                McpDiscoveryFailure::TimedOut => Failure::TimedOut,
            }
            .into(),
        ),
    };
    Ok(wire::TestMcpConnectionResult {
        outcome: Some(outcome),
    })
}
