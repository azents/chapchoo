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
            "description":"Update Conversation Work and optionally send a Chat message. Continue starts work (requires title on first call) or replaces its complete ordered task list (an empty list clears tasks). Request_input asks a question and waits; finish publishes a result and completes work; ignore completes work silently. Ordinary model output is not sent.",
            "inputSchema":{
                "type":"object",
                "properties":{
                    "action":{"type":"string","enum":["continue","request_input","finish","ignore"]},
                    "title":{"type":"string"},
                    "tasks":{"type":"array","items":{
                        "type":"object","properties":{
                            "text":{"type":"string"},
                            "status":{"type":"string","enum":["pending","in_progress","completed"]}
                        },"required":["text","status"],"additionalProperties":false
                    }},
                    "message":{"type":"string"}
                },
                "required":["action"],"additionalProperties":false,
                "oneOf":[
                    {"properties":{"action":{"const":"continue"}},"required":["tasks"]},
                    {"properties":{"action":{"enum":["request_input","finish"]},"title":false,"tasks":false},"required":["message"]},
                    {"properties":{"action":{"const":"ignore"},"title":false,"tasks":false,"message":false}}
                ]
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
