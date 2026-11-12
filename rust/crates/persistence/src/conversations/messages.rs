//! Conversation-owned chat is independent of native sources and delivery cleanup.
use super::records::source_conversation;
use super::*;
use crate::ConnectorSource;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversationMessageRole {
    User,
    Agent,
}
#[derive(Clone, PartialEq, Eq)]
pub struct ConversationMessageReceipt {
    pub id: String,
    pub conversation_id: String,
    pub role: ConversationMessageRole,
    pub receipt_order: i64,
    pub received_at: i64,
    pub source_input_id: Option<String>,
    pub source_input_request_id: Option<String>,
    pub original_accepted_at: Option<i64>,
}
pub struct ConversationMessageMetadata {
    pub receipt: ConversationMessageReceipt,
    pub bytes: i64,
}
pub struct ConversationMessageScope<'a> {
    pub agent_id: &'a str,
    pub conversation_id: &'a str,
}
/// Newer pages remain inside the finite head observed by their first page.
#[derive(Debug, Clone, Copy)]
pub enum ConversationMessageWindow {
    Latest,
    Before(i64),
    After { order: i64, through: Option<i64> },
}
pub struct ConversationMessageRegion {
    pub messages: Vec<ConversationMessageMetadata>,
    pub through: i64,
}
pub struct ConversationMessageRead<'a> {
    pub scope: ConversationMessageScope<'a>,
    pub message_id: &'a str,
    pub offset: u32,
    pub length: u32,
}
pub struct ConversationMessageChunk {
    pub bytes: Vec<u8>,
    pub offset: u32,
    pub total_bytes: i64,
}
/// Constructed by the component caller from its sealed native-owner binding.
/// Native readiness/IDs do not provide another authorization factor.
pub struct PublishConversationMessage<'a> {
    pub agent_id: &'a str,
    pub conversation_id: &'a str,
    pub text: &'a ConversationText,
}
macro_rules! safe_debug { ($($name:ty),+) => { $(impl fmt::Debug for $name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(concat!(stringify!($name), "([REDACTED])")) }
})+ }; }
safe_debug!(
    ConversationMessageReceipt,
    ConversationMessageMetadata,
    ConversationMessageScope<'_>,
    ConversationMessageRegion,
    ConversationMessageRead<'_>,
    ConversationMessageChunk,
    PublishConversationMessage<'_>
);

#[derive(Clone, Copy)]
pub(super) enum MessageOrigin<'a> {
    User {
        input: EntityId,
        request_id: &'a str,
        accepted_at: i64,
    },
    Agent,
}
#[derive(diesel::QueryableByName)]
pub(super) struct MessageOrder {
    #[diesel(sql_type=BigInt)]
    pub(super) last_order: i64,
}
macro_rules! append_message {
    ($connection:expr,$dialect:expr,$conversation:expr,$origin:expr,$text:expr,$now:expr) => {{
        use $crate::conversations::messages::{MessageOrder,MessageOrigin};
        let conversation = $conversation;
        let now = $now;
        let message = EntityId::generate(now)?;
        let (role, stored_role, input, source_request, original_accepted_at) = match $origin {
            MessageOrigin::User { input, request_id, accepted_at } =>
                (ConversationMessageRole::User,"USER",Some(input),Some(request_id),Some(accepted_at)),
            MessageOrigin::Agent => (ConversationMessageRole::Agent,"AGENT",None,None,None),
        };
        diesel::sql_query($dialect.query("INSERT INTO conversation_message_heads(conversation_id,last_order) VALUES(?,0) ON CONFLICT(conversation_id) DO NOTHING"))
            .bind::<IdentitySql,_>(conversation).execute($connection).await?;
        // The UPDATE owns the allocation row until the whole operation commits.
        // Rollback restores both the allocation head and the immutable message.
        let order = diesel::sql_query($dialect.query("UPDATE conversation_message_heads SET last_order=last_order+1 WHERE conversation_id=? AND last_order<9223372036854775807 RETURNING last_order"))
            .bind::<IdentitySql,_>(conversation).get_result::<MessageOrder>($connection).await.optional()?.ok_or(StoreError::InvalidTransition)?.last_order;
        diesel::sql_query($dialect.query("INSERT INTO conversation_messages(id,conversation_id,role,body,receipt_order,received_at,source_input_id,source_input_request_id,original_accepted_at) VALUES(?,?,?,?,?,?,?,?,?)"))
            .bind::<IdentitySql,_>(message).bind::<IdentitySql,_>(conversation)
            .bind::<Text,_>(stored_role).bind::<Text,_>($text.as_str())
            .bind::<BigInt,_>(order).bind::<BigInt,_>(now)
            .bind::<Nullable<IdentitySql>,_>(input).bind::<Nullable<Text>,_>(source_request).bind::<Nullable<BigInt>,_>(original_accepted_at)
            .execute($connection).await?;
        ConversationMessageReceipt { id: message.to_dto(EntityKind::ConversationMessage),
            conversation_id: conversation.to_dto(EntityKind::Conversation), role,
            receipt_order: order, received_at: now,
            source_input_id: input.map(|id| id.to_dto(EntityKind::ConversationOperation)),
            source_input_request_id: source_request.map(str::to_owned), original_accepted_at }
    }};
}
pub(super) use append_message;

