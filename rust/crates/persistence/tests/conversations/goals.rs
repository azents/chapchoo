use super::*;
use chapchoo_domain::{InstructionSource, InstructionSourceKind, ProjectInstructions, SendAction};
use chapchoo_persistence::{
    ConnectionStartupAdmission, ConversationContinuationDecision as Decision,
    ConversationDeliveryTarget, ConversationForwardingAdmission, ConversationGoalChange as Goal,
    ConversationInputPreparation, ConversationMessageScope, ConversationNormalEnding,
    ConversationOperationForwarding, ConversationPreparationOutcome,
};

async fn exercise(
    store: &Store,
    f: &Fixture,
    mut fixture_sql: impl FnMut(&str) -> Result<(), TestError>,
) -> Result<(), TestError> {
    let owner = f.owner.digest();
    let credential = f.connector.digest();
    let source = f.source(&credential);
    let mut draft = AcceptConversationInput {
        agent_id: f.agent.id.clone(),
        conversation_id: None,
        request_id: "goal-fixture-initial-input".into(),
        target: ConversationTarget::NotStarted,
        text: ConversationText::parse("Fixture input".into())?,
    };
    let initial = store
        .accept_conversation_input(&owner, &draft, f.now)
        .await?;
    let id = initial.conversation_id.as_str();
    let snapshot = snapshot("goal-fixture-source")?;
    let native = NativeConversationId::parse("goal-fixture-native".into())?;
    let target = ConversationDeliveryTarget {
        snapshot: snapshot.clone(),
        native_session: native.clone(),
    };
    store
        .bind_conversation_source(
            &source,
            &SourceBinding {
                conversation_id: id.into(),
                expected_previous: None,
                snapshot: snapshot.clone(),
                native_session: native.clone(),
            },
        )
        .await?;
    store
        .complete_conversation_source(&source, id, &snapshot, None)
        .await?;
    let route = store
        .resolve_connector_route(&f.agent.computer.id)
        .await?
        .ok_or("Missing Connector route")?;
    let setup = ConversationInputPreparation {
        agent_id: &f.agent.id,
        conversation_id: id,
        operation_id: &initial.id,
    };
    store
        .begin_conversation_input_preparation(
            &owner,
            &route,
            &setup,
            f.now,
            &f.claim(store, id, &initial.id).await?,
        )
        .await?;
    let applied = store
        .begin_execution_authentication(
            &source,
            &setup,
            None,
            Some(&chapchoo_domain::LaunchCapture {
                snapshot: snapshot.clone(),
                through: None,
            }),
        )
        .await?;
    instructions_fixture::apply(
        store,
        f,
        &owner,
        &route,
        &source,
        &ConversationOperationForwarding {
            agent_id: &f.agent.id,
            conversation_id: id,
            operation_id: &initial.id,
            target: &target,
        },
        f.now,
    )
    .await?;
    assert_eq!(
        store
            .conversation_goal_context(&source, id, &target)
            .await?,
        b"[]"
    );
    store
        .change_conversation_goal(
            &source,
            id,
            &target,
            "fixture-a",
            Goal::Add {
                objective: "Objective A",
            },
        )
        .await?;
    store
        .change_conversation_goal(
            &source,
            id,
            &target,
            "fixture-b",
            Goal::Add {
                objective: "Objective B",
            },
        )
        .await?;
    let too_large = "\t".repeat(5_000) + "X";
    assert!(matches!(
        store
            .change_conversation_goal(
                &source,
                id,
                &target,
                "fixture-b",
                Goal::Amend {
                    objective: &too_large
                }
            )
            .await,
        Err(StoreError::InvalidInput)
    ));
    let projection = store
        .conversation_goal_context(&source, id, &target)
        .await?;
    assert!(String::from_utf8(projection)?.contains("Objective B"));
    let mut endings = Vec::new();
    for n in 1..=14 {
        let ending = ConversationNormalEnding {
            target: target.clone(),
            position: position(n, &format!("fixture-ending-{n}"))?,
            work_id: format!("fixture-work-{n}"),
            launch_operation_id: applied.operation_id.clone(),
        };
        let raw = serde_json::to_vec(&serde_json::json!({
            "method":"session/update",
            "params":{"sessionId":native.as_str(),"update":{"sessionUpdate":"state_update","state":"idle","stopReason":if n==8 {"cancelled"} else {"end_turn"}}}
        }))?;
        store
            .ingest_conversation_pair(
                &source,
                id,
                &CapturedEventPair {
                    snapshot: snapshot.clone(),
                    position: ending.position.clone(),
                    raw,
                    normalized: b"fixture".to_vec(),
                },
            )
            .await?;
        endings.push(ending);
    }
    // A genuinely pending user input wins the first decision.
    assert_eq!(
        store
            .admit_conversation_continuation(&source, id, &endings[0])
            .await?,
        Decision::NoAction
    );
    store.fail_conversation_input(id, &initial.id).await?;
    let (first, duplicate) = tokio::join!(
        store.admit_conversation_continuation(&source, id, &endings[1]),
        store.admit_conversation_continuation(&source, id, &endings[1]),
    );
    let first = first?;
    let Decision::Admitted { operation_id } = &first else {
        return Err("Qualified ending did not admit a continuation".into());
    };
    assert_eq!(first, duplicate?);
    assert!(matches!(
        store
            .begin_conversation_continuation(&source, id, operation_id, &endings[0])
            .await,
        Err(StoreError::NotFound)
    ));
    store
        .change_conversation_goal(
            &source,
            id,
            &target,
            "fixture-b",
            Goal::Amend {
                objective: "Latest objective",
            },
        )
        .await?;
    let (grant, duplicate_grant) = tokio::join!(
        store.begin_conversation_continuation(&source, id, operation_id, &endings[1]),
        store.begin_conversation_continuation(&source, id, operation_id, &endings[1]),
    );
    let projections = [grant?, duplicate_grant?];
    assert_eq!(projections.iter().flatten().count(), 1);
    let projection = String::from_utf8(
        projections
            .into_iter()
            .flatten()
            .next()
            .ok_or("No continuation forwarding projection")?,
    )?;
    assert!(projection.contains("Objective A") && projection.contains("Latest objective"));
    for _ in 0..2 {
        store
            .close_conversation_continuation(&source, id, operation_id, &endings[1])
            .await?;
    }
    assert!(
        store
            .begin_conversation_continuation(&source, id, operation_id, &endings[1])
            .await?
            .is_none(),
    );
    for ending in &endings[2..4] {
        let Decision::Admitted { operation_id } = store
            .admit_conversation_continuation(&source, id, ending)
            .await?
        else {
            return Err("Continuation limit reached before three admissions".into());
        };
        store
            .close_conversation_continuation(&source, id, &operation_id, ending)
            .await?;
    }
    assert_eq!(
        store
            .admit_conversation_continuation(&source, id, &endings[4])
            .await?,
        Decision::NoAction
    );
    store
        .change_conversation_goal(
            &source,
            id,
            &target,
            "fixture-a",
            Goal::Add {
                objective: "Objective A",
            },
        )
        .await?;
    store
        .change_conversation_goal(&source, id, &target, "fixture-a", Goal::Complete)
        .await?;
    assert_eq!(
        store
            .admit_conversation_continuation(&source, id, &endings[5])
            .await?,
        Decision::NoAction
    );
    assert_eq!(
        store
            .admit_conversation_continuation(&source, id, &endings[1])
            .await?,
        first
    );
    draft.request_id = "goal-fixture-explicit-input".into();
    draft.conversation_id = Some(id.into());
    draft.target = ConversationTarget::Snapshot(snapshot.clone());
    let explicit = store
        .accept_conversation_input(&owner, &draft, f.now)
        .await?;
    store.fail_conversation_input(id, &explicit.id).await?;
    assert_eq!(
        store
            .admit_conversation_continuation(&source, id, &endings[5])
            .await?,
        Decision::NoAction
    );
    let Decision::Admitted { operation_id } = store
        .admit_conversation_continuation(&source, id, &endings[6])
        .await?
    else {
        return Err("Fresh user input did not reset the limit".into());
    };
    ordinary_follows_continuation(store, f, id, &operation_id, &endings[6], true).await?;
    assert!(matches!(
        store
            .admit_conversation_continuation(&source, id, &endings[7])
            .await,
        Err(StoreError::InvalidTransition)
    ));
    let current = String::from_utf8(
        store
            .conversation_goal_context(&source, id, &target)
            .await?,
    )?;
    assert!(!current.contains("Objective A") && current.contains("Latest objective"));
    let scope = ConversationMessageScope {
        agent_id: &f.agent.id,
        conversation_id: id,
    };
    store
        .send_conversation_work(
            &source,
            &scope,
            &SendAction::Continue {
                title: Some(ConversationText::parse("Retained draft Work".into())?),
                tasks: Vec::new(),
                message: None,
            },
        )
        .await?;
    let Decision::Admitted { operation_id } = store
        .admit_conversation_continuation(&source, id, &endings[8])
        .await?
    else {
        return Err("Expected an earlier admitted slot".into());
    };
    // Only this isolated fixture has a Work row. Simulate a task list
    // retained from before the full projection was checked at Work writes.
    let oversized_tasks: Vec<_> = (0..16)
        .map(|_| serde_json::json!({"text": "x".repeat(600), "status": "pending"}))
        .collect();
    fixture_sql(&format!(
        "UPDATE conversation_work_items SET tasks_json='{}'",
        serde_json::to_string(&oversized_tasks)?
    ))?;
    assert!(matches!(
        store.conversation_goal_context(&source, id, &target).await,
        Err(StoreError::Integrity)
    ));
    assert_eq!(
        store
            .admit_conversation_continuation(&source, id, &endings[9])
            .await?,
        Decision::NoAction
    );
    ordinary_follows_continuation(store, f, id, &operation_id, &endings[8], false).await?;
    let retained = store
        .conversation_work_summary(&owner, &scope, f.now)
        .await?
        .ok_or("Oversized context discarded draft Work")?;
    assert_eq!(retained.tasks.len(), oversized_tasks.len());
    store
        .send_conversation_work(
            &source,
            &scope,
            &SendAction::Continue {
                title: None,
                tasks: vec![],
                message: None,
            },
        )
        .await?;
    settlement_before_grant(store, f, &scope, &endings[10..13]).await?;
    // Remove the synthetic Work overlay before the migration guards run.
    fixture_sql("DELETE FROM conversation_work_items")?;
    store
        .change_conversation_goal(
            &source,
            id,
            &target,
            "fixture-c",
            Goal::Add {
                objective: "Objective C",
            },
        )
        .await?;
    replaced_launch_does_not_inherit_slot(store, f, id, &endings[13]).await?;
    Ok(())
}

