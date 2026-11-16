//! Saved owner instructions and the latest operation-qualified prompt payload.
use super::*;
use chapchoo_domain::{InstructionSource, InstructionSourceKind, ProjectInstructions};

pub const MAX_SYSTEM_PROMPT_BYTES: usize = 16 * 1024;

#[derive(Clone, PartialEq, Eq)]
pub struct ProjectInstructionEvidence {
    pub agent_id: String,
    pub conversation_id: String,
    pub operation_id: String,
    pub execution_id: String,
    pub instructions: ProjectInstructions,
    pub rendered_document: String,
    pub created_at: i64,
    pub tool_catalog: Vec<u8>,
    pub applied_target: Option<ConversationDeliveryTarget>,
    pub applied_at: Option<i64>,
}
impl fmt::Debug for ProjectInstructionEvidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ProjectInstructionEvidence([REDACTED])")
    }
}
#[derive(diesel::QueryableByName)]
struct PromptRow {
    #[diesel(sql_type=Text)]
    content: String,
}
#[derive(diesel::QueryableByName)]
struct ProjectOwner {
    #[diesel(sql_type=IdentitySql)]
    computer_id: EntityId,
}
#[derive(diesel::QueryableByName)]
struct EvidenceRow {
    #[diesel(sql_type=IdentitySql)]
    agent_id: EntityId,
    #[diesel(sql_type=IdentitySql)]
    conversation_id: EntityId,
    #[diesel(sql_type=IdentitySql)]
    operation_id: EntityId,
    #[diesel(sql_type=IdentitySql)]
    execution_id: EntityId,
    #[diesel(sql_type=Text)]
    sources_json: String,
    #[diesel(sql_type=Text)]
    rendered_document: String,
    #[diesel(sql_type=BigInt)]
    created_at: i64,
    #[diesel(sql_type=Binary)]
    tool_catalog: Vec<u8>,
    #[diesel(sql_type=Nullable<Text>)]
    applied_snapshot: Option<String>,
    #[diesel(sql_type=Nullable<Text>)]
    applied_native_session: Option<String>,
    #[diesel(sql_type=Nullable<BigInt>)]
    applied_at: Option<i64>,
}
impl EvidenceRow {
    fn evidence(self) -> Result<ProjectInstructionEvidence, StoreError> {
        let instructions: ProjectInstructions =
            serde_json::from_str(&self.sources_json).map_err(|_| StoreError::Integrity)?;
        instructions.validate().map_err(|_| StoreError::Integrity)?;
        if instructions.document() != self.rendered_document {
            return Err(StoreError::Integrity);
        }
        let applied_target = match (
            self.applied_snapshot,
            self.applied_native_session,
            self.applied_at,
        ) {
            (None, None, None) => None,
            (Some(snapshot), Some(native), Some(_)) => Some(ConversationDeliveryTarget {
                snapshot: SnapshotId::parse(snapshot).map_err(|_| StoreError::Integrity)?,
                native_session: NativeConversationId::parse(native)
                    .map_err(|_| StoreError::Integrity)?,
            }),
            _ => return Err(StoreError::Integrity),
        };
        Ok(ProjectInstructionEvidence {
            agent_id: self.agent_id.to_dto(EntityKind::Agent),
            conversation_id: self.conversation_id.to_dto(EntityKind::Conversation),
            operation_id: self.operation_id.to_dto(EntityKind::ConversationOperation),
            execution_id: self.execution_id.to_dto(EntityKind::Execution),
            instructions,
            rendered_document: self.rendered_document,
            created_at: self.created_at,
            tool_catalog: self.tool_catalog,
            applied_target,
            applied_at: self.applied_at,
        })
    }
}
#[derive(diesel::QueryableByName)]
struct PreparationExecution {
    #[diesel(sql_type=IdentitySql)]
    execution_id: EntityId,
}
const EVIDENCE_SELECT: &str = "SELECT agent_id,conversation_id,operation_id,execution_id,sources_json,rendered_document,created_at,tool_catalog,applied_snapshot,applied_native_session,applied_at FROM project_instruction_evidence WHERE conversation_id=? AND operation_id=?";
macro_rules! evidence {
    ($connection:expr,$dialect:expr,$conversation:expr,$operation:expr) => {
        diesel::sql_query($dialect.query(EVIDENCE_SELECT))
            .bind::<IdentitySql, _>($conversation)
            .bind::<IdentitySql, _>($operation)
            .get_result::<EvidenceRow>($connection)
            .await
            .optional()?
    };
}
macro_rules! preparation_execution {
    ($connection:expr,$dialect:expr,$conversation:expr,$operation:expr) => {
        diesel::sql_query($dialect.query("SELECT execution_id FROM conversation_startups WHERE conversation_id=? AND operation_id=? UNION ALL SELECT p.execution_id FROM conversation_input_preparation p JOIN conversation_inputs i ON i.id=p.operation_id WHERE i.conversation_id=? AND p.operation_id=?"))
            .bind::<IdentitySql,_>($conversation).bind::<IdentitySql,_>($operation)
            .bind::<IdentitySql,_>($conversation).bind::<IdentitySql,_>($operation)
            .get_result::<PreparationExecution>($connection).await.optional()?
            .ok_or(StoreError::InvalidTransition)?
    };
}

