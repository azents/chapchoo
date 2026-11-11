use super::projection::project_records;
use super::*;
use crate::ConnectorSource;
use diesel::sql_types::Integer;

pub struct SourceBinding {
    pub conversation_id: String,
    pub expected_previous: Option<SnapshotId>,
    pub snapshot: SnapshotId,
    pub native_session: NativeConversationId,
}
/// Persistence transparently retains both representations; normalization is service-owned.
pub struct CapturedEventPair {
    pub snapshot: SnapshotId,
    pub position: SourcePosition,
    pub raw: Vec<u8>,
    pub normalized: Vec<u8>,
}
pub struct EventPairMetadata {
    pub position: SourcePosition,
    pub raw_bytes: i64,
    pub normalized_bytes: i64,
}
#[derive(Debug, Clone, Copy)]
pub enum EventPayloadKind {
    Raw,
    Normalized,
}
pub struct EventPayloadChunk {
    pub bytes: Vec<u8>,
    pub offset: u32,
    pub total_bytes: i64,
}
#[derive(Debug)]
pub struct EventMetadataRead<'a> {
    pub agent_id: &'a str,
    pub conversation_id: &'a str,
    pub snapshot: &'a SnapshotId,
    pub after: Option<&'a SourcePosition>,
    pub limit: i64,
}
#[derive(Debug)]
pub struct EventChunkRead<'a> {
    pub agent_id: &'a str,
    pub conversation_id: &'a str,
    pub snapshot: &'a SnapshotId,
    pub occurrence: &'a EventOccurrenceId,
    pub kind: EventPayloadKind,
    pub offset: u32,
    pub length: u32,
}
macro_rules! safe_debug {
    ($name:ident) => {
        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(concat!(stringify!($name), "([REDACTED])"))
            }
        }
    };
}
safe_debug!(SourceBinding);
safe_debug!(CapturedEventPair);
safe_debug!(EventPairMetadata);
safe_debug!(EventPayloadChunk);

macro_rules! source_conversation {
    ($connection:expr,$dialect:expr,$source:expr,$id:expr) => {{
        use $crate::conversations::authority::LockedTeam;
        let team_sql=match $dialect {
            Dialect::Postgres => "SELECT t.id FROM teams t JOIN agents a ON a.team_id=t.id JOIN conversations c ON c.agent_id=a.id JOIN memberships m ON m.team_id=t.id AND m.user_id=a.owner_user_id WHERE c.id=? FOR NO KEY UPDATE OF t",
            Dialect::Sqlite => "SELECT t.id FROM teams t JOIN agents a ON a.team_id=t.id JOIN conversations c ON c.agent_id=a.id JOIN memberships m ON m.team_id=t.id AND m.user_id=a.owner_user_id WHERE c.id=?",
        };
        diesel::sql_query($dialect.query(team_sql)).bind::<IdentitySql,_>($id)
            .get_result::<LockedTeam>($connection).await.optional()?.ok_or(StoreError::NotFound)?;
        let context=connector!($connection,$dialect,$source);
        diesel::sql_query($dialect.query("SELECT c.* FROM conversations c JOIN agents a ON a.id=c.agent_id JOIN memberships m ON m.user_id=a.owner_user_id AND m.team_id=a.team_id WHERE c.id=? AND a.computer_id=?"))
            .bind::<IdentitySql,_>($id).bind::<IdentitySql,_>(context.computer_id)
            .get_result::<ConversationRow>($connection).await.optional()?.ok_or(StoreError::NotFound)?
    }};
}
pub(super) use source_conversation;
#[derive(diesel::QueryableByName)]
struct ExistingPair {
    #[diesel(sql_type=BigInt)]
    ordinal: i64,
}
#[derive(diesel::QueryableByName)]
struct MetadataRow {
    #[diesel(sql_type=BigInt)]
    ordinal: i64,
    #[diesel(sql_type=Text)]
    occurrence_id: String,
    #[diesel(sql_type=BigInt)]
    raw_bytes: i64,
    #[diesel(sql_type=BigInt)]
    normalized_bytes: i64,
}
#[derive(diesel::QueryableByName)]
struct ChunkRow {
    #[diesel(sql_type=Binary)]
    bytes: Vec<u8>,
    #[diesel(sql_type=BigInt)]
    total_bytes: i64,
}

