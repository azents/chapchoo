//! Agent-owned integration settings and acknowledged management mutations.

use super::{AgentTeamRow, valid_id, valid_name};
use crate::{
    Store, StoreError,
    entity_id::{EntityId, EntityKind, IdentitySql},
    models::authenticate,
    store::{Dialect, operation},
};
use chapchoo_crypto::{
    CredentialCipher, ExternalSecret, ExternalSecretContext, ExternalSecretEnvelope,
    ExternalSecretKind, ExternalSecretOwner, SecretDigest,
};
use diesel::{
    OptionalExtension,
    sql_types::{BigInt, Binary, Nullable, Text},
};
use diesel_async::RunQueryDsl;
use std::fmt;
use url::Url;
use zeroize::Zeroizing;

const PAGE_SIZE: u16 = 100;

#[cfg(test)]
pub(super) mod tests;

/// Syntax validation only. The outbound transport owns destination authorization.
#[derive(Clone, PartialEq, Eq)]
pub struct McpEndpoint(Url);

impl McpEndpoint {
    pub fn parse(value: &str) -> Result<Self, StoreError> {
        if value.len() > 4096
            || !value.starts_with("https://")
            || value.chars().any(|c| c.is_control() || c.is_whitespace())
        {
            return Err(StoreError::InvalidInput);
        }
        let url = Url::parse(value).map_err(|_| StoreError::InvalidInput)?;
        if url.scheme() != "https"
            || url.host().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.fragment().is_some()
            || url.as_str().len() > 4096
        {
            return Err(StoreError::InvalidInput);
        }
        Ok(Self(url))
    }

    pub fn url(&self) -> &Url {
        &self.0
    }
}

impl fmt::Debug for McpEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("McpEndpoint([REDACTED])")
    }
}

/// Opaque value suitable for the Bearer authorization header, without its prefix.
#[derive(Debug)]
pub struct McpBearer(ExternalSecret);

impl McpBearer {
    pub fn parse(value: String) -> Result<Self, StoreError> {
        let value = Zeroizing::new(value);
        if !valid_bearer(value.as_bytes()) {
            return Err(StoreError::InvalidInput);
        }
        Ok(Self(ExternalSecret::new(value.as_bytes().to_vec())?))
    }

    fn from_secret(secret: ExternalSecret) -> Result<Self, StoreError> {
        let value = secret.expose();
        if !valid_bearer(value) {
            return Err(StoreError::Integrity);
        }
        Ok(Self(secret))
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.expose()
    }
}

fn valid_bearer(value: &[u8]) -> bool {
    !value.is_empty() && value.len() <= 8192 && value.iter().all(u8::is_ascii_graphic)
}

#[derive(Debug)]
pub enum McpAuthEdit {
    NoAuth,
    KeepBearer,
    ReplaceBearer(McpBearer),
}

#[derive(Debug)]
pub struct McpSettingsDraft {
    pub display_name: String,
    pub endpoint: McpEndpoint,
    pub auth: McpAuthEdit,
}

#[derive(Debug)]
pub enum AgentIntegrationChange {
    CreateMcp(McpSettingsDraft),
    UpdateMcp { id: String, draft: McpSettingsDraft },
    Delete { id: String },
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AgentIntegrationAction {
    Create,
    Update,
    Delete,
}

impl AgentIntegrationAction {
    fn name(self) -> &'static str {
        match self {
            Self::Create => "CREATE",
            Self::Update => "UPDATE",
            Self::Delete => "DELETE",
        }
    }

