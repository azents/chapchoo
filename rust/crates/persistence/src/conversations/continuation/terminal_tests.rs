use super::*;
use serde_json::json;

#[test]
fn app_server_requires_normal_completion_of_the_exact_thread_and_turn()
-> Result<(), Box<dyn std::error::Error>> {
    let target = ConversationDeliveryTarget {
        snapshot: SnapshotId::parse("snapshot".to_owned())?,
        native_session: NativeConversationId::parse("retained-thread".to_owned())?,
    };
    let event = json!({
        "method": "turn/completed",
        "params": {
            "threadId": "retained-thread",
            "turn": { "id": "known-turn", "status": "completed" },
        }
    });
    verify_terminal(&serde_json::to_vec(&event)?, &target, "known-turn")?;
    assert!(verify_terminal(&serde_json::to_vec(&event)?, &target, "other-turn").is_err());
    for (pointer, value) in [
        ("/params/threadId", json!("other-thread")),
        ("/params/turn/status", json!("interrupted")),
        ("/params/turn/status", json!("failed")),
        ("/method", json!("turn/started")),
    ] {
        let mut other = event.clone();
        *other.pointer_mut(pointer).ok_or("Missing fixture path")? = value;
        assert!(verify_terminal(&serde_json::to_vec(&other)?, &target, "known-turn").is_err());
    }
    for id in [json!(null), json!("reply")] {
        let mut not_notification = event.clone();
        not_notification["id"] = id;
        assert!(
            verify_terminal(
                &serde_json::to_vec(&not_notification)?,
                &target,
                "known-turn"
            )
            .is_err()
        );
    }

    let retained = br#"{"method":"session/update","params":{"sessionId":"retained-thread","update":{"sessionUpdate":"state_update","state":"idle","stopReason":"end_turn"}}}"#;
    verify_terminal(retained, &target, "retained-work")?;
    Ok(())
}