#[derive(diesel::QueryableByName)]
struct MetadataRow {
    #[diesel(sql_type=IdentitySql)]
    id: EntityId,
    #[diesel(sql_type=Text)]
    role: String,
    #[diesel(sql_type=BigInt)]
    receipt_order: i64,
    #[diesel(sql_type=BigInt)]
    received_at: i64,
    #[diesel(sql_type=Nullable<IdentitySql>)]
    source_input_id: Option<EntityId>,
    #[diesel(sql_type=Nullable<Text>)]
    source_input_request_id: Option<String>,
    #[diesel(sql_type=Nullable<BigInt>)]
    original_accepted_at: Option<i64>,
    #[diesel(sql_type=BigInt)]
    bytes: i64,
}
impl MetadataRow {
    fn metadata(self, conversation: EntityId) -> Result<ConversationMessageMetadata, StoreError> {
        let role = match self.role.as_str() {
            "USER" => ConversationMessageRole::User,
            "AGENT" => ConversationMessageRole::Agent,
            _ => return Err(StoreError::Integrity),
        };
        Ok(ConversationMessageMetadata {
            receipt: ConversationMessageReceipt {
                id: self.id.to_dto(EntityKind::ConversationMessage),
                conversation_id: conversation.to_dto(EntityKind::Conversation),
                role,
                receipt_order: self.receipt_order,
                received_at: self.received_at,
                source_input_id: self
                    .source_input_id
                    .map(|id| id.to_dto(EntityKind::ConversationOperation)),
                original_accepted_at: self.original_accepted_at,
                source_input_request_id: self.source_input_request_id,
            },
            bytes: self.bytes,
        })
    }
}
#[derive(diesel::QueryableByName)]
struct BodyChunk {
    #[diesel(sql_type=Binary)]
    bytes: Vec<u8>,
    #[diesel(sql_type=BigInt)]
    total_bytes: i64,
}
impl Store {
    /// Current domain authority for discovery, dispatch and protected disclosure.
    /// Native restoration/capture IDs are deliberately outside this boundary.
    pub async fn check_conversation_tool_scope(
        &self,
        source: &ConnectorSource<'_>,
        scope: &ConversationMessageScope<'_>,
    ) -> Result<(), StoreError> {
        let agent = EntityId::from_dto(EntityKind::Agent, scope.agent_id)?;
        let conversation = EntityId::from_dto(EntityKind::Conversation, scope.conversation_id)?;
        operation!(self, |connection, dialect| {
            let row = source_conversation!(connection, dialect, source, conversation);
            if row.agent_id != agent {
                return Err(StoreError::NotFound);
            }
            Ok(())
        })
    }

    /// One immutable domain receipt; no publication retry/replay key is accepted.
    pub async fn publish_conversation_message(
        &self,
        source: &ConnectorSource<'_>,
        draft: &PublishConversationMessage<'_>,
    ) -> Result<ConversationMessageReceipt, StoreError> {
        let agent = EntityId::from_dto(EntityKind::Agent, draft.agent_id)?;
        let conversation = EntityId::from_dto(EntityKind::Conversation, draft.conversation_id)?;
        operation!(self, |connection, dialect| {
            let row = source_conversation!(connection, dialect, source, conversation);
            if row.agent_id != agent {
                return Err(StoreError::NotFound);
            }
            let now = crate::directory::database_now!(connection, dialect);
            Ok(append_message!(
                connection,
                dialect,
                conversation,
                MessageOrigin::Agent,
                draft.text,
                now
            ))
        })
    }

