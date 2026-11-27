//! Saved MCP configuration scoped to an authenticated execution.

use super::{records::source_conversation, system_prompts::selected_preparation, *};
use crate::{
    ConnectorSource, McpTestTarget,
    computers::integrations::{SECRET_ROW, SecretRow},
};
use diesel::QueryableByName;

const PAGE_SIZE: u16 = 16;

#[derive(Debug)]
pub struct McpExecutionScope<'a> {
    pub agent_id: &'a str,
    pub conversation_id: &'a str,
    pub execution_id: &'a str,
}

#[derive(Debug)]
pub struct McpPreparationScope<'a> {
    pub execution: McpExecutionScope<'a>,
    pub operation_id: &'a str,
}

#[derive(Debug)]
pub struct McpAppliedScope<'a> {
    pub preparation: McpPreparationScope<'a>,
    pub target: &'a ConversationDeliveryTarget,
}

/// Identities for retention only; this carries no tool authority.
#[derive(Debug)]
pub struct McpRetainedSelection<'a> {
    pub preparation: McpPreparationScope<'a>,
    pub session_id: &'a str,
    pub control_id: &'a str,
    pub connection_id: &'a str,
}

#[derive(diesel::QueryableByName)]
struct RetentionRow {
    #[diesel(sql_type = diesel::sql_types::Bool)]
    retained: bool,
}

#[derive(diesel::QueryableByName)]
struct AppliedPreparation {
    #[diesel(sql_type = IdentitySql)]
    operation_id: EntityId,
}

#[derive(Debug)]
pub struct SavedMcpConnection {
    pub integration_id: String,
    pub settings_key: SecretDigest,
    pub target: McpTestTarget,
}

#[derive(Debug)]
pub struct SavedMcpPage {
    pub items: Vec<SavedMcpConnection>,
    pub next_after: Option<String>,
}

#[derive(diesel::QueryableByName)]
struct IntegrationRow {
    #[diesel(sql_type = IdentitySql)]
    id: EntityId,
}

#[derive(diesel::QueryableByName)]
pub(super) struct SavedRow {
    #[diesel(sql_type = IdentitySql)]
    pub(super) id: EntityId,
    #[diesel(embed)]
    pub(super) secret: SecretRow,
}

impl McpExecutionScope<'_> {
    fn ids(&self) -> Result<(EntityId, EntityId, EntityId), StoreError> {
        Ok((
            EntityId::from_dto(EntityKind::Agent, self.agent_id)?,
            EntityId::from_dto(EntityKind::Conversation, self.conversation_id)?,
            EntityId::from_dto(EntityKind::Execution, self.execution_id)?,
        ))
    }
}

macro_rules! check_mcp_execution {
    ($connection:expr, $dialect:expr, $source:expr, $agent:expr, $conversation:expr, $execution:expr) => {{
        let row = source_conversation!($connection, $dialect, $source, $conversation);
        let route = connector!($connection, $dialect, $source);
        if row.agent_id != $agent || route._execution_id != $execution {
            return Err(StoreError::NotFound);
        }
    }};
}

macro_rules! check_mcp_preparation {
    ($connection:expr, $dialect:expr, $source:expr, $agent:expr, $conversation:expr, $execution:expr, $preparation:expr) => {{
        check_mcp_execution!(
            $connection,
            $dialect,
            $source,
            $agent,
            $conversation,
            $execution
        );
        let current = selected_preparation!($connection, $dialect, $conversation)
            .ok_or(StoreError::InvalidTransition)?;
        if current.operation_id != $preparation
            || current.execution_id != Some($execution)
            || current.preparation_state != "POSSIBLE"
        {
            return Err(StoreError::InvalidTransition);
        }
    }};
}

