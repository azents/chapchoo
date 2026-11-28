//! Configuration changes reuse startup ownership and retain the original native source.
use super::*;
use chapchoo_domain::ProjectInstructions;
use diesel::sql_types::Bool;

#[derive(diesel::QueryableByName)]
struct RefreshOwner {
    #[diesel(sql_type=IdentitySql)]
    computer_id: EntityId,
    #[diesel(sql_type=Text)]
    current_request_id: String,
    #[diesel(sql_type=Text)]
    desired_state: String,
}
#[derive(diesel::QueryableByName)]
struct RefreshHead {
    #[diesel(sql_type=Nullable<Text>)]
    sources_json: Option<String>,
    #[diesel(sql_type=Nullable<Binary>)]
    tool_catalog: Option<Vec<u8>>,
    #[diesel(sql_type=Nullable<IdentitySql>)]
    applied_operation: Option<EntityId>,
}
#[derive(diesel::QueryableByName)]
struct SavedPrompt {
    #[diesel(sql_type=Text)]
    content: String,
}
#[derive(diesel::QueryableByName)]
struct PossibleInputCount {
    #[diesel(sql_type=BigInt)]
    n: i64,
}

#[derive(diesel::QueryableByName)]
pub(super) struct ExternalInput {
    #[diesel(sql_type=IdentitySql)]
    pub(super) id: EntityId,
}

// Select queued demand for the common preparation owner, not a forwarding
// grant. A running turn retains its applied supply.
macro_rules! external_input {
    ($cx:expr, $dialect:expr, $agent:expr, $conversation:expr, $execution:expr, $snapshot:expr, $native:expr) => {{
        diesel::sql_query($dialect.query(
            "SELECT i.id FROM conversation_inputs i JOIN auth_sessions a ON a.id=i.submitting_session_id
             WHERE i.conversation_id=? AND i.queue_state='PENDING' AND a.revoked_at IS NULL
               AND a.expires_at>? AND (i.target_snapshot IS NULL OR i.target_snapshot=?)
               AND NOT EXISTS(SELECT 1 FROM conversation_input_delivery d WHERE d.operation_id=i.id)
               AND EXISTS(SELECT 1 FROM agent_integration_receipts r WHERE r.agent_id=?)
               AND NOT EXISTS(SELECT 1 FROM conversation_native_state n
                 WHERE n.conversation_id=i.conversation_id AND n.execution_id=? AND n.source_snapshot=?
                   AND n.native_session_id=? AND n.foreground<>'IDLE')
               AND NOT EXISTS(SELECT 1 FROM conversation_continuation_inputs q
                 JOIN conversation_ending_decisions d USING(conversation_id,execution_id,source_snapshot,occurrence_id)
                 JOIN execution_authentication_applications aa ON aa.operation_id=d.launch_operation_id AND aa.is_current=TRUE
                 WHERE q.conversation_id=i.conversation_id AND q.execution_id=? AND q.source_snapshot=?
                   AND d.native_session_id=? AND q.delivery_state<>'CLOSED')
             ORDER BY i.accepted_at,i.id LIMIT 1",
        ))
        .bind::<IdentitySql,_>($conversation)
        .bind::<BigInt,_>($crate::directory::database_now!($cx,$dialect))
        .bind::<Text,_>($snapshot)
        .bind::<IdentitySql,_>($agent)
        .bind::<IdentitySql,_>($execution)
        .bind::<Text,_>($snapshot)
        .bind::<Text,_>($native)
        .bind::<IdentitySql,_>($execution)
        .bind::<Text,_>($snapshot)
        .bind::<Text,_>($native)
        .get_result::<$crate::conversations::project_refresh::ExternalInput>($cx)
        .await.optional()?.map(|input| input.id)
    }};
}
pub(super) use external_input;

