//! App Server message items adapt into the document's existing record fields.
//! The enclosing normalizer keeps the original wire bytes as a separate pair.
use super::*;
use chapchoo_domain::local_mcp;

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
    server: Option<&'a str>,
    tool: Option<&'a str>,
    status: Option<&'a str>,
    #[serde(borrow)]
    arguments: Option<&'a serde_json::value::RawValue>,
    #[serde(borrow)]
    result: Option<&'a serde_json::value::RawValue>,
    #[serde(borrow)]
    error: Option<&'a serde_json::value::RawValue>,
    #[serde(borrow)]
    changes: Option<&'a serde_json::value::RawValue>,
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
        requested_schema: Option<serde_json::Value>,
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
    if frame
        .meta
        .as_ref()
        .is_none_or(|meta| meta.codex_approval_kind.is_none())
        && frame.turn_id.as_deref().is_some_and(key)
        && frame.server_name == "chapchoo"
        && frame.mode == "form"
        && let Some(message) = frame
            .message
            .as_ref()
            .filter(|text| !text.is_empty() && text.len() <= 8192)
        && let Some(schema) = frame.requested_schema.as_ref()
        && let Ok(schema) = serde_json::from_value::<schema::ElicitationSchema>(schema.clone())
        && !schema.properties.is_empty()
        && validate_form(&schema).is_ok()
    {
        let form = schema::ElicitationFormMode::new(
            schema::ElicitationSessionScope::new(native.as_str()),
            schema,
        );
        let request = schema::CreateElicitationRequest::new(form, message);
        return Ok(record(
            Kind::Request,
            position.occurrence().as_str(),
            vec![replace(Field::Request, &request)?],
        ));
    }
    let tool = frame
        .meta
        .filter(|meta| meta.codex_approval_kind.as_deref() == Some("mcp_tool_call"))
        .and_then(|meta| meta.tool_name)
        .filter(|name| key(name));
    let fields = if let Some(tool) = tool
        && frame.turn_id.as_deref().is_some_and(key)
        && (frame.server_name == "chapchoo" || local_mcp::is_external_name(&frame.server_name))
        && frame.mode == "form"
        && frame
            .requested_schema
            .and_then(|schema| serde_json::from_value::<EmptyForm>(schema).ok())
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

/// Render an already-admitted native approval at its captured occurrence.
/// The physical owner independently validates the options before answering.
pub(super) fn item_permission(
    method: &str,
    raw: &str,
    position: &SourcePosition,
    native: &NativeConversationId,
) -> Result<Record, NativeNormalizationError> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct ItemApproval {
        thread_id: String,
        turn_id: String,
        item_id: String,
        started_at_ms: i64,
        reason: Option<String>,
        command: Option<String>,
        cwd: Option<String>,
        network_approval_context: Option<NetworkApproval>,
        available_decisions: Option<Vec<serde_json::Value>>,
        additional_permissions: Option<serde_json::Value>,
        grant_root: Option<String>,
    }
    #[derive(Deserialize)]
    struct NetworkApproval {
        host: String,
        protocol: String,
    }
    let item: ItemApproval =
        serde_json::from_str(raw).map_err(|_| NativeNormalizationError::InvalidEvent)?;
    if item.thread_id != native.as_str() {
        return Err(NativeNormalizationError::WrongSession);
    }
    let valid = |key: &str| {
        !key.is_empty() && key.len() <= 256 && key.bytes().all(|b| b.is_ascii_graphic())
    };
    if !valid(&item.turn_id) || !valid(&item.item_id) || item.started_at_ms < 0 {
        return Err(NativeNormalizationError::InvalidEvent);
    }
    let visible = |text: &str| {
        text.len() <= 8192
            && !text.trim().is_empty()
            && !text
                .chars()
                .any(|ch| ch.is_control() && ch != '\n' && ch != '\t')
    };
    use schema::PermissionOptionKind::{AllowAlways, AllowOnce, RejectOnce};
    let decisions = [
        ("accept", "approve-once", "Allow once", AllowOnce),
        (
            "acceptForSession",
            "approve-for-session",
            "Allow for this session",
            AllowAlways,
        ),
        ("decline", "decline", "Decline and continue", RejectOnce),
        ("cancel", "cancel", "Decline and stop the turn", RejectOnce),
    ];
    let (title, mut lines, offered) = match method {
        "item/commandExecution/requestApproval" => {
            let is_network = item.network_approval_context.is_some();
            let context = match (item.command, item.cwd, item.network_approval_context) {
                (Some(command), Some(cwd), None) if visible(&command) && visible(&cwd) => {
                    format!("Directory: {cwd}\nCommand: {command}")
                }
                (None, None, Some(network))
                    if visible(&network.host) && visible(&network.protocol) =>
                {
                    format!(
                        "Network destination: {} ({})",
                        network.host, network.protocol
                    )
                }
                _ => return Err(NativeNormalizationError::UnsupportedRequest),
            };
            let offered = item.available_decisions.unwrap_or_else(|| {
                if !is_network && item.additional_permissions.is_some() {
                    vec![serde_json::json!("accept"), serde_json::json!("cancel")]
                } else {
                    vec![
                        serde_json::json!("accept"),
                        serde_json::json!("acceptForSession"),
                        serde_json::json!("decline"),
                        serde_json::json!("cancel"),
                    ]
                }
            });
            if !offered
                .iter()
                .any(|decision| matches!(decision.as_str(), Some("decline" | "cancel")))
            {
                return Err(NativeNormalizationError::UnsupportedRequest);
            }
            ("Allow command execution?", vec![context], offered)
        }
        "item/fileChange/requestApproval" => {
            let scope = item
                .grant_root
                .filter(|root| visible(root))
                .map(|root| format!("Requested root: {root}"));
            let mut lines =
                vec!["Review the affected files in the file-change activity.".to_owned()];
            lines.extend(scope);
            (
                "Allow file changes?",
                lines,
                vec![
                    serde_json::json!("accept"),
                    serde_json::json!("acceptForSession"),
                    serde_json::json!("cancel"),
                ],
            )
        }
        _ => return Err(NativeNormalizationError::UnsupportedRequest),
    };
    if let Some(reason) = item.reason.filter(|reason| visible(reason)) {
        lines.push(reason);
    }
    let options = decisions
        .into_iter()
        .filter(|(native, _, _, _)| {
            offered
                .iter()
                .any(|choice| choice.as_str() == Some(*native))
        })
        .map(|(_, id, title, kind)| schema::PermissionOption::new(id, title, kind))
        .collect();
    let question = schema::RequestPermissionRequest::new(native.as_str(), title, options)
        .description((!lines.is_empty()).then(|| lines.join("\n")));
    Ok(record(
        Kind::Request,
        position.occurrence().as_str(),
        vec![replace(Field::Request, &question)?],
    ))
}

