use super::*;
use chapchoo_persistence::{
    ConversationDeliveryTarget, ConversationForwardingAdmission as Admission,
    ConversationInputDelivery as Delivery, ConversationInputOutcome as Outcome,
    ConversationOperationForwarding as Forwarding,
};

pub(super) async fn exercise(
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
        .ok_or("Missing Connector route")?;
    let target = ConversationDeliveryTarget {
        snapshot: snapshot.clone(),
        native_session: native.clone(),
    };
    let draft = Forwarding {
        agent_id: &f.agent.id,
        conversation_id: &first.conversation_id,
        operation_id: &first.id,
        target: &target,
    };
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
            .begin_conversation_input_forwarding(&f.other.digest(), &route, &draft, f.now)
            .await,
        Err(StoreError::NotFound)
    ));
    let (one, two) = tokio::join!(
        store.begin_conversation_input_forwarding(&owner, &route, &draft, f.now),
        store.begin_conversation_input_forwarding(&owner, &route, &draft, f.now),
    );
    assert!(matches!(
        (one?, two?),
        (
            Admission::Granted(_),
            Admission::Existing(Delivery::PossibleForwarding)
        ) | (
            Admission::Existing(Delivery::PossibleForwarding),
            Admission::Granted(_)
        )
    ));
    // A separate read observes committed evidence without another effect grant.
    assert_eq!(
        store
            .clone()
            .conversation_input_delivery(
                &owner,
                &f.agent.id,
                &first.conversation_id,
                &first.id,
                f.now
            )
            .await?,
        Some(Delivery::PossibleForwarding)
    );
    let stale = ConversationDeliveryTarget {
        snapshot: super::snapshot("wrong-source")?,
        native_session: native.clone(),
    };
    let stale = Forwarding {
        target: &stale,
        ..draft
    };
    assert!(matches!(
        store
            .begin_conversation_input_forwarding(&owner, &route, &stale, f.now)
            .await,
        Err(StoreError::Conflict)
    ));
    assert!(matches!(
        store
            .complete_conversation_input_forwarding(&source, &stale, &Outcome::Steered)
            .await,
        Err(StoreError::Conflict)
    ));
    let outcome = Outcome::Inserted {
        native_message_id: "real-message-id".into(),
    };
    store
        .complete_conversation_input_forwarding(&source, &draft, &outcome)
        .await?;
    store
        .complete_conversation_input_forwarding(&source, &draft, &outcome)
        .await?;
    assert!(matches!(
        store
            .complete_conversation_input_forwarding(&source, &draft, &Outcome::Steered)
            .await,
        Err(StoreError::Conflict)
    ));
    assert!(
        matches!(store.begin_conversation_input_forwarding(&owner,&route,&draft,f.now).await?,Admission::Existing(Delivery::Known(known)) if known==outcome)
    );
    assert_eq!(
        store
            .conversation_input_delivery(
                &owner,
                &f.agent.id,
                &first.conversation_id,
                &first.id,
                f.now
            )
            .await?,
        Some(Delivery::Known(outcome))
    );
    assert!(matches!(
        store
            .conversation_input_delivery(
                &f.other.digest(),
                &f.agent.id,
                &first.conversation_id,
                &first.id,
                f.now
            )
            .await,
        Err(StoreError::NotFound)
    ));
    let pending = store
        .accept_conversation_input(
            &owner,
            &AcceptConversationInput {
                agent_id: f.agent.id.clone(),
                conversation_id: Some(first.conversation_id.clone()),
                request_id: "unresolved-input".into(),
                target: ConversationTarget::Snapshot(snapshot.clone()),
                text: ConversationText::parse(
                    "한글🦀 same text, independent operation\n".repeat(4000),
                )?,
            },
            f.now,
        )
        .await?;
    assert!(matches!(
        store
            .begin_conversation_input_forwarding(
                &owner,
                &route,
                &Forwarding {
                    operation_id: &pending.id,
                    ..draft
                },
                f.now
            )
            .await?,
        Admission::Granted(_)
    ));
    let pending_draft = Forwarding {
        operation_id: &pending.id,
        ..draft
    };
    let mut bytes = Vec::new();
    while bytes.len() < pending.text.as_str().len() {
        let chunk = store
            .connector_input_chunk(&source, &pending_draft, u32::try_from(bytes.len())?)
            .await?;
        assert_eq!(chunk.offset as usize, bytes.len());
        assert_eq!(chunk.total_bytes as usize, pending.text.as_str().len());
        assert!(!chunk.bytes.is_empty() && chunk.bytes.len() <= 32768);
        bytes.extend(chunk.bytes);
    }
    assert_eq!(bytes, pending.text.as_str().as_bytes());
    assert!(
        store
            .connector_input_chunk(&source, &pending_draft, u32::try_from(bytes.len())?)
            .await?
            .bytes
            .is_empty()
    );
    assert!(matches!(
        store
            .connector_input_chunk(&source, &pending_draft, u32::try_from(bytes.len() + 1)?)
            .await,
        Err(StoreError::InvalidInput)
    ));
    assert!(matches!(
        store.connector_input_chunk(&source, &draft, 0).await,
        Err(StoreError::Conflict)
    ));
    Ok(())
}

pub(super) async fn replaced(
    store: &Store,
    f: &Fixture,
    first: &chapchoo_persistence::ConversationInputReceipt,
    snapshot: &SnapshotId,
    native: &NativeConversationId,
) -> Result<(), TestError> {
    let owner = f.owner.digest();
    assert!(matches!(
        store
            .conversation_input_delivery(
                &owner,
                &f.agent.id,
                &first.conversation_id,
                &first.id,
                f.now
            )
            .await,
        Err(StoreError::NotFound)
    ));
    let target = ConversationDeliveryTarget {
        snapshot: snapshot.clone(),
        native_session: native.clone(),
    };
    let draft = Forwarding {
        agent_id: &f.agent.id,
        conversation_id: &first.conversation_id,
        operation_id: &first.id,
        target: &target,
    };
    assert!(matches!(
        store
            .complete_conversation_input_forwarding(
                &f.source(&f.connector.digest()),
                &draft,
                &Outcome::Steered
            )
            .await,
        Err(StoreError::Conflict)
    ));
    Ok(())
}
