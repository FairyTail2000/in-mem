use std::sync::Arc;
use async_trait::async_trait;
use bson::Bson;
use tokio::sync::RwLock;
use common::command_input::UserRemoveCommandInput;
use common::connection::Connection;
use common::message::{Message, MessageResponse, OperationStatus};
use crate::commands::Command;
use crate::store::{Store, UserAble};

pub struct UserRemoveCommand {}

#[async_trait]
impl Command for UserRemoveCommand {
    async fn pre_exec(&mut self, _connection: &Connection, _encrypted: bool) -> bool { true }

    async fn execute(&mut self, store: Arc<RwLock<Store>>, args: Bson, _message: &Message) -> Option<MessageResponse> {
        let mut store = store.write().await;
        let args: UserRemoveCommandInput = match args.as_document() {
            None => {
                return None;
            }
            Some(doc) => {
                match bson::from_bson(Bson::Document(doc.clone())) {
                    Ok(val) => val,
                    Err(_) => {
                        return None;
                    }
                }
            }
        };

        let rsp = if store.user_remove(&args.user) {
            MessageResponse {
                content: None,
                status: OperationStatus::Success,
            }
        } else {
            MessageResponse {
                content: None,
                status: OperationStatus::NotFound,
            }
        };
        Some(rsp)
    }

    async fn post_exec(&mut self, _connection: &mut Connection, _: Option<&MessageResponse>) {}
}
