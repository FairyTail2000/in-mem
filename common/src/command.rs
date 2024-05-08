use std::fmt::Display;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub enum CommandID {
    Get = 0,
    Set = 1,
    Delete = 2,
    Heartbeat = 3,
    AclList = 4,
    AclSet = 5,
    AclRemove = 6,
    Login = 7,
    HGET = 8,
    HSET = 9,
    HDEL = 10,
    HGETALL = 11,
    HKEYS = 12,
    HVALS = 13,
    HLEN = 14,
    HEXISTS = 15,
    HINCRBY = 16,
    HSTRLEN = 17,
    KEYEXCHANGE = 18,
    HUPSERT = 19,
    UserRemove = 20,
}

impl Display for CommandID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            CommandID::Get => { "GET".to_string() }
            CommandID::Set => { "SET".to_string() }
            CommandID::Delete => { "DELETE".to_string() }
            CommandID::Heartbeat => { "HEARTBEAT".to_string() }
            CommandID::AclList => { "ACLList".to_string() }
            CommandID::AclSet => { "ACLSet".to_string() }
            CommandID::AclRemove => { "ACLRemove".to_string() }
            CommandID::Login => { "LOGIN".to_string() }
            CommandID::HGET => { "HGET".to_string() }
            CommandID::HSET => { "HSET".to_string() }
            CommandID::HDEL => { "HDEL".to_string() }
            CommandID::HGETALL => { "HGETALL".to_string() }
            CommandID::HKEYS => { "HKEYS".to_string() }
            CommandID::HVALS => { "HVALS".to_string() }
            CommandID::HLEN => { "HLEN".to_string() }
            CommandID::HEXISTS => { "HEXISTS".to_string() }
            CommandID::HINCRBY => { "HINCRBY".to_string() }
            CommandID::HSTRLEN => { "HSTRLEN".to_string() }
            CommandID::KEYEXCHANGE => { "KEYEXCHANGE".to_string() }
            CommandID::HUPSERT => { "HUPSERT".to_string() }
            CommandID::UserRemove => { "UserRemove".to_string() }
        };
        write!(f, "{}", str)
    }
}

impl TryFrom<u32> for CommandID {
    type Error = std::io::Error;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(CommandID::Get),
            1 => Ok(CommandID::Set),
            2 => Ok(CommandID::Delete),
            3 => Ok(CommandID::Heartbeat),
            4 => Ok(CommandID::AclList),
            5 => Ok(CommandID::AclSet),
            6 => Ok(CommandID::AclRemove),
            7 => Ok(CommandID::Login),
            8 => Ok(CommandID::HGET),
            9 => Ok(CommandID::HSET),
            10 => Ok(CommandID::HDEL),
            11 => Ok(CommandID::HGETALL),
            12 => Ok(CommandID::HKEYS),
            13 => Ok(CommandID::HVALS),
            14 => Ok(CommandID::HLEN),
            15 => Ok(CommandID::HEXISTS),
            16 => Ok(CommandID::HINCRBY),
            17 => Ok(CommandID::HSTRLEN),
            18 => Ok(CommandID::KEYEXCHANGE),
            19 => Ok(CommandID::HUPSERT),
            20 => Ok(CommandID::UserRemove),
            _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid command id")),
        }
    }
}

pub fn str_to_command_id(value: String) -> Result<CommandID, std::io::Error> {
    match &*value {
        "GET" => Ok(CommandID::Get),
        "SET" => Ok(CommandID::Set),
        "DELETE" => Ok(CommandID::Delete),
        "HEARTBEAT" => Ok(CommandID::Heartbeat),
        "ACLList" => Ok(CommandID::AclList),
        "ACLSet" => Ok(CommandID::AclSet),
        "ACLRemove" => Ok(CommandID::AclRemove),
        "LOGIN" => Ok(CommandID::Login),
        "HGET" => Ok(CommandID::HGET),
        "HSET" => Ok(CommandID::HSET),
        "HDEL" => Ok(CommandID::HDEL),
        "HGETALL" => Ok(CommandID::HGETALL),
        "HKEYS" => Ok(CommandID::HKEYS),
        "HVALS" => Ok(CommandID::HVALS),
        "HLEN" => Ok(CommandID::HLEN),
        "HEXISTS" => Ok(CommandID::HEXISTS),
        "HINCRBY" => Ok(CommandID::HINCRBY),
        "HSTRLEN" => Ok(CommandID::HSTRLEN),
        "KEYEXCHANGE" => Ok(CommandID::KEYEXCHANGE),
        "HUPSERT" => Ok(CommandID::HUPSERT),
        "UserRemove" => Ok(CommandID::UserRemove),
        _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid command id"))
    }
}