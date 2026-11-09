use super::*;
use chapchoo_service::{
    ConversationFieldChunkRead, ConversationFieldRead, ConversationRecordAnchor,
    ConversationRecordField as Field, ConversationRecordKind as Kind, ConversationRegionRead,
};
macro_rules! typed_enum {
    ($parse:ident,$wire:ident,$domain:ident,[$($variant:ident),+]) => {
        fn $parse(value: i32) -> Result<$domain, Status> {
            match wire::$wire::try_from(value) {
                $(Ok(wire::$wire::$variant) => Ok($domain::$variant),)+
                _ => Err(Status::invalid_argument(concat!(stringify!($wire), " is invalid"))),
            }
        }
    };
}
typed_enum!(
    kind,
    RecordKind,
    Kind,
    [
        User, Assistant, Thought, Tool, Plan, Terminal, Request, State, Notice
    ]
);
typed_enum!(
    field,
    RecordField,
    Field,
    [
        Content,
        Title,
        Status,
        Kind,
        Locations,
        Description,
        Request,
        Foreground
    ]
);
fn wire_kind(value: Kind) -> i32 {
    (match value {
        Kind::User => wire::RecordKind::User,
        Kind::Assistant => wire::RecordKind::Assistant,
        Kind::Thought => wire::RecordKind::Thought,
        Kind::Tool => wire::RecordKind::Tool,
        Kind::Plan => wire::RecordKind::Plan,
        Kind::Terminal => wire::RecordKind::Terminal,
        Kind::Request => wire::RecordKind::Request,
        Kind::State => wire::RecordKind::State,
        Kind::Notice => wire::RecordKind::Notice,
    }) as i32
}
fn wire_field(value: Field) -> i32 {
    (match value {
        Field::Content => wire::RecordField::Content,
        Field::Title => wire::RecordField::Title,
        Field::Status => wire::RecordField::Status,
        Field::Kind => wire::RecordField::Kind,
        Field::Locations => wire::RecordField::Locations,
        Field::Description => wire::RecordField::Description,
        Field::Request => wire::RecordField::Request,
        Field::Foreground => wire::RecordField::Foreground,
    }) as i32
}
fn source(value: Option<wire::RecordSource>) -> Result<(wire::RecordSource, SnapshotId), Status> {
    let value = value.ok_or_else(|| Status::invalid_argument("Record source is required"))?;
    let snapshot = SnapshotId::parse(value.snapshot_id.clone())
        .map_err(|_| Status::invalid_argument("Record snapshot is invalid"))?;
    Ok((value, snapshot))
}
pub(super) async fn region(
    service: &ConversationServices,
    request: Request<wire::OwnerRecordsRequest>,
) -> Result<Response<wire::RecordRegion>, Status> {
    service.admit()?;
    let actor = actor(&request)?;
    let value = request.into_inner();
    let (source, snapshot) = source(value.source)?;
    let records = service
        .conversation
        .records(
            &actor,
            &ConversationRegionRead {
                agent_id: &source.agent_id,
                conversation_id: &source.conversation_id,
                snapshot: &snapshot,
                before: value.before.map(|anchor| ConversationRecordAnchor {
                    ordinal: anchor.ordinal,
                    index: anchor.index,
                }),
                limit: i64::from(value.limit),
            },
            now()?,
        )
        .await
        .map_err(status)?;
    Ok(Response::new(wire::RecordRegion {
        records: records
            .into_iter()
            .map(|record| wire::RecordMetadata {
                kind: wire_kind(record.kind),
                native_id: record.native_id,
                anchor: Some(wire::RecordAnchor {
                    ordinal: record.anchor.ordinal,
                    index: record.anchor.index,
                }),
                updated_ordinal: record.updated_ordinal,
            })
            .collect(),
    }))
}
pub(super) async fn fields(
    service: &ConversationServices,
    request: Request<wire::OwnerFieldsRequest>,
) -> Result<Response<wire::RecordFields>, Status> {
    service.admit()?;
    let actor = actor(&request)?;
    let value = request.into_inner();
    let (source, snapshot) = source(value.source)?;
    let fields = service
        .conversation
        .fields(
            &actor,
            &ConversationFieldRead {
                agent_id: &source.agent_id,
                conversation_id: &source.conversation_id,
                snapshot: &snapshot,
                kind: kind(value.kind)?,
                native_id: &value.native_id,
            },
            now()?,
        )
        .await
        .map_err(status)?;
    Ok(Response::new(wire::RecordFields {
        fields: fields
            .into_iter()
            .map(|value| wire::FieldMetadata {
                field: wire_field(value.field),
                updated_ordinal: value.updated_ordinal,
                total_bytes: value.bytes,
            })
            .collect(),
    }))
}
pub(super) async fn read(
    service: &ConversationServices,
    request: Request<wire::OwnerFieldRead>,
) -> Result<Response<wire::FieldChunk>, Status> {
    service.admit()?;
    let actor = actor(&request)?;
    let value = request.into_inner();
    let record = value
        .record
        .ok_or_else(|| Status::invalid_argument("Record is required"))?;
    let (source, snapshot) = source(record.source)?;
    let chunk = service
        .conversation
        .field_chunk(
            &actor,
            &ConversationFieldChunkRead {
                record: ConversationFieldRead {
                    agent_id: &source.agent_id,
                    conversation_id: &source.conversation_id,
                    snapshot: &snapshot,
                    kind: kind(record.kind)?,
                    native_id: &record.native_id,
                },
                field: field(value.field)?,
                updated_ordinal: value.updated_ordinal,
                offset: value.offset,
                length: value.length,
            },
            now()?,
        )
        .await
        .map_err(status)?;
    Ok(Response::new(wire::FieldChunk {
        content: chunk.bytes,
        offset: chunk.offset,
        total_bytes: chunk.total_bytes,
    }))
}
