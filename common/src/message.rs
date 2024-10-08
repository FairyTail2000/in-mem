use std::fmt::Display;

use bson;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::command::CommandID;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MessageResponse {
    pub content: Option<bson::Bson>,
    pub status: OperationStatus,
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum OperationStatus {
    Success,
    Failure,
    NotFound,
    NotAllowed,
    OutOfMemory,
    /// Happens when you try to access a string as a number
    TypeError,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Command {
    pub command_id: CommandID,
    pub payload: bson::Bson,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MessageContent {
    Command(Command),
    Response(MessageResponse),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub content: MessageContent,
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
        }
    }

    pub fn new_command(id: Uuid, command: Command) -> Self {
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

    pub fn to_vec(&self) -> bson::ser::Result<Vec<u8>> {
        bson::to_vec(self)
    }

    pub fn from_slice(slice: &[u8]) -> bson::de::Result<Self> {
        bson::from_slice(slice)
    }
}
