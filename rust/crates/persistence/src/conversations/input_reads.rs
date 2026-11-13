//! Source-bound retained-input discovery never authorizes another dispatch.
use super::*;

pub struct ConversationInputScope<'a> {
    pub agent_id: &'a str,
    pub conversation_id: &'a str,
    pub target: &'a ConversationTarget,
}
pub struct ConversationInputAnchor {
    pub accepted_at: i64,
    pub request_id: String,
}
#[derive(Clone)]
pub struct RetryConversationInput {
    pub agent_id: String,
    pub conversation_id: String,
    pub target: ConversationTarget,
    pub original_input_id: String,
    pub original_request_id: String,
    pub request_id: String,
}
pub struct RetainedConversationInput {
    pub reference: ConversationInputReference,
    pub bytes: i64,
}
macro_rules! safe_debug { ($($name:ty),+) => { $(impl fmt::Debug for $name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(concat!(stringify!($name), "([REDACTED])")) }
})+ }; }
safe_debug!(
    ConversationInputScope<'_>,
    ConversationInputAnchor,
    RetryConversationInput,
    RetainedConversationInput
);
fn matches(
    row: &ConversationRow,
    scope: &ConversationInputScope<'_>,
    agent: EntityId,
) -> Result<(), StoreError> {
    if row.agent_id != agent {
        return Err(StoreError::NotFound);
    }
    let valid = match scope.target {
        ConversationTarget::NotStarted => {
            matches!(row.source_state.as_str(), "NOT_STARTED" | "LOADING")
                && row.source_snapshot.is_none()
        }
        ConversationTarget::Snapshot(snapshot) => {
            matches!(row.source_state.as_str(), "LOADING" | "READY")
                && row.source_snapshot.as_deref() == Some(snapshot.as_str())
        }
    };
    if !valid {
        return Err(StoreError::Conflict);
    }
    Ok(())
}
impl Store {
    /// Explicit replacement deletes the failed row before fresh admission.
    /// Both changes commit together, preserving its text if admission fails.
    pub async fn retry_failed_conversation_input(
        &self,
        actor: &SecretDigest,
        draft: &RetryConversationInput,
        now: i64,
    ) -> Result<ConversationInputAdmission, StoreError> {
        self.retry_conversation_input_for_consumer(
            actor,
            draft,
            now,
            crate::DirectoryPolicy::default(),
        )
        .await
    }
    pub async fn retry_conversation_input_for_consumer(
        &self,
        actor: &SecretDigest,
        draft: &RetryConversationInput,
        now: i64,
        policy: crate::DirectoryPolicy,
    ) -> Result<ConversationInputAdmission, StoreError> {
        request_id(&draft.request_id)?;
        request_id(&draft.original_request_id)?;
        if draft.request_id == draft.original_request_id {
            return Err(StoreError::InvalidInput);
        }
        let agent = EntityId::from_dto(EntityKind::Agent, &draft.agent_id)?;
        let conversation = EntityId::from_dto(EntityKind::Conversation, &draft.conversation_id)?;
        let original =
            EntityId::from_dto(EntityKind::ConversationOperation, &draft.original_input_id)?;
        #[derive(diesel::QueryableByName)]
        struct Failure {
            #[diesel(sql_type=diesel::sql_types::Bool)]
            failed: bool,
        }
        #[derive(diesel::QueryableByName)]
        struct Replacement {
            #[diesel(sql_type=Nullable<IdentitySql>)]
            replaces_input_id: Option<EntityId>,
        }
        operation!(self, |connection, dialect| {
            let owner = owned_agent!(connection, dialect, actor, agent, now);
            let row = conversation!(connection, dialect, conversation);
            matches(
                &row,
                &ConversationInputScope {
                    agent_id: &draft.agent_id,
                    conversation_id: &draft.conversation_id,
                    target: &draft.target,
                },
                agent,
            )?;
            let previous = diesel::sql_query(dialect.query(
                "SELECT * FROM conversation_inputs WHERE conversation_id=? AND request_id=?",
            ))
            .bind::<IdentitySql, _>(conversation)
            .bind::<Text, _>(&draft.request_id)
            .get_result::<super::inputs::InputRow>(connection)
            .await
            .optional()?;
            if let Some(previous) = previous {
                let replacement = diesel::sql_query(dialect.query(
                    "SELECT replaces_input_id FROM conversation_inputs WHERE conversation_id=? AND request_id=?",
                ))
                .bind::<IdentitySql, _>(conversation)
                .bind::<Text, _>(&draft.request_id)
                .get_result::<Replacement>(connection)
                .await?;
                if replacement.replaces_input_id != Some(original) {
                    return Err(StoreError::Conflict);
                }
                let previous = previous.receipt()?;
                if previous.target != draft.target {
                    return Err(StoreError::Conflict);
                }
                return Ok(ConversationInputAdmission::Existing(previous));
            }
            if owner.desired_state == "DELETING" {
                return Err(StoreError::InvalidTransition);
            }
            let failure = diesel::sql_query(dialect.query(
                "SELECT (queue_state='ERROR') AS failed FROM conversation_inputs WHERE conversation_id=? AND id=? AND request_id=?",
            ))
            .bind::<IdentitySql, _>(conversation)
            .bind::<IdentitySql, _>(original)
            .bind::<Text, _>(&draft.original_request_id)
            .get_result::<Failure>(connection)
            .await
            .optional()?
            .ok_or(StoreError::NotFound)?;
            if !failure.failed {
                return Err(StoreError::InvalidTransition);
            }
            let old = diesel::sql_query(
                dialect.query("SELECT * FROM conversation_inputs WHERE conversation_id=? AND id=?"),
            )
            .bind::<IdentitySql, _>(conversation)
            .bind::<IdentitySql, _>(original)
            .get_result::<super::inputs::InputRow>(connection)
            .await?
            .receipt()?;
            diesel::sql_query(dialect.query("DELETE FROM conversation_inputs WHERE id=?"))
                .bind::<IdentitySql, _>(original)
                .execute(connection)
                .await?;
            let id = EntityId::generate(now)?;
            let consumer_now = crate::directory::database_now!(connection, dialect);
            let consumer_expires = policy.expiry(consumer_now, true)?;
            diesel::sql_query(dialect.query(
                "INSERT INTO conversation_inputs(id,conversation_id,request_id,target_snapshot,submitted_text,accepted_at,replaces_input_id,queue_state,consumer_expires_at) VALUES(?,?,?,?,?,?,?,'PENDING',?)",
            ))
            .bind::<IdentitySql, _>(id)
            .bind::<IdentitySql, _>(conversation)
            .bind::<Text, _>(&draft.request_id)
            .bind::<Nullable<Text>, _>(draft.target.snapshot().map(SnapshotId::as_str))
            .bind::<Text, _>(old.text.as_str())
            .bind::<BigInt, _>(now)
            .bind::<IdentitySql, _>(original)
            .bind::<BigInt, _>(consumer_expires)
            .execute(connection).await?;
            Ok(ConversationInputAdmission::Accepted(
                ConversationInputReceipt {
                    id: id.to_dto(EntityKind::ConversationOperation),
                    conversation_id: draft.conversation_id.clone(),
                    request_id: draft.request_id.clone(),
                    target: draft.target.clone(),
                    text: old.text,
                    accepted_at: now,
                },
            ))
        })
    }

