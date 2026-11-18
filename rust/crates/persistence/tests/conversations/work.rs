use super::*;
use chapchoo_domain::{SendAction, WorkTask, WorkTaskStatus};
use chapchoo_persistence::{
    ConversationDeliveryTarget, ConversationGoalChange as Goal, ConversationMessageScope,
    ConversationMessageWindow,
};

async fn exercise(store: &Store, f: &Fixture) -> Result<(), TestError> {
    let owner = f.owner.digest();
    let credential = f.connector.digest();
    let source = f.source(&credential);
    let mut draft = AcceptConversationInput {
        agent_id: f.agent.id.clone(),
        conversation_id: None,
        request_id: "work-original-input".into(),
        target: ConversationTarget::NotStarted,
        text: ConversationText::parse("Start work".into())?,
    };
    let original = store
        .accept_conversation_input(&owner, &draft, f.now)
        .await?;
    let scope = ConversationMessageScope {
        agent_id: &f.agent.id,
        conversation_id: &original.conversation_id,
    };
    let target = ConversationDeliveryTarget {
        snapshot: SnapshotId::parse("work-source".into())?,
        native_session: NativeConversationId::parse("work-native".into())?,
    };
    store
        .bind_conversation_source(
            &source,
            &SourceBinding {
                conversation_id: original.conversation_id.clone(),
                expected_previous: None,
                snapshot: target.snapshot.clone(),
                native_session: target.native_session.clone(),
            },
        )
        .await?;
    store
        .complete_conversation_source(&source, scope.conversation_id, &target.snapshot, None)
        .await?;
    let text = |value: &str| ConversationText::parse(value.into());
    let tasks = vec![WorkTask {
        text: text("Review evidence")?,
        status: WorkTaskStatus::Pending,
    }];
    assert!(matches!(
        store
            .send_conversation_work(
                &source,
                &scope,
                &SendAction::Continue {
                    title: None,
                    tasks: tasks.clone(),
                    message: Some(text("Must not be published")?),
                }
            )
            .await,
        Err(StoreError::InvalidInput)
    ));
    assert!(
        store
            .conversation_work_summary(&owner, &scope, f.now)
            .await?
            .is_none()
    );
    assert_eq!(
        store
            .conversation_messages(&owner, &scope, ConversationMessageWindow::Latest, 10, f.now)
            .await?
            .through,
        0
    );
    // Escaping expands the projection after the Goal insert. The rejected
    // transaction must not retain the Goal, Work, or optional publication.
    let escaped = "\t".repeat(5_000) + "X";
    assert!(matches!(
        store
            .send_conversation_work(
                &source,
                &scope,
                &SendAction::Continue {
                    title: Some(text(&escaped)?),
                    tasks: tasks.clone(),
                    message: Some(text("Must not survive a rollback")?),
                },
            )
            .await,
        Err(StoreError::InvalidInput)
    ));
    assert!(
        store
            .conversation_work_summary(&owner, &scope, f.now)
            .await?
            .is_none()
    );

    let question = SendAction::RequestInput {
        message: text("Which source?")?,
    };
    let simple = store
        .send_conversation_work(&source, &scope, &question)
        .await?
        .ok_or("Missing initial question")?;
    assert_eq!(simple.receipt_order, 1);
    assert!(
        store
            .conversation_work_summary(&owner, &scope, f.now)
            .await?
            .is_none()
    );
    assert_eq!(
        store
            .conversation_goal_context(&source, scope.conversation_id, &target)
            .await?,
        b"[]"
    );
    assert!(
        store
            .send_conversation_work(&source, &scope, &SendAction::Ignore)
            .await?
            .is_none()
    );

    let start = SendAction::Continue {
        title: Some(text("Inspect records")?),
        tasks: tasks.clone(),
        message: None,
    };
    assert!(
        store
            .send_conversation_work(&source, &scope, &start)
            .await?
            .is_none()
    );
    let first = store
        .conversation_work_summary(&owner, &scope, f.now)
        .await?
        .ok_or("Missing work")?;
    assert_eq!(first.tasks, tasks);
    assert!(!first.waiting_for_input);
    assert!(
        store
            .conversation_work_summary(&f.other.digest(), &scope, f.now)
            .await
            .is_err()
    );
    let question_receipt = store
        .send_conversation_work(&source, &scope, &question)
        .await?
        .ok_or("Missing work question")?;
    assert_eq!(question_receipt.receipt_order, 2);
    assert_eq!(
        store
            .accept_conversation_input(&owner, &draft, f.now)
            .await?
            .id,
        original.id
    );
    let update = SendAction::Continue {
        title: Some(text("Inspect revised records")?),
        tasks: Vec::new(),
        message: None,
    };
    store
        .send_conversation_work(&source, &scope, &update)
        .await?;
    let waiting = store
        .conversation_work_summary(&owner, &scope, f.now)
        .await?
        .ok_or("Missing wait")?;
    assert!(waiting.waiting_for_input);
    assert!(waiting.tasks.is_empty());
    assert_eq!(waiting.id, first.id);
    assert!(
        String::from_utf8(
            store
                .conversation_goal_context(&source, scope.conversation_id, &target)
                .await?
        )?
        .contains("waiting_for_input")
    );
    draft.conversation_id = Some(original.conversation_id.clone());
    draft.request_id = "work-fresh-input".into();
    draft.target = ConversationTarget::Snapshot(target.snapshot.clone());
    store
        .accept_conversation_input(&owner, &draft, f.now)
        .await?;
    assert!(
        !store
            .conversation_work_summary(&owner, &scope, f.now)
            .await?
            .ok_or("Missing resumed work")?
            .waiting_for_input
    );
    // Reverse the serialization order: the input committed before this question
    // does not count as a reply to it.
    let later_question = store
        .send_conversation_work(&source, &scope, &question)
        .await?
        .ok_or("Missing later question")?;
    assert_eq!(later_question.receipt_order, 3);
    assert!(
        store
            .conversation_work_summary(&owner, &scope, f.now)
            .await?
            .ok_or("Missing later wait")?
            .waiting_for_input
    );

    let finish = SendAction::Finish {
        message: text("Result")?,
    };
    let result = store
        .send_conversation_work(&source, &scope, &finish)
        .await?
        .ok_or("Missing result")?;
    assert_eq!(result.receipt_order, 4);
    assert!(
        store
            .conversation_work_summary(&owner, &scope, f.now)
            .await?
            .is_none()
    );
    assert_eq!(
        store
            .conversation_goal_context(&source, scope.conversation_id, &target)
            .await?,
        b"[]"
    );
    store
        .send_conversation_work(&source, &scope, &start)
        .await?;
    let restarted = store
        .conversation_work_summary(&owner, &scope, f.now)
        .await?
        .ok_or("Missing successor")?;
    assert_ne!(restarted.id, first.id);
    verify_explicit_failure(store, f, &scope, &target, &restarted.id, &start, &question).await
}

