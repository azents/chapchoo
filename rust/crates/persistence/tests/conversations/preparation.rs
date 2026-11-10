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
            .begin_conversation_input_preparation(&f.other.digest(), &route, &draft, f.now)
            .await,
        Err(StoreError::NotFound)
    ));
    let no_native = store
        .accept_conversation_input(
            &owner,
            &AcceptConversationInput {
                agent_id: f.agent.id.clone(),
                conversation_id: Some(first.conversation_id.clone()),
                request_id: "known-no-native".into(),
                target: ConversationTarget::NotStarted,
                text: ConversationText::parse("explicit preparation attempt".into())?,
            },
            f.now,
        )
        .await?;
    let no_native = Draft {
        operation_id: &no_native.id,
        ..draft
    };
    rejected_initial_recovery(store, f, &route, &draft).await?;
    assert!(matches!(
        store
            .begin_conversation_input_preparation(&owner, &route, &no_native, f.now)
            .await?,
        Admission::Granted {
            fresh_native_allowed: true,
            ..
        }
    ));
    store
        .complete_conversation_input_preparation(
            &source,
            &no_native,
            &Outcome::Failed(Failure::BeforeNativeLaunch),
        )
        .await?;
    let selection = store.harness_supply(&owner, &f.agent.id, f.now).await?;
    let selection_id = selection.saved.map(|saved| saved.request_id);
    let (one, two) = tokio::join!(
        store.begin_conversation_input_preparation(&owner, &route, &draft, f.now),
        store.begin_conversation_input_preparation(&owner, &route, &draft, f.now)
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
    store
        .check_conversation_input_preparation(
            &source,
            &draft,
            &HarnessSupply::Managed,
            Some(true),
            None,
        )
        .await?;
    assert!(matches!(
        store
            .check_conversation_input_preparation(
                &source,
                &draft,
                &HarnessSupply::Managed,
                Some(false),
                None
            )
            .await,
        Err(StoreError::InvalidTransition)
    ));
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
    assert!(matches!(
        store
            .begin_conversation_input_preparation(&owner, &route, &pending, f.now)
            .await?,
        Admission::Granted {
            fresh_native_allowed: false,
            ..
        }
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
        .begin_conversation_input_preparation(&owner, &route, &draft, f.now)
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
            Some(true),
            None,
        )
        .await?;
    assert!(matches!(
        store
            .check_conversation_input_preparation(
                &source,
                &draft,
                &HarnessSupply::Existing(ExistingCodexPath::parse("/next/codex".into())?),
                Some(true),
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
    let failed = store
        .accept_conversation_input(
            &owner,
            &AcceptConversationInput {
                agent_id: f.agent.id.clone(),
                conversation_id: Some(first.conversation_id.clone()),
                request_id: "setup-failed".into(),
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
            .begin_conversation_input_preparation(&owner, &route, &failed, f.now)
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
        matches!(store.begin_conversation_input_preparation(&owner,&route,&failed,f.now).await?,Admission::Existing(facts) if facts.state==State::Known(Outcome::Failed(Failure::SourceUnavailable)))
    );
    Ok(())
}
async fn rejected_initial_recovery(
    store: &Store,
    f: &Fixture,
    route: &chapchoo_persistence::ResolvedRoute,
    draft: &Draft<'_>,
) -> Result<(), TestError> {
    let owner = f.owner.digest();
    let key = f.connector.digest();
    let source = f.source(&key);
    let mut operations = Vec::new();
    for request_id in ["initial-rejected", "resume-denied", "retry-rejected"] {
        let input = store
            .accept_conversation_input(
                &owner,
                &AcceptConversationInput {
                    agent_id: f.agent.id.clone(),
                    conversation_id: Some(draft.conversation_id.into()),
                    request_id: request_id.into(),
                    target: ConversationTarget::NotStarted,
                    text: ConversationText::parse("explicit new input".into())?,
                },
                f.now,
            )
            .await?;
        operations.push(input);
    }
    let first = Draft {
        operation_id: &operations[0].id,
        ..*draft
    };
    let denied = Draft {
        operation_id: &operations[1].id,
        ..*draft
    };
    let retry = Draft {
        operation_id: &operations[2].id,
        ..*draft
    };
    assert!(matches!(
        store
            .begin_conversation_input_preparation(&owner, route, &first, f.now)
            .await?,
        Admission::Granted {
            fresh_native_allowed: true,
            ..
        }
    ));
    assert!(matches!(
        store
            .begin_conversation_input_preparation(&owner, route, &denied, f.now)
            .await?,
        Admission::Granted {
            fresh_native_allowed: false,
            ..
        }
    ));
    store
        .complete_conversation_input_preparation(
            &source,
            &denied,
            &Outcome::Failed(Failure::SourceUnavailable),
        )
        .await?;
    // A resume-only lookup failure cannot clear an unresolved creation attempt.
    assert!(matches!(
        store
            .begin_conversation_input_preparation(&owner, route, &retry, f.now)
            .await?,
        Admission::Granted {
            fresh_native_allowed: false,
            ..
        }
    ));
    store
        .complete_conversation_input_preparation(
            &source,
            &retry,
            &Outcome::Failed(Failure::SourceUnavailable),
        )
        .await?;
    store
        .complete_conversation_input_preparation(
            &source,
            &first,
            &Outcome::Failed(Failure::NativeRejected),
        )
        .await?;
    // The old operation remains settled; only a new explicit input can start.
    assert!(
        matches!(store.begin_conversation_input_preparation(&owner, route, &first, f.now).await?,
        Admission::Existing(facts) if facts.state == State::Known(Outcome::Failed(Failure::NativeRejected)))
    );
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
