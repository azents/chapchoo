//! Queue-owned reads and native admission survive rotation, but not withdrawal.
use super::*;
use chapchoo_persistence::{
    ComputerRequest, ConversationDeliveryTarget, ConversationForwardingAdmission,
    ConversationInputAuthority, ConversationInputPreparation, ConversationOperationForwarding,
    ConversationPreparationAdmission, ConversationPreparationOutcome, ConversationPreparationState,
    QueuedConversationCommand, QueuedSetupContext,
};

async fn exercise(
    store: &Store,
    f: &Fixture,
    mut sql: impl FnMut(&str) -> Result<(), TestError>,
) -> Result<(), TestError> {
    let receipt = store
        .accept_conversation_input(
            &f.owner.digest(),
            &AcceptConversationInput {
                agent_id: f.agent.id.clone(),
                conversation_id: None,
                request_id: "rotation-bound-input".into(),
                target: ConversationTarget::NotStarted,
                text: ConversationText::parse("One original input through rotation".into())?,
            },
            f.now,
        )
        .await?;
    let claim = f
        .claim(store, &receipt.conversation_id, &receipt.id)
        .await?;
    let authority = ConversationInputAuthority {
        agent_id: f.agent.id.clone(),
        conversation_id: receipt.conversation_id.clone(),
        input_id: receipt.id.clone(),
        claim_id: claim,
    };
    let route = store
        .resolve_connector_route(&f.agent.computer.id)
        .await?
        .ok_or("Missing route")?;
    let scope = ComputerRequest {
        computer_id: &f.agent.computer.id,
        request_id: &f.agent.computer.current_request_id,
    };
    let snapshot = store
        .delivery_snapshot(Some(&f.owner.digest()), scope, None)
        .await?;
    let execution = snapshot
        .computer
        .current_execution_id
        .as_deref()
        .ok_or("Missing execution")?;
    // Exercise the actual account rotation operation with a supported short
    // positive overlap. A second rotation replaces the original previous hash.
    let policy = SessionPolicy {
        rotation_ms: 1,
        overlap_ms: 1,
        ..SessionPolicy::default()
    };
    let first = store.renew_session(&f.owner, f.now + 1, policy).await?;
    let second = store.renew_session(&first.token, f.now + 2, policy).await?;
    assert!(matches!(
        store.current_identity(&f.owner.digest(), now()?).await,
        Err(StoreError::Unauthenticated)
    ));
    assert!(store.queued_conversation_input_pending(&authority).await?);
    assert_eq!(
        store.queued_conversation_agent(&authority).await?.id,
        f.agent.id
    );
    assert_eq!(
        store
            .queued_current_conversation(&authority)
            .await?
            .ok_or("Missing conversation")?
            .id,
        receipt.conversation_id
    );
    assert!(store.queued_connection_startup(&authority).await?.is_none());
    assert_eq!(
        store.queued_system_prompt(&authority).await?,
        Some(String::new())
    );
    assert!(
        store
            .queued_latest_system_prompt(&authority)
            .await?
            .is_none()
    );
    let draft = ConversationInputPreparation {
        agent_id: &f.agent.id,
        conversation_id: &receipt.conversation_id,
        operation_id: &receipt.id,
    };
    let ConversationPreparationAdmission::Granted {
        requested_supply,
        fresh_native_allowed,
        require_retained_source,
        ..
    } = store
        .queued_begin_conversation_input_preparation(&authority, &route, &draft)
        .await?
    else {
        return Err("Expected one input-owned setup grant".into());
    };
    let command = QueuedConversationCommand::Setup {
        draft: ConversationInputPreparation { ..draft },
        supply: &requested_supply,
        context: QueuedSetupContext::Initial {
            allow_fresh: fresh_native_allowed,
            require_retained_source,
        },
    };
    assert!(
        matches!(
            store
                .queued_connector_delivery_snapshot(&authority, scope, execution, &command)
                .await,
            Err(StoreError::InvalidTransition)
        ),
        "A preparation without sealed instructions cannot dispatch"
    );
    let instructions =
        chapchoo_domain::ProjectInstructions::new(vec![chapchoo_domain::InstructionSource {
            kind: chapchoo_domain::InstructionSourceKind::Product,
            text: "Preserve user files.".into(),
        }])?;
    store
        .seal_queued_project_instructions(
            &authority,
            &route,
            &draft,
            &instructions,
            br#"{"tools":[]}"#,
        )
        .await?;
    let third = store
        .renew_session(&second.token, f.now + 3, policy)
        .await?;
    assert!(
        matches!(
            store
                .queued_begin_conversation_input_preparation(&authority, &route, &draft)
                .await?,
            ConversationPreparationAdmission::Existing(_)
        ),
        "Rotation never grants another setup"
    );
    assert_eq!(
        store
            .queued_conversation_input_preparation(&authority, &draft)
            .await?
            .ok_or("Missing setup")?
            .state,
        ConversationPreparationState::PossibleForwarding
    );
    assert_eq!(
        store
            .queued_latest_system_prompt(&authority)
            .await?
            .ok_or("Missing seal")?
            .instructions,
        instructions
    );
    store
        .queued_connector_delivery_snapshot(&authority, scope, execution, &command)
        .await?;
    let mut wrong = authority.clone();
    wrong.claim_id = receipt.id.clone();
    assert!(
        store
            .queued_connector_delivery_snapshot(&wrong, scope, execution, &command)
            .await
            .is_err()
    );
    wrong = authority.clone();
    wrong.agent_id = f.other_agent.clone();
    assert!(
        store
            .queued_connector_delivery_snapshot(&wrong, scope, execution, &command)
            .await
            .is_err()
    );
    assert!(
        store
            .queued_connector_delivery_snapshot(
                &authority,
                ComputerRequest {
                    request_id: "different-request",
                    ..scope
                },
                execution,
                &command
            )
            .await
            .is_err()
    );
    assert!(
        store
            .queued_connector_delivery_snapshot(&authority, scope, &receipt.id, &command)
            .await
            .is_err()
    );
    let other_input = store
        .accept_conversation_input(
            &third.token.digest(),
            &AcceptConversationInput {
                agent_id: f.agent.id.clone(),
                conversation_id: Some(receipt.conversation_id.clone()),
                request_id: "rotation-other-input".into(),
                target: ConversationTarget::NotStarted,
                text: ConversationText::parse("A distinct queued input".into())?,
            },
            now()?,
        )
        .await?;
    let unrelated = QueuedConversationCommand::Setup {
        draft: ConversationInputPreparation {
            operation_id: &other_input.id,
            ..draft
        },
        supply: &requested_supply,
        context: QueuedSetupContext::Initial {
            allow_fresh: fresh_native_allowed,
            require_retained_source,
        },
    };
    assert!(
        matches!(
            store
                .queued_connector_delivery_snapshot(&authority, scope, execution, &unrelated)
                .await,
            Err(StoreError::Forbidden)
        ),
        "A distinct issued operation cannot borrow this input's claim"
    );
    let key = f.connector.digest();
    let source = f.source(&key);
    let target = ConversationDeliveryTarget {
        snapshot: super::snapshot("rotation-source")?,
        native_session: NativeConversationId::parse("rotation-native".into())?,
    };
    store
        .complete_conversation_input_preparation(
            &source,
            &draft,
            &ConversationPreparationOutcome::SessionReady(target.clone()),
        )
        .await?;
    assert!(
        store
            .queued_connector_delivery_snapshot(&authority, scope, execution, &command)
            .await
            .is_err(),
        "A known setup result is not permission to invoke again"
    );
    store
        .bind_conversation_source(
            &source,
            &SourceBinding {
                conversation_id: receipt.conversation_id.clone(),
                expected_previous: None,
                snapshot: target.snapshot.clone(),
                native_session: target.native_session.clone(),
            },
        )
        .await?;
    store
        .complete_conversation_source(&source, &receipt.conversation_id, &target.snapshot, None)
        .await?;
    let forwarding = ConversationOperationForwarding {
        agent_id: &f.agent.id,
        conversation_id: &receipt.conversation_id,
        operation_id: &receipt.id,
        target: &target,
    };
    assert!(matches!(
        store
            .queued_begin_conversation_input_forwarding(&authority, &route, &forwarding)
            .await?,
        ConversationForwardingAdmission::Granted(_)
    ));
    let _fourth = store.renew_session(&third.token, f.now + 4, policy).await?;
    assert!(store.queued_conversation_input_pending(&authority).await?);
    assert!(matches!(
        store
            .queued_begin_conversation_input_forwarding(&authority, &route, &forwarding)
            .await?,
        ConversationForwardingAdmission::Existing(_)
    ));
    let input = QueuedConversationCommand::Input(ConversationOperationForwarding { ..forwarding });
    store
        .queued_connector_delivery_snapshot(&authority, scope, execution, &input)
        .await?;
    sql(
        "UPDATE auth_sessions SET created_at=0,expires_at=1 WHERE id=(SELECT submitting_session_id FROM conversation_inputs WHERE request_id='rotation-bound-input')",
    )?;
    assert!(matches!(
        store.queued_conversation_input_pending(&authority).await,
        Err(StoreError::Unauthenticated)
    ));
    assert!(matches!(
        store
            .queued_connector_delivery_snapshot(&authority, scope, execution, &input)
            .await,
        Err(StoreError::Unauthenticated)
    ));
    sql(&format!(
        "UPDATE auth_sessions SET expires_at={},revoked_at=1 WHERE id=(SELECT submitting_session_id FROM conversation_inputs WHERE request_id='rotation-bound-input')",
        now()? + 60_000
    ))?;
    assert!(matches!(
        store.queued_system_prompt(&authority).await,
        Err(StoreError::Unauthenticated)
    ));
    // Withdrawal stops queued admission, not an independently authenticated late
    // native success. Consumption must end polling even though the binding is gone.
    store
        .complete_conversation_input_forwarding(
            &source,
            &forwarding,
            &chapchoo_persistence::ConversationInputOutcome::Steered,
        )
        .await?;
    assert!(!store.queued_conversation_input_pending(&authority).await?);
    Ok(())
}

#[test]
fn sqlite_rotation_bound_processing_and_peer() -> Result<(), TestError> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("rotation.sqlite");
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
fn postgres_rotation_bound_processing_and_peer() -> Result<(), TestError> {
    let _lock = POSTGRES_CONVERSATION_LOCK
        .lock()
        .map_err(|_| "Fixture lock poisoned")?;
    let base = std::env::var("CHAPCHOO_TEST_POSTGRES_URL")
        .map_err(|_| "Disposable PostgreSQL is required")?;
    let mut control = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&base)?;
    let schema = format!("rotation_{}", std::process::id());
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
