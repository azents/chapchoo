//! Native liveness is explicit observation, never reconstructed request history.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversationRequestKind {
    Permission,
    Form,
}
impl ConversationRequestKind {
    fn database(self) -> &'static str {
        match self {
            Self::Permission => "PERMISSION",
            Self::Form => "FORM",
        }
    }
}
pub struct LiveConversationRequest {
    pub responder_token: String,
    pub occurrence: SourcePosition,
    pub kind: ConversationRequestKind,
    pub open: bool,
}
pub struct ConversationLiveRequestReference {
    pub responder_token: String,
    pub occurrence: SourcePosition,
    pub kind: ConversationRequestKind,
    pub answer_request_id: Option<String>,
}
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum ConversationAnswerValue {
    Text(String),
    Integer(i64),
    Number(f64),
    Boolean(bool),
    Strings(Vec<String>),
}
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum ConversationFormAnswer {
    Accept {
        content: Option<BTreeMap<String, ConversationAnswerValue>>,
    },
    Decline,
    Cancel,
}
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum ConversationAnswer {
    PermissionSelected { option_id: String },
    PermissionCancelled,
    Form(ConversationFormAnswer),
}
impl ConversationAnswer {
    fn kind(&self) -> ConversationRequestKind {
        match self {
            Self::PermissionSelected { .. } | Self::PermissionCancelled => {
                ConversationRequestKind::Permission
            }
            Self::Form(_) => ConversationRequestKind::Form,
        }
    }
    fn encoded(&self) -> Result<Vec<u8>, StoreError> {
        match self {
            Self::PermissionSelected { option_id } if option_id.is_empty() || option_id.len()>512 => return Err(StoreError::InvalidInput),
            Self::Form(ConversationFormAnswer::Accept { content:Some(content) })
                if content.len()>64 || content.values().any(|value|matches!(value,ConversationAnswerValue::Number(number) if !number.is_finite())) => return Err(StoreError::InvalidInput),
            _=>{},
        }
        let bytes = serde_json::to_vec(self).map_err(|_| StoreError::InvalidInput)?;
        if bytes.len() > 16 * 1024 * 1024 {
            return Err(StoreError::InvalidInput);
        }
        Ok(bytes)
    }
}
pub struct AcceptConversationAnswer<'a> {
    pub agent_id: &'a str,
    pub conversation_id: &'a str,
    pub request_id: &'a str,
    pub target: &'a ConversationDeliveryTarget,
    pub responder_token: &'a str,
    pub answer: &'a ConversationAnswer,
}
pub struct ConversationAnswerReceipt {
    pub id: String,
    pub conversation_id: String,
    pub request_id: String,
    pub responder_token: String,
    pub occurrence: SourcePosition,
    pub answer: ConversationAnswer,
    pub accepted_at: i64,
    pub delivery: ConversationAnswerDelivery,
}
pub struct ConversationAnswerReference {
    pub id: String,
    pub conversation_id: String,
    pub request_id: String,
    pub responder_token: String,
    pub occurrence: SourcePosition,
    pub accepted_at: i64,
    pub delivery: ConversationAnswerDelivery,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversationAnswerResult {
    ResponderAccepted,
    Rejected,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversationAnswerDelivery {
    Accepted,
    PossibleForwarding,
    Known(ConversationAnswerResult),
}
impl ConversationAnswerDelivery {
    fn parse(value: &str) -> Result<Self, StoreError> {
        match value {
            "ACCEPTED" => Ok(Self::Accepted),
            "POSSIBLE" => Ok(Self::PossibleForwarding),
            "RESPONDER_ACCEPTED" => Ok(Self::Known(ConversationAnswerResult::ResponderAccepted)),
            "REJECTED" => Ok(Self::Known(ConversationAnswerResult::Rejected)),
            _ => Err(StoreError::Integrity),
        }
    }
}
pub enum ConversationAnswerForwardingAdmission {
    Granted(ConversationForwardingGrant),
    Existing(ConversationAnswerDelivery),
}
pub enum ConversationAnswerAdmission {
    Accepted(ConversationAnswerReceipt),
    Existing(ConversationAnswerReceipt),
}
/// Internal normalized question for typed service validation, never a raw-event DTO.
pub struct ConversationAnswerQuestion {
    pub kind: ConversationRequestKind,
    pub occurrence: SourcePosition,
    pub content: Vec<u8>,
}
pub struct ConversationAnswerChunk {
    pub content: ConversationInputChunk,
    pub responder_token: String,
    pub occurrence: SourcePosition,
    pub kind: ConversationRequestKind,
}
macro_rules! safe_debug {
    ($($name:ty),+)=>{$(impl fmt::Debug for $name {
        fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result { f.write_str(concat!(stringify!($name),"([REDACTED])")) }
    })+};
}
safe_debug!(
    LiveConversationRequest,
    ConversationLiveRequestReference,
    ConversationAnswerValue,
    ConversationFormAnswer,
    ConversationAnswer,
    AcceptConversationAnswer<'_>,
    ConversationAnswerReceipt,
    ConversationAnswerReference,
    ConversationAnswerAdmission,
    ConversationAnswerQuestion,
    ConversationAnswerChunk,
    ConversationAnswerForwardingAdmission
);

#[derive(diesel::QueryableByName)]
struct RequestRow {
    #[diesel(sql_type=IdentitySql)]
    execution_id: EntityId,
    #[diesel(sql_type=Text)]
    source_snapshot: String,
    #[diesel(sql_type=Text)]
    native_session_id: String,
    #[diesel(sql_type=BigInt)]
    ordinal: i64,
    #[diesel(sql_type=Text)]
    occurrence_id: String,
    #[diesel(sql_type=Text)]
    request_kind: String,
    #[diesel(sql_type=diesel::sql_types::Bool)]
    is_open: bool,
}
impl RequestRow {
    fn matches(
        &self,
        execution: EntityId,
        target: &ConversationDeliveryTarget,
        request: &LiveConversationRequest,
    ) -> bool {
        self.execution_id == execution
            && self.source_snapshot == target.snapshot.as_str()
            && self.native_session_id == target.native_session.as_str()
            && self.ordinal == request.occurrence.ordinal()
            && self.occurrence_id == request.occurrence.occurrence().as_str()
            && self.request_kind == request.kind.database()
    }
}
#[derive(diesel::QueryableByName)]
struct AnswerRow {
    #[diesel(sql_type=IdentitySql)]
    id: EntityId,
    #[diesel(sql_type=IdentitySql)]
    conversation_id: EntityId,
    #[diesel(sql_type=Text)]
    request_id: String,
    #[diesel(sql_type=Text)]
    responder_token: String,
    #[diesel(sql_type=BigInt)]
    ordinal: i64,
    #[diesel(sql_type=Text)]
    occurrence_id: String,
    #[diesel(sql_type=Binary)]
    answer: Vec<u8>,
    #[diesel(sql_type=BigInt)]
    accepted_at: i64,
    #[diesel(sql_type=Text)]
    delivery_state: String,
}
#[derive(diesel::QueryableByName)]
struct PairOrdinal {
    #[diesel(sql_type=BigInt)]
    n: i64,
}
#[derive(diesel::QueryableByName)]
struct QuestionContent {
    #[diesel(sql_type=Binary)]
    content: Vec<u8>,
}
impl AnswerRow {
    fn receipt(self) -> Result<ConversationAnswerReceipt, StoreError> {
        Ok(ConversationAnswerReceipt {
            id: self.id.to_dto(EntityKind::ConversationOperation),
            conversation_id: self.conversation_id.to_dto(EntityKind::Conversation),
            request_id: self.request_id,
            responder_token: self.responder_token,
            occurrence: SourcePosition::new(
                self.ordinal,
                EventOccurrenceId::parse(self.occurrence_id).map_err(|_| StoreError::Integrity)?,
            )
            .map_err(|_| StoreError::Integrity)?,
            answer: serde_json::from_slice(&self.answer).map_err(|_| StoreError::Integrity)?,
            accepted_at: self.accepted_at,
            delivery: ConversationAnswerDelivery::parse(&self.delivery_state)?,
        })
    }
}
macro_rules! request {
    ($connection:expr,$dialect:expr,$conversation:expr,$token:expr) => {{
        diesel::sql_query($dialect.query(
            "SELECT * FROM conversation_requests WHERE conversation_id=? AND responder_token=?",
        ))
        .bind::<IdentitySql, _>($conversation)
        .bind::<Text, _>($token)
        .get_result::<RequestRow>($connection)
        .await
        .optional()?
    }};
}
macro_rules! answer {
    ($connection:expr,$dialect:expr,$conversation:expr,$request:expr) => {{
        diesel::sql_query(
            $dialect.query(
                "SELECT * FROM conversation_answers WHERE conversation_id=? AND request_id=?",
            ),
        )
        .bind::<IdentitySql, _>($conversation)
        .bind::<Text, _>($request)
        .get_result::<AnswerRow>($connection)
        .await
        .optional()?
    }};
}
macro_rules! answer_operation {
    ($connection:expr,$dialect:expr,$conversation:expr,$operation:expr) => {{
        diesel::sql_query(
            $dialect.query("SELECT * FROM conversation_answers WHERE conversation_id=? AND id=?"),
        )
        .bind::<IdentitySql, _>($conversation)
        .bind::<IdentitySql, _>($operation)
        .get_result::<AnswerRow>($connection)
        .await
        .optional()?
        .ok_or(StoreError::NotFound)?
    }};
}
impl Store {
    /// Discover a retained answer without requiring a currently open responder.
    pub async fn conversation_request_answer(
        &self,
        actor: &SecretDigest,
        agent: &str,
        conversation: &str,
        snapshot: &SnapshotId,
        occurrence: &EventOccurrenceId,
        now: i64,
    ) -> Result<Option<ConversationAnswerReference>, StoreError> {
        let agent = EntityId::from_dto(EntityKind::Agent, agent)?;
        let conversation = EntityId::from_dto(EntityKind::Conversation, conversation)?;
        #[derive(diesel::QueryableByName)]
        struct AnswerId {
            #[diesel(sql_type=IdentitySql)]
            id: EntityId,
            #[diesel(sql_type=Text)]
            request_id: String,
            #[diesel(sql_type=Text)]
            responder_token: String,
            #[diesel(sql_type=BigInt)]
            ordinal: i64,
            #[diesel(sql_type=BigInt)]
            accepted_at: i64,
            #[diesel(sql_type=Text)]
            delivery_state: String,
        }
        operation!(self, |connection, dialect| {
            let _ = owned_agent!(connection, dialect, actor, agent, now);
            let row = conversation!(connection, dialect, conversation);
            if row.agent_id != agent {
                return Err(StoreError::NotFound);
            }
            if row.source_snapshot.as_deref() != Some(snapshot.as_str()) {
                return Err(StoreError::Conflict);
            }
            diesel::sql_query(dialect.query(
                "SELECT a.id,a.request_id,a.responder_token,a.ordinal,a.accepted_at,a.delivery_state FROM conversation_answers a JOIN conversations c ON c.id=a.conversation_id WHERE a.conversation_id=? AND a.occurrence_id=? AND c.source_snapshot=?",
            )).bind::<IdentitySql,_>(conversation).bind::<Text,_>(occurrence.as_str()).bind::<Text,_>(snapshot.as_str())
                .get_result::<AnswerId>(connection).await.optional()?.map(|value| {
                    Ok(ConversationAnswerReference {
                        id: value.id.to_dto(EntityKind::ConversationOperation),
                        conversation_id: conversation.to_dto(EntityKind::Conversation),
                        request_id: value.request_id,
                        responder_token: value.responder_token,
                        occurrence: SourcePosition::new(value.ordinal, occurrence.clone()).map_err(|_| StoreError::Integrity)?,
                        accepted_at: value.accepted_at,
                        delivery: ConversationAnswerDelivery::parse(&value.delivery_state)?,
                    })
                }).transpose()
        })
    }
    pub async fn conversation_retained_answer(
        &self,
        actor: &SecretDigest,
        agent: &str,
        conversation: &str,
        snapshot: &SnapshotId,
        operation: &str,
        now: i64,
    ) -> Result<ConversationAnswer, StoreError> {
        let agent = EntityId::from_dto(EntityKind::Agent, agent)?;
        let conversation = EntityId::from_dto(EntityKind::Conversation, conversation)?;
        let operation = EntityId::from_dto(EntityKind::ConversationOperation, operation)?;
        operation!(self, |connection, dialect| {
            let _ = owned_agent!(connection, dialect, actor, agent, now);
            let row = conversation!(connection, dialect, conversation);
            if row.agent_id != agent {
                return Err(StoreError::NotFound);
            }
            if row.source_snapshot.as_deref() != Some(snapshot.as_str()) {
                return Err(StoreError::Conflict);
            }
            Ok(
                answer_operation!(connection, dialect, conversation, operation)
                    .receipt()?
                    .answer,
            )
        })
    }
    /// Current open responder references only. Original questions remain in normalized fields.
    pub async fn conversation_live_requests(
        &self,
        actor: &SecretDigest,
        agent: &str,
        conversation: &str,
        target: &ConversationDeliveryTarget,
        now: i64,
    ) -> Result<Vec<ConversationLiveRequestReference>, StoreError> {
        let agent = EntityId::from_dto(EntityKind::Agent, agent)?;
        let id = EntityId::from_dto(EntityKind::Conversation, conversation)?;
        #[derive(diesel::QueryableByName)]
        struct Live {
            #[diesel(sql_type=Text)]
            responder_token: String,
            #[diesel(sql_type=BigInt)]
            ordinal: i64,
            #[diesel(sql_type=Text)]
            occurrence_id: String,
            #[diesel(sql_type=Text)]
            request_kind: String,
            #[diesel(sql_type=Nullable<Text>)]
            answer_request_id: Option<String>,
        }
        operation!(self, |connection, dialect| {
            let owner = owned_agent!(connection, dialect, actor, agent, now);
            let row = conversation!(connection, dialect, id);
            super::delivery::current(&row, agent, target)?;
            if row.source_state != "READY" {
                return Err(StoreError::InvalidTransition);
            }
            let values = diesel::sql_query(dialect.query(
                "SELECT r.responder_token,r.ordinal,r.occurrence_id,r.request_kind,(SELECT a.request_id FROM conversation_answers a WHERE a.conversation_id=r.conversation_id AND a.responder_token=r.responder_token) AS answer_request_id FROM conversation_requests r JOIN computers c ON c.id=? AND c.current_execution_id=r.execution_id WHERE r.conversation_id=? AND r.source_snapshot=? AND r.native_session_id=? AND r.is_open=TRUE ORDER BY r.ordinal,r.responder_token LIMIT 33",
            )).bind::<IdentitySql,_>(owner.computer_id).bind::<IdentitySql,_>(id)
                .bind::<Text,_>(target.snapshot.as_str()).bind::<Text,_>(target.native_session.as_str())
                .load::<Live>(connection).await?;
            if values.len() > 32 {
                return Err(StoreError::Integrity);
            }
            values
                .into_iter()
                .map(|value| {
                    Ok(ConversationLiveRequestReference {
                        responder_token: value.responder_token,
                        occurrence: SourcePosition::new(
                            value.ordinal,
                            EventOccurrenceId::parse(value.occurrence_id)
                                .map_err(|_| StoreError::Integrity)?,
                        )
                        .map_err(|_| StoreError::Integrity)?,
                        kind: match value.request_kind.as_str() {
                            "PERMISSION" => ConversationRequestKind::Permission,
                            "FORM" => ConversationRequestKind::Form,
                            _ => return Err(StoreError::Integrity),
                        },
                        answer_request_id: value.answer_request_id,
                    })
                })
                .collect()
        })
    }
    pub async fn conversation_answer_reference(
        &self,
        actor: &SecretDigest,
        agent: &str,
        conversation: &str,
        request: &str,
        now: i64,
    ) -> Result<Option<ConversationAnswerReference>, StoreError> {
        request_id(request)?;
        let agent = EntityId::from_dto(EntityKind::Agent, agent)?;
        let conversation = EntityId::from_dto(EntityKind::Conversation, conversation)?;
        #[derive(diesel::QueryableByName)]
        struct Reference {
            #[diesel(sql_type=IdentitySql)]
            id: EntityId,
            #[diesel(sql_type=Text)]
            responder_token: String,
            #[diesel(sql_type=BigInt)]
            ordinal: i64,
            #[diesel(sql_type=Text)]
            occurrence_id: String,
            #[diesel(sql_type=BigInt)]
            accepted_at: i64,
            #[diesel(sql_type=Text)]
            delivery_state: String,
        }
        operation!(self, |connection, dialect| {
            let _ = owned_agent!(connection, dialect, actor, agent, now);
            let row = conversation!(connection, dialect, conversation);
            if row.agent_id != agent {
                return Err(StoreError::NotFound);
            }
            let value = diesel::sql_query(dialect.query(
                "SELECT id,responder_token,ordinal,occurrence_id,accepted_at,delivery_state FROM conversation_answers WHERE conversation_id=? AND request_id=?",
            )).bind::<IdentitySql,_>(conversation).bind::<Text,_>(request)
                .get_result::<Reference>(connection).await.optional()?;
            value
                .map(|value| {
                    Ok(ConversationAnswerReference {
                        id: value.id.to_dto(EntityKind::ConversationOperation),
                        conversation_id: conversation.to_dto(EntityKind::Conversation),
                        request_id: request.into(),
                        responder_token: value.responder_token,
                        occurrence: SourcePosition::new(
                            value.ordinal,
                            EventOccurrenceId::parse(value.occurrence_id)
                                .map_err(|_| StoreError::Integrity)?,
                        )
                        .map_err(|_| StoreError::Integrity)?,
                        accepted_at: value.accepted_at,
                        delivery: ConversationAnswerDelivery::parse(&value.delivery_state)?,
                    })
                })
                .transpose()
        })
    }
    /// Read admitted original-answer bytes; the empty suffix is a final authority check.
    pub async fn connector_answer_chunk(
        &self,
        source: &crate::ConnectorSource<'_>,
        draft: &ConversationOperationForwarding<'_>,
        offset: u32,
    ) -> Result<ConversationAnswerChunk, StoreError> {
        let (agent, id, operation) = super::delivery::ids(draft)?;
        #[derive(diesel::QueryableByName)]
        struct Chunk {
            #[diesel(sql_type=Nullable<Binary>)]
            bytes: Option<Vec<u8>>,
            #[diesel(sql_type=BigInt)]
            total_bytes: i64,
            #[diesel(sql_type=Text)]
            responder_token: String,
            #[diesel(sql_type=BigInt)]
            ordinal: i64,
            #[diesel(sql_type=Text)]
            occurrence_id: String,
            #[diesel(sql_type=Text)]
            delivery_state: String,
        }
        operation!(self, |connection, dialect| {
            let row = super::records::source_conversation!(connection, dialect, source, id);
            let route = connector!(connection, dialect, source);
            super::delivery::current(&row, agent, draft.target)?;
            let query = match dialect {
                Dialect::Postgres => {
                    "SELECT substring(answer FROM CAST(? AS INTEGER) FOR 32768) AS bytes,CAST(length(answer) AS BIGINT) AS total_bytes,responder_token,ordinal,occurrence_id,delivery_state FROM conversation_answers WHERE conversation_id=? AND id=?"
                }
                Dialect::Sqlite => {
                    "SELECT substr(answer,?,32768) AS bytes,CAST(length(answer) AS BIGINT) AS total_bytes,responder_token,ordinal,occurrence_id,delivery_state FROM conversation_answers WHERE conversation_id=? AND id=?"
                }
            };
            let chunk = diesel::sql_query(dialect.query(query))
                .bind::<BigInt, _>(i64::from(offset) + 1)
                .bind::<IdentitySql, _>(id)
                .bind::<IdentitySql, _>(operation)
                .get_result::<Chunk>(connection)
                .await
                .optional()?
                .ok_or(StoreError::NotFound)?;
            let request = request!(connection, dialect, id, &chunk.responder_token)
                .ok_or(StoreError::Conflict)?;
            if ConversationAnswerDelivery::parse(&chunk.delivery_state)?
                != ConversationAnswerDelivery::PossibleForwarding
                || request.execution_id != route._execution_id
                || request.source_snapshot != draft.target.snapshot.as_str()
                || request.native_session_id != draft.target.native_session.as_str()
                || request.ordinal != chunk.ordinal
                || request.occurrence_id != chunk.occurrence_id
                || !request.is_open
                || row.source_state != "READY"
            {
                return Err(StoreError::Conflict);
            }
            let total = u32::try_from(chunk.total_bytes).map_err(|_| StoreError::Integrity)?;
            if total == 0 || total > 16 * 1024 * 1024 || offset > total {
                return Err(StoreError::InvalidInput);
            }
            let bytes = match chunk.bytes {
                Some(bytes) => bytes,
                None if offset == total => Vec::new(),
                None => return Err(StoreError::Integrity),
            };
            let kind = match request.request_kind.as_str() {
                "PERMISSION" => ConversationRequestKind::Permission,
                "FORM" => ConversationRequestKind::Form,
                _ => return Err(StoreError::Integrity),
            };
            Ok(ConversationAnswerChunk {
                content: ConversationInputChunk {
                    bytes,
                    offset,
                    total_bytes: total,
                },
                responder_token: chunk.responder_token,
                occurrence: SourcePosition::new(
                    chunk.ordinal,
                    EventOccurrenceId::parse(chunk.occurrence_id)
                        .map_err(|_| StoreError::Integrity)?,
                )
                .map_err(|_| StoreError::Integrity)?,
                kind,
            })
        })
    }
    /// An exact native owner announces or closes a responder after raw ingestion.
    /// Closure is irreversible for that token, including when updates arrive late.
    pub async fn observe_conversation_request(
        &self,
        source: &crate::ConnectorSource<'_>,
        conversation_id: &str,
        target: &ConversationDeliveryTarget,
        request: &LiveConversationRequest,
    ) -> Result<(), StoreError> {
        request_id(&request.responder_token)?;
        let id = EntityId::from_dto(EntityKind::Conversation, conversation_id)?;
        operation!(self, |connection, dialect| {
            let row = super::records::source_conversation!(connection, dialect, source, id);
            let route = connector!(connection, dialect, source);
            super::delivery::current(&row, row.agent_id, target)?;
            let pair=diesel::sql_query(dialect.query("SELECT ordinal AS n FROM conversation_event_pairs WHERE conversation_id=? AND source_snapshot=? AND occurrence_id=?"))
                .bind::<IdentitySql,_>(id).bind::<Text,_>(target.snapshot.as_str()).bind::<Text,_>(request.occurrence.occurrence().as_str())
                .get_result::<PairOrdinal>(connection).await.optional()?;
            if pair.is_none_or(|pair| pair.n != request.occurrence.ordinal()) {
                return Err(StoreError::InvalidTransition);
            }
            if let Some(previous) = request!(connection, dialect, id, &request.responder_token) {
                if !previous.matches(route._execution_id, target, request) {
                    return Err(StoreError::Conflict);
                }
                if !request.open && previous.is_open {
                    diesel::sql_query(dialect.query("UPDATE conversation_requests SET is_open=FALSE WHERE conversation_id=? AND responder_token=?"))
                        .bind::<IdentitySql,_>(id).bind::<Text,_>(&request.responder_token).execute(connection).await?;
                }
                return Ok(());
            }
            diesel::sql_query(dialect.query("INSERT INTO conversation_requests(conversation_id,responder_token,execution_id,source_snapshot,native_session_id,ordinal,occurrence_id,request_kind,is_open) VALUES(?,?,?,?,?,?,?,?,?)"))
                .bind::<IdentitySql,_>(id).bind::<Text,_>(&request.responder_token).bind::<IdentitySql,_>(route._execution_id)
                .bind::<Text,_>(target.snapshot.as_str()).bind::<Text,_>(target.native_session.as_str())
                .bind::<BigInt,_>(request.occurrence.ordinal()).bind::<Text,_>(request.occurrence.occurrence().as_str())
                .bind::<Text,_>(request.kind.database()).bind::<diesel::sql_types::Bool,_>(request.open).execute(connection).await?;
            Ok(())
        })
    }
    /// Read the original normalized question under current private/source authority.
    /// Validation happens outside locks; final answer admission revalidates the same
    /// immutable occurrence and its actual live responder.
    pub async fn conversation_answer_question(
        &self,
        actor: &SecretDigest,
        draft: &AcceptConversationAnswer<'_>,
        now: i64,
    ) -> Result<ConversationAnswerQuestion, StoreError> {
        request_id(draft.responder_token)?;
        let agent = EntityId::from_dto(EntityKind::Agent, draft.agent_id)?;
        let id = EntityId::from_dto(EntityKind::Conversation, draft.conversation_id)?;
        operation!(self, |connection, dialect| {
            let _ = owned_agent!(connection, dialect, actor, agent, now);
            let row = conversation!(connection, dialect, id);
            super::delivery::current(&row, agent, draft.target)?;
            let request = request!(connection, dialect, id, draft.responder_token)
                .ok_or(StoreError::NotFound)?;
            let question=diesel::sql_query(dialect.query("SELECT content FROM conversation_record_fields WHERE conversation_id=? AND record_kind='REQUEST' AND native_id=? AND field_name='REQUEST' AND updated_ordinal=?"))
                .bind::<IdentitySql,_>(id).bind::<Text,_>(&request.occurrence_id).bind::<BigInt,_>(request.ordinal)
                .get_result::<QuestionContent>(connection).await.optional()?.ok_or(StoreError::InvalidTransition)?;
            if question.content.len() > 16 * 1024 * 1024 {
                return Err(StoreError::Integrity);
            }
            let kind = match request.request_kind.as_str() {
                "FORM" => ConversationRequestKind::Form,
                "PERMISSION" => ConversationRequestKind::Permission,
                _ => return Err(StoreError::Integrity),
            };
            Ok(ConversationAnswerQuestion {
                kind,
                content: question.content,
                occurrence: SourcePosition::new(
                    request.ordinal,
                    EventOccurrenceId::parse(request.occurrence_id)
                        .map_err(|_| StoreError::Integrity)?,
                )
                .map_err(|_| StoreError::Integrity)?,
            })
        })
    }
    /// Persist a single effective typed answer. This receipt is not a dispatch grant.
    pub async fn accept_conversation_answer(
        &self,
        actor: &SecretDigest,
        draft: &AcceptConversationAnswer<'_>,
        now: i64,
    ) -> Result<ConversationAnswerAdmission, StoreError> {
        request_id(draft.request_id)?;
        request_id(draft.responder_token)?;
        let bytes = draft.answer.encoded()?;
        let agent = EntityId::from_dto(EntityKind::Agent, draft.agent_id)?;
        let id = EntityId::from_dto(EntityKind::Conversation, draft.conversation_id)?;
        operation!(self, |connection, dialect| {
            let _ = owned_agent!(connection, dialect, actor, agent, now);
            let row = conversation!(connection, dialect, id);
            super::delivery::current(&row, agent, draft.target)?;
            if let Some(previous) = answer!(connection, dialect, id, draft.request_id) {
                if previous.responder_token != draft.responder_token || previous.answer != bytes {
                    return Err(StoreError::Conflict);
                }
                return Ok(ConversationAnswerAdmission::Existing(previous.receipt()?));
            }
            if row.source_state != "READY" {
                return Err(StoreError::InvalidTransition);
            }
            let request = request!(connection, dialect, id, draft.responder_token)
                .ok_or(StoreError::NotFound)?;
            if !request.is_open
                || request.source_snapshot != draft.target.snapshot.as_str()
                || request.native_session_id != draft.target.native_session.as_str()
                || request.request_kind != draft.answer.kind().database()
            {
                return Err(StoreError::InvalidTransition);
            }
            // A responder from an old native execution cannot regain authority just
            // because its source and question are still visible in product history.
            #[derive(diesel::QueryableByName)]
            struct Execution {
                #[diesel(sql_type=Nullable<IdentitySql>)]
                current_execution_id: Option<EntityId>,
            }
            let execution=diesel::sql_query(dialect.query("SELECT c.current_execution_id FROM computers c JOIN agents a ON a.computer_id=c.id WHERE a.id=? AND c.desired_state<>'DELETING'"))
                .bind::<IdentitySql,_>(agent).get_result::<Execution>(connection).await.optional()?.ok_or(StoreError::InvalidTransition)?;
            if execution.current_execution_id != Some(request.execution_id) {
                return Err(StoreError::InvalidTransition);
            }
            let operation = EntityId::generate(now)?;
            diesel::sql_query(dialect.query("INSERT INTO conversation_answers(id,conversation_id,request_id,responder_token,ordinal,occurrence_id,answer,accepted_at,delivery_state) VALUES(?,?,?,?,?,?,?,?,'ACCEPTED')"))
                .bind::<IdentitySql,_>(operation).bind::<IdentitySql,_>(id).bind::<Text,_>(draft.request_id).bind::<Text,_>(draft.responder_token)
                .bind::<BigInt,_>(request.ordinal).bind::<Text,_>(&request.occurrence_id).bind::<Binary,_>(&bytes).bind::<BigInt,_>(now).execute(connection).await?;
            let receipt = answer!(connection, dialect, id, draft.request_id)
                .ok_or(StoreError::Integrity)?
                .receipt()?;
            Ok(ConversationAnswerAdmission::Accepted(receipt))
        })
    }
    /// Persist possible forwarding before crossing the exact native response boundary.
    pub async fn begin_conversation_answer_forwarding(
        &self,
        actor: &SecretDigest,
        expected: &crate::ResolvedRoute,
        draft: &ConversationOperationForwarding<'_>,
        now: i64,
    ) -> Result<ConversationAnswerForwardingAdmission, StoreError> {
        let (agent, id, operation) = super::delivery::ids(draft)?;
        operation!(self, |connection, dialect| {
            let owner = owned_agent!(connection, dialect, actor, agent, now);
            let (route, session_id) =
                super::delivery::dispatch_route!(connection, dialect, expected, owner.computer_id);
            let row = conversation!(connection, dialect, id);
            super::delivery::current(&row, agent, draft.target)?;
            let answer = answer_operation!(connection, dialect, id, operation);
            let request = request!(connection, dialect, id, &answer.responder_token)
                .ok_or(StoreError::Integrity)?;
            if request.execution_id != route._execution_id
                || request.source_snapshot != draft.target.snapshot.as_str()
                || request.native_session_id != draft.target.native_session.as_str()
            {
                return Err(StoreError::Conflict);
            }
            let previous = ConversationAnswerDelivery::parse(&answer.delivery_state)?;
            if previous != ConversationAnswerDelivery::Accepted {
                return Ok(ConversationAnswerForwardingAdmission::Existing(previous));
            }
            if row.source_state != "READY" || !request.is_open {
                return Err(StoreError::InvalidTransition);
            }
            diesel::sql_query(
                dialect
                    .query("UPDATE conversation_answers SET delivery_state='POSSIBLE' WHERE id=?"),
            )
            .bind::<IdentitySql, _>(operation)
            .execute(connection)
            .await?;
            Ok(ConversationAnswerForwardingAdmission::Granted(
                ConversationForwardingGrant {
                    execution_id: route._execution_id.to_dto(EntityKind::Execution),
                    connector_session_id: session_id,
                },
            ))
        })
    }
    /// Local SDK responder acceptance is not remote consumption or work completion.
    /// Only an exact closed-target rejection is known rejection; lost acknowledgements
    /// retain possible forwarding and never permit a different answer or retry.
    pub async fn complete_conversation_answer_forwarding(
        &self,
        source: &crate::ConnectorSource<'_>,
        draft: &ConversationOperationForwarding<'_>,
        result: ConversationAnswerResult,
    ) -> Result<(), StoreError> {
        let (agent, id, operation) = super::delivery::ids(draft)?;
        operation!(self, |connection, dialect| {
            let row = super::records::source_conversation!(connection, dialect, source, id);
            let route = connector!(connection, dialect, source);
            super::delivery::current(&row, agent, draft.target)?;
            let answer = answer_operation!(connection, dialect, id, operation);
            let request = request!(connection, dialect, id, &answer.responder_token)
                .ok_or(StoreError::Integrity)?;
            if request.execution_id != route._execution_id
                || request.source_snapshot != draft.target.snapshot.as_str()
                || request.native_session_id != draft.target.native_session.as_str()
            {
                return Err(StoreError::Conflict);
            }
            match ConversationAnswerDelivery::parse(&answer.delivery_state)? {
                ConversationAnswerDelivery::Known(previous) if previous == result => return Ok(()),
                ConversationAnswerDelivery::PossibleForwarding => {}
                _ => return Err(StoreError::Conflict),
            }
            let state = match result {
                ConversationAnswerResult::ResponderAccepted => "RESPONDER_ACCEPTED",
                ConversationAnswerResult::Rejected => "REJECTED",
            };
            diesel::sql_query(
                dialect.query("UPDATE conversation_answers SET delivery_state=? WHERE id=?"),
            )
            .bind::<Text, _>(state)
            .bind::<IdentitySql, _>(operation)
            .execute(connection)
            .await?;
            Ok(())
        })
    }
    /// Lookup retains original typed values and never delivers an answer.
    pub async fn conversation_answer(
        &self,
        actor: &SecretDigest,
        agent_id: &str,
        conversation_id: &str,
        request_id_value: &str,
        now: i64,
    ) -> Result<Option<ConversationAnswerReceipt>, StoreError> {
        request_id(request_id_value)?;
        let agent = EntityId::from_dto(EntityKind::Agent, agent_id)?;
        let id = EntityId::from_dto(EntityKind::Conversation, conversation_id)?;
        operation!(self, |connection, dialect| {
            let _ = owned_agent!(connection, dialect, actor, agent, now);
            let row = conversation!(connection, dialect, id);
            if row.agent_id != agent {
                return Err(StoreError::NotFound);
            }
            answer!(connection, dialect, id, request_id_value)
                .map(AnswerRow::receipt)
                .transpose()
        })
    }
}
