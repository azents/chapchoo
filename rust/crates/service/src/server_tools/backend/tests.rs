use super::*;

#[test]
fn send_catalog_guides_work_closure_on_failure_or_cancellation()
-> Result<(), Box<dyn std::error::Error>> {
    let catalog: serde_json::Value =
        serde_json::from_slice(&ConversationToolBackend::catalog_definition()?)?;
    let description = catalog["tools"][0]["description"]
        .as_str()
        .ok_or("Missing send description")?;
    assert!(description.contains("When ending work because it failed or was cancelled"));
    assert!(description.contains("use finish to send a message or ignore to close silently"));
    Ok(())
}

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
    assert!(matches!(
        action("send", br#"{"action":"ignore"}"#),
        Ok(SendAction::Ignore)
    ));
    Ok(())
}
