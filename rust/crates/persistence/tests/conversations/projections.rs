use super::*;
use chapchoo_persistence::{
    ConversationFieldChange as Change, ConversationFieldChunkRead, ConversationFieldRead,
    ConversationFieldUpdate as FieldUpdate, ConversationRecordField as Field,
    ConversationRecordKind as Kind, ConversationRecordUpdate as Update, ConversationRegionRead,
};
fn update(kind: Kind, id: &str, change: Change) -> Update {
    Update {
        kind,
        native_id: id.into(),
        fields: vec![FieldUpdate {
            field: Field::Content,
            change,
        }],
    }
}
async fn ingest(
    store: &Store,
    f: &Fixture,
    conversation: &str,
    snapshot: &SnapshotId,
    ordinal: i64,
    records: &[Update],
) -> Result<CapturedEventPair, TestError> {
    let pair = CapturedEventPair {
        snapshot: snapshot.clone(),
        position: position(ordinal, &format!("event-{ordinal}"))?,
        raw: format!("raw-{ordinal}").into_bytes(),
        normalized: serde_json::to_vec(records)?,
    };
    store
        .ingest_conversation_records(
            &f.source(&f.connector.digest()),
            conversation,
            &pair,
            records,
        )
        .await?;
    Ok(pair)
}
async fn body(
    store: &Store,
    f: &Fixture,
    conversation: &str,
    snapshot: &SnapshotId,
    kind: Kind,
    id: &str,
    ordinal: i64,
) -> Result<Vec<u8>, TestError> {
    let mut bytes = Vec::new();
    loop {
        let chunk = store
            .conversation_field_chunk(
                &f.owner.digest(),
                &ConversationFieldChunkRead {
                    record: ConversationFieldRead {
                        agent_id: &f.agent.id,
                        conversation_id: conversation,
                        snapshot,
                        kind,
                        native_id: id,
                    },
                    field: Field::Content,
                    updated_ordinal: ordinal,
                    offset: u32::try_from(bytes.len())?,
                    length: 32768,
                },
                f.now,
            )
            .await?;
        bytes.extend(&chunk.bytes);
        if i64::try_from(bytes.len())? == chunk.total_bytes {
            return Ok(bytes);
        }
        assert!(!chunk.bytes.is_empty());
    }
}
pub(super) async fn exercise(
    store: &Store,
    f: &Fixture,
    conversation: &str,
    snapshot: &SnapshotId,
) -> Result<(), TestError> {
    let first = ingest(
        store,
        f,
        conversation,
        snapshot,
        1,
        &[update(
            Kind::Assistant,
            "z-first",
            Change::Append(b"first ".to_vec()),
        )],
    )
    .await?;
    let second = ingest(
        store,
        f,
        conversation,
        snapshot,
        2,
        &[update(
            Kind::Assistant,
            "a-second",
            Change::Append(b"second".to_vec()),
        )],
    )
    .await?;
    let third_records = [
        update(Kind::Assistant, "z-first", Change::Append(b"part".to_vec())),
        update(
            Kind::Thought,
            "z-first",
            Change::Append(b"activity only".to_vec()),
        ),
    ];
    let third = ingest(store, f, conversation, snapshot, 3, &third_records).await?;
    store
        .ingest_conversation_records(
            &f.source(&f.connector.digest()),
            conversation,
            &third,
            &third_records,
        )
        .await?;
    assert_eq!(
        body(
            store,
            f,
            conversation,
            snapshot,
            Kind::Assistant,
            "z-first",
            3
        )
        .await?,
        b"first part"
    );
    let page = store
        .conversation_region(
            &f.owner.digest(),
            &ConversationRegionRead {
                agent_id: &f.agent.id,
                conversation_id: conversation,
                snapshot,
                before: None,
                limit: 2,
            },
            f.now,
        )
        .await?;
    assert_eq!(page.len(), 2);
    assert_eq!(page[0].kind, Kind::Thought);
    assert_eq!(page[1].native_id, "a-second");
    let older = store
        .conversation_region(
            &f.owner.digest(),
            &ConversationRegionRead {
                agent_id: &f.agent.id,
                conversation_id: conversation,
                snapshot,
                before: Some(page[1].anchor),
                limit: 2,
            },
            f.now,
        )
        .await?;
    assert_eq!(older.len(), 1);
    assert_eq!(older[0].native_id, "z-first");
    let no_content = [Update {
        kind: Kind::Assistant,
        native_id: "z-first".into(),
        fields: vec![],
    }];
    ingest(store, f, conversation, snapshot, 4, &no_content).await?;
    assert_eq!(
        body(
            store,
            f,
            conversation,
            snapshot,
            Kind::Assistant,
            "z-first",
            3
        )
        .await?,
        b"first part"
    );
    let large = "한글\0full".repeat(20_000).into_bytes();
    ingest(
        store,
        f,
        conversation,
        snapshot,
        5,
        &[update(
            Kind::Assistant,
            "z-first",
            Change::Replace(large.clone()),
        )],
    )
    .await?;
    assert_eq!(
        body(
            store,
            f,
            conversation,
            snapshot,
            Kind::Assistant,
            "z-first",
            5
        )
        .await?,
        large
    );
    let stale = store
        .conversation_field_chunk(
            &f.owner.digest(),
            &ConversationFieldChunkRead {
                record: ConversationFieldRead {
                    agent_id: &f.agent.id,
                    conversation_id: conversation,
                    snapshot,
                    kind: Kind::Assistant,
                    native_id: "z-first",
                },
                field: Field::Content,
                updated_ordinal: 3,
                offset: 32768,
                length: 32768,
            },
            f.now,
        )
        .await;
    assert!(matches!(stale, Err(StoreError::Conflict)));
    ingest(
        store,
        f,
        conversation,
        snapshot,
        6,
        &[update(Kind::Assistant, "z-first", Change::Replace(vec![]))],
    )
    .await?;
    assert!(
        body(
            store,
            f,
            conversation,
            snapshot,
            Kind::Assistant,
            "z-first",
            6
        )
        .await?
        .is_empty()
    );
    let conflicting = [update(
        Kind::Assistant,
        "a-second",
        Change::Append(b"not in pair".to_vec()),
    )];
    assert!(matches!(
        store
            .ingest_conversation_records(
                &f.source(&f.connector.digest()),
                conversation,
                &second,
                &conflicting
            )
            .await,
        Err(StoreError::InvalidInput)
    ));
    let rows = store
        .conversation_fields(
            &f.owner.digest(),
            &ConversationFieldRead {
                agent_id: &f.agent.id,
                conversation_id: conversation,
                snapshot,
                kind: Kind::Thought,
                native_id: "z-first",
            },
            f.now,
        )
        .await?;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].bytes, 13);
    assert!(matches!(
        store
            .conversation_region(
                &f.other.digest(),
                &ConversationRegionRead {
                    agent_id: &f.agent.id,
                    conversation_id: conversation,
                    snapshot,
                    before: None,
                    limit: 25
                },
                f.now
            )
            .await,
        Err(StoreError::NotFound)
    ));
    // Reject the whole invalid normalized publication before publishing a
    // source pair, any candidate record or cursor movement.
    let invalid = [
        update(
            Kind::Assistant,
            "candidate",
            Change::Append(b"would be new".to_vec()),
        ),
        Update {
            kind: Kind::Tool,
            native_id: "duplicate-fields".into(),
            fields: vec![
                FieldUpdate {
                    field: Field::Content,
                    change: Change::Replace(vec![]),
                },
                FieldUpdate {
                    field: Field::Content,
                    change: Change::Append(vec![]),
                },
            ],
        },
    ];
    let pair = CapturedEventPair {
        snapshot: snapshot.clone(),
        position: position(7, "invalid")?,
        raw: b"raw".to_vec(),
        normalized: serde_json::to_vec(&invalid)?,
    };
    assert!(matches!(
        store
            .ingest_conversation_records(
                &f.source(&f.connector.digest()),
                conversation,
                &pair,
                &invalid
            )
            .await,
        Err(StoreError::InvalidInput)
    ));
    assert_eq!(
        store
            .current_conversation(&f.owner.digest(), &f.agent.id, f.now)
            .await?
            .ok_or("Conversation")?
            .source
            .position()
            .ok_or("Cursor")?
            .ordinal(),
        6
    );
    let current = store
        .current_conversation(&f.owner.digest(), &f.agent.id, f.now)
        .await?
        .ok_or("Conversation")?;
    let replacement = super::snapshot("projection-replacement")?;
    store
        .bind_conversation_source(
            &f.source(&f.connector.digest()),
            &SourceBinding {
                conversation_id: conversation.into(),
                expected_previous: Some(snapshot.clone()),
                snapshot: replacement.clone(),
                native_session: current.native_session_id.ok_or("Native")?,
            },
        )
        .await?;
    assert!(matches!(
        store
            .conversation_region(
                &f.owner.digest(),
                &ConversationRegionRead {
                    agent_id: &f.agent.id,
                    conversation_id: conversation,
                    snapshot,
                    before: None,
                    limit: 25
                },
                f.now
            )
            .await,
        Err(StoreError::Conflict)
    ));
    assert!(
        store
            .conversation_region(
                &f.owner.digest(),
                &ConversationRegionRead {
                    agent_id: &f.agent.id,
                    conversation_id: conversation,
                    snapshot: &replacement,
                    before: None,
                    limit: 25
                },
                f.now
            )
            .await?
            .is_empty()
    );
    assert!(
        store
            .conversation_fields(
                &f.owner.digest(),
                &ConversationFieldRead {
                    agent_id: &f.agent.id,
                    conversation_id: conversation,
                    snapshot: &replacement,
                    kind: Kind::Thought,
                    native_id: "z-first"
                },
                f.now
            )
            .await?
            .is_empty()
    );
    assert!(matches!(
        store
            .ingest_conversation_records(
                &f.source(&f.connector.digest()),
                conversation,
                &first,
                &[update(
                    Kind::Assistant,
                    "z-first",
                    Change::Append(b"first ".to_vec())
                )]
            )
            .await,
        Err(StoreError::Conflict)
    ));
    ingest(
        store,
        f,
        conversation,
        &replacement,
        1,
        &[update(
            Kind::Assistant,
            "new",
            Change::Replace(b"new source".to_vec()),
        )],
    )
    .await?;
    tool_io(store, f, conversation, &replacement).await?;
    Ok(())
}

