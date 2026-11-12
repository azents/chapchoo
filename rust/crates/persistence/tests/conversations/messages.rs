use super::*;
use chapchoo_persistence::{
    ConversationDeliveryTarget, ConversationInputOutcome, ConversationInputScope,
    ConversationMessageRead, ConversationMessageRole, ConversationMessageScope,
    ConversationMessageWindow, ConversationOperationForwarding, PublishConversationMessage,
};

async fn exercise(store: &Store, f: &Fixture) -> Result<String, TestError> {
    let actor = f.owner.digest();
    let original = "  사용자 원문 **보존** 🙂\n두 번째 줄  ";
    let draft = AcceptConversationInput {
        agent_id: f.agent.id.clone(),
        conversation_id: None,
        request_id: "chat-first".into(),
        target: ConversationTarget::NotStarted,
        text: ConversationText::parse(original.into())?,
    };
    // Deliberately skew the application policy clock while the test database
    // retains the host clock, so receipt time cannot pass by copying acceptance.
    let acceptance_now = f.now + 60_000;
    let receipt_before = now()? / 1000 * 1000;
    let input = store
        .accept_conversation_input(&actor, &draft, acceptance_now)
        .await?;
    let receipt_after = now()?;
    let recovered = store
        .accept_conversation_input(&actor, &draft, acceptance_now)
        .await?;
    assert_eq!(input.id, recovered.id);
    let conversation = input.conversation_id.clone();
    let scope = ConversationMessageScope {
        agent_id: &f.agent.id,
        conversation_id: &conversation,
    };
    let first = store
        .conversation_messages(&actor, &scope, ConversationMessageWindow::Latest, 25, f.now)
        .await?;
    assert_eq!(first.messages.len(), 1);
    assert_eq!(first.through, 1);
    let first = first
        .messages
        .into_iter()
        .next()
        .ok_or("Missing first message")?;
    assert_eq!(first.receipt.role, ConversationMessageRole::User);
    assert_eq!(
        first.receipt.source_input_id.as_deref(),
        Some(input.id.as_str())
    );
    assert_eq!(first.receipt.original_accepted_at, Some(acceptance_now));
    assert!((receipt_before..=receipt_after).contains(&first.receipt.received_at));
    assert_eq!(
        first.receipt.source_input_request_id.as_deref(),
        Some("chat-first")
    );
    assert_eq!(first.bytes, i64::try_from(original.len())?);
    let mut bytes = Vec::new();
    for offset in 0..=u32::try_from(original.len())? {
        let chunk = store
            .conversation_message_chunk(
                &actor,
                &ConversationMessageRead {
                    scope: ConversationMessageScope {
                        agent_id: &f.agent.id,
                        conversation_id: &conversation,
                    },
                    message_id: &first.receipt.id,
                    offset,
                    length: 1,
                },
                f.now,
            )
            .await?;
        assert_eq!(chunk.total_bytes, first.bytes);
        bytes.extend(chunk.bytes);
    }
    assert_eq!(String::from_utf8(bytes)?, original);
    assert!(!format!("{:?}", first.receipt).contains(original));
    assert!(matches!(
        store
            .conversation_messages(
                &f.other.digest(),
                &scope,
                ConversationMessageWindow::Latest,
                25,
                f.now
            )
            .await,
        Err(StoreError::NotFound)
    ));
    for (offset, length) in [
        (u32::try_from(original.len())? + 1, 1),
        (0, 0),
        (0, 32769),
        (u32::MAX, 1),
    ] {
        assert!(matches!(
            store
                .conversation_message_chunk(
                    &actor,
                    &ConversationMessageRead {
                        scope: ConversationMessageScope {
                            agent_id: &f.agent.id,
                            conversation_id: &conversation
                        },
                        message_id: &first.receipt.id,
                        offset,
                        length
                    },
                    f.now
                )
                .await,
            Err(StoreError::InvalidInput)
        ));
    }
    assert!(
        store
            .conversation_messages(&actor, &scope, ConversationMessageWindow::Latest, 26, f.now)
            .await
            .is_err()
    );
    let snapshot_a = snapshot("message-source-a")?;
    let native = NativeConversationId::parse("message-native".into())?;
    let key = f.connector.digest();
    // An actual tool request uses current component/domain authority before
    // setup knows a native ID. This does not add automatic startup execution.
    assert!(matches!(
        store
            .current_conversation(&actor, &f.agent.id, f.now)
            .await?
            .ok_or("Missing product conversation")?
            .source,
        ConversationSource::NotStarted
    ));
    let text = ConversationText::parse("Agent domain receipt".into())?;
    let published = store
        .publish_conversation_message(
            &f.source(&key),
            &PublishConversationMessage {
                agent_id: &f.agent.id,
                conversation_id: &conversation,
                text: &text,
            },
        )
        .await?;
    assert_eq!(published.receipt_order, 2);
    assert_eq!(published.role, ConversationMessageRole::Agent);
    assert!(published.source_input_id.is_none());
    assert!(published.source_input_request_id.is_none());
    assert!(matches!(
        store
            .publish_conversation_message(
                &f.source(&key),
                &PublishConversationMessage {
                    agent_id: &f.other_agent,
                    conversation_id: &conversation,
                    text: &text
                }
            )
            .await,
        Err(StoreError::NotFound)
    ));
    let invalid = BearerSecret::generate()?.digest();
    assert!(matches!(
        store
            .publish_conversation_message(
                &f.source(&invalid),
                &PublishConversationMessage {
                    agent_id: &f.agent.id,
                    conversation_id: &conversation,
                    text: &text
                }
            )
            .await,
        Err(StoreError::Unauthenticated)
    ));

    store
        .bind_conversation_source(
            &f.source(&key),
            &SourceBinding {
                conversation_id: conversation.clone(),
                expected_previous: None,
                snapshot: snapshot_a.clone(),
                native_session: native.clone(),
            },
        )
        .await?;
    store
        .complete_conversation_source(&f.source(&key), &conversation, &snapshot_a, None)
        .await?;
    exercise_delivery_retention(store, f, &input, &snapshot_a, &native).await?;

    // Concurrent writers share one conversation allocation, not timestamp or UUID order.
    let mut tasks = tokio::task::JoinSet::new();
    let actor = Arc::new(actor);
    let key = Arc::new(key);
    for index in 0..8 {
        let store = store.clone();
        let actor = actor.clone();
        let key = key.clone();
        let agent = f.agent.id.clone();
        let conversation = conversation.clone();
        let snapshot = snapshot_a.clone();
        let session = f.session.clone();
        let control = f.control.clone();
        let connection = f.connection.clone();
        let now = f.now;
        tasks.spawn(async move {
            let text = ConversationText::parse("Intentionally repeated body".into())
                .map_err(|_| StoreError::InvalidInput)?;
            if index % 2 == 0 {
                store
                    .accept_conversation_input(
                        &actor,
                        &AcceptConversationInput {
                            agent_id: agent,
                            conversation_id: Some(conversation),
                            request_id: format!("concurrent-chat-{index}"),
                            target: ConversationTarget::Snapshot(snapshot),
                            text,
                        },
                        now,
                    )
                    .await?;
            } else {
                store
                    .publish_conversation_message(
                        &ConnectorSource {
                            session_id: &session,
                            control_id: &control,
                            connection_id: &connection,
                            credential: &key,
                        },
                        &PublishConversationMessage {
                            agent_id: &agent,
                            conversation_id: &conversation,
                            text: &text,
                        },
                    )
                    .await?;
            }
            Ok::<(), StoreError>(())
        });
    }
    while let Some(result) = tasks.join_next().await {
        result??;
    }
    let all = store
        .conversation_messages(&actor, &scope, ConversationMessageWindow::Latest, 25, f.now)
        .await?;
    assert_eq!(all.messages.len(), 11);
    assert_eq!(
        all.messages
            .iter()
            .map(|m| m.receipt.receipt_order)
            .collect::<Vec<_>>(),
        (1..=11).collect::<Vec<_>>()
    );
    let older = store
        .conversation_messages(
            &actor,
            &scope,
            ConversationMessageWindow::Before(5),
            2,
            f.now,
        )
        .await?;
    assert_eq!(
        older
            .messages
            .iter()
            .map(|m| m.receipt.receipt_order)
            .collect::<Vec<_>>(),
        vec![3, 4]
    );
    let bounded = store
        .conversation_messages(
            &actor,
            &scope,
            ConversationMessageWindow::After {
                order: 0,
                through: Some(3),
            },
            25,
            f.now,
        )
        .await?;
    assert_eq!(bounded.through, 3);
    assert_eq!(bounded.messages.len(), 3);
    assert!(
        store
            .conversation_messages(
                &actor,
                &scope,
                ConversationMessageWindow::After {
                    order: 3,
                    through: Some(3)
                },
                25,
                f.now
            )
            .await?
            .messages
            .is_empty()
    );
    assert!(
        store
            .conversation_messages(
                &actor,
                &scope,
                ConversationMessageWindow::After {
                    order: 0,
                    through: Some(12)
                },
                25,
                f.now
            )
            .await
            .is_err()
    );
    let ids = all
        .messages
        .iter()
        .map(|m| m.receipt.id.clone())
        .collect::<Vec<_>>();
    let snapshot_b = snapshot("message-source-b")?;
    store
        .bind_conversation_source(
            &f.source(&key),
            &SourceBinding {
                conversation_id: conversation.clone(),
                expected_previous: Some(snapshot_a),
                snapshot: snapshot_b.clone(),
                native_session: native,
            },
        )
        .await?;
    store
        .complete_conversation_source(&f.source(&key), &conversation, &snapshot_b, None)
        .await?;
    assert!(
        store
            .conversation_input_reference(&actor, &f.agent.id, &conversation, "chat-first", f.now)
            .await?
            .is_none()
    );
    let retained = store
        .conversation_messages(&actor, &scope, ConversationMessageWindow::Latest, 25, f.now)
        .await?;
    assert_eq!(
        retained
            .messages
            .iter()
            .map(|m| m.receipt.id.clone())
            .collect::<Vec<_>>(),
        ids
    );
    // This nonce is provenance only. Once its pending operation has ended, an
    // explicit new submission is not permanently deduplicated through chat.
    let fresh = store
        .accept_conversation_input(
            &actor,
            &AcceptConversationInput {
                agent_id: f.agent.id.clone(),
                conversation_id: Some(conversation.clone()),
                request_id: "chat-first".into(),
                target: ConversationTarget::Snapshot(snapshot_b),
                text: ConversationText::parse("An explicit new submission".into())?,
            },
            f.now,
        )
        .await?;
    assert_ne!(fresh.id, input.id);
    let after = store
        .conversation_messages(&actor, &scope, ConversationMessageWindow::Latest, 25, f.now)
        .await?;
    assert_eq!(after.messages.len(), 12);
    assert_eq!(
        after
            .messages
            .iter()
            .filter(|m| m.receipt.source_input_request_id.as_deref() == Some("chat-first"))
            .count(),
        2
    );
    Ok(conversation)
}

