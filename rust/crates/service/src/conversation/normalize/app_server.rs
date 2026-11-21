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

/// Project into the existing permission question shape used by the browser and
/// answer validator. Only the original wire frame enters the raw capture lane.
pub(super) fn permission(
    raw: &str,
    position: &SourcePosition,
    native: &NativeConversationId,
) -> Result<Record, NativeNormalizationError> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Frame {
        thread_id: String,
        turn_id: Option<String>,
        server_name: String,
        mode: String,
        message: Option<String>,
        #[serde(rename = "_meta")]
        meta: Option<Meta>,
        requested_schema: Option<EmptyForm>,
    }
    #[derive(Deserialize)]
    struct Meta {
        codex_approval_kind: Option<String>,
        tool_name: Option<String>,
    }
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct EmptyForm {
        #[serde(rename = "type")]
        kind: String,
        properties: serde_json::Map<String, serde_json::Value>,
    }
    let frame: Frame =
        serde_json::from_str(raw).map_err(|_| NativeNormalizationError::InvalidEvent)?;
    if frame.thread_id != native.as_str() {
        return Err(NativeNormalizationError::WrongSession);
    }
    let key = |value: &str| {
        !value.is_empty() && value.len() <= 256 && value.bytes().all(|byte| byte.is_ascii_graphic())
    };
    let tool = frame
        .meta
        .filter(|meta| meta.codex_approval_kind.as_deref() == Some("mcp_tool_call"))
        .and_then(|meta| meta.tool_name)
        .filter(|name| key(name));
    let fields = if let Some(tool) = tool
        && frame.turn_id.as_deref().is_some_and(key)
        && frame.server_name == "chapchoo"
        && frame.mode == "form"
        && frame
            .requested_schema
            .is_some_and(|f| f.kind == "object" && f.properties.is_empty())
    {
        let question = schema::RequestPermissionRequest::new(
            native.as_str(),
            format!("Allow {tool}?"),
            vec![
                schema::PermissionOption::new(
                    "approve-once",
                    "Allow once",
                    schema::PermissionOptionKind::AllowOnce,
                ),
                schema::PermissionOption::new(
                    "decline",
                    "Decline",
                    schema::PermissionOptionKind::RejectOnce,
                ),
            ],
        )
        .description(frame.message.filter(|message| {
            message.len() <= 8192
                && !message.trim().is_empty()
                && !message
                    .chars()
                    .any(|ch| ch.is_control() && ch != '\n' && ch != '\t')
        }));
        vec![replace(Field::Request, &question)?]
    } else {
        vec![
            replace(Field::Title, &"Unsupported approval request")?,
            replace(Field::Status, &"unsupported")?,
        ]
    };
    Ok(record(
        Kind::Request,
        position.occurrence().as_str(),
        fields,
    ))
}

/// Keep a single foreground state record for both old and new capture lanes.
/// A native completed event is not automatically a successful continuation.
pub(super) fn turn(
    method: &str,
    raw: &str,
    native: &NativeConversationId,
) -> Result<Record, NativeNormalizationError> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Frame {
        thread_id: String,
        turn: Turn,
    }
    #[derive(Deserialize)]
    struct Turn {
        id: String,
        status: String,
    }
    let frame: Frame =
        serde_json::from_str(raw).map_err(|_| NativeNormalizationError::InvalidEvent)?;
    if frame.thread_id != native.as_str() {
        return Err(NativeNormalizationError::WrongSession);
    }
    if frame.turn.id.is_empty()
        || frame.turn.id.len() > 128
        || !frame.turn.id.bytes().all(|byte| byte.is_ascii_graphic())
    {
        return Err(NativeNormalizationError::InvalidEvent);
    }
    let state = match (method, frame.turn.status.as_str()) {
        ("turn/started", "inProgress") => serde_json::json!({"state": "running"}),
        ("turn/completed", "completed" | "failed" | "interrupted") => {
            serde_json::json!({"state": "idle"})
        }
        _ => return Err(NativeNormalizationError::InvalidEvent),
    };
    Ok(record(
        Kind::State,
        native.as_str(),
        vec![replace(Field::Foreground, &state)?],
    ))
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
