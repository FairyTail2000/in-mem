use std::fmt::Display;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::command;
use bson;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct MessageResponse {
    pub content: Option<bson::Bson>,
    pub status: OperationStatus,
    pub in_reply_to: Option<Uuid>,
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum OperationStatus {
    Success,
    Failure,
    NotFound,
    NotAllowed
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum MessageContent {
    Command(command::Command),
    Response(MessageResponse),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub content: MessageContent,
    pub fragment: bool,
    pub last_fragment: bool
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Message {
    pub fn new(id: Uuid, content: MessageContent) -> Self {
        Self {
            id,
            content,
            fragment: false,
            last_fragment: false,
        }
    }

    pub fn new_command(id: Uuid, command: command::Command) -> Self {
        Self {
            id,
            content: MessageContent::Command(command),
            fragment: false,
            last_fragment: false,
        }
    }

    pub fn new_response(id: Uuid, response: MessageResponse) -> Self {
        Self {
            id,
            content: MessageContent::Response(response),
            fragment: false,
            last_fragment: false,
        }
    }
    
    pub fn to_vec(&self) -> bson::ser::Result<Vec<u8>> {
        return bson::to_vec(self);
    }
}
