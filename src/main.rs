use std::collections::HashMap;
use std::fs::create_dir_all;
use std::io::prelude::Read;
use std::net::{IpAddr, SocketAddr};
use std::path::{MAIN_SEPARATOR, PathBuf};
use std::str::FromStr;
use std::sync::Arc;

use age::secrecy::ExposeSecret;
use age::x25519::{Identity, Recipient};
use clap::Parser;
use directories::ProjectDirs;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use uuid::Uuid;

use common::command::{CommandID, str_to_command_id};
use common::connection::Connection;
use common::init_env_logger;
use common::message::{Message, MessageContent, MessageResponse, OperationStatus};

use crate::commands::{GetCommand, SetCommand, DeleteCommand, HeartbeatCommand, AclListCommand, AclSetCommand, AclRemoveCommand, LoginCommand, KeyExchangeCommand, HashMapGetCommand, HashMapSetCommand, HashMapDeleteCommand, HashMapKeysCommand, HashMapValuesCommand, HashMapLenCommand, HashMapExistsCommand, HashMapGetAllCommand, HashMapIncrByCommand, HashMapStringLenCommand, HashMapUpsertCommand};
use crate::store::{ACLAble, Store, UserAble};

mod store;
mod config;
mod commands;

#[derive(Parser, Debug)]
#[command(name = "in-mem", version = "1.0", about = "A small in mem server")]
struct Cli {
    /// Name of the person to greet
    #[arg(short, long, default_value = "6", env = "BROTLI_EFFORT", help = "Brotli compression effort level, 0-11", value_parser = clap::value_parser ! (u8).range(0..12))]
    brotli_effort: u8,
    /// The host to bind to
    #[arg(default_value = "127.0.0.1", env = "HOST", help = "The host to bind to")]
    host: IpAddr,
    /// The port to bind to
    #[arg(env = "PORT", help = "The port to bind to")]
    port: Option<u16>,
    /// The private key location
    #[arg(env = "PRIVATE_KEY", help = "The location of the private key")]
    private_key_loc: Option<String>,
}

async fn handle_message(message: Message, connection: &mut Connection, store: &Arc<RwLock<Store>>, encrypted: bool, rsp_id: Uuid, command_registry: &mut HashMap<CommandID, Box<dyn commands::Command>>) -> Option<Message> {
    let original_message = message.clone();
    return match message.content {
        MessageContent::Command(cmd) => {
            log::trace!("Received command: {:?}", cmd);
            let cmd_id: CommandID = cmd.command_id.try_into().unwrap();
            // Check if the command is allowed
            {
                let store = store.read().await;
                if store.acl_is_allowed(&connection.get_user().unwrap_or_else(|| "".to_string()), cmd_id) {
                    log::trace!("Command allowed: {:?}", cmd_id);
                } else {
                    log::error!("Command not allowed: {:?}", cmd_id);
                    let rsp = Message::new_response(rsp_id, MessageResponse {
                        content: None,
                        status: OperationStatus::NotAllowed,
                    });
                    return Some(rsp);
                }
            }


            let rsvp = command_registry.get_mut(&cmd_id);
            match rsvp {
                Some(handler) => {
                    let early_exit = handler.pre_exec(connection, encrypted).await;
                    if !early_exit {
                        let rsp = Message::new_response(rsp_id, MessageResponse {
                            content: None,
                            status: OperationStatus::Failure,
                        });
                        return Some(rsp);
                    }

                    let result = handler.execute(store.clone(), cmd.payload, &original_message).await;
                    handler.post_exec(connection, result.as_ref()).await;
                    match result {
                        Some(result) => {
                            Some(Message::new_response(rsp_id, result))
                        }
                        None => {
                            log::error!("Error executing command: {:?}", cmd.command_id);
                            None
                        }
                    }
                }
                None => {
                    log::error!("Received unknown command: {:?}", cmd.command_id);
                    return None;
                }
            }
        }
        MessageContent::Response(_) => {
            log::error!("Received unexpected response from client: {}", connection.get_id());
            None
        }
    };
}

async fn worker_loop(mut connection: Connection, store: Arc<RwLock<Store>>, key: Identity) {
    let mut command_registry = populate_command_registry();
    loop {
        match connection.read_message(&key).await {
            Ok((message, encrypted)) => {
                log::trace!("Read from socket: {}", connection.get_id());
                let rsp_id = Uuid::new_v4();
                let resp = handle_message(message, &mut connection, &store, encrypted, rsp_id, &mut command_registry).await;
                match resp {
                    None => {
                        log::trace!("Closing connection: {}, Client behaved badly", connection.get_id());
                        connection.close();
                        break;
                    }
                    Some(rsp) => {
                        match connection.send_message(&rsp).await {
                            Ok(_) => {}
                            Err(err) => {
                                log::error!("Error sending response: {}", err);
                                connection.close();
                                break;
                            }
                        };
                    }
                }
            }
            Err(err) => {
                log::error!("Error reading from socket: {}", err);
                connection.close();
            }
        }
    }
}

