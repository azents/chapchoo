use super::*;
use chapchoo_persistence::{ConversationDeliveryTarget, ConversationOperationForwarding};

async fn exercise(
    store: &Store,
    f: &Fixture,
    mut sql: impl FnMut(&str) -> Result<(), TestError>,
    postgres: bool,
) -> Result<(), TestError> {
    let downgrade = if postgres {
        include_str!("../../migrations/postgres/00000000000022_consumed_inputs/down.sql")
    } else {
        include_str!("../../migrations/sqlite/00000000000022_consumed_inputs/down.sql")
    };
    // Model the old schema before its completion rows are seeded.
    sql(downgrade)?;
    let owner = f.owner.digest();
    let key = f.connector.digest();
    let source = f.source(&key);
    let first = store
        .accept_conversation_input(
            &owner,
            &AcceptConversationInput {
                agent_id: f.agent.id.clone(),
                conversation_id: None,
                request_id: "migration-source".into(),
                target: ConversationTarget::NotStarted,
                text: ConversationText::parse("An unresolved original input".into())?,
            },
            f.now,
        )
        .await?;
    let snapshot = snapshot("migration-source")?;
    let native = NativeConversationId::parse("migration-native".into())?;
    store
        .bind_conversation_source(
            &source,
            &SourceBinding {
                conversation_id: first.conversation_id.clone(),
                expected_previous: None,
                snapshot: snapshot.clone(),
                native_session: native.clone(),
            },
        )
        .await?;
    store
        .complete_conversation_source(&source, &first.conversation_id, &snapshot, None)
        .await?;
    let route = store
        .resolve_connector_route(&f.agent.computer.id)
        .await?
        .ok_or("Missing route")?;
    let target = ConversationDeliveryTarget {
        snapshot: snapshot.clone(),
        native_session: native,
    };
    let mut rows = Vec::new();
    for name in [
        "legacy-inserted",
        "legacy-steered",
        "legacy-failed",
        "legacy-possible",
        "legacy-other-source",
    ] {
        let receipt = store
            .accept_conversation_input(
                &owner,
                &AcceptConversationInput {
                    agent_id: f.agent.id.clone(),
                    conversation_id: Some(first.conversation_id.clone()),
                    request_id: name.into(),
                    target: ConversationTarget::Snapshot(snapshot.clone()),
                    text: ConversationText::parse(format!("Original {name}"))?,
                },
                f.now,
            )
            .await?;
        store
            .begin_conversation_input_forwarding(
                &owner,
                &route,
                &ConversationOperationForwarding {
                    agent_id: &f.agent.id,
                    conversation_id: &first.conversation_id,
                    operation_id: &receipt.id,
                    target: &target,
                },
                f.now,
            )
            .await?;
        rows.push(receipt);
    }
    // Seed the pre-upgrade completion representation through this migration's
    // isolated database fixture. Runtime product code never writes these states.
    sql(
        "UPDATE conversation_input_delivery SET delivery_state='INSERTED',native_message_id='legacy-native-message' WHERE operation_id IN (SELECT id FROM conversation_inputs WHERE request_id='legacy-inserted')",
    )?;
    sql(
        "UPDATE conversation_input_delivery SET delivery_state='STEERED' WHERE operation_id IN (SELECT id FROM conversation_inputs WHERE request_id='legacy-steered')",
    )?;
    sql(
        "UPDATE conversation_input_delivery SET delivery_state='FAILED' WHERE operation_id IN (SELECT id FROM conversation_inputs WHERE request_id='legacy-failed')",
    )?;
    sql(
        "UPDATE conversation_input_delivery SET delivery_state='STEERED',source_snapshot='different-source' WHERE operation_id IN (SELECT id FROM conversation_inputs WHERE request_id='legacy-other-source')",
    )?;
    let upgrade = if postgres {
        include_str!("../../migrations/postgres/00000000000022_consumed_inputs/up.sql")
    } else {
        include_str!("../../migrations/sqlite/00000000000022_consumed_inputs/up.sql")
    };
    sql(upgrade)?;
    sql(upgrade)?;
    for (index, receipt) in rows.iter().enumerate() {
        let remaining = store
            .conversation_input(
                &owner,
                &f.agent.id,
                &first.conversation_id,
                &receipt.request_id,
                f.now,
            )
            .await?;
        if index < 2 {
            assert!(remaining.is_none());
        } else {
            assert_eq!(
                remaining.ok_or("Non-consumed content was removed")?.text,
                receipt.text
            );
        }
    }
    assert!(
        store
            .conversation_input(
                &owner,
                &f.agent.id,
                &first.conversation_id,
                &first.request_id,
                f.now
            )
            .await?
            .is_some()
    );
    // Old connected writers can complete after the pre-upgrade sweep. Their
    // UPDATE and INSERT success representations must consume atomically too.
    for (name, write) in [
        (
            "late-inserted",
            "UPDATE conversation_input_delivery SET delivery_state='INSERTED',native_message_id='late-native-message' WHERE operation_id IN (SELECT id FROM conversation_inputs WHERE request_id='late-inserted')",
        ),
        (
            "late-steered",
            "UPDATE conversation_input_delivery SET delivery_state='STEERED' WHERE operation_id IN (SELECT id FROM conversation_inputs WHERE request_id='late-steered')",
        ),
        (
            "late-insert",
            "INSERT INTO conversation_input_delivery(operation_id,execution_id,source_snapshot,native_session_id,delivery_state) SELECT i.id,d.execution_id,d.source_snapshot,d.native_session_id,'STEERED' FROM conversation_inputs i CROSS JOIN conversation_input_delivery d WHERE i.request_id='late-insert' AND d.operation_id=(SELECT id FROM conversation_inputs WHERE request_id='legacy-possible')",
        ),
    ] {
        let receipt = store
            .accept_conversation_input(
                &owner,
                &AcceptConversationInput {
                    agent_id: f.agent.id.clone(),
                    conversation_id: Some(first.conversation_id.clone()),
                    request_id: name.into(),
                    target: ConversationTarget::Snapshot(snapshot.clone()),
                    text: ConversationText::parse(format!("Late {name}"))?,
                },
                f.now,
            )
            .await?;
        if name != "late-insert" {
            store
                .begin_conversation_input_forwarding(
                    &owner,
                    &route,
                    &ConversationOperationForwarding {
                        agent_id: &f.agent.id,
                        conversation_id: &first.conversation_id,
                        operation_id: &receipt.id,
                        target: &target,
                    },
                    f.now,
                )
                .await?;
        }
        sql(write)?;
        assert!(
            store
                .conversation_input_reference(
                    &owner,
                    &f.agent.id,
                    &first.conversation_id,
                    name,
                    f.now
                )
                .await?
                .is_none()
        );
    }
    sql(downgrade)?;
    assert!(
        store
            .conversation_input_reference(
                &owner,
                &f.agent.id,
                &first.conversation_id,
                &rows[0].request_id,
                f.now
            )
            .await?
            .is_none()
    );
    Ok(())
}

