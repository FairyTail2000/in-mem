use bson::Bson;
use serde::{Deserialize, Serialize};
use crate::command::CommandID;

#[derive(Debug, Deserialize, Serialize)]
pub struct AclSetCommandInput {
    pub user: String,
    pub command: CommandID,
}

impl TryFrom<Bson> for AclSetCommandInput {
    type Error = bson::de::Error;

    fn try_from(bson: Bson) -> Result<Self, Self::Error> {
        bson::from_bson(bson)
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct AclRemoveCommandInput {
    pub user: String,
    pub command: CommandID,
}

impl TryFrom<Bson> for AclRemoveCommandInput {
    type Error = bson::de::Error;

    fn try_from(bson: Bson) -> Result<Self, Self::Error> {
        bson::from_bson(bson)
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct AclListCommandInput {
    pub user: String,
    pub command: CommandID,
}

impl TryFrom<Bson> for AclListCommandInput {
    type Error = bson::de::Error;

    fn try_from(bson: Bson) -> Result<Self, Self::Error> {
        bson::from_bson(bson)
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct DeleteCommandInput {
    pub key: String,
}

impl TryFrom<Bson> for DeleteCommandInput {
    type Error = bson::de::Error;

    fn try_from(bson: Bson) -> Result<Self, Self::Error> {
        bson::from_bson(bson)
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct SetCommandInput {
    pub key: String,
    pub value: String,
}

impl TryFrom<Bson> for SetCommandInput {
    type Error = bson::de::Error;

    fn try_from(bson: Bson) -> Result<Self, Self::Error> {
        bson::from_bson(bson)
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct LoginCommandInput {
    pub user: String,
    pub password: String,
}

impl TryFrom<Bson> for LoginCommandInput {
    type Error = bson::de::Error;

    fn try_from(bson: Bson) -> Result<Self, Self::Error> {
        bson::from_bson(bson)
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct KeyExchangeCommandInput {
    pub pub_key: String,
}

impl TryFrom<Bson> for KeyExchangeCommandInput {
    type Error = bson::de::Error;

    fn try_from(bson: Bson) -> Result<Self, Self::Error> {
        bson::from_bson(bson)
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct HashMapDeleteCommandInput {
    pub key: String,
    pub field: String,
}

impl TryFrom<Bson> for HashMapDeleteCommandInput {
    type Error = bson::de::Error;

    fn try_from(bson: Bson) -> Result<Self, Self::Error> {
        bson::from_bson(bson)
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct HashMapGetCommandInput {
    pub key: String,
    pub field: String,
}

impl TryFrom<Bson> for HashMapGetCommandInput {
    type Error = bson::de::Error;

    fn try_from(bson: Bson) -> Result<Self, Self::Error> {
        bson::from_bson(bson)
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct HashMapSetCommandInput {
    pub key: String,
    pub value: std::collections::HashMap<String, String>,
}

impl TryFrom<Bson> for HashMapSetCommandInput {
    type Error = bson::de::Error;

    fn try_from(bson: Bson) -> Result<Self, Self::Error> {
        bson::from_bson(bson)
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct HashMapGetAllCommandInput {
    pub key: String,
    pub field: String,
}

impl TryFrom<Bson> for HashMapGetAllCommandInput {
    type Error = bson::de::Error;

    fn try_from(bson: Bson) -> Result<Self, Self::Error> {
        bson::from_bson(bson)
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct HashMapKeysCommandInput {
    pub key: String,
    pub field: String,
}

impl TryFrom<Bson> for HashMapKeysCommandInput {
    type Error = bson::de::Error;

    fn try_from(bson: Bson) -> Result<Self, Self::Error> {
        bson::from_bson(bson)
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct HashMapLenCommandInput {
    pub key: String,
}

impl TryFrom<Bson> for HashMapLenCommandInput {
    type Error = bson::de::Error;

    fn try_from(bson: Bson) -> Result<Self, Self::Error> {
        bson::from_bson(bson)
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct HashMapValuesCommandInput {
    pub key: String,
}

impl TryFrom<Bson> for HashMapValuesCommandInput {
    type Error = bson::de::Error;

    fn try_from(bson: Bson) -> Result<Self, Self::Error> {
        bson::from_bson(bson)
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct HashMapExistsCommandInput {
    pub key: String,
    pub field: String,
}

impl TryFrom<Bson> for HashMapExistsCommandInput {
    type Error = bson::de::Error;

    fn try_from(bson: Bson) -> Result<Self, Self::Error> {
        bson::from_bson(bson)
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct HashMapIncrByCommandInput {
    pub key: String,
    pub field: String,
    pub value: i64,
}

impl TryFrom<Bson> for HashMapIncrByCommandInput {
    type Error = bson::de::Error;

    fn try_from(bson: Bson) -> Result<Self, Self::Error> {
        bson::from_bson(bson)
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct HashMapStringLenCommandInput {
    pub key: String,
    pub field: String,
}

impl TryFrom<Bson> for HashMapStringLenCommandInput {
    type Error = bson::de::Error;

    fn try_from(bson: Bson) -> Result<Self, Self::Error> {
        bson::from_bson(bson)
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct HashMapUpsertCommandInput {
    pub key: String,
    pub field: String,
    pub value: String,
}

impl TryFrom<Bson> for HashMapUpsertCommandInput {
    type Error = bson::de::Error;

    fn try_from(bson: Bson) -> Result<Self, Self::Error> {
        bson::from_bson(bson)
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct GetCommandInput {
    pub key: String,
    pub default: Option<String>,
}

impl TryFrom<Bson> for GetCommandInput {
    type Error = bson::de::Error;

    fn try_from(bson: Bson) -> Result<Self, Self::Error> {
        bson::from_bson(bson)
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct UserRemoveCommandInput {
    pub user: String,
}

impl TryFrom<Bson> for UserRemoveCommandInput {
    type Error = bson::de::Error;

    fn try_from(bson: Bson) -> Result<Self, Self::Error> {
        bson::from_bson(bson)
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct LLenCommandInput {
    pub list: String,
}

impl TryFrom<Bson> for LLenCommandInput {
    type Error = bson::de::Error;

    fn try_from(bson: Bson) -> Result<Self, Self::Error> {
        bson::from_bson(bson)
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct LIndexCommandInput {
    pub list: String,
    pub key: String,
}

impl TryFrom<Bson> for LIndexCommandInput {
    type Error = bson::de::Error;

    fn try_from(bson: Bson) -> Result<Self, Self::Error> {
        bson::from_bson(bson)
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct LMoveCommandInput {
    pub src: String,
    pub dest: String,
    pub left_right: String,
    pub right_left: String,
}

impl TryFrom<Bson> for LMoveCommandInput {
    type Error = bson::de::Error;

    fn try_from(bson: Bson) -> Result<Self, Self::Error> {
        bson::from_bson(bson)
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct LPopCommandInput {
    pub list: String,
    pub count: Option<usize>,
}

impl TryFrom<Bson> for LPopCommandInput {
    type Error = bson::de::Error;

    fn try_from(bson: Bson) -> Result<Self, Self::Error> {
        bson::from_bson(bson)
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct LPosCommandInput {
    pub list: String,
    pub value: String,
    pub rank: Option<isize>,
    pub count: Option<usize>,
    pub max_len: Option<usize>,
}

impl TryFrom<Bson> for LPosCommandInput {
    type Error = bson::de::Error;

    fn try_from(bson: Bson) -> Result<Self, Self::Error> {
        bson::from_bson(bson)
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct LPushCommandInput {
    pub list: String,
    pub values: Vec<String>,
}

impl TryFrom<Bson> for LPushCommandInput {
    type Error = bson::de::Error;

    fn try_from(bson: Bson) -> Result<Self, Self::Error> {
        bson::from_bson(bson)
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct LPushxCommandInput {
    pub list: String,
    pub values: Vec<String>,
}

impl TryFrom<Bson> for LPushxCommandInput {
    type Error = bson::de::Error;

    fn try_from(bson: Bson) -> Result<Self, Self::Error> {
        bson::from_bson(bson)
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct LRangeCommandInput {
    pub list: String,
    pub start: isize,
    pub stop: isize,
}

impl TryFrom<Bson> for LRangeCommandInput {
    type Error = bson::de::Error;

    fn try_from(bson: Bson) -> Result<Self, Self::Error> {
        bson::from_bson(bson)
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct LRemCommandInput {
    pub list: String,
    pub count: isize,
    pub value: String,
}

impl TryFrom<Bson> for LRemCommandInput {
    type Error = bson::de::Error;

    fn try_from(bson: Bson) -> Result<Self, Self::Error> {
        bson::from_bson(bson)
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct LSetCommandInput {
    pub list: String,
    pub index: isize,
    pub value: String,
}

impl TryFrom<Bson> for LSetCommandInput {
    type Error = bson::de::Error;

    fn try_from(bson: Bson) -> Result<Self, Self::Error> {
        bson::from_bson(bson)
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct LTrimCommandInput {
    pub list: String,
    pub start: isize,
    pub stop: isize,
}

impl TryFrom<Bson> for LTrimCommandInput {
    type Error = bson::de::Error;

    fn try_from(bson: Bson) -> Result<Self, Self::Error> {
        bson::from_bson(bson)
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct RPopCommandInput {
    pub list: String,
    pub count: Option<usize>,
}

impl TryFrom<Bson> for RPopCommandInput {
    type Error = bson::de::Error;

    fn try_from(bson: Bson) -> Result<Self, Self::Error> {
        bson::from_bson(bson)
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct RPushCommandInput {
    pub list: String,
    pub values: Vec<String>,
}

impl TryFrom<Bson> for RPushCommandInput {
    type Error = bson::de::Error;

    fn try_from(bson: Bson) -> Result<Self, Self::Error> {
        bson::from_bson(bson)
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct RPushxCommandInput {
    pub list: String,
    pub values: Vec<String>,
}

impl TryFrom<Bson> for RPushxCommandInput {
    type Error = bson::de::Error;

    fn try_from(bson: Bson) -> Result<Self, Self::Error> {
        bson::from_bson(bson)
    }
}
