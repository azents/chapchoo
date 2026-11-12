use super::*;
#[path = "authentication_refresh.rs"]
mod refresh;
use chapchoo_persistence::{
    ConversationInputPreparation as Draft, DeleteAuthentication, DeleteAuthenticationOutcome,
    ExpectedAgentAuthentication, ReplaceApiKey, SelectAgentAuthentication,
};

async fn preparation(
    store: &Store,
    f: &Fixture,
    request: &str,
) -> Result<chapchoo_persistence::ConversationInputReceipt, TestError> {
    let current = store
        .current_conversation(&f.owner.digest(), &f.agent.id, f.now)
        .await?;
    let input = store
        .accept_conversation_input(
            &f.owner.digest(),
            &AcceptConversationInput {
                agent_id: f.agent.id.clone(),
                conversation_id: current.map(|c| c.id),
                request_id: request.into(),
                target: ConversationTarget::NotStarted,
                text: ConversationText::parse("synthetic authentication start".into())?,
            },
            f.now,
        )
        .await?;
    let route = store
        .resolve_connector_route(&f.agent.computer.id)
        .await?
        .ok_or("Missing route")?;
    store
        .begin_conversation_input_preparation(&f.owner.digest(), &route, &draft(f, &input), f.now)
        .await?;
    Ok(input)
}
fn draft<'a>(
    f: &'a Fixture,
    input: &'a chapchoo_persistence::ConversationInputReceipt,
) -> Draft<'a> {
    Draft {
        agent_id: &f.agent.id,
        conversation_id: &input.conversation_id,
        operation_id: &input.id,
    }
}
async fn exercise(store: &Store, f: &Fixture) -> Result<(), TestError> {
    let key = f.connector.digest();
    let source = f.source(&key);
    let first = preparation(store, f, "authentication-start-one").await?;
    let start = draft(f, &first);
    let saved = f
        .agent
        .authentication
        .as_ref()
        .ok_or("Missing saved authentication")?;
    assert!(
        store
            .execution_authentication_binding(&source, &start)
            .await?
            .is_none()
    );
    assert!(matches!(
        store
            .execution_authentication_material(&source, &start, None)
            .await,
        Err(StoreError::NotFound)
    ));
    // Simultaneous consumers recover one exact binding, never two live choices.
    let (one, two) = tokio::join!(
        store.begin_execution_authentication(&source, &start, None),
        store.begin_execution_authentication(&source, &start, None),
    );
    let applied = one?;
    assert_eq!(applied, two?);
    assert_eq!(applied.authentication_id, saved.authentication_id);
    assert_eq!(applied.selection_request_id, saved.request_id);
    let material = store
        .execution_authentication_material(&source, &start, None)
        .await?;
    assert_eq!(material.secret.expose(), b"synthetic-never-used-model-key");
    let original_source = material.source;
    let scope = AuthenticationScope::Team {
        team_id: f.team_id.clone(),
    };
    let other = match store
        .register_external_api_key(
            &f.owner.digest(),
            &RegisterApiKey {
                scope: scope.clone(),
                request_id: "authentication-next-record".into(),
                name: String::new(),
                key: ExternalSecret::new(b"synthetic-next-key".to_vec())?,
            },
            f.now,
        )
        .await?
    {
        AuthenticationAcceptedResult::Record { authentication_id } => authentication_id,
        _ => return Err("Expected second record".into()),
    };
    store
        .select_agent_authentication(
            &f.owner.digest(),
            &SelectAgentAuthentication {
                agent_id: f.agent.id.clone(),
                request_id: "authentication-next-selection".into(),
                authentication_id: other.clone(),
                expected: Some(ExpectedAgentAuthentication {
                    authentication_id: saved.authentication_id.clone(),
                    request_id: saved.request_id.clone(),
                }),
            },
            f.now,
        )
        .await?;
    // A saved future choice cannot retarget this operation or its secret read.
    assert_eq!(
        store
            .begin_execution_authentication(&source, &start, None)
            .await?,
        applied
    );
    let unchanged = store
        .execution_authentication_material(&source, &start, Some(&original_source))
        .await?;
    assert_eq!(unchanged.applied.authentication_id, saved.authentication_id);
    assert_eq!(unchanged.secret.expose(), b"synthetic-never-used-model-key");
    store
        .replace_external_api_key(
            &f.owner.digest(),
            &ReplaceApiKey {
                scope: scope.clone(),
                request_id: "authentication-replace-applied".into(),
                authentication_id: saved.authentication_id.clone(),
                expected_request_id: "conversation-auth".into(),
                key: ExternalSecret::new(b"synthetic-replaced-key".to_vec())?,
            },
            f.now,
        )
        .await?;
    assert!(matches!(
        store
            .execution_authentication_material(&source, &start, Some(&original_source))
            .await,
        Err(StoreError::Conflict)
    ));
    let replaced = store
        .execution_authentication_material(&source, &start, None)
        .await?;
    assert_ne!(replaced.source, original_source);
    assert_eq!(replaced.secret.expose(), b"synthetic-replaced-key");
    assert_eq!(replaced.applied, applied);
    let deletion = DeleteAuthentication {
        scope,
        request_id: "authentication-delete-applied".into(),
        authentication_id: saved.authentication_id.clone(),
        expected_request_id: "authentication-replace-applied".into(),
    };
    // The other fixture Agent still has a real saved reference. Preserve that
    // blocker before isolating the applied-only deletion case.
    assert!(matches!(
        store
            .delete_external_authentication(&f.owner.digest(), &deletion, f.now)
            .await?,
        DeleteAuthenticationOutcome::InUse
    ));
    let other_saved = store
        .agent(&f.other.digest(), &f.other_agent, f.now)
        .await?
        .authentication
        .ok_or("Missing other fixture selection")?;
    store
        .select_agent_authentication(
            &f.other.digest(),
            &SelectAgentAuthentication {
                agent_id: f.other_agent.clone(),
                request_id: "authentication-other-selection".into(),
                authentication_id: other.clone(),
                expected: Some(ExpectedAgentAuthentication {
                    authentication_id: other_saved.authentication_id,
                    request_id: other_saved.request_id,
                }),
            },
            f.now,
        )
        .await?;
    // Runtime-only provenance adds no blocker once saved references are gone.
    assert!(matches!(
        store
            .delete_external_authentication(&f.owner.digest(), &deletion, f.now)
            .await?,
        DeleteAuthenticationOutcome::Accepted(_)
    ));
    assert_eq!(
        store
            .execution_authentication_binding(&source, &start)
            .await?,
        Some(applied.clone())
    );
    assert!(matches!(
        store
            .execution_authentication_material(&source, &start, None)
            .await,
        Err(StoreError::NotFound)
    ));
    assert!(matches!(
        store
            .begin_execution_authentication(&source, &start, None)
            .await,
        Err(StoreError::NotFound)
    ));
    let next = preparation(store, f, "authentication-start-two").await?;
    let next_draft = draft(f, &next);
    assert!(matches!(
        store
            .begin_execution_authentication(&source, &next_draft, None)
            .await,
        Err(StoreError::Conflict)
    ));
    let second = store
        .begin_execution_authentication(&source, &next_draft, Some(&first.id))
        .await?;
    assert_eq!(second.authentication_id, other);
    assert_eq!(second.selection_request_id, "authentication-next-selection");
    assert!(matches!(
        store
            .begin_execution_authentication(&source, &start, Some(&next.id))
            .await,
        Err(StoreError::Conflict)
    ));
    assert!(matches!(
        store
            .execution_authentication_material(&source, &start, None)
            .await,
        Err(StoreError::Conflict)
    ));
    let current = store
        .execution_authentication_material(&source, &next_draft, None)
        .await?;
    assert_eq!(current.secret.expose(), b"synthetic-next-key");
    assert_eq!(current.applied, second);
    let wrong_agent = Draft {
        agent_id: &f.other_agent,
        ..next_draft
    };
    assert!(matches!(
        store
            .execution_authentication_material(&source, &wrong_agent, None)
            .await,
        Err(StoreError::NotFound)
    ));
    let bad_key = BearerSecret::generate()?.digest();
    assert!(matches!(
        store
            .execution_authentication_material(&f.source(&bad_key), &next_draft, None)
            .await,
        Err(StoreError::Unauthenticated)
    ));
    // Known setup readiness still grants same-record reads, never another start.
    store
        .complete_conversation_input_preparation(
            &source,
            &next_draft,
            &chapchoo_persistence::ConversationPreparationOutcome::SessionReady(
                chapchoo_persistence::ConversationDeliveryTarget {
                    snapshot: snapshot("authentication-native-copy")?,
                    native_session: NativeConversationId::parse("authentication-native".into())?,
                },
            ),
        )
        .await?;
    assert_eq!(
        store
            .execution_authentication_material(&source, &next_draft, None)
            .await?
            .applied,
        second
    );
    assert!(matches!(
        store
            .begin_execution_authentication(&source, &next_draft, Some(&first.id))
            .await,
        Err(StoreError::InvalidTransition)
    ));
    let native = NativeConversationId::parse("authentication-native".into())?;
    let original = snapshot("authentication-native-copy")?;
    store
        .bind_conversation_source(
            &source,
            &SourceBinding {
                conversation_id: next.conversation_id.clone(),
                expected_previous: None,
                snapshot: original.clone(),
                native_session: native.clone(),
            },
        )
        .await?;
    store
        .complete_conversation_source(&source, &next.conversation_id, &original, None)
        .await?;
    let replacement = snapshot("authentication-new-copy")?;
    store
        .bind_conversation_source(
            &source,
            &SourceBinding {
                conversation_id: next.conversation_id.clone(),
                expected_previous: Some(original),
                snapshot: replacement.clone(),
                native_session: native,
            },
        )
        .await?;
    store
        .complete_conversation_source(&source, &next.conversation_id, &replacement, None)
        .await?;
    // D19 clears input/preparation history, not the live execution's applied
    // authentication. These old operation IDs no longer grant a new start.
    assert!(matches!(
        store
            .conversation_input_preparation(&f.owner.digest(), &next_draft, f.now)
            .await,
        Err(StoreError::NotFound)
    ));
    assert_eq!(
        store
            .execution_authentication_material(&source, &next_draft, None)
            .await?
            .applied,
        second
    );
    assert!(matches!(
        store
            .begin_execution_authentication(&source, &next_draft, None)
            .await,
        Err(StoreError::NotFound)
    ));
    let new_route = store
        .prepare_connector_connection(&f.session, &key, &f.control)
        .await?;
    store
        .admit_connector_connection(
            &f.session,
            &key,
            new_route.expected.as_ref(),
            &new_route.owner,
            DirectoryPolicy::default(),
        )
        .await?;
    assert!(matches!(
        store
            .execution_authentication_material(&source, &next_draft, None)
            .await,
        Err(StoreError::Unauthenticated)
    ));
    let current_source = ConnectorSource {
        connection_id: &new_route.owner.connection_id,
        ..source
    };
    assert_eq!(
        store
            .execution_authentication_material(&current_source, &next_draft, None)
            .await?
            .applied,
        second
    );
    store
        .close_connector_connection(&f.session, &key, &new_route.owner)
        .await?;
    assert!(matches!(
        store
            .execution_authentication_material(&current_source, &next_draft, None)
            .await,
        Err(StoreError::Unauthenticated)
    ));
    assert_eq!(
        format!("{current:?}"),
        "ExecutionAuthenticationMaterial([REDACTED])"
    );
    refresh::exercise(store, f).await?;
    Ok(())
}

