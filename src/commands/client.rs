use std::sync::Arc;
use async_trait::async_trait;
use bson::Bson;
use tokio::sync::RwLock;
use uuid::Uuid;
use common::connection::Connection;
use common::message::{Message, MessageResponse, OperationStatus};
use crate::commands::Command;
use crate::store::Store;

#[derive(Default)]
pub struct ClientIDCommand {
    conn_id: Option<Uuid>,
}

#[async_trait]
impl Command for ClientIDCommand {
    async fn pre_exec(&mut self, connection: &Connection, _encrypted: bool) -> bool {
        self.conn_id = Some(connection.get_id());
        true
    }

    async fn execute(&mut self, _store: Arc<RwLock<Store>>, _args: Bson, _message: &Message) -> Option<MessageResponse> {
        Some(MessageResponse {
            content: self.conn_id.map(|x| Bson::String(x.to_string())),
            status: OperationStatus::Failure,
        })
    }

    async fn post_exec(&mut self, _connection: &mut Connection, _response: Option<&MessageResponse>) {}
}