async fn settlement_before_grant(
    store: &Store,
    f: &Fixture,
    scope: &ConversationMessageScope<'_>,
    endings: &[ConversationNormalEnding],
) -> Result<(), TestError> {
    let [first, last, next] = endings else {
        return Err("Expected three distinct fixture endings".into());
    };
    let credential = f.connector.digest();
    let source = f.source(&credential);
    let conversation = scope.conversation_id;
    let Decision::Admitted { operation_id } = store
        .admit_conversation_continuation(&source, conversation, first)
        .await?
    else {
        return Err("Expected admission before Work completion".into());
    };
    store
        .send_conversation_work(&source, scope, &SendAction::Ignore)
        .await?;
    let projection = store
        .begin_conversation_continuation(&source, conversation, &operation_id, first)
        .await?
        .ok_or("Work completion displaced the other eligible goal")?;
    let projection: serde_json::Value = serde_json::from_slice(&projection)?;
    assert_eq!(
        projection,
        serde_json::json!([{
            "goal_key": "fixture-b", "objective": "Latest objective", "disposition": "active"
        }])
    );
    store
        .close_conversation_continuation(&source, conversation, &operation_id, first)
        .await?;

    let decision = store
        .admit_conversation_continuation(&source, conversation, last)
        .await?;
    let Decision::Admitted { operation_id } = &decision else {
        return Err("Expected admission before the last Goal completed".into());
    };
    store
        .change_conversation_goal(
            &source,
            conversation,
            &last.target,
            "fixture-b",
            Goal::Complete,
        )
        .await?;
    assert_eq!(
        store
            .admit_conversation_continuation(&source, conversation, next)
            .await?,
        Decision::NoAction
    );
    ordinary_follows_continuation(store, f, conversation, operation_id, last, false).await?;
    assert_eq!(
        store
            .admit_conversation_continuation(&source, conversation, last)
            .await?,
        decision
    );
    assert!(
        store
            .begin_conversation_continuation(&source, conversation, operation_id, last)
            .await?
            .is_none()
    );
    Ok(())
}

