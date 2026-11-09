use super::*;
use serde::{Deserialize, Serialize};

macro_rules! database_enum {
    ($name:ident { $($variant:ident=>$db:literal),+ $(,)? })=>{
        #[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize)]
        pub enum $name { $($variant),+ }
        impl $name {
            pub(super) fn database(self)->&'static str { match self { $(Self::$variant=>$db),+ } }
            fn parse(value:&str)->Result<Self,StoreError> { match value { $($db=>Ok(Self::$variant)),+, _=>Err(StoreError::Integrity) } }
        }
    };
}
database_enum!(ConversationRecordKind {
    User=>"USER",Assistant=>"ASSISTANT",Thought=>"THOUGHT",Tool=>"TOOL",Plan=>"PLAN",
    Terminal=>"TERMINAL",Request=>"REQUEST",State=>"STATE",Notice=>"NOTICE"
});
database_enum!(ConversationRecordField {
    Content=>"CONTENT",Title=>"TITLE",Status=>"STATUS",Kind=>"KIND",Locations=>"LOCATIONS",
    Description=>"DESCRIPTION",Request=>"REQUEST",Foreground=>"FOREGROUND"
});
#[derive(Clone, Serialize, Deserialize)]
pub enum ConversationFieldChange {
    Replace(Vec<u8>),
    Append(Vec<u8>),
}
#[derive(Clone, Serialize, Deserialize)]
pub struct ConversationFieldUpdate {
    pub field: ConversationRecordField,
    pub change: ConversationFieldChange,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct ConversationRecordUpdate {
    pub kind: ConversationRecordKind,
    pub native_id: String,
    pub fields: Vec<ConversationFieldUpdate>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConversationRecordAnchor {
    pub ordinal: i64,
    pub index: i64,
}
pub struct ConversationRecordMetadata {
    pub kind: ConversationRecordKind,
    pub native_id: String,
    pub anchor: ConversationRecordAnchor,
    pub updated_ordinal: i64,
}
pub struct ConversationFieldMetadata {
    pub field: ConversationRecordField,
    pub updated_ordinal: i64,
    pub bytes: i64,
}
pub struct ConversationRegionRead<'a> {
    pub agent_id: &'a str,
    pub conversation_id: &'a str,
    pub snapshot: &'a SnapshotId,
    pub before: Option<ConversationRecordAnchor>,
    pub limit: i64,
}
pub struct ConversationFieldRead<'a> {
    pub agent_id: &'a str,
    pub conversation_id: &'a str,
    pub snapshot: &'a SnapshotId,
    pub kind: ConversationRecordKind,
    pub native_id: &'a str,
}
pub struct ConversationFieldChunkRead<'a> {
    pub record: ConversationFieldRead<'a>,
    pub field: ConversationRecordField,
    pub updated_ordinal: i64,
    pub offset: u32,
    pub length: u32,
}
macro_rules! safe_debug {
    ($($name:ty),+)=>{$(
        impl fmt::Debug for $name {
            fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result { f.write_str(concat!(stringify!($name),"([REDACTED])")) }
        }
    )+};
}
safe_debug!(
    ConversationFieldChange,
    ConversationFieldUpdate,
    ConversationRecordUpdate,
    ConversationRecordMetadata,
    ConversationFieldMetadata,
    ConversationRegionRead<'_>,
    ConversationFieldRead<'_>,
    ConversationFieldChunkRead<'_>
);

pub(super) fn validate(records: &[ConversationRecordUpdate]) -> Result<Vec<u8>, StoreError> {
    if records.len() > 32 {
        return Err(StoreError::InvalidInput);
    }
    let mut keys = std::collections::BTreeSet::new();
    for record in records {
        if record.native_id.is_empty()
            || record.native_id.len() > 512
            || record.native_id.chars().any(char::is_control)
            || record.fields.len() > 8
            || !keys.insert((record.kind.database(), &record.native_id))
        {
            return Err(StoreError::InvalidInput);
        }
        let mut fields = std::collections::BTreeSet::new();
        for field in &record.fields {
            let bytes = match &field.change {
                ConversationFieldChange::Append(b) | ConversationFieldChange::Replace(b) => b,
            };
            if bytes.len() > 16 * 1024 * 1024 || !fields.insert(field.field.database()) {
                return Err(StoreError::InvalidInput);
            }
        }
    }
    serde_json::to_vec(records).map_err(|_| StoreError::InvalidInput)
}

