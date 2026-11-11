//! Actual-start admission and exact applied-record reads, separate from saved selection.
use super::*;
use crate::{
    ConnectorSource, ConversationInputPreparation, Store, conversations::authority::connector,
    store::operation,
};
use chapchoo_crypto::{ExternalSecret, ExternalSecretEnvelope};
use diesel::sql_types::Bool;

#[derive(Clone, PartialEq, Eq)]
pub struct AppliedExecutionAuthentication {
    pub operation_id: String,
    pub execution_id: String,
    pub agent_id: String,
    pub conversation_id: String,
    pub authentication_id: String,
    pub selection_request_id: String,
    pub admitted_at: i64,
}
/// Server-internal material; ChatGPT bundles still contain refresh authority.
/// The execution delivery adapter must convert them before any wire handoff.
pub struct ExecutionAuthenticationMaterial {
    pub applied: AppliedExecutionAuthentication,
    pub source: AuthenticationCredentialSource,
    pub kind: AuthenticationKind,
    pub account_id: Option<String>,
    pub expires_at: Option<i64>,
    pub secret: ExternalSecret,
}
macro_rules! safe_debug {($($name:ty),+)=>{$(impl fmt::Debug for $name {
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result {f.write_str(concat!(stringify!($name),"([REDACTED])"))}
})+};}
safe_debug!(
    AppliedExecutionAuthentication,
    ExecutionAuthenticationMaterial
);