async fn exercise_delivery_retention(
    store: &Store,
    f: &Fixture,
    first: &chapchoo_persistence::ConversationInputReceipt,
    snapshot: &SnapshotId,
    native: &NativeConversationId,
) -> Result<(), TestError> {
    let owner = f.owner.digest();
    let key = f.connector.digest();
    let route = store
        .resolve_connector_route(&f.agent.computer.id)
        .await?
        .ok_or("Missing route")?;
    let target = ConversationDeliveryTarget {
        snapshot: snapshot.clone(),
        native_session: native.clone(),
    };
    let forwarding = ConversationOperationForwarding {
        agent_id: &f.agent.id,
        conversation_id: &first.conversation_id,
        operation_id: &first.id,
        target: &target,
    };
    store
        .begin_conversation_input_forwarding(&owner, &route, &forwarding, f.now)
        .await?;
    store
        .complete_conversation_input_forwarding(
            &f.source(&key),
            &forwarding,
            &ConversationInputOutcome::Steered,
        )
        .await?;
    assert!(
        store
            .conversation_input_reference(
                &owner,
                &f.agent.id,
                &first.conversation_id,
                &first.request_id,
                f.now
            )
            .await?
            .is_none()
    );
    let failed = store
        .accept_conversation_input(
            &owner,
            &AcceptConversationInput {
                agent_id: f.agent.id.clone(),
                conversation_id: Some(first.conversation_id.clone()),
                request_id: "failed-chat-input".into(),
                target: ConversationTarget::Snapshot(snapshot.clone()),
                text: ConversationText::parse(
                    "Accepted chat survives failed delivery deletion".into(),
                )?,
            },
            f.now,
        )
        .await?;
    let forwarding = ConversationOperationForwarding {
        operation_id: &failed.id,
        ..forwarding
    };
    store
        .begin_conversation_input_forwarding(&owner, &route, &forwarding, f.now)
        .await?;
    store
        .complete_conversation_input_forwarding(
            &f.source(&key),
            &forwarding,
            &ConversationInputOutcome::Failed,
        )
        .await?;
    store
        .delete_failed_conversation_input(
            &owner,
            &ConversationInputScope {
                agent_id: &f.agent.id,
                conversation_id: &first.conversation_id,
                target: &ConversationTarget::Snapshot(snapshot.clone()),
            },
            &failed.request_id,
            f.now,
        )
        .await?;
    let scope = ConversationMessageScope {
        agent_id: &f.agent.id,
        conversation_id: &first.conversation_id,
    };
    let messages = store
        .conversation_messages(&owner, &scope, ConversationMessageWindow::Latest, 25, f.now)
        .await?;
    assert_eq!(messages.messages.len(), 3);
    for input in [first, &failed] {
        let message = messages
            .messages
            .iter()
            .find(|message| message.receipt.source_input_id.as_deref() == Some(input.id.as_str()))
            .ok_or("Chat disappeared with delivery data")?;
        let chunk = store
            .conversation_message_chunk(
                &owner,
                &ConversationMessageRead {
                    scope: ConversationMessageScope {
                        agent_id: &f.agent.id,
                        conversation_id: &first.conversation_id,
                    },
                    message_id: &message.receipt.id,
                    offset: 0,
                    length: 32768,
                },
                f.now,
            )
            .await?;
        assert_eq!(String::from_utf8(chunk.bytes)?, input.text.as_str());
    }
    Ok(())
}

