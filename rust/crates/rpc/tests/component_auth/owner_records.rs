use super::*;
use chapchoo_protocol::conversations::{self as c, conversations_client::ConversationsClient};
pub(super) async fn exercise(
    context: &super::capture::Context<'_>,
    conversation: &str,
    expected_text: &str,
) -> Result<(), TestError> {
    let mut owner = ConversationsClient::connect(context.endpoint.to_owned())
        .await?
        .max_encoding_message_size(64 * 1024)
        .max_decoding_message_size(64 * 1024);
    let source = c::RecordSource {
        agent_id: context.agent.id.clone(),
        conversation_id: conversation.into(),
        snapshot_id: "rpc-source".into(),
    };
    let region = c::OwnerRecordsRequest {
        source: Some(source.clone()),
        before: None,
        limit: 25,
    };
    let records = owner
        .records(authenticated(region.clone(), context.user_key)?)
        .await?
        .into_inner()
        .records;
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].kind, c::RecordKind::Assistant as i32);
    assert_eq!(records[0].native_id, "message");
    assert!(
        owner
            .records(authenticated(
                c::OwnerRecordsRequest {
                    before: records[0].anchor,
                    ..region.clone()
                },
                context.user_key
            )?)
            .await?
            .into_inner()
            .records
            .is_empty()
    );
    assert_eq!(
        owner
            .records(authenticated(
                c::OwnerRecordsRequest {
                    limit: 26,
                    ..region.clone()
                },
                context.user_key
            )?)
            .await
            .err()
            .ok_or("Unbounded region accepted")?
            .code(),
        Code::InvalidArgument
    );
    let record = c::OwnerFieldsRequest {
        source: Some(source),
        kind: c::RecordKind::Assistant as i32,
        native_id: "message".into(),
    };
    let fields = owner
        .fields(authenticated(record.clone(), context.user_key)?)
        .await?
        .into_inner()
        .fields;
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].field, c::RecordField::Content as i32);
    let read = c::OwnerFieldRead {
        record: Some(record),
        field: fields[0].field,
        updated_ordinal: fields[0].updated_ordinal,
        offset: 0,
        length: 32768,
    };
    assert!(fields[0].total_bytes > 64 * 1024);
    let mut complete = Vec::new();
    loop {
        let chunk = owner
            .read_field(authenticated(
                c::OwnerFieldRead {
                    offset: u32::try_from(complete.len())?,
                    ..read.clone()
                },
                context.user_key,
            )?)
            .await?
            .into_inner();
        assert_eq!(chunk.offset as usize, complete.len());
        assert_eq!(chunk.total_bytes, fields[0].total_bytes);
        if chunk.content.is_empty() {
            break;
        }
        complete.extend(chunk.content);
    }
    assert_eq!(i64::try_from(complete.len())?, fields[0].total_bytes);
    let expected = format!("{{\"type\":\"text\",\"text\":\"{expected_text}\"}}\n");
    assert_eq!(complete, expected.as_bytes());
    assert_eq!(
        owner
            .read_field(authenticated(
                c::OwnerFieldRead {
                    updated_ordinal: read.updated_ordinal + 1,
                    ..read.clone()
                },
                context.user_key
            )?)
            .await
            .err()
            .ok_or("Changed content identity accepted")?
            .code(),
        Code::AlreadyExists
    );
    assert_eq!(
        owner
            .records(authenticated(region.clone(), context.key)?)
            .await
            .err()
            .ok_or("Component gained owner projection")?
            .code(),
        Code::Unauthenticated
    );
    let mut stale = region;
    stale.source.as_mut().ok_or("Missing source")?.snapshot_id = "not-current".into();
    assert_eq!(
        owner
            .records(authenticated(stale, context.user_key)?)
            .await
            .err()
            .ok_or("Wrong snapshot accepted")?
            .code(),
        Code::AlreadyExists
    );
    Ok(())
}
