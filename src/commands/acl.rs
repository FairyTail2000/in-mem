use std::sync::Arc;
use async_trait::async_trait;
use bson::Bson;
use tokio::sync::RwLock;
use common::command_input::{AclListCommandInput, AclRemoveCommandInput, AclSetCommandInput};
use common::connection::Connection;
use common::message::{Message, MessageResponse, OperationStatus};
use crate::commands::Command;
use crate::store::{ACLAble, Store};

pub struct AclSetCommand {}


#[async_trait]
impl Command for AclSetCommand {
    async fn pre_exec(&mut self, _connection: &Connection, _encrypted: bool) -> bool { true }

    async fn execute(&mut self, store: Arc<RwLock<Store>>, args: Bson, _message: &Message) -> Option<MessageResponse> {
        let mut store = store.write().await;
        let args: AclSetCommandInput = match args.as_document() {
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

        store.acl_add(&args.user, args.command);
        let rsp = MessageResponse {
            content: None,
            status: OperationStatus::Success,
        };
        Some(rsp)
    }

    async fn post_exec(&mut self, _connection: &mut Connection, _response: Option<&MessageResponse>) {}
}

pub struct AclRemoveCommand {}

#[async_trait]
impl Command for AclRemoveCommand {
    async fn pre_exec(&mut self, _connection: &Connection, _encrypted: bool) -> bool { true }

    async fn execute(&mut self, store: Arc<RwLock<Store>>, args: Bson, _message: &Message) -> Option<MessageResponse> {
        let mut store = store.write().await;
        let args: AclRemoveCommandInput = match args.as_document() {
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

        store.acl_remove(&args.user, args.command);
        let rsp = MessageResponse {
            content: None,
            status: OperationStatus::Success,
        };
        Some(rsp)
    }

    async fn post_exec(&mut self, _connection: &mut Connection, _response: Option<&MessageResponse>) {}
}

pub struct AclListCommand {}

#[async_trait]
impl Command for AclListCommand {
    async fn pre_exec(&mut self, _connection: &Connection, _encrypted: bool) -> bool { true }

    async fn execute(&mut self, store: Arc<RwLock<Store>>, args: Bson, _message: &Message) -> Option<MessageResponse> {
        let args: AclListCommandInput = match args.as_document() {
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

        let store = store.read().await;
        let commands = store.acl_list(&args.user);
        let res = commands.iter().map(|cmd| cmd.to_string()).collect::<Vec<String>>().join(", ").to_string();
        let rsp = MessageResponse {
            content: Some(Bson::String(res)),
            status: OperationStatus::Success,
        };
        Some(rsp)
    }

    async fn post_exec(&mut self, _connection: &mut Connection, _response: Option<&MessageResponse>) {}
}