/// Project the admitted tool questions with the same captured field IDs.
pub(super) fn user_input(
    raw: &str,
    position: &SourcePosition,
    native: &NativeConversationId,
) -> Result<Record, NativeNormalizationError> {
    use std::collections::BTreeSet;
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Input {
        thread_id: String,
        turn_id: String,
        item_id: String,
        questions: Vec<Question>,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Question {
        id: String,
        header: String,
        question: String,
        options: Option<Vec<Choice>>,
        #[serde(default)]
        is_other: bool,
        #[serde(default)]
        is_secret: bool,
    }
    #[derive(Deserialize)]
    struct Choice {
        label: String,
        description: String,
    }
    use serde_json::json;
    let input: Input =
        serde_json::from_str(raw).map_err(|_| NativeNormalizationError::InvalidEvent)?;
    if input.thread_id != native.as_str() {
        return Err(NativeNormalizationError::WrongSession);
    }
    let key = |id: &str| {
        !id.is_empty() && id.len() <= 256 && id.bytes().all(|byte| byte.is_ascii_graphic())
    };
    if !key(&input.turn_id) || !key(&input.item_id) || !(1..=16).contains(&input.questions.len()) {
        return Err(NativeNormalizationError::UnsupportedRequest);
    }
    let mut ids: BTreeSet<_> = input.questions.iter().map(|q| q.id.clone()).collect();
    if ids.len() != input.questions.len() {
        return Err(NativeNormalizationError::UnsupportedRequest);
    }
    let mut properties = serde_json::Map::new();
    let mut required = Vec::new();
    for (index, q) in input.questions.into_iter().enumerate() {
        if q.is_secret
            || !key(&q.id)
            || q.question.trim().is_empty()
            || q.question.len() > 8192
            || q.header.len() > 8192
        {
            return Err(NativeNormalizationError::UnsupportedRequest);
        }
        let mut field =
            json!({"type":"string","title":q.question,"description":q.header,"maxLength":8192});
        if let Some(options) = q.options.filter(|options| !options.is_empty()) {
            let mut labels = BTreeSet::new();
            if options.len() > 32 {
                return Err(NativeNormalizationError::UnsupportedRequest);
            }
            let mut choices = Vec::new();
            for option in options {
                if option.label.trim().is_empty()
                    || option.label.len() > 8192
                    || option.description.len() > 8192
                    || !labels.insert(option.label.clone())
                {
                    return Err(NativeNormalizationError::UnsupportedRequest);
                }
                choices.push(json!({"const":option.label,"title":option.label,"description":option.description}));
            }
            if q.is_other {
                if !labels.contains("None of the above") {
                    choices.push(json!({"const":"None of the above","title":"None of the above"}));
                }
                let mut name = format!("chapchoo-note-{index}");
                while !ids.insert(name.clone()) {
                    name.push('-');
                }
                properties.insert(
                    name,
                    json!({"type":"string","title":"Additional answer or note","maxLength":8192}),
                );
            }
            field["oneOf"] = json!(choices);
        }
        required.push(q.id.clone());
        properties.insert(q.id, field);
    }
    let form = serde_json::from_value(
        json!({"type":"object","properties":properties,"required":required}),
    )
    .map_err(|_| NativeNormalizationError::UnsupportedRequest)?;
    validate_form(&form)?;
    let request = schema::CreateElicitationRequest::new(
        schema::ElicitationFormMode::new(
            schema::ElicitationSessionScope::new(native.as_str()),
            form,
        ),
        "Codex needs your input to continue.",
    );
    Ok(record(
        Kind::Request,
        position.occurrence().as_str(),
        vec![replace(Field::Request, &request)?],
    ))
}

/// Render the admitted profile at its exact capture occurrence. The live
/// owner independently validates the profile and owns the native grant.
pub(super) fn sandbox_permission(
    raw: &str,
    position: &SourcePosition,
    native: &NativeConversationId,
) -> Result<Record, NativeNormalizationError> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Request {
        thread_id: String,
        turn_id: String,
        item_id: String,
        started_at_ms: i64,
        cwd: String,
        reason: Option<String>,
        environment_id: Option<String>,
        permissions: chapchoo_domain::SandboxPermissionProfile,
    }
    let params: Request =
        serde_json::from_str(raw).map_err(|_| NativeNormalizationError::InvalidEvent)?;
    if params.thread_id != native.as_str() {
        return Err(NativeNormalizationError::WrongSession);
    }
    if [&params.turn_id, &params.item_id].into_iter().any(|id| {
        id.is_empty() || id.len() > 256 || !id.bytes().all(|byte| byte.is_ascii_graphic())
    }) || params.started_at_ms < 0
    {
        return Err(NativeNormalizationError::UnsupportedRequest);
    }
    let description = params
        .permissions
        .consent_description(
            &params.cwd,
            params.environment_id.as_deref(),
            params.reason.as_deref(),
        )
        .ok_or(NativeNormalizationError::UnsupportedRequest)?;
    use schema::PermissionOptionKind::{AllowAlways, AllowOnce, RejectOnce};
    let options = [
        ("grant-turn", "Grant for this turn", AllowOnce),
        (
            "grant-turn-strict",
            "Grant for this turn with strict review",
            AllowOnce,
        ),
        ("grant-session", "Grant for this session", AllowAlways),
        (
            "deny-permissions",
            "Do not grant these permissions",
            RejectOnce,
        ),
    ]
    .into_iter()
    .map(|(id, label, kind)| schema::PermissionOption::new(id, label, kind))
    .collect();
    let request = schema::RequestPermissionRequest::new(
        native.as_str(),
        "Grant sandbox permissions?",
        options,
    )
    .description(Some(description));
    Ok(record(
        Kind::Request,
        position.occurrence().as_str(),
        vec![replace(Field::Request, &request)?],
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
    if method == "item/started" && item.kind != "fileChange" {
        return Ok(None);
    }
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
        "mcpToolCall" => (Kind::Tool, mcp_fields(&item)?),
        "fileChange" => (Kind::Tool, file_change_fields(&item)?),
        _ => return Ok(None),
    };
    Ok(Some(record(kind, &key(params.turn_id, item.id)?, fields)))
}

fn file_change_fields(item: &Item<'_>) -> Result<Vec<Update>, NativeNormalizationError> {
    let changes = item.changes.ok_or(NativeNormalizationError::InvalidEvent)?;
    let status = match item.status {
        Some("inProgress") => schema::ToolCallStatus::InProgress,
        Some("completed") => schema::ToolCallStatus::Completed,
        Some("declined" | "failed") => schema::ToolCallStatus::Failed,
        _ => return Err(NativeNormalizationError::InvalidEvent),
    };
    Ok(vec![
        replace(Field::Title, &"File changes")?,
        replace(Field::Status, &status)?,
        replace(Field::ToolInput, &changes)?,
    ])
}

fn mcp_fields(item: &Item<'_>) -> Result<Vec<Update>, NativeNormalizationError> {
    let names = item.server.zip(item.tool).filter(|(server, tool)| {
        [*server, *tool].into_iter().all(|name| {
            !name.is_empty() && name.len() <= 256 && name.bytes().all(|b| b.is_ascii_graphic())
        })
    });
    let (server, tool) = names.ok_or(NativeNormalizationError::InvalidEvent)?;
    let status = match item.status {
        Some("completed") => schema::ToolCallStatus::Completed,
        Some("failed") => schema::ToolCallStatus::Failed,
        _ => return Err(NativeNormalizationError::InvalidEvent),
    };
    let mut fields = vec![
        replace(Field::Title, &format!("mcp.{server}.{tool}"))?,
        replace(Field::Description, &tool)?,
        replace(Field::Status, &status)?,
    ];
    #[derive(Serialize)]
    struct ToolInput<'a> {
        server: &'a str,
        tool: &'a str,
        arguments: &'a serde_json::value::RawValue,
    }
    #[derive(Serialize)]
    struct ToolOutput<'a> {
        result: Option<&'a serde_json::value::RawValue>,
        error: Option<&'a serde_json::value::RawValue>,
    }
    if let Some(arguments) = item.arguments {
        fields.push(replace(
            Field::ToolInput,
            &ToolInput {
                server,
                tool,
                arguments,
            },
        )?);
    }
    if item.result.is_some() || item.error.is_some() {
        fields.push(replace(
            Field::ToolOutput,
            &ToolOutput {
                result: item.result,
                error: item.error,
            },
        )?);
    }
    Ok(fields)
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
