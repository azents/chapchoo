use super::*;
use chapchoo_persistence::{ConversationInputAnchor, ConversationInputScope};
pub(super) async fn exercise(
    store: &Store,
    f: &Fixture,
    conversation: &str,
    target: &ConversationTarget,
    original: &str,
) -> Result<(), TestError> {
    let scope = ConversationInputScope {
        agent_id: &f.agent.id,
        conversation_id: conversation,
        target,
    };
    let owner = f.owner.digest();
    let inputs = store
        .conversation_inputs(&owner, &scope, None, 25, f.now)
        .await?;
    let input = inputs
        .iter()
        .find(|value| value.reference.request_id == "first-input")
        .ok_or("Missing retained first input")?;
    assert_eq!(input.bytes, i64::try_from(original.len())?);
    let oldest = inputs.last().ok_or("Missing input page")?;
    let anchor = ConversationInputAnchor {
        accepted_at: oldest.reference.accepted_at,
        request_id: oldest.reference.request_id.clone(),
    };
    assert!(
        store
            .conversation_inputs(&owner, &scope, Some(&anchor), 1, f.now)
            .await?
            .is_empty()
    );
    let mut bytes = Vec::new();
    for offset in 0..=u32::try_from(original.len())? {
        let chunk = store
            .conversation_input_text(
                &owner,
                &scope,
                &input.reference.request_id,
                offset,
                1,
                f.now,
            )
            .await?;
        assert_eq!(chunk.offset, offset);
        assert_eq!(chunk.total_bytes, input.bytes);
        bytes.extend(chunk.bytes);
    }
    assert_eq!(String::from_utf8(bytes)?, original);
    assert!(matches!(
        store
            .conversation_inputs(&f.other.digest(), &scope, None, 1, f.now)
            .await,
        Err(StoreError::NotFound)
    ));
    assert!(matches!(
        store
            .conversation_input_text(
                &f.other.digest(),
                &scope,
                &input.reference.request_id,
                0,
                1,
                f.now
            )
            .await,
        Err(StoreError::NotFound)
    ));
    assert!(matches!(
        store
            .conversation_input_text(
                &owner,
                &scope,
                &input.reference.request_id,
                u32::try_from(original.len())? + 1,
                1,
                f.now
            )
            .await,
        Err(StoreError::InvalidInput)
    ));
    assert!(matches!(
        store
            .conversation_input_text(&owner, &scope, &input.reference.request_id, 0, 32769, f.now)
            .await,
        Err(StoreError::InvalidInput)
    ));
    assert!(matches!(
        store
            .conversation_inputs(&owner, &scope, None, 26, f.now)
            .await,
        Err(StoreError::InvalidInput)
    ));
    let stale = ConversationInputScope {
        target: &ConversationTarget::Snapshot(SnapshotId::parse("not-the-current-source".into())?),
        ..scope
    };
    assert!(matches!(
        store
            .conversation_inputs(&owner, &stale, None, 1, f.now)
            .await,
        Err(StoreError::Conflict)
    ));
    assert!(matches!(
        store
            .conversation_input_text(&owner, &stale, &input.reference.request_id, 0, 1, f.now)
            .await,
        Err(StoreError::Conflict)
    ));
    Ok(())
}