async fn tool_io(
    store: &Store,
    f: &Fixture,
    conversation: &str,
    snapshot: &SnapshotId,
) -> Result<(), TestError> {
    let input = serde_json::to_vec(&serde_json::json!({
        "path":"notes.txt", "limit":9223372036854775807_i64
    }))?;
    let output = serde_json::to_vec(&serde_json::json!({"lines":["first","second"]}))?;
    ingest(
        store,
        f,
        conversation,
        snapshot,
        2,
        &[
            Update {
                kind: Kind::Tool,
                native_id: "raw-only".into(),
                fields: vec![
                    FieldUpdate {
                        field: Field::ToolInput,
                        change: Change::Replace(input.clone()),
                    },
                    FieldUpdate {
                        field: Field::ToolOutput,
                        change: Change::Replace(output.clone()),
                    },
                ],
            },
            update(
                Kind::Assistant,
                "retained-binary",
                Change::Replace("한글\0legacy".repeat(4_000).into_bytes()),
            ),
        ],
    )
    .await?;
    let record = || ConversationFieldRead {
        agent_id: &f.agent.id,
        conversation_id: conversation,
        snapshot,
        kind: Kind::Tool,
        native_id: "raw-only",
    };
    for (field, expected) in [(Field::ToolInput, &input), (Field::ToolOutput, &output)] {
        let chunk = store
            .conversation_field_chunk(
                &f.owner.digest(),
                &ConversationFieldChunkRead {
                    record: record(),
                    field,
                    updated_ordinal: 2,
                    offset: 0,
                    length: 32768,
                },
                f.now,
            )
            .await?;
        assert_eq!(&chunk.bytes, expected);
        assert_eq!(chunk.total_bytes, i64::try_from(expected.len())?);
    }
    ingest(
        store,
        f,
        conversation,
        snapshot,
        3,
        &[Update {
            kind: Kind::Tool,
            native_id: "raw-only".into(),
            fields: vec![FieldUpdate {
                field: Field::ToolInput,
                change: Change::Replace(Vec::new()),
            }],
        }],
    )
    .await?;
    let fields = store
        .conversation_fields(&f.owner.digest(), &record(), f.now)
        .await?;
    assert_eq!(fields.len(), 2);
    assert!(
        fields
            .iter()
            .any(|field| field.field == Field::ToolOutput && field.updated_ordinal == 2)
    );
    assert!(fields.iter().any(|field| field.field == Field::ToolInput
        && field.updated_ordinal == 3
        && field.bytes == 0));
    let cleared = store
        .conversation_field_chunk(
            &f.owner.digest(),
            &ConversationFieldChunkRead {
                record: record(),
                field: Field::ToolInput,
                updated_ordinal: 3,
                offset: 0,
                length: 32768,
            },
            f.now,
        )
        .await?;
    assert!(cleared.bytes.is_empty());
    assert_eq!(cleared.total_bytes, 0);
    Ok(())
}
