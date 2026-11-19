//! Explicit Conversation Work actions, separate from native turn outcomes.

use crate::ConversationText;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkTaskStatus {
    Pending,
    InProgress,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkTask {
    pub text: ConversationText,
    pub status: WorkTaskStatus,
}

#[derive(Debug)]
pub enum SendAction {
    Continue {
        title: Option<ConversationText>,
        tasks: Vec<WorkTask>,
        message: Option<ConversationText>,
    },
    RequestInput {
        message: ConversationText,
    },
    Finish {
        message: ConversationText,
    },
    Ignore,
}

impl SendAction {
    pub fn message(&self) -> Option<&ConversationText> {
        match self {
            Self::Continue { message, .. } => message.as_ref(),
            Self::RequestInput { message } | Self::Finish { message } => Some(message),
            Self::Ignore => None,
        }
    }
}
