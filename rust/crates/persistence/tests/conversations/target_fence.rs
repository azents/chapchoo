//! A Running target during Stop belongs to the successor, not its retiring route.
use super::*;
use chapchoo_persistence::{
    ConnectionStartupAdmission, ConversationDeliveryTarget, ConversationForwardingAdmission,
    ConversationInputCandidate, ConversationInputPreparation, ConversationInputQueueState,
    ConversationOperationForwarding, ConversationPreparationAdmission,
    ConversationPreparationOutcome, ConversationPreparationState,
};

async fn exercise(
    store: &Store,
    f: &Fixture,
    mut sql: impl FnMut(&str) -> Result<(), TestError>,
) -> Result<(), TestError> {
    let owner = f.owner.digest();
    let key = f.connector.digest();
    let source = f.source(&key);
    let computer = &f.agent.computer.id;
    let route = store
        .resolve_connector_route(computer)
        .await?
        .ok_or("Missing live route")?;
    let ConnectionStartupAdmission::Dispatch {
        conversation_id,
        operation_id,
        requested_supply,
        ..
    } = store
        .admit_connection_startup(computer, &route)
        .await?
        .pop()
        .ok_or("Missing startup grant")?
    else {
        return Err("Expected startup dispatch".into());
    };
    let startup = ConversationInputPreparation {
        agent_id: &f.agent.id,
        conversation_id: &conversation_id,
        operation_id: &operation_id,
    };
    let instructions =
        chapchoo_domain::ProjectInstructions::new(vec![chapchoo_domain::InstructionSource {
            kind: chapchoo_domain::InstructionSourceKind::Product,
            text: "Preserve existing user files.".into(),
        }])?;
    store
        .seal_startup_project_instructions(
            &route,
            &startup,
            &instructions,
            br#"{"tools":[]}"#,
            now()?,
        )
        .await?;
    let target = ConversationDeliveryTarget {
        snapshot: snapshot("target-fence-source")?,
        native_session: NativeConversationId::parse("target-fence-native".into())?,
    };
    store
        .bind_conversation_source(
            &source,
            &SourceBinding {
                conversation_id: conversation_id.clone(),
                expected_previous: None,
                snapshot: target.snapshot.clone(),
                native_session: target.native_session.clone(),
            },
        )
        .await?;
    store
        .complete_conversation_source(&source, &conversation_id, &target.snapshot, None)
        .await?;
    let input = store
        .accept_conversation_input(
            &owner,
            &AcceptConversationInput {
                agent_id: f.agent.id.clone(),
                conversation_id: Some(conversation_id.clone()),
                request_id: "before-stop-target".into(),
                target: ConversationTarget::Snapshot(target.snapshot.clone()),
                text: ConversationText::parse("Retain until the successor is available".into())?,
            },
            now()?,
        )
        .await?;
    let candidate = ConversationInputCandidate {
        id: input.id.clone(),
        agent_id: f.agent.id.clone(),
        conversation_id: conversation_id.clone(),
    };
    let claim = store
        .claim_conversation_input(&candidate, DirectoryPolicy::default())
        .await?
        .ok_or("Missing pre-Stop claim")?;
    let draft = ConversationInputPreparation {
        operation_id: &input.id,
        ..startup
    };
    let forwarding = ConversationOperationForwarding {
        agent_id: &f.agent.id,
        conversation_id: &conversation_id,
        operation_id: &input.id,
        target: &target,
    };
    // Only disposable persistence fixtures synthesize these provider/lifecycle facts.
    // The Connector remains current and authenticated throughout both phases.
    for phase in ["RETIRING", "UNCERTAIN"] {
        sql(&format!(
            "UPDATE computers SET desired_state='RUNNING',current_action='STOP',current_phase='{phase}',expected_old_execution_id=current_execution_id WHERE id=(SELECT computer_id FROM agents WHERE handle='chat-fixture')"
        ))?;
        assert!(store.resolve_connector_route(computer).await?.is_some());
        assert!(
            matches!(
                store
                    .begin_conversation_input_preparation(
                        &owner,
                        &route,
                        &draft,
                        now()?,
                        &claim.claim_id
                    )
                    .await,
                Err(StoreError::InvalidTransition)
            ),
            "A pre-Stop claim grants no new setup on the retiring execution"
        );
        assert!(
            !store
                .connection_startup_may_dispatch(computer, &route, &operation_id)
                .await?
        );
        assert!(
            store
                .admit_connection_startup(computer, &route)
                .await?
                .is_empty()
        );
        assert!(matches!(
            store
                .check_conversation_input_preparation(
                    &source,
                    &startup,
                    &requested_supply,
                    Some((true, false)),
                    None
                )
                .await,
            Err(StoreError::InvalidTransition)
        ));
        assert_eq!(
            store
                .connection_startup(&owner, &f.agent.id, now()?)
                .await?
                .ok_or("Missing retained startup")?
                .facts
                .state,
            ConversationPreparationState::PossibleForwarding
        );
    }
    // A valid late report remains accepted; blocked dispatch never declares nonexecution.
    store
        .complete_conversation_input_preparation(
            &source,
            &startup,
            &ConversationPreparationOutcome::SessionReady(target.clone()),
        )
        .await?;
    assert!(matches!(
        store
            .connection_startup(&owner, &f.agent.id, now()?)
            .await?
            .ok_or("Missing completed startup")?
            .facts
            .state,
        ConversationPreparationState::Known(ConversationPreparationOutcome::SessionReady(_))
    ));
    let selected = store
        .latest_system_prompt(&owner, &f.agent.id, now()?)
        .await?;
    let changed =
        chapchoo_domain::ProjectInstructions::new(vec![chapchoo_domain::InstructionSource {
            kind: chapchoo_domain::InstructionSourceKind::Product,
            text: "Updated product guidance for the successor.".into(),
        }])?;
    assert!(
        store
            .admit_project_refresh(
                &route,
                &f.agent.id,
                &conversation_id,
                &changed,
                br#"{"tools":[]}"#,
                now()?
            )
            .await?
            .is_none(),
        "A project refresh cannot grant another startup to the retiring execution"
    );
    assert_eq!(
        store
            .latest_system_prompt(&owner, &f.agent.id, now()?)
            .await?,
        selected
    );
    assert_eq!(
        store
            .connection_startup(&owner, &f.agent.id, now()?)
            .await?
            .ok_or("Missing retained startup")?
            .operation_id,
        operation_id
    );
    assert!(
        matches!(
            store
                .begin_conversation_input_forwarding(
                    &owner,
                    &route,
                    &forwarding,
                    now()?,
                    &claim.claim_id
                )
                .await,
            Err(StoreError::InvalidTransition)
        ),
        "Even an otherwise ready source and applied setup cannot forward to the retiring execution"
    );
    assert!(
        store
            .conversation_input_preparation(&owner, &draft, now()?)
            .await?
            .is_none()
    );
    assert!(
        store
            .conversation_input_delivery(&owner, &f.agent.id, &conversation_id, &input.id, now()?)
            .await?
            .is_none()
    );
    assert_eq!(
        store
            .release_conversation_input_claim(&conversation_id, &input.id, &claim.claim_id, false)
            .await?,
        ConversationInputQueueState::Pending
    );
    assert!(
        store
            .claim_conversation_input(&candidate, DirectoryPolicy::default())
            .await?
            .is_none()
    );
    // Model the old execution as a separate retired predecessor. Stored Stop
    // provenance stays in place while the live current route is now its successor.
    sql(
        "INSERT INTO computer_executions(id,computer_id,initial_allocation,created_at,retired_at) SELECT i.id,a.computer_id,FALSE,i.accepted_at,i.accepted_at FROM conversation_inputs i JOIN conversations c ON c.id=i.conversation_id JOIN agents a ON a.id=c.agent_id WHERE i.request_id='before-stop-target'; UPDATE computers SET expected_old_execution_id=(SELECT id FROM conversation_inputs WHERE request_id='before-stop-target') WHERE id=(SELECT computer_id FROM agents WHERE handle='chat-fixture')",
    )?;
    let next = store
        .claim_conversation_input(&candidate, DirectoryPolicy::default())
        .await?
        .ok_or("Successor must restore claim eligibility despite stored Stop")?;
    assert!(matches!(
        store
            .begin_conversation_input_preparation(&owner, &route, &draft, now()?, &next.claim_id)
            .await?,
        ConversationPreparationAdmission::ObservedStartup { .. }
    ));
    assert!(matches!(
        store
            .begin_conversation_input_forwarding(
                &owner,
                &route,
                &forwarding,
                now()?,
                &next.claim_id
            )
            .await?,
        ConversationForwardingAdmission::Granted(_)
    ));
    // An admitted uncertainty remains recoverable facts after the target fence returns.
    sql(
        "UPDATE computers SET expected_old_execution_id=current_execution_id WHERE id=(SELECT computer_id FROM agents WHERE handle='chat-fixture')",
    )?;
    assert!(matches!(
        store
            .begin_conversation_input_forwarding(
                &owner,
                &route,
                &forwarding,
                now()?,
                &next.claim_id
            )
            .await?,
        ConversationForwardingAdmission::Existing(
            chapchoo_persistence::ConversationInputDelivery::PossibleForwarding
        )
    ));
    store
        .complete_conversation_input_forwarding(
            &source,
            &forwarding,
            &chapchoo_persistence::ConversationInputOutcome::Steered,
        )
        .await?;
    assert!(
        !store
            .queued_conversation_input_pending(&chapchoo_persistence::ConversationInputAuthority {
                agent_id: f.agent.id.clone(),
                conversation_id: conversation_id.clone(),
                input_id: input.id.clone(),
                claim_id: next.claim_id.clone(),
            })
            .await?,
        "Consumed input closes observation before missing session binding is considered"
    );
    assert!(
        store
            .conversation_input_reference(
                &owner,
                &f.agent.id,
                &conversation_id,
                &input.request_id,
                now()?
            )
            .await?
            .is_none()
    );
    sql(
        "UPDATE computers SET expected_old_execution_id=(SELECT id FROM computer_executions WHERE retired_at IS NOT NULL LIMIT 1) WHERE id=(SELECT computer_id FROM agents WHERE handle='chat-fixture')",
    )?;
    assert!(
        matches!(
            store
                .admit_project_refresh(
                    &route,
                    &f.agent.id,
                    &conversation_id,
                    &changed,
                    br#"{"tools":[]}"#,
                    now()?
                )
                .await?,
            Some(ConnectionStartupAdmission::Dispatch { .. })
        ),
        "The successor can still admit the changed configuration"
    );
    Ok(())
}