impl Store {
    /// Discover the current product conversation of this exact execution. Reading
    /// absence never creates a conversation or starts native work.
    pub async fn connector_current_conversation(
        &self,
        source: &ConnectorSource<'_>,
    ) -> Result<Option<ConversationSummary>, StoreError> {
        let session = EntityId::from_dto(EntityKind::ConnectorSession, source.session_id)?;
        operation!(self, |connection, dialect| {
            let initial=diesel::sql_query(dialect.query("SELECT c.* FROM conversations c JOIN agent_conversations ac ON ac.conversation_id=c.id AND ac.agent_id=c.agent_id JOIN agents a ON a.id=c.agent_id JOIN computer_executions e ON e.computer_id=a.computer_id JOIN connector_sessions s ON s.execution_id=e.id WHERE s.id=?"))
                .bind::<IdentitySql,_>(session).get_result::<ConversationRow>(connection).await.optional()?;
            let Some(initial) = initial else {
                let _ = connector!(connection, dialect, source);
                return Ok(None);
            };
            let row = source_conversation!(connection, dialect, source, initial.id);
            let current=diesel::sql_query(dialect.query("SELECT c.* FROM conversations c JOIN agent_conversations ac ON ac.conversation_id=c.id AND ac.agent_id=c.agent_id WHERE c.id=?"))
                .bind::<IdentitySql,_>(row.id).get_result::<ConversationRow>(connection).await.optional()?.ok_or(StoreError::Conflict)?;
            Ok(Some(current.summary()?))
        })
    }
    /// Current execution-bound source facts for server-owned normalization.
    pub async fn connector_conversation(
        &self,
        source: &ConnectorSource<'_>,
        conversation_id: &str,
    ) -> Result<ConversationSummary, StoreError> {
        let id = EntityId::from_dto(EntityKind::Conversation, conversation_id)?;
        operation!(self, |connection, dialect| {
            source_conversation!(connection, dialect, source, id).summary()
        })
    }
    pub async fn bind_conversation_source(
        &self,
        source: &ConnectorSource<'_>,
        binding: &SourceBinding,
    ) -> Result<ConversationSummary, StoreError> {
        let id = EntityId::from_dto(EntityKind::Conversation, &binding.conversation_id)?;
        operation!(self, |connection, dialect| {
            let row = source_conversation!(connection, dialect, source, id);
            if row
                .native_session_id
                .as_deref()
                .is_some_and(|native| native != binding.native_session.as_str())
            {
                return Err(StoreError::InvalidTransition);
            }
            if row.source_snapshot.as_deref() == Some(binding.snapshot.as_str()) {
                return row.summary();
            }
            if row.source_snapshot.as_deref()
                != binding.expected_previous.as_ref().map(SnapshotId::as_str)
            {
                return Err(StoreError::Conflict);
            }
            if row.source_snapshot.is_none() {
                let known=diesel::sql_query(dialect.query("SELECT CAST(count(*) AS BIGINT) AS ordinal FROM (SELECT p.ready_snapshot,p.ready_native_session FROM conversation_input_preparation p JOIN conversation_inputs i ON i.id=p.operation_id WHERE i.conversation_id=? AND p.preparation_state='READY' UNION ALL SELECT ready_snapshot,ready_native_session FROM conversation_startups WHERE conversation_id=? AND preparation_state='READY') ready WHERE ready_snapshot<>? OR ready_native_session<>?"))
                    .bind::<IdentitySql,_>(id).bind::<IdentitySql,_>(id).bind::<Text,_>(binding.snapshot.as_str()).bind::<Text,_>(binding.native_session.as_str())
                    .get_result::<ExistingPair>(connection).await?;
                if known.ordinal != 0 {
                    return Err(StoreError::Conflict);
                }
            }
            if row.source_snapshot.is_some() {
                diesel::sql_query(
                    dialect.query("DELETE FROM conversation_native_state WHERE conversation_id=?"),
                )
                .bind::<IdentitySql, _>(id)
                .execute(connection)
                .await?;
                diesel::sql_query(
                    dialect.query("DELETE FROM conversation_stops WHERE conversation_id=?"),
                )
                .bind::<IdentitySql, _>(id)
                .execute(connection)
                .await?;
                diesel::sql_query(
                    dialect.query("DELETE FROM conversation_requests WHERE conversation_id=?"),
                )
                .bind::<IdentitySql, _>(id)
                .execute(connection)
                .await?;
                diesel::sql_query(
                    dialect.query("DELETE FROM conversation_records WHERE conversation_id=?"),
                )
                .bind::<IdentitySql, _>(id)
                .execute(connection)
                .await?;
                // Source replacement retires all source-local preparation facts.
                diesel::sql_query(
                    dialect.query("DELETE FROM conversation_startups WHERE conversation_id=?"),
                )
                .bind::<IdentitySql, _>(id)
                .execute(connection)
                .await?;
                // D19 replaces independently admitted inputs too; no hidden archive remains.
                diesel::sql_query(
                    dialect.query("DELETE FROM conversation_inputs WHERE conversation_id=?"),
                )
                .bind::<IdentitySql, _>(id)
                .execute(connection)
                .await?;
                diesel::sql_query(
                    dialect.query("DELETE FROM conversation_event_pairs WHERE conversation_id=?"),
                )
                .bind::<IdentitySql, _>(id)
                .execute(connection)
                .await?;
            }
            diesel::sql_query(dialect.query("UPDATE conversations SET native_session_id=?,source_snapshot=?,source_state='LOADING',source_ordinal=0,source_event_id=NULL WHERE id=?"))
                .bind::<Text,_>(binding.native_session.as_str()).bind::<Text,_>(binding.snapshot.as_str())
                .bind::<IdentitySql,_>(id).execute(connection).await?;
            conversation!(connection, dialect, id).summary()
        })
    }