async fn verify_explicit_failure(
    store: &Store,
    f: &Fixture,
    scope: &ConversationMessageScope<'_>,
    target: &ConversationDeliveryTarget,
    work: &str,
    start: &SendAction,
    question: &SendAction,
) -> Result<(), TestError> {
    // This models a trusted Store caller declaring failure, not an execution
    // fault, a `send` call, or a product entrypoint for harness settlement.
    let owner = f.owner.digest();
    let credential = f.connector.digest();
    let source = f.source(&credential);
    let id = scope.conversation_id;
    let independent = "independent-fixture-goal";
    store
        .change_conversation_goal(
            &source,
            id,
            target,
            independent,
            Goal::Add {
                objective: "Compare the independent fixture",
            },
        )
        .await?;
    store
        .send_conversation_work(&source, scope, question)
        .await?;
    let before = store
        .conversation_messages(&owner, scope, ConversationMessageWindow::Latest, 10, f.now)
        .await?;
    let replacement = ConversationDeliveryTarget {
        snapshot: snapshot("work-replacement")?,
        native_session: target.native_session.clone(),
    };
    store
        .bind_conversation_source(
            &source,
            &SourceBinding {
                conversation_id: id.into(),
                expected_previous: Some(target.snapshot.clone()),
                snapshot: replacement.snapshot.clone(),
                native_session: replacement.native_session.clone(),
            },
        )
        .await?;
    store
        .complete_conversation_source(&source, id, &replacement.snapshot, None)
        .await?;
    assert!(matches!(
        store
            .change_conversation_goal(&source, id, target, work, Goal::Fail)
            .await,
        Err(StoreError::Conflict)
    ));
    let current_work = store
        .conversation_work_summary(&owner, scope, f.now)
        .await?
        .ok_or("Stale failure removed the current work")?;
    assert_eq!(current_work.id, work);
    assert!(current_work.waiting_for_input);
    store
        .change_conversation_goal(&source, id, &replacement, work, Goal::Fail)
        .await?;
    assert!(
        store
            .conversation_work_summary(&owner, scope, f.now)
            .await?
            .is_none()
    );
    let remaining: serde_json::Value = serde_json::from_slice(
        &store
            .conversation_goal_context(&source, id, &replacement)
            .await?,
    )?;
    assert_eq!(
        remaining,
        serde_json::json!([{
            "goal_key": independent,
            "objective": "Compare the independent fixture",
            "disposition": "active",
        }])
    );
    assert!(matches!(
        store
            .change_conversation_goal(&source, id, &replacement, work, Goal::Complete)
            .await,
        Err(StoreError::InvalidTransition)
    ));
    store.send_conversation_work(&source, scope, start).await?;
    let next = store
        .conversation_work_summary(&owner, scope, f.now)
        .await?
        .ok_or("Missing work after failure")?;
    assert_ne!(next.id, work);
    store
        .change_conversation_goal(&source, id, &replacement, work, Goal::Fail)
        .await?;
    let after_retry = store
        .conversation_work_summary(&owner, scope, f.now)
        .await?
        .ok_or("Failure retry removed the successor")?;
    assert_eq!(after_retry.id, next.id);
    assert_eq!(
        store
            .conversation_messages(&owner, scope, ConversationMessageWindow::Latest, 10, f.now)
            .await?
            .through,
        before.through
    );
    Ok(())
}

