use super::*;
use chapchoo_persistence::{
    ConversationDeliveryTarget, ConversationInputPreparation as Draft,
    ConversationPreparationAdmission as Admission, ConversationPreparationFailure as Failure,
    ConversationPreparationOutcome as Outcome, ConversationPreparationState as State,
};
pub(super) async fn initial(
    store: &Store,
    f: &Fixture,
    first: &chapchoo_persistence::ConversationInputReceipt,
    snapshot: &SnapshotId,
    native: &NativeConversationId,
) -> Result<(), TestError> {
    let owner = f.owner.digest();
    let key = f.connector.digest();
    let source = f.source(&key);
    let route = store
        .resolve_connector_route(&f.agent.computer.id)
        .await?
        .ok_or("Missing route")?;
    let draft = Draft {
        agent_id: &f.agent.id,
        conversation_id: &first.conversation_id,
        operation_id: &first.id,
    };
    assert!(
        store
            .conversation_input_preparation(&owner, &draft, f.now)
            .await?
            .is_none()
    );
    assert!(matches!(
        store
            .begin_conversation_input_preparation(
                &f.other.digest(),
                &route,
                &draft,
                f.now,
                &f.claim(store, draft.conversation_id, draft.operation_id)
                    .await?
            )
            .await,
        Err(StoreError::NotFound)
    ));
    let selection = store.harness_supply(&owner, &f.agent.id, f.now).await?;
    let selection_id = selection.saved.map(|saved| saved.request_id);
    let claim = f
        .claim(store, draft.conversation_id, draft.operation_id)
        .await?;
    let (one, two) = tokio::join!(
        store.begin_conversation_input_preparation(&owner, &route, &draft, f.now, &claim),
        store.begin_conversation_input_preparation(&owner, &route, &draft, f.now, &claim)
    );
    let (granted, existing) = match (one?, two?) {
        (
            Admission::Granted {
                requested_supply,
                selection_request_id,
                fresh_native_allowed: true,
                ..
            },
            Admission::Existing(existing),
        )
        | (
            Admission::Existing(existing),
            Admission::Granted {
                requested_supply,
                selection_request_id,
                fresh_native_allowed: true,
                ..
            },
        ) => ((requested_supply, selection_request_id), existing),
        _ => return Err("Duplicate preparation granted another effect".into()),
    };
    assert_eq!(granted.0, HarnessSupply::Managed);
    assert_eq!(granted.1, selection_id);
    assert_eq!(existing.state, State::PossibleForwarding);
    assert!(existing.fresh_native_allowed);
    assert!(existing.require_retained_source);
    store
        .check_conversation_input_preparation(
            &source,
            &draft,
            &HarnessSupply::Managed,
            Some((true, true)),
            None,
        )
        .await?;
    assert!(matches!(
        store
            .check_conversation_input_preparation(
                &source,
                &draft,
                &HarnessSupply::Managed,
                Some((false, true)),
                None
            )
            .await,
        Err(StoreError::InvalidTransition)
    ));
    assert!(matches!(
        store
            .check_conversation_input_preparation(
                &source,
                &draft,
                &HarnessSupply::Managed,
                Some((true, false)),
                None
            )
            .await,
        Err(StoreError::InvalidTransition)
    ));
    store
        .save_harness_supply(
            &owner,
            &SaveHarnessSupply {
                agent_id: f.agent.id.clone(),
                request_id: "next-existing".into(),
                expected_request_id: selection_id.clone(),
                supply: HarnessSupply::Existing(ExistingCodexPath::parse("/next/codex".into())?),
            },
            f.now,
        )
        .await?;
    let Admission::Existing(existing) = store
        .begin_conversation_input_preparation(
            &owner,
            &route,
            &draft,
            f.now,
            &f.claim(store, draft.conversation_id, draft.operation_id)
                .await?,
        )
        .await?
    else {
        return Err("Saved selection granted another startup".into());
    };
    assert_eq!(existing.requested_supply, HarnessSupply::Managed);
    assert_eq!(existing.selection_request_id, selection_id);
    assert!(existing.fresh_native_allowed);
    store
        .check_conversation_input_preparation(
            &source,
            &draft,
            &HarnessSupply::Managed,
            Some((true, true)),
            None,
        )
        .await?;
    assert!(matches!(
        store
            .check_conversation_input_preparation(
                &source,
                &draft,
                &HarnessSupply::Existing(ExistingCodexPath::parse("/next/codex".into())?),
                Some((true, true)),
                None
            )
            .await,
        Err(StoreError::Conflict)
    ));
    store
        .save_harness_supply(
            &owner,
            &SaveHarnessSupply {
                agent_id: f.agent.id.clone(),
                request_id: "next-managed".into(),
                expected_request_id: Some("next-existing".into()),
                supply: HarnessSupply::Managed,
            },
            f.now,
        )
        .await?;
    let result = Outcome::SessionReady(ConversationDeliveryTarget {
        snapshot: snapshot.clone(),
        native_session: native.clone(),
    });
    let instructions = chapchoo_domain::ProjectInstructions::new(vec![
        chapchoo_domain::InstructionSource {
            kind: chapchoo_domain::InstructionSourceKind::Product,
            text: "Preserve existing user files.".into(),
        },
        chapchoo_domain::InstructionSource {
            kind: chapchoo_domain::InstructionSourceKind::Owner,
            text: "".into(),
        },
    ])?;
    store
        .seal_project_instructions(
            &owner,
            &route,
            &draft,
            &instructions,
            br#"{"tools":[]}"#,
            f.now,
        )
        .await?;
    store
        .complete_conversation_input_preparation(&source, &draft, &result)
        .await?;
    store
        .complete_conversation_input_preparation(&source, &draft, &result)
        .await?;
    assert!(matches!(
        store
            .complete_conversation_input_preparation(
                &source,
                &draft,
                &Outcome::Failed(Failure::NotAdmitted)
            )
            .await,
        Err(StoreError::Conflict)
    ));
    assert_eq!(
        store
            .conversation_input_preparation(&owner, &draft, f.now)
            .await?
            .ok_or("Missing preparation")?
            .state,
        State::Known(result)
    );
    // Setup readiness does not fabricate a source cursor or native input consumption.
    assert!(matches!(
        store
            .current_conversation(&owner, &f.agent.id, f.now)
            .await?
            .ok_or("Missing conversation")?
            .source,
        ConversationSource::NotStarted
    ));
    assert!(
        store
            .conversation_input_delivery(
                &owner,
                &f.agent.id,
                &first.conversation_id,
                &first.id,
                f.now
            )
            .await?
            .is_none()
    );
    assert!(matches!(
        store
            .bind_conversation_source(
                &source,
                &SourceBinding {
                    conversation_id: first.conversation_id.clone(),
                    expected_previous: None,
                    snapshot: super::snapshot("wrong-initial-source")?,
                    native_session: native.clone()
                }
            )
            .await,
        Err(StoreError::Conflict)
    ));
    Ok(())
}
pub(super) async fn rejected_initial_recovery(store: &Store, f: &Fixture) -> Result<(), TestError> {
    let owner = f.owner.digest();
    let key = f.connector.digest();
    let source = f.source(&key);
    let route = store
        .resolve_connector_route(&f.agent.computer.id)
        .await?
        .ok_or("Missing initial recovery route")?;
    let mut operations = Vec::new();
    for request_id in ["initial-rejected", "resume-denied", "retry-rejected"] {
        operations.push(
            store
                .accept_conversation_input(
                    &owner,
                    &AcceptConversationInput {
                        agent_id: f.agent.id.clone(),
                        conversation_id: None,
                        request_id: request_id.into(),
                        target: ConversationTarget::NotStarted,
                        text: ConversationText::parse("explicit new input".into())?,
                    },
                    f.now,
                )
                .await?,
        );
    }
    let first = Draft {
        agent_id: &f.agent.id,
        conversation_id: &operations[0].conversation_id,
        operation_id: &operations[0].id,
    };
    let denied = Draft {
        operation_id: &operations[1].id,
        ..first
    };
    let retry = Draft {
        operation_id: &operations[2].id,
        ..first
    };
    let claim = f
        .claim(store, first.conversation_id, first.operation_id)
        .await?;
    assert!(matches!(
        store
            .begin_conversation_input_preparation(&owner, &route, &first, f.now, &claim)
            .await?,
        Admission::Granted {
            fresh_native_allowed: true,
            ..
        }
    ));
    for blocked in [&denied, &retry] {
        let candidate = chapchoo_persistence::ConversationInputCandidate {
            id: blocked.operation_id.into(),
            agent_id: f.agent.id.clone(),
            conversation_id: blocked.conversation_id.into(),
        };
        assert!(
            store
                .claim_conversation_input(&candidate, DirectoryPolicy::default())
                .await?
                .is_none()
        );
        assert!(matches!(
            store
                .begin_conversation_input_preparation(&owner, &route, blocked, f.now, &claim)
                .await,
            Err(StoreError::InvalidTransition)
        ));
    }
    store
        .complete_conversation_input_preparation(
            &source,
            &first,
            &Outcome::Failed(Failure::NativeRejected),
        )
        .await?;
    // Resolve the deliberately blocked middle item before claiming the next FIFO head.
    store
        .fail_conversation_input(denied.conversation_id, denied.operation_id)
        .await?;
    let retry_claim = f
        .claim(store, retry.conversation_id, retry.operation_id)
        .await?;
    assert!(matches!(
        store
            .begin_conversation_input_preparation(&owner, &route, &retry, f.now, &retry_claim)
            .await?,
        Admission::Granted {
            fresh_native_allowed: true,
            ..
        }
    ));
    store
        .complete_conversation_input_preparation(
            &source,
            &retry,
            &Outcome::Failed(Failure::BeforeNativeLaunch),
        )
        .await?;
    assert!(matches!(
        store.begin_conversation_input_preparation(&owner, &route, &first, f.now, &claim).await?,
        Admission::Existing(facts) if facts.state == State::Known(Outcome::Failed(Failure::NativeRejected))
    ));
    Ok(())
}
pub(super) async fn replaced(
    store: &Store,
    f: &Fixture,
    first: &chapchoo_persistence::ConversationInputReceipt,
) -> Result<(), TestError> {
    let draft = Draft {
        agent_id: &f.agent.id,
        conversation_id: &first.conversation_id,
        operation_id: &first.id,
    };
    assert!(matches!(
        store
            .conversation_input_preparation(&f.owner.digest(), &draft, f.now)
            .await,
        Err(StoreError::NotFound)
    ));
    Ok(())
}