macro_rules! project_records {
    ($connection:expr,$dialect:expr,$id:expr,$pair:expr,$records:expr)=>{{
        for (index,record) in $records.iter().enumerate() {
            let index=i64::try_from(index).map_err(|_|StoreError::InvalidInput)?;
            diesel::sql_query($dialect.query("INSERT INTO conversation_records(conversation_id,source_snapshot,record_kind,native_id,first_ordinal,first_index,updated_ordinal) VALUES(?,?,?,?,?,?,?) ON CONFLICT(conversation_id,record_kind,native_id) DO UPDATE SET updated_ordinal=excluded.updated_ordinal"))
                .bind::<IdentitySql,_>($id).bind::<Text,_>($pair.snapshot.as_str()).bind::<Text,_>(record.kind.database())
                .bind::<Text,_>(&record.native_id).bind::<BigInt,_>($pair.position.ordinal()).bind::<BigInt,_>(index)
                .bind::<BigInt,_>($pair.position.ordinal()).execute($connection).await?;
            for field in &record.fields {
                let (bytes,append)=match &field.change {
                    $crate::conversations::ConversationFieldChange::Replace(bytes)=>(bytes,false),
                    $crate::conversations::ConversationFieldChange::Append(bytes)=>(bytes,true),
                };
                let merge=if append {
                    match $dialect { Dialect::Postgres=>"conversation_record_fields.content || excluded.content",Dialect::Sqlite=>"CAST(conversation_record_fields.content || excluded.content AS BLOB)" }
                } else { "excluded.content" };
                let query=format!("INSERT INTO conversation_record_fields(conversation_id,record_kind,native_id,field_name,content,updated_ordinal) VALUES(?,?,?,?,?,?) ON CONFLICT(conversation_id,record_kind,native_id,field_name) DO UPDATE SET content={merge},updated_ordinal=excluded.updated_ordinal");
                diesel::sql_query($dialect.query(&query)).bind::<IdentitySql,_>($id)
                    .bind::<Text,_>(record.kind.database()).bind::<Text,_>(&record.native_id).bind::<Text,_>(field.field.database())
                    .bind::<Binary,_>(bytes).bind::<BigInt,_>($pair.position.ordinal()).execute($connection).await?;
            }
        }
    }};
}
pub(super) use project_records;

