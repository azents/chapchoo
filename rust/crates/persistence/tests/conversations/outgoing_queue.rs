use super::*;
use chapchoo_persistence::{
    ConversationDeliveryTarget, ConversationForwardingAdmission, ConversationInputAdmission,
    ConversationInputOutcome, ConversationInputScope, ConversationMessageScope,
    ConversationMessageWindow, ConversationOperationForwarding, RetryConversationInput,
};

async fn exercise_queue(store: &Store, f: &Fixture) -> Result<(), TestError> {
    let actor = f.owner.digest();
    let key = f.connector.digest();
    let first = store
        .accept_conversation_input(
            &actor,
            &AcceptConversationInput {
                agent_id: f.agent.id.clone(),
                conversation_id: None,
                target: ConversationTarget::NotStarted,
                request_id: "queue-first".into(),
                text: ConversationText::parse("  Retained original 🙂\n ".into())?,
            },
            f.now,
        )
        .await?;
    let conversation = &first.conversation_id;
    let messages = ConversationMessageScope {
        agent_id: &f.agent.id,
        conversation_id: conversation,
    };
    assert!(
        store
            .conversation_messages(
                &actor,
                &messages,
                ConversationMessageWindow::Latest,
                25,
                f.now,
            )
            .await?
            .messages
            .is_empty(),
        "Enqueue must not fabricate delivered Chat"
    );
    let snapshot = snapshot("queue-source")?;
    let native = NativeConversationId::parse("queue-native".into())?;
    store
        .bind_conversation_source(
            &f.source(&key),
            &SourceBinding {
                conversation_id: conversation.clone(),
                expected_previous: None,
                snapshot: snapshot.clone(),
                native_session: native.clone(),
            },
        )
        .await?;
    let target = ConversationTarget::Snapshot(snapshot.clone());
    let scope = ConversationInputScope {
        agent_id: &f.agent.id,
        conversation_id: conversation,
        target: &target,
    };
    // The outgoing table remains readable before native history becomes ready.
    assert_eq!(
        store
            .conversation_inputs(&actor, &scope, None, 25, f.now)
            .await?
            .len(),
        1
    );
    store
        .complete_conversation_source(&f.source(&key), conversation, &snapshot, None)
        .await?;
    let delivery = ConversationDeliveryTarget {
        snapshot,
        native_session: native,
    };
    let operation = ConversationOperationForwarding {
        agent_id: &f.agent.id,
        conversation_id: conversation,
        operation_id: &first.id,
        target: &delivery,
    };
    let route = store
        .resolve_connector_route(&f.agent.computer.id)
        .await?
        .ok_or("Missing queue route")?;
    assert!(matches!(
        store
            .begin_conversation_input_forwarding(&actor, &route, &operation, f.now,)
            .await?,
        ConversationForwardingAdmission::Granted(_)
    ));
    store
        .complete_conversation_input_forwarding(
            &f.source(&key),
            &operation,
            &ConversationInputOutcome::Failed,
        )
        .await?;
    assert_eq!(
        store
            .conversation_inputs(&actor, &scope, None, 25, f.now)
            .await?
            .len(),
        1
    );
    assert!(matches!(
        store
            .begin_conversation_input_forwarding(&actor, &route, &operation, f.now,)
            .await?,
        ConversationForwardingAdmission::Existing(_)
    ));
    let retry = RetryConversationInput {
        agent_id: f.agent.id.clone(),
        conversation_id: conversation.clone(),
        target: target.clone(),
        original_input_id: first.id.clone(),
        original_request_id: first.request_id.clone(),
        request_id: "queue-replacement".into(),
    };
    assert!(matches!(
        store
            .retry_failed_conversation_input(&f.other.digest(), &retry, f.now,)
            .await,
        Err(StoreError::NotFound)
    ));
    let rejected = RetryConversationInput {
        request_id: "queue-rejected-replacement".into(),
        ..retry.clone()
    };
    assert!(matches!(
        store
            .retry_failed_conversation_input(&actor, &rejected, f.now)
            .await,
        Err(StoreError::Database(_))
    ));
    assert!(
        store
            .conversation_input_reference(
                &actor,
                &f.agent.id,
                conversation,
                &first.request_id,
                f.now,
            )
            .await?
            .is_some(),
        "A rejected replacement must restore the old queue row"
    );
    let competitor = RetryConversationInput {
        request_id: "queue-competing-replacement".into(),
        ..retry.clone()
    };
    let (one, two) = tokio::join!(
        store.retry_failed_conversation_input(&actor, &retry, f.now),
        store.retry_failed_conversation_input(&actor, &competitor, f.now),
    );
    let (replacement, retry) = match (one, two) {
        (Ok(ConversationInputAdmission::Accepted(value)), Err(StoreError::NotFound)) => {
            (value, retry)
        }
        (Err(StoreError::NotFound), Ok(ConversationInputAdmission::Accepted(value))) => {
            (value, competitor)
        }
        _ => return Err("Competing retries must create exactly one replacement".into()),
    };
    assert_ne!(replacement.id, first.id);
    assert_eq!(replacement.text, first.text);
    assert!(
        store
            .conversation_input_reference(
                &actor,
                &f.agent.id,
                conversation,
                &first.request_id,
                f.now,
            )
            .await?
            .is_none()
    );
    assert!(matches!(store.retry_failed_conversation_input(
        &actor, &retry, f.now,
    ).await?, ConversationInputAdmission::Existing(value) if value.id == replacement.id));
    let another = RetryConversationInput {
        request_id: "queue-another-replacement".into(),
        ..retry
    };
    assert!(matches!(
        store
            .retry_failed_conversation_input(&actor, &another, f.now,)
            .await,
        Err(StoreError::NotFound)
    ));
    assert_eq!(
        store
            .conversation_inputs(&actor, &scope, None, 25, f.now)
            .await?
            .len(),
        1
    );
    let operation = ConversationOperationForwarding {
        operation_id: &replacement.id,
        ..operation
    };
    store
        .begin_conversation_input_forwarding(&actor, &route, &operation, f.now)
        .await?;
    let outcome = ConversationInputOutcome::Inserted {
        native_message_id: "queue-actual-message".into(),
    };
    for _ in 0..2 {
        assert_eq!(
            store
                .complete_conversation_input_forwarding(&f.source(&key), &operation, &outcome)
                .await?,
            chapchoo_persistence::ConversationInputReportAcknowledgement::Recorded
        );
    }
    assert!(
        store
            .conversation_inputs(&actor, &scope, None, 25, f.now)
            .await?
            .is_empty()
    );
    let received = store
        .conversation_messages(
            &actor,
            &messages,
            ConversationMessageWindow::Latest,
            25,
            f.now,
        )
        .await?;
    assert_eq!(received.messages.len(), 1);
    assert_eq!(
        received.messages[0].receipt.source_input_id.as_deref(),
        Some(replacement.id.as_str())
    );
    assert_eq!(
        received.messages[0].receipt.original_accepted_at,
        Some(replacement.accepted_at)
    );
    let last = store
        .accept_conversation_input(
            &actor,
            &AcceptConversationInput {
                agent_id: f.agent.id.clone(),
                conversation_id: Some(conversation.clone()),
                target: target.clone(),
                request_id: "queue-delete".into(),
                text: ConversationText::parse("Delete this failed queue item".into())?,
            },
            f.now,
        )
        .await?;
    let operation = ConversationOperationForwarding {
        operation_id: &last.id,
        ..operation
    };
    store
        .begin_conversation_input_forwarding(&actor, &route, &operation, f.now)
        .await?;
    store
        .complete_conversation_input_forwarding(
            &f.source(&key),
            &operation,
            &ConversationInputOutcome::Failed,
        )
        .await?;
    store
        .delete_failed_conversation_input(&actor, &scope, &last.request_id, f.now)
        .await?;
    store
        .delete_failed_conversation_input(&actor, &scope, &last.request_id, f.now)
        .await?;
    assert_eq!(
        store
            .complete_conversation_input_forwarding(
                &f.source(&key),
                &operation,
                &ConversationInputOutcome::Steered,
            )
            .await?,
        chapchoo_persistence::ConversationInputReportAcknowledgement::Closed,
        "Deletion closure cannot open a successful user-receipt publication fence"
    );
    assert!(
        store
            .conversation_input_reference(
                &actor,
                &f.agent.id,
                conversation,
                &last.request_id,
                f.now,
            )
            .await?
            .is_none()
    );
    assert_eq!(
        store
            .conversation_messages(
                &actor,
                &messages,
                ConversationMessageWindow::Latest,
                25,
                f.now,
            )
            .await?
            .messages
            .len(),
        1
    );
    exercise_consumer(store, f, conversation, &target).await
}

