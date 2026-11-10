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
    };
    store
        .observe_conversation_native(&source, conversation, &idle)
        .await?;
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