impl Store {
    /// Admit a new common preparation for changed configuration or an eligible
    /// next input whose external supply has not already been selected.
    pub async fn admit_project_refresh(
        &self,
        expected: &crate::ResolvedRoute,
        agent_id: &str,
        conversation_id: &str,
        instructions: &ProjectInstructions,
        tool_catalog: &[u8],
        now: i64,
    ) -> Result<Option<ConnectionStartupAdmission>, StoreError> {
        instructions
            .validate()
            .map_err(|_| StoreError::InvalidInput)?;
        system_prompts::validate_catalog(instructions, tool_catalog)?;
        let owner_content = system_prompts::owner_content(instructions);
        let rendered = instructions.document();
        let sources = serde_json::to_string(instructions).map_err(|_| StoreError::InvalidInput)?;
        let agent = EntityId::from_dto(EntityKind::Agent, agent_id)?;
        let conversation = EntityId::from_dto(EntityKind::Conversation, conversation_id)?;
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
            let owner = diesel::sql_query(dialect.query("SELECT a.computer_id,c.current_request_id,c.desired_state FROM agents a JOIN computers c ON c.id=a.computer_id JOIN conversations v ON v.agent_id=a.id JOIN agent_conversations ac ON ac.agent_id=a.id AND ac.conversation_id=v.id JOIN memberships m ON m.team_id=a.team_id AND m.user_id=a.owner_user_id WHERE a.id=? AND v.id=?"))
                .bind::<IdentitySql,_>(agent).bind::<IdentitySql,_>(conversation)
                .get_result::<RefreshOwner>(connection).await.optional()?.ok_or(StoreError::NotFound)?;
            let (route, _) =
                delivery::dispatch_route!(connection, dialect, expected, owner.computer_id);
            if !authority::input_ready!(connection, dialect, route.computer_id) {
                return Ok(None);
            }
            let owner = diesel::sql_query(dialect.query("SELECT a.computer_id,c.current_request_id,c.desired_state FROM agents a JOIN computers c ON c.id=a.computer_id WHERE a.id=?"))
                .bind::<IdentitySql,_>(agent).get_result::<RefreshOwner>(connection).await?;
            if owner.desired_state != "RUNNING" {
                return Ok(None);
            }
            let current = conversation!(connection, dialect, conversation);
            let (Some(snapshot), Some(native)) = (
                current.source_snapshot.as_deref(),
                current.native_session_id.as_deref(),
            ) else {
                return Ok(None);
            };
            if current.source_state != "READY" {
                return Ok(None);
            }
            // A forwarding grant owns its native effect before the Connector
            // pulls the body. Queue-attempt errors do not settle that ownership.
            let possible = diesel::sql_query(dialect.query("SELECT CAST(count(*) AS BIGINT) AS n FROM conversation_input_delivery d JOIN conversation_inputs i ON i.id=d.operation_id WHERE i.conversation_id=? AND d.execution_id=? AND d.delivery_state='POSSIBLE'"))
                .bind::<IdentitySql,_>(conversation).bind::<IdentitySql,_>(route._execution_id)
                .get_result::<PossibleInputCount>(connection).await?;
            if possible.n > 0 {
                return Ok(None);
            }
            let saved = diesel::sql_query(
                dialect.query("SELECT content FROM agent_system_prompts WHERE agent_id=?"),
            )
            .bind::<IdentitySql, _>(agent)
            .get_result::<SavedPrompt>(connection)
            .await
            .optional()?
            .map_or_else(String::new, |row| row.content);
            if saved != owner_content {
                return Ok(None);
            }
            let Some(current_preparation) =
                system_prompts::selected_preparation!(connection, dialect, conversation)
            else {
                return Ok(None);
            };
            if current_preparation
                .execution_id
                .is_some_and(|execution| execution != route._execution_id)
                || current_preparation.preparation_state == "POSSIBLE"
            {
                return Ok(None);
            }
            let external_input = external_input!(
                connection,
                dialect,
                agent,
                conversation,
                route._execution_id,
                snapshot,
                native
            );
            let connection_id =
                EntityId::from_dto(EntityKind::Connection, &expected.component.connection_id)?;
            let previous = diesel::sql_query(dialect.query(
                "SELECT sources_json,tool_catalog,
                  CASE WHEN e.execution_id=? AND e.applied_at IS NOT NULL AND e.applied_snapshot=?
                    AND e.applied_native_session=? THEN COALESCE(
                      CASE WHEN s.external_selection_reused=TRUE THEN s.external_predecessor_id END,
                      e.operation_id
                    ) ELSE NULL END AS applied_operation
                 FROM project_instruction_evidence e LEFT JOIN conversation_startups s ON s.operation_id=e.operation_id
                 WHERE e.conversation_id=? AND e.operation_id=?"
            ))
                .bind::<IdentitySql,_>(route._execution_id)
                .bind::<Text,_>(snapshot).bind::<Text,_>(native)
                .bind::<IdentitySql,_>(conversation).bind::<IdentitySql,_>(current_preparation.operation_id)
                .get_result::<RefreshHead>(connection).await.optional()?;
            let previous_instructions = previous
                .as_ref()
                .and_then(|previous| previous.sources_json.as_deref())
                .map(serde_json::from_str::<ProjectInstructions>)
                .transpose()
                .map_err(|_| StoreError::Integrity)?;
            if let Some(previous_instructions) = &previous_instructions {
                previous_instructions
                    .validate()
                    .map_err(|_| StoreError::Integrity)?;
            }
            if previous_instructions.as_ref() == Some(instructions)
                && previous
                    .as_ref()
                    .and_then(|previous| previous.tool_catalog.as_deref())
                    == Some(tool_catalog)
                && external_input.is_none_or(|input| {
                    current_preparation.external_input_id == Some(input)
                        && current_preparation.ready_connection_id == Some(connection_id)
                })
            {
                return Ok(None);
            }
            if !matches!(
                current_preparation.preparation_state.as_str(),
                "READY" | "FAILED"
            ) {
                return Err(StoreError::Integrity);
            }
            let selection = supply::current_supply!(connection, dialect, agent);
            let (kind, path) = match &selection.supply {
                HarnessSupply::Managed => ("MANAGED", None),
                HarnessSupply::Existing(path) => ("EXISTING", Some(path.as_str())),
            };
            let admitted_at = crate::directory::database_now!(connection, dialect);
            let operation = EntityId::generate(admitted_at)?;
            let selection_request = selection
                .saved
                .as_ref()
                .map(|saved| saved.request_id.as_str());
            let predecessor = previous.and_then(|previous| previous.applied_operation);
            diesel::sql_query(dialect.query("INSERT INTO conversation_startups(operation_id,conversation_id,execution_id,supply_kind,executable_path,selection_request_id,preparation_state,fresh_native_allowed,require_retained_source,intended_snapshot,intended_native_session,admitted_at,external_input_id,external_predecessor_id) VALUES(?,?,?,?,?,?,'POSSIBLE',?,?,?,?,?,?,?)"))
                .bind::<IdentitySql,_>(operation).bind::<IdentitySql,_>(conversation).bind::<IdentitySql,_>(route._execution_id)
                .bind::<Text,_>(kind).bind::<Nullable<Text>,_>(path).bind::<Nullable<Text>,_>(selection_request)
                .bind::<Bool,_>(false).bind::<Bool,_>(true).bind::<Text,_>(snapshot).bind::<Text,_>(native).bind::<BigInt,_>(admitted_at)
                .bind::<Nullable<IdentitySql>,_>(external_input)
                .bind::<Nullable<IdentitySql>,_>(predecessor)
                .execute(connection).await?;
            diesel::sql_query(
                dialect.query("DELETE FROM project_instruction_evidence WHERE agent_id=?"),
            )
            .bind::<IdentitySql, _>(agent)
            .execute(connection)
            .await?;
            diesel::sql_query(dialect.query("INSERT INTO project_instruction_evidence(operation_id,agent_id,conversation_id,execution_id,sources_json,rendered_document,owner_content,tool_catalog,created_at) VALUES(?,?,?,?,?,?,?,?,?)"))
                .bind::<IdentitySql,_>(operation).bind::<IdentitySql,_>(agent).bind::<IdentitySql,_>(conversation)
                .bind::<IdentitySql,_>(route._execution_id).bind::<Text,_>(&sources).bind::<Text,_>(&rendered)
                .bind::<Text,_>(&owner_content).bind::<Binary,_>(tool_catalog).bind::<BigInt,_>(now).execute(connection).await?;
            diesel::sql_query(
                dialect.query(
                    "UPDATE conversations SET current_preparation_operation_id=? WHERE id=?",
                ),
            )
            .bind::<IdentitySql, _>(operation)
            .bind::<IdentitySql, _>(conversation)
            .execute(connection)
            .await?;
            Ok(Some(ConnectionStartupAdmission::Dispatch {
                agent_id: agent_id.to_owned(),
                conversation_id: conversation_id.to_owned(),
                operation_id: operation.to_dto(EntityKind::ConversationOperation),
                computer_request_id: owner.current_request_id,
                execution_id: route._execution_id.to_dto(EntityKind::Execution),
                requested_supply: selection.supply,
                fresh_native_allowed: false,
                require_retained_source: true,
                context: ConnectionStartupContext::Existing {
                    snapshot: SnapshotId::parse(snapshot.to_owned())
                        .map_err(|_| StoreError::Integrity)?,
                    native_session: NativeConversationId::parse(native.to_owned())
                        .map_err(|_| StoreError::Integrity)?,
                },
            }))
        })
    }
}
