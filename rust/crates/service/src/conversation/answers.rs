//! Original question validation is separate from atomic live-answer admission.
use super::*;
use agent_client_protocol::v2::schema;
use chapchoo_crypto::SecretDigest;
use chapchoo_persistence::{
    AcceptConversationAnswer, ConversationAnswer as Answer, ConversationAnswerAdmission,
    ConversationAnswerQuestion, ConversationAnswerValue as Value, ConversationDeliveryTarget,
    ConversationFormAnswer as FormAnswer, ConversationRequestKind,
};

#[derive(Debug, Clone, Copy)]
pub enum NativeAnswerError {
    InvalidQuestion,
    InvalidAnswer,
}
impl fmt::Display for NativeAnswerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidQuestion => {
                "The original question is not supported or no longer matches the target"
            }
            Self::InvalidAnswer => "The submitted answer does not match the original question",
        })
    }
}
impl std::error::Error for NativeAnswerError {}

/// Canonical SDK response body; never a captured JSON-RPC envelope.
pub enum EncodedNativeAnswer {
    Permission(Vec<u8>),
    Form(Vec<u8>),
}
impl fmt::Debug for EncodedNativeAnswer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("EncodedNativeAnswer([REDACTED])")
    }
}
pub fn encode_native_answer(answer: Answer) -> Result<EncodedNativeAnswer, NativeAnswerError> {
    let encode = |value: Result<Vec<u8>, serde_json::Error>| {
        value.map_err(|_| NativeAnswerError::InvalidAnswer)
    };
    Ok(match answer {
        Answer::PermissionSelected { option_id } => EncodedNativeAnswer::Permission(encode(
            serde_json::to_vec(&schema::RequestPermissionResponse::new(
                schema::RequestPermissionOutcome::Selected(schema::SelectedPermissionOutcome::new(
                    option_id,
                )),
            )),
        )?),
        Answer::PermissionCancelled => EncodedNativeAnswer::Permission(encode(
            serde_json::to_vec(&schema::RequestPermissionResponse::new(
                schema::RequestPermissionOutcome::Cancelled(
                    schema::CancelledPermissionOutcome::new(),
                ),
            )),
        )?),
        Answer::Form(form) => {
            let action = match form {
                FormAnswer::Decline => schema::ElicitationAction::Decline,
                FormAnswer::Cancel => schema::ElicitationAction::Cancel,
                FormAnswer::Accept { content } => {
                    let content = content
                        .map(|values| {
                            values
                                .into_iter()
                                .map(|(name, value)| {
                                    let value = match value {
                                        Value::Text(value) => {
                                            schema::ElicitationContentValue::String(value)
                                        }
                                        Value::Integer(value) => {
                                            schema::ElicitationContentValue::Integer(value)
                                        }
                                        Value::Number(value) if value.is_finite() => {
                                            schema::ElicitationContentValue::Number(value)
                                        }
                                        Value::Number(_) => {
                                            return Err(NativeAnswerError::InvalidAnswer);
                                        }
                                        Value::Boolean(value) => {
                                            schema::ElicitationContentValue::Boolean(value)
                                        }
                                        Value::Strings(value) => {
                                            schema::ElicitationContentValue::StringArray(value)
                                        }
                                    };
                                    Ok((name, value))
                                })
                                .collect::<Result<std::collections::BTreeMap<_, _>, _>>()
                        })
                        .transpose()?;
                    schema::ElicitationAction::Accept(
                        schema::ElicitationAcceptAction::new().content(content),
                    )
                }
            };
            EncodedNativeAnswer::Form(encode(serde_json::to_vec(
                &schema::CreateElicitationResponse::new(action),
            ))?)
        }
    })
}

