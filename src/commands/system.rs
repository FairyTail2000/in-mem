use std::sync::Arc;
use async_trait::async_trait;
use bson::Bson;
use tokio::sync::RwLock;
use common::connection::Connection;
use common::message::{Message, MessageResponse};
use crate::commands::Command;
use crate::store::Store;

pub struct ShutdownCommand {}

#[async_trait]
impl Command for ShutdownCommand {
    async fn pre_exec(&mut self, _connection: &Connection, _encrypted: bool) -> bool { true }

    async fn execute(&mut self, _store: Arc<RwLock<Store>>, _args: Bson, _message: &Message) -> Option<MessageResponse> {
        std::process::exit(0);
    }

    async fn post_exec(&mut self, _connection: &mut Connection, _response: Option<&MessageResponse>) {}
}