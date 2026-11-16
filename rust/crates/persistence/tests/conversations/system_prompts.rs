use super::*;
use chapchoo_domain::{InstructionSource, InstructionSourceKind, ProjectInstructions};
use chapchoo_persistence::{ConversationInputPreparation, ConversationPreparationAdmission};

async fn exercise(store: &Store, f: &Fixture) -> Result<(), TestError> {
    let actor = f.owner.digest();
    assert_eq!(store.system_prompt(&actor, &f.agent.id, f.now).await?, "");
    assert!(matches!(
        store
            .system_prompt(&f.other.digest(), &f.agent.id, f.now)
            .await,
        Err(StoreError::NotFound)
    ));
    let original = "  Exact owner 🙂\n\n";
    assert_eq!(
        store
            .save_system_prompt(&actor, &f.agent.id, original, f.now)
            .await?,
        original
    );
    assert_eq!(
        store.system_prompt(&actor, &f.agent.id, f.now).await?,
        original
    );
    assert!(matches!(
        store
            .save_system_prompt(&f.other.digest(), &f.agent.id, "forbidden", f.now)
            .await,
        Err(StoreError::NotFound)
    ));
    assert!(matches!(
        store
            .save_system_prompt(&actor, &f.agent.id, &"a".repeat(16 * 1024 + 1), f.now)
            .await,
        Err(StoreError::InvalidInput)
    ));
    assert!(matches!(
        store
            .save_system_prompt(&actor, &f.agent.id, "nul\0byte", f.now)
            .await,
        Err(StoreError::InvalidInput)
    ));
    let accepted = store
        .accept_conversation_input(
            &actor,
            &AcceptConversationInput {
                agent_id: f.agent.id.clone(),
                conversation_id: None,
                request_id: "instruction-evidence".into(),
                target: ConversationTarget::NotStarted,
                text: ConversationText::parse("ordinary user input".into())?,
            },
            f.now,
        )
        .await?;
    let draft = ConversationInputPreparation {
        agent_id: &f.agent.id,
        conversation_id: &accepted.conversation_id,
        operation_id: &accepted.id,
    };
    let route = store
        .resolve_connector_route(&f.agent.computer.id)
        .await?
        .ok_or("Missing route")?;
    assert!(matches!(
        store
            .begin_conversation_input_preparation(&actor, &route, &draft, f.now)
            .await?,
        ConversationPreparationAdmission::Granted { .. }
    ));
    let selected = ProjectInstructions::new(vec![
        InstructionSource {
            kind: InstructionSourceKind::Product,
            text: "Product unchanged".into(),
        },
        InstructionSource {
            kind: InstructionSourceKind::Owner,
            text: original.into(),
        },
    ])?;
    let catalog = br#"{"tools":[]}"#;
    store
        .save_system_prompt(&actor, &f.agent.id, "saved after selection", f.now)
        .await?;
    assert!(
        matches!(
            store
                .seal_project_instructions(&actor, &route, &draft, &selected, catalog, f.now)
                .await,
            Err(StoreError::InvalidTransition)
        ),
        "First seal must revalidate the selected owner bytes under save serialization"
    );
    assert!(
        matches!(
            store
                .instruction_evidence(
                    &actor,
                    &f.agent.id,
                    &accepted.conversation_id,
                    &accepted.id,
                    f.now,
                )
                .await,
            Err(StoreError::NotFound)
        ),
        "Rejected first seal does not fabricate immutable or applied evidence"
    );
    store
        .save_system_prompt(&actor, &f.agent.id, original, f.now)
        .await?;
    let evidence = store
        .seal_project_instructions(&actor, &route, &draft, &selected, catalog, f.now)
        .await?;
    assert!(
        !store.reject_unsealed_preparation(&route, &draft).await?,
        "Durable input seal prevents no-effect rejection before native reporting"
    );
    assert_eq!(evidence.instructions, selected);
    assert_eq!(evidence.rendered_document, selected.document());
    assert_eq!(
        store
            .seal_project_instructions(&actor, &route, &draft, &selected, catalog, f.now + 1)
            .await?,
        evidence
    );
    let changed = ProjectInstructions::new(vec![InstructionSource {
        kind: InstructionSourceKind::Owner,
        text: "changed".into(),
    }])?;
    assert!(matches!(
        store
            .seal_project_instructions(&actor, &route, &draft, &changed, catalog, f.now)
            .await,
        Err(StoreError::Conflict)
    ));
    assert!(matches!(
        store
            .seal_project_instructions(
                &actor,
                &route,
                &draft,
                &selected,
                br#"{"tools":[{"name":"different"}]}"#,
                f.now
            )
            .await,
        Err(StoreError::Conflict)
    ));
    assert_eq!(
        store
            .project_instructions(
                &f.source(&f.connector.digest()),
                &f.agent.id,
                &accepted.conversation_id,
                &accepted.id
            )
            .await?,
        evidence
    );
    let key = f.connector.digest();
    let source = f.source(&key);
    let target = chapchoo_persistence::ConversationDeliveryTarget {
        snapshot: snapshot("instruction-source")?,
        native_session: NativeConversationId::parse("instruction-native".into())?,
    };
    store
        .bind_conversation_source(
            &source,
            &SourceBinding {
                conversation_id: accepted.conversation_id.clone(),
                expected_previous: None,
                snapshot: target.snapshot.clone(),
                native_session: target.native_session.clone(),
            },
        )
        .await?;
    store
        .complete_conversation_source(&source, &accepted.conversation_id, &target.snapshot, None)
        .await?;
    let forwarding = chapchoo_persistence::ConversationOperationForwarding {
        agent_id: &f.agent.id,
        conversation_id: &accepted.conversation_id,
        operation_id: &accepted.id,
        target: &target,
    };
    assert!(
        matches!(
            store
                .begin_conversation_input_forwarding(&actor, &route, &forwarding, f.now)
                .await,
            Err(StoreError::InvalidTransition)
        ),
        "Prepared originals are not applied evidence"
    );
    store
        .complete_conversation_input_preparation(
            &source,
            &draft,
            &chapchoo_persistence::ConversationPreparationOutcome::SessionReady(target.clone()),
        )
        .await?;
    store
        .save_system_prompt(&actor, &f.agent.id, "new desired", f.now)
        .await?;
    assert!(
        matches!(
            store
                .begin_conversation_input_forwarding(&actor, &route, &forwarding, f.now)
                .await,
            Err(StoreError::InvalidTransition)
        ),
        "An already applied older selection cannot authorize a new effect after save"
    );
    store
        .save_system_prompt(&actor, &f.agent.id, original, f.now)
        .await?;
    store
        .begin_conversation_input_forwarding(&actor, &route, &forwarding, f.now)
        .await?;
    store
        .save_system_prompt(&actor, &f.agent.id, "saved after effect grant", f.now)
        .await?;
    assert!(
        matches!(
            store
                .begin_conversation_input_forwarding(&actor, &route, &forwarding, f.now)
                .await?,
            chapchoo_persistence::ConversationForwardingAdmission::Existing(
                chapchoo_persistence::ConversationInputDelivery::PossibleForwarding
            )
        ),
        "Later settings never erase an existing possible effect or grant its replay"
    );
    store
        .complete_conversation_input_forwarding(
            &source,
            &forwarding,
            &chapchoo_persistence::ConversationInputOutcome::Inserted {
                native_message_id: "native-input".into(),
            },
        )
        .await?;
    assert!(
        store
            .conversation_input(
                &actor,
                &f.agent.id,
                &accepted.conversation_id,
                "instruction-evidence",
                f.now
            )
            .await?
            .is_none()
    );
    assert!(matches!(
        store
            .project_instructions(
                &source,
                &f.agent.id,
                &accepted.conversation_id,
                &accepted.id
            )
            .await,
        Err(StoreError::InvalidTransition)
    ));
    store
        .save_system_prompt(&actor, &f.agent.id, "newer", f.now + 1)
        .await?;
    store
        .save_system_prompt(&actor, &f.agent.id, "", f.now + 2)
        .await?;
    assert_eq!(
        store.system_prompt(&actor, &f.agent.id, f.now + 2).await?,
        ""
    );
    let retained = store
        .instruction_evidence(
            &actor,
            &f.agent.id,
            &accepted.conversation_id,
            &accepted.id,
            f.now + 2,
        )
        .await?;
    assert_eq!(retained.instructions, evidence.instructions);
    assert_eq!(retained.rendered_document, evidence.rendered_document);
    assert_eq!(retained.tool_catalog, evidence.tool_catalog);
    assert_eq!(retained.execution_id, evidence.execution_id);
    assert!(matches!(
        store
            .instruction_evidence(
                &f.other.digest(),
                &f.agent.id,
                &accepted.conversation_id,
                &accepted.id,
                f.now
            )
            .await,
        Err(StoreError::NotFound)
    ));
    assert_eq!(
        store
            .instruction_history(&actor, &f.agent.id, &accepted.conversation_id, None, f.now)
            .await?
            .items,
        vec![retained]
    );
    // Equal timestamps exercise the operation-identity tie breaker across pages.
    let mut page_selection = selected.clone();
    for source in &mut page_selection.sources {
        if source.kind == InstructionSourceKind::Owner {
            source.text.clear();
        }
    }
    for index in 0..101 {
        let input = store
            .accept_conversation_input(
                &actor,
                &AcceptConversationInput {
                    agent_id: f.agent.id.clone(),
                    conversation_id: Some(accepted.conversation_id.clone()),
                    request_id: format!("instruction-page-{index}"),
                    target: ConversationTarget::Snapshot(target.snapshot.clone()),
                    text: ConversationText::parse(format!("page fixture {index}"))?,
                },
                f.now + 4,
            )
            .await?;
        let next = ConversationInputPreparation {
            agent_id: &f.agent.id,
            conversation_id: &accepted.conversation_id,
            operation_id: &input.id,
        };
        store
            .begin_conversation_input_preparation(&actor, &route, &next, f.now + 4)
            .await?;
        store
            .seal_project_instructions(&actor, &route, &next, &page_selection, catalog, f.now + 4)
            .await?;
        store
            .complete_conversation_input_preparation(
                &source,
                &next,
                &chapchoo_persistence::ConversationPreparationOutcome::Failed(
                    chapchoo_persistence::ConversationPreparationFailure::BeforeNativeLaunch,
                ),
            )
            .await?;
    }
    let first_page = store
        .instruction_history(
            &actor,
            &f.agent.id,
            &accepted.conversation_id,
            None,
            f.now + 4,
        )
        .await?;
    assert_eq!(first_page.items.len(), 100);
    let before = first_page
        .next_before
        .as_deref()
        .ok_or("Expected history continuation")?;
    assert_eq!(
        Some(before),
        first_page
            .items
            .last()
            .map(|item| item.operation_id.as_str())
    );
    let second_page = store
        .instruction_history(
            &actor,
            &f.agent.id,
            &accepted.conversation_id,
            Some(before),
            f.now + 4,
        )
        .await?;
    assert_eq!(second_page.items.len(), 2);
    assert!(second_page.next_before.is_none());
    let identities = first_page
        .items
        .iter()
        .chain(second_page.items.iter())
        .map(|item| item.operation_id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(identities.len(), 102);
    assert!(identities.contains(accepted.id.as_str()));
    assert!(matches!(
        store
            .instruction_history(
                &f.other.digest(),
                &f.agent.id,
                &accepted.conversation_id,
                Some(before),
                f.now + 4
            )
            .await,
        Err(StoreError::NotFound)
    ));
    exercise_forwarding_save_race(store, f, &accepted.conversation_id, &target).await?;
    Ok(())
}

async fn exercise_forwarding_save_race(
    store: &Store,
    f: &Fixture,
    conversation: &str,
    target: &chapchoo_persistence::ConversationDeliveryTarget,
) -> Result<(), TestError> {
    let actor = f.owner.digest();
    let key = f.connector.digest();
    let route = store
        .resolve_connector_route(&f.agent.computer.id)
        .await?
        .ok_or("Missing race route")?;
    let input = store
        .accept_conversation_input(
            &actor,
            &AcceptConversationInput {
                agent_id: f.agent.id.clone(),
                conversation_id: Some(conversation.into()),
                request_id: "save-forward-race".into(),
                target: ConversationTarget::Snapshot(target.snapshot.clone()),
                text: ConversationText::parse("Keep input through settings save".into())?,
            },
            f.now,
        )
        .await?;
    let forwarding = chapchoo_persistence::ConversationOperationForwarding {
        agent_id: &f.agent.id,
        conversation_id: conversation,
        operation_id: &input.id,
        target,
    };
    instructions_fixture::apply(store, &actor, &route, &f.source(&key), &forwarding, f.now).await?;
    let (saved, admission) = tokio::join!(
        store.save_system_prompt(&actor, &f.agent.id, "racing save", f.now),
        store.begin_conversation_input_forwarding(&actor, &route, &forwarding, f.now),
    );
    saved?;
    assert_eq!(
        store.system_prompt(&actor, &f.agent.id, f.now).await?,
        "racing save"
    );
    match admission {
        Ok(chapchoo_persistence::ConversationForwardingAdmission::Granted(_)) => {
            assert!(matches!(
                store
                    .begin_conversation_input_forwarding(&actor, &route, &forwarding, f.now)
                    .await?,
                chapchoo_persistence::ConversationForwardingAdmission::Existing(
                    chapchoo_persistence::ConversationInputDelivery::PossibleForwarding
                )
            ));
        }
        Err(StoreError::InvalidTransition) => {
            assert!(
                store
                    .conversation_input_delivery(
                        &actor,
                        &f.agent.id,
                        conversation,
                        &input.id,
                        f.now,
                    )
                    .await?
                    .is_none()
            );
            assert!(matches!(
                store
                    .begin_conversation_input_forwarding(&actor, &route, &forwarding, f.now)
                    .await,
                Err(StoreError::InvalidTransition)
            ));
        }
        other => {
            return Err(format!("Unexpected save/forward serialization result: {other:?}").into());
        }
    }
    assert!(
        store
            .conversation_input_reference(
                &actor,
                &f.agent.id,
                conversation,
                &input.request_id,
                f.now,
            )
            .await?
            .is_some(),
        "Neither save nor possible-forwarding consumes the pending original"
    );
    store
        .save_system_prompt(&actor, &f.agent.id, "", f.now)
        .await?;
    Ok(())
}

#[test]
fn sqlite_saved_prompts_and_retained_sources() -> Result<(), TestError> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("instructions.sqlite");
    let cipher = cipher()?;
    prepare_sqlite(&path, &cipher)?;
    let runtime = tokio::runtime::Runtime::new()?;
    let store = runtime.block_on(Store::connect_sqlite(path, cipher))?;
    let f = runtime.block_on(fixture(&store))?;
    runtime.block_on(exercise(&store, &f))
}
#[test]
fn postgres_saved_prompts_and_retained_sources() -> Result<(), TestError> {
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
    let schema = format!("instructions_{suffix}");
    control.batch_execute(&format!("CREATE SCHEMA {schema}"))?;
    let separator = if base.contains('?') { '&' } else { '?' };
    let url = format!("{base}{separator}options=-csearch_path%3D{schema}");
    let result = (|| -> Result<(), TestError> {
        let cipher = cipher()?;
        prepare_postgres(&url, &cipher)?;
        let runtime = tokio::runtime::Runtime::new()?;
        let store = runtime.block_on(Store::connect_postgres(url, cipher))?;
        let f = runtime.block_on(fixture(&store))?;
        runtime.block_on(exercise(&store, &f))
    })();
    control.batch_execute(&format!("DROP SCHEMA {schema} CASCADE"))?;
    result
}
