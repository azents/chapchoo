use super::*;
use chapchoo_domain::{InstructionSource, InstructionSourceKind, ProjectInstructions};
use chapchoo_persistence::{
    ConnectionStartupAdmission, ConversationInputPreparation, ConversationPreparationFailure,
    ConversationPreparationOutcome,
};
fn selection(owner: &str) -> Result<ProjectInstructions, TestError> {
    Ok(ProjectInstructions::new(vec![InstructionSource {
        kind: InstructionSourceKind::Owner,
        text: owner.into(),
    }])?)
}
async fn exercise(
    store: &Store,
    f: &Fixture,
    mut sql: impl FnMut(&str) -> Result<(), TestError>,
) -> Result<(), TestError> {
    let actor = f.owner.digest();
    let key = f.connector.digest();
    let source = f.source(&key);
    let route = store
        .resolve_connector_route(&f.agent.computer.id)
        .await?
        .ok_or("Missing route")?;
    let initial = store
        .admit_connection_startup(&f.agent.computer.id, &route)
        .await?
        .pop()
        .ok_or("Missing initial startup")?;
    let ConnectionStartupAdmission::Dispatch {
        conversation_id,
        operation_id,
        ..
    } = initial
    else {
        return Err("Expected initial dispatch".into());
    };
    let initial_selection = selection("")?;
    let catalog = br#"{"tools":[]}"#;
    let draft = ConversationInputPreparation {
        agent_id: &f.agent.id,
        conversation_id: &conversation_id,
        operation_id: &operation_id,
    };
    store
        .save_system_prompt(&actor, &f.agent.id, "newer than startup selection", f.now)
        .await?;
    assert!(matches!(
        store
            .seal_startup_project_instructions(&route, &draft, &initial_selection, catalog, f.now,)
            .await,
        Err(StoreError::InvalidTransition)
    ));
    store
        .save_system_prompt(&actor, &f.agent.id, "", f.now)
        .await?;
    assert!(store.reject_unsealed_preparation(&route, &draft).await?);
    assert!(!store.reject_unsealed_preparation(&route, &draft).await?);
    let Some(ConnectionStartupAdmission::Dispatch { operation_id, .. }) = store
        .admit_connection_startup(&f.agent.computer.id, &route)
        .await?
        .pop()
    else {
        return Err("Known before-native failure must obtain a new actual admission".into());
    };
    let draft = ConversationInputPreparation {
        agent_id: &f.agent.id,
        conversation_id: &conversation_id,
        operation_id: &operation_id,
    };
    store
        .seal_startup_project_instructions(&route, &draft, &initial_selection, catalog, f.now)
        .await?;
    assert!(
        !store.reject_unsealed_preparation(&route, &draft).await?,
        "A possibly lost seal acknowledgement is not proof of no native effect"
    );
    let target = chapchoo_persistence::ConversationDeliveryTarget {
        snapshot: snapshot("refresh-source")?,
        native_session: NativeConversationId::parse("refresh-native".into())?,
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
    store
        .complete_conversation_input_preparation(
            &source,
            &draft,
            &ConversationPreparationOutcome::SessionReady(target.clone()),
        )
        .await?;
    assert!(
        store
            .admit_project_refresh(
                &route,
                &f.agent.id,
                &conversation_id,
                &initial_selection,
                catalog,
                f.now + 1
            )
            .await?
            .is_none()
    );
    let same_render = ProjectInstructions::new(vec![
        InstructionSource {
            kind: InstructionSourceKind::Owner,
            text: "".into(),
        },
        InstructionSource {
            kind: InstructionSourceKind::Execution,
            text: "".into(),
        },
    ])?;
    assert_eq!(same_render.document(), initial_selection.document());
    let Some(ConnectionStartupAdmission::Dispatch {
        operation_id: structured_operation,
        ..
    }) = store
        .admit_project_refresh(
            &route,
            &f.agent.id,
            &conversation_id,
            &same_render,
            catalog,
            f.now + 1,
        )
        .await?
    else {
        return Err(
            "Different originals must admit refresh even when presentation is equal".into(),
        );
    };
    store
        .complete_conversation_input_preparation(
            &source,
            &ConversationInputPreparation {
                agent_id: &f.agent.id,
                conversation_id: &conversation_id,
                operation_id: &structured_operation,
            },
            &ConversationPreparationOutcome::SessionReady(target.clone()),
        )
        .await?;
    store
        .save_system_prompt(&actor, &f.agent.id, "latest", f.now + 1)
        .await?;
    assert!(
        store
            .admit_project_refresh(
                &route,
                &f.agent.id,
                &conversation_id,
                &initial_selection,
                catalog,
                f.now + 2
            )
            .await?
            .is_none(),
        "stale preselection cannot dispatch"
    );
    let selected = selection("latest")?;
    let (one, two) = tokio::join!(
        store.admit_project_refresh(
            &route,
            &f.agent.id,
            &conversation_id,
            &selected,
            catalog,
            f.now + 2
        ),
        store.admit_project_refresh(
            &route,
            &f.agent.id,
            &conversation_id,
            &selected,
            catalog,
            f.now + 2
        )
    );
    let mut admitted = vec![one?, two?].into_iter().flatten().collect::<Vec<_>>();
    assert_eq!(admitted.len(), 1);
    let Some(ConnectionStartupAdmission::Dispatch {
        operation_id: new_operation,
        fresh_native_allowed,
        require_retained_source,
        context,
        ..
    }) = admitted.pop()
    else {
        return Err("Expected refresh dispatch".into());
    };
    assert!(!fresh_native_allowed);
    assert!(require_retained_source);
    assert!(
        matches!(context,chapchoo_persistence::ConnectionStartupContext::Existing{snapshot,native_session} if snapshot==target.snapshot && native_session==target.native_session)
    );
    let refresh = ConversationInputPreparation {
        agent_id: &f.agent.id,
        conversation_id: &conversation_id,
        operation_id: &new_operation,
    };
    let input = store
        .accept_conversation_input(
            &actor,
            &AcceptConversationInput {
                agent_id: f.agent.id.clone(),
                conversation_id: Some(conversation_id.clone()),
                request_id: "refresh-blocked-input".into(),
                target: ConversationTarget::Snapshot(target.snapshot.clone()),
                text: ConversationText::parse("Retain this input during refresh".into())?,
            },
            f.now,
        )
        .await?;
    let forwarding = chapchoo_persistence::ConversationOperationForwarding {
        agent_id: &f.agent.id,
        conversation_id: &conversation_id,
        operation_id: &input.id,
        target: &target,
    };
    store
        .save_system_prompt(&actor, &f.agent.id, "", f.now)
        .await?;
    store
        .complete_conversation_input_preparation(
            &source,
            &draft,
            &ConversationPreparationOutcome::SessionReady(target.clone()),
        )
        .await?;
    assert!(
        matches!(
            store
                .project_instructions(&source, &f.agent.id, &conversation_id, &operation_id)
                .await,
            Err(StoreError::InvalidTransition)
        ),
        "Late historical completion cannot select its operation again"
    );
    assert!(
        matches!(
            store
                .begin_conversation_input_forwarding(
                    &actor,
                    &route,
                    &forwarding,
                    f.now,
                    &f.claim(store, forwarding.conversation_id, forwarding.operation_id)
                        .await?
                )
                .await,
            Err(StoreError::InvalidTransition)
        ),
        "Returning desired content to old applied content cannot bypass pending refresh"
    );
    store
        .save_system_prompt(&actor, &f.agent.id, "latest", f.now)
        .await?;
    assert_eq!(
        store
            .project_instructions(&source, &f.agent.id, &conversation_id, &new_operation)
            .await?
            .instructions,
        selected
    );
    store
        .complete_conversation_input_preparation(
            &source,
            &refresh,
            &ConversationPreparationOutcome::Failed(ConversationPreparationFailure::NativeRejected),
        )
        .await?;
    assert!(
        matches!(
            store
                .begin_conversation_input_forwarding(
                    &actor,
                    &route,
                    &forwarding,
                    f.now,
                    &f.claim(store, forwarding.conversation_id, forwarding.operation_id)
                        .await?
                )
                .await,
            Err(StoreError::InvalidTransition)
        ),
        "Failed startup cannot authorize a fresh text effect"
    );
    assert!(
        store
            .admit_project_refresh(
                &route,
                &f.agent.id,
                &conversation_id,
                &selected,
                catalog,
                f.now + 3
            )
            .await?
            .is_none(),
        "same failed selection is not retried forever"
    );
    let updated_catalog = br#"{"tools":[{"name":"new-tool"}]}"#;
    let stale = ConversationInputPreparation {
        agent_id: &f.agent.id,
        conversation_id: &conversation_id,
        operation_id: &input.id,
    };
    store
        .begin_conversation_input_preparation(
            &actor,
            &route,
            &stale,
            f.now,
            &f.claim(store, stale.conversation_id, stale.operation_id)
                .await?,
        )
        .await?;
    let before_launch =
        ConversationPreparationOutcome::Failed(ConversationPreparationFailure::BeforeNativeLaunch);
    assert!(store.reject_unsealed_preparation(&route, &stale).await?);
    assert!(!store.reject_unsealed_preparation(&route, &stale).await?);
    let recovery_input = store
        .accept_conversation_input(
            &actor,
            &AcceptConversationInput {
                agent_id: f.agent.id.clone(),
                conversation_id: Some(conversation_id.clone()),
                request_id: "current-input-recovery".into(),
                target: ConversationTarget::Snapshot(target.snapshot.clone()),
                text: ConversationText::parse("Recover with the exact selected operation".into())?,
            },
            f.now,
        )
        .await?;
    let recovery = ConversationInputPreparation {
        operation_id: &recovery_input.id,
        ..stale
    };
    store
        .begin_conversation_input_preparation(
            &actor,
            &route,
            &recovery,
            f.now,
            &f.claim(store, recovery.conversation_id, recovery.operation_id)
                .await?,
        )
        .await?;
    assert!(
        store
            .admit_project_refresh(
                &route,
                &f.agent.id,
                &conversation_id,
                &selected,
                updated_catalog,
                f.now,
            )
            .await?
            .is_none(),
        "Refresh joins an active input-owned preparation before its first seal"
    );
    assert!(
        matches!(
            store
                .seal_project_instructions(&actor, &route, &stale, &selected, catalog, f.now)
                .await,
            Err(StoreError::InvalidTransition)
        ),
        "An old unsealed operation cannot overwrite the actual newer admission"
    );
    let forwarding = chapchoo_persistence::ConversationOperationForwarding {
        operation_id: &recovery_input.id,
        ..forwarding
    };
    instructions_fixture::apply(store, f, &actor, &route, &source, &forwarding, f.now).await?;
    store
        .complete_conversation_input_preparation(&source, &stale, &before_launch)
        .await?;
    // Both operations were genuinely admitted/applied above. Equalize metadata
    // solely to prove authorization does not infer causality from wall-clock order.
    sql(
        "UPDATE conversation_startups SET admitted_at=1; UPDATE project_instruction_evidence SET created_at=1,applied_at=CASE WHEN applied_at IS NULL THEN NULL ELSE 1 END",
    )?;
    assert!(
        matches!(
            store
                .begin_conversation_input_forwarding(
                    &actor,
                    &route,
                    &forwarding,
                    f.now,
                    &f.claim(store, forwarding.conversation_id, forwarding.operation_id)
                        .await?
                )
                .await?,
            chapchoo_persistence::ConversationForwardingAdmission::Granted(_)
        ),
        "A later exact input-owned application can recover from an older failed startup"
    );
    let selected = selection("latest-after-forward")?;
    store
        .save_system_prompt(&actor, &f.agent.id, "latest-after-forward", f.now)
        .await?;
    assert!(
        store
            .admit_project_refresh(
                &route,
                &f.agent.id,
                &conversation_id,
                &selected,
                updated_catalog,
                f.now,
            )
            .await?
            .is_none(),
        "A granted input retains its configuration before any Connector body pull"
    );
    store
        .fail_conversation_input_attempt(&source, &forwarding)
        .await?;
    assert!(
        store
            .admit_project_refresh(
                &route,
                &f.agent.id,
                &conversation_id,
                &selected,
                updated_catalog,
                f.now,
            )
            .await?
            .is_none(),
        "AttemptError cannot release an unresolved possible native effect"
    );
    assert_eq!(
        store
            .conversation_input_delivery(
                &actor,
                &f.agent.id,
                &conversation_id,
                &recovery_input.id,
                f.now,
            )
            .await?,
        Some(chapchoo_persistence::ConversationInputDelivery::PossibleForwarding)
    );
    assert_eq!(
        store
            .complete_conversation_input_forwarding(
                &source,
                &forwarding,
                &chapchoo_persistence::ConversationInputOutcome::Inserted {
                    native_message_id: "picked-up-before-refresh".into(),
                },
            )
            .await?,
        chapchoo_persistence::ConversationInputReportAcknowledgement::Recorded
    );
    assert!(
        store
            .conversation_input_reference(
                &actor,
                &f.agent.id,
                &conversation_id,
                &recovery_input.request_id,
                f.now,
            )
            .await?
            .is_none()
    );
    let messages = store
        .conversation_messages(
            &actor,
            &chapchoo_persistence::ConversationMessageScope {
                agent_id: &f.agent.id,
                conversation_id: &conversation_id,
            },
            chapchoo_persistence::ConversationMessageWindow::Latest,
            25,
            f.now,
        )
        .await?;
    assert!(
        messages
            .messages
            .iter()
            .any(|message| message.receipt.source_input_id.as_deref()
                == Some(recovery_input.id.as_str())),
        "Known native pickup must commit its immutable Chat receipt before refresh admission"
    );
    let Some(ConnectionStartupAdmission::Dispatch {
        operation_id: catalog_operation,
        ..
    }) = store
        .admit_project_refresh(
            &route,
            &f.agent.id,
            &conversation_id,
            &selected,
            updated_catalog,
            f.now + 4,
        )
        .await?
    else {
        return Err("Changed catalogue must admit legitimate preparation".into());
    };
    assert!(
        store
            .admit_project_refresh(
                &route,
                &f.agent.id,
                &conversation_id,
                &selected,
                updated_catalog,
                f.now + 5
            )
            .await?
            .is_none()
    );
    store
        .complete_conversation_input_preparation(
            &source,
            &ConversationInputPreparation {
                operation_id: &catalog_operation,
                ..draft
            },
            &ConversationPreparationOutcome::SessionReady(target.clone()),
        )
        .await?;
    sql("UPDATE conversations SET current_preparation_operation_id=NULL")?;
    let null_input = store
        .accept_conversation_input(
            &actor,
            &AcceptConversationInput {
                agent_id: f.agent.id.clone(),
                conversation_id: Some(conversation_id.clone()),
                request_id: "null-operation-selection".into(),
                target: ConversationTarget::Snapshot(target.snapshot.clone()),
                text: ConversationText::parse("Require a new authentic selection".into())?,
            },
            f.now,
        )
        .await?;
    let null_forwarding = chapchoo_persistence::ConversationOperationForwarding {
        operation_id: &null_input.id,
        ..forwarding
    };
    assert!(
        matches!(
            store
                .begin_conversation_input_forwarding(
                    &actor,
                    &route,
                    &null_forwarding,
                    f.now,
                    &f.claim(
                        store,
                        null_forwarding.conversation_id,
                        null_forwarding.operation_id
                    )
                    .await?
                )
                .await,
            Err(StoreError::InvalidTransition)
        ),
        "A migrated null pointer never borrows historical READY evidence"
    );
    let Some(ConnectionStartupAdmission::Dispatch {
        operation_id: recovered_operation,
        ..
    }) = store
        .admit_connection_startup(&f.agent.computer.id, &route)
        .await?
        .pop()
    else {
        return Err("Null selection must obtain a new retained-source admission".into());
    };
    let recovered = ConversationInputPreparation {
        operation_id: &recovered_operation,
        ..draft
    };
    store
        .seal_startup_project_instructions(&route, &recovered, &selected, catalog, f.now)
        .await?;
    store
        .complete_conversation_input_preparation(
            &source,
            &recovered,
            &ConversationPreparationOutcome::SessionReady(target.clone()),
        )
        .await?;
    assert!(matches!(
        store
            .begin_conversation_input_forwarding(
                &actor,
                &route,
                &null_forwarding,
                f.now,
                &f.claim(
                    store,
                    null_forwarding.conversation_id,
                    null_forwarding.operation_id
                )
                .await?
            )
            .await?,
        chapchoo_persistence::ConversationForwardingAdmission::Granted(_)
    ));
    store
        .complete_conversation_input_forwarding(
            &source,
            &null_forwarding,
            &chapchoo_persistence::ConversationInputOutcome::Steered,
        )
        .await?;
    let race_input = store
        .accept_conversation_input(
            &actor,
            &AcceptConversationInput {
                agent_id: f.agent.id.clone(),
                conversation_id: Some(conversation_id.clone()),
                request_id: "forwarding-refresh-one-winner".into(),
                target: ConversationTarget::Snapshot(target.snapshot.clone()),
                text: ConversationText::parse(
                    "Serialize input forwarding with configuration refresh".into(),
                )?,
            },
            f.now,
        )
        .await?;
    let race_forwarding = chapchoo_persistence::ConversationOperationForwarding {
        operation_id: &race_input.id,
        ..forwarding
    };
    // Owner bytes stay equal so both admissions are individually valid before
    // the shared Team/Computer transaction lock decides the actual winner.
    let race_catalog = br#"{"tools":[{"name":"race-tool"}]}"#;
    let claim = f
        .claim(
            store,
            race_forwarding.conversation_id,
            race_forwarding.operation_id,
        )
        .await?;
    let (input_admission, refresh_admission) = tokio::join!(
        store.begin_conversation_input_forwarding(&actor, &route, &race_forwarding, f.now, &claim),
        store.admit_project_refresh(
            &route,
            &f.agent.id,
            &conversation_id,
            &selected,
            race_catalog,
            f.now,
        ),
    );
    let (refresh_operation, input_won) = match (input_admission, refresh_admission?) {
        (Ok(chapchoo_persistence::ConversationForwardingAdmission::Granted(_)), None) => {
            store
                .complete_conversation_input_forwarding(
                    &source,
                    &race_forwarding,
                    &chapchoo_persistence::ConversationInputOutcome::Steered,
                )
                .await?;
            let Some(ConnectionStartupAdmission::Dispatch { operation_id, .. }) = store
                .admit_project_refresh(
                    &route,
                    &f.agent.id,
                    &conversation_id,
                    &selected,
                    race_catalog,
                    f.now,
                )
                .await?
            else {
                return Err("Known input pickup must release the refresh admission fence".into());
            };
            (operation_id, true)
        }
        (
            Err(StoreError::InvalidTransition),
            Some(ConnectionStartupAdmission::Dispatch { operation_id, .. }),
        ) => {
            assert!(
                store
                    .conversation_input_delivery(
                        &actor,
                        &f.agent.id,
                        &conversation_id,
                        &race_input.id,
                        f.now,
                    )
                    .await?
                    .is_none(),
                "Refresh winner must prevent any input effect grant"
            );
            (operation_id, false)
        }
        other => {
            return Err(
                format!("Forwarding and refresh must have exactly one winner: {other:?}").into(),
            );
        }
    };
    store
        .complete_conversation_input_preparation(
            &source,
            &ConversationInputPreparation {
                operation_id: &refresh_operation,
                ..draft
            },
            &ConversationPreparationOutcome::SessionReady(target.clone()),
        )
        .await?;
    if !input_won {
        assert!(
            matches!(
                store
                    .begin_conversation_input_forwarding(
                        &actor,
                        &route,
                        &race_forwarding,
                        f.now,
                        &f.claim(
                            store,
                            race_forwarding.conversation_id,
                            race_forwarding.operation_id
                        )
                        .await?
                    )
                    .await?,
                chapchoo_persistence::ConversationForwardingAdmission::Granted(_)
            ),
            "The losing unforwarded input may use the successor only after its actual application"
        );
        store
            .complete_conversation_input_forwarding(
                &source,
                &race_forwarding,
                &chapchoo_persistence::ConversationInputOutcome::Steered,
            )
            .await?;
    }
    let current = store
        .current_conversation(&actor, &f.agent.id, f.now)
        .await?
        .ok_or("Missing source")?;
    assert_eq!(current.native_session_id, Some(target.native_session));
    assert!(matches!(current.source, ConversationSource::Ready { .. }));
    Ok(())
}
#[test]
fn sqlite_project_refresh_admission() -> Result<(), TestError> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("project-refresh.sqlite");
    let cipher = cipher()?;
    prepare_sqlite(&path, &cipher)?;
    let runtime = tokio::runtime::Runtime::new()?;
    let mut connection = diesel::SqliteConnection::establish(path.to_str().ok_or("Invalid path")?)?;
    let store = runtime.block_on(Store::connect_sqlite(path, cipher))?;
    let f = runtime.block_on(fixture(&store))?;
    runtime.block_on(exercise(&store, &f, |query| {
        connection.batch_execute(query)?;
        Ok(())
    }))
}
#[test]
fn postgres_project_refresh_admission() -> Result<(), TestError> {
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
    let schema = format!("refresh_{suffix}");
    control.batch_execute(&format!("CREATE SCHEMA {schema}"))?;
    let separator = if base.contains('?') { '&' } else { '?' };
    let url = format!("{base}{separator}options=-csearch_path%3D{schema}");
    let result = (|| -> Result<(), TestError> {
        let cipher = cipher()?;
        prepare_postgres(&url, &cipher)?;
        let runtime = tokio::runtime::Runtime::new()?;
        let mut connection = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&url)?;
        let store = runtime.block_on(Store::connect_postgres(url, cipher))?;
        let f = runtime.block_on(fixture(&store))?;
        runtime.block_on(exercise(&store, &f, |query| {
            tokio::task::block_in_place(|| connection.batch_execute(query))?;
            Ok(())
        }))
    })();
    control.batch_execute(&format!("DROP SCHEMA {schema} CASCADE"))?;
    result
}
