use std::sync::Arc;
use async_trait::async_trait;
use bson::Bson;
use tokio::sync::RwLock;
use common::command_input::{LIndexCommandInput, LLenCommandInput, LMoveCommandInput, LPopCommandInput, LPosCommandInput, LPushCommandInput, LPushxCommandInput, LRangeCommandInput, LRemCommandInput, LSetCommandInput, LTrimCommandInput, RPopCommandInput, RPushCommandInput, RPushxCommandInput};
use common::connection::Connection;
use common::message::{Message, MessageResponse, OperationStatus};
use crate::commands::Command;
use crate::store::{Store, ListAble};

pub struct LlenCommand {}

#[async_trait]
impl Command for LlenCommand {
    async fn pre_exec(&mut self, _connection: &Connection, _encrypted: bool) -> bool { true }

    async fn execute(&mut self, store: Arc<RwLock<Store>>, args: Bson, _message: &Message) -> Option<MessageResponse> {
        let store = store.read().await;
        let args: LLenCommandInput = match args.try_into() {
            Err(_) => { return None; }
            Ok(doc) => doc
        };

        let rsp = MessageResponse {
            content: Some(Bson::String(store.llen(args.list).to_string())),
            status: OperationStatus::Success,
        };
        Some(rsp)
    }

    async fn post_exec(&mut self, _connection: &mut Connection, _response: Option<&MessageResponse>) {}
}

pub struct LindexCommand {}

#[async_trait]
impl Command for LindexCommand {
    async fn pre_exec(&mut self, _connection: &Connection, _encrypted: bool) -> bool { true }