async fn verify_bound_conversation(
    store: &Store,
    f: &Fixture,
    conversation: &str,
) -> Result<(), TestError> {
    assert!(
        store
            .current_conversation(&f.owner.digest(), &f.agent.id, f.now)
            .await?
            .is_none()
    );
    let key = f.connector.digest();
    let message = store
        .publish_conversation_message(
            &f.source(&key),
            &PublishConversationMessage {
                agent_id: &f.agent.id,
                conversation_id: conversation,
                text: &ConversationText::parse(
                    "The actual caller binding is not UI selection".into(),
                )?,
            },
        )
        .await?;
    let region = store
        .conversation_messages(
            &f.owner.digest(),
            &ConversationMessageScope {
                agent_id: &f.agent.id,
                conversation_id: conversation,
            },
            ConversationMessageWindow::Latest,
            1,
            f.now,
        )
        .await?;
    assert_eq!(
        region
            .messages
            .first()
            .ok_or("Missing bound publication")?
            .receipt
            .id,
        message.id
    );
    assert!(
        store
            .current_conversation(&f.owner.digest(), &f.agent.id, f.now)
            .await?
            .is_none()
    );
    Ok(())
}

async fn verify_rollback(
    store: &Store,
    f: &Fixture,
    conversation: &str,
    expected: &str,
) -> Result<(), TestError> {
    let actor = f.owner.digest();
    let scope = ConversationMessageScope {
        agent_id: &f.agent.id,
        conversation_id: conversation,
    };
    let before = store
        .conversation_messages(&actor, &scope, ConversationMessageWindow::Latest, 25, f.now)
        .await?;
    let result = store
        .accept_conversation_input(
            &actor,
            &AcceptConversationInput {
                agent_id: f.agent.id.clone(),
                conversation_id: Some(conversation.into()),
                request_id: "rollback-chat".into(),
                target: ConversationTarget::Snapshot(snapshot("message-source-b")?),
                text: ConversationText::parse("Must roll back both receipts".into())?,
            },
            f.now,
        )
        .await;
    match result {
        Err(StoreError::Database(diesel::result::Error::DatabaseError(_, information))) => {
            assert_eq!(information.message(), expected);
        }
        _ => return Err("Receipt did not encounter the injected database failure".into()),
    }
    let after = store
        .conversation_messages(&actor, &scope, ConversationMessageWindow::Latest, 25, f.now)
        .await?;
    assert_eq!(before.through, after.through);
    assert_eq!(before.messages.len(), after.messages.len());
    assert!(
        store
            .conversation_input_reference(&actor, &f.agent.id, conversation, "rollback-chat", f.now)
            .await?
            .is_none()
    );
    Ok(())
}
const OLD_WRITER: &str = "INSERT INTO conversation_inputs(id,conversation_id,request_id,target_snapshot,submitted_text,accepted_at) SELECT c.id,c.id,'old-unreceipted-writer',c.source_snapshot,'Must not be accepted',0 FROM conversations c JOIN agents a ON a.id=c.agent_id WHERE a.handle='chat-fixture'";
const SQLITE_ROLLBACK: &str = "CREATE TRIGGER reject_chat_fixture BEFORE INSERT ON conversation_inputs WHEN NEW.request_id='rollback-chat' BEGIN SELECT RAISE(ABORT,'Injected input failure'); END";
const POSTGRES_ROLLBACK: &str = "CREATE FUNCTION reject_chat_fixture() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF NEW.request_id='rollback-chat' THEN RAISE EXCEPTION 'Injected input failure'; END IF; RETURN NEW; END; $$; CREATE TRIGGER reject_chat_fixture BEFORE INSERT ON conversation_inputs FOR EACH ROW EXECUTE FUNCTION reject_chat_fixture()";
const SQLITE_CHAT_ROLLBACK: &str = "CREATE TRIGGER reject_chat_write_fixture BEFORE INSERT ON conversation_messages WHEN NEW.body='Must roll back both receipts' BEGIN SELECT RAISE(ABORT,'Injected chat failure'); END";
const POSTGRES_CHAT_ROLLBACK: &str = "CREATE FUNCTION reject_chat_write_fixture() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF NEW.body='Must roll back both receipts' THEN RAISE EXCEPTION 'Injected chat failure'; END IF; RETURN NEW; END; $$; CREATE TRIGGER reject_chat_write_fixture BEFORE INSERT ON conversation_messages FOR EACH ROW EXECUTE FUNCTION reject_chat_write_fixture()";
macro_rules! verify_owned_cascade {
    ($connection:expr) => {{
        $connection.batch_execute("DELETE FROM conversations")?;
        #[derive(diesel::QueryableByName)]
        struct Children { #[diesel(sql_type=diesel::sql_types::BigInt)] n:i64 }
        assert_eq!(diesel::sql_query("SELECT CAST((SELECT count(*) FROM conversation_messages)+(SELECT count(*) FROM conversation_message_heads) AS BIGINT) AS n")
            .get_result::<Children>($connection)?.n,0);
    }};
}

#[test]
fn sqlite_chat_receipts_and_lifetime() -> Result<(), TestError> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("chat.sqlite");
    let cipher = cipher()?;
    prepare_sqlite(&path, &cipher)?;
    let runtime = tokio::runtime::Runtime::new()?;
    let store = runtime.block_on(Store::connect_sqlite(path.clone(), cipher))?;
    let f = runtime.block_on(fixture(&store))?;
    let conversation = runtime.block_on(exercise(&store, &f))?;
    let mut connection = diesel::SqliteConnection::establish(path.to_str().ok_or("Invalid path")?)?;
    connection.batch_execute("PRAGMA foreign_keys=ON")?;
    let old = connection
        .batch_execute(OLD_WRITER)
        .err()
        .ok_or("Old writer was accepted")?;
    assert!(old.to_string().contains("requires its chat receipt"));
    connection.batch_execute(SQLITE_ROLLBACK)?;
    runtime.block_on(verify_rollback(
        &store,
        &f,
        &conversation,
        "Injected input failure",
    ))?;
    connection.batch_execute("DROP TRIGGER reject_chat_fixture")?;
    connection.batch_execute(SQLITE_CHAT_ROLLBACK)?;
    runtime.block_on(verify_rollback(
        &store,
        &f,
        &conversation,
        "Injected chat failure",
    ))?;
    connection.batch_execute("DROP TRIGGER reject_chat_write_fixture")?;
    assert!(connection.revert_last_migration(SQLITE_MIGRATIONS).is_err());
    connection.batch_execute("DELETE FROM agent_conversations")?;
    runtime.block_on(verify_bound_conversation(&store, &f, &conversation))?;
    verify_owned_cascade!(&mut connection);
    Ok(())
}
#[test]
fn postgres_chat_receipts_and_lifetime() -> Result<(), TestError> {
    let _fixture_lock = POSTGRES_CONVERSATION_LOCK
        .lock()
        .map_err(|_| "Fixture lock poisoned")?;
    let base = std::env::var("CHAPCHOO_TEST_POSTGRES_URL")
        .map_err(|_| "Disposable PostgreSQL is required")?;
    let mut control = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&base)?;
    let suffix = BearerSecret::generate()?.digest().as_bytes()[..8]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let schema = format!("chat_{suffix}");
    control.batch_execute(&format!("CREATE SCHEMA {schema}"))?;
    let separator = if base.contains('?') { '&' } else { '?' };
    let url = format!("{base}{separator}options=-csearch_path%3D{schema}");
    let result = (|| -> Result<(), TestError> {
        let cipher = cipher()?;
        prepare_postgres(&url, &cipher)?;
        let runtime = tokio::runtime::Runtime::new()?;
        let store = runtime.block_on(Store::connect_postgres(url.clone(), cipher))?;
        let f = runtime.block_on(fixture(&store))?;
        let conversation = runtime.block_on(exercise(&store, &f))?;
        let mut connection = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&url)?;
        let old = connection
            .batch_execute(OLD_WRITER)
            .err()
            .ok_or("Old writer was accepted")?;
        assert!(old.to_string().contains("requires its chat receipt"));
        connection.batch_execute(POSTGRES_ROLLBACK)?;
        runtime.block_on(verify_rollback(
            &store,
            &f,
            &conversation,
            "Injected input failure",
        ))?;
        connection.batch_execute("DROP TRIGGER reject_chat_fixture ON conversation_inputs; DROP FUNCTION reject_chat_fixture()")?;
        connection.batch_execute(POSTGRES_CHAT_ROLLBACK)?;
        runtime.block_on(verify_rollback(
            &store,
            &f,
            &conversation,
            "Injected chat failure",
        ))?;
        connection.batch_execute("DROP TRIGGER reject_chat_write_fixture ON conversation_messages; DROP FUNCTION reject_chat_write_fixture()")?;
        assert!(
            connection
                .revert_last_migration(POSTGRES_MIGRATIONS)
                .is_err()
        );
        connection.batch_execute("DELETE FROM agent_conversations")?;
        runtime.block_on(verify_bound_conversation(&store, &f, &conversation))?;
        verify_owned_cascade!(&mut connection);
        Ok(())
    })();
    let cleanup = control.batch_execute(&format!("DROP SCHEMA {schema} CASCADE"));
    support::combine(result, cleanup.map_err(Into::into)).map(|_| ())
}