#[test]
fn sqlite_consumed_input_cleanup_preserves_unresolved_and_unbound_history() -> Result<(), TestError>
{
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("consumed.sqlite");
    let cipher = cipher()?;
    prepare_sqlite(&path, &cipher)?;
    let runtime = tokio::runtime::Runtime::new()?;
    let store = runtime.block_on(Store::connect_sqlite(path.clone(), cipher))?;
    let f = runtime.block_on(fixture(&store))?;
    let mut connection = diesel::SqliteConnection::establish(path.to_str().ok_or("Invalid path")?)?;
    connection.batch_execute("PRAGMA foreign_keys=ON")?;
    runtime.block_on(exercise(
        &store,
        &f,
        |query| {
            connection.batch_execute(query)?;
            Ok(())
        },
        false,
    ))
}

#[test]
fn postgres_consumed_input_cleanup_preserves_unresolved_and_unbound_history()
-> Result<(), TestError> {
    let _lock = POSTGRES_CONVERSATION_LOCK
        .lock()
        .map_err(|_| "Fixture lock poisoned")?;
    let base = std::env::var("CHAPCHOO_TEST_POSTGRES_URL")
        .map_err(|_| "Disposable PostgreSQL is required")?;
    let mut control = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&base)?;
    let suffix = BearerSecret::generate()?.digest().as_bytes()[..8]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let schema = format!("consumed_{suffix}");
    control.batch_execute(&format!("CREATE SCHEMA {schema}"))?;
    let separator = if base.contains('?') { '&' } else { '?' };
    let url = format!("{base}{separator}options=-csearch_path%3D{schema}");
    let result = (|| -> Result<(), TestError> {
        let cipher = cipher()?;
        prepare_postgres(&url, &cipher)?;
        let runtime = tokio::runtime::Runtime::new()?;
        let store = runtime.block_on(Store::connect_postgres(url.clone(), cipher))?;
        let f = runtime.block_on(fixture(&store))?;
        let mut connection = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&url)?;
        runtime.block_on(exercise(
            &store,
            &f,
            |query| {
                std::thread::scope(|scope| {
                    scope
                        .spawn(|| -> Result<(), TestError> {
                            connection.batch_execute(query)?;
                            Ok(())
                        })
                        .join()
                        .map_err(|_| "Migration fixture worker failed")?
                })
            },
            true,
        ))
    })();
    let cleanup = control.batch_execute(&format!("DROP SCHEMA {schema} CASCADE"));
    support::combine(result, cleanup.map_err(Into::into)).map(|_| ())
}
