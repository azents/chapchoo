use super::*;
use crate::{
    ConversationMessageScope, ConversationService, DomainReason, PublishConversationMessage,
};
use chapchoo_domain::ConversationText;
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
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PublishText {
    text: String,
}
fn text(name: &str, arguments: &[u8]) -> Result<ConversationText, ServerToolValidation> {
    if name != "send_message" {
        return Err(ServerToolValidation::UnknownTool);
    }
    let input: PublishText =
        serde_json::from_slice(arguments).map_err(|_| ServerToolValidation::InvalidArguments)?;
    ConversationText::parse(input.text).map_err(|_| ServerToolValidation::InvalidArguments)
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
        serde_json::to_vec(&json!({"tools":[{
            "name":"send_message",
            "description":"Send text to the user in this conversation. Ordinary model output is not sent.",
            "inputSchema":{"type":"object","properties":{"text":{"type":"string"}},"required":["text"],"additionalProperties":false},
            "outputSchema":{"type":"object","properties":{"message_id":{"type":"string"}},"required":["message_id"],"additionalProperties":false}
        }]})).map_err(ServiceError::infrastructure)
    }
    fn validate(&self, name: &str, arguments: &[u8]) -> Result<(), ServerToolValidation> {
        text(name, arguments).map(|_| ())
    }
    fn execute<'a>(
        &'a self,
        request: &'a ServerToolRequest,
        _cancellation: ToolCancellation,
    ) -> ServerToolFuture<'a, Vec<u8>> {
        Box::pin(async move {
            let text = text(&request.name, &request.arguments)
                .map_err(|_| ServiceError::domain(DomainReason::InvalidInput))?;
            // No recall promise: once dispatch wins, finish the database write.
            let receipt = self
                .conversation
                .publish_message(
                    &request.scope.source(),
                    &PublishConversationMessage {
                        agent_id: &request.scope.agent_id,
                        conversation_id: &request.scope.conversation_id,
                        text: &text,
                    },
                )
                .await?;
            serde_json::to_vec(&json!({
                "content":[{"type":"text","text":"Message accepted."}],
                "structuredContent":{"message_id":receipt.id},
                "isError":false
            }))
            .map_err(ServiceError::infrastructure)
        })
    }
}
