use std::sync::Arc;
use async_trait::async_trait;
use bson::{Bson, Document};
use tokio::sync::RwLock;
use common::command_input::{HashMapDeleteCommandInput, HashMapExistsCommandInput, HashMapGetAllCommandInput, HashMapGetCommandInput, HashMapIncrByCommandInput, HashMapKeysCommandInput, HashMapLenCommandInput, HashMapSetCommandInput, HashMapStringLenCommandInput, HashMapValuesCommandInput};
use common::connection::Connection;
use common::message::{Message, MessageResponse, OperationStatus};
use crate::commands::Command;
use crate::store::{HashMapAble, Store};

pub struct HashMapDeleteCommand {}

#[async_trait]
impl Command for HashMapDeleteCommand {
    async fn pre_exec(&mut self, _connection: &Connection, _encrypted: bool) -> bool { true }

    async fn execute(&mut self, store: Arc<RwLock<Store>>, args: Bson, message: &Message) -> Option<MessageResponse> {
        let args: HashMapDeleteCommandInput = match args.as_document() {
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

        let mut store = store.write().await;
        let rsp = match store.hremove(args.key, args.field) {
            true => {
                MessageResponse {
                    content: None,
                    status: OperationStatus::Success,
                    in_reply_to: Some(message.id),
                }
            }
            false => {
                MessageResponse {
                    content: None,
                    status: OperationStatus::NotFound,
                    in_reply_to: Some(message.id),
                }
            }
        };
        Some(rsp)
    }

    async fn post_exec(&mut self, _connection: &mut Connection, _response: Option<&MessageResponse>) {}
}

pub struct HashMapGetCommand {}

#[async_trait]
impl Command for HashMapGetCommand {
    async fn pre_exec(&mut self, _connection: &Connection, _encrypted: bool) -> bool { true }

    async fn execute(&mut self, store: Arc<RwLock<Store>>, args: Bson, message: &Message) -> Option<MessageResponse> {
        let args: HashMapGetCommandInput = match args.as_document() {
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
        let rsp = match store.hget(args.key, args.field) {
            None => {
                MessageResponse {
                    content: None,
                    status: OperationStatus::NotFound,
                    in_reply_to: Some(message.id),
                }
            }
            Some(val) => {
                MessageResponse {
                    content: Some(Bson::String(val.clone())),
                    status: OperationStatus::Success,
                    in_reply_to: Some(message.id),
                }
            }
        };
        Some(rsp)
    }

    async fn post_exec(&mut self, _connection: &mut Connection, _response: Option<&MessageResponse>) {}
}

pub struct HashMapSetCommand {}

#[async_trait]
impl Command for HashMapSetCommand {
    async fn pre_exec(&mut self, _connection: &Connection, _encrypted: bool) -> bool { true }

    // Some might fail to insert. But it's not reported which failed ;)
    async fn execute(&mut self, store: Arc<RwLock<Store>>, args: Bson, message: &Message) -> Option<MessageResponse> {
        let args: HashMapSetCommandInput = match args.as_document() {
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

        let mut store = store.write().await;
        let mut okay = Vec::new();
        match okay.try_reserve_exact(args.value.len()) {
            Ok(_) => {}
            Err(err) => {
                log::error!("Error reserving space for values: {}", err);
                let rsp = MessageResponse {
                    content: None,
                    status: OperationStatus::Failure,
                    in_reply_to: Some(message.id),
                };
                return Some(rsp);
            }
        }
        for kv in args.value.into_iter() {
            let ok = store.hadd(args.key.clone(), kv.0, kv.1).is_ok();
            okay.push(ok);
        }
        let okay = okay.iter().all(|x| *x);
        let rsp = if okay {
            MessageResponse {
                content: None,
                status: OperationStatus::Success,
                in_reply_to: Some(message.id),
            }
        } else {
            MessageResponse {
                content: None,
                status: OperationStatus::Failure,
                in_reply_to: Some(message.id),
            }
        };
        Some(rsp)
    }

    async fn post_exec(&mut self, _connection: &mut Connection, _response: Option<&MessageResponse>) {}
}

pub struct HashMapGetAllCommand {}

#[async_trait]
impl Command for HashMapGetAllCommand {
    async fn pre_exec(&mut self, _: &Connection, _: bool) -> bool { true }

    async fn execute(&mut self, store: Arc<RwLock<Store>>, args: Bson, message: &Message) -> Option<MessageResponse> {
        let args: HashMapGetAllCommandInput = match args.as_document() {
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
        let rsp = match store.hget_all(args.key) {
            Ok(map) => {
                let map = map.into_iter().map(|(k, v)| (k, Bson::String(v))).collect::<Document>();
                MessageResponse {
                    content: Some(Bson::Document(map)),
                    status: OperationStatus::Success,
                    in_reply_to: Some(message.id),
                }
            }
            Err(err) => {
                MessageResponse {
                    content: Some(Bson::String(err.to_string())),
                    status: OperationStatus::Failure,
                    in_reply_to: Some(message.id),
                }
            }
        };
        Some(rsp)
    }

    async fn post_exec(&mut self, _connection: &mut Connection, _response: Option<&MessageResponse>) {}
}

pub struct HashMapKeysCommand {}

#[async_trait]
impl Command for HashMapKeysCommand {
    async fn pre_exec(&mut self, _connection: &Connection, _encrypted: bool) -> bool { true }

