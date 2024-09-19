use std::sync::Arc;
use async_trait::async_trait;

use bson::Bson;
use tokio::sync::RwLock;
use common::command_input::{DeleteCommandInput, GetCommandInput, SetCommandInput};
use common::connection::Connection;

use common::message::{Message, MessageResponse, OperationStatus};

use crate::commands::Command;
use crate::store::{Store, StoreAble};


pub struct GetCommand {}

#[async_trait]
impl Command for GetCommand {
    async fn pre_exec(&mut self, _connection: &Connection, _encrypted: bool) -> bool { true }

    async fn execute(&mut self, store: Arc<RwLock<Store>>, args: Bson, _message: &Message) -> Option<MessageResponse> {
        let store = store.read().await;
        let args: GetCommandInput = match args.try_into() {
            Err(_) => { return None; }
            Ok(doc) => doc
        };

        let rsp = match store.get(&args.key) {
            None => {
                MessageResponse {
                    content: args.default.map(|x| Bson::String(x.to_string())),
                    status: OperationStatus::Failure,
                }
            }
            Some(val) => {
                MessageResponse {
                    content: Some(Bson::String(val.to_string())),
                    status: OperationStatus::Success,
                }
            }
        };
        Some(rsp)
    }

    async fn post_exec(&mut self, _connection: &mut Connection, _response: Option<&MessageResponse>) {}
}

pub struct SetCommand {}

#[async_trait]
impl Command for SetCommand {
    async fn pre_exec(&mut self, _connection: &Connection, _encrypted: bool) -> bool { true }

    async fn execute(&mut self, store: Arc<RwLock<Store>>, args: Bson, _message: &Message) -> Option<MessageResponse> {
        let mut store = store.write().await;
        let args: SetCommandInput = match args.try_into() {
            Err(_) => { return None; }
            Ok(doc) => doc
        };

        let rsp = match store.set(args.key, args.value) {
            Ok(_) => {
                MessageResponse {
                    content: None,
                    status: OperationStatus::Success,
                }
            }
            Err(err) => {
                MessageResponse {
                    content: Some(Bson::String(err.to_string())),
                    status: OperationStatus::Failure,
                }
            }
        };
        Some(rsp)
    }

    async fn post_exec(&mut self, _connection: &mut Connection, _response: Option<&MessageResponse>) {}
}

pub struct DeleteCommand {}

#[async_trait]
impl Command for DeleteCommand {
    async fn pre_exec(&mut self, _connection: &Connection, _encrypted: bool) -> bool { true }

    async fn execute(&mut self, store: Arc<RwLock<Store>>, args: Bson, _message: &Message) -> Option<MessageResponse> {
        let mut store = store.write().await;
        let args: DeleteCommandInput = match args.try_into() {
            Err(_) => { return None; }
            Ok(doc) => doc
        };

        let rsp = match store.remove(&args.key) {
            Some(val) => {
                MessageResponse {
                    content: Some(Bson::String(val.to_string())),
                    status: OperationStatus::Success,
                }
            }
            None => {
                MessageResponse {
                    content: None,
                    status: OperationStatus::NotFound,
                }
            }
        };
        Some(rsp)
    }

    async fn post_exec(&mut self, _connection: &mut Connection, _response: Option<&MessageResponse>) {}
}