    /// A duplicate occurrence cannot publish a second normalization or advance progress.
    pub async fn ingest_conversation_pair(
        &self,
        source: &ConnectorSource<'_>,
        conversation_id: &str,
        pair: &CapturedEventPair,
    ) -> Result<ConversationSummary, StoreError> {
        self.ingest_pair(source, conversation_id, pair, &[]).await
    }
    /// Pair, cursor and normalized record fields are one completed publication.
    pub async fn ingest_conversation_records(
        &self,
        source: &ConnectorSource<'_>,
        conversation_id: &str,
        pair: &CapturedEventPair,
        records: &[ConversationRecordUpdate],
    ) -> Result<ConversationSummary, StoreError> {
        if super::projection::validate(records)? != pair.normalized {
            return Err(StoreError::InvalidInput);
        }
        self.ingest_pair(source, conversation_id, pair, records)
            .await
    }
    async fn ingest_pair(
        &self,
        source: &ConnectorSource<'_>,
        conversation_id: &str,
        pair: &CapturedEventPair,
        records: &[ConversationRecordUpdate],
    ) -> Result<ConversationSummary, StoreError> {
        if pair.raw.is_empty()
            || pair.normalized.is_empty()
            || pair.raw.len() > i32::MAX as usize
            || pair.normalized.len() > i32::MAX as usize
        {
            return Err(StoreError::InvalidInput);
        }
        let id = EntityId::from_dto(EntityKind::Conversation, conversation_id)?;
        operation!(self, |connection, dialect| {
            let row = source_conversation!(connection, dialect, source, id);
            if row.source_snapshot.as_deref() != Some(pair.snapshot.as_str()) {
                return Err(StoreError::Conflict);
            }
            let existing=diesel::sql_query(dialect.query("SELECT ordinal FROM conversation_event_pairs WHERE conversation_id=? AND occurrence_id=?"))
                .bind::<IdentitySql,_>(id).bind::<Text,_>(pair.position.occurrence().as_str())
                .get_result::<ExistingPair>(connection).await.optional()?;
            if let Some(existing) = existing {
                let identical=diesel::sql_query(dialect.query("SELECT ordinal FROM conversation_event_pairs WHERE conversation_id=? AND occurrence_id=? AND source_snapshot=? AND ordinal=? AND raw_event=? AND normalized_event=?"))
                    .bind::<IdentitySql,_>(id).bind::<Text,_>(pair.position.occurrence().as_str())
                    .bind::<Text,_>(pair.snapshot.as_str()).bind::<BigInt,_>(pair.position.ordinal())
                    .bind::<Binary,_>(&pair.raw).bind::<Binary,_>(&pair.normalized)
                    .get_result::<ExistingPair>(connection).await.optional()?;
                if existing.ordinal != pair.position.ordinal() || identical.is_none() {
                    return Err(StoreError::Conflict);
                }
                return row.summary();
            }
            if row.source_ordinal.checked_add(1) != Some(pair.position.ordinal()) {
                return Err(StoreError::Conflict);
            }
            diesel::sql_query(dialect.query("INSERT INTO conversation_event_pairs(conversation_id,source_snapshot,ordinal,occurrence_id,raw_event,normalized_event) VALUES(?,?,?,?,?,?)"))
                .bind::<IdentitySql,_>(id).bind::<Text,_>(pair.snapshot.as_str()).bind::<BigInt,_>(pair.position.ordinal())
                .bind::<Text,_>(pair.position.occurrence().as_str()).bind::<Binary,_>(&pair.raw).bind::<Binary,_>(&pair.normalized)
                .execute(connection).await?;
            project_records!(connection, dialect, id, pair, records);
            diesel::sql_query(
                dialect.query(
                    "UPDATE conversations SET source_ordinal=?,source_event_id=? WHERE id=?",
                ),
            )
            .bind::<BigInt, _>(pair.position.ordinal())
            .bind::<Text, _>(pair.position.occurrence().as_str())
            .bind::<IdentitySql, _>(id)
            .execute(connection)
            .await?;
            conversation!(connection, dialect, id).summary()
        })
    }

