use super::*;
use chapchoo_domain::{EventOccurrenceId, NativeConversationId};
use serde_json::json;
use std::collections::BTreeMap;
type TestError = Box<dyn std::error::Error>;
#[test]
fn native_encoding_uses_sdk_types_and_preserves_exact_answer_values() -> Result<(), TestError> {
    let answer = Answer::Form(FormAnswer::Accept {
        content: Some(BTreeMap::from([
            ("enabled".into(), Value::Boolean(false)),
            ("text".into(), Value::Text(String::new())),
            ("large".into(), Value::Integer(i64::MAX)),
            ("choices".into(), Value::Strings(Vec::new())),
        ])),
    });
    let EncodedNativeAnswer::Form(bytes) = encode_native_answer(answer)? else {
        return Err("Wrong encoded answer kind".into());
    };
    let response: schema::CreateElicitationResponse = serde_json::from_slice(&bytes)?;
    let schema::ElicitationAction::Accept(content) = response.action else {
        return Err("Wrong action".into());
    };
    let content = content.content.ok_or("Missing values")?;
    assert_eq!(
        content.get("enabled"),
        Some(&schema::ElicitationContentValue::Boolean(false))
    );
    assert_eq!(
        content.get("large"),
        Some(&schema::ElicitationContentValue::Integer(i64::MAX))
    );
    assert_eq!(
        content.get("text"),
        Some(&schema::ElicitationContentValue::String(String::new()))
    );
    assert_eq!(
        content.get("choices"),
        Some(&schema::ElicitationContentValue::StringArray(Vec::new()))
    );
    let EncodedNativeAnswer::Permission(bytes) =
        encode_native_answer(Answer::PermissionSelected {
            option_id: "deny-once".into(),
        })?
    else {
        return Err("Wrong permission kind".into());
    };
    let response: schema::RequestPermissionResponse = serde_json::from_slice(&bytes)?;
    assert!(
        matches!(response.outcome, schema::RequestPermissionOutcome::Selected(value) if value.option_id.to_string() == "deny-once")
    );
    assert!(
        encode_native_answer(Answer::Form(FormAnswer::Accept {
            content: Some(BTreeMap::from([("number".into(), Value::Number(f64::NAN))])),
        }))
        .is_err()
    );
    Ok(())
}
fn target() -> Result<ConversationDeliveryTarget, TestError> {
    Ok(ConversationDeliveryTarget {
        snapshot: SnapshotId::parse("source".into())?,
        native_session: NativeConversationId::parse("native".into())?,
    })
}
fn question(
    kind: ConversationRequestKind,
    content: serde_json::Value,
) -> Result<ConversationAnswerQuestion, TestError> {
    Ok(ConversationAnswerQuestion {
        kind,
        occurrence: SourcePosition::new(1, EventOccurrenceId::parse("occurrence".into())?)?,
        content: serde_json::to_vec(&content)?,
    })
}
#[test]
fn permission_requires_the_original_session_kind_and_offered_option() -> Result<(), TestError> {
    let target = target()?;
    let original = question(
        ConversationRequestKind::Permission,
        json!({"sessionId":"native","title":"Original question","options":[{"optionId":"allow","name":"Allow once","kind":"allow_once"}]}),
    )?;
    validate_conversation_answer(
        &target,
        &original,
        &Answer::PermissionSelected {
            option_id: "allow".into(),
        },
    )?;
    validate_conversation_answer(&target, &original, &Answer::PermissionCancelled)?;
    assert!(
        validate_conversation_answer(
            &target,
            &original,
            &Answer::PermissionSelected {
                option_id: "not-offered".into()
            }
        )
        .is_err()
    );
    assert!(
        validate_conversation_answer(&target, &original, &Answer::Form(FormAnswer::Cancel))
            .is_err()
    );
    let other = ConversationDeliveryTarget {
        native_session: NativeConversationId::parse("another".into())?,
        ..target
    };
    assert!(validate_conversation_answer(&other, &original, &Answer::PermissionCancelled).is_err());
    Ok(())
}
#[test]
fn form_preserves_valid_empty_false_omission_and_rejects_changed_values() -> Result<(), TestError> {
    let target = target()?;
    let original = question(
        ConversationRequestKind::Form,
        json!({"sessionId":"native","mode":"form","message":"Original form","requestedSchema":{
            "type":"object","properties":{"enabled":{"type":"boolean"},"text":{"type":"string","maxLength":3},
            "choice":{"type":"string","enum":["a","b"]},"many":{"type":"array","items":{"type":"string","enum":["a","b"]}},
            "number":{"type":"number","maximum":9007199254740992.0},"optional":{"type":"object"}},"required":["enabled","text"]
        }}),
    )?;
    let valid = BTreeMap::from([
        ("enabled".into(), Value::Boolean(false)),
        ("text".into(), Value::Text(String::new())),
        ("many".into(), Value::Strings(Vec::new())),
    ]);
    validate_conversation_answer(
        &target,
        &original,
        &Answer::Form(FormAnswer::Accept {
            content: Some(valid.clone()),
        }),
    )?;
    for (key, value) in [
        ("enabled", Value::Text("false".into())),
        ("text", Value::Text("long text".into())),
        ("choice", Value::Text("not-offered".into())),
        ("many", Value::Strings(vec!["unknown".into()])),
        ("optional", Value::Text("unsupported".into())),
        ("number", Value::Integer(9_007_199_254_740_993)),
    ] {
        let mut changed = valid.clone();
        changed.insert(key.into(), value);
        assert!(
            validate_conversation_answer(
                &target,
                &original,
                &Answer::Form(FormAnswer::Accept {
                    content: Some(changed)
                })
            )
            .is_err()
        );
    }
    assert!(
        validate_conversation_answer(
            &target,
            &original,
            &Answer::Form(FormAnswer::Accept { content: None })
        )
        .is_err()
    );
    validate_conversation_answer(&target, &original, &Answer::Form(FormAnswer::Decline))?;
    validate_conversation_answer(&target, &original, &Answer::Form(FormAnswer::Cancel))?;
    let url = question(
        ConversationRequestKind::Form,
        json!({"sessionId":"native","mode":"url","message":"Not supported","elicitationId":"request","url":"https://example.com"}),
    )?;
    assert!(
        validate_conversation_answer(&target, &url, &Answer::Form(FormAnswer::Cancel)).is_err()
    );
    Ok(())
}