pub fn validate_conversation_answer(
    target: &ConversationDeliveryTarget,
    question: &ConversationAnswerQuestion,
    answer: &Answer,
) -> Result<(), NativeAnswerError> {
    let valid = match question.kind {
        ConversationRequestKind::Permission => {
            let request: schema::RequestPermissionRequest =
                serde_json::from_slice(&question.content)
                    .map_err(|_| NativeAnswerError::InvalidQuestion)?;
            if request.session_id.to_string() != target.native_session.as_str() {
                return Err(NativeAnswerError::InvalidQuestion);
            }
            let mut options = std::collections::BTreeSet::new();
            if request.options.is_empty()
                || request
                    .options
                    .iter()
                    .any(|option| !options.insert(option.option_id.to_string()))
            {
                return Err(NativeAnswerError::InvalidQuestion);
            }
            match answer {
                Answer::PermissionSelected { option_id } => request
                    .options
                    .iter()
                    .any(|option| option.option_id.to_string() == *option_id),
                Answer::PermissionCancelled => true,
                _ => false,
            }
        }
        ConversationRequestKind::Form => {
            let request: schema::CreateElicitationRequest =
                serde_json::from_slice(&question.content)
                    .map_err(|_| NativeAnswerError::InvalidQuestion)?;
            let schema::ElicitationMode::Form(form) = request.mode else {
                return Err(NativeAnswerError::InvalidQuestion);
            };
            let schema::ElicitationScope::Session(scope) = form.scope else {
                return Err(NativeAnswerError::InvalidQuestion);
            };
            if scope.session_id.to_string() != target.native_session.as_str()
                || normalize::validate_form(&form.requested_schema).is_err()
            {
                return Err(NativeAnswerError::InvalidQuestion);
            }
            match answer {
                Answer::Form(FormAnswer::Decline | FormAnswer::Cancel) => true,
                Answer::Form(FormAnswer::Accept { content }) => {
                    form.requested_schema.required.iter().flatten().all(|name| {
                        content
                            .as_ref()
                            .is_some_and(|values| values.contains_key(name))
                    }) && content
                        .iter()
                        .flat_map(|values| values.iter())
                        .all(|(name, value)| {
                            form.requested_schema
                                .properties
                                .get(name)
                                .is_some_and(|property| valid_value(property, value))
                        })
                }
                _ => false,
            }
        }
    };
    if valid {
        Ok(())
    } else {
        Err(NativeAnswerError::InvalidAnswer)
    }
}
fn valid_value(property: &schema::ElicitationPropertySchema, value: &Value) -> bool {
    use schema::ElicitationPropertySchema as P;
    if !normalize::supported(property) {
        return false;
    }
    match (property, value) {
        (P::String(p), Value::Text(value)) => {
            let length = value.chars().count();
            normalize::text_rule(p).is_ok_and(|rule| rule.accepts(value))
                && p.min_length.is_none_or(|min| length >= min as usize)
                && p.max_length.is_none_or(|max| length <= max as usize)
                && p.enum_values
                    .as_ref()
                    .is_none_or(|options| options.contains(value))
                && p.one_of
                    .as_ref()
                    .is_none_or(|options| options.iter().any(|option| &option.value == value))
        }
        (P::Number(p), Value::Number(value)) => {
            value.is_finite()
                && p.minimum.is_none_or(|min| *value >= min)
                && p.maximum.is_none_or(|max| *value <= max)
        }
        (P::Number(p), Value::Integer(value)) => {
            p.minimum.is_none_or(|bound| integer_minimum(*value, bound))
                && p.maximum.is_none_or(|bound| integer_maximum(*value, bound))
        }
        (P::Integer(p), Value::Integer(value)) => {
            p.minimum.is_none_or(|min| *value >= min) && p.maximum.is_none_or(|max| *value <= max)
        }
        (P::Integer(p), Value::Number(value)) if value.is_finite() && value.fract() == 0.0 => {
            format!("{value:.0}").parse::<i64>().is_ok_and(|value| {
                p.minimum.is_none_or(|min| value >= min) && p.maximum.is_none_or(|max| value <= max)
            })
        }
        (P::Boolean(_), Value::Boolean(_)) => true,
        (P::Array(p), Value::Strings(values)) => {
            let length = values.len() as u64;
            p.min_items.is_none_or(|min| length >= min)
                && p.max_items.is_none_or(|max| length <= max)
                && values.iter().all(|value| match &p.items {
                    schema::MultiSelectItems::String(items) => items.values.contains(value),
                    schema::MultiSelectItems::Titled(items) => {
                        items.options.iter().any(|option| &option.value == value)
                    }
                    _ => false,
                })
        }
        _ => false,
    }
}
fn integer_minimum(value: i64, bound: f64) -> bool {
    format!("{:.0}", bound.ceil())
        .parse::<i64>()
        .map_or(bound.is_sign_negative(), |min| value >= min)
}
fn integer_maximum(value: i64, bound: f64) -> bool {
    format!("{:.0}", bound.floor())
        .parse::<i64>()
        .map_or(bound.is_sign_positive(), |max| value <= max)
}
impl ConversationService {
    pub async fn accept_answer(
        &self,
        actor: &SecretDigest,
        draft: &AcceptConversationAnswer<'_>,
        now: i64,
    ) -> Result<ConversationAnswerAdmission, ServiceError> {
        let original = self
            .store
            .conversation_answer_question(actor, draft, now)
            .await?;
        validate_conversation_answer(draft.target, &original, draft.answer)
            .map_err(|_| ServiceError::domain(DomainReason::InvalidInput))?;
        Ok(self
            .store
            .accept_conversation_answer(actor, draft, now)
            .await?)
    }
}
#[cfg(test)]
mod tests;