async fn socket_listener(host: IpAddr, port: u16, brotli_effort: u8, store: Arc<RwLock<Store>>, key: Identity) {
    let addr = SocketAddr::from((host, port));
    log::info!("Starting server on tcp://{}", addr);
    let listener = match TcpListener::bind(&addr).await {
        Ok(listener) => listener,
        Err(err) => {
            log::error!("Error binding to {}: {}", addr, err);
            return;
        }
    };
    loop {
        let (socket, info) = match listener.accept().await {
            Ok(res) => res,
            Err(err) => {
                log::error!("Error accepting connection: {}", err);
                continue;
            }
        };
        log::debug!("Accepted connection from: {}", info);
        let connection = Connection::new(socket, Uuid::new_v4(), brotli_effort);
        let store = store.clone();
        let key = key.clone();
        tokio::spawn(async move {
            worker_loop(connection, store, key).await;
        });
    }
}


fn config_path(file: &str) -> PathBuf {
    let path = match ProjectDirs::from("", "", "in-mem") {
        None => PathBuf::from(format!(".{}{}", MAIN_SEPARATOR, file)),
        Some(dirs) => {
            if !dirs.data_dir().exists() {
                match create_dir_all(dirs.data_dir()) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("{}", e);
                        std::process::exit(-1);
                    }
                }
            }
            PathBuf::from(format!(
                "{}{}{}",
                dirs.data_dir().to_str().unwrap(),
                MAIN_SEPARATOR,
                file
            ))
        }
    };

    path
}

fn merge_config(config: config::Config, cli: Cli) -> config::Config {
    let mut config = config;
    // The CLI Args override the config file
    // Therefore if port is in the config and different to the CLI port, we override it or if it's just not there
    if config.port.is_some_and(|x| x != cli.port.unwrap_or(3000)) || config.port.is_none() {
        config.port = Some(cli.port.unwrap_or(3000));
    }
    // Same goes for host
    if config.host.is_some_and(|x| x != cli.host) || config.host.is_none() {
        config.host = Some(cli.host);
    }
    // Same goes for brotli_effort
    if config.brotli_quality.is_some_and(|x| x != cli.brotli_effort) || config.brotli_quality.is_none() {
        config.brotli_quality = Some(cli.brotli_effort);
    }
    // And for the private key loc
    // This is a little more lengthy because Strings cannot be copied like numbers
    if config.private_key_loc.clone().is_some_and(|x| x != cli.private_key_loc.clone().unwrap_or(String::from("server-identity.age"))) || config.private_key_loc.is_none() {
        config.private_key_loc = cli.private_key_loc.map_or_else(|| Some(String::from("server-identity.age")), |x| Some(x));
    }
    config
}

fn populate_command_registry() -> HashMap<CommandID, Box<dyn commands::Command>> {
    let mut registry: HashMap<CommandID, Box<dyn commands::Command>> = HashMap::new();
    registry.insert(CommandID::Get, Box::new(GetCommand {}));
    registry.insert(CommandID::Set, Box::new(SetCommand {}));
    registry.insert(CommandID::Delete, Box::new(DeleteCommand {}));
    registry.insert(CommandID::Heartbeat, Box::new(HeartbeatCommand {}));
    registry.insert(CommandID::AclList, Box::new(AclListCommand {}));
    registry.insert(CommandID::AclSet, Box::new(AclSetCommand {}));
    registry.insert(CommandID::AclRemove, Box::new(AclRemoveCommand {}));
    registry.insert(CommandID::Login, Box::new(LoginCommand::default()));
    registry.insert(CommandID::KEYEXCHANGE, Box::new(KeyExchangeCommand::default()));
    registry.insert(CommandID::HGET, Box::new(HashMapGetCommand {}));
    registry.insert(CommandID::HSET, Box::new(HashMapSetCommand {}));
    registry.insert(CommandID::HDEL, Box::new(HashMapDeleteCommand {}));
    registry.insert(CommandID::HKEYS, Box::new(HashMapKeysCommand {}));
    registry.insert(CommandID::HVALS, Box::new(HashMapValuesCommand {}));
    registry.insert(CommandID::HLEN, Box::new(HashMapLenCommand {}));
    registry.insert(CommandID::HGETALL, Box::new(HashMapGetAllCommand {}));
    registry.insert(CommandID::HEXISTS, Box::new(HashMapExistsCommand {}));
    registry.insert(CommandID::HINCRBY, Box::new(HashMapIncrByCommand {}));
    registry.insert(CommandID::HSTRLEN, Box::new(HashMapStringLenCommand {}));
    registry.insert(CommandID::HUPSERT, Box::new(HashMapUpsertCommand {}));
    registry.insert(CommandID::UserRemove, Box::new(commands::UserRemoveCommand {}));

    return registry;
}

