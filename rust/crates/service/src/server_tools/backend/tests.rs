use super::*;

#[test]
fn publication_arguments_preserve_text_and_reject_unbound_target_or_retry_fields()
-> Result<(), Box<dyn std::error::Error>> {
    let body = "  원문 **공백** 🙂\n다음 줄  ";
    let encoded = serde_json::to_vec(&json!({"action":"finish","message":body}))?;
    assert_eq!(
        action("send", &encoded)
            .map_err(|_| "Invalid fixture")?
            .message()
            .ok_or("Missing fixture message")?
            .as_str(),
        body
    );
    for input in [
        json!({"action":"finish","message":"hello","agent_id":"other"}),
        json!({"action":"finish","message":"hello","conversation_id":"other"}),
        json!({"action":"finish","message":"hello","request_id":"retry"}),
        json!({"message":"hello"}),
        json!({"action":"finish","message":"  "}),
        json!({"action":"request_input"}),
        json!({"action":"ignore","message":"must not publish"}),
        json!({"action":"continue","title":"missing tasks"}),
        json!({"action":"continue","tasks":[],"title":null}),
    ] {
        assert!(matches!(
            action("send", &serde_json::to_vec(&input)?),
            Err(ServerToolValidation::InvalidArguments)
        ));
    }
    assert!(matches!(
        action("other", &encoded),
        Err(ServerToolValidation::UnknownTool)
    ));
    assert!(matches!(
        action("send", br#"{"action":"continue","tasks":[]}"#),
        Ok(SendAction::Continue { title: None, tasks, message: None }) if tasks.is_empty()
    ));
    Ok(())
}
