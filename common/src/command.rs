use serde::{Deserialize, Serialize};
use crate::acl::CommandID;


#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum ACLOperation {
    Set {
        user: String,
        command: CommandID,
    },
    Remove {
        user: String,
        command: CommandID,
    },
    List {
        user: String,
    },
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Command {
    Get {
        key: String,
        default: Option<String>,
    },
    Set {
        key: String,
        value: String,
    },
    Delete {
        key: String,
    },
    Heartbeat,
    ACL {
        op: ACLOperation
    },
    Login {
        user: String,
        password: String,
    },
    HGET {
        key: String,
        field: String,
    },
    HSET {
        key: String,
        value: std::collections::HashMap<String, String>,
    },
    HDEL {
        key: String,
        field: String,
    },
    HGETALL {
        key: String,
    },
    HKEYS {
        key: String,
    },
    HVALS {
        key: String,
    },
    HLEN {
        key: String,
    },
    HEXISTS {
        key: String,
        field: String,
    },
    HINCRBY {
        key: String,
        field: String,
        value: i64,
    },
    HSTRLEN {
        key: String,
        field: String,
    },
    KEYEXCHANGE {
        pub_key: String,
    }
}


impl TryFrom<Vec<u8>> for Command {
    type Error = String;

    fn try_from(buf: Vec<u8>) -> Result<Command, Self::Error> {
        let buf = std::str::from_utf8(&buf).unwrap().replace('\0', "");
        let mut parts = buf.splitn(3, ' ');
        let part = parts.next().unwrap().trim();
        match part {
            "GET" => {
                let key = match parts.next() {
                    Some(key) => key.to_string(),
                    None => return Err("Key not provided".to_string()),
                };
                let default = parts.next().map(|s| s.to_string());
                Ok(Command::Get { key, default })
            }
            "SET" => {
                let key = match parts.next() {
                    None => return Err("Key not provided".to_string()),
                    Some(key) => key.to_string()
                };
                let value = match parts.next() {
                    None => return Err("Value not provided".to_string()),
                    Some(value) => value.to_string()
                };
                Ok(Command::Set { key, value })
            }
            "HEARTBEAT" => Ok(Command::Heartbeat),
            "DELETE" => {
                let key = match parts.next() {
                    None => return Err("Key not provided".to_string()),
                    Some(key) => key.to_string()
                };
                Ok(Command::Delete { key })
            },
            "ACL" => {
                let op = match parts.next() {
                    None => return Err("Operation not provided".to_string()),
                    Some(op) => op.to_string()
                };
                let mut op_parts = op.splitn(3, ' ');
                let op = match op_parts.next() {
                    None => return Err("Operation not provided".to_string()),
                    Some(op) => op
                };
                let user = match op_parts.next() {
                    None => return Err("User not provided".to_string()),
                    Some(user) => user.to_string()
                };
                match op {
                    "SET" => {
                        let command = match op_parts.next() {
                            None => return Err("Command not provided".to_string()),
                            Some(cmd) => {
                                match cmd.parse() { 
                                    Ok(cmd) => cmd,
                                    Err(_) => return Err("Invalid command".to_string())
                                }
                            }
                        };
                        Ok(Command::ACL { op: ACLOperation::Set { user, command } })
                    }
                    "REMOVE" => {
                        let command = match op_parts.next() {
                            None => return Err("Command not provided".to_string()),
                            Some(cmd) => {
                                match cmd.parse() {
                                    Ok(cmd) => cmd,
                                    Err(_) => return Err("Invalid command".to_string())
                                }
                            }
                        };
                        Ok(Command::ACL { op: ACLOperation::Remove { user, command } })
                    }
                    "LIST" => {
                        Ok(Command::ACL { op: ACLOperation::List { user } })
                    }
                    _ => Err(format!("Invalid ACL operation: {}", op)),
                }
            },
            "LOGIN" => {
                let user = match parts.next() {
                    None => return Err("User not provided".to_string()),
                    Some(user) => user.to_string()
                };
                let password = match parts.next() {
                    None => return Err("Password not provided".to_string()),
                    Some(password) => password.to_string()
                };
                Ok(Command::Login { user, password })
            },
            "HGET" => {
                let key = match parts.next() {
                    None => return Err("Key not provided".to_string()),
                    Some(key) => key.to_string()
                };
                let field = match parts.next() {
                    None => return Err("Field not provided".to_string()),
                    Some(field) => field.to_string()
                };
                Ok(Command::HGET { key, field })
            },
            "HSET" => {
                let key = match parts.next() {
                    None => return Err("Key not provided".to_string()),
                    Some(key) => key.to_string()
                };
                let mut value = std::collections::HashMap::new();
                for part in parts {
                    let mut kv = part.splitn(2, '=');
                    let k = match kv.next() {
                        None => return Err("Invalid key-value pair".to_string()),
                        Some(k) => k.to_string()
                    };
                    let v = match kv.next() {
                        None => return Err("Invalid key-value pair".to_string()),
                        Some(v) => v.to_string()
                    };
                    value.insert(k, v);
                }
                Ok(Command::HSET { key, value })
            },
            "HDEL" => {
                let key = match parts.next() {
                    None => return Err("Key not provided".to_string()),
                    Some(key) => key.to_string()
                };
                let field = match parts.next() {
                    None => return Err("Field not provided".to_string()),
                    Some(field) => field.to_string()
                };
                Ok(Command::HDEL { key, field })
            },
            "HGETALL" => {
                let key = match parts.next() {
                    None => return Err("Key not provided".to_string()),
                    Some(key) => key.to_string()
                };
                Ok(Command::HGETALL { key })
            },
            "HKEYS" => {
                let key = match parts.next() {
                    None => return Err("Key not provided".to_string()),
                    Some(key) => key.to_string()
                };
                Ok(Command::HKEYS { key })
            },
            "HVALS" => {
                let key = match parts.next() {
                    None => return Err("Key not provided".to_string()),
                    Some(key) => key.to_string()
                };
                Ok(Command::HVALS { key })
            },
            "HLEN" => {
                let key = match parts.next() {
                    None => return Err("Key not provided".to_string()),
                    Some(key) => key.to_string()
                };
                Ok(Command::HLEN { key })
            },
            "HEXISTS" => {
                let key = match parts.next() {
                    None => return Err("Key not provided".to_string()),
                    Some(key) => key.to_string()
                };
                let field = match parts.next() {
                    None => return Err("Field not provided".to_string()),
                    Some(field) => field.to_string()
                };
                Ok(Command::HEXISTS { key, field })
            },
            "HINCRBY" => {
                let key = match parts.next() {
                    None => return Err("Key not provided".to_string()),
                    Some(key) => key.to_string()
                };
                let field = match parts.next() {
                    None => return Err("Field not provided".to_string()),
                    Some(field) => field.to_string()
                };
                let value = match parts.next() {
                    None => return Err("Value not provided".to_string()),
                    Some(value) => {
                        match value.parse() {
                            Ok(value) => value,
                            Err(_) => return Err("Invalid value".to_string())
                        }
                    }
                };
                Ok(Command::HINCRBY { key, field, value })
            },
            "HSTRLEN" => {
                let key = match parts.next() {
                    None => return Err("Key not provided".to_string()),
                    Some(key) => key.to_string()
                };
                let field = match parts.next() {
                    None => return Err("Field not provided".to_string()),
                    Some(field) => field.to_string()
                };
                Ok(Command::HSTRLEN { key, field })
            },
            _ => Err(format!("Invalid command: {}", part)),
        }
    }
}

impl TryFrom<&str> for Command {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let val = value.as_bytes();
        let mut buf = Vec::with_capacity(val.len());
        buf[..val.len()].copy_from_slice(val);
        return Self::try_from(buf);
    }
}

impl Command {
    pub fn to_id(&self) -> CommandID {
        match self {
            Command::Get { .. } => 0,
            Command::Set { .. } => 1,
            Command::Delete { .. } => 2,
            Command::Heartbeat => 3,
            Command::ACL { .. } => 4,
            Command::Login { .. } => 5,
            Command::HGET { .. } => 6,
            Command::HSET { .. } => 7,
            Command::HDEL { .. } => 8,
            Command::HGETALL { .. } => 9,
            Command::HKEYS { .. } => 10,
            Command::HVALS { .. } => 11,
            Command::HLEN { .. } => 12,
            Command::HEXISTS { .. } => 13,
            Command::HINCRBY { .. } => 14,
            Command::HSTRLEN { .. } => 15,
            Command::KEYEXCHANGE { .. } => 16,
        }
    }
}