#[test]
fn sqlite_stop_running_target_waits_for_successor() -> Result<(), TestError> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("target.sqlite");
    let cipher = cipher()?;
    prepare_sqlite(&path, &cipher)?;
    let mut connection = diesel::SqliteConnection::establish(path.to_str().ok_or("Invalid path")?)?;
    connection.batch_execute("PRAGMA foreign_keys=ON")?;
    tokio::runtime::Runtime::new()?.block_on(async {
        let store = Store::connect_sqlite(path, cipher).await?;
        let f = fixture(&store).await?;
        exercise(&store, &f, |query| {
            connection.batch_execute(query)?;
            Ok(())
        })
        .await
    })
}

#[test]
fn postgres_stop_running_target_waits_for_successor() -> Result<(), TestError> {
    let _lock = POSTGRES_CONVERSATION_LOCK
        .lock()
        .map_err(|_| "Fixture lock poisoned")?;
    let base = std::env::var("CHAPCHOO_TEST_POSTGRES_URL")
        .map_err(|_| "Disposable PostgreSQL is required")?;
    let mut control = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&base)?;
    let schema = format!("target_fence_{}", std::process::id());
    control.batch_execute(&format!("CREATE SCHEMA {schema}"))?;
    let separator = if base.contains('?') { '&' } else { '?' };
    let url = format!("{base}{separator}options=-csearch_path%3D{schema}");
    let result = (|| -> Result<(), TestError> {
        let cipher = cipher()?;
        prepare_postgres(&url, &cipher)?;
        let mut connection = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&url)?;
        tokio::runtime::Runtime::new()?.block_on(async {
            let store = Store::connect_postgres(url, cipher).await?;
            let f = fixture(&store).await?;
            exercise(&store, &f, |query| {
                std::thread::scope(|scope| {
                    scope
                        .spawn(|| -> Result<(), TestError> {
                            connection.batch_execute(query)?;
                            Ok(())
                        })
                        .join()
                        .map_err(|_| "Fixture SQL worker failed")?
                })
            })
            .await
        })
    })();
    support::combine(
        result,
        control
            .batch_execute(&format!("DROP SCHEMA {schema} CASCADE"))
            .map_err(Into::into),
    )
    .map(|_| ())
}
