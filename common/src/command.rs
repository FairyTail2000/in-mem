use serde::{Deserialize, Serialize};

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
                let key = parts.next().unwrap().to_string();
                let default = parts.next().map(|s| s.to_string());
                Ok(Command::Get { key, default })
            }
            "SET" => {
                let key = parts.next().unwrap().to_string();
                let value = parts.next().unwrap().to_string();
                Ok(Command::Set { key, value })
            }
            "HEARTBEAT" => Ok(Command::Heartbeat),
            "DELETE" => {
                let key = parts.next().unwrap().to_string();
                Ok(Command::Delete { key })
            }
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
    pub fn as_bytes(&self) -> Vec<u8> {
        match self {
            Command::Get { key, default } => {
                let buf = match default {
                    None => {
                        format!("GET {}", key)
                    }
                    Some(default) => {
                        format!("GET {} {}", key, default)
                    }
                };
                buf.into_bytes()
            }
            Command::Set { key, value } => {
                format!("SET {} {}\n", key, value).into_bytes()
            },
            Command::Heartbeat => {
                String::from("HEARTBEAT").into_bytes()
            },
            Command::Delete { key } => {
                format!("DELETE {}", key).into_bytes()
            }
        }
    }
}