    /// Complete one finite captured boundary; later append does not require native idle.
    pub async fn complete_conversation_source(
        &self,
        source: &ConnectorSource<'_>,
        conversation_id: &str,
        snapshot: &SnapshotId,
        through: Option<&SourcePosition>,
    ) -> Result<ConversationSummary, StoreError> {
        let id = EntityId::from_dto(EntityKind::Conversation, conversation_id)?;
        operation!(self, |connection, dialect| {
            let row = source_conversation!(connection, dialect, source, id);
            if row.source_snapshot.as_deref() != Some(snapshot.as_str()) {
                return Err(StoreError::Conflict);
            }
            if let Some(through) = through {
                let boundary=diesel::sql_query(dialect.query("SELECT ordinal FROM conversation_event_pairs WHERE conversation_id=? AND source_snapshot=? AND occurrence_id=? AND ordinal=?"))
                    .bind::<IdentitySql,_>(id).bind::<Text,_>(snapshot.as_str())
                    .bind::<Text,_>(through.occurrence().as_str()).bind::<BigInt,_>(through.ordinal())
                    .get_result::<ExistingPair>(connection).await.optional()?;
                if boundary.is_none() || row.source_ordinal < through.ordinal() {
                    return Err(StoreError::InvalidTransition);
                }
            }
            diesel::sql_query(
                dialect.query("UPDATE conversations SET source_state='READY' WHERE id=?"),
            )
            .bind::<IdentitySql, _>(id)
            .execute(connection)
            .await?;
            conversation!(connection, dialect, id).summary()
        })
    }