impl Store {
    /// Keep only selections whose preparation and component route still exist.
    pub async fn retained_mcp_selections(
        &self,
        selections: &[McpRetainedSelection<'_>],
    ) -> Result<Vec<bool>, StoreError> {
        if selections.len() > 32 {
            return Err(StoreError::InvalidInput);
        }
        operation!(self, |connection, dialect| {
            let mut retained = Vec::with_capacity(selections.len());
            let sql = dialect.query(
                "SELECT EXISTS(
                  SELECT 1 FROM conversations c
                  JOIN agent_conversations ac ON ac.agent_id=c.agent_id AND ac.conversation_id=c.id
                  JOIN agents a ON a.id=c.agent_id JOIN computers co ON co.id=a.computer_id
                  JOIN connector_connections r ON r.computer_id=co.id
                  JOIN connector_sessions s ON s.id=r.session_id AND s.execution_id=co.current_execution_id
                  LEFT JOIN conversation_startups p ON p.operation_id=c.current_preparation_operation_id
                    AND p.conversation_id=c.id AND p.execution_id=co.current_execution_id
                    AND p.preparation_state='POSSIBLE' AND c.source_state='READY'
                    AND p.intended_snapshot=c.source_snapshot AND p.intended_native_session=c.native_session_id
                  WHERE c.id=? AND c.agent_id=?
                    AND ? IN (c.current_preparation_operation_id,p.external_predecessor_id)
                    AND co.current_execution_id=? AND r.session_id=? AND r.control_id=? AND r.connection_id=?
                    AND s.revoked_at IS NULL AND co.desired_state<>'DELETING'
                ) AS retained",
            );
            for selection in selections {
                let (agent, conversation, execution) = selection.preparation.execution.ids()?;
                let operation = EntityId::from_dto(
                    EntityKind::ConversationOperation,
                    selection.preparation.operation_id,
                )?;
                let row = diesel::sql_query(&sql)
                    .bind::<IdentitySql, _>(conversation)
                    .bind::<IdentitySql, _>(agent)
                    .bind::<IdentitySql, _>(operation)
                    .bind::<IdentitySql, _>(execution)
                    .bind::<IdentitySql, _>(EntityId::from_dto(
                        EntityKind::ConnectorSession,
                        selection.session_id,
                    )?)
                    .bind::<IdentitySql, _>(EntityId::from_dto(
                        EntityKind::ControlInstance,
                        selection.control_id,
                    )?)
                    .bind::<IdentitySql, _>(EntityId::from_dto(
                        EntityKind::Connection,
                        selection.connection_id,
                    )?)
                    .get_result::<RetentionRow>(connection)
                    .await?;
                retained.push(row.retained);
            }
            Ok(retained)
        })
    }

    /// Authenticate the route before disclosing any selected catalogue facts.
    /// Actual dispatch still requires `admit_mcp_settings_for_call`.
    pub async fn check_mcp_execution_route(
        &self,
        source: &ConnectorSource<'_>,
        scope: &McpExecutionScope<'_>,
    ) -> Result<(), StoreError> {
        let (agent, conversation, execution) = scope.ids()?;
        operation!(self, |connection, dialect| {
            check_mcp_execution!(connection, dialect, source, agent, conversation, execution);
            Ok(())
        })
    }

    /// Match exact candidates after discovery without re-opening credentials.
    /// This observation does not grant native application or tool execution.
    pub async fn current_mcp_preparation_candidates(
        &self,
        source: &ConnectorSource<'_>,
        scope: &McpPreparationScope<'_>,
        candidates: &[(&str, &SecretDigest)],
    ) -> Result<Vec<bool>, StoreError> {
        if candidates.len() > usize::from(PAGE_SIZE) {
            return Err(StoreError::InvalidInput);
        }
        let (agent, conversation, execution) = scope.execution.ids()?;
        let preparation =
            EntityId::from_dto(EntityKind::ConversationOperation, scope.operation_id)?;
        operation!(self, |connection, dialect| {
            check_mcp_preparation!(
                connection,
                dialect,
                source,
                agent,
                conversation,
                execution,
                preparation
            );
            let mut current = Vec::with_capacity(candidates.len());
            for (id, expected) in candidates {
                let row = diesel::sql_query(dialect.query(SECRET_ROW))
                    .bind::<IdentitySql, _>(agent)
                    .bind::<IdentitySql, _>(EntityId::from_dto(EntityKind::AgentIntegration, id)?)
                    .get_result::<SecretRow>(connection)
                    .await
                    .optional()?;
                let matches = match row {
                    Some(row) => row
                        .settings_key(&self.cipher, scope.execution.agent_id, id)?
                        .matches_digest(expected),
                    None => false,
                };
                current.push(matches);
            }
            Ok(current)
        })
    }

