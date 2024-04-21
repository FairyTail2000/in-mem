use std::fs::create_dir_all;
use std::io::prelude::Read;
use std::net::{IpAddr, SocketAddr};
use std::path::{MAIN_SEPARATOR, PathBuf};
use std::str::FromStr;
use std::sync::Arc;

use age::secrecy::ExposeSecret;
use age::x25519::{Identity, Recipient};
use bson::{Bson, Document};
use clap::Parser;
use directories::ProjectDirs;
use sha2::{Digest, Sha512};
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use uuid::Uuid;

use common::command::{ACLOperation, Command, str_to_command_id};
use common::connection::Connection;
use common::init_env_logger;
use common::message::{Message, MessageContent, MessageResponse, OperationStatus};

use crate::store::{ACLAble, HashMapAble, Store, StoreAble, UserAble};

mod store;
mod config;

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

async fn handle_message<T>(message: Message, connection: &mut Connection, store: &Arc<RwLock<T>>, encrypted: bool, rsp_id: Uuid) -> Option<Message> where T: StoreAble + ACLAble + UserAble + Send + Sync + HashMapAble<String> {
    return match message.content {
        MessageContent::Command(cmd) => {
            log::trace!("Received command: {:?}", cmd);
            // Block scope to release the lock as soon as possible
            {
                let store = store.read().await;
                if !store.acl_is_allowed(&connection.get_id().to_string(), cmd.to_id()) {
                    log::trace!("Command is not allowed for user: {:?}", connection.get_user().clone().unwrap_or("Not Logged In".to_string()));
                    let rsp = Message::new_response(rsp_id, MessageResponse {
                        content: None,
                        status: OperationStatus::NotAllowed,
                        in_reply_to: Some(message.id),
                    });
                    return Some(rsp);
                }
            }
            log::trace!("Command is allowed");

            match cmd {
                Command::Get { key, default } => {
                    let store = store.read().await;
                    let rsp = match store.get(&key) {
                        None => {
                            Message::new_response(rsp_id, MessageResponse {
                                content: default.map(|x| Bson::String(x.to_string())),
                                status: OperationStatus::Failure,
                                in_reply_to: Some(message.id),
                            })
                        }
                        Some(val) => {
                            Message::new_response(rsp_id, MessageResponse {
                                content: Some(Bson::String(val.to_string())),
                                status: OperationStatus::Success,
                                in_reply_to: Some(message.id),
                            })
                        }
                    };
                    Some(rsp)
                }
                Command::Set { key, value } => {
                    let mut store = store.write().await;
                    let rsp = match store.set(key, value) {
                        Ok(_) => {
                            Message::new_response(rsp_id, MessageResponse {
                                content: None,
                                status: OperationStatus::Success,
                                in_reply_to: Some(message.id),
                            })
                        }
                        Err(err) => {
                            Message::new_response(rsp_id, MessageResponse {
                                content: Some(Bson::String(err.to_string())),
                                status: OperationStatus::Failure,
                                in_reply_to: Some(message.id),
                            })
                        }
                    };
                    Some(rsp)
                }
                Command::Heartbeat => {
                    let rsp = Message::new_response(rsp_id, MessageResponse {
                        content: None,
                        status: OperationStatus::Success,
                        in_reply_to: Some(message.id),
                    });
                    Some(rsp)
                }
                Command::Delete { key } => {
                    let mut store = store.write().await;
                    let rsp = match store.remove(&key) {
                        None => {
                            Message::new_response(rsp_id, MessageResponse {
                                content: None,
                                status: OperationStatus::NotFound,
                                in_reply_to: Some(message.id),
                            })
                        }
                        Some(_) => {
                            Message::new_response(rsp_id, MessageResponse {
                                content: None,
                                status: OperationStatus::Success,
                                in_reply_to: Some(message.id),
                            })
                        }
                    };
                    Some(rsp)
                }
                Command::ACL { op } => {
                    match op {
                        ACLOperation::Set { user, command } => {
                            let mut store = store.write().await;
                            store.acl_add(&user, command);
                            let rsp = Message::new_response(rsp_id, MessageResponse {
                                content: None,
                                status: OperationStatus::Success,
                                in_reply_to: Some(message.id),
                            });
                            Some(rsp)
                        }
                        ACLOperation::Remove { user, command } => {
                            let mut store = store.write().await;
                            store.acl_remove(&user, command);
                            let rsp = Message::new_response(rsp_id, MessageResponse {
                                content: None,
                                status: OperationStatus::Success,
                                in_reply_to: Some(message.id),
                            });
                            Some(rsp)
                        }
                        ACLOperation::List { user } => {
                            let store = store.read().await;
                            let commands = store.acl_list(&user);
                            let res = commands.iter().map(|cmd| cmd.to_string()).collect::<Vec<String>>().join(", ").to_string();
                            let rsp = Message::new_response(rsp_id, MessageResponse {
                                content: Some(Bson::String(res)),
                                status: OperationStatus::Success,
                                in_reply_to: Some(message.id),
                            });
                            Some(rsp)
                        }
                    }
                }
                Command::Login { user, password } => {
                    if !encrypted {
                        log::error!("Received unencrypted login message for user {} on connection {}", user, connection.get_id());
                        return None;
                    }
                    if connection.get_user().is_some() {
                        log::error!("User {} is already logged in on connection {}", connection.get_user().clone().unwrap(), connection.get_id());
                        return None;
                    }

                    let mut hasher = Sha512::new();
                    hasher.update(&password);

                    let result = hasher.finalize();
                    let password = format!("{:x}", result);
                    let store = store.read().await;

                    let rsp = if store.user_is_valid(&user, &password) {
                        connection.set_user(user.clone());
                        if store.user_has_key(&user) {
                            if !store.verify_key(&user, connection.get_pub_key().as_ref().unwrap()) {
                                log::error!("User {} has a public key but it's not valid. Therefor login will be denied", user);
                                return None;
                            }
                            let rsp = Message::new_response(rsp_id, MessageResponse {
                                content: None,
                                status: OperationStatus::Success,
                                in_reply_to: Some(message.id),
                            });
                            return Some(rsp);
                        } else {
                            log::warn!("User {} has no public key. Continuing anyway", user);
                        }

                        Message::new_response(rsp_id, MessageResponse {
                            content: None,
                            status: OperationStatus::Success,
                            in_reply_to: Some(message.id),
                        })
                    } else {
                        Message::new_response(rsp_id, MessageResponse {
                            content: None,
                            status: OperationStatus::Failure,
                            in_reply_to: Some(message.id),
                        })
                    };
                    Some(rsp)
                }
                Command::HDEL { key, field } => {
                    let mut store = store.write().await;
                    let rsp = match store.hremove(key, field) {
                        true => {
                            Message::new_response(rsp_id, MessageResponse {
                                content: None,
                                status: OperationStatus::Success,
                                in_reply_to: Some(message.id),
                            })
                        }
                        false => {
                            Message::new_response(rsp_id, MessageResponse {
                                content: None,
                                status: OperationStatus::NotFound,
                                in_reply_to: Some(message.id),
                            })
                        }
                    };
                    Some(rsp)
                }
                Command::HGET { key, field } => {
                    let store = store.read().await;
                    let rsp = match store.hget(key, field) {
                        None => {
                            Message::new_response(rsp_id, MessageResponse {
                                content: None,
                                status: OperationStatus::NotFound,
                                in_reply_to: Some(message.id),
                            })
                        }
                        Some(val) => {
                            Message::new_response(rsp_id, MessageResponse {
                                content: Some(Bson::String(val.clone())),
                                status: OperationStatus::Success,
                                in_reply_to: Some(message.id),
                            })
                        }
                    };
                    Some(rsp)
                }
                // Some might fail to insert. But it's not reported which failed ;)
                Command::HSET { key, value } => {
                    let mut okay = Vec::new();
                    match okay.try_reserve_exact(value.len()) {
                        Ok(_) => {}
                        Err(err) => {
                            log::error!("Error reserving space for values: {}", err);
                            let rsp = Message::new_response(rsp_id, MessageResponse {
                                content: None,
                                status: OperationStatus::Failure,
                                in_reply_to: Some(message.id),
                            });
                            return Some(rsp);
                        }
                    }
                    let mut store = store.write().await;
                    for kv in value.into_iter() {
                        let ok = store.hadd(key.clone(), kv.0, kv.1).is_ok();
                        okay.push(ok);
                    }
                    let okay = okay.iter().all(|x| *x);
                    let rsp = if okay {
                        Message::new_response(rsp_id, MessageResponse {
                            content: None,
                            status: OperationStatus::Success,
                            in_reply_to: Some(message.id),
                        })
                    } else {
                        Message::new_response(rsp_id, MessageResponse {
                            content: None,
                            status: OperationStatus::Failure,
                            in_reply_to: Some(message.id),
                        })
                    };
                    Some(rsp)
                }
                Command::HGETALL { key } => {
                    let store = store.read().await;
                    let rsp = match store.hget_all(key) {
                        Ok(map) => {
                            let map = map.into_iter().map(|(k, v)| (k, Bson::String(v))).collect::<Document>();
                            Message::new_response(rsp_id, MessageResponse {
                                content: Some(Bson::Document(map)),
                                status: OperationStatus::Success,
                                in_reply_to: Some(message.id),
                            })
                        }
                        Err(err) => {
                            Message::new_response(rsp_id, MessageResponse {
                                content: Some(Bson::String(err.to_string())),
                                status: OperationStatus::Failure,
                                in_reply_to: Some(message.id),
                            })
                        }
                    };
                    Some(rsp)
                }
                Command::HKEYS { key } => {
                    let store = store.read().await;
                    let rsp = match store.hkeys(key) {
                        Ok(keys) => {
                            let keys = keys.into_iter().map(|k| Bson::String(k)).collect::<Vec<Bson>>();
                            Message::new_response(rsp_id, MessageResponse {
                                content: Some(Bson::Array(keys)),
                                status: OperationStatus::Success,
                                in_reply_to: Some(message.id),
                            })
                        }
                        Err(err) => {
                            Message::new_response(rsp_id, MessageResponse {
                                content: Some(Bson::String(err.to_string())),
                                status: OperationStatus::Failure,
                                in_reply_to: Some(message.id),
                            })
                        }
                    };
                    Some(rsp)
                }
                Command::HLEN { key } => {
                    let store = store.read().await;
                    let rsp = Message::new_response(rsp_id, MessageResponse {
                        content: Some(Bson::Int64(store.hlen(key) as i64)),
                        status: OperationStatus::Success,
                        in_reply_to: Some(message.id),
                    });
                    Some(rsp)
                }
                Command::HVALS { key } => {
                    let store = store.read().await;
                    let rsp = match store.hget_all_values(key) {
                        Ok(values) => {
                            let values = values.into_iter().map(|v| Bson::String(v)).collect::<Vec<Bson>>();
                            Message::new_response(rsp_id, MessageResponse {
                                content: Some(Bson::Array(values)),
                                status: OperationStatus::Success,
                                in_reply_to: Some(message.id),
                            })
                        }
                        Err(err) => {
                            Message::new_response(rsp_id, MessageResponse {
                                content: Some(Bson::String(err.to_string())),
                                status: OperationStatus::Failure,
                                in_reply_to: Some(message.id),
                            })
                        }
                    };
                    Some(rsp)
                }
                Command::HEXISTS { key, field } => {
                    let store = store.read().await;
                    let rsp = Message::new_response(rsp_id, MessageResponse {
                        content: Some(Bson::Boolean(store.hcontains(key, field))),
                        status: OperationStatus::Success,
                        in_reply_to: Some(message.id),
                    });
                    Some(rsp)
                }
                Command::HINCRBY { key, field, value } => {
                    let mut store = store.write().await;
                    let rsp = match store.hincrby(key, field, value) {
                        Ok(val) => {
                            Message::new_response(rsp_id, MessageResponse {
                                content: Some(Bson::Int64(val)),
                                status: OperationStatus::Success,
                                in_reply_to: Some(message.id),
                            })
                        }
                        Err(err) => {
                            Message::new_response(rsp_id, MessageResponse {
                                content: Some(Bson::String(err.to_string())),
                                status: OperationStatus::Failure,
                                in_reply_to: Some(message.id),
                            })
                        }
                    };
                    Some(rsp)
                }
                Command::HSTRLEN { key, field } => {
                    let store = store.read().await;
                    let rsp = match store.hstr_len(key, field) {
                        Some(len) => {
                            Message::new_response(rsp_id, MessageResponse {
                                content: Some(Bson::Int64(len as i64)),
                                status: OperationStatus::Success,
                                in_reply_to: Some(message.id),
                            })
                        }
                        None => {
                            Message::new_response(rsp_id, MessageResponse {
                                content: None,
                                status: OperationStatus::NotFound,
                                in_reply_to: Some(message.id),
                            })
                        }
                    };
                    Some(rsp)
                }
                Command::KEYEXCHANGE { pub_key } => {
                    if !encrypted {
                        log::error!("Received unencrypted key exchange message");
                        return None;
                    }
                    match age::x25519::Recipient::from_str(&*pub_key) {
                        Ok(key) => {
                            connection.set_pub_key(key.clone())
                        }
                        Err(err) => {
                            log::error!("Error parsing public key: {}", err);
                            let rsp = Message::new_response(rsp_id, MessageResponse {
                                content: Some(Bson::String(err.to_string())),
                                status: OperationStatus::Failure,
                                in_reply_to: Some(message.id),
                            });
                            return Some(rsp);
                        }
                    };
                    let rsp = Message::new_response(rsp_id, MessageResponse {
                        content: None,
                        status: OperationStatus::Success,
                        in_reply_to: Some(message.id),
                    });
                    Some(rsp)
                }
            }
        }
        MessageContent::Response(_) => {
            log::error!("Received unexpected response from client: {}", connection.get_id());
            None
        }
    };
}

async fn worker_loop<T>(mut connection: Connection, store: Arc<RwLock<T>>, key: Identity) where T: StoreAble + ACLAble + UserAble + Send + Sync + HashMapAble<String> {
    loop {
        match connection.read_message(&key).await {
            Ok((message, encrypted)) => {
                log::trace!("Read from socket: {}", connection.get_id());
                let rsp_id = Uuid::new_v4();
                let resp = handle_message(message, &mut connection, &store, encrypted, rsp_id).await;
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

async fn socket_listener<T>(host: IpAddr, port: u16, brotli_effort: u8, store: Arc<RwLock<T>>, key: Identity)
    where T: StoreAble + ACLAble + UserAble + Send + Sync + HashMapAble<String> + Clone + 'static {
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