async fn ordinary_follows_continuation(
    store: &Store,
    f: &Fixture,
    conversation: &str,
    operation: &str,
    ending: &ConversationNormalEnding,
    expect_grant: bool,
) -> Result<(), TestError> {
    let owner = f.owner.digest();
    let credential = f.connector.digest();
    let source = f.source(&credential);
    let route = store
        .resolve_connector_route(&f.agent.computer.id)
        .await?
        .ok_or("Missing Connector route")?;
    let later = store
        .accept_conversation_input(
            &owner,
            &AcceptConversationInput {
                agent_id: f.agent.id.clone(),
                conversation_id: Some(conversation.into()),
                request_id: format!("later-user-input-{}", ending.position.ordinal()),
                target: ConversationTarget::Snapshot(ending.target.snapshot.clone()),
                text: ConversationText::parse("Later ordinary input".into())?,
            },
            f.now,
        )
        .await?;
    let draft = ConversationOperationForwarding {
        agent_id: &f.agent.id,
        conversation_id: conversation,
        operation_id: &later.id,
        target: &ending.target,
    };
    instructions_fixture::apply(store, f, &owner, &route, &source, &draft, f.now).await?;
    let claim = f.claim(store, conversation, &later.id).await?;
    assert!(matches!(
        store
            .begin_conversation_input_forwarding(&owner, &route, &draft, f.now, &claim)
            .await,
        Err(StoreError::InvalidTransition)
    ));
    assert_eq!(
        store
            .begin_conversation_continuation(&source, conversation, operation, ending)
            .await?
            .is_some(),
        expect_grant,
    );
    if expect_grant {
        assert!(matches!(
            store
                .begin_conversation_input_forwarding(&owner, &route, &draft, f.now, &claim)
                .await,
            Err(StoreError::InvalidTransition)
        ));
        store
            .close_conversation_continuation(&source, conversation, operation, ending)
            .await?;
    }
    assert!(matches!(
        store
            .begin_conversation_input_forwarding(&owner, &route, &draft, f.now, &claim)
            .await?,
        ConversationForwardingAdmission::Granted(_)
    ));
    store
        .complete_conversation_input_forwarding(
            &source,
            &draft,
            &chapchoo_persistence::ConversationInputOutcome::Steered,
        )
        .await?;
    Ok(())
}