    /// Capture candidates for server-owned preparation, not a tool grant.
    /// A discovery result cannot publish against superseded settings.
    pub async fn capture_mcp_settings_for_preparation(
        &self,
        source: &ConnectorSource<'_>,
        scope: &McpPreparationScope<'_>,
        after: Option<&str>,
    ) -> Result<SavedMcpPage, StoreError> {
        let (agent, conversation, execution) = scope.execution.ids()?;
        let preparation =
            EntityId::from_dto(EntityKind::ConversationOperation, scope.operation_id)?;
        let cursor = after
            .map(|id| EntityId::from_dto(EntityKind::AgentIntegration, id))
            .transpose()?;
        let mut rows = operation!(self, |connection, dialect| {
            check_mcp_preparation!(
                connection,
                dialect,
                source,
                agent,
                conversation,
                execution,
                preparation
            );
            let id_type = match dialect {
                Dialect::Postgres => "UUID",
                Dialect::Sqlite => "BLOB",
            };
            let sql = format!(
                "SELECT i.id,s.endpoint,s.token_nonce,s.token_ciphertext FROM agent_integrations i JOIN agent_mcp_settings s ON s.integration_id=i.id WHERE i.agent_id=? AND i.kind='MCP' AND (CAST(? AS {id_type}) IS NULL OR i.id>?) ORDER BY i.id LIMIT ?"
            );
            Ok(diesel::sql_query(dialect.query(&sql))
                .bind::<IdentitySql, _>(agent)
                .bind::<Nullable<IdentitySql>, _>(cursor)
                .bind::<Nullable<IdentitySql>, _>(cursor)
                .bind::<BigInt, _>(i64::from(PAGE_SIZE) + 1)
                .load::<SavedRow>(connection)
                .await?)
        })?;
        let has_more = rows.len() > usize::from(PAGE_SIZE);
        rows.truncate(usize::from(PAGE_SIZE));
        let next_after = if has_more {
            rows.last()
                .map(|row| row.id.to_dto(EntityKind::AgentIntegration))
        } else {
            None
        };
        let items = rows
            .into_iter()
            .map(|row| {
                let id = row.id.to_dto(EntityKind::AgentIntegration);
                Ok(SavedMcpConnection {
                    settings_key: row.secret.settings_key(
                        &self.cipher,
                        scope.execution.agent_id,
                        &id,
                    )?,
                    target: row
                        .secret
                        .into_target(&self.cipher, scope.execution.agent_id, &id)?,
                    integration_id: id,
                })
            })
            .collect::<Result<_, StoreError>>()?;
        Ok(SavedMcpPage { items, next_after })
    }