    pub async fn conversation_event_metadata(
        &self,
        actor: &SecretDigest,
        read: &EventMetadataRead<'_>,
        now: i64,
    ) -> Result<Vec<EventPairMetadata>, StoreError> {
        let EventMetadataRead {
            agent_id,
            conversation_id,
            snapshot,
            after,
            limit,
        } = *read;
        if !(1..=128).contains(&limit) {
            return Err(StoreError::InvalidInput);
        }
        let agent = EntityId::from_dto(EntityKind::Agent, agent_id)?;
        let id = EntityId::from_dto(EntityKind::Conversation, conversation_id)?;
        operation!(self, |connection, dialect| {
            let _ = owned_agent!(connection, dialect, actor, agent, now);
            let row = conversation!(connection, dialect, id);
            if row.agent_id != agent {
                return Err(StoreError::NotFound);
            }
            if row.source_snapshot.as_deref() != Some(snapshot.as_str()) {
                return Err(StoreError::Conflict);
            }
            if let Some(after) = after {
                let known=diesel::sql_query(dialect.query("SELECT ordinal FROM conversation_event_pairs WHERE conversation_id=? AND occurrence_id=? AND ordinal=?"))
                    .bind::<IdentitySql,_>(id).bind::<Text,_>(after.occurrence().as_str()).bind::<BigInt,_>(after.ordinal())
                    .get_result::<ExistingPair>(connection).await.optional()?;
                if known.is_none() {
                    return Err(StoreError::Conflict);
                }
            }
            let rows=diesel::sql_query(dialect.query("SELECT ordinal,occurrence_id,CAST(length(raw_event) AS BIGINT) AS raw_bytes,CAST(length(normalized_event) AS BIGINT) AS normalized_bytes FROM conversation_event_pairs WHERE conversation_id=? AND ordinal>? ORDER BY ordinal LIMIT ?"))
                .bind::<IdentitySql,_>(id).bind::<BigInt,_>(after.map_or(0,SourcePosition::ordinal))
                .bind::<BigInt,_>(limit).load::<MetadataRow>(connection).await?;
            rows.into_iter()
                .map(|row| {
                    Ok(EventPairMetadata {
                        position: SourcePosition::new(
                            row.ordinal,
                            EventOccurrenceId::parse(row.occurrence_id)
                                .map_err(|_| StoreError::Integrity)?,
                        )
                        .map_err(|_| StoreError::Integrity)?,
                        raw_bytes: row.raw_bytes,
                        normalized_bytes: row.normalized_bytes,
                    })
                })
                .collect()
        })
    }

    /// Source-qualified, byte-bounded continuation; never mix payloads across replacement.
    pub async fn conversation_event_chunk(
        &self,
        actor: &SecretDigest,
        read: &EventChunkRead<'_>,
        now: i64,
    ) -> Result<EventPayloadChunk, StoreError> {
        let EventChunkRead {
            agent_id,
            conversation_id,
            snapshot,
            occurrence,
            kind,
            offset,
            length,
        } = *read;
        if length == 0 || length > 32768 {
            return Err(StoreError::InvalidInput);
        }
        let start = i32::try_from(offset)
            .ok()
            .and_then(|offset| offset.checked_add(1))
            .ok_or(StoreError::InvalidInput)?;
        let length = i32::try_from(length).map_err(|_| StoreError::InvalidInput)?;
        let agent = EntityId::from_dto(EntityKind::Agent, agent_id)?;
        let id = EntityId::from_dto(EntityKind::Conversation, conversation_id)?;
        let column = match kind {
            EventPayloadKind::Raw => "raw_event",
            EventPayloadKind::Normalized => "normalized_event",
        };
        let sql = format!(
            "SELECT substr({column},?,?) AS bytes,CAST(length({column}) AS BIGINT) AS total_bytes FROM conversation_event_pairs WHERE conversation_id=? AND source_snapshot=? AND occurrence_id=?"
        );
        operation!(self, |connection, dialect| {
            let _ = owned_agent!(connection, dialect, actor, agent, now);
            let row = conversation!(connection, dialect, id);
            if row.agent_id != agent {
                return Err(StoreError::NotFound);
            }
            if row.source_snapshot.as_deref() != Some(snapshot.as_str()) {
                return Err(StoreError::Conflict);
            }
            let chunk = diesel::sql_query(dialect.query(&sql))
                .bind::<Integer, _>(start)
                .bind::<Integer, _>(length)
                .bind::<IdentitySql, _>(id)
                .bind::<Text, _>(snapshot.as_str())
                .bind::<Text, _>(occurrence.as_str())
                .get_result::<ChunkRow>(connection)
                .await
                .optional()?
                .ok_or(StoreError::NotFound)?;
            if i64::from(offset) > chunk.total_bytes {
                return Err(StoreError::InvalidInput);
            }
            Ok(EventPayloadChunk {
                bytes: chunk.bytes,
                offset,
                total_bytes: chunk.total_bytes,
            })
        })
    }
}