#[tokio::main]
async fn main() {
    init_env_logger();

    let cli = Cli::parse();

    let config_path = config_path("config.yaml");
    log::debug!("Using config file: {}", config_path.display());
    let config_string = std::fs::read_to_string(config_path.clone());
    let config = match config_string {
        Ok(config) => {
            let config = match serde_yaml::from_str(&config) {
                Ok(config) => config,
                Err(err) => {
                    log::error!("Error parsing config file: {}", err);
                    std::process::exit(-1);
                }
            };
            config
        }
        Err(_) => {
            log::warn!("No config file found or not readable. Using default config");
            let conf = config::Config::default();
            // Save the default config
            let parent = config_path.parent().unwrap();
            if !parent.exists() {
                match create_dir_all(parent) {
                    Ok(_) => {}
                    Err(err) => {
                        log::error!("Error creating config directory: {}", err);
                        std::process::exit(-1);
                    }
                }
            }
            conf.save(&config_path).unwrap();
            conf
        }
    };
    let config = merge_config(config, cli);
    // config.private_key_loc will be some, because it's set in the merging if it's not there
    log::debug!("Loading private key: {:?}", config.private_key_loc);
    let private_key = match std::fs::File::open(config.private_key_loc.clone().unwrap()) {
        Ok(mut file) => {
            let mut buf = Vec::new();
            match file.read_to_end(&mut buf) {
                Ok(_) => {}
                Err(err) => {
                    log::error!("Error reading identity file: {}", err);
                    std::process::exit(-1);
                }
            }
            match std::str::from_utf8(&buf) {
                Ok(read) => {
                    match Identity::from_str(read) {
                        Ok(key) => key,
                        Err(err) => {
                            log::error!("Error parsing identity file: {}", err);
                            std::process::exit(-1);
                        }
                    }
                }
                Err(err) => {
                    log::error!("Error parsing identity file: {}", err);
                    std::process::exit(-1);
                }
            }
        }
        Err(_) => {
            log::warn!("No identity file found or not readable. Generating new identity file");
            let key = Identity::generate();
            match std::fs::write(config.private_key_loc.unwrap(), key.to_string().expose_secret()) {
                Ok(_) => {}
                Err(err) => {
                    log::error!("Error writing identity file: {}", err);
                    std::process::exit(-1);
                }
            }
            key
        }
    };
    let public_key = private_key.to_public();
    log::info!("Public key: \"{}\"", public_key);
    let store = Arc::new(RwLock::new(Store::default()));

    let mut locked = store.write().await;
    for user in config.users {
        if user.name.is_empty() {
            log::warn!("User has no name. Skipping");
            continue;
        }
        if user.password.is_empty() {
            log::warn!("User {} has no password. Skipping", user.name);
            continue;
        }
        if user.password.len() != 128 {
            log::warn!("User {} has a password that is not hashed with sha512. Skipping", user.name);
            continue;
        }
        if user.acls.is_empty() {
            log::warn!("User {} has no acls. Continuing anyway", user.name);
        }
        match user.public_key {
            None => {
                log::debug!("Adding user without public key: {}", user.name);
                locked.user_add(&user.name, &user.password, None);
            }
            Some(key_str) => {
                match Recipient::from_str(&key_str) {
                    Ok(key) => {
                        log::debug!("Adding user with public key: {}", user.name);
                        locked.user_add(&user.name, &user.password, Some(key));
                    }
                    Err(err) => {
                        log::warn!("Error parsing public key. Not adding it: {}", err);
                    }
                }
            }
        }
        for acl in user.acls {
            let command = str_to_command_id(acl);
            match command {
                Ok(command) => {
                    locked.acl_add(&user.name, command)
                }
                Err(err) => {
                    log::warn!("Error parsing command: {}", err);
                }
            }
        }
    }
    drop(locked);

    socket_listener(config.host.unwrap(), config.port.unwrap(), config.brotli_quality.unwrap(), store, private_key).await;
}