async fn retained_setup_reads(store: &Store, f: &Fixture) -> Result<(), TestError> {
    let owner = f.owner.digest();
    let key = f.connector.digest();
    let source = f.source(&key);
    let route = store
        .resolve_connector_route(&f.agent.computer.id)
        .await?
        .ok_or("Missing route")?;
    let first = store
        .accept_conversation_input(
            &owner,
            &AcceptConversationInput {
                agent_id: f.agent.id.clone(),
                conversation_id: None,
                request_id: "setup-read-seed".into(),
                target: ConversationTarget::NotStarted,
                text: ConversationText::parse("Retained original seed".into())?,
            },
            f.now,
        )
        .await?;
    store
        .fail_conversation_input(&first.conversation_id, &first.id)
        .await?;
    let draft = Draft {
        agent_id: &f.agent.id,
        conversation_id: &first.conversation_id,
        operation_id: &first.id,
    };
    let snapshot = super::snapshot("setup-read-source")?;
    let native = NativeConversationId::parse("setup-read-native".into())?;
    let instructions = chapchoo_domain::ProjectInstructions::new(vec![
        chapchoo_domain::InstructionSource {
            kind: chapchoo_domain::InstructionSourceKind::Product,
            text: "Preserve existing user files.".into(),
        },
        chapchoo_domain::InstructionSource {
            kind: chapchoo_domain::InstructionSourceKind::Owner,
            text: "".into(),
        },
    ])?;
    let failed = store
        .accept_conversation_input(
            &owner,
            &AcceptConversationInput {
                agent_id: f.agent.id.clone(),
                conversation_id: Some(first.conversation_id.clone()),
                request_id: "zz-setup-failed".into(),
                target: ConversationTarget::NotStarted,
                text: ConversationText::parse("another explicit input".into())?,
            },
            f.now,
        )
        .await?;
    let failed = Draft {
        operation_id: &failed.id,
        ..draft
    };
    assert!(matches!(
        store
            .begin_conversation_input_preparation(
                &owner,
                &route,
                &failed,
                f.now,
                &f.claim(store, failed.conversation_id, failed.operation_id)
                    .await?
            )
            .await?,
        Admission::Granted { .. }
    ));
    store
        .complete_conversation_input_preparation(
            &source,
            &failed,
            &Outcome::Failed(Failure::SourceUnavailable),
        )
        .await?;
    assert!(
        matches!(store.begin_conversation_input_preparation(&owner, &route, &failed, f.now, &f.claim(store, failed.conversation_id, failed.operation_id).await?).await?,Admission::Existing(facts) if facts.state==State::Known(Outcome::Failed(Failure::SourceUnavailable)))
    );
    let pending = store
        .accept_conversation_input(
            &owner,
            &AcceptConversationInput {
                agent_id: f.agent.id.clone(),
                conversation_id: Some(first.conversation_id.clone()),
                request_id: "unresolved-start".into(),
                target: ConversationTarget::NotStarted,
                text: ConversationText::parse("independent accepted input".into())?,
            },
            f.now,
        )
        .await?;
    let pending = Draft {
        operation_id: &pending.id,
        ..draft
    };
    let scope = chapchoo_persistence::ConversationInputScope {
        agent_id: &f.agent.id,
        conversation_id: &first.conversation_id,
        target: &ConversationTarget::NotStarted,
    };
    assert!(matches!(
        store
            .delete_failed_conversation_input(&f.other.digest(), &scope, "zz-setup-failed", f.now)
            .await,
        Err(StoreError::NotFound)
    ));
    assert!(matches!(
        store
            .delete_failed_conversation_input(&owner, &scope, "unresolved-start", f.now)
            .await,
        Err(StoreError::InvalidTransition)
    ));
    let stale_scope = chapchoo_persistence::ConversationInputScope {
        target: &ConversationTarget::Snapshot(snapshot.clone()),
        ..scope
    };
    assert!(matches!(
        store
            .delete_failed_conversation_input(&owner, &stale_scope, "zz-setup-failed", f.now)
            .await,
        Err(StoreError::Conflict)
    ));
    assert_eq!(
        store
            .latest_conversation_input(&owner, &scope, f.now)
            .await?
            .ok_or("Missing newest input")?
            .reference
            .request_id,
        "zz-setup-failed"
    );
    let deleted = store
        .conversation_inputs(&owner, &scope, None, 25, f.now)
        .await?
        .into_iter()
        .find(|item| item.reference.request_id == "zz-setup-failed")
        .ok_or("Missing failed queue item")?;
    store
        .delete_failed_conversation_input(&owner, &scope, "zz-setup-failed", f.now)
        .await?;
    store
        .delete_failed_conversation_input(&owner, &scope, "zz-setup-failed", f.now)
        .await?;
    let listed = store
        .conversation_inputs(&owner, &scope, None, 25, f.now)
        .await?;
    assert!(
        !listed
            .iter()
            .any(|item| item.reference.request_id == "zz-setup-failed")
    );
    assert_eq!(
        store
            .closed_conversation_inputs(
                &owner,
                &scope,
                std::slice::from_ref(&deleted.reference.id),
                f.now
            )
            .await?,
        vec![deleted.reference.id.clone()],
    );
    assert!(
        store
            .conversation_input_reference(
                &owner,
                &f.agent.id,
                &first.conversation_id,
                "zz-setup-failed",
                f.now,
            )
            .await?
            .is_none()
    );
    let live = store
        .latest_conversation_input(&owner, &scope, f.now)
        .await?
        .ok_or("Missing earlier live input")?;
    assert_eq!(live.reference.request_id, "unresolved-start");
    assert!(matches!(
        store
            .latest_conversation_input(&owner, &stale_scope, f.now)
            .await,
        Err(StoreError::Conflict)
    ));
    let older = store
        .conversation_inputs(
            &owner,
            &scope,
            Some(&chapchoo_persistence::ConversationInputAnchor {
                accepted_at: deleted.reference.accepted_at,
                request_id: deleted.reference.request_id.clone(),
            }),
            25,
            f.now,
        )
        .await?;
    assert!(
        older
            .iter()
            .all(|item| item.reference.request_id != "zz-setup-failed")
    );
    assert_eq!(
        store
            .latest_conversation_input(&owner, &scope, f.now)
            .await?
            .ok_or("Missing live input after an older page")?
            .reference
            .request_id,
        "unresolved-start"
    );
    assert!(matches!(
        store
            .conversation_input_text(&owner, &scope, "zz-setup-failed", 0, 1, f.now)
            .await,
        Err(StoreError::NotFound)
    ));
    // The deleted failure remains the selected historical operation. Recover
    // with a genuinely new pending setup, never by reselecting an old READY fact.
    store
        .begin_conversation_input_preparation(
            &owner,
            &route,
            &pending,
            f.now,
            &f.claim(store, pending.conversation_id, pending.operation_id)
                .await?,
        )
        .await?;
    store
        .seal_project_instructions(
            &owner,
            &route,
            &pending,
            &instructions,
            br#"{"tools":[]}"#,
            f.now,
        )
        .await?;
    store
        .complete_conversation_input_preparation(
            &source,
            &pending,
            &Outcome::SessionReady(ConversationDeliveryTarget {
                snapshot: snapshot.clone(),
                native_session: native.clone(),
            }),
        )
        .await?;
    Ok(())
}

