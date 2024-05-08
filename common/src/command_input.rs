use serde::{Deserialize, Serialize};
use crate::command::CommandID;

#[derive(Debug, Deserialize, Serialize)]
pub struct AclSetCommandInput {
    pub user: String,
    pub command: CommandID,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct AclRemoveCommandInput {
    pub user: String,
    pub command: CommandID,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct AclListCommandInput {
    pub user: String,
    pub command: CommandID,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct DeleteCommandInput {
    pub key: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct SetCommandInput {
    pub key: String,
    pub value: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct LoginCommandInput {
    pub user: String,
    pub password: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct KeyExchangeCommandInput {
    pub pub_key: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct HashMapDeleteCommandInput {
    pub key: String,
    pub field: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct HashMapGetCommandInput {
    pub key: String,
    pub field: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct HashMapSetCommandInput {
    pub key: String,
    pub value: std::collections::HashMap<String, String>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct HashMapGetAllCommandInput {
    pub key: String,
    pub field: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct HashMapKeysCommandInput {
    pub key: String,
    pub field: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct HashMapLenCommandInput {
    pub key: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct HashMapValuesCommandInput {
    pub key: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct HashMapExistsCommandInput {
    pub key: String,
    pub field: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct HashMapIncrByCommandInput {
    pub key: String,
    pub field: String,
    pub value: i64,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct HashMapStringLenCommandInput {
    pub key: String,
    pub field: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct HashMapUpsertCommandInput {
    pub key: String,
    pub field: String,
    pub value: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct GetCommandInput {
    pub key: String,
    pub default: Option<String>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct UserRemoveCommandInput {
    pub user: String,
}