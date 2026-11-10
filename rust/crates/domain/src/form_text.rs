//! Offline bounded JSON-Schema string constraints shared by admission and native response.
use std::fmt;

#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("Form text constraint is invalid")]
pub struct InvalidFormTextRule;

pub struct FormTextRule(jsonschema::Validator);
impl fmt::Debug for FormTextRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("FormTextRule([REDACTED])")
    }
}
impl FormTextRule {
    pub fn parse(pattern: Option<&str>, format: Option<&str>) -> Result<Self, InvalidFormTextRule> {
        // Ordinary forms must not become a credential-collection surface.
        if format == Some("password") {
            return Err(InvalidFormTextRule);
        }
        let mut schema = serde_json::json!({"type":"string"});
        if let Some(pattern) = pattern {
            schema["pattern"] = pattern.into();
        }
        // Unknown producer formats remain ordinary text, not invented input contracts.
        if let Some(format @ ("email" | "uri" | "date" | "date-time")) = format {
            schema["format"] = format.into();
        }
        let validator = jsonschema::options()
            .offline()
            .should_validate_formats(true)
            .should_ignore_unknown_formats(true)
            .with_pattern_options(
                jsonschema::PatternOptions::fancy_regex()
                    .backtrack_limit(1_000_000)
                    .size_limit(1024 * 1024)
                    .dfa_size_limit(1024 * 1024),
            )
            .build(&schema)
            .map_err(|_| InvalidFormTextRule)?;
        Ok(Self(validator))
    }
    pub fn accepts(&self, value: &str) -> bool {
        self.0.is_valid(&serde_json::Value::String(value.into()))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn actual_constraints_preserve_the_original_value_and_unknown_formats()
    -> Result<(), InvalidFormTextRule> {
        for (format, valid, invalid) in [
            ("email", "person@example.com", " person@example.com "),
            ("uri", "https://example.com/path", "relative-path"),
            ("date", "2024-02-29", "2025-02-29"),
            (
                "date-time",
                "2026-11-09T10:15:00+09:00",
                "2026-11-09T10:15:00",
            ),
        ] {
            let rule = FormTextRule::parse(None, Some(format))?;
            assert!(rule.accepts(valid), "Expected valid {format}");
            assert!(!rule.accepts(invalid), "Expected invalid {format}");
        }
        assert!(FormTextRule::parse(None, Some("future-format"))?.accepts(" unchanged "));
        assert!(FormTextRule::parse(None, Some("password")).is_err());
        Ok(())
    }
    #[test]
    fn ecma_patterns_are_validated_instead_of_being_ignored() -> Result<(), InvalidFormTextRule> {
        let rule = FormTextRule::parse(Some(r"^(?=.{3}$)[a-z]+$"), None)?;
        assert!(rule.accepts("abc"));
        assert!(!rule.accepts("ab"));
        assert!(!rule.accepts("ABC"));
        assert!(FormTextRule::parse(Some("["), None).is_err());
        let unicode = FormTextRule::parse(Some(r"^\uAC00.$"), None)?;
        assert!(unicode.accepts("가🙂"));
        assert!(!unicode.accepts("나🙂"));
        Ok(())
    }
}
