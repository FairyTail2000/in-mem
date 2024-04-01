use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::command;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct MessageResponse {
    pub content: Option<String>,
    pub status: OperationStatus,
    pub in_reply_to: Option<Uuid>,
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum OperationStatus {
    Success,
    Failure,
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum MessageContent {
    Command(command::Command),
    Response(MessageResponse),
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub content: MessageContent,
}

impl Message {
    pub fn new(id: Uuid, content: MessageContent) -> Self {
        Self {
            id,
            content,
        }
    }

    pub fn new_command(id: Uuid, command: command::Command) -> Self {
        Self {
            id,
            content: MessageContent::Command(command),
        }
    }

    pub fn new_response(id: Uuid, response: MessageResponse) -> Self {
        Self {
            id,
            content: MessageContent::Response(response),
        }
    }
}