    pub async fn conversation_messages(
        &self,
        actor: &SecretDigest,
        scope: &ConversationMessageScope<'_>,
        window: ConversationMessageWindow,
        limit: i64,
        now: i64,
    ) -> Result<ConversationMessageRegion, StoreError> {
        if !(1..=25).contains(&limit) {
            return Err(StoreError::InvalidInput);
        }
        let agent = EntityId::from_dto(EntityKind::Agent, scope.agent_id)?;
        let conversation = EntityId::from_dto(EntityKind::Conversation, scope.conversation_id)?;
        operation!(self, |connection, dialect| {
            let _ = owned_agent!(connection, dialect, actor, agent, now);
            let row = conversation!(connection, dialect, conversation);
            if row.agent_id != agent {
                return Err(StoreError::NotFound);
            }
            let head = diesel::sql_query(dialect.query(
                "SELECT last_order FROM conversation_message_heads WHERE conversation_id=?",
            ))
            .bind::<IdentitySql, _>(conversation)
            .get_result::<MessageOrder>(connection)
            .await
            .optional()?
            .map_or(0, |row| row.last_order);
            let (after, before, through, newer) = match window {
                ConversationMessageWindow::Latest => (0, None, head, false),
                ConversationMessageWindow::Before(order) if order > 0 => {
                    (0, Some(order), head, false)
                }
                ConversationMessageWindow::After { order, through } if order >= 0 => {
                    let through = through.unwrap_or(head);
                    if through < order || through > head {
                        return Err(StoreError::InvalidInput);
                    }
                    (order, None, through, true)
                }
                _ => return Err(StoreError::InvalidInput),
            };
            let bytes = match dialect {
                Dialect::Postgres => "octet_length(body)",
                Dialect::Sqlite => "length(CAST(body AS BLOB))",
            };
            let direction = if newer { "ASC" } else { "DESC" };
            let sql = format!(
                "SELECT id,role,receipt_order,received_at,source_input_id,source_input_request_id,original_accepted_at,CAST({bytes} AS BIGINT) AS bytes FROM conversation_messages WHERE conversation_id=? AND receipt_order>? AND receipt_order<=? AND (? IS NULL OR receipt_order<?) ORDER BY receipt_order {direction} LIMIT ?"
            );
            let rows = diesel::sql_query(dialect.query(&sql))
                .bind::<IdentitySql, _>(conversation)
                .bind::<BigInt, _>(after)
                .bind::<BigInt, _>(through)
                .bind::<Nullable<BigInt>, _>(before)
                .bind::<Nullable<BigInt>, _>(before)
                .bind::<BigInt, _>(limit)
                .load::<MetadataRow>(connection)
                .await?;
            let mut messages = rows
                .into_iter()
                .map(|row| row.metadata(conversation))
                .collect::<Result<Vec<_>, _>>()?;
            if !newer {
                messages.reverse();
            }
            Ok(ConversationMessageRegion { messages, through })
        })
    }

    pub async fn conversation_message_chunk(
        &self,
        actor: &SecretDigest,
        read: &ConversationMessageRead<'_>,
        now: i64,
    ) -> Result<ConversationMessageChunk, StoreError> {
        if read.length == 0
            || read.length > 32 * 1024
            || i64::from(read.offset) + 1 > i64::from(i32::MAX)
        {
            return Err(StoreError::InvalidInput);
        }
        let agent = EntityId::from_dto(EntityKind::Agent, read.scope.agent_id)?;
        let conversation =
            EntityId::from_dto(EntityKind::Conversation, read.scope.conversation_id)?;
        let message = EntityId::from_dto(EntityKind::ConversationMessage, read.message_id)?;
        operation!(self, |connection, dialect| {
            let _ = owned_agent!(connection, dialect, actor, agent, now);
            let row = conversation!(connection, dialect, conversation);
            if row.agent_id != agent {
                return Err(StoreError::NotFound);
            }
            let sql = match dialect {
                Dialect::Postgres => {
                    "SELECT substring(convert_to(body,'UTF8') FROM CAST(? AS INTEGER) FOR CAST(? AS INTEGER)) AS bytes,CAST(octet_length(body) AS BIGINT) AS total_bytes FROM conversation_messages WHERE conversation_id=? AND id=?"
                }
                Dialect::Sqlite => {
                    "SELECT substr(CAST(body AS BLOB),?,?) AS bytes,CAST(length(CAST(body AS BLOB)) AS BIGINT) AS total_bytes FROM conversation_messages WHERE conversation_id=? AND id=?"
                }
            };
            let chunk = diesel::sql_query(dialect.query(sql))
                .bind::<BigInt, _>(i64::from(read.offset) + 1)
                .bind::<BigInt, _>(i64::from(read.length))
                .bind::<IdentitySql, _>(conversation)
                .bind::<IdentitySql, _>(message)
                .get_result::<BodyChunk>(connection)
                .await
                .optional()?
                .ok_or(StoreError::NotFound)?;
            if i64::from(read.offset) > chunk.total_bytes {
                return Err(StoreError::InvalidInput);
            }
            Ok(ConversationMessageChunk {
                bytes: chunk.bytes,
                offset: read.offset,
                total_bytes: chunk.total_bytes,
            })
        })
    }
}