#[derive(diesel::QueryableByName)]
pub(super) struct SelectedPreparation {
    #[diesel(sql_type=IdentitySql)]
    pub(super) operation_id: EntityId,
    #[diesel(sql_type=Nullable<IdentitySql>)]
    pub(super) execution_id: Option<EntityId>,
    #[diesel(sql_type=Text)]
    pub(super) preparation_state: String,
}
macro_rules! selected_preparation {
    ($connection:expr,$dialect:expr,$conversation:expr) => {
        diesel::sql_query($dialect.query(
            "SELECT c.current_preparation_operation_id AS operation_id,COALESCE(s.execution_id,p.execution_id,e.execution_id) AS execution_id,COALESCE(s.preparation_state,p.preparation_state,CASE WHEN e.applied_at IS NOT NULL THEN 'READY' ELSE 'FAILED' END) AS preparation_state FROM conversations c LEFT JOIN conversation_startups s ON s.operation_id=c.current_preparation_operation_id LEFT JOIN conversation_input_preparation p ON p.operation_id=c.current_preparation_operation_id LEFT JOIN project_instruction_evidence e ON e.operation_id=c.current_preparation_operation_id AND e.conversation_id=c.id WHERE c.id=? AND c.current_preparation_operation_id IS NOT NULL",
        ))
        .bind::<IdentitySql,_>($conversation)
        .get_result::<$crate::conversations::system_prompts::SelectedPreparation>($connection)
        .await.optional()?
    };
}
pub(super) use selected_preparation;

