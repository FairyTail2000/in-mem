use std::sync::Arc;
use async_trait::async_trait;

use bson::Bson;
use tokio::sync::RwLock;
use common::connection::Connection;

use common::message::{Message, MessageResponse, OperationStatus};

use crate::commands::Command;
use crate::store::Store;

pub struct HeartbeatCommand {}

#[async_trait]
impl Command for HeartbeatCommand {
    async fn pre_exec(&mut self, _connection: &Connection, _encrypted: bool) -> bool { true }

    async fn execute(&mut self, _: Arc<RwLock<Store>>, _: Bson, message: &Message) -> Option<MessageResponse> {
        Some(MessageResponse {
            content: None,
            status: OperationStatus::Success,
            in_reply_to: Some(message.id),
        })
    }

    async fn post_exec(&mut self, _connection: &mut Connection, _response: Option<&MessageResponse>) {}
}