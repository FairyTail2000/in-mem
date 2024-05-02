use std::net::IpAddr;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Hash, Debug, Default, Serialize, Deserialize)]
pub struct ConfigUser {
    pub name: String,
    /// The password that the user will use to authenticate
    /// The password is hashed with sha512
    /// Not hashing it in the config file will result in the user not being loaded
    pub password: String,
    /// The public key of the user
    /// The public key is used to ensure that the user is who they say they are. So setting this effectively removes MITM attacks
    pub public_key: Option<String>,
    /// ACLs that the user has
    /// A list of commands the user is allowed to execute
    pub acls: Vec<String>,
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Hash, Debug, Default, Serialize, Deserialize)]
pub struct Config {
    /// The users that are allowed to connect to the server
    ///
    /// Format
    /// ```yaml
    /// users:
    /// - name: "user1"
    ///   acls:
    ///     - "HGET"
    ///     - "HSET"
    /// ```
    /// It's always allowed to send the KEYEXCHANGE, HEARTBEAT and LOGIN Messages
    pub users: Vec<ConfigUser>,
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
    pub fn save(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let file = std::fs::File::create(path)?;
        serde_yaml::to_writer(file, self)?;
        Ok(())
    }
}
