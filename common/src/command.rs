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

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
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
}

const BUFFER_SIZE: usize = 1024;

impl TryFrom<[u8; BUFFER_SIZE]> for Command {
    type Error = String;

    fn try_from(buf: [u8; BUFFER_SIZE]) -> Result<Command, Self::Error> {
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
            _ => Err(format!("Invalid command: {}", part)),
        }
    }
}

impl TryFrom<&str> for Command {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let val = value.as_bytes();
        let mut buf = [0u8; BUFFER_SIZE];
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
        }
    }
}
