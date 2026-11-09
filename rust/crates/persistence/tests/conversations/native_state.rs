use super::*;
use chapchoo_persistence::{
    AcceptConversationStop as Stop, ConversationDeliveryTarget,
    ConversationForeground as Foreground, ConversationNativeObservation as Observation,
    ConversationStopAdmission as Admission, ConversationStopState as StopState,
    ConversationWorkTarget as Target,
};
pub(super) async fn exercise(
    store: &Store,
    f: &Fixture,
    conversation: &str,
    snapshot: &SnapshotId,
    native: &NativeConversationId,
) -> Result<(), TestError> {
    let owner = f.owner.digest();
    let credential = f.connector.digest();
    let source = f.source(&credential);
    let target = ConversationDeliveryTarget {
        snapshot: snapshot.clone(),
        native_session: native.clone(),
    };
    let running = Observation {
        target: target.clone(),
        position: position(10, "running-state")?,
        foreground: Foreground::Running,
        work_id: Some("exact-work".into()),
        preparation: None,
        authentication_id: None,
        applied_supply: Some(HarnessSupply::Managed),
    };
    assert!(
        store
            .conversation_native_observation(&owner, &f.agent.id, conversation, f.now)
            .await?
            .is_none()
    );
    assert!(matches!(
        store
            .observe_conversation_native(&source, conversation, &running)
            .await,
        Err(StoreError::InvalidTransition)
    ));
    for observed in [
        &running,
        &Observation {
            target: target.clone(),
            position: position(11, "idle-state")?,
            foreground: Foreground::Idle,
            work_id: None,
            preparation: None,
            authentication_id: None,
            applied_supply: Some(HarnessSupply::Managed),
        },
    ] {
        store
            .ingest_conversation_pair(
                &source,
                conversation,
                &CapturedEventPair {
                    snapshot: snapshot.clone(),
                    position: observed.position.clone(),
                    raw: b"native state".to_vec(),
                    normalized: b"state".to_vec(),
                },
            )
            .await?;
    }
    let unknown_supply = Observation {
        applied_supply: None,
        ..running.clone()
    };
    store
        .observe_conversation_native(&source, conversation, &unknown_supply)
        .await?;
    assert!(
        store
            .conversation_native_observation(&owner, &f.agent.id, conversation, f.now)
            .await?
            .ok_or("Missing original observation")?
            .applied_supply
            .is_none()
    );
    store
        .observe_conversation_native(&source, conversation, &running)
        .await?;
    store
        .observe_conversation_native(&source, conversation, &running)
        .await?;
    assert_eq!(
        store
            .conversation_native_observation(&owner, &f.agent.id, conversation, f.now)
            .await?
            .ok_or("Missing native observation")?,
        running
    );
    // Old reports remain unknown until one physical owner explicitly supplies details.
    let unknown = running.clone();
    let preparation = chapchoo_persistence::ConversationNativePreparation {
        freshness: chapchoo_persistence::ConversationPreparationFreshness::Current,
        node: "24.0.0".into(),
        adapter: "2.6.0".into(),
        codex: "0.165.0".into(),
    };
    let running = Observation {
        preparation: Some(preparation.clone()),
        authentication_id: Some(
            f.agent
                .authentication
                .as_ref()
                .ok_or("Missing saved authentication")?
                .authentication_id
                .clone(),
        ),
        ..running
    };
    store
        .observe_conversation_native(&source, conversation, &running)
        .await?;
    store
        .observe_conversation_native(&source, conversation, &unknown)
        .await?;
    assert_eq!(
        store
            .conversation_native_observation(&owner, &f.agent.id, conversation, f.now)
            .await?
            .ok_or("Missing explicit details")?,
        running
    );
    let conflicting = Observation {
        preparation: Some(chapchoo_persistence::ConversationNativePreparation {
            codex: "0.166.0".into(),
            ..preparation.clone()
        }),
        ..running.clone()
    };
    assert!(matches!(
        store
            .observe_conversation_native(&source, conversation, &conflicting)
            .await,
        Err(StoreError::Conflict)
    ));
    authentication_changes_preserve_applied(store, f, conversation, &running).await?;
    let work = Target {
        source: target.clone(),
        work_id: "exact-work".into(),
    };
    let stop = Stop {
        agent_id: &f.agent.id,
        conversation_id: conversation,
        request_id: "stop-exact",
        target: &work,
    };
    let (one, two) = tokio::join!(
        store.accept_conversation_stop(&owner, &stop, f.now),
        store.accept_conversation_stop(&owner, &stop, f.now)
    );
    let receipt = match (one?, two?) {
        (Admission::Accepted(receipt), Admission::Existing(existing))
        | (Admission::Existing(existing), Admission::Accepted(receipt)) => {
            assert_eq!(receipt.id, existing.id);
            receipt
        }
        _ => return Err("Duplicate Stop granted another effect".into()),
    };
    assert_eq!(receipt.state, StopState::PossibleForwarding);
    store
        .check_conversation_stop(&source, conversation, &receipt.id, &work)
        .await?;
    let wrong = Target {
        work_id: "other-work".into(),
        ..work.clone()
    };
    assert!(matches!(
        store
            .accept_conversation_stop(
                &owner,
                &Stop {
                    target: &wrong,
                    ..stop
                },
                f.now
            )
            .await,
        Err(StoreError::Conflict)
    ));
    let idle = Observation {
        target,
        position: position(11, "idle-state")?,
        foreground: Foreground::Idle,
        work_id: None,
        // A later captured position can describe a later physical start.
        preparation: Some(chapchoo_persistence::ConversationNativePreparation {
            freshness:
                chapchoo_persistence::ConversationPreparationFreshness::TransientLookupFailure,
            ..preparation
        }),
        authentication_id: running.authentication_id.clone(),
        applied_supply: Some(HarnessSupply::Managed),
    };
    store
        .observe_conversation_native(&source, conversation, &idle)
        .await?;
    let changed_supply = Observation {
        applied_supply: Some(HarnessSupply::Existing(ExistingCodexPath::parse(
            "/owner/codex".into(),
        )?)),
        ..idle.clone()
    };
    assert!(matches!(
        store
            .observe_conversation_native(&source, conversation, &changed_supply)
            .await,
        Err(StoreError::Conflict)
    ));
    store
        .observe_conversation_native(&source, conversation, &running)
        .await?;
    assert_eq!(
        store
            .conversation_native_observation(&owner, &f.agent.id, conversation, f.now)
            .await?
            .ok_or("Missing idle")?,
        idle
    );
    assert!(matches!(
        store
            .accept_conversation_stop(
                &owner,
                &Stop {
                    request_id: "stale-stop",
                    ..stop
                },
                f.now
            )
            .await,
        Err(StoreError::InvalidTransition)
    ));
    assert!(matches!(
        store
            .accept_conversation_stop(&f.other.digest(), &stop, f.now)
            .await,
        Err(StoreError::NotFound)
    ));
    // The exact native owner remains final authority even if stored work is stale.
    store
        .complete_conversation_stop(&source, conversation, &receipt.id, &work, false)
        .await?;
    store
        .complete_conversation_stop(&source, conversation, &receipt.id, &work, false)
        .await?;
    assert!(matches!(
        store
            .complete_conversation_stop(&source, conversation, &receipt.id, &work, true)
            .await,
        Err(StoreError::Conflict)
    ));
    assert_eq!(
        store
            .conversation_stop(&owner, &f.agent.id, conversation, "stop-exact", f.now)
            .await?
            .ok_or("Missing Stop")?
            .state,
        StopState::Rejected
    );
    let conflict = Observation {
        work_id: Some("unexpected".into()),
        foreground: Foreground::Running,
        ..idle
    };
    assert!(matches!(
        store
            .observe_conversation_native(&source, conversation, &conflict)
            .await,
        Err(StoreError::Conflict)
    ));
    // A retained source can describe a later physical start with a new applied choice.
    let resumed = Observation {
        target: running.target.clone(),
        position: position(12, "resumed-state")?,
        foreground: Foreground::Idle,
        work_id: None,
        preparation: conflict.preparation.clone(),
        authentication_id: conflict.authentication_id.clone(),
        applied_supply: Some(HarnessSupply::Existing(ExistingCodexPath::parse(
            "/owner/new codex".into(),
        )?)),
    };
    store
        .ingest_conversation_pair(
            &source,
            conversation,
            &CapturedEventPair {
                snapshot: snapshot.clone(),
                position: resumed.position.clone(),
                raw: b"resumed native state".to_vec(),
                normalized: b"state".to_vec(),
            },
        )
        .await?;
    store
        .observe_conversation_native(&source, conversation, &resumed)
        .await?;
    store
        .observe_conversation_native(&source, conversation, &running)
        .await?;
    assert_eq!(
        store
            .conversation_native_observation(&owner, &f.agent.id, conversation, f.now)
            .await?
            .ok_or("Missing later applied observation")?,
        resumed
    );
    assert_eq!(
        store
            .agent(&owner, &f.agent.id, f.now)
            .await?
            .computer
            .current_request_id,
        f.agent.computer.current_request_id
    );
    Ok(())
}

