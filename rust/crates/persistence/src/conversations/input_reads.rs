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
        ConversationTarget::NotStarted => row.source_state == "NOT_STARTED",
        ConversationTarget::Snapshot(snapshot) => {
            row.source_state == "READY" && row.source_snapshot.as_deref() == Some(snapshot.as_str())
        }
    };
    if !valid {
        return Err(StoreError::Conflict);
    }
    Ok(())
}
impl Store {
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
                    "SELECT i.id,i.request_id,i.target_snapshot,i.accepted_at,CAST(octet_length(i.submitted_text) AS BIGINT) AS bytes FROM conversation_inputs i JOIN conversations c ON c.id=i.conversation_id WHERE i.conversation_id=? AND (c.source_snapshot=? OR (c.source_snapshot IS NULL AND ? IS NULL)) AND (i.accepted_at<? OR (i.accepted_at=? AND i.request_id<?)) ORDER BY i.accepted_at DESC,i.request_id DESC LIMIT ?"
                }
                Dialect::Sqlite => {
                    "SELECT i.id,i.request_id,i.target_snapshot,i.accepted_at,CAST(length(CAST(i.submitted_text AS BLOB)) AS BIGINT) AS bytes FROM conversation_inputs i JOIN conversations c ON c.id=i.conversation_id WHERE i.conversation_id=? AND (c.source_snapshot=? OR (c.source_snapshot IS NULL AND ? IS NULL)) AND (i.accepted_at<? OR (i.accepted_at=? AND i.request_id<?)) ORDER BY i.accepted_at DESC,i.request_id DESC LIMIT ?"
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
