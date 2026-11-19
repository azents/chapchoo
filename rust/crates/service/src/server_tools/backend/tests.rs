use super::*;

#[test]
fn send_catalog_declares_one_flat_typed_input_object() -> Result<(), Box<dyn std::error::Error>> {
    let catalog: serde_json::Value =
        serde_json::from_slice(&ConversationToolBackend::catalog_definition()?)?;
    let input = catalog["tools"][0]["inputSchema"]
        .as_object()
        .ok_or("Missing send input schema")?;
    let mut keys = input.keys().map(String::as_str).collect::<Vec<_>>();
    keys.sort_unstable();
    assert_eq!(
        keys,
        ["additionalProperties", "properties", "required", "type"]
    );
    assert_eq!(input["type"], "object");
    assert_eq!(input["required"], json!(["action"]));
    assert_eq!(input["additionalProperties"], false);
    let fields = &input["properties"];
    assert_eq!(
        fields["action"]["enum"],
        json!(["continue", "request_input", "finish", "ignore"])
    );
    assert_eq!(fields["title"]["type"], "string");
    assert_eq!(fields["message"]["type"], "string");
    assert_eq!(fields["tasks"]["type"], "array");
    assert_eq!(
        fields["tasks"]["items"]["required"],
        json!(["text", "status"])
    );
    assert_eq!(
        fields["tasks"]["items"]["properties"]["status"]["enum"],
        json!(["pending", "in_progress", "completed"])
    );
    Ok(())
}

#[test]
fn send_validates_action_specific_fields_in_server_logic() {
    let valid = [
        r#"{"action":"continue","title":"Review","tasks":[{"text":"Check","status":"pending"}],"message":"Working"}"#,
        r#"{"action":"continue","tasks":[]}"#,
        r#"{"action":"request_input","message":"Which source?"}"#,
        r#"{"action":"finish","message":"Done"}"#,
        r#"{"action":"ignore"}"#,
    ];
    for payload in valid {
        assert!(action("send", payload.as_bytes()).is_ok(), "{payload}");
    }

    let invalid = [
        r#"{"action":"continue","title":"Review"}"#,
        r#"{"action":"continue","tasks":"Check"}"#,
        r#"{"action":"continue","tasks":[{"text":"Check"}]}"#,
        r#"{"action":"continue","tasks":[{"text":"Check","status":"unknown"}]}"#,
        r#"{"action":"continue","tasks":[],"title":null}"#,
        r#"{"action":"continue","tasks":[],"message":null}"#,
        r#"{"action":"request_input"}"#,
        r#"{"action":"request_input","message":"Which source?","title":"Review"}"#,
        r#"{"action":"finish"}"#,
        r#"{"action":"finish","message":"Done","tasks":[]}"#,
        r#"{"action":"ignore","title":"Review"}"#,
        r#"{"action":"ignore","tasks":[]}"#,
        r#"{"action":"ignore","message":"Done"}"#,
    ];
    for payload in invalid {
        assert!(
            matches!(
                action("send", payload.as_bytes()),
                Err(ServerToolValidation::InvalidArguments)
            ),
            "{payload}"
        );
    }
}

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
