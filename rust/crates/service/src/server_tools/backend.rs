use super::*;
use crate::{ConversationMessageScope, ConversationService, DomainReason};
use chapchoo_domain::{ConversationText, SendAction, WorkTask, WorkTaskStatus};
use serde::Deserialize;
use serde_json::json;

pub struct ConversationToolBackend {
    conversation: Arc<ConversationService>,
}
redacted!(ConversationToolBackend);
impl ConversationToolBackend {
    pub fn new(conversation: Arc<ConversationService>) -> Self {
        Self { conversation }
    }
    pub fn catalog_definition() -> Result<Vec<u8>, ServiceError> {
        serde_json::to_vec(&json!({"tools":[{
            "name":"send",
            "description":concat!(
                "Update Conversation Work and optionally send a Chat message. ",
                "Use action \"continue\" with the complete ordered tasks array (an empty list clears tasks). ",
                "Supply title when starting work; omit it on updates to retain the current title. Message is optional. ",
                "Use \"request_input\" with message to ask and wait, or \"finish\" with message to send and complete work. ",
                "Supply only action for \"ignore\" to complete work silently. ",
                "When ending work because it failed or was cancelled, complete it the same way: use finish to send a message or ignore to close silently. ",
                "Ordinary model output is not sent."
            ),
            "inputSchema":{
                "type":"object",
                "properties":{
                    "action":{"type":"string","enum":["continue","request_input","finish","ignore"]},
                    "title":{"type":"string","description":"For continue only. Required to start work; optional on updates."},
                    "tasks":{"type":"array","description":"Required for continue only. Full ordered task list.","items":{
                        "type":"object","properties":{
                            "text":{"type":"string"},
                            "status":{"type":"string","enum":["pending","in_progress","completed"]}
                        },"required":["text","status"],"additionalProperties":false
                    }},
                    "message":{"type":"string","description":"Required for request_input and finish, optional for continue. Omit for ignore."}
                },
                "required":["action"],"additionalProperties":false
            },
            "outputSchema":{"type":"object","properties":{"message_id":{"type":"string"}},"additionalProperties":false}
        }]})).map_err(ServiceError::infrastructure)
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TaskInput {
    text: String,
    status: WorkTaskStatus,
}

#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
enum SendInput {
    Continue {
        #[serde(default, deserialize_with = "supplied_string")]
        title: Option<String>,
        tasks: Vec<TaskInput>,
        #[serde(default, deserialize_with = "supplied_string")]
        message: Option<String>,
    },
    RequestInput {
        message: String,
    },
    Finish {
        message: String,
    },
    Ignore {},
}

fn supplied_string<'de, D: serde::Deserializer<'de>>(input: D) -> Result<Option<String>, D::Error> {
    String::deserialize(input).map(Some)
}

fn action(name: &str, arguments: &[u8]) -> Result<SendAction, ServerToolValidation> {
    if name != "send" {
        return Err(ServerToolValidation::UnknownTool);
    }
    let input: SendInput =
        serde_json::from_slice(arguments).map_err(|_| ServerToolValidation::InvalidArguments)?;
    let text =
        |value| ConversationText::parse(value).map_err(|_| ServerToolValidation::InvalidArguments);
    Ok(match input {
        SendInput::Continue {
            title,
            tasks,
            message,
        } => SendAction::Continue {
            title: title.map(text).transpose()?,
            tasks: tasks
                .into_iter()
                .map(|task| {
                    Ok(WorkTask {
                        text: text(task.text)?,
                        status: task.status,
                    })
                })
                .collect::<Result<_, ServerToolValidation>>()?,
            message: message.map(text).transpose()?,
        },
        SendInput::RequestInput { message } => SendAction::RequestInput {
            message: text(message)?,
        },
        SendInput::Finish { message } => SendAction::Finish {
            message: text(message)?,
        },
        SendInput::Ignore {} => SendAction::Ignore,
    })
}

#[cfg(test)]
mod tests;

impl ServerToolBackend for ConversationToolBackend {
    fn authorize<'a>(&'a self, scope: &'a ServerToolScope) -> ServerToolFuture<'a, ()> {
        Box::pin(async move {
            self.conversation
                .check_tool_scope(
                    &scope.source(),
                    &ConversationMessageScope {
                        agent_id: &scope.agent_id,
                        conversation_id: &scope.conversation_id,
                    },
                )
                .await
        })
    }
    fn catalog(&self) -> Result<Vec<u8>, ServiceError> {
        Self::catalog_definition()
    }
    fn validate(&self, name: &str, arguments: &[u8]) -> Result<(), ServerToolValidation> {
        action(name, arguments).map(|_| ())
    }
    fn execute<'a>(
        &'a self,
        request: &'a ServerToolRequest,
        _cancellation: ToolCancellation,
    ) -> ServerToolFuture<'a, Vec<u8>> {
        Box::pin(async move {
            let action = action(&request.name, &request.arguments)
                .map_err(|_| ServiceError::domain(DomainReason::InvalidInput))?;
            // No recall promise: once dispatch wins, finish the database write.
            let receipt = self
                .conversation
                .send_work(
                    &request.scope.source(),
                    &ConversationMessageScope {
                        agent_id: &request.scope.agent_id,
                        conversation_id: &request.scope.conversation_id,
                    },
                    &action,
                )
                .await?;
            let (text, structured) = match receipt {
                Some(receipt) => ("Message accepted.", json!({"message_id":receipt.id})),
                None => ("Work action accepted.", json!({})),
            };
            serde_json::to_vec(&json!({
                "content":[{"type":"text","text":text}],
                "structuredContent":structured,
                "isError":false
            }))
            .map_err(ServiceError::infrastructure)
        })
    }
}