async fn authentication_changes_preserve_applied(
    store: &Store,
    f: &Fixture,
    conversation: &str,
    observed: &Observation,
) -> Result<(), TestError> {
    use chapchoo_persistence::{
        DeleteAuthentication, DeleteAuthenticationOutcome, ExpectedAgentAuthentication,
        ReplaceApiKey, SelectAgentAuthentication,
    };
    let saved = f
        .agent
        .authentication
        .as_ref()
        .ok_or("Missing authentication")?;
    let scope = AuthenticationScope::Team {
        team_id: f.team_id.clone(),
    };
    let next = match store
        .register_external_api_key(
            &f.owner.digest(),
            &RegisterApiKey {
                scope: scope.clone(),
                request_id: "native-next-record".into(),
                name: String::new(),
                key: ExternalSecret::new(b"synthetic-next-native-key".to_vec())?,
            },
            f.now,
        )
        .await?
    {
        AuthenticationAcceptedResult::Record { authentication_id } => authentication_id,
        _ => return Err("Expected next authentication".into()),
    };
    for (actor, agent, request) in [
        (
            f.owner.digest(),
            f.agent.id.as_str(),
            "native-owner-selection",
        ),
        (
            f.other.digest(),
            f.other_agent.as_str(),
            "native-other-selection",
        ),
    ] {
        let selected = store
            .agent(&actor, agent, f.now)
            .await?
            .authentication
            .ok_or("Missing selection")?;
        store
            .select_agent_authentication(
                &actor,
                &SelectAgentAuthentication {
                    agent_id: agent.into(),
                    request_id: request.into(),
                    authentication_id: next.clone(),
                    expected: Some(ExpectedAgentAuthentication {
                        authentication_id: selected.authentication_id,
                        request_id: selected.request_id,
                    }),
                },
                f.now,
            )
            .await?;
    }
    assert_eq!(
        store
            .agent(&f.owner.digest(), &f.agent.id, f.now)
            .await?
            .authentication
            .ok_or("Missing fresh selection")?
            .authentication_id,
        next
    );
    store
        .replace_external_api_key(
            &f.owner.digest(),
            &ReplaceApiKey {
                scope: scope.clone(),
                request_id: "native-same-record-update".into(),
                authentication_id: saved.authentication_id.clone(),
                expected_request_id: "conversation-auth".into(),
                key: ExternalSecret::new(b"synthetic-updated-native-key".to_vec())?,
            },
            f.now,
        )
        .await?;
    assert_eq!(
        store
            .conversation_native_observation(&f.owner.digest(), &f.agent.id, conversation, f.now)
            .await?
            .as_ref(),
        Some(observed)
    );
    assert!(matches!(
        store
            .delete_external_authentication(
                &f.owner.digest(),
                &DeleteAuthentication {
                    scope,
                    request_id: "native-delete-applied-only".into(),
                    authentication_id: saved.authentication_id.clone(),
                    expected_request_id: "native-same-record-update".into(),
                },
                f.now
            )
            .await?,
        DeleteAuthenticationOutcome::Accepted(_)
    ));
    // Catalogue mutation and deletion cannot rewrite already observed native facts.
    assert_eq!(
        store
            .conversation_native_observation(&f.owner.digest(), &f.agent.id, conversation, f.now)
            .await?
            .as_ref(),
        Some(observed)
    );
    Ok(())
}
pub(super) async fn replaced(
    store: &Store,
    f: &Fixture,
    conversation: &str,
) -> Result<(), TestError> {
    assert!(
        store
            .conversation_native_observation(&f.owner.digest(), &f.agent.id, conversation, f.now)
            .await?
            .is_none()
    );
    assert!(
        store
            .conversation_stop(
                &f.owner.digest(),
                &f.agent.id,
                conversation,
                "stop-exact",
                f.now
            )
            .await?
            .is_none()
    );
    Ok(())
}