    fn parse(value: &str) -> Result<Self, StoreError> {
        match value {
            "CREATE" => Ok(Self::Create),
            "UPDATE" => Ok(Self::Update),
            "DELETE" => Ok(Self::Delete),
            _ => Err(StoreError::Integrity),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct AgentIntegrationReceipt {
    pub id: String,
    pub action: AgentIntegrationAction,
    pub accepted_at: i64,
}

#[derive(Debug)]
pub struct AgentMcpSummary {
    pub id: String,
    pub display_name: String,
    pub endpoint: McpEndpoint,
    pub has_bearer: bool,
}

#[derive(Debug)]
pub struct AgentMcpPage {
    pub items: Vec<AgentMcpSummary>,
    pub next_after: Option<String>,
}

#[derive(Debug)]
pub struct McpTestTarget {
    pub endpoint: McpEndpoint,
    pub bearer: Option<McpBearer>,
}

#[derive(diesel::QueryableByName)]
struct AgentRow {
    #[diesel(sql_type = IdentitySql)]
    id: EntityId,
}

// Keep lock order aligned with the surrounding private Agent operations.
macro_rules! own_integration_agent {
    ($connection:expr, $dialect:expr, $actor:expr, $agent:expr, $agent_id:expr, $now:expr) => {{
        let user = authenticate!($connection, $dialect, $actor, $now);
        lock_owned_agent_team!($connection, $dialect, $agent_id, &user.id);
        let sql = match $dialect {
            Dialect::Postgres => {
                "SELECT id FROM agents WHERE id=? AND owner_user_id=? FOR NO KEY UPDATE"
            }
            Dialect::Sqlite => "SELECT id FROM agents WHERE id=? AND owner_user_id=?",
        };
        diesel::sql_query($dialect.query(sql))
            .bind::<IdentitySql, _>($agent)
            .bind::<IdentitySql, _>(EntityId::from_dto(EntityKind::User, &user.id)?)
            .get_result::<AgentRow>($connection)
            .await
            .optional()?
            .ok_or(StoreError::NotFound)?
            .id
    }};
}

#[derive(diesel::QueryableByName)]
struct SummaryRow {
    #[diesel(sql_type = IdentitySql)]
    id: EntityId,
    #[diesel(sql_type = Text)]
    display_name: String,
    #[diesel(sql_type = Text)]
    endpoint: String,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    has_bearer: bool,
}

impl SummaryRow {
    fn summary(self) -> Result<AgentMcpSummary, StoreError> {
        Ok(AgentMcpSummary {
            id: self.id.to_dto(EntityKind::AgentIntegration),
            display_name: self.display_name,
            endpoint: McpEndpoint::parse(&self.endpoint).map_err(|_| StoreError::Integrity)?,
            has_bearer: self.has_bearer,
        })
    }
}

#[derive(diesel::QueryableByName)]
struct SecretRow {
    #[diesel(sql_type = Text)]
    endpoint: String,
    #[diesel(sql_type = Nullable<Binary>)]
    token_nonce: Option<Vec<u8>>,
    #[diesel(sql_type = Nullable<Binary>)]
    token_ciphertext: Option<Vec<u8>>,
}

impl SecretRow {
    fn keeper(&self, endpoint: &McpEndpoint) -> Result<ExternalSecretEnvelope, StoreError> {
        if self.endpoint != endpoint.url().as_str() {
            return Err(StoreError::InvalidInput);
        }
        match (&self.token_nonce, &self.token_ciphertext) {
            (Some(nonce), Some(ciphertext)) => Ok(ExternalSecretEnvelope::from_parts(
                nonce
                    .as_slice()
                    .try_into()
                    .map_err(|_| StoreError::Integrity)?,
                ciphertext.clone(),
            )?),
            (None, None) => Err(StoreError::InvalidInput),
            _ => Err(StoreError::Integrity),
        }
    }
}

#[derive(diesel::QueryableByName)]
struct ReceiptRow {
    #[diesel(sql_type = Binary)]
    payload_fingerprint: Vec<u8>,
    #[diesel(sql_type = IdentitySql)]
    integration_id: EntityId,
    #[diesel(sql_type = Text)]
    action: String,
    #[diesel(sql_type = BigInt)]
    accepted_at: i64,
}

impl ReceiptRow {
    fn receipt(self) -> Result<AgentIntegrationReceipt, StoreError> {
        Ok(AgentIntegrationReceipt {
            id: self.integration_id.to_dto(EntityKind::AgentIntegration),
            action: AgentIntegrationAction::parse(&self.action)?,
            accepted_at: self.accepted_at,
        })
    }
}

fn context<'a>(agent: &'a str, object: &'a str) -> Result<ExternalSecretContext<'a>, StoreError> {
    Ok(ExternalSecretContext::new(
        ExternalSecretOwner::Agent(agent),
        object,
        ExternalSecretKind::McpBearer,
    )?)
}

fn protect_settings(
    cipher: &CredentialCipher,
    agent: &str,
    id: &str,
    draft: &McpSettingsDraft,
    previous: Option<SecretRow>,
) -> Result<Option<ExternalSecretEnvelope>, StoreError> {
    match &draft.auth {
        McpAuthEdit::NoAuth => Ok(None),
        McpAuthEdit::ReplaceBearer(bearer) => Ok(Some(
            cipher.protect_external(&context(agent, id)?, &bearer.0)?,
        )),
        McpAuthEdit::KeepBearer => Ok(Some(
            previous
                .ok_or(StoreError::InvalidInput)?
                .keeper(&draft.endpoint)?,
        )),
    }
}

const SECRET_ROW: &str = "SELECT s.endpoint,s.token_nonce,s.token_ciphertext FROM agent_integrations i JOIN agent_mcp_settings s ON s.integration_id=i.id WHERE i.agent_id=? AND i.id=? AND i.kind='MCP'";
const RECEIPT_ROW: &str = "SELECT payload_fingerprint,integration_id,action,accepted_at FROM agent_integration_receipts WHERE agent_id=? AND request_id=?";

impl Store {
    pub async fn agent_mcp_integrations(
        &self,
        actor: &SecretDigest,
        agent_id: &str,
        after: Option<&str>,
        now: i64,
    ) -> Result<AgentMcpPage, StoreError> {
        let agent = EntityId::from_dto(EntityKind::Agent, agent_id)?;
        let cursor = after
            .map(|a| EntityId::from_dto(EntityKind::AgentIntegration, a))
            .transpose()?;
        operation!(self, |connection, dialect| {
            let agent = own_integration_agent!(connection, dialect, actor, agent, agent_id, now);
            let key_type = match dialect {
                Dialect::Postgres => "UUID",
                Dialect::Sqlite => "BLOB",
            };
            let sql = format!(
                "SELECT i.id,i.display_name,s.endpoint,(s.token_nonce IS NOT NULL) AS has_bearer FROM agent_integrations i JOIN agent_mcp_settings s ON s.integration_id=i.id WHERE i.agent_id=? AND i.kind='MCP' AND (CAST(? AS {key_type}) IS NULL OR i.id>?) ORDER BY i.id LIMIT ?"
            );
            let mut rows = diesel::sql_query(dialect.query(&sql))
                .bind::<IdentitySql, _>(agent)
                .bind::<Nullable<IdentitySql>, _>(cursor)
                .bind::<Nullable<IdentitySql>, _>(cursor)
                .bind::<BigInt, _>(i64::from(PAGE_SIZE) + 1)
                .load::<SummaryRow>(connection)
                .await?;
            let has_more = rows.len() > usize::from(PAGE_SIZE);
            rows.truncate(usize::from(PAGE_SIZE));
            let next_after = if has_more {
                rows.last()
                    .map(|r| r.id.to_dto(EntityKind::AgentIntegration))
            } else {
                None
            };
            let items = rows
                .into_iter()
                .map(SummaryRow::summary)
                .collect::<Result<_, _>>()?;
            Ok(AgentMcpPage { items, next_after })
        })
    }

    pub async fn agent_integration_result(
        &self,
        actor: &SecretDigest,
        agent_id: &str,
        request_id: &str,
        now: i64,
    ) -> Result<Option<AgentIntegrationReceipt>, StoreError> {
        if !valid_id(request_id) {
            return Err(StoreError::InvalidInput);
        }
        let agent = EntityId::from_dto(EntityKind::Agent, agent_id)?;
        operation!(self, |connection, dialect| {
            let agent = own_integration_agent!(connection, dialect, actor, agent, agent_id, now);
            diesel::sql_query(dialect.query(RECEIPT_ROW))
                .bind::<IdentitySql, _>(agent)
                .bind::<Text, _>(request_id)
                .get_result::<ReceiptRow>(connection)
                .await
                .optional()?
                .map(ReceiptRow::receipt)
                .transpose()
        })
    }

    pub async fn change_agent_integration(
        &self,
        actor: &SecretDigest,
        agent_id: &str,
        request_id: &str,
        change: &AgentIntegrationChange,
        now: i64,
    ) -> Result<AgentIntegrationReceipt, StoreError> {
        if !valid_id(request_id) {
            return Err(StoreError::InvalidInput);
        }
        let agent = EntityId::from_dto(EntityKind::Agent, agent_id)?;
        let (action, id, draft) = match change {
            AgentIntegrationChange::CreateMcp(draft) => {
                (AgentIntegrationAction::Create, None, Some(draft))
            }
            AgentIntegrationChange::UpdateMcp { id, draft } => (
                AgentIntegrationAction::Update,
                Some(id.as_str()),
                Some(draft),
            ),
            AgentIntegrationChange::Delete { id } => {
                (AgentIntegrationAction::Delete, Some(id.as_str()), None)
            }
        };
        let requested = id
            .map(|id| EntityId::from_dto(EntityKind::AgentIntegration, id))
            .transpose()?;
        let fingerprint = if let Some(draft) = draft {
            if !valid_name(&draft.display_name) {
                return Err(StoreError::InvalidInput);
            }
            let (mode, secret): (&[u8], &[u8]) = match &draft.auth {
                McpAuthEdit::NoAuth => (b"none", b""),
                McpAuthEdit::KeepBearer => (b"keep", b""),
                McpAuthEdit::ReplaceBearer(value) => (b"replace", value.as_bytes()),
            };
            self.cipher.external_request_fingerprint(
                &context(agent_id, agent_id)?,
                action.name(),
                &[
                    request_id.as_bytes(),
                    id.unwrap_or_default().as_bytes(),
                    draft.display_name.as_bytes(),
                    draft.endpoint.url().as_str().as_bytes(),
                    mode,
                    secret,
                ],
            )?
        } else {
            self.cipher.external_request_fingerprint(
                &context(agent_id, agent_id)?,
                action.name(),
                &[request_id.as_bytes(), id.unwrap_or_default().as_bytes()],
            )?
        };
        operation!(self, |connection, dialect| {
            let agent = own_integration_agent!(connection, dialect, actor, agent, agent_id, now);
            if let Some(row) = diesel::sql_query(dialect.query(RECEIPT_ROW))
                .bind::<IdentitySql, _>(agent)
                .bind::<Text, _>(request_id)
                .get_result::<ReceiptRow>(connection)
                .await
                .optional()?
            {
                let accepted = SecretDigest::from_bytes(
                    row.payload_fingerprint
                        .as_slice()
                        .try_into()
                        .map_err(|_| StoreError::Integrity)?,
                );
                if !fingerprint.matches_digest(&accepted) {
                    return Err(StoreError::Conflict);
                }
                return row.receipt();
            }
            let integration = match requested {
                Some(id) => id,
                None => EntityId::generate(now)?,
            };
            let integration_id = integration.to_dto(EntityKind::AgentIntegration);
            if let Some(draft) = draft {
                let previous = if action == AgentIntegrationAction::Update {
                    Some(
                        diesel::sql_query(dialect.query(SECRET_ROW))
                            .bind::<IdentitySql, _>(agent)
                            .bind::<IdentitySql, _>(integration)
                            .get_result::<SecretRow>(connection)
                            .await
                            .optional()?
                            .ok_or(StoreError::NotFound)?,
                    )
                } else {
                    None
                };
                let envelope =
                    protect_settings(&self.cipher, agent_id, &integration_id, draft, previous)?;
                if action == AgentIntegrationAction::Create {
                    diesel::sql_query(dialect.query("INSERT INTO agent_integrations (id,agent_id,display_name,kind) VALUES (?,?,?,'MCP')"))
                        .bind::<IdentitySql, _>(integration).bind::<IdentitySql, _>(agent)
                        .bind::<Text, _>(&draft.display_name).execute(connection).await?;
                } else {
                    diesel::sql_query(dialect.query("UPDATE agent_integrations SET display_name=? WHERE id=? AND agent_id=? AND kind='MCP'"))
                        .bind::<Text, _>(&draft.display_name).bind::<IdentitySql, _>(integration)
                        .bind::<IdentitySql, _>(agent).execute(connection).await?;
                }
                diesel::sql_query(dialect.query("INSERT INTO agent_mcp_settings (integration_id,endpoint,token_nonce,token_ciphertext) VALUES (?,?,?,?) ON CONFLICT (integration_id) DO UPDATE SET endpoint=excluded.endpoint,token_nonce=excluded.token_nonce,token_ciphertext=excluded.token_ciphertext"))
                    .bind::<IdentitySql, _>(integration).bind::<Text, _>(draft.endpoint.url().as_str())
                    .bind::<Nullable<Binary>, _>(envelope.as_ref().map(|e| e.nonce().as_slice()))
                    .bind::<Nullable<Binary>, _>(envelope.as_ref().map(ExternalSecretEnvelope::ciphertext))
                    .execute(connection).await?;
            } else if diesel::sql_query(
                dialect.query("DELETE FROM agent_integrations WHERE id=? AND agent_id=?"),
            )
            .bind::<IdentitySql, _>(integration)
            .bind::<IdentitySql, _>(agent)
            .execute(connection)
            .await?
                != 1
            {
                return Err(StoreError::NotFound);
            }
            diesel::sql_query(dialect.query("INSERT INTO agent_integration_receipts (agent_id,request_id,payload_fingerprint,integration_id,action,accepted_at) VALUES (?,?,?,?,?,?)"))
                .bind::<IdentitySql, _>(agent).bind::<Text, _>(request_id)
                .bind::<Binary, _>(fingerprint.as_bytes().as_slice())
                .bind::<IdentitySql, _>(integration).bind::<Text, _>(action.name()).bind::<BigInt, _>(now)
                .execute(connection).await?;
            Ok(AgentIntegrationReceipt {
                id: integration_id,
                action,
                accepted_at: now,
            })
        })
    }

    /// Complete the owner check and capture test credentials before external I/O.
    pub async fn prepare_agent_mcp_test(
        &self,
        actor: &SecretDigest,
        agent_id: &str,
        id: Option<&str>,
        endpoint: McpEndpoint,
        auth: McpAuthEdit,
        now: i64,
    ) -> Result<McpTestTarget, StoreError> {
        let agent = EntityId::from_dto(EntityKind::Agent, agent_id)?;
        let integration = id
            .map(|id| EntityId::from_dto(EntityKind::AgentIntegration, id))
            .transpose()?;
        let stored = operation!(self, |connection, dialect| {
            let agent = own_integration_agent!(connection, dialect, actor, agent, agent_id, now);
            if let Some(integration) = integration {
                Ok(Some(
                    diesel::sql_query(dialect.query(SECRET_ROW))
                        .bind::<IdentitySql, _>(agent)
                        .bind::<IdentitySql, _>(integration)
                        .get_result::<SecretRow>(connection)
                        .await
                        .optional()?
                        .ok_or(StoreError::NotFound)?,
                ))
            } else {
                Ok(None)
            }
        })?;
        let bearer = match auth {
            McpAuthEdit::NoAuth => None,
            McpAuthEdit::ReplaceBearer(value) => Some(value),
            McpAuthEdit::KeepBearer => {
                let id = id.ok_or(StoreError::InvalidInput)?;
                let envelope = stored.ok_or(StoreError::InvalidInput)?.keeper(&endpoint)?;
                Some(McpBearer::from_secret(
                    self.cipher
                        .recover_external(&context(agent_id, id)?, &envelope)?,
                )?)
            }
        };
        Ok(McpTestTarget { endpoint, bearer })
    }
}
