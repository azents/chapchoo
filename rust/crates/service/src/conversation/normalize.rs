use agent_client_protocol::{schema::MaybeUndefined, v2::schema};
use chapchoo_domain::{NativeConversationId, SnapshotId, SourcePosition};
use chapchoo_persistence::{
    CapturedEventPair, ConversationFieldChange as Change, ConversationFieldUpdate as Update,
    ConversationRecordField as Field, ConversationRecordKind as Kind,
    ConversationRecordUpdate as Record,
};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy)]
pub enum NativeNormalizationError {
    InvalidEvent,
    WrongSession,
    UnsupportedRequest,
    Capacity,
}
impl fmt::Display for NativeNormalizationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidEvent => "Native session event is invalid",
            Self::WrongSession => "Native event belongs to another session",
            Self::UnsupportedRequest => "Native interactive request is unsupported",
            Self::Capacity => "Native session event exceeds normalization capacity",
        })
    }
}
impl std::error::Error for NativeNormalizationError {}
pub struct NormalizedCapture {
    pub pair: CapturedEventPair,
    pub records: Vec<Record>,
}
impl fmt::Debug for NormalizedCapture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("NormalizedCapture([REDACTED])")
    }
}
#[derive(Deserialize)]
struct Envelope {
    method: String,
    params: Box<serde_json::value::RawValue>,
}

