use super::*;
use chapchoo_domain::LaunchCapture;
use chapchoo_persistence::{
    ConversationDeliveryTarget, ConversationInputOutcome, ConversationInputPreparation,
    ConversationModelChoice, ConversationModelObservation, ConversationModelSave,
    ConversationModelStart, ConversationModelStartAdmission, ConversationModelStartOutcome,
    ConversationModelTarget, ConversationOperationForwarding,
};

async fn exercise(store: &Store, f: &Fixture) -> Result<(), TestError> {
    let owner = f.owner.digest();
    let credential = f.connector.digest();
    let source = f.source(&credential);
    let initial = store
        .accept_conversation_input(
            &owner,
            &AcceptConversationInput {
                agent_id: f.agent.id.clone(),
                conversation_id: None,
                request_id: "model-initial-input".into(),
                target: ConversationTarget::NotStarted,
                text: ConversationText::parse("Synthetic model fixture".into())?,
            },
            f.now,
        )
        .await?;
    let id = initial.conversation_id.as_str();
    let snapshot = snapshot("model-fixture-capture")?;
    let native_session = NativeConversationId::parse("model-fixture-session".into())?;
    store
        .bind_conversation_source(
            &source,
            &SourceBinding {
                conversation_id: id.into(),
                expected_previous: None,
                snapshot: snapshot.clone(),
                native_session: native_session.clone(),
            },
        )
        .await?;
    store
        .complete_conversation_source(&source, id, &snapshot, None)
        .await?;
    let route = store
        .resolve_connector_route(&f.agent.computer.id)
        .await?
        .ok_or("Missing route")?;
    let setup = ConversationInputPreparation {
        agent_id: &f.agent.id,
        conversation_id: id,
        operation_id: &initial.id,
    };
    let claim = f.claim(store, id, &initial.id).await?;
    store
        .begin_conversation_input_preparation(&owner, &route, &setup, f.now, &claim)
        .await?;
    let applied = store
        .begin_execution_authentication(
            &source,
            &setup,
            None,
            Some(&LaunchCapture {
                snapshot: snapshot.clone(),
                through: None,
            }),
        )
        .await?;
    let material = store
        .execution_authentication_material(&source, &setup, None)
        .await?;
    let target = ConversationModelTarget {
        conversation_id: id.into(),
        snapshot: snapshot.clone(),
        native_session: native_session.clone(),
        launch_operation_id: applied.operation_id,
    };
    let first = ConversationModelChoice {
        id: "fixture-a".into(),
        label: "Fixture A".into(),
        capabilities: None,
    };
    let second = ConversationModelChoice {
        id: "fixture-b".into(),
        label: "Fixture B".into(),
        capabilities: Some(chapchoo_domain::ConversationModelCapabilities {
            efforts: vec!["medium".into()],
            speeds: vec![chapchoo_domain::ConversationSpeed::Standard],
        }),
    };
    let mode = chapchoo_domain::ConversationPermissionMode::ApproveForMe;
    store
        .observe_conversation_model(
            &source,
            &ConversationModelObservation {
                target: target.clone(),
                order: 0,
                current: None,
                options: Some(Vec::new()),
                api_key_source: Some(*material.source.as_bytes()),
                permission_modes: vec![mode],
            },
        )
        .await?;
    let permission = ConversationModelSave {
        target: target.clone(),
        request_id: "save-permission-without-models".into(),
        expected_request_id: None,
        model_id: None,
        tuning: chapchoo_domain::ConversationModelTuning {
            permission_mode: Some(mode),
            ..Default::default()
        },
    };
    let permission_receipt = store
        .save_conversation_model(&owner, &f.agent.id, &permission, f.now)
        .await?;
    assert!(permission_receipt.model.is_none());
    assert_eq!(permission_receipt.tuning, permission.tuning);
    assert!(
        store
            .conversation_model_save_result(&owner, &f.agent.id, &permission.request_id, f.now)
            .await?
            .is_some_and(|receipt| receipt.model.is_none() && receipt.tuning == permission.tuning)
    );
    store
        .observe_conversation_model(
            &source,
            &ConversationModelObservation {
                target: target.clone(),
                order: 1,
                current: Some(first.clone()),
                options: Some(vec![first, second.clone()]),
                api_key_source: Some(*material.source.as_bytes()),
                permission_modes: vec![mode],
            },
        )
        .await?;
    let saved = ConversationModelSave {
        target: target.clone(),
        request_id: "model-owner-save-b".into(),
        expected_request_id: Some(permission.request_id.clone()),
        model_id: Some(second.id.clone()),
        tuning: chapchoo_domain::ConversationModelTuning {
            effort: Some("medium".into()),
            speed: Some(chapchoo_domain::ConversationSpeed::Standard),
            permission_mode: None,
        },
    };
    assert!(
        store
            .save_conversation_model(&f.other.digest(), &f.agent.id, &saved, f.now)
            .await
            .is_err()
    );
    let receipt = store
        .save_conversation_model(&owner, &f.agent.id, &saved, f.now)
        .await?;
    assert_eq!(receipt.model.ok_or("Missing saved model")?.id, second.id);
    assert_eq!(receipt.tuning, saved.tuning);
    let active_source = ConversationDeliveryTarget {
        snapshot,
        native_session,
    };
    let draft = ConversationOperationForwarding {
        agent_id: &f.agent.id,
        conversation_id: id,
        operation_id: &initial.id,
        target: &active_source,
    };
    instructions_fixture::apply(store, f, &owner, &route, &source, &draft, f.now).await?;
    store
        .begin_conversation_input_forwarding(&owner, &route, &draft, f.now, &claim)
        .await?;
    assert!(matches!(
        store.connector_input_chunk(&source, &draft, 0).await,
        Err(StoreError::InvalidTransition)
    ));
    assert!(
        !store
            .connector_input_chunk_guarded(&source, &draft, 0, true)
            .await?
            .bytes
            .is_empty()
    );
    let start = ConversationModelStart {
        target: target.clone(),
        operation_id: initial.id.clone(),
        capable: true,
        permission_mode_capable: true,
    };
    let old_connector = ConversationModelStart {
        target: target.clone(),
        operation_id: initial.id.clone(),
        capable: true,
        permission_mode_capable: false,
    };
    assert!(matches!(
        store
            .reserve_conversation_model_start(&source, &old_connector)
            .await,
        Err(StoreError::InvalidTransition)
    ));
    assert!(matches!(
        store.reserve_conversation_model_start(&source, &start).await?,
        ConversationModelStartAdmission::Reserved { model_id: Some(id), tuning }
            if id == second.id && tuning.effort == saved.tuning.effort
                && tuning.speed == saved.tuning.speed && tuning.permission_mode == Some(mode)
    ));
    let next = ConversationModelSave {
        target: target.clone(),
        request_id: "model-owner-save-a".into(),
        expected_request_id: Some(saved.request_id.clone()),
        model_id: Some("fixture-a".into()),
        tuning: Default::default(),
    };
    assert!(matches!(
        store
            .save_conversation_model(&owner, &f.agent.id, &next, f.now)
            .await,
        Err(StoreError::InvalidTransition)
    ));
    assert!(
        store
            .conversation_model_save_result(&owner, &f.agent.id, &next.request_id, f.now)
            .await?
            .is_none()
    );
    let confirmed = ConversationModelStartOutcome::Inserted(NativeConversationId::parse(
        "model-native-message".into(),
    )?);
    store
        .settle_conversation_model_start(&source, &start, &confirmed)
        .await?;
    store
        .settle_conversation_model_start(&source, &start, &confirmed)
        .await?;
    assert!(
        store
            .reserve_conversation_model_start(&source, &start)
            .await
            .is_err()
    );
    assert_eq!(
        store
            .save_conversation_model(&owner, &f.agent.id, &next, f.now)
            .await?
            .model
            .ok_or("Missing saved model")?
            .id,
        "fixture-a"
    );
    let repeated = store
        .save_conversation_model(&owner, &f.agent.id, &saved, f.now)
        .await?;
    assert_eq!(
        repeated.model.ok_or("Missing original model")?.id,
        second.id
    );
    assert_eq!(repeated.tuning, saved.tuning);
    assert_eq!(
        store
            .conversation_model(&owner, &f.agent.id, f.now)
            .await?
            .ok_or("Missing model projection")?
            .saved_request_id
            .as_deref(),
        Some(next.request_id.as_str()),
    );
    store
        .complete_conversation_input_forwarding(
            &source,
            &draft,
            &ConversationInputOutcome::Inserted {
                native_message_id: "model-native-message".into(),
            },
        )
        .await?;
    for (request, outcome, visible) in [
        (
            "model-no-publication",
            ConversationModelStartOutcome::NotPublished,
            false,
        ),
        (
            "model-dual-evidence",
            ConversationModelStartOutcome::ModelUnavailableBeforeStart("fixture-a".into()),
            true,
        ),
    ] {
        let input = store
            .accept_conversation_input(
                &owner,
                &AcceptConversationInput {
                    agent_id: f.agent.id.clone(),
                    conversation_id: Some(id.into()),
                    request_id: request.into(),
                    target: ConversationTarget::Snapshot(active_source.snapshot.clone()),
                    text: ConversationText::parse("Synthetic governed input".into())?,
                },
                f.now,
            )
            .await?;
        let draft = ConversationOperationForwarding {
            agent_id: &f.agent.id,
            conversation_id: id,
            operation_id: &input.id,
            target: &active_source,
        };
        let claim = f.claim(store, id, &input.id).await?;
        store
            .begin_conversation_input_forwarding(&owner, &route, &draft, f.now, &claim)
            .await?;
        let start = ConversationModelStart {
            target: target.clone(),
            operation_id: input.id.clone(),
            capable: true,
            permission_mode_capable: true,
        };
        assert!(matches!(
            store.reserve_conversation_model_start(&source, &start).await?,
            ConversationModelStartAdmission::Reserved { model_id: Some(value), tuning }
                if value == "fixture-a" && !tuning.has_model_tuning() && tuning.permission_mode == Some(mode)
        ));
        assert!(matches!(
            store
                .settle_conversation_model_start(
                    &source,
                    &start,
                    &ConversationModelStartOutcome::ModelUnavailableBeforeStart(
                        "wrong-model".into()
                    )
                )
                .await,
            Err(StoreError::Conflict)
        ));
        store
            .settle_conversation_model_start(&source, &start, &outcome)
            .await?;
        let view = store
            .conversation_model(&owner, &f.agent.id, f.now)
            .await?
            .ok_or("Missing recovery projection")?;
        assert_eq!(
            view.recovery_operation_id.as_deref(),
            visible.then_some(input.id.as_str())
        );
        store
            .complete_conversation_input_forwarding(
                &source,
                &draft,
                &ConversationInputOutcome::Failed,
            )
            .await?;
    }
    Ok(())
}