async fn replaced_launch_does_not_inherit_slot(
    store: &Store,
    f: &Fixture,
    conversation: &str,
    ending: &ConversationNormalEnding,
) -> Result<(), TestError> {
    let owner = f.owner.digest();
    let credential = f.connector.digest();
    let source = f.source(&credential);
    let route = store
        .resolve_connector_route(&f.agent.computer.id)
        .await?
        .ok_or("Missing Connector route")?;
    let Decision::Admitted { operation_id } = store
        .admit_conversation_continuation(&source, conversation, ending)
        .await?
    else {
        return Err("Expected outstanding slot before launch replacement".into());
    };
    let next_project = ProjectInstructions::new(vec![InstructionSource {
        kind: InstructionSourceKind::Product,
        text: "New launch with the same retained native conversation.".into(),
    }])?;
    let Some(ConnectionStartupAdmission::Dispatch {
        operation_id: next, ..
    }) = store
        .admit_project_refresh(
            &route,
            &f.agent.id,
            conversation,
            &next_project,
            br#"{"tools":[]}"#,
            f.now + 1,
        )
        .await?
    else {
        return Err("Expected actual successor launch".into());
    };
    let setup = ConversationInputPreparation {
        agent_id: &f.agent.id,
        conversation_id: conversation,
        operation_id: &next,
    };
    // This old occurrence is already in the retained local copy but has not
    // yet been uploaded to the server when the new launch records its floor.
    let mut old_unreported = ConversationNormalEnding {
        target: ending.target.clone(),
        position: position(ending.position.ordinal() + 1, "old-unreported-ending")?,
        work_id: "old-unreported-work".into(),
        launch_operation_id: next.clone(),
    };
    let capture = chapchoo_domain::LaunchCapture {
        snapshot: ending.target.snapshot.clone(),
        through: Some(old_unreported.position.clone()),
    };
    store
        .begin_execution_authentication(
            &source,
            &setup,
            Some(&ending.launch_operation_id),
            Some(&capture),
        )
        .await?;
    assert!(matches!(
        store
            .begin_execution_authentication(
                &source,
                &setup,
                Some(&ending.launch_operation_id),
                Some(&chapchoo_domain::LaunchCapture {
                    through: None,
                    ..capture.clone()
                }),
            )
            .await,
        Err(StoreError::Conflict)
    ));
    store
        .complete_conversation_input_preparation(
            &source,
            &setup,
            &ConversationPreparationOutcome::SessionReady(ending.target.clone()),
        )
        .await?;
    assert!(matches!(
        store
            .admit_conversation_continuation(&source, conversation, ending)
            .await,
        Err(StoreError::Conflict)
    ));
    assert!(
        store
            .begin_conversation_continuation(&source, conversation, &operation_id, ending)
            .await?
            .is_none()
    );
    let terminal = serde_json::to_vec(&serde_json::json!({
        "method":"session/update",
        "params":{"sessionId":ending.target.native_session.as_str(),
            "update":{"sessionUpdate":"state_update","state":"idle","stopReason":"end_turn"}}
    }))?;
    for is_after_launch in [false, true] {
        if is_after_launch {
            old_unreported.position =
                position(old_unreported.position.ordinal() + 1, "new-launch-ending")?;
            old_unreported.work_id = "new-launch-work".into();
        }
        store
            .ingest_conversation_pair(
                &source,
                conversation,
                &CapturedEventPair {
                    snapshot: capture.snapshot.clone(),
                    position: old_unreported.position.clone(),
                    raw: terminal.clone(),
                    normalized: b"fixture".to_vec(),
                },
            )
            .await?;
        let decision = store
            .admit_conversation_continuation(&source, conversation, &old_unreported)
            .await;
        if is_after_launch {
            let Decision::Admitted { operation_id } = decision? else {
                return Err("Fresh occurrence after the launch floor was not admitted".into());
            };
            store
                .close_conversation_continuation(
                    &source,
                    conversation,
                    &operation_id,
                    &old_unreported,
                )
                .await?;
        } else {
            assert!(matches!(decision, Err(StoreError::InvalidTransition)));
        }
    }
    let next_input = store
        .accept_conversation_input(
            &owner,
            &AcceptConversationInput {
                agent_id: f.agent.id.clone(),
                conversation_id: Some(conversation.into()),
                request_id: "successor-user-input".into(),
                target: ConversationTarget::Snapshot(ending.target.snapshot.clone()),
                text: ConversationText::parse("User input on the successor".into())?,
            },
            f.now + 2,
        )
        .await?;
    let draft = ConversationOperationForwarding {
        agent_id: &f.agent.id,
        conversation_id: conversation,
        operation_id: &next_input.id,
        target: &ending.target,
    };
    let claim = f.claim(store, conversation, &next_input.id).await?;
    let ConversationForwardingAdmission::Granted(grant) = store
        .begin_conversation_input_forwarding(&owner, &route, &draft, f.now + 2, &claim)
        .await?
    else {
        return Err("The replacement launch did not acquire an ordinary slot".into());
    };
    let authority = chapchoo_persistence::ConversationInputAuthority {
        agent_id: f.agent.id.clone(),
        conversation_id: conversation.into(),
        input_id: next_input.id.clone(),
        claim_id: claim,
    };
    let connecting = store
        .prepare_connector_connection(&f.session, &credential, &f.control)
        .await?;
    let replacement = store
        .admit_connector_connection(
            &f.session,
            &credential,
            connecting.expected.as_ref(),
            &connecting.owner,
            DirectoryPolicy::default(),
        )
        .await?;
    let reconnected = ConnectorSource {
        connection_id: &replacement.connection_id,
        ..source
    };
    assert!(
        store
            .queued_execution_input_pending(&authority, &grant.execution_id)
            .await?,
        "A replacement route released a still-owned input"
    );
    assert!(
        store
            .connector_input_chunk(&source, &draft, 0)
            .await
            .is_err()
    );
    assert!(
        !store
            .connector_input_chunk(&reconnected, &draft, 0)
            .await?
            .bytes
            .is_empty()
    );
    store
        .fail_conversation_input(conversation, &next_input.id)
        .await?;
    assert!(
        !store
            .queued_execution_input_pending(&authority, &grant.execution_id)
            .await?
    );
    assert!(matches!(
        store.connector_input_chunk(&reconnected, &draft, 0).await,
        Err(StoreError::NotFound)
    ));
    Ok(())
}