/// Preserve original bytes and decode supported native semantics only at ingress.
pub fn normalize_captured_event(
    snapshot: &SnapshotId,
    position: &SourcePosition,
    native: &NativeConversationId,
    raw: &[u8],
) -> Result<NormalizedCapture, NativeNormalizationError> {
    if raw.len() > 16 * 1024 * 1024 {
        return Err(NativeNormalizationError::Capacity);
    }
    let envelope: Envelope =
        serde_json::from_slice(raw).map_err(|_| NativeNormalizationError::InvalidEvent)?;
    let mut records = Vec::new();
    match envelope.method.as_str() {
        "session/update" => {
            let update: schema::UpdateSessionNotification =
                serde_json::from_str(envelope.params.get())
                    .map_err(|_| NativeNormalizationError::InvalidEvent)?;
            same_session(native, &update.session_id)?;
            records.extend(normalize_update(update.update, position, native)?);
        }
        "session/request_permission" => {
            let request: schema::RequestPermissionRequest =
                serde_json::from_str(envelope.params.get())
                    .map_err(|_| NativeNormalizationError::InvalidEvent)?;
            same_session(native, &request.session_id)?;
            let mut options = std::collections::BTreeSet::new();
            let unsupported = request.options.is_empty()
                || request
                    .options
                    .iter()
                    .any(|o| !options.insert(o.option_id.to_string()));
            let mut fields = vec![replace(Field::Request, &request)?];
            if unsupported {
                fields.push(replace(Field::Status, &"unsupported")?);
            }
            records.push(record(
                Kind::Request,
                position.occurrence().as_str(),
                fields,
            ));
        }
        "elicitation/create" => {
            let request: schema::CreateElicitationRequest =
                serde_json::from_str(envelope.params.get())
                    .map_err(|_| NativeNormalizationError::InvalidEvent)?;
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Scoped {
                session_id: schema::SessionId,
            }
            let scoped: Scoped = serde_json::from_str(envelope.params.get())
                .map_err(|_| NativeNormalizationError::UnsupportedRequest)?;
            same_session(native, &scoped.session_id)?;
            let supported = match &request.mode {
                schema::ElicitationMode::Form(form)
                    if matches!(form.scope, schema::ElicitationScope::Session(_)) =>
                {
                    validate_form(&form.requested_schema).is_ok()
                }
                _ => false,
            };
            let fields = if supported {
                vec![replace(Field::Request, &request)?]
            } else {
                // Retain a request error rather than a fabricated refusal or a
                // browser consent URL. The original source remains in the pair.
                vec![
                    replace(Field::Title, &request.message)?,
                    replace(Field::Status, &"unsupported")?,
                ]
            };
            records.push(record(
                Kind::Request,
                position.occurrence().as_str(),
                fields,
            ));
        }
        _ => return Err(NativeNormalizationError::InvalidEvent),
    }
    let normalized =
        serde_json::to_vec(&records).map_err(|_| NativeNormalizationError::InvalidEvent)?;
    Ok(NormalizedCapture {
        pair: CapturedEventPair {
            snapshot: snapshot.clone(),
            position: position.clone(),
            raw: raw.into(),
            normalized,
        },
        records,
    })
}
fn same_session(
    expected: &NativeConversationId,
    actual: &schema::SessionId,
) -> Result<(), NativeNormalizationError> {
    if expected.as_str() != actual.to_string() {
        Err(NativeNormalizationError::WrongSession)
    } else {
        Ok(())
    }
}
fn record(kind: Kind, id: &str, fields: Vec<Update>) -> Record {
    Record {
        kind,
        native_id: id.into(),
        fields,
    }
}
fn encoded(value: &impl Serialize) -> Result<Vec<u8>, NativeNormalizationError> {
    let mut bytes =
        serde_json::to_vec(value).map_err(|_| NativeNormalizationError::InvalidEvent)?;
    bytes.push(b'\n');
    Ok(bytes)
}
fn replace(field: Field, value: &impl Serialize) -> Result<Update, NativeNormalizationError> {
    Ok(Update {
        field,
        change: Change::Replace(encoded(value)?),
    })
}
fn append(field: Field, value: &impl Serialize) -> Result<Update, NativeNormalizationError> {
    Ok(Update {
        field,
        change: Change::Append(encoded(value)?),
    })
}
fn patch<T: Serialize>(
    field: Field,
    value: MaybeUndefined<T>,
    fields: &mut Vec<Update>,
) -> Result<(), NativeNormalizationError> {
    match value {
        MaybeUndefined::Undefined => {}
        MaybeUndefined::Null => fields.push(Update {
            field,
            change: Change::Replace(Vec::new()),
        }),
        MaybeUndefined::Value(value) => fields.push(replace(field, &value)?),
    }
    Ok(())
}
fn content<T: Serialize>(
    value: MaybeUndefined<Vec<T>>,
    fields: &mut Vec<Update>,
) -> Result<(), NativeNormalizationError> {
    match value {
        MaybeUndefined::Undefined => {}
        MaybeUndefined::Null => fields.push(Update {
            field: Field::Content,
            change: Change::Replace(Vec::new()),
        }),
        MaybeUndefined::Value(blocks) => {
            let mut bytes = Vec::new();
            for block in blocks {
                bytes.extend(encoded(&block)?);
            }
            fields.push(Update {
                field: Field::Content,
                change: Change::Replace(bytes),
            });
        }
    }
    Ok(())
}
fn utterance<T: Serialize>(
    kind: Kind,
    id: schema::MessageId,
    body: MaybeUndefined<Vec<T>>,
) -> Result<Record, NativeNormalizationError> {
    let mut fields = Vec::new();
    content(body, &mut fields)?;
    Ok(record(kind, &id.to_string(), fields))
}
fn normalize_update(
    update: schema::SessionUpdate,
    position: &SourcePosition,
    native: &NativeConversationId,
) -> Result<Option<Record>, NativeNormalizationError> {
    Ok(Some(match update {
        schema::SessionUpdate::UserMessageChunk(chunk) => record(
            Kind::User,
            &chunk.message_id.to_string(),
            vec![append(Field::Content, &chunk.content)?],
        ),
        schema::SessionUpdate::AgentMessageChunk(chunk) => record(
            Kind::Assistant,
            &chunk.message_id.to_string(),
            vec![append(Field::Content, &chunk.content)?],
        ),
        schema::SessionUpdate::AgentThoughtChunk(chunk) => record(
            Kind::Thought,
            &chunk.message_id.to_string(),
            vec![append(Field::Content, &chunk.content)?],
        ),
        schema::SessionUpdate::UserMessage(message) => {
            utterance(Kind::User, message.message_id, message.content)?
        }
        schema::SessionUpdate::AgentMessage(message) => {
            utterance(Kind::Assistant, message.message_id, message.content)?
        }
        schema::SessionUpdate::AgentThought(message) => {
            utterance(Kind::Thought, message.message_id, message.content)?
        }
        schema::SessionUpdate::ToolCallContentChunk(chunk) => record(
            Kind::Tool,
            &chunk.tool_call_id.to_string(),
            vec![append(Field::Content, &chunk.content)?],
        ),
        schema::SessionUpdate::ToolCallUpdate(tool) => {
            let mut fields = Vec::new();
            content(tool.content, &mut fields)?;
            patch(Field::Title, tool.title, &mut fields)?;
            patch(Field::Description, tool.name, &mut fields)?;
            patch(Field::Kind, tool.kind, &mut fields)?;
            patch(Field::Status, tool.status, &mut fields)?;
            patch(Field::Locations, tool.locations, &mut fields)?;
            patch(Field::ToolInput, tool.raw_input, &mut fields)?;
            patch(Field::ToolOutput, tool.raw_output, &mut fields)?;
            record(Kind::Tool, &tool.tool_call_id.to_string(), fields)
        }
        schema::SessionUpdate::StateUpdate(state) => record(
            Kind::State,
            native.as_str(),
            vec![replace(Field::Foreground, &state)?],
        ),
        schema::SessionUpdate::PlanUpdate(plan) => {
            let id = match &plan.plan {
                schema::PlanUpdateContent::Items(items) => items.plan_id.to_string(),
                schema::PlanUpdateContent::Other(other) => other.plan_id.to_string(),
                _ => return Ok(None),
            };
            record(Kind::Plan, &id, vec![replace(Field::Content, &plan.plan)?])
        }
        schema::SessionUpdate::Notice(notice) => record(
            Kind::Notice,
            position.occurrence().as_str(),
            vec![
                replace(Field::Title, &notice.title)?,
                replace(Field::Description, &notice.description)?,
                replace(Field::Status, &notice.severity)?,
            ],
        ),
        schema::SessionUpdate::TerminalUpdate(terminal) => {
            let mut fields = Vec::new();
            patch(Field::Title, terminal.command, &mut fields)?;
            patch(Field::Status, terminal.exit_status, &mut fields)?;
            patch(Field::Content, terminal.output, &mut fields)?;
            record(Kind::Terminal, &terminal.terminal_id.to_string(), fields)
        }
        schema::SessionUpdate::TerminalOutputChunk(chunk) => record(
            Kind::Terminal,
            &chunk.terminal_id.to_string(),
            vec![append(
                Field::Content,
                &schema::TerminalOutput::new(chunk.data),
            )?],
        ),
        _ => return Ok(None),
    }))
}
pub(super) fn validate_form(
    form: &schema::ElicitationSchema,
) -> Result<(), NativeNormalizationError> {
    if form.properties.len() > 64 {
        return Err(NativeNormalizationError::Capacity);
    }
    for required in form.required.iter().flatten() {
        let property = form
            .properties
            .get(required)
            .ok_or(NativeNormalizationError::UnsupportedRequest)?;
        if !supported(property) {
            return Err(NativeNormalizationError::UnsupportedRequest);
        }
    }
    Ok(())
}
pub(super) fn supported(property: &schema::ElicitationPropertySchema) -> bool {
    use schema::ElicitationPropertySchema as P;
    match property {
        P::String(p) => {
            text_rule(p).is_ok()
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
pub(super) fn text_rule(
    p: &schema::StringPropertySchema,
) -> Result<chapchoo_domain::FormTextRule, chapchoo_domain::InvalidFormTextRule> {
    let format = match p.format.as_ref() {
        Some(schema::StringFormat::Email) => Some("email"),
        Some(schema::StringFormat::Uri) => Some("uri"),
        Some(schema::StringFormat::Date) => Some("date"),
        Some(schema::StringFormat::DateTime) => Some("date-time"),
        Some(schema::StringFormat::Other(value)) => Some(value.as_str()),
        _ => None,
    };
    chapchoo_domain::FormTextRule::parse(p.pattern.as_deref(), format)
}
#[cfg(test)]
mod tests;