async fn record_lock_precedes_computer_admission(
    store: &Store,
    f: &Fixture,
    url: &str,
) -> Result<(), TestError> {
    use diesel_async::{AsyncConnection, SimpleAsyncConnection};
    let actor = f.owner.digest();
    let key = f.connector.digest();
    let connected = store
        .prepare_connector_connection(&f.session, &key, &f.control)
        .await?;
    store
        .admit_connector_connection(
            &f.session,
            &key,
            connected.expected.as_ref(),
            &connected.owner,
            DirectoryPolicy::default(),
        )
        .await?;
    let source = ConnectorSource {
        session_id: &f.session,
        control_id: &f.control,
        connection_id: &connected.owner.connection_id,
        credential: &key,
    };
    let conversation = store
        .current_conversation(&actor, &f.agent.id, f.now)
        .await?
        .ok_or("Missing conversation")?;
    let input = store
        .accept_conversation_input(
            &actor,
            &AcceptConversationInput {
                agent_id: f.agent.id.clone(),
                conversation_id: Some(conversation.id.clone()),
                request_id: "authentication-lock-order".into(),
                text: ConversationText::parse("synthetic lock witness".into())?,
                target: ConversationTarget::Snapshot(
                    conversation
                        .source
                        .snapshot()
                        .ok_or("Missing source")?
                        .clone(),
                ),
            },
            f.now,
        )
        .await?;
    let application = draft(f, &input);
    let route = store
        .resolve_connector_route(&f.agent.computer.id)
        .await?
        .ok_or("Missing route")?;
    store
        .begin_conversation_input_preparation(&actor, &route, &application, f.now)
        .await?;
    let previous = store
        .execution_authentication_binding(&source, &application)
        .await?
        .ok_or("Missing application")?;
    let applied = store
        .begin_execution_authentication(&source, &application, Some(&previous.operation_id))
        .await?;
    let mut blocker = AsyncPgConnection::establish(url).await?;
    let pid: BackendPid = diesel_async::RunQueryDsl::get_result(
        diesel::sql_query("SELECT pg_backend_pid() AS pid"),
        &mut blocker,
    )
    .await?;
    blocker.batch_execute("BEGIN; SELECT x.id FROM external_authentications x JOIN execution_authentication_applications p ON p.authentication_id=x.id JOIN agents a ON a.id=p.agent_id WHERE a.handle='chat-fixture' AND p.is_current=TRUE FOR UPDATE OF x").await?;
    let worker_store = store.clone();
    let session = f.session.clone();
    let control = f.control.clone();
    let connection = connected.owner.connection_id;
    let worker = tokio::spawn(async move {
        worker_store
            .execution_authentication_material(
                &ConnectorSource {
                    session_id: &session,
                    control_id: &control,
                    connection_id: &connection,
                    credential: &key,
                },
                &Draft {
                    agent_id: &applied.agent_id,
                    conversation_id: &applied.conversation_id,
                    operation_id: &applied.operation_id,
                },
                None,
            )
            .await
            .map(|_| ())
    });
    let mut observer = AsyncPgConnection::establish(url).await?;
    let queued = tokio::time::timeout(std::time::Duration::from_secs(5),async {
        loop {
            let wait:LockWait = diesel_async::RunQueryDsl::get_result(diesel::sql_query(
                "SELECT EXISTS (SELECT 1 FROM pg_stat_activity WHERE $1=ANY(pg_blocking_pids(pid))) AS blocked")
                .bind::<diesel::sql_types::Integer,_>(pid.pid),&mut observer).await?;
            if wait.blocked {return Ok::<(),TestError>(());}
        }
    }).await;
    // The authoritative record-lock wait has begun. Computer must still be
    // available, otherwise a catalogue -> Provider -> Computer cycle is possible.
    let order = observer.batch_execute("BEGIN; SELECT c.id FROM computers c JOIN agents a ON a.computer_id=c.id WHERE a.handle='chat-fixture' FOR NO KEY UPDATE OF c NOWAIT").await;
    let observer_cleanup = observer.batch_execute("ROLLBACK").await;
    let release = blocker.batch_execute("ROLLBACK").await;
    let result = worker.await?;
    queued??;
    order?;
    observer_cleanup?;
    release?;
    result?;
    Ok(())
}

