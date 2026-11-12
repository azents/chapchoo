use super::*;
use chapchoo_persistence::{
    ConnectionStartupAdmission as Startup, ConnectionStartupContext,
    ConversationInputPreparation as Draft, ConversationPreparationAdmission as Admission,
    ConversationPreparationOutcome as Outcome, ConversationPreparationState as State,
};

const WITHDRAW_SELECTION: &str = "CREATE TEMP TABLE fixture_authentication_backup AS SELECT * FROM agent_authentication_selections WHERE agent_id=(SELECT id FROM agents WHERE handle='chat-fixture'); DELETE FROM agent_authentication_selections WHERE agent_id=(SELECT id FROM agents WHERE handle='chat-fixture')";
const RESTORE_SELECTION: &str = "INSERT INTO agent_authentication_selections SELECT * FROM fixture_authentication_backup; DROP TABLE fixture_authentication_backup";

async fn missing_authentication(store: &Store, f: &Fixture) -> Result<(), TestError> {
    let route = store
        .resolve_connector_route(&f.agent.computer.id)
        .await?
        .ok_or("Missing route")?;
    assert!(
        store
            .current_conversation(&f.owner.digest(), &f.agent.id, f.now)
            .await?
            .is_none()
    );
    assert!(matches!(
        store
            .admit_connection_startup(&f.agent.computer.id, &route)
            .await?
            .pop(),
        Some(Startup::AuthenticationRequired { .. })
    ));
    let current = store
        .current_conversation(&f.owner.digest(), &f.agent.id, f.now)
        .await?
        .ok_or("Missing empty conversation")?;
    assert!(matches!(current.source, ConversationSource::NotStarted));
    assert!(current.native_session_id.is_none());
    assert!(
        store
            .connection_startup(&f.owner.digest(), &f.agent.id, f.now)
            .await?
            .is_none()
    );
    assert!(
        store
            .conversation_input(
                &f.owner.digest(),
                &f.agent.id,
                &current.id,
                "first-real-input",
                f.now
            )
            .await?
            .is_none()
    );
    Ok(())
}

