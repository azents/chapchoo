use super::*;

#[test]
fn publication_arguments_preserve_text_and_reject_unbound_target_or_retry_fields()
-> Result<(), Box<dyn std::error::Error>> {
    let body = "  원문 **공백** 🙂\n다음 줄  ";
    let encoded = serde_json::to_vec(&json!({"text":body}))?;
    assert_eq!(
        text("send_message", &encoded)
            .map_err(|_| "Invalid fixture")?
            .as_str(),
        body
    );
    for input in [
        json!({"text":"hello","agent_id":"other"}),
        json!({"text":"hello","conversation_id":"other"}),
        json!({"text":"hello","request_id":"retry"}),
        json!({"text":"hello","finish":true}),
        json!({"text":"  "}),
        json!({"text":null}),
    ] {
        assert!(matches!(
            text("send_message", &serde_json::to_vec(&input)?),
            Err(ServerToolValidation::InvalidArguments)
        ));
    }
    assert!(matches!(
        text("other", &encoded),
        Err(ServerToolValidation::UnknownTool)
    ));
    Ok(())
}