    pub async fn closed_conversation_inputs(
        &self,
        actor: &SecretDigest,
        scope: &ConversationInputScope<'_>,
        known: &[String],
        now: i64,
    ) -> Result<Vec<String>, StoreError> {
        if known.len() > 25 {
            return Err(StoreError::InvalidInput);
        }
        let ids = known
            .iter()
            .map(|value| EntityId::from_dto(EntityKind::ConversationOperation, value))
            .collect::<Result<Vec<_>, _>>()?;
        let agent = EntityId::from_dto(EntityKind::Agent, scope.agent_id)?;
        let conversation = EntityId::from_dto(EntityKind::Conversation, scope.conversation_id)?;
        #[derive(diesel::QueryableByName)]
        struct Count {
            #[diesel(sql_type=BigInt)]
            n: i64,
        }
        operation!(self, |connection, dialect| {
            let _ = owned_agent!(connection, dialect, actor, agent, now);
            let row = conversation!(connection, dialect, conversation);
            matches(&row, scope, agent)?;
            let mut closed = Vec::new();
            for (value, id) in known.iter().zip(&ids) {
                let count = diesel::sql_query(dialect.query(
                    "SELECT CAST(count(*) AS BIGINT) AS n FROM conversation_inputs WHERE conversation_id=? AND id=?",
                ))
                .bind::<IdentitySql, _>(conversation)
                .bind::<IdentitySql, _>(*id)
                .get_result::<Count>(connection)
                .await?;
                if count.n == 0 {
                    closed.push(value.clone());
                }
            }
            Ok(closed)
        })
    }
    pub async fn latest_conversation_input(
        &self,
        actor: &SecretDigest,
        scope: &ConversationInputScope<'_>,
        now: i64,
    ) -> Result<Option<RetainedConversationInput>, StoreError> {
        Ok(self
            .conversation_inputs(actor, scope, None, 1, now)
            .await?
            .into_iter()
            .next())
    }
    pub async fn conversation_inputs(
        &self,
        actor: &SecretDigest,
        scope: &ConversationInputScope<'_>,
        before: Option<&ConversationInputAnchor>,
        limit: i64,
        now: i64,
    ) -> Result<Vec<RetainedConversationInput>, StoreError> {
        if !(1..=25).contains(&limit) {
            return Err(StoreError::InvalidInput);
        }
        if let Some(before) = before {
            request_id(&before.request_id)?;
        }
        let agent = EntityId::from_dto(EntityKind::Agent, scope.agent_id)?;
        let conversation = EntityId::from_dto(EntityKind::Conversation, scope.conversation_id)?;
        let time = before.map_or(i64::MAX, |value| value.accepted_at);
        let request = before.map_or("", |value| value.request_id.as_str());
        #[derive(diesel::QueryableByName)]
        struct Row {
            #[diesel(sql_type=IdentitySql)]
            id: EntityId,
            #[diesel(sql_type=Text)]
            request_id: String,
            #[diesel(sql_type=Nullable<Text>)]
            target_snapshot: Option<String>,
            #[diesel(sql_type=BigInt)]
            accepted_at: i64,
            #[diesel(sql_type=BigInt)]
            bytes: i64,
        }
        operation!(self, |connection, dialect| {
            let _ = owned_agent!(connection, dialect, actor, agent, now);
            let row = conversation!(connection, dialect, conversation);
            matches(&row, scope, agent)?;
            let query = match dialect {
                Dialect::Postgres => {
                    "SELECT i.id,i.request_id,i.target_snapshot,i.accepted_at,CAST(COALESCE(octet_length(i.submitted_text),0) AS BIGINT) AS bytes FROM conversation_inputs i JOIN conversations c ON c.id=i.conversation_id WHERE i.conversation_id=? AND (c.source_snapshot=? OR (c.source_snapshot IS NULL AND ? IS NULL)) AND (i.accepted_at<? OR (i.accepted_at=? AND i.request_id<?)) ORDER BY i.accepted_at DESC,i.request_id DESC LIMIT ?"
                }
                Dialect::Sqlite => {
                    "SELECT i.id,i.request_id,i.target_snapshot,i.accepted_at,CAST(COALESCE(length(CAST(i.submitted_text AS BLOB)),0) AS BIGINT) AS bytes FROM conversation_inputs i JOIN conversations c ON c.id=i.conversation_id WHERE i.conversation_id=? AND (c.source_snapshot=? OR (c.source_snapshot IS NULL AND ? IS NULL)) AND (i.accepted_at<? OR (i.accepted_at=? AND i.request_id<?)) ORDER BY i.accepted_at DESC,i.request_id DESC LIMIT ?"
                }
            };
            let snapshot = scope.target.snapshot().map(SnapshotId::as_str);
            let rows = diesel::sql_query(dialect.query(query))
                .bind::<IdentitySql, _>(conversation)
                .bind::<Nullable<Text>, _>(snapshot)
                .bind::<Nullable<Text>, _>(snapshot)
                .bind::<BigInt, _>(time)
                .bind::<BigInt, _>(time)
                .bind::<Text, _>(request)
                .bind::<BigInt, _>(limit)
                .load::<Row>(connection)
                .await?;
            rows.into_iter()
                .map(|value| {
                    Ok(RetainedConversationInput {
                        reference: super::inputs::InputReferenceRow {
                            id: value.id,
                            conversation_id: conversation,
                            request_id: value.request_id,
                            target_snapshot: value.target_snapshot,
                            accepted_at: value.accepted_at,
                        }
                        .reference()?,
                        bytes: value.bytes,
                    })
                })
                .collect()
        })
    }
    /// Remove a settled failed outgoing item, not a recorded Chat message.
    pub async fn delete_failed_conversation_input(
        &self,
        actor: &SecretDigest,
        scope: &ConversationInputScope<'_>,
        request: &str,
        now: i64,
    ) -> Result<(), StoreError> {
        request_id(request)?;
        let agent = EntityId::from_dto(EntityKind::Agent, scope.agent_id)?;
        let conversation = EntityId::from_dto(EntityKind::Conversation, scope.conversation_id)?;
        #[derive(diesel::QueryableByName)]
        struct FailedInput {
            #[diesel(sql_type=IdentitySql)]
            id: EntityId,
            #[diesel(sql_type=diesel::sql_types::Bool)]
            failed: bool,
        }
        operation!(self, |connection, dialect| {
            let _ = owned_agent!(connection, dialect, actor, agent, now);
            let row = conversation!(connection, dialect, conversation);
            matches(&row, scope, agent)?;
            let input = diesel::sql_query(dialect.query("SELECT id,(queue_state='ERROR') AS failed FROM conversation_inputs WHERE conversation_id=? AND request_id=?"))
                .bind::<IdentitySql,_>(conversation).bind::<Text,_>(request)
                .get_result::<FailedInput>(connection).await.optional()?;
            let Some(input) = input else {
                // Exact deletion response recovery performs no new effect.
                return Ok(());
            };
            if !input.failed {
                return Err(StoreError::InvalidTransition);
            }
            diesel::sql_query(dialect.query("DELETE FROM conversation_inputs WHERE id=?"))
                .bind::<IdentitySql, _>(input.id)
                .execute(connection)
                .await?;
            Ok(())
        })
    }
    pub async fn conversation_input_text(
        &self,
        actor: &SecretDigest,
        scope: &ConversationInputScope<'_>,
        request: &str,
        offset: u32,
        length: u32,
        now: i64,
    ) -> Result<EventPayloadChunk, StoreError> {
        request_id(request)?;
        if length == 0 || length > 32768 {
            return Err(StoreError::InvalidInput);
        }
        let agent = EntityId::from_dto(EntityKind::Agent, scope.agent_id)?;
        let conversation = EntityId::from_dto(EntityKind::Conversation, scope.conversation_id)?;
        #[derive(diesel::QueryableByName)]
        struct Chunk {
            #[diesel(sql_type=Binary)]
            bytes: Vec<u8>,
            #[diesel(sql_type=BigInt)]
            total_bytes: i64,
        }
        operation!(self, |connection, dialect| {
            let _ = owned_agent!(connection, dialect, actor, agent, now);
            let row = conversation!(connection, dialect, conversation);
            matches(&row, scope, agent)?;
            let query = match dialect {
                Dialect::Postgres => {
                    "SELECT substring(convert_to(i.submitted_text,'UTF8') FROM CAST(? AS INTEGER) FOR CAST(? AS INTEGER)) AS bytes,CAST(octet_length(i.submitted_text) AS BIGINT) AS total_bytes FROM conversation_inputs i JOIN conversations c ON c.id=i.conversation_id WHERE i.conversation_id=? AND i.request_id=? AND (c.source_snapshot=? OR (c.source_snapshot IS NULL AND ? IS NULL))"
                }
                Dialect::Sqlite => {
                    "SELECT substr(CAST(i.submitted_text AS BLOB),?,?) AS bytes,CAST(length(CAST(i.submitted_text AS BLOB)) AS BIGINT) AS total_bytes FROM conversation_inputs i JOIN conversations c ON c.id=i.conversation_id WHERE i.conversation_id=? AND i.request_id=? AND (c.source_snapshot=? OR (c.source_snapshot IS NULL AND ? IS NULL))"
                }
            };
            let snapshot = scope.target.snapshot().map(SnapshotId::as_str);
            let chunk = diesel::sql_query(dialect.query(query))
                .bind::<BigInt, _>(i64::from(offset) + 1)
                .bind::<BigInt, _>(i64::from(length))
                .bind::<IdentitySql, _>(conversation)
                .bind::<Text, _>(request)
                .bind::<Nullable<Text>, _>(snapshot)
                .bind::<Nullable<Text>, _>(snapshot)
                .get_result::<Chunk>(connection)
                .await
                .optional()?
                .ok_or(StoreError::NotFound)?;
            if i64::from(offset) > chunk.total_bytes {
                return Err(StoreError::InvalidInput);
            }
            Ok(EventPayloadChunk {
                offset,
                total_bytes: chunk.total_bytes,
                bytes: chunk.bytes,
            })
        })
    }
}