async fn exercise_consumer(
    store: &Store,
    f: &Fixture,
    conversation: &str,
    target: &ConversationTarget,
) -> Result<(), TestError> {
    use chapchoo_persistence::ConversationInputQueueState as State;
    let actor = f.owner.digest();
    let draft = AcceptConversationInput {
        agent_id: f.agent.id.clone(),
        conversation_id: Some(conversation.into()),
        target: target.clone(),
        request_id: "orphan-consumer".into(),
        text: ConversationText::parse("Retained after owner loss".into())?,
    };
    let short = DirectoryPolicy {
        component_validity_ms: 1,
        ..DirectoryPolicy::default()
    };
    let admitted = store
        .admit_conversation_input_for_consumer(&actor, &draft, f.now, short)
        .await?;
    let ConversationInputAdmission::Accepted(first) = admitted else {
        return Err("Expected fresh consumer".into());
    };
    // Real elapsed time is the database lease's contract at this boundary.
    tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    assert!(matches!(
        store
            .admit_conversation_input(&actor, &draft, f.now)
            .await?,
        ConversationInputAdmission::Existing(_)
    ));
    assert_eq!(
        store
            .renew_conversation_input_consumer(conversation, &first.id, DirectoryPolicy::default())
            .await?,
        State::Error,
        "Recovery reads must not renew an expired owner"
    );
    assert_eq!(
        store
            .renew_conversation_input_consumer(conversation, &first.id, DirectoryPolicy::default())
            .await?,
        State::Error,
        "Heartbeat cannot resurrect a failed attempt"
    );
    let retry = RetryConversationInput {
        agent_id: f.agent.id.clone(),
        conversation_id: conversation.into(),
        target: target.clone(),
        original_input_id: first.id.clone(),
        original_request_id: first.request_id.clone(),
        request_id: "orphan-replacement".into(),
    };
    let ConversationInputAdmission::Accepted(replacement) = store
        .retry_failed_conversation_input(&actor, &retry, f.now)
        .await?
    else {
        return Err("Expected fresh retry consumer".into());
    };
    assert_eq!(replacement.text, first.text);
    store
        .fail_conversation_input(conversation, &first.id)
        .await?;
    assert_eq!(
        store
            .conversation_input_queue_state(conversation, &replacement.id)
            .await?,
        State::Pending,
        "Late old finalization cannot close the replacement"
    );
    assert_eq!(
        store
            .renew_conversation_input_consumer(
                conversation,
                &replacement.id,
                DirectoryPolicy::default()
            )
            .await?,
        State::Pending
    );
    store
        .fail_conversation_input(conversation, &replacement.id)
        .await?;
    assert_eq!(
        store
            .conversation_input_queue_state(conversation, &replacement.id)
            .await?,
        State::Error
    );
    let orphan = AcceptConversationInput {
        request_id: "restart-orphan".into(),
        ..draft
    };
    let ConversationInputAdmission::Accepted(orphan) = store
        .admit_conversation_input_for_consumer(&actor, &orphan, f.now, short)
        .await?
    else {
        return Err("Expected fresh restart orphan".into());
    };
    tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    store.expire_conversation_input_consumers().await?;
    assert_eq!(
        store
            .conversation_input_queue_state(conversation, &orphan.id)
            .await?,
        State::Error,
        "Independent reaping closes work whose process disappeared"
    );
    Ok(())
}

