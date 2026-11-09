use super::*;
use serde_json::json;
type TestError = Box<dyn std::error::Error + Send + Sync>;
fn normalized(raw: &[u8]) -> Result<NormalizedCapture, NativeNormalizationError> {
    normalize_captured_event(
        &SnapshotId::parse("snapshot".into())
            .map_err(|_| NativeNormalizationError::InvalidEvent)?,
        &SourcePosition::new(
            1,
            chapchoo_domain::EventOccurrenceId::parse("occurrence".into())
                .map_err(|_| NativeNormalizationError::InvalidEvent)?,
        )
        .map_err(|_| NativeNormalizationError::InvalidEvent)?,
        &NativeConversationId::parse("native".into())
            .map_err(|_| NativeNormalizationError::InvalidEvent)?,
        raw,
    )
}
fn event(update: serde_json::Value) -> Result<NormalizedCapture, TestError> {
    Ok(normalized(&serde_json::to_vec(
        &json!({"method":"session/update","params":{"sessionId":"native","update":update}}),
    )?)?)
}
fn bytes(change: &Change) -> &[u8] {
    match change {
        Change::Append(b) | Change::Replace(b) => b,
    }
}
#[test]
fn utterances_use_typed_entity_and_role_without_merging_by_text() -> Result<(), TestError> {
    for (native, kind) in [
        ("user_message_chunk", Kind::User),
        ("agent_message_chunk", Kind::Assistant),
        ("agent_thought_chunk", Kind::Thought),
    ] {
        let result = event(
            json!({"sessionUpdate":native,"messageId":"same-id","content":{"type":"text","text":"identical"}}),
        )?;
        assert_eq!(result.records.len(), 1);
        let record = &result.records[0];
        assert_eq!(record.kind, kind);
        assert_eq!(record.native_id, "same-id");
        assert!(matches!(record.fields[0].change, Change::Append(_)));
        let content: schema::ContentBlock =
            serde_json::from_slice(bytes(&record.fields[0].change))?;
        assert_eq!(content, schema::ContentBlock::from("identical"));
    }
    let first = event(
        json!({"sessionUpdate":"agent_message_chunk","messageId":"z-first","content":{"type":"text","text":"same"}}),
    )?;
    let next = event(
        json!({"sessionUpdate":"agent_message_chunk","messageId":"a-second","content":{"type":"text","text":"same"}}),
    )?;
    assert_ne!(first.records[0].native_id, next.records[0].native_id);
    Ok(())
}
#[test]
fn full_content_replace_clear_and_omission_remain_different() -> Result<(), TestError> {
    let omitted = event(
        json!({"sessionUpdate":"agent_message","messageId":"id","_meta":{"origin":"replay"}}),
    )?;
    assert!(omitted.records[0].fields.is_empty());
    for content in [json!(null), json!([])] {
        let cleared =
            event(json!({"sessionUpdate":"agent_message","messageId":"id","content":content}))?;
        assert!(matches!(&cleared.records[0].fields[0].change,Change::Replace(b) if b.is_empty()));
    }
    let full = event(
        json!({"sessionUpdate":"agent_message","messageId":"id","content":[{"type":"text","text":"full"},{"type":"image","data":"aGk=","mimeType":"image/png"}]}),
    )?;
    let field = &full.records[0].fields[0];
    assert!(matches!(field.change, Change::Replace(_)));
    let blocks = bytes(&field.change)
        .split(|b| *b == b'\n')
        .filter(|b| !b.is_empty())
        .map(serde_json::from_slice::<schema::ContentBlock>)
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(blocks.len(), 2);
    assert!(
        matches!(&blocks[1],schema::ContentBlock::Image(image) if image.data=="aGk=" && image.mime_type.as_ref()=="image/png")
    );
    Ok(())
}
#[test]
fn tool_fields_are_independent_patches_not_assistant_prose() -> Result<(), TestError> {
    let result = event(
        json!({"sessionUpdate":"tool_call_update","toolCallId":"tool","title":"Read file","status":"in_progress","content":[{"type":"content","content":{"type":"text","text":"output"}}]}),
    )?;
    assert_eq!(result.records[0].kind, Kind::Tool);
    let fields = &result.records[0].fields;
    assert_eq!(fields.len(), 3);
    assert!(fields.iter().any(|f| f.field == Field::Title));
    assert!(fields.iter().any(|f| f.field == Field::Status));
    let later = event(
        json!({"sessionUpdate":"tool_call_update","toolCallId":"tool","status":"completed"}),
    )?;
    assert_eq!(later.records[0].fields.len(), 1);
    assert_eq!(later.records[0].fields[0].field, Field::Status);
    Ok(())
}
#[test]
fn tool_raw_input_and_output_are_independent_replacements_and_explicit_clears()
-> Result<(), TestError> {
    let first = event(json!({
        "sessionUpdate":"tool_call_update","toolCallId":"tool",
        "rawInput":{"path":"notes.txt","limit":9223372036854775807_i64},
        "rawOutput":{"lines":["first","second"]}
    }))?;
    let fields = &first.records[0].fields;
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].field, Field::ToolInput);
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(bytes(&fields[0].change))?,
        json!({"path":"notes.txt","limit":9223372036854775807_i64})
    );
    assert_eq!(fields[1].field, Field::ToolOutput);
    let clear = event(json!({
        "sessionUpdate":"tool_call_update","toolCallId":"tool","rawInput":null
    }))?;
    assert_eq!(clear.records[0].fields.len(), 1);
    assert_eq!(clear.records[0].fields[0].field, Field::ToolInput);
    assert!(
        matches!(&clear.records[0].fields[0].change, Change::Replace(value) if value.is_empty())
    );
    let status = event(json!({
        "sessionUpdate":"tool_call_update","toolCallId":"tool","status":"completed"
    }))?;
    assert_eq!(status.records[0].fields.len(), 1);
    assert_eq!(status.records[0].fields[0].field, Field::Status);
    Ok(())
}
#[test]
fn original_unknown_event_is_retained_without_fabricated_user_records() -> Result<(), TestError> {
    let raw=br#" {"method":"session/update","params":{"sessionId":"native","update":{"sessionUpdate":"_future","unknown":[1,false]}}} "#;
    let result = normalized(raw)?;
    assert_eq!(result.pair.raw, raw);
    assert!(result.records.is_empty());
    assert_eq!(result.pair.normalized, b"[]");
    assert!(!format!("{result:?}").contains("unknown"));
    Ok(())
}
#[test]
fn cross_session_and_credential_traffic_do_not_become_normalized_history() -> Result<(), TestError>
{
    assert!(matches!(normalized(br#"{"method":"session/update","params":{"sessionId":"other","update":{"sessionUpdate":"state_update","state":"idle"}}}"#),Err(NativeNormalizationError::WrongSession)));
    for raw in [
        br#"{"method":"auth/login","params":{"token":"secret-marker"}}"#.as_slice(),
        br#"{"id":3,"result":{"accessToken":"secret-marker"}}"#.as_slice(),
    ] {
        assert!(matches!(
            normalized(raw),
            Err(NativeNormalizationError::InvalidEvent)
        ));
    }
    Ok(())
}
#[test]
fn original_permissions_use_occurrence_identity_not_reusable_rpc_id() -> Result<(), TestError> {
    let request = json!({"id":1,"method":"session/request_permission","params":{"sessionId":"native","title":"Choose","options":[{"optionId":"allow","name":"Allow once","kind":"allow_once"},{"optionId":"reject","name":"Reject once","kind":"reject_once"}]}});
    let result = normalized(&serde_json::to_vec(&request)?)?;
    assert_eq!(result.records[0].kind, Kind::Request);
    assert_eq!(result.records[0].native_id, "occurrence");
    let saved: schema::RequestPermissionRequest =
        serde_json::from_slice(bytes(&result.records[0].fields[0].change))?;
    assert_eq!(saved.options.len(), 2);
    assert_eq!(saved.options[1].option_id.to_string(), "reject");
    Ok(())
}
#[test]
fn flat_forms_preserve_false_empty_and_optional_omission_contracts() -> Result<(), TestError> {
    let request = json!({"method":"elicitation/create","params":{"sessionId":"native","mode":"form","message":"Provide values","requestedSchema":{"type":"object","properties":{"enabled":{"type":"boolean","default":false},"text":{"type":"string","default":""},"optional":{"type":"number"}},"required":["enabled","text"]}}});
    let result = normalized(&serde_json::to_vec(&request)?)?;
    assert_eq!(result.records[0].native_id, "occurrence");
    let saved: schema::CreateElicitationRequest =
        serde_json::from_slice(bytes(&result.records[0].fields[0].change))?;
    let schema::ElicitationMode::Form(form) = saved.mode else {
        return Err("Expected form".into());
    };
    assert!(
        matches!(form.requested_schema.properties.get("enabled"),Some(schema::ElicitationPropertySchema::Boolean(field)) if field.default==Some(false))
    );
    assert!(
        matches!(form.requested_schema.properties.get("text"),Some(schema::ElicitationPropertySchema::String(field)) if field.default.as_deref()==Some(""))
    );
    assert_eq!(
        form.requested_schema
            .required
            .as_ref()
            .ok_or("Required")?
            .len(),
        2
    );
    Ok(())
}
#[test]
fn unsupported_required_field_and_url_modes_are_errors_not_user_refusal() -> Result<(), TestError> {
    for params in [
        json!({"sessionId":"native","mode":"form","message":"Unsupported","requestedSchema":{"type":"object","properties":{"nested":{"type":"object"}},"required":["nested"]}}),
        json!({"sessionId":"native","mode":"url","message":"URL","url":"https://example.invalid","elicitationId":"id"}),
    ] {
        let result = normalized(&serde_json::to_vec(
            &json!({"method":"elicitation/create","params":params}),
        )?)?;
        assert_eq!(result.records.len(), 1);
        assert_eq!(result.records[0].kind, Kind::Request);
        assert!(result.records[0].fields.iter().any(|f| {
            f.field == Field::Status
                && serde_json::from_slice::<String>(bytes(&f.change))
                    .is_ok_and(|s| s == "unsupported")
        }));
        assert!(
            !result.records[0]
                .fields
                .iter()
                .any(|f| f.field == Field::Request)
        );
    }
    assert!(normalized(&serde_json::to_vec(&json!({"method":"elicitation/create","params":{"requestId":1,"mode":"form","message":"Credential scope","requestedSchema":{"type":"object","properties":{}}}}))?).is_err());
    Ok(())
}
