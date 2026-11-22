//! Text rendering for system-generated inputs sent in the native user role.

use crate::AppError;
use chapchoo_domain::ConversationText;

pub(super) fn render(metadata: &[(&str, &str)], body: &str) -> Result<ConversationText, AppError> {
    let mut text = String::from("[System-generated input]\n");
    for &(key, value) in metadata {
        const LINE_BREAKS: &[char] = &[
            '\n', '\r', '\u{000b}', '\u{000c}', '\u{0085}', '\u{2028}', '\u{2029}',
        ];
        if key.trim().is_empty()
            || key.contains(':')
            || key.contains(LINE_BREAKS)
            || value.contains(LINE_BREAKS)
        {
            return Err("Invalid system-generated input metadata".into());
        }
        text.push_str(key);
        text.push_str(": ");
        text.push_str(value);
        text.push('\n');
    }
    text.push('\n');
    text.push_str(body);
    Ok(ConversationText::parse(text)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_metadata_lines_and_preserves_the_body() -> Result<(), AppError> {
        let body = "  \"quoted\"\r\n\nkind: part of the body\n[System-generated input]\n끝\n";
        let input = render(&[("kind", "example"), ("channel-id", "123")], body)?;
        assert_eq!(
            input.as_str().split_once("\n\n"),
            Some((
                "[System-generated input]\nkind: example\nchannel-id: 123",
                body
            ))
        );
        Ok(())
    }

    #[test]
    fn rejects_metadata_that_would_break_line_boundaries() {
        for fields in [
            [("bad:key", "value")],
            [("kind", "one\nother: two")],
            [("kind", "one\r\ntwo")],
            [("kind", "one\u{2028}two")],
        ] {
            assert!(render(&fields, "body").is_err());
        }
    }
}