async fn exercise(
    store: &Store,
    f: &Fixture,
    mut set_desired: impl FnMut(&str) -> Result<(), TestError>,
) -> Result<(), TestError> {
    let owner = f.owner.digest();
    let key = f.connector.digest();
    let source = f.source(&key);
    let computer = &f.agent.computer.id;
    let route = store
        .resolve_connector_route(computer)
        .await?
        .ok_or("Missing route")?;
    assert!(
        store
            .running_computer_ids(None, 1)
            .await?
            .contains(computer)
    );
    assert!(
        store
            .running_computer_ids(Some(computer), 1)
            .await?
            .is_empty()
    );
    let empty = store
        .current_conversation(&owner, &f.agent.id, f.now)
        .await?
        .ok_or("Missing empty conversation")?;
    assert!(matches!(empty.source, ConversationSource::NotStarted));
    let (one, two) = tokio::join!(
        store.admit_connection_startup(computer, &route),
        store.admit_connection_startup(computer, &route),
    );
    let first = one?.pop().ok_or("Missing startup")?;
    let Startup::Dispatch {
        conversation_id,
        operation_id,
        requested_supply,
        context,
        fresh_native_allowed,
        require_retained_source,
        ..
    } = first
    else {
        return Err("Expected initial startup".into());
    };
    assert_eq!(conversation_id, empty.id);
    assert!(matches!(context, ConnectionStartupContext::Initial));
    assert!(fresh_native_allowed);
    assert!(!require_retained_source);
    assert!(
        matches!(two?.pop(), Some(Startup::Dispatch { operation_id: same, .. }) if same == operation_id)
    );
    assert!(
        store
            .conversation_input(
                &owner,
                &f.agent.id,
                &conversation_id,
                "first-real-input",
                f.now
            )
            .await?
            .is_none()
    );
    assert!(matches!(
        store
            .connection_startup(&f.other.digest(), &f.agent.id, f.now)
            .await,
        Err(StoreError::NotFound)
    ));
    let draft = Draft {
        agent_id: &f.agent.id,
        conversation_id: &conversation_id,
        operation_id: &operation_id,
    };
    store
        .check_conversation_input_preparation(
            &source,
            &draft,
            &requested_supply,
            Some((true, false)),
            None,
        )
        .await?;
    assert!(
        store
            .connection_startup_may_dispatch(computer, &route, &operation_id)
            .await?
    );
    // A stop after any prior dispatch cannot turn unknown effects into a safe
    // fresh grant. Native dequeue checks the current desired state as well.
    set_desired("STOPPED")?;
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
                &draft,
                &requested_supply,
                Some((true, false)),
                None
            )
            .await,
        Err(StoreError::InvalidTransition)
    ));
    assert_eq!(
        store
            .connection_startup(&owner, &f.agent.id, f.now)
            .await?
            .ok_or("Missing startup")?
            .facts
            .state,
        State::PossibleForwarding
    );
    set_desired("RUNNING")?;
    assert!(
        matches!(store.admit_connection_startup(computer, &route).await?.pop(), Some(Startup::Dispatch { operation_id: same, .. }) if same == operation_id)
    );
    let applied = store
        .begin_execution_authentication(&source, &draft, None)
        .await?;
    assert_eq!(
        applied.authentication_id,
        f.agent
            .authentication
            .as_ref()
            .ok_or("Missing auth")?
            .authentication_id
    );
    let snapshot = snapshot("startup-snapshot")?;
    let native = NativeConversationId::parse("startup-native".into())?;
    store
        .bind_conversation_source(
            &source,
            &SourceBinding {
                conversation_id: conversation_id.clone(),
                expected_previous: None,
                snapshot: snapshot.clone(),
                native_session: native.clone(),
            },
        )
        .await?;
    // The owner can send its first real input while capture is still loading.
    let first = store
        .accept_conversation_input(
            &owner,
            &AcceptConversationInput {
                agent_id: f.agent.id.clone(),
                conversation_id: Some(conversation_id.clone()),
                request_id: "first-real-input".into(),
                target: ConversationTarget::NotStarted,
                text: ConversationText::parse("first real user message".into())?,
            },
            f.now,
        )
        .await?;
    assert_eq!(first.conversation_id, conversation_id);
    let input = Draft {
        operation_id: &first.id,
        ..draft
    };
    assert!(
        matches!(store.begin_conversation_input_preparation(&owner, &route, &input, f.now).await?, Admission::ObservedStartup { operation_id: same, .. } if same == operation_id)
    );
    store
        .complete_conversation_source(&source, &conversation_id, &snapshot, None)
        .await?;
    let target = chapchoo_persistence::ConversationDeliveryTarget {
        snapshot,
        native_session: native,
    };
    store
        .complete_conversation_input_preparation(
            &source,
            &draft,
            &Outcome::SessionReady(target.clone()),
        )
        .await?;
    store
        .complete_conversation_input_preparation(
            &source,
            &draft,
            &Outcome::SessionReady(target.clone()),
        )
        .await?;
    assert!(matches!(
        store
            .admit_connection_startup(computer, &route)
            .await?
            .pop(),
        Some(Startup::Settled { .. })
    ));
    assert!(
        matches!(store.begin_conversation_input_preparation(&owner, &route, &input, f.now).await?, Admission::ObservedStartup { facts, .. } if facts.state == State::Known(Outcome::SessionReady(target.clone())))
    );
    assert!(matches!(
        store
            .begin_conversation_input_forwarding(
                &owner,
                &route,
                &chapchoo_persistence::ConversationOperationForwarding {
                    agent_id: &f.agent.id,
                    conversation_id: &conversation_id,
                    operation_id: &first.id,
                    target: &target,
                },
                f.now
            )
            .await?,
        chapchoo_persistence::ConversationForwardingAdmission::Granted(_)
    ));
    // A new Connector process may join the same execution. Historical READY
    // is not proof that its new NativeOwner has a live session.
    let next = store
        .prepare_connector_connection(&f.session, &key, &f.control)
        .await?;
    store
        .admit_connector_connection(
            &f.session,
            &key,
            next.expected.as_ref(),
            &next.owner,
            DirectoryPolicy::default(),
        )
        .await?;
    let next_source = ConnectorSource {
        connection_id: &next.owner.connection_id,
        ..source
    };
    let next_route = store
        .resolve_connector_route(computer)
        .await?
        .ok_or("Missing new route")?;
    let resumed = store
        .admit_connection_startup(computer, &next_route)
        .await?
        .pop()
        .ok_or("Missing retained setup")?;
    let Startup::Dispatch {
        operation_id: resume_id,
        fresh_native_allowed: false,
        context:
            ConnectionStartupContext::Existing {
                snapshot: resumed_snapshot,
                native_session: resumed_native,
            },
        ..
    } = resumed
    else {
        return Err("Expected exact retained resume on replacement connection".into());
    };
    assert_ne!(resume_id, operation_id);
    assert_eq!(resumed_snapshot, target.snapshot);
    assert_eq!(resumed_native, target.native_session);
    let resume_draft = Draft {
        operation_id: &resume_id,
        ..draft
    };
    store
        .check_conversation_input_preparation(
            &next_source,
            &resume_draft,
            &requested_supply,
            None,
            Some(&target),
        )
        .await?;
    assert!(
        matches!(store.begin_conversation_input_preparation(&owner, &next_route, &input, f.now).await?, Admission::ObservedStartup { operation_id: same, facts } if same == resume_id && facts.state == State::PossibleForwarding)
    );
    store
        .complete_conversation_input_preparation(
            &next_source,
            &resume_draft,
            &Outcome::SessionReady(target.clone()),
        )
        .await?;
    assert!(matches!(
        store
            .admit_connection_startup(computer, &next_route)
            .await?
            .pop(),
        Some(Startup::Settled { .. })
    ));
    store
        .complete_conversation_input_forwarding(
            &next_source,
            &chapchoo_persistence::ConversationOperationForwarding {
                agent_id: &f.agent.id,
                conversation_id: &conversation_id,
                operation_id: &first.id,
                target: &target,
            },
            &chapchoo_persistence::ConversationInputOutcome::Steered,
        )
        .await?;
    assert!(
        store
            .conversation_input_reference(
                &owner,
                &f.agent.id,
                &conversation_id,
                &first.request_id,
                f.now
            )
            .await?
            .is_none()
    );
    let subsequent = store
        .accept_conversation_input(
            &owner,
            &AcceptConversationInput {
                agent_id: f.agent.id.clone(),
                conversation_id: Some(conversation_id.clone()),
                request_id: "after-consumed-input".into(),
                target: ConversationTarget::NotStarted,
                text: ConversationText::parse("A new explicit input".into())?,
            },
            f.now,
        )
        .await?;
    assert!(
        matches!(store.begin_conversation_input_preparation(&owner, &next_route, &Draft {
            agent_id: &f.agent.id,
            conversation_id: &conversation_id,
            operation_id: &subsequent.id,
        }, f.now).await?, Admission::ObservedStartup { facts, .. } if !facts.fresh_native_allowed && facts.state == State::Known(Outcome::SessionReady(target)))
    );
    Ok(())
}