#[derive(diesel::QueryableByName)]
struct AgentContext {
    #[diesel(sql_type=IdentitySql)]
    owner_user_id: EntityId,
    #[diesel(sql_type=IdentitySql)]
    team_id: EntityId,
    #[diesel(sql_type=IdentitySql)]
    computer_id: EntityId,
}
struct Context {
    owner_user_id: EntityId,
    team_id: EntityId,
    computer_id: EntityId,
    execution_id: EntityId,
}
#[derive(diesel::QueryableByName)]
struct Execution {
    #[diesel(sql_type=IdentitySql)]
    execution_id: EntityId,
}
#[derive(diesel::QueryableByName)]
struct Preparation {
    #[diesel(sql_type=Text)]
    preparation_state: String,
}
const CONTEXT: &str = "SELECT a.owner_user_id,a.team_id,a.computer_id FROM agents a JOIN teams t ON t.id=a.team_id JOIN memberships m ON m.team_id=t.id AND m.user_id=a.owner_user_id JOIN agent_conversations ac ON ac.agent_id=a.id JOIN conversations c ON c.id=ac.conversation_id AND c.agent_id=a.id WHERE a.id=? AND c.id=? AND t.deletion_requested_at IS NULL";
fn ids(
    draft: &ConversationInputPreparation<'_>,
) -> Result<(EntityId, EntityId, EntityId), StoreError> {
    Ok((
        EntityId::from_dto(EntityKind::Agent, draft.agent_id)?,
        EntityId::from_dto(EntityKind::Conversation, draft.conversation_id)?,
        EntityId::from_dto(EntityKind::ConversationOperation, draft.operation_id)?,
    ))
}
macro_rules! context {
    ($connection:expr,$dialect:expr,$source:expr,$ids:expr) => {{
        let (agent, conversation, _) = $ids;
        // Discover under Team authority first. A later record lock must precede
        // Computer admission in the catalogue -> Provider -> Computer order.
        let query = match $dialect {
            Dialect::Postgres => format!("{CONTEXT} FOR NO KEY UPDATE OF t"),
            Dialect::Sqlite => CONTEXT.to_owned(),
        };
        let initial = diesel::sql_query($dialect.query(&query))
            .bind::<IdentitySql, _>(agent)
            .bind::<IdentitySql, _>(conversation)
            .get_result::<AgentContext>($connection)
            .await
            .optional()?
            .ok_or(StoreError::NotFound)?;
        let execution = diesel::sql_query($dialect.query("SELECT e.id AS execution_id FROM connector_sessions s JOIN computer_executions e ON e.id=s.execution_id JOIN computers c ON c.id=e.computer_id AND c.current_execution_id=e.id WHERE s.id=? AND c.id=? AND e.retired_at IS NULL"))
            .bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::ConnectorSession,$source.session_id)?)
            .bind::<IdentitySql,_>(initial.computer_id).get_result::<Execution>($connection)
            .await.optional()?.ok_or(StoreError::Unauthenticated)?;
        Context {
            owner_user_id: initial.owner_user_id,
            team_id: initial.team_id,
            computer_id: initial.computer_id,
            execution_id: execution.execution_id,
        }
    }};
}
macro_rules! authorize {
    ($connection:expr,$dialect:expr,$source:expr,$context:expr,$ids:expr) => {{
        let (agent, conversation, _) = $ids;
        let route = connector!($connection, $dialect, $source);
        let current = diesel::sql_query($dialect.query(CONTEXT))
            .bind::<IdentitySql, _>(agent)
            .bind::<IdentitySql, _>(conversation)
            .get_result::<AgentContext>($connection)
            .await
            .optional()?
            .ok_or(StoreError::NotFound)?;
        if current.team_id != $context.team_id
            || current.owner_user_id != $context.owner_user_id
            || current.computer_id != route.computer_id
            || current.computer_id != $context.computer_id
            || route._execution_id != $context.execution_id
        {
            return Err(StoreError::Conflict);
        }
    }};
}
#[derive(diesel::QueryableByName)]
struct AppliedRow {
    #[diesel(sql_type=IdentitySql)]
    operation_id: EntityId,
    #[diesel(sql_type=IdentitySql)]
    execution_id: EntityId,
    #[diesel(sql_type=IdentitySql)]
    agent_id: EntityId,
    #[diesel(sql_type=IdentitySql)]
    conversation_id: EntityId,
    #[diesel(sql_type=IdentitySql)]
    authentication_id: EntityId,
    #[diesel(sql_type=Text)]
    selection_request_id: String,
    #[diesel(sql_type=Bool)]
    is_current: bool,
    #[diesel(sql_type=BigInt)]
    admitted_at: i64,
}
impl AppliedRow {
    fn summary(&self) -> AppliedExecutionAuthentication {
        AppliedExecutionAuthentication {
            operation_id: self.operation_id.to_dto(EntityKind::ConversationOperation),
            execution_id: self.execution_id.to_dto(EntityKind::Execution),
            agent_id: self.agent_id.to_dto(EntityKind::Agent),
            conversation_id: self.conversation_id.to_dto(EntityKind::Conversation),
            authentication_id: self.authentication_id.to_dto(EntityKind::Authentication),
            selection_request_id: self.selection_request_id.clone(),
            admitted_at: self.admitted_at,
        }
    }
}
#[derive(diesel::QueryableByName)]
struct Selection {
    #[diesel(sql_type=IdentitySql)]
    authentication_id: EntityId,
    #[diesel(sql_type=Text)]
    current_request_id: String,
}
#[derive(diesel::QueryableByName)]
struct SecretRow {
    #[diesel(sql_type=Binary)]
    nonce: Vec<u8>,
    #[diesel(sql_type=Binary)]
    ciphertext: Vec<u8>,
}
macro_rules! current {
    ($connection:expr,$dialect:expr,$execution:expr) => {{
        diesel::sql_query($dialect.query("SELECT * FROM execution_authentication_applications WHERE execution_id=? AND is_current=TRUE"))
            .bind::<IdentitySql,_>($execution).get_result::<AppliedRow>($connection).await.optional()?
    }};
}
macro_rules! record {
    ($connection:expr,$dialect:expr,$context:expr,$authentication:expr) => {{
        let query = match $dialect {
            Dialect::Postgres => format!("{AUTH_COLUMNS} WHERE id=? FOR UPDATE"),
            Dialect::Sqlite => format!("{AUTH_COLUMNS} WHERE id=?"),
        };
        let row = diesel::sql_query($dialect.query(&query))
            .bind::<IdentitySql, _>($authentication)
            .get_result::<AuthenticationRow>($connection)
            .await
            .optional()?
            .ok_or(StoreError::NotFound)?;
        if row.owner() != Owner::personal($context.owner_user_id)
            && row.owner() != Owner::team($context.team_id)
        {
            return Err(StoreError::NotFound);
        }
        row
    }};
}
macro_rules! eligible_record {
    ($connection:expr,$dialect:expr,$source:expr,$context:expr,$ids:expr) => {{
        let (agent, conversation, operation) = $ids;
        let applied =
            current!($connection, $dialect, $context.execution_id).ok_or(StoreError::NotFound)?;
        if applied.operation_id != operation
            || applied.agent_id != agent
            || applied.conversation_id != conversation
        {
            return Err(StoreError::Conflict);
        }
        let row = record!($connection, $dialect, $context, applied.authentication_id);
        authorize!($connection, $dialect, $source, $context, $ids);
        let latest =
            current!($connection, $dialect, $context.execution_id).ok_or(StoreError::Conflict)?;
        if latest.operation_id != applied.operation_id
            || latest.authentication_id != applied.authentication_id
        {
            return Err(StoreError::Conflict);
        }
        (applied, row)
    }};
}
impl Store {
    /// A genuine execution consumer shares the record/source refresh attempt.
    /// No account session or synthetic owner credential substitutes for its route.
    pub async fn begin_execution_authentication_refresh(
        &self,
        source: &ConnectorSource<'_>,
        draft: &ConversationInputPreparation<'_>,
        now: i64,
    ) -> Result<AuthenticationRefreshAdmission, StoreError> {
        let ids = ids(draft)?;
        operation!(self, |connection, dialect| {
            let context = context!(connection, dialect, source, ids);
            let (_, row) = eligible_record!(connection, dialect, source, context, ids);
            super::refresh::begin_record_refresh!(self, connection, dialect, row, now)
        })
    }
    pub async fn complete_execution_authentication_refresh(
        &self,
        source: &ConnectorSource<'_>,
        draft: &ConversationInputPreparation<'_>,
        step: &AuthenticationRefreshStep,
        tokens: &DeviceAuthenticationTokens,
        now: i64,
    ) -> Result<AuthenticationCredentialSource, StoreError> {
        let ids = ids(draft)?;
        let protected = self.protect_refresh_result(step, tokens)?;
        operation!(self, |connection, dialect| {
            let context = context!(connection, dialect, source, ids);
            let (_, row) = eligible_record!(connection, dialect, source, context, ids);
            super::refresh::complete_record_refresh!(
                connection, dialect, row, step, &protected, tokens, now
            )
        })
    }
    /// Read current applied provenance without selecting, refreshing or starting.
    /// A deleted record's retained identity does not authorize another record.
    pub async fn execution_authentication_binding(
        &self,
        source: &ConnectorSource<'_>,
        draft: &ConversationInputPreparation<'_>,
    ) -> Result<Option<AppliedExecutionAuthentication>, StoreError> {
        let ids = ids(draft)?;
        operation!(self, |connection, dialect| {
            let context = context!(connection, dialect, source, ids);
            authorize!(connection, dialect, source, context, ids);
            let current = current!(connection, dialect, context.execution_id);
            if current
                .as_ref()
                .is_some_and(|row| row.agent_id != ids.0 || row.conversation_id != ids.1)
            {
                return Err(StoreError::Conflict);
            }
            Ok(current.map(|row| row.summary()))
        })
    }
    /// Invoke only at an actual native launch, not at every Setup receipt.
    /// The exact setup operation seals one selection. A superseded application
    /// cannot reclaim current authority by replaying its old operation.
    pub async fn begin_execution_authentication(
        &self,
        source: &ConnectorSource<'_>,
        draft: &ConversationInputPreparation<'_>,
        expected_previous_operation: Option<&str>,
    ) -> Result<AppliedExecutionAuthentication, StoreError> {
        let ids = ids(draft)?;
        let previous = expected_previous_operation
            .map(|id| EntityId::from_dto(EntityKind::ConversationOperation, id))
            .transpose()?;
        operation!(self, |connection, dialect| {
            let context = context!(connection, dialect, source, ids);
            let preparation = diesel::sql_query(dialect.query("SELECT preparation_state FROM (SELECT p.preparation_state FROM conversation_input_preparation p JOIN conversation_inputs i ON i.id=p.operation_id WHERE p.operation_id=? AND p.execution_id=? AND i.conversation_id=? UNION ALL SELECT preparation_state FROM conversation_startups WHERE operation_id=? AND execution_id=? AND conversation_id=?) attempts"))
                .bind::<IdentitySql,_>(ids.2).bind::<IdentitySql,_>(context.execution_id).bind::<IdentitySql,_>(ids.1)
                .bind::<IdentitySql,_>(ids.2).bind::<IdentitySql,_>(context.execution_id).bind::<IdentitySql,_>(ids.1)
                .load::<Preparation>(connection).await?;
            if preparation.len() > 1 {
                return Err(StoreError::Integrity);
            }
            let preparation = preparation.into_iter().next().ok_or(StoreError::NotFound)?;
            if preparation.preparation_state != "POSSIBLE" {
                return Err(StoreError::InvalidTransition);
            }
            let prior =
                diesel::sql_query(dialect.query(
                    "SELECT * FROM execution_authentication_applications WHERE operation_id=?",
                ))
                .bind::<IdentitySql, _>(ids.2)
                .get_result::<AppliedRow>(connection)
                .await
                .optional()?;
            if let Some(prior) = prior {
                if !prior.is_current
                    || prior.execution_id != context.execution_id
                    || prior.agent_id != ids.0
                    || prior.conversation_id != ids.1
                {
                    return Err(StoreError::Conflict);
                }
                let _ = record!(connection, dialect, context, prior.authentication_id);
                authorize!(connection, dialect, source, context, ids);
                let current = current!(connection, dialect, context.execution_id)
                    .ok_or(StoreError::Conflict)?;
                if current.operation_id != prior.operation_id {
                    return Err(StoreError::Conflict);
                }
                return Ok(prior.summary());
            }
            let selected = diesel::sql_query(dialect.query("SELECT authentication_id,current_request_id FROM agent_authentication_selections WHERE agent_id=?"))
                .bind::<IdentitySql,_>(ids.0).get_result::<Selection>(connection).await.optional()?.ok_or(StoreError::NotFound)?;
            let _ = record!(connection, dialect, context, selected.authentication_id);
            authorize!(connection, dialect, source, context, ids);
            let current = current!(connection, dialect, context.execution_id);
            if current.as_ref().map(|row| row.operation_id) != previous {
                return Err(StoreError::Conflict);
            }
            let now = crate::directory::database_now!(connection, dialect);
            diesel::sql_query(dialect.query("UPDATE execution_authentication_applications SET is_current=FALSE WHERE execution_id=? AND is_current=TRUE"))
                .bind::<IdentitySql,_>(context.execution_id).execute(connection).await?;
            diesel::sql_query(dialect.query("INSERT INTO execution_authentication_applications(operation_id,execution_id,agent_id,conversation_id,authentication_id,selection_request_id,is_current,admitted_at) VALUES(?,?,?,?,?,?,TRUE,?)"))
                .bind::<IdentitySql,_>(ids.2).bind::<IdentitySql,_>(context.execution_id).bind::<IdentitySql,_>(ids.0).bind::<IdentitySql,_>(ids.1)
                .bind::<IdentitySql,_>(selected.authentication_id).bind::<Text,_>(&selected.current_request_id).bind::<BigInt,_>(now).execute(connection).await?;
            Ok(AppliedExecutionAuthentication {
                operation_id: draft.operation_id.into(),
                execution_id: context.execution_id.to_dto(EntityKind::Execution),
                agent_id: draft.agent_id.into(),
                conversation_id: draft.conversation_id.into(),
                authentication_id: selected
                    .authentication_id
                    .to_dto(EntityKind::Authentication),
                selection_request_id: selected.current_request_id,
                admitted_at: now,
            })
        })
    }
    /// Return only the current application record's latest protected source.
    /// Saved selection is deliberately not consulted. A source equality check
    /// is required at later handoff; this result grants no native replay.
    pub async fn execution_authentication_material(
        &self,
        source: &ConnectorSource<'_>,
        draft: &ConversationInputPreparation<'_>,
        expected_source: Option<&AuthenticationCredentialSource>,
    ) -> Result<ExecutionAuthenticationMaterial, StoreError> {
        let ids = ids(draft)?;
        operation!(self, |connection, dialect| {
            let context = context!(connection, dialect, source, ids);
            let (applied, record) = eligible_record!(connection, dialect, source, context, ids);
            let secret = diesel::sql_query(
                dialect.query("SELECT nonce,ciphertext FROM external_authentications WHERE id=?"),
            )
            .bind::<IdentitySql, _>(applied.authentication_id)
            .get_result::<SecretRow>(connection)
            .await?;
            let nonce: [u8; 24] = secret
                .nonce
                .as_slice()
                .try_into()
                .map_err(|_| StoreError::Integrity)?;
            let source = AuthenticationCredentialSource::from_bytes(nonce);
            if expected_source.is_some_and(|expected| expected != &source) {
                return Err(StoreError::Conflict);
            }
            let kind = AuthenticationKind::parse(&record.kind)?;
            let authentication = applied.authentication_id.to_dto(EntityKind::Authentication);
            let secret = self.cipher.recover_external(
                &super::context(
                    &record.owner().key()?,
                    &authentication,
                    match kind {
                        AuthenticationKind::ChatGpt => ExternalSecretKind::ChatGpt,
                        AuthenticationKind::OpenAiApiKey => ExternalSecretKind::OpenAiApiKey,
                    },
                )?,
                &ExternalSecretEnvelope::from_parts(nonce, secret.ciphertext)?,
            )?;
            Ok(ExecutionAuthenticationMaterial {
                applied: applied.summary(),
                source,
                kind,
                account_id: record.account_id,
                expires_at: record.expires_at,
                secret,
            })
        })
    }
}
