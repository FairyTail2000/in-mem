use std::sync::Arc;
use async_trait::async_trait;

use tokio::sync::RwLock;

use common::message::{Message, MessageResponse};
use common::connection::Connection;

use crate::store::Store;

pub use basic::{GetCommand};
pub use basic::{SetCommand};
pub use basic::{DeleteCommand};
pub use heartbeat::HeartbeatCommand;
pub use acl::{AclListCommand};
pub use acl::{AclSetCommand};
pub use acl::{AclRemoveCommand};
pub use connection::{LoginCommand};
pub use connection::{KeyExchangeCommand};

pub use hashmap::HashMapGetCommand;
pub use hashmap::HashMapGetAllCommand;
pub use hashmap::HashMapSetCommand;
pub use hashmap::HashMapDeleteCommand;
pub use hashmap::HashMapKeysCommand;
pub use hashmap::HashMapValuesCommand;
pub use hashmap::HashMapLenCommand;
pub use hashmap::HashMapExistsCommand;
pub use hashmap::HashMapIncrByCommand;
pub use hashmap::HashMapStringLenCommand;
pub use hashmap::HashMapUpsertCommand;

pub use user::UserRemoveCommand;

mod basic;
mod hashmap;
mod heartbeat;
mod acl;
mod connection;
mod user;

#[async_trait]
pub trait Command: Send {
    /// Pre-checks for the command, like checking if the connection is encrypted
    /// Result determines if the command should be executed, otherwise an error is returned to the client
    async fn pre_exec(&mut self, connection: &Connection, encrypted: bool) -> bool;
    /// Executes the command
    async fn execute(&mut self, store: Arc<RwLock<Store>>, args: bson::Bson, message: &Message) -> Option<MessageResponse>;
    /// Post hook for the command, like logging the command, or cleaning up state
    /// Or setting connection parameters based on the state
    async fn post_exec(&mut self, connection: &mut Connection, response: Option<&MessageResponse>);
}