impl Store {
    /// Settle only a reserved preparation that could not obtain native setup
    /// material. Persisted evidence, including a lost seal acknowledgement,
    /// prevents this positive before-native fence from changing the outcome.
    pub async fn reject_unsealed_preparation(
        &self,
        expected: &crate::ResolvedRoute,
        draft: &ConversationInputPreparation<'_>,
    ) -> Result<bool, StoreError> {
        let agent = EntityId::from_dto(EntityKind::Agent, draft.agent_id)?;
        let conversation = EntityId::from_dto(EntityKind::Conversation, draft.conversation_id)?;
        let operation = EntityId::from_dto(EntityKind::ConversationOperation, draft.operation_id)?;
        operation!(self, |connection, dialect| {
            let team_sql = match dialect {
                Dialect::Postgres => {
                    "SELECT t.id FROM teams t JOIN agents a ON a.team_id=t.id JOIN memberships m ON m.team_id=t.id AND m.user_id=a.owner_user_id WHERE a.id=? FOR NO KEY UPDATE OF t"
                }
                Dialect::Sqlite => {
                    "SELECT t.id FROM teams t JOIN agents a ON a.team_id=t.id JOIN memberships m ON m.team_id=t.id AND m.user_id=a.owner_user_id WHERE a.id=?"
                }
            };
            diesel::sql_query(dialect.query(team_sql))
                .bind::<IdentitySql, _>(agent)
                .get_result::<authority::LockedTeam>(connection)
                .await
                .optional()?
                .ok_or(StoreError::NotFound)?;
            let owner = diesel::sql_query(dialect.query("SELECT a.computer_id FROM agents a JOIN conversations c ON c.agent_id=a.id JOIN memberships m ON m.team_id=a.team_id AND m.user_id=a.owner_user_id WHERE a.id=? AND c.id=?"))
                .bind::<IdentitySql,_>(agent).bind::<IdentitySql,_>(conversation)
                .get_result::<ProjectOwner>(connection).await.optional()?.ok_or(StoreError::NotFound)?;
            let (route, _) =
                delivery::dispatch_route!(connection, dialect, expected, owner.computer_id);
            let selected = selected_preparation!(connection, dialect, conversation)
                .ok_or(StoreError::InvalidTransition)?;
            if selected.operation_id != operation
                || selected.execution_id != Some(route._execution_id)
            {
                return Err(StoreError::InvalidTransition);
            }
            if selected.preparation_state != "POSSIBLE"
                || evidence!(connection, dialect, conversation, operation).is_some()
            {
                return Ok(false);
            }
            let input = diesel::sql_query(dialect.query("UPDATE conversation_input_preparation SET preparation_state='FAILED',failure_kind='BEFORE_NATIVE_LAUNCH' WHERE operation_id=? AND execution_id=? AND preparation_state='POSSIBLE'"))
                .bind::<IdentitySql,_>(operation).bind::<IdentitySql,_>(route._execution_id)
                .execute(connection).await?;
            let startup = diesel::sql_query(dialect.query("UPDATE conversation_startups SET preparation_state='FAILED',failure_kind='BEFORE_NATIVE_LAUNCH' WHERE operation_id=? AND execution_id=? AND preparation_state='POSSIBLE'"))
                .bind::<IdentitySql,_>(operation).bind::<IdentitySql,_>(route._execution_id)
                .execute(connection).await?;
            if input + startup != 1 {
                return Err(StoreError::Integrity);
            }
            if input == 1 {
                diesel::sql_query(
                    dialect.query("UPDATE conversation_inputs SET queue_state='ERROR' WHERE id=?"),
                )
                .bind::<IdentitySql, _>(operation)
                .execute(connection)
                .await?;
            }
            Ok(true)
        })
    }