#[test]
fn sqlite_execution_authentication_preserves_applied_identity_and_denies_stale_delivery()
-> Result<(), TestError> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("applied.sqlite");
    let cipher = cipher()?;
    prepare_sqlite(&path, &cipher)?;
    let runtime = tokio::runtime::Runtime::new()?;
    let store = runtime.block_on(Store::connect_sqlite(path.clone(), cipher))?;
    let fixture = runtime.block_on(fixture(&store))?;
    runtime.block_on(exercise(&store, &fixture))?;
    let mut connection = diesel::SqliteConnection::establish(path.to_str().ok_or("Invalid path")?)?;
    isolate_pre_chat_migration!(&mut connection, SQLITE_MIGRATIONS);
    while connection
        .applied_migrations()?
        .iter()
        .any(|version| version.to_string().as_str() > "00000000000014")
    {
        connection.revert_last_migration(SQLITE_MIGRATIONS)?;
    }
    assert!(connection.revert_last_migration(SQLITE_MIGRATIONS).is_err());
    Ok(())
}
#[test]
fn postgres_execution_authentication_preserves_applied_identity_and_denies_stale_delivery()
-> Result<(), TestError> {
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
    let schema = format!("applied_{suffix}");
    control.batch_execute(&format!("CREATE SCHEMA {schema}"))?;
    let separator = if base.contains('?') { '&' } else { '?' };
    let url = format!("{base}{separator}options=-csearch_path%3D{schema}");
    let result = (|| -> Result<(), TestError> {
        let cipher = cipher()?;
        prepare_postgres(&url, &cipher)?;
        let runtime = tokio::runtime::Runtime::new()?;
        let store = runtime.block_on(Store::connect_postgres(url.clone(), cipher))?;
        let fixture = runtime.block_on(fixture(&store))?;
        runtime.block_on(exercise(&store, &fixture))?;
        runtime.block_on(record_lock_precedes_computer_admission(
            &store, &fixture, &url,
        ))?;
        let mut connection = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&url)?;
        isolate_pre_chat_migration!(&mut connection, POSTGRES_MIGRATIONS);
        while connection
            .applied_migrations()?
            .iter()
            .any(|version| version.to_string().as_str() > "00000000000014")
        {
            connection.revert_last_migration(POSTGRES_MIGRATIONS)?;
        }
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