    /// The application first matches the selected native declaration and tool.
    /// Recheck the route and lock the saved record against delete/replace. No
    /// HTTP is performed in the transaction; an admitted operation owns its
    /// captured target even after the integration record has been removed.
    pub async fn admit_mcp_settings_for_call(
        &self,
        source: &ConnectorSource<'_>,
        scope: &McpAppliedScope<'_>,
        integration_id: &str,
        expected_settings: &SecretDigest,
    ) -> Result<McpTestTarget, StoreError> {
        let (agent, conversation, execution) = scope.preparation.execution.ids()?;
        let operation = EntityId::from_dto(
            EntityKind::ConversationOperation,
            scope.preparation.operation_id,
        )?;
        let agent_id = scope.preparation.execution.agent_id;
        let integration = EntityId::from_dto(EntityKind::AgentIntegration, integration_id)?;
        let connection_id = EntityId::from_dto(EntityKind::Connection, source.connection_id)?;
        let row = operation!(self, |connection, dialect| {
            check_mcp_execution!(connection, dialect, source, agent, conversation, execution);
            let lock = match dialect {
                Dialect::Postgres => {
                    "SELECT id FROM agent_integrations WHERE agent_id=? AND id=? AND kind='MCP' FOR NO KEY UPDATE"
                }
                Dialect::Sqlite => {
                    "SELECT id FROM agent_integrations WHERE agent_id=? AND id=? AND kind='MCP'"
                }
            };
            let locked = diesel::sql_query(dialect.query(lock))
                .bind::<IdentitySql, _>(agent)
                .bind::<IdentitySql, _>(integration)
                .get_result::<IntegrationRow>(connection)
                .await
                .optional()?
                .ok_or(StoreError::NotFound)?;
            // A possible exact-context refresh can prepare the next supply
            // without revoking the predecessor still serving the current turn.
            // A ready/failed successor no longer carries that predecessor.
            let applied = diesel::sql_query(dialect.query(
                "SELECT COALESCE(e.operation_id,p.external_predecessor_id) AS operation_id
                 FROM conversations c
                 JOIN agent_conversations a ON a.agent_id=c.agent_id AND a.conversation_id=c.id
                 LEFT JOIN project_instruction_evidence e
                   ON e.conversation_id=c.id AND e.agent_id=c.agent_id AND e.operation_id=c.current_preparation_operation_id
                   AND e.applied_at IS NOT NULL AND e.applied_snapshot=c.source_snapshot AND e.applied_native_session=c.native_session_id
                   AND e.execution_id=?
                 LEFT JOIN conversation_startups p
                   ON p.operation_id=c.current_preparation_operation_id AND p.conversation_id=c.id
                   AND p.preparation_state='POSSIBLE' AND p.execution_id=?
                   AND p.intended_snapshot=c.source_snapshot AND p.intended_native_session=c.native_session_id
                 LEFT JOIN conversation_startups s
                   ON s.operation_id=COALESCE(e.operation_id,p.external_predecessor_id)
                   AND s.conversation_id=c.id AND s.execution_id=COALESCE(e.execution_id,p.execution_id)
                 LEFT JOIN conversation_input_preparation i
                   ON i.operation_id=COALESCE(e.operation_id,p.external_predecessor_id)
                   AND i.execution_id=COALESCE(e.execution_id,p.execution_id)
                 WHERE c.id=? AND c.agent_id=? AND COALESCE(e.operation_id,p.external_predecessor_id)=?
                   AND COALESCE(s.external_ready_connection_id,i.ready_connection_id)=?
                   AND c.source_state='READY' AND c.source_snapshot=? AND c.native_session_id=?",
            ))
            .bind::<IdentitySql, _>(execution)
            .bind::<IdentitySql, _>(execution)
            .bind::<IdentitySql, _>(conversation)
            .bind::<IdentitySql, _>(agent)
            .bind::<IdentitySql, _>(operation)
            .bind::<IdentitySql, _>(connection_id)
            .bind::<Text, _>(scope.target.snapshot.as_str())
            .bind::<Text, _>(scope.target.native_session.as_str())
            .get_result::<AppliedPreparation>(connection)
            .await
            .optional()?;
            if !applied.is_some_and(|row| row.operation_id == operation) {
                return Err(StoreError::InvalidTransition);
            }
            // Read after locking, not from a join snapshot that preceded an
            // update or deletion that held the PostgreSQL row lock.
            let row = diesel::sql_query(dialect.query(SECRET_ROW))
                .bind::<IdentitySql, _>(agent)
                .bind::<IdentitySql, _>(locked.id)
                .get_result::<SecretRow>(connection)
                .await?;
            let key = row.settings_key(&self.cipher, agent_id, integration_id)?;
            if !key.matches_digest(expected_settings) {
                return Err(StoreError::Conflict);
            }
            Ok(row)
        })?;
        row.into_target(&self.cipher, agent_id, integration_id)
    }
}
