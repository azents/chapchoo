//! App Server message items adapt into the document's existing record fields.
//! The enclosing normalizer keeps the original wire bytes as a separate pair.
use super::*;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Frame<'a> {
    thread_id: &'a str,
    turn_id: &'a str,
    #[serde(borrow)]
    item: Option<Item<'a>>,
    item_id: Option<&'a str>,
    delta: Option<String>,
}

#[derive(Deserialize)]
struct Item<'a> {
    id: &'a str,
    #[serde(rename = "type")]
    kind: &'a str,
    text: Option<String>,
    content: Option<Vec<Part>>,
}

#[derive(Deserialize)]
struct Part {
    #[serde(rename = "type")]
    kind: String,
    text: Option<String>,
}

pub(super) fn message(
    method: &str,
    raw: &str,
    native: &NativeConversationId,
) -> Result<Option<Record>, NativeNormalizationError> {
    let params: Frame<'_> =
        serde_json::from_str(raw).map_err(|_| NativeNormalizationError::InvalidEvent)?;
    if params.thread_id != native.as_str() {
        return Err(NativeNormalizationError::WrongSession);
    }
    if method == "item/agentMessage/delta" {
        let id = key(
            params.turn_id,
            params
                .item_id
                .ok_or(NativeNormalizationError::InvalidEvent)?,
        )?;
        let delta = params.delta.ok_or(NativeNormalizationError::InvalidEvent)?;
        return Ok(Some(record(
            Kind::Assistant,
            &id,
            vec![append(Field::Content, &schema::ContentBlock::from(delta))?],
        )));
    }

    let item = params.item.ok_or(NativeNormalizationError::InvalidEvent)?;
    let (kind, fields) = match item.kind {
        "agentMessage" => {
            let text = item.text.ok_or(NativeNormalizationError::InvalidEvent)?;
            (
                Kind::Assistant,
                vec![replace(Field::Content, &schema::ContentBlock::from(text))?],
            )
        }
        "userMessage" => {
            let parts = item.content.ok_or(NativeNormalizationError::InvalidEvent)?;
            let mut bytes = Vec::new();
            for part in parts {
                // The message lane currently accepts only the text input sent
                // by Chapchoo. Do not infer text from another input type.
                if part.kind != "text" {
                    return Err(NativeNormalizationError::InvalidEvent);
                }
                let text = part.text.ok_or(NativeNormalizationError::InvalidEvent)?;
                bytes.extend(encoded(&schema::ContentBlock::from(text))?);
            }
            (
                Kind::User,
                vec![Update {
                    field: Field::Content,
                    change: Change::Replace(bytes),
                }],
            )
        }
        _ => return Ok(None),
    };
    Ok(Some(record(kind, &key(params.turn_id, item.id)?, fields)))
}

fn key(turn: &str, item: &str) -> Result<String, NativeNormalizationError> {
    let valid = |id: &str| {
        !id.is_empty() && id.len() <= 96 && id.bytes().all(|b| b.is_ascii_graphic() && b != b':')
    };
    if !valid(turn) || !valid(item) {
        return Err(NativeNormalizationError::InvalidEvent);
    }
    Ok(format!("app-server:{turn}:{item}"))
}
