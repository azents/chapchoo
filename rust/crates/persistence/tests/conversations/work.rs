use super::*;
use chapchoo_domain::{SendAction, WorkTask, WorkTaskStatus};
use chapchoo_persistence::{
    ConversationDeliveryTarget, ConversationMessageScope, ConversationMessageWindow,
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
    assert!(matches!(
        store
            .send_conversation_work(
                &source,
                &scope,
                &SendAction::Continue {
                    title: None,
                    tasks: vec![WorkTask {
                        text: text(&escaped)?,
                        status: WorkTaskStatus::Pending,
                    }],
                    message: Some(text("Rejected amendment must not publish")?),
                },
            )
            .await,
        Err(StoreError::InvalidInput)
    ));
    let unchanged = store
        .conversation_work_summary(&owner, &scope, f.now)
        .await?
        .ok_or("Rejected amendment removed Work")?;
    assert_eq!(unchanged.title, first.title);
    assert_eq!(unchanged.tasks, first.tasks);
    assert!(unchanged.waiting_for_input);
    let update = SendAction::Continue {
        title: Some(text("Inspect revised records")?),
        tasks: vec![
            WorkTask {
                text: text("Report findings")?,
                status: WorkTaskStatus::Pending,
            },
            WorkTask {
                text: text("Review evidence")?,
                status: WorkTaskStatus::Completed,
            },
        ],
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
    assert_eq!(waiting.tasks.len(), 2);
    assert_eq!(waiting.id, first.id);
    let current: serde_json::Value = serde_json::from_slice(
        &store
            .conversation_goal_context(&source, scope.conversation_id, &target)
            .await?,
    )?;
    assert_eq!(
        current,
        serde_json::json!([{
            "goal_key": first.id,
            "title": "Inspect revised records",
            "tasks": [
                {"text": "Report findings", "status": "pending"},
                {"text": "Review evidence", "status": "completed"},
            ],
            "disposition": "waiting_for_input",
        }])
    );
    store
        .send_conversation_work(
            &source,
            &scope,
            &SendAction::Continue {
                title: None,
                tasks: Vec::new(),
                message: None,
            },
        )
        .await?;
    let cleared: serde_json::Value = serde_json::from_slice(
        &store
            .conversation_goal_context(&source, scope.conversation_id, &target)
            .await?,
    )?;
    assert_eq!(
        cleared,
        serde_json::json!([{
            "goal_key": first.id,
            "title": "Inspect revised records",
            "tasks": [],
            "disposition": "waiting_for_input",
        }])
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
    store
        .send_conversation_work(&source, &scope, &question)
        .await?;
    let before = store
        .conversation_messages(&owner, &scope, ConversationMessageWindow::Latest, 10, f.now)
        .await?;
    assert!(
        store
            .send_conversation_work(&source, &scope, &SendAction::Ignore)
            .await?
            .is_none()
    );
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
    assert_eq!(
        store
            .conversation_messages(&owner, &scope, ConversationMessageWindow::Latest, 10, f.now)
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