    async fn execute(&mut self, store: Arc<RwLock<Store>>, args: Bson, message: &Message) -> Option<MessageResponse> {
        let args: HashMapKeysCommandInput = match args.as_document() {
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
        let rsp = match store.hkeys(args.key) {
            Ok(keys) => {
                let keys = keys.into_iter().map(|k| Bson::String(k)).collect::<Vec<Bson>>();
                MessageResponse {
                    content: Some(Bson::Array(keys)),
                    status: OperationStatus::Success,
                    in_reply_to: Some(message.id),
                }
            }
            Err(err) => {
                MessageResponse {
                    content: Some(Bson::String(err.to_string())),
                    status: OperationStatus::Failure,
                    in_reply_to: Some(message.id),
                }
            }
        };
        Some(rsp)
    }

    async fn post_exec(&mut self, _connection: &mut Connection, _response: Option<&MessageResponse>) {}
}

pub struct HashMapLenCommand {}

#[async_trait]
impl Command for HashMapLenCommand {
    async fn pre_exec(&mut self, _connection: &Connection, _encrypted: bool) -> bool { true }

    async fn execute(&mut self, store: Arc<RwLock<Store>>, args: Bson, message: &Message) -> Option<MessageResponse> {
        let args: HashMapLenCommandInput = match args.as_document() {
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
        let rsp = MessageResponse {
            content: Some(Bson::Int64(store.hlen(args.key) as i64)),
            status: OperationStatus::Success,
            in_reply_to: Some(message.id),
        };
        Some(rsp)
    }

    async fn post_exec(&mut self, _connection: &mut Connection, _response: Option<&MessageResponse>) {}
}

pub struct HashMapValuesCommand {}

#[async_trait]
impl Command for HashMapValuesCommand {
    async fn pre_exec(&mut self, _connection: &Connection, _encrypted: bool) -> bool { true }

    async fn execute(&mut self, store: Arc<RwLock<Store>>, args: Bson, message: &Message) -> Option<MessageResponse> {
        let args: HashMapValuesCommandInput = match args.as_document() {
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
        let rsp = match store.hget_all_values(args.key) {
            Ok(values) => {
                let values = values.into_iter().map(|v| Bson::String(v)).collect::<Vec<Bson>>();
                MessageResponse {
                    content: Some(Bson::Array(values)),
                    status: OperationStatus::Success,
                    in_reply_to: Some(message.id),
                }
            }
            Err(err) => {
                MessageResponse {
                    content: Some(Bson::String(err.to_string())),
                    status: OperationStatus::Failure,
                    in_reply_to: Some(message.id),
                }
            }
        };
        Some(rsp)
    }

    async fn post_exec(&mut self, _connection: &mut Connection, _response: Option<&MessageResponse>) {}
}

pub struct HashMapExistsCommand {}

#[async_trait]
impl Command for HashMapExistsCommand {
    async fn pre_exec(&mut self, _connection: &Connection, _encrypted: bool) -> bool { true }

    async fn execute(&mut self, store: Arc<RwLock<Store>>, args: Bson, message: &Message) -> Option<MessageResponse> {
        let args: HashMapExistsCommandInput = match args.as_document() {
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
        let rsp = MessageResponse {
            content: Some(Bson::Boolean(store.hcontains(args.key, args.field))),
            status: OperationStatus::Success,
            in_reply_to: Some(message.id),
        };
        Some(rsp)
    }

    async fn post_exec(&mut self, _connection: &mut Connection, _response: Option<&MessageResponse>) {}
}

pub struct HashMapIncrByCommand {}

#[async_trait]
impl Command for HashMapIncrByCommand {
    async fn pre_exec(&mut self, _connection: &Connection, _encrypted: bool) -> bool { true }

    async fn execute(&mut self, store: Arc<RwLock<Store>>, args: Bson, message: &Message) -> Option<MessageResponse> {
        let args: HashMapIncrByCommandInput = match args.as_document() {
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

        let mut store = store.write().await;
        let rsp = match store.hincrby(args.key, args.field, args.value) {
            Ok(val) => {
                MessageResponse {
                    content: Some(Bson::Int64(val)),
                    status: OperationStatus::Success,
                    in_reply_to: Some(message.id),
                }
            }
            Err(err) => {
                MessageResponse {
                    content: Some(Bson::String(err.to_string())),
                    status: OperationStatus::Failure,
                    in_reply_to: Some(message.id),
                }
            }
        };
        Some(rsp)
    }

    async fn post_exec(&mut self, _connection: &mut Connection, _response: Option<&MessageResponse>) {}
}

pub struct HashMapStringLenCommand {}

#[async_trait]
impl Command for HashMapStringLenCommand {
    async fn pre_exec(&mut self, _connection: &Connection, _encrypted: bool) -> bool { true }

    async fn execute(&mut self, store: Arc<RwLock<Store>>, args: Bson, message: &Message) -> Option<MessageResponse> {
        let args: HashMapStringLenCommandInput = match args.as_document() {
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
        let rsp = match store.hstr_len(args.key, args.field) {
            Some(len) => {
                MessageResponse {
                    content: Some(Bson::Int64(len as i64)),
                    status: OperationStatus::Success,
                    in_reply_to: Some(message.id),
                }
            }
            None => {
                MessageResponse {
                    content: None,
                    status: OperationStatus::NotFound,
                    in_reply_to: Some(message.id),
                }
            }
        };
        Some(rsp)
    }

    async fn post_exec(&mut self, _connection: &mut Connection, _response: Option<&MessageResponse>) {}
}
