//! Validate exactly supported flat values before consuming a native responder.
use super::*;

pub(super) fn supported(property: &schema::ElicitationPropertySchema) -> bool {
    use schema::ElicitationPropertySchema as P;
    match property {
        P::String(p) => {
            p.pattern.is_none()
                && p.format.is_none()
                && p.min_length
                    .zip(p.max_length)
                    .is_none_or(|(min, max)| min <= max)
        }
        P::Number(p) => {
            p.minimum.is_none_or(f64::is_finite)
                && p.maximum.is_none_or(f64::is_finite)
                && p.minimum.zip(p.maximum).is_none_or(|(min, max)| min <= max)
        }
        P::Integer(p) => p.minimum.zip(p.maximum).is_none_or(|(min, max)| min <= max),
        P::Boolean(_) => true,
        P::Array(p) => {
            matches!(
                p.items,
                schema::MultiSelectItems::String(_) | schema::MultiSelectItems::Titled(_)
            ) && p
                .min_items
                .zip(p.max_items)
                .is_none_or(|(min, max)| min <= max)
        }
        _ => false,
    }
}
fn value(
    property: &schema::ElicitationPropertySchema,
    value: &schema::ElicitationContentValue,
) -> bool {
    use schema::{ElicitationContentValue as V, ElicitationPropertySchema as P};
    if !supported(property) {
        return false;
    }
    match (property, value) {
        (P::String(p), V::String(value)) => {
            let length = value.chars().count();
            p.min_length.is_none_or(|min| length >= min as usize)
                && p.max_length.is_none_or(|max| length <= max as usize)
                && p.enum_values
                    .as_ref()
                    .is_none_or(|options| options.contains(value))
                && p.one_of
                    .as_ref()
                    .is_none_or(|options| options.iter().any(|option| &option.value == value))
        }
        (P::Number(p), V::Number(value)) => number(p, *value),
        (P::Number(p), V::Integer(value)) => {
            // Keep large exact integers exact instead of rounding them into a
            // floating-point neighbour before testing the schema boundary.
            p.minimum.is_none_or(|bound| integer_minimum(*value, bound))
                && p.maximum.is_none_or(|bound| integer_maximum(*value, bound))
        }
        (P::Integer(p), V::Integer(value)) => {
            p.minimum.is_none_or(|min| *value >= min) && p.maximum.is_none_or(|max| *value <= max)
        }
        (P::Integer(p), V::Number(value)) if value.is_finite() && value.fract() == 0.0 => {
            format!("{value:.0}").parse::<i64>().is_ok_and(|value| {
                p.minimum.is_none_or(|min| value >= min) && p.maximum.is_none_or(|max| value <= max)
            })
        }
        (P::Boolean(_), V::Boolean(_)) => true,
        (P::Array(p), V::StringArray(values)) => {
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
fn number(p: &schema::NumberPropertySchema, value: f64) -> bool {
    value.is_finite()
        && p.minimum.is_none_or(|min| value >= min)
        && p.maximum.is_none_or(|max| value <= max)
}
pub(super) fn response(
    form: &schema::ElicitationFormMode,
    response: &schema::CreateElicitationResponse,
) -> bool {
    match &response.action {
        schema::ElicitationAction::Decline | schema::ElicitationAction::Cancel => true,
        schema::ElicitationAction::Accept(answer) => {
            let present = |name: &str| {
                answer
                    .content
                    .as_ref()
                    .is_some_and(|content| content.contains_key(name))
            };
            form.requested_schema
                .required
                .iter()
                .flatten()
                .all(|name| present(name))
                && answer
                    .content
                    .iter()
                    .flat_map(|content| content.iter())
                    .all(|(name, answer)| {
                        form.requested_schema
                            .properties
                            .get(name)
                            .is_some_and(|property| value(property, answer))
                    })
        }
        _ => false,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    type TestError = Box<dyn std::error::Error>;
    fn form() -> Result<schema::ElicitationFormMode, TestError> {
        Ok(serde_json::from_value(
            json!({"sessionId":"session","requestedSchema":{"type":"object","properties":{
            "enabled":{"type":"boolean"},"text":{"type":"string","minLength":0,"maxLength":2},
            "choice":{"type":"string","oneOf":[{"const":"a","title":"A"},{"const":"b","title":"B"}]},
            "count":{"type":"integer","minimum":0,"maximum":5},"ratio":{"type":"number","minimum":0,"maximum":1},
            "many":{"type":"array","items":{"type":"string","enum":["a","b"]},"maxItems":2},
            "unknown":{"type":"object"}
        },"required":["enabled","text"]}}),
        )?)
    }
    fn answer(value: serde_json::Value) -> Result<schema::CreateElicitationResponse, TestError> {
        Ok(serde_json::from_value(value)?)
    }
    #[test]
    fn required_false_empty_optional_omission_and_typed_constraints() -> Result<(), TestError> {
        let form = form()?;
        for content in [
            json!({"enabled":false,"text":""}),
            json!({"enabled":true,"text":"한글","choice":"b","count":5,"ratio":0.5,"many":[]}),
        ] {
            assert!(response(
                &form,
                &answer(json!({"action":"accept","content":content}))?
            ));
        }
        for content in [
            json!({"enabled":false}),
            json!({"text":""}),
            json!({"enabled":"false","text":""}),
            json!({"enabled":false,"text":"long"}),
            json!({"enabled":false,"text":"","extra":false}),
            json!({"enabled":false,"text":"","choice":"c"}),
            json!({"enabled":false,"text":"","count":1.5}),
            json!({"enabled":false,"text":"","count":6}),
            json!({"enabled":false,"text":"","ratio":2}),
            json!({"enabled":false,"text":"","many":["c"]}),
            json!({"enabled":false,"text":"","many":["a","b","a"]}),
            json!({"enabled":false,"text":"","unknown":{}}),
        ] {
            assert!(!response(
                &form,
                &answer(json!({"action":"accept","content":content}))?
            ));
        }
        assert!(!response(&form, &answer(json!({"action":"accept"}))?));
        assert!(response(&form, &answer(json!({"action":"decline"}))?));
        assert!(response(&form, &answer(json!({"action":"cancel"}))?));
        Ok(())
    }
    #[test]
    fn unsupported_constraints_do_not_masquerade_as_validated_fields() -> Result<(), TestError> {
        for property in [
            json!({"type":"string","pattern":"^[a-z]$"}),
            json!({"type":"string","format":"email"}),
            json!({"type":"array","items":{"type":"object"}}),
            json!({"type":"number","minimum":2,"maximum":1}),
        ] {
            assert!(!supported(&serde_json::from_value(property)?));
        }
        Ok(())
    }
    #[test]
    fn number_bounds_do_not_round_exact_integer_answers() {
        assert!(!integer_minimum(
            9_007_199_254_740_991,
            9_007_199_254_740_992.0
        ));
        assert!(!integer_maximum(
            9_007_199_254_740_993,
            9_007_199_254_740_992.0
        ));
        assert!(integer_maximum(i64::MAX, 9_223_372_036_854_775_808.0));
        assert!(!integer_minimum(i64::MAX, 9_223_372_036_854_775_808.0));
        assert!(!integer_minimum(0, 0.25));
        assert!(!integer_maximum(1, 0.75));
    }
}