#[test]
fn sqlite_model_selection_serializes_with_start_and_keeps_exact_receipts() -> Result<(), TestError>
{
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("models.sqlite");
    let cipher = cipher()?;
    prepare_sqlite(&path, &cipher)?;
    let runtime = tokio::runtime::Runtime::new()?;
    let store = runtime.block_on(Store::connect_sqlite(path.clone(), cipher))?;
    let f = runtime.block_on(fixture(&store))?;
    runtime.block_on(exercise(&store, &f))?;
    let mut connection = diesel::SqliteConnection::establish(path.to_str().ok_or("Invalid path")?)?;
    for expected in [
        "00000000000037",
        "00000000000036",
        "00000000000035",
        "00000000000034",
    ] {
        assert_eq!(
            connection
                .revert_last_migration(SQLITE_MIGRATIONS)?
                .to_string(),
            expected
        );
    }
    assert!(connection.revert_last_migration(SQLITE_MIGRATIONS).is_err());
    Ok(())
}

#[test]
fn postgres_model_selection_serializes_with_start_and_keeps_exact_receipts() -> Result<(), TestError>
{
    let _lock = POSTGRES_CONVERSATION_LOCK
        .lock()
        .map_err(|_| "PostgreSQL fixture lock poisoned")?;
    let base = std::env::var("CHAPCHOO_TEST_POSTGRES_URL")?;
    let mut control = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&base)?;
    let suffix = BearerSecret::generate()?.digest().as_bytes()[..8]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let schema = format!("models_{suffix}");
    control.batch_execute(&format!("CREATE SCHEMA {schema}"))?;
    let separator = if base.contains('?') { '&' } else { '?' };
    let url = format!("{base}{separator}options=-csearch_path%3D{schema}");
    let result = (|| -> Result<(), TestError> {
        let cipher = cipher()?;
        prepare_postgres(&url, &cipher)?;
        let runtime = tokio::runtime::Runtime::new()?;
        let store = runtime.block_on(Store::connect_postgres(url.clone(), cipher))?;
        let f = runtime.block_on(fixture(&store))?;
        runtime.block_on(exercise(&store, &f))?;
        let mut connection = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&url)?;
        for expected in [
            "00000000000037",
            "00000000000036",
            "00000000000035",
            "00000000000034",
        ] {
            assert_eq!(
                connection
                    .revert_last_migration(POSTGRES_MIGRATIONS)?
                    .to_string(),
                expected
            );
        }
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