#[test]
fn sqlite_goal_projection_and_continuation_admission() -> Result<(), TestError> {
    let root = tempfile::tempdir()?;
    let path = root.path().join("goals.sqlite");
    let cipher = cipher()?;
    prepare_sqlite(&path, &cipher)?;
    let rt = tokio::runtime::Runtime::new()?;
    let store = rt.block_on(Store::connect_sqlite(path.clone(), cipher))?;
    let fixture = rt.block_on(fixture(&store))?;
    let mut connection = diesel::SqliteConnection::establish(path.to_str().ok_or("Invalid path")?)?;
    rt.block_on(exercise(&store, &fixture, |sql| {
        connection.batch_execute(sql)?;
        Ok(())
    }))?;
    assert_eq!(
        connection
            .revert_last_migration(SQLITE_MIGRATIONS)?
            .to_string(),
        "00000000000029"
    );
    assert!(connection.revert_last_migration(SQLITE_MIGRATIONS).is_err());
    Ok(())
}

#[test]
fn postgres_goal_projection_and_continuation_admission() -> Result<(), TestError> {
    let _fixture_lock = POSTGRES_CONVERSATION_LOCK
        .lock()
        .map_err(|_| "PostgreSQL fixture lock poisoned")?;
    let base = std::env::var("CHAPCHOO_TEST_POSTGRES_URL")
        .map_err(|_| "Disposable PostgreSQL is required")?;
    let mut control = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&base)?;
    let suffix = BearerSecret::generate()?.digest().as_bytes()[..8]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let schema = format!("goals_{suffix}");
    control.batch_execute(&format!("CREATE SCHEMA {schema}"))?;
    let separator = if base.contains('?') { '&' } else { '?' };
    let url = format!("{base}{separator}options=-csearch_path%3D{schema}");
    let result = (|| -> Result<(), TestError> {
        let cipher = cipher()?;
        prepare_postgres(&url, &cipher)?;
        let rt = tokio::runtime::Runtime::new()?;
        let store = rt.block_on(Store::connect_postgres(url.clone(), cipher))?;
        let fixture = rt.block_on(fixture(&store))?;
        let mut connection = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&url)?;
        rt.block_on(exercise(&store, &fixture, |sql| {
            tokio::task::block_in_place(|| {
                connection.batch_execute(sql)?;
                Ok(())
            })
        }))?;
        assert_eq!(
            connection
                .revert_last_migration(POSTGRES_MIGRATIONS)?
                .to_string(),
            "00000000000029"
        );
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