#[test]
fn sqlite_conversation_work_transition() -> Result<(), TestError> {
    let root = tempfile::tempdir()?;
    let path = root.path().join("work.sqlite");
    let key = cipher()?;
    prepare_sqlite(&path, &key)?;
    let rt = tokio::runtime::Runtime::new()?;
    let store = rt.block_on(Store::connect_sqlite(path.clone(), key))?;
    let fixture = rt.block_on(fixture(&store))?;
    rt.block_on(exercise(&store, &fixture))?;
    let mut connection = diesel::SqliteConnection::establish(path.to_str().ok_or("Invalid path")?)?;
    assert!(connection.revert_last_migration(SQLITE_MIGRATIONS).is_err());
    Ok(())
}

#[test]
fn postgres_conversation_work_transition() -> Result<(), TestError> {
    let _guard = POSTGRES_CONVERSATION_LOCK
        .lock()
        .map_err(|_| "PostgreSQL fixture lock poisoned")?;
    let base = std::env::var("CHAPCHOO_TEST_POSTGRES_URL")?;
    let mut control = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&base)?;
    let suffix = BearerSecret::generate()?.digest().as_bytes()[..8]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let schema = format!("work_{suffix}");
    control.batch_execute(&format!("CREATE SCHEMA {schema}"))?;
    let separator = if base.contains('?') { '&' } else { '?' };
    let url = format!("{base}{separator}options=-csearch_path%3D{schema}");
    let result = (|| -> Result<(), TestError> {
        let key = cipher()?;
        prepare_postgres(&url, &key)?;
        let rt = tokio::runtime::Runtime::new()?;
        let store = rt.block_on(Store::connect_postgres(url.clone(), key))?;
        let fixture = rt.block_on(fixture(&store))?;
        rt.block_on(exercise(&store, &fixture))?;
        let mut connection = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&url)?;
        assert!(
            connection
                .revert_last_migration(POSTGRES_MIGRATIONS)
                .is_err()
        );
        Ok(())
    })();
    control.batch_execute(&format!("DROP SCHEMA {schema} CASCADE"))?;
    result
}