    pub async fn system_prompt(
        &self,
        actor: &SecretDigest,
        agent_id: &str,
        now: i64,
    ) -> Result<String, StoreError> {
        self.system_prompt_for(
            super::authority::OwnerAuthority::Credential { actor, now },
            agent_id,
        )
        .await?
        .ok_or(StoreError::Integrity)
    }
    pub async fn queued_system_prompt(
        &self,
        authority: &ConversationInputAuthority,
    ) -> Result<Option<String>, StoreError> {
        self.system_prompt_for(
            super::authority::OwnerAuthority::Queued(authority),
            &authority.agent_id,
        )
        .await
    }
    async fn system_prompt_for(
        &self,
        authority: super::authority::OwnerAuthority<'_>,
        agent_id: &str,
    ) -> Result<Option<String>, StoreError> {
        let agent = EntityId::from_dto(EntityKind::Agent, agent_id)?;
        operation!(self, |connection, dialect| {
            let Some((_owner, _now)) =
                super::authority::owned_authority!(connection, dialect, &authority, agent, None)
            else {
                return Ok(None);
            };
            Ok(Some(
                diesel::sql_query(
                    dialect.query("SELECT content FROM agent_system_prompts WHERE agent_id=?"),
                )
                .bind::<IdentitySql, _>(agent)
                .get_result::<PromptRow>(connection)
                .await
                .optional()?
                .map_or_else(String::new, |row| row.content),
            ))
        })
    }
    pub async fn save_system_prompt(
        &self,
        actor: &SecretDigest,
        agent_id: &str,
        content: &str,
        now: i64,
    ) -> Result<String, StoreError> {
        if content.len() > MAX_SYSTEM_PROMPT_BYTES || content.contains('\0') {
            return Err(StoreError::InvalidInput);
        }
        let agent = EntityId::from_dto(EntityKind::Agent, agent_id)?;
        operation!(self, |connection, dialect| {
            let owner = owned_agent!(connection, dialect, actor, agent, now);
            if owner.desired_state == "DELETING" {
                return Err(StoreError::InvalidTransition);
            }
            diesel::sql_query(dialect.query("INSERT INTO agent_system_prompts(agent_id,content) VALUES(?,?) ON CONFLICT(agent_id) DO UPDATE SET content=excluded.content"))
                .bind::<IdentitySql,_>(agent).bind::<Text,_>(content).execute(connection).await?;
            Ok(content.to_owned())
        })
    }
    pub async fn project_instructions_for_startup(
        &self,
        expected: &crate::ResolvedRoute,
        agent_id: &str,
        conversation_id: &str,
    ) -> Result<ProjectInstructions, StoreError> {
        let agent = EntityId::from_dto(EntityKind::Agent, agent_id)?;
        let conversation = EntityId::from_dto(EntityKind::Conversation, conversation_id)?;
        operation!(self, |connection, dialect| {
            let owner = diesel::sql_query(dialect.query("SELECT a.computer_id FROM agents a JOIN conversations c ON c.agent_id=a.id JOIN memberships m ON m.team_id=a.team_id AND m.user_id=a.owner_user_id WHERE a.id=? AND c.id=?"))
                .bind::<IdentitySql,_>(agent).bind::<IdentitySql,_>(conversation)
                .get_result::<ProjectOwner>(connection).await.optional()?.ok_or(StoreError::NotFound)?;
            let _ = delivery::dispatch_route!(connection, dialect, expected, owner.computer_id);
            let content = diesel::sql_query(
                dialect.query("SELECT content FROM agent_system_prompts WHERE agent_id=?"),
            )
            .bind::<IdentitySql, _>(agent)
            .get_result::<PromptRow>(connection)
            .await
            .optional()?
            .map_or_else(String::new, |row| row.content);
            ProjectInstructions::new(vec![InstructionSource {
                kind: InstructionSourceKind::Owner,
                text: content,
            }])
            .map_err(|_| StoreError::Integrity)
        })
    }
    /// Seal the selected content after the setup grant, before native dispatch.
    pub async fn seal_project_instructions(
        &self,
        actor: &SecretDigest,
        expected: &crate::ResolvedRoute,
        draft: &ConversationInputPreparation<'_>,
        instructions: &ProjectInstructions,
        tool_catalog: &[u8],
        now: i64,
    ) -> Result<ProjectInstructionEvidence, StoreError> {
        self.seal_project_instructions_authorized(
            &authority::OwnerAuthority::Credential { actor, now },
            expected,
            draft,
            instructions,
            tool_catalog,
        )
        .await
    }
    pub async fn seal_queued_project_instructions(
        &self,
        authority: &ConversationInputAuthority,
        expected: &crate::ResolvedRoute,
        draft: &ConversationInputPreparation<'_>,
        instructions: &ProjectInstructions,
        tool_catalog: &[u8],
    ) -> Result<ProjectInstructionEvidence, StoreError> {
        if draft.operation_id != authority.input_id {
            return Err(StoreError::InvalidTransition);
        }
        self.seal_project_instructions_authorized(
            &authority::OwnerAuthority::Queued(authority),
            expected,
            draft,
            instructions,
            tool_catalog,
        )
        .await
    }
    async fn seal_project_instructions_authorized(
        &self,
        authority: &authority::OwnerAuthority<'_>,
        expected: &crate::ResolvedRoute,
        draft: &ConversationInputPreparation<'_>,
        instructions: &ProjectInstructions,
        tool_catalog: &[u8],
    ) -> Result<ProjectInstructionEvidence, StoreError> {
        instructions
            .validate()
            .map_err(|_| StoreError::InvalidInput)?;
        validate_catalog(instructions, tool_catalog)?;
        let agent = EntityId::from_dto(EntityKind::Agent, draft.agent_id)?;
        let conversation = EntityId::from_dto(EntityKind::Conversation, draft.conversation_id)?;
        let operation = EntityId::from_dto(EntityKind::ConversationOperation, draft.operation_id)?;
        let document = instructions.document();
        let serialized =
            serde_json::to_string(instructions).map_err(|_| StoreError::InvalidInput)?;
        let owner_content = owner_content(instructions);
        operation!(self, |connection, dialect| {
            let (owner, now) = authority::owned_authority!(
                connection,
                dialect,
                authority,
                agent,
                Some(conversation)
            )
            .ok_or(StoreError::InvalidTransition)?;
            let (route, _) =
                delivery::dispatch_route!(connection, dialect, expected, owner.computer_id);
            let row = conversation!(connection, dialect, conversation);
            if row.agent_id != agent {
                return Err(StoreError::NotFound);
            }
            let now = match authority {
                authority::OwnerAuthority::Credential { .. } => now,
                authority::OwnerAuthority::Queued(bound) => {
                    super::input_queue::require_consumer!(
                        connection,
                        dialect,
                        operation,
                        &bound.claim_id
                    );
                    crate::directory::database_now!(connection, dialect)
                }
            };
            let selected = preparation_execution!(connection, dialect, conversation, operation);
            if selected.execution_id != route._execution_id {
                return Err(StoreError::Conflict);
            }
            let current = selected_preparation!(connection, dialect, conversation)
                .ok_or(StoreError::InvalidTransition)?;
            if current.operation_id != operation {
                return Err(StoreError::InvalidTransition);
            }
            if let Some(previous) = evidence!(connection, dialect, conversation, operation) {
                if previous.agent_id != agent
                    || previous.execution_id != route._execution_id
                    || previous.sources_json != serialized
                    || previous.rendered_document != document
                    || previous.tool_catalog != tool_catalog
                {
                    return Err(StoreError::Conflict);
                }
                return previous.evidence();
            }
            let current = selected_preparation!(connection, dialect, conversation)
                .ok_or(StoreError::InvalidTransition)?;
            if current.operation_id != operation || current.preparation_state != "POSSIBLE" {
                return Err(StoreError::InvalidTransition);
            }
            let saved = diesel::sql_query(
                dialect.query("SELECT content FROM agent_system_prompts WHERE agent_id=?"),
            )
            .bind::<IdentitySql, _>(agent)
            .get_result::<PromptRow>(connection)
            .await
            .optional()?
            .map_or_else(String::new, |saved| saved.content);
            if saved != owner_content {
                return Err(StoreError::InvalidTransition);
            }
            diesel::sql_query(dialect.query("INSERT INTO project_instruction_evidence(operation_id,agent_id,conversation_id,execution_id,sources_json,rendered_document,owner_content,tool_catalog,created_at) VALUES(?,?,?,?,?,?,?,?,?)"))
                .bind::<IdentitySql,_>(operation).bind::<IdentitySql,_>(agent).bind::<IdentitySql,_>(conversation)
                .bind::<IdentitySql,_>(route._execution_id).bind::<Text,_>(&serialized).bind::<Text,_>(&document)
                .bind::<Text,_>(&owner_content).bind::<Binary,_>(tool_catalog)
                .bind::<BigInt,_>(now).execute(connection).await?;
            evidence!(connection, dialect, conversation, operation)
                .ok_or(StoreError::Integrity)?
                .evidence()
        })
    }
    /// Seal a route-owned startup selection before native dispatch.
    pub async fn seal_startup_project_instructions(
        &self,
        expected: &crate::ResolvedRoute,
        draft: &ConversationInputPreparation<'_>,
        instructions: &ProjectInstructions,
        tool_catalog: &[u8],
        now: i64,
    ) -> Result<ProjectInstructionEvidence, StoreError> {
        instructions
            .validate()
            .map_err(|_| StoreError::InvalidInput)?;
        validate_catalog(instructions, tool_catalog)?;
        let agent = EntityId::from_dto(EntityKind::Agent, draft.agent_id)?;
        let conversation = EntityId::from_dto(EntityKind::Conversation, draft.conversation_id)?;
        let operation = EntityId::from_dto(EntityKind::ConversationOperation, draft.operation_id)?;
        let document = instructions.document();
        let serialized =
            serde_json::to_string(instructions).map_err(|_| StoreError::InvalidInput)?;
        let owner_content = owner_content(instructions);
        operation!(self, |connection, dialect| {
            let team_sql = match dialect {
                Dialect::Postgres => {
                    "SELECT t.id FROM teams t JOIN agents a ON a.team_id=t.id JOIN memberships m ON m.team_id=t.id AND m.user_id=a.owner_user_id WHERE a.id=? FOR NO KEY UPDATE OF t"
                }
                Dialect::Sqlite => {
                    "SELECT t.id FROM teams t JOIN agents a ON a.team_id=t.id JOIN memberships m ON m.team_id=t.id AND m.user_id=a.owner_user_id WHERE a.id=?"
                }
            };
            diesel::sql_query(dialect.query(team_sql))
                .bind::<IdentitySql, _>(agent)
                .get_result::<authority::LockedTeam>(connection)
                .await
                .optional()?
                .ok_or(StoreError::NotFound)?;
            let owner = diesel::sql_query(dialect.query("SELECT a.computer_id FROM agents a JOIN conversations c ON c.agent_id=a.id JOIN memberships m ON m.team_id=a.team_id AND m.user_id=a.owner_user_id WHERE a.id=? AND c.id=?"))
                .bind::<IdentitySql,_>(agent).bind::<IdentitySql,_>(conversation)
                .get_result::<ProjectOwner>(connection).await.optional()?.ok_or(StoreError::NotFound)?;
            let (route, _) =
                delivery::dispatch_route!(connection, dialect, expected, owner.computer_id);
            let row = conversation!(connection, dialect, conversation);
            if row.agent_id != agent {
                return Err(StoreError::NotFound);
            }
            let selected = preparation_execution!(connection, dialect, conversation, operation);
            if selected.execution_id != route._execution_id {
                return Err(StoreError::Conflict);
            }
            let current = selected_preparation!(connection, dialect, conversation)
                .ok_or(StoreError::InvalidTransition)?;
            if current.operation_id != operation {
                return Err(StoreError::InvalidTransition);
            }
            if let Some(previous) = evidence!(connection, dialect, conversation, operation) {
                if previous.agent_id != agent
                    || previous.execution_id != route._execution_id
                    || previous.sources_json != serialized
                    || previous.rendered_document != document
                    || previous.tool_catalog != tool_catalog
                {
                    return Err(StoreError::Conflict);
                }
                return previous.evidence();
            }
            let current = selected_preparation!(connection, dialect, conversation)
                .ok_or(StoreError::InvalidTransition)?;
            if current.operation_id != operation || current.preparation_state != "POSSIBLE" {
                return Err(StoreError::InvalidTransition);
            }
            let saved = diesel::sql_query(
                dialect.query("SELECT content FROM agent_system_prompts WHERE agent_id=?"),
            )
            .bind::<IdentitySql, _>(agent)
            .get_result::<PromptRow>(connection)
            .await
            .optional()?
            .map_or_else(String::new, |saved| saved.content);
            if saved != owner_content {
                return Err(StoreError::InvalidTransition);
            }
            diesel::sql_query(dialect.query("INSERT INTO project_instruction_evidence(operation_id,agent_id,conversation_id,execution_id,sources_json,rendered_document,owner_content,tool_catalog,created_at) VALUES(?,?,?,?,?,?,?,?,?)"))
                .bind::<IdentitySql,_>(operation).bind::<IdentitySql,_>(agent).bind::<IdentitySql,_>(conversation)
                .bind::<IdentitySql,_>(route._execution_id).bind::<Text,_>(&serialized).bind::<Text,_>(&document)
                .bind::<Text,_>(&owner_content).bind::<Binary,_>(tool_catalog)
                .bind::<BigInt,_>(now).execute(connection).await?;
            evidence!(connection, dialect, conversation, operation)
                .ok_or(StoreError::Integrity)?
                .evidence()
        })
    }
    pub async fn latest_system_prompt(
        &self,
        actor: &SecretDigest,
        agent_id: &str,
        now: i64,
    ) -> Result<Option<ProjectInstructionEvidence>, StoreError> {
        self.latest_system_prompt_authorized(
            &authority::OwnerAuthority::Credential { actor, now },
            agent_id,
        )
        .await
    }
    pub async fn queued_latest_system_prompt(
        &self,
        authority: &ConversationInputAuthority,
    ) -> Result<Option<ProjectInstructionEvidence>, StoreError> {
        self.latest_system_prompt_authorized(
            &authority::OwnerAuthority::Queued(authority),
            &authority.agent_id,
        )
        .await
    }
    async fn latest_system_prompt_authorized(
        &self,
        authority: &authority::OwnerAuthority<'_>,
        agent_id: &str,
    ) -> Result<Option<ProjectInstructionEvidence>, StoreError> {
        let agent = EntityId::from_dto(EntityKind::Agent, agent_id)?;
        operation!(self, |connection, dialect| {
            let _ = authority::owned_authority!(connection, dialect, authority, agent, None)
                .ok_or(StoreError::InvalidTransition)?;
            diesel::sql_query(dialect.query("SELECT e.agent_id,e.conversation_id,e.operation_id,e.execution_id,e.sources_json,e.rendered_document,e.created_at,e.tool_catalog,e.applied_snapshot,e.applied_native_session,e.applied_at FROM agent_conversations a JOIN conversations c ON c.id=a.conversation_id AND c.agent_id=a.agent_id JOIN project_instruction_evidence e ON e.agent_id=a.agent_id AND e.conversation_id=c.id AND e.operation_id=c.current_preparation_operation_id WHERE a.agent_id=?"))
                .bind::<IdentitySql,_>(agent).get_result::<EvidenceRow>(connection).await.optional()?
                .map(EvidenceRow::evidence).transpose()
        })
    }
    /// Component reads are source/execution-qualified and never create evidence.
    pub async fn project_instructions(
        &self,
        source: &crate::ConnectorSource<'_>,
        agent_id: &str,
        conversation_id: &str,
        operation_id: &str,
    ) -> Result<ProjectInstructionEvidence, StoreError> {
        let agent = EntityId::from_dto(EntityKind::Agent, agent_id)?;
        let conversation = EntityId::from_dto(EntityKind::Conversation, conversation_id)?;
        let operation = EntityId::from_dto(EntityKind::ConversationOperation, operation_id)?;
        operation!(self, |connection, dialect| {
            let row = records::source_conversation!(connection, dialect, source, conversation);
            let route = connector!(connection, dialect, source);
            if row.agent_id != agent {
                return Err(StoreError::NotFound);
            }
            let grant = preparation_execution!(connection, dialect, conversation, operation);
            if grant.execution_id != route._execution_id {
                return Err(StoreError::Conflict);
            }
            let current = selected_preparation!(connection, dialect, conversation)
                .ok_or(StoreError::InvalidTransition)?;
            if current.operation_id != operation
                || current.execution_id != Some(route._execution_id)
            {
                return Err(StoreError::InvalidTransition);
            }
            let evidence = evidence!(connection, dialect, conversation, operation)
                .ok_or(StoreError::NotFound)?;
            if evidence.execution_id != route._execution_id || evidence.agent_id != agent {
                return Err(StoreError::Conflict);
            }
            evidence.evidence()
        })
    }
}

pub(super) fn owner_content(instructions: &ProjectInstructions) -> String {
    instructions
        .sources
        .iter()
        .filter(|source| source.kind == InstructionSourceKind::Owner)
        .map(|source| source.text.as_str())
        .collect::<Vec<_>>()
        .join("\n\n")
}
pub(super) fn validate_catalog(
    instructions: &ProjectInstructions,
    catalog: &[u8],
) -> Result<(), StoreError> {
    let source_bytes = instructions
        .sources
        .iter()
        .map(|source| source.text.len() + 16)
        .sum::<usize>();
    if source_bytes + instructions.document().len() + catalog.len() > 48 * 1024 {
        return Err(StoreError::InvalidInput);
    }
    #[derive(serde::Deserialize)]
    struct Catalog {
        tools: Vec<serde_json::Map<String, serde_json::Value>>,
    }
    let decoded: Catalog = serde_json::from_slice(catalog).map_err(|_| StoreError::InvalidInput)?;
    if decoded.tools.len() > 32 {
        return Err(StoreError::InvalidInput);
    }
    Ok(())
}