#[test]
fn sqlite_retained_setup_failure_reads() -> Result<(), TestError> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("setup-reads.sqlite");
    let cipher = cipher()?;
    prepare_sqlite(&path, &cipher)?;
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        let store = Store::connect_sqlite(path, cipher).await?;
        let f = fixture(&store).await?;
        retained_setup_reads(&store, &f).await
    })
}

#[test]
fn postgres_retained_setup_failure_reads() -> Result<(), TestError> {
    let _lock = POSTGRES_CONVERSATION_LOCK
        .lock()
        .map_err(|_| "Fixture lock poisoned")?;
    let base = std::env::var("CHAPCHOO_TEST_POSTGRES_URL")
        .map_err(|_| "Disposable PostgreSQL is required")?;
    let mut control = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&base)?;
    let schema = format!("setup_reads_{}", std::process::id());
    control.batch_execute(&format!("CREATE SCHEMA {schema}"))?;
    let separator = if base.contains('?') { '&' } else { '?' };
    let url = format!("{base}{separator}options=-csearch_path%3D{schema}");
    let result = (|| -> Result<(), TestError> {
        let cipher = cipher()?;
        prepare_postgres(&url, &cipher)?;
        tokio::runtime::Runtime::new()?.block_on(async {
            let store = Store::connect_postgres(url, cipher).await?;
            let f = fixture(&store).await?;
            retained_setup_reads(&store, &f).await
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