    async fn execute(&mut self, store: Arc<RwLock<Store>>, args: Bson, _message: &Message) -> Option<MessageResponse> {
        let store = store.read().await;
        let args: LIndexCommandInput = match args.try_into() {
            Err(_) => { return None; }
            Ok(doc) => doc
        };

        let rsp = match store.lindex(args.list, args.key) {
            None => {
                MessageResponse {
                    content: None,
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

pub struct LmoveCommand {}

#[async_trait]
impl Command for LmoveCommand {
    async fn pre_exec(&mut self, _connection: &Connection, _encrypted: bool) -> bool { true }

    async fn execute(&mut self, store: Arc<RwLock<Store>>, args: Bson, _message: &Message) -> Option<MessageResponse> {
        let mut store = store.write().await;
        let args: LMoveCommandInput = match args.try_into() {
            Err(_) => { return None; }
            Ok(doc) => doc
        };

        let rsp = match store.lmove(args.src, args.dest, args.left_right, args.right_left) {
            None => {
                MessageResponse {
                    content: None,
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

pub struct LpopCommand {}

#[async_trait]
impl Command for LpopCommand {
    async fn pre_exec(&mut self, _connection: &Connection, _encrypted: bool) -> bool { true }

    async fn execute(&mut self, store: Arc<RwLock<Store>>, args: Bson, _message: &Message) -> Option<MessageResponse> {
        let mut store = store.write().await;
        let args: LPopCommandInput = match args.try_into() {
            Err(_) => { return None; }
            Ok(doc) => doc
        };

        let rsp = match store.lpop(args.list, args.count) {
            Ok(result) => {
                match result {
                    None => {
                        MessageResponse {
                            content: None,
                            status: OperationStatus::Failure,
                        }
                    }
                    Some(val) => {
                        MessageResponse {
                            content: Some(Bson::Array(val.iter().map(|x| Bson::String(x.to_string())).collect())),
                            status: OperationStatus::Success,
                        }
                    }
                }
            }
            Err(_err) => {
                MessageResponse {
                    content: None,
                    status: OperationStatus::OutOfMemory,
                }
            }
        };
        Some(rsp)
    }

    async fn post_exec(&mut self, _connection: &mut Connection, _response: Option<&MessageResponse>) {}
}

pub struct LposCommand {}

#[async_trait]
impl Command for LposCommand {
    async fn pre_exec(&mut self, _connection: &Connection, _encrypted: bool) -> bool { true }

    async fn execute(&mut self, store: Arc<RwLock<Store>>, args: Bson, _message: &Message) -> Option<MessageResponse> {
        let store = store.read().await;
        let args: LPosCommandInput = match args.try_into() {
            Err(_) => { return None; }
            Ok(doc) => doc
        };

        let rsp = match store.lpos(args.list, args.value, args.rank, args.count, args.max_len) {
            Ok(result) => {
                match result {
                    None => {
                        MessageResponse {
                            content: None,
                            status: OperationStatus::Failure,
                        }
                    }
                    Some(val) => {
                        MessageResponse {
                            content: Some(Bson::Array(val.iter().map(|x| Bson::Int64(*x as i64)).collect())),
                            status: OperationStatus::Success,
                        }
                    }
                }
            }
            Err(_err) => {
                MessageResponse {
                    content: None,
                    status: OperationStatus::OutOfMemory,
                }
            }
        };
        Some(rsp)
    }

    async fn post_exec(&mut self, _connection: &mut Connection, _response: Option<&MessageResponse>) {}
}

pub struct LpushCommand {}

#[async_trait]
impl Command for LpushCommand {
    async fn pre_exec(&mut self, _connection: &Connection, _encrypted: bool) -> bool { true }

    async fn execute(&mut self, store: Arc<RwLock<Store>>, args: Bson, _message: &Message) -> Option<MessageResponse> {
        let mut store = store.write().await;
        let args: LPushCommandInput = match args.try_into() {
            Err(_) => { return None; }
            Ok(doc) => doc
        };

        let rsp = match store.lpush(args.list.to_string(), args.values) {
            Ok(_) => {
                MessageResponse {
                    content: Some(Bson::Int64(store.llen(args.list) as i64)),
                    status: OperationStatus::Success,
                }
            }
            Err(_err) => {
                MessageResponse {
                    content: None,
                    status: OperationStatus::OutOfMemory,
                }
            }
        };
        Some(rsp)
    }

    async fn post_exec(&mut self, _connection: &mut Connection, _response: Option<&MessageResponse>) {}
}

pub struct LpushxCommand {}

#[async_trait]
impl Command for LpushxCommand {
    async fn pre_exec(&mut self, _connection: &Connection, _encrypted: bool) -> bool { true }

    async fn execute(&mut self, store: Arc<RwLock<Store>>, args: Bson, _message: &Message) -> Option<MessageResponse> {
        let mut store = store.write().await;
        let args: LPushxCommandInput = match args.try_into() {
            Err(_) => { return None; }
            Ok(doc) => doc
        };

        let rsp = match store.lpushx(args.list.to_string(), args.values) {
            Ok(_) => {
                MessageResponse {
                    content: Some(Bson::Int64(store.llen(args.list) as i64)),
                    status: OperationStatus::Success,
                }
            }
            Err(_err) => {
                MessageResponse {
                    content: None,
                    status: OperationStatus::OutOfMemory,
                }
            }
        };
        Some(rsp)
    }

    async fn post_exec(&mut self, _connection: &mut Connection, _response: Option<&MessageResponse>) {}
}

pub struct LrangeCommand {}

#[async_trait]
impl Command for LrangeCommand {
    async fn pre_exec(&mut self, _connection: &Connection, _encrypted: bool) -> bool { true }

    async fn execute(&mut self, store: Arc<RwLock<Store>>, args: Bson, _message: &Message) -> Option<MessageResponse> {
        let store = store.read().await;
        let args: LRangeCommandInput = match args.try_into() {
            Err(_) => { return None; }
            Ok(doc) => doc
        };

        let rsp = match store.lrange(args.list, args.start, args.stop) {
            Ok(result) => {
                MessageResponse {
                    content: Some(Bson::Array(result.iter().map(|x| Bson::String(x.to_string())).collect())),
                    status: OperationStatus::Failure,
                }
            }
            Err(_err) => {
                MessageResponse {
                    content: None,
                    status: OperationStatus::Success,
                }
            }
        };
        Some(rsp)
    }

    async fn post_exec(&mut self, _connection: &mut Connection, _response: Option<&MessageResponse>) {}
}

pub struct LremCommand {}

#[async_trait]
impl Command for LremCommand {
    async fn pre_exec(&mut self, _connection: &Connection, _encrypted: bool) -> bool { true }

    async fn execute(&mut self, store: Arc<RwLock<Store>>, args: Bson, _message: &Message) -> Option<MessageResponse> {
        let mut store = store.write().await;
        let args: LRemCommandInput = match args.try_into() {
            Err(_) => { return None; }
            Ok(doc) => doc
        };

        let rsp = MessageResponse {
            content: Some(Bson::Int64(store.lrem(args.list, args.count, args.value) as i64)),
            status: OperationStatus::Success,
        };
        Some(rsp)
    }

    async fn post_exec(&mut self, _connection: &mut Connection, _response: Option<&MessageResponse>) {}
}

pub struct LsetCommand {}

#[async_trait]
impl Command for LsetCommand {
    async fn pre_exec(&mut self, _connection: &Connection, _encrypted: bool) -> bool { true }

    async fn execute(&mut self, store: Arc<RwLock<Store>>, args: Bson, _message: &Message) -> Option<MessageResponse> {
        let mut store = store.write().await;
        let args: LSetCommandInput = match args.try_into() {
            Err(_) => { return None; }
            Ok(doc) => doc
        };

        let rsp = MessageResponse {
            content: Some(Bson::Boolean(store.lset(args.list, args.index, args.value))),
            status: OperationStatus::Success,
        };
        Some(rsp)
    }

    async fn post_exec(&mut self, _connection: &mut Connection, _response: Option<&MessageResponse>) {}
}

pub struct LtrimCommand {}

#[async_trait]
impl Command for LtrimCommand {
    async fn pre_exec(&mut self, _connection: &Connection, _encrypted: bool) -> bool { true }

    async fn execute(&mut self, store: Arc<RwLock<Store>>, args: Bson, _message: &Message) -> Option<MessageResponse> {
        let mut store = store.write().await;
        let args: LTrimCommandInput = match args.try_into() {
            Err(_) => { return None; }
            Ok(doc) => doc
        };

        let rsp = if store.ltrim(args.list, args.start, args.stop) {
            MessageResponse {
                content: None,
                status: OperationStatus::Success,
            }
        } else {
            MessageResponse {
                content: None,
                status: OperationStatus::Failure,
            }
        };
        Some(rsp)
    }

    async fn post_exec(&mut self, _connection: &mut Connection, _response: Option<&MessageResponse>) {}
}

pub struct RpopCommand {}

#[async_trait]
impl Command for RpopCommand {
    async fn pre_exec(&mut self, _connection: &Connection, _encrypted: bool) -> bool { true }

    async fn execute(&mut self, store: Arc<RwLock<Store>>, args: Bson, _message: &Message) -> Option<MessageResponse> {
        let mut store = store.write().await;
        let args: RPopCommandInput = match args.try_into() {
            Err(_) => { return None; }
            Ok(doc) => doc
        };

        let rsp = match store.rpop(args.list, args.count) {
            None => {
                MessageResponse {
                    content: None,
                    status: OperationStatus::Failure,
                }
            }
            Some(val) => {
                MessageResponse {
                    content: Some(Bson::Array(val.iter().map(|x| Bson::String(x.to_string())).collect())),
                    status: OperationStatus::Success,
                }
            }
        };
        Some(rsp)
    }

    async fn post_exec(&mut self, _connection: &mut Connection, _response: Option<&MessageResponse>) {}
}

pub struct RpushCommand {}

#[async_trait]
impl Command for RpushCommand {
    async fn pre_exec(&mut self, _connection: &Connection, _encrypted: bool) -> bool { true }

    async fn execute(&mut self, store: Arc<RwLock<Store>>, args: Bson, _message: &Message) -> Option<MessageResponse> {
        let mut store = store.write().await;
        let args: RPushCommandInput = match args.try_into() {
            Err(_) => { return None; }
            Ok(doc) => doc
        };

        let rsp = match store.rpush(args.list, args.values) {
            Err(_err) => {
                MessageResponse {
                    content: None,
                    status: OperationStatus::OutOfMemory,
                }
            }
            Ok(_) => {
                MessageResponse {
                    content: None,
                    status: OperationStatus::Success,
                }
            }
        };
        Some(rsp)
    }

    async fn post_exec(&mut self, _connection: &mut Connection, _response: Option<&MessageResponse>) {}
}

pub struct RpushxCommand {}

#[async_trait]
impl Command for RpushxCommand {
    async fn pre_exec(&mut self, _connection: &Connection, _encrypted: bool) -> bool { true }

    async fn execute(&mut self, store: Arc<RwLock<Store>>, args: Bson, _message: &Message) -> Option<MessageResponse> {
        let mut store = store.write().await;
        let args: RPushxCommandInput = match args.try_into() {
            Err(_) => { return None; }
            Ok(doc) => doc
        };

        let rsp = match store.rpushx(args.list, args.values) {
            Err(_err) => {
                MessageResponse {
                    content: None,
                    status: OperationStatus::OutOfMemory,
                }
            }
            Ok(_) => {
                MessageResponse {
                    content: None,
                    status: OperationStatus::Success,
                }
            }
        };
        Some(rsp)
    }

    async fn post_exec(&mut self, _connection: &mut Connection, _response: Option<&MessageResponse>) {}
}