#[test]
fn sqlite_connection_startup_and_first_input() -> Result<(), TestError> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("startup.sqlite");
    let cipher = cipher()?;
    prepare_sqlite(&path, &cipher)?;
    let runtime = tokio::runtime::Runtime::new()?;
    let store = runtime.block_on(Store::connect_sqlite(path.clone(), cipher))?;
    let f = runtime.block_on(fixture(&store))?;
    let mut connection = diesel::SqliteConnection::establish(path.to_str().ok_or("Invalid path")?)?;
    connection.batch_execute(WITHDRAW_SELECTION)?;
    runtime.block_on(missing_authentication(&store, &f))?;
    connection.batch_execute(RESTORE_SELECTION)?;
    runtime.block_on(exercise(&store, &f, |desired| {
        diesel::sql_query("UPDATE computers SET desired_state=? WHERE id=(SELECT computer_id FROM agents WHERE handle='chat-fixture')")
            .bind::<diesel::sql_types::Text, _>(desired).execute(&mut connection)?;
        Ok(())
    }))?;
    isolate_pre_chat_migration!(&mut connection, SQLITE_MIGRATIONS);
    assert_eq!(
        connection
            .revert_last_migration(SQLITE_MIGRATIONS)?
            .to_string(),
        "00000000000022"
    );
    assert!(connection.revert_last_migration(SQLITE_MIGRATIONS).is_err());
    Ok(())
}

#[test]
fn postgres_connection_startup_and_first_input() -> Result<(), TestError> {
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
    let schema = format!("startup_{suffix}");
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
        connection.batch_execute(WITHDRAW_SELECTION)?;
        runtime.block_on(missing_authentication(&store, &f))?;
        connection.batch_execute(RESTORE_SELECTION)?;
        runtime.block_on(exercise(&store, &f, |desired| {
            std::thread::scope(|scope| {
                scope.spawn(|| -> Result<(), TestError> {
                    diesel::sql_query("UPDATE computers SET desired_state=$1 WHERE id=(SELECT computer_id FROM agents WHERE handle='chat-fixture')")
                        .bind::<diesel::sql_types::Text, _>(desired).execute(&mut connection)?;
                    Ok(())
                }).join().map_err(|_| "Desired-state fixture worker failed")?
            })
        }))?;
        isolate_pre_chat_migration!(&mut connection, POSTGRES_MIGRATIONS);
        assert_eq!(
            connection
                .revert_last_migration(POSTGRES_MIGRATIONS)?
                .to_string(),
            "00000000000022"
        );
        assert!(
            connection
                .revert_last_migration(POSTGRES_MIGRATIONS)
                .is_err()
        );
        Ok(())
    })();
    let cleanup = control.batch_execute(&format!("DROP SCHEMA {schema} CASCADE"));
    support::combine(result, cleanup.map_err(Into::into)).map(|_| ())
}