#[test]
fn sqlite_outgoing_queue_lifecycle() -> Result<(), TestError> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("outgoing.sqlite");
    let cipher = cipher()?;
    prepare_sqlite(&path, &cipher)?;
    let mut connection = diesel::SqliteConnection::establish(path.to_str().ok_or("Invalid path")?)?;
    connection.batch_execute("CREATE TRIGGER reject_queue_fixture BEFORE INSERT ON conversation_inputs WHEN NEW.request_id='queue-rejected-replacement' BEGIN SELECT RAISE(ABORT,'Injected replacement failure'); END")?;
    let runtime = tokio::runtime::Runtime::new()?;
    let store = runtime.block_on(Store::connect_sqlite(path, cipher))?;
    let f = runtime.block_on(fixture(&store))?;
    runtime.block_on(exercise_queue(&store, &f))
}

#[test]
fn postgres_outgoing_queue_lifecycle() -> Result<(), TestError> {
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
    let schema = format!("outgoing_{suffix}");
    control.batch_execute(&format!("CREATE SCHEMA {schema}"))?;
    let separator = if base.contains('?') { '&' } else { '?' };
    let url = format!("{base}{separator}options=-csearch_path%3D{schema}");
    let result = (|| -> Result<(), TestError> {
        let cipher = cipher()?;
        prepare_postgres(&url, &cipher)?;
        let mut connection = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&url)?;
        connection.batch_execute("CREATE FUNCTION reject_queue_fixture() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF NEW.request_id='queue-rejected-replacement' THEN RAISE EXCEPTION 'Injected replacement failure'; END IF; RETURN NEW; END; $$; CREATE TRIGGER reject_queue_fixture BEFORE INSERT ON conversation_inputs FOR EACH ROW EXECUTE FUNCTION reject_queue_fixture()")?;
        let runtime = tokio::runtime::Runtime::new()?;
        let store = runtime.block_on(Store::connect_postgres(url, cipher))?;
        let f = runtime.block_on(fixture(&store))?;
        runtime.block_on(exercise_queue(&store, &f))
    })();
    control.batch_execute(&format!("DROP SCHEMA {schema} CASCADE"))?;
    result
}
