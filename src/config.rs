use std::net::IpAddr;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Hash, Debug, Default, Serialize, Deserialize)]
pub struct ConfigUser {
    pub name: String,
    pub password: String,
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Hash, Debug, Default, Serialize, Deserialize)]
pub struct ConfigAcl {
    /// The name of the user that this ACL applies to
    pub name: String,
    /// The commands that the user is allowed to execute
    pub commands: Vec<String>,
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Hash, Debug, Default, Serialize, Deserialize)]
pub struct Config {
    /// The users that are allowed to connect to the server
    pub users: Vec<ConfigUser>,
    /// The ACLs for users
    ///
    /// Format
    /// ```yaml
    /// acls:
    ///  - name: "user1"
    ///    commands:
    ///     - "HGET"
    ///     - "HSET"
    /// ```
    /// It's always allowed to send the KEYEXCHANGE, HEARTBEAT and LOGIN Messages
    pub acls: Vec<ConfigAcl>,
    /// The port that the server will listen on
    /// Can be overridden by the CLI
    pub port: Option<u16>,
    /// The host that the server will listen on
    /// Can be overridden by the CLI
    pub host: Option<IpAddr>,
    /// The path to the server's age private key
    /// Can be overridden by the CLI
    pub private_key_loc: Option<String>,
    /// The effort to put into brotli compression. Needs to be between 0 and 11
    /// Can be overridden by the CLI
    pub brotli_quality: Option<u8>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            users: vec![],
            acls: vec![],
            port: None,
            host: None,
            private_key_loc: None,
            brotli_quality: None,
        }
    }

    pub fn save(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let file = std::fs::File::create(path)?;
        serde_yaml::to_writer(file, self)?;
        Ok(())
    }
}