#[derive(diesel::QueryableByName)]
struct RecordRow {
    #[diesel(sql_type=Text)]
    record_kind: String,
    #[diesel(sql_type=Text)]
    native_id: String,
    #[diesel(sql_type=BigInt)]
    first_ordinal: i64,
    #[diesel(sql_type=BigInt)]
    first_index: i64,
    #[diesel(sql_type=BigInt)]
    updated_ordinal: i64,
}
#[derive(diesel::QueryableByName)]
struct FieldRow {
    #[diesel(sql_type=Text)]
    field_name: String,
    #[diesel(sql_type=BigInt)]
    updated_ordinal: i64,
    #[diesel(sql_type=BigInt)]
    bytes: i64,
}
impl Store {
    /// Keyset metadata only; large content has independently fenced continuations.
    pub async fn conversation_region(
        &self,
        actor: &SecretDigest,
        read: &ConversationRegionRead<'_>,
        now: i64,
    ) -> Result<Vec<ConversationRecordMetadata>, StoreError> {
        if !(1..=25).contains(&read.limit)
            || read.before.is_some_and(|a| a.ordinal < 1 || a.index < 0)
        {
            return Err(StoreError::InvalidInput);
        }
        let agent = EntityId::from_dto(EntityKind::Agent, read.agent_id)?;
        let id = EntityId::from_dto(EntityKind::Conversation, read.conversation_id)?;
        let anchor = read.before.unwrap_or(ConversationRecordAnchor {
            ordinal: i64::MAX,
            index: i64::MAX,
        });
        operation!(self, |connection, dialect| {
            let _ = owned_agent!(connection, dialect, actor, agent, now);
            let row = conversation!(connection, dialect, id);
            if row.agent_id != agent {
                return Err(StoreError::NotFound);
            }
            if row.source_snapshot.as_deref() != Some(read.snapshot.as_str()) {
                return Err(StoreError::Conflict);
            }
            let rows=diesel::sql_query(dialect.query("SELECT record_kind,native_id,first_ordinal,first_index,updated_ordinal FROM conversation_records WHERE conversation_id=? AND source_snapshot=? AND (first_ordinal<? OR (first_ordinal=? AND first_index<?)) ORDER BY first_ordinal DESC,first_index DESC LIMIT ?"))
                .bind::<IdentitySql,_>(id).bind::<Text,_>(read.snapshot.as_str())
                .bind::<BigInt,_>(anchor.ordinal).bind::<BigInt,_>(anchor.ordinal).bind::<BigInt,_>(anchor.index)
                .bind::<BigInt,_>(read.limit).load::<RecordRow>(connection).await?;
            rows.into_iter()
                .map(|r| {
                    Ok(ConversationRecordMetadata {
                        kind: ConversationRecordKind::parse(&r.record_kind)?,
                        native_id: r.native_id,
                        anchor: ConversationRecordAnchor {
                            ordinal: r.first_ordinal,
                            index: r.first_index,
                        },
                        updated_ordinal: r.updated_ordinal,
                    })
                })
                .collect()
        })
    }
    pub async fn conversation_fields(
        &self,
        actor: &SecretDigest,
        read: &ConversationFieldRead<'_>,
        now: i64,
    ) -> Result<Vec<ConversationFieldMetadata>, StoreError> {
        let agent = EntityId::from_dto(EntityKind::Agent, read.agent_id)?;
        let id = EntityId::from_dto(EntityKind::Conversation, read.conversation_id)?;
        operation!(self, |connection, dialect| {
            let _ = owned_agent!(connection, dialect, actor, agent, now);
            let row = conversation!(connection, dialect, id);
            if row.agent_id != agent {
                return Err(StoreError::NotFound);
            }
            if row.source_snapshot.as_deref() != Some(read.snapshot.as_str()) {
                return Err(StoreError::Conflict);
            }
            let rows=diesel::sql_query(dialect.query("SELECT field_name,updated_ordinal,CAST(length(content) AS BIGINT) AS bytes FROM conversation_record_fields WHERE conversation_id=? AND record_kind=? AND native_id=? ORDER BY field_name"))
                .bind::<IdentitySql,_>(id).bind::<Text,_>(read.kind.database()).bind::<Text,_>(read.native_id).load::<FieldRow>(connection).await?;
            rows.into_iter()
                .map(|r| {
                    Ok(ConversationFieldMetadata {
                        field: ConversationRecordField::parse(&r.field_name)?,
                        updated_ordinal: r.updated_ordinal,
                        bytes: r.bytes,
                    })
                })
                .collect()
        })
    }
    /// Bind every chunk to the same source and last content update, never mix bodies.
    pub async fn conversation_field_chunk(
        &self,
        actor: &SecretDigest,
        read: &ConversationFieldChunkRead<'_>,
        now: i64,
    ) -> Result<EventPayloadChunk, StoreError> {
        if read.length == 0 || read.length > 32768 || read.updated_ordinal < 1 {
            return Err(StoreError::InvalidInput);
        }
        let agent = EntityId::from_dto(EntityKind::Agent, read.record.agent_id)?;
        let id = EntityId::from_dto(EntityKind::Conversation, read.record.conversation_id)?;
        #[derive(diesel::QueryableByName)]
        struct Chunk {
            #[diesel(sql_type=Nullable<Binary>)]
            bytes: Option<Vec<u8>>,
            #[diesel(sql_type=BigInt)]
            total_bytes: i64,
        }
        operation!(self, |connection, dialect| {
            let _ = owned_agent!(connection, dialect, actor, agent, now);
            let row = conversation!(connection, dialect, id);
            if row.agent_id != agent {
                return Err(StoreError::NotFound);
            }
            if row.source_snapshot.as_deref() != Some(read.record.snapshot.as_str()) {
                return Err(StoreError::Conflict);
            }
            let query = match dialect {
                Dialect::Postgres => {
                    "SELECT substring(content FROM ? FOR ?) AS bytes,CAST(length(content) AS BIGINT) AS total_bytes FROM conversation_record_fields WHERE conversation_id=? AND record_kind=? AND native_id=? AND field_name=? AND updated_ordinal=?"
                }
                Dialect::Sqlite => {
                    "SELECT substr(content,?,?) AS bytes,CAST(length(content) AS BIGINT) AS total_bytes FROM conversation_record_fields WHERE conversation_id=? AND record_kind=? AND native_id=? AND field_name=? AND updated_ordinal=?"
                }
            };
            let offset = i32::try_from(read.offset)
                .map_err(|_| StoreError::InvalidInput)?
                .checked_add(1)
                .ok_or(StoreError::InvalidInput)?;
            let length = i32::try_from(read.length).map_err(|_| StoreError::InvalidInput)?;
            let chunk = diesel::sql_query(dialect.query(query))
                .bind::<diesel::sql_types::Integer, _>(offset)
                .bind::<diesel::sql_types::Integer, _>(length)
                .bind::<IdentitySql, _>(id)
                .bind::<Text, _>(read.record.kind.database())
                .bind::<Text, _>(read.record.native_id)
                .bind::<Text, _>(read.field.database())
                .bind::<BigInt, _>(read.updated_ordinal)
                .get_result::<Chunk>(connection)
                .await
                .optional()?
                .ok_or(StoreError::Conflict)?;
            if i64::from(read.offset) > chunk.total_bytes {
                return Err(StoreError::InvalidInput);
            }
            let bytes = if i64::from(read.offset) == chunk.total_bytes {
                // SQLite's empty computed BLOB is represented by a null data
                // pointer. Only an exactly empty suffix permits empty decoding.
                chunk.bytes.unwrap_or_default()
            } else {
                chunk.bytes.ok_or(StoreError::Integrity)?
            };
            Ok(EventPayloadChunk {
                bytes,
                offset: read.offset,
                total_bytes: chunk.total_bytes,
            })
        })
    }
}
