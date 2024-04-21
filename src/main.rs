use std::io::prelude::Read;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;

use age::secrecy::ExposeSecret;
use age::x25519::Identity;
use bson::{Bson, Document};
use clap::Parser;
use sha2::{Digest, Sha512};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use uuid::Uuid;

use common::command::{ACLOperation, Command};
use common::connection::Connection;
use common::init_env_logger;
use common::message::{Message, MessageContent, MessageResponse, OperationStatus};

use crate::store::{ACLAble, HashMapAble, Store, StoreAble, UserAble};

mod store;

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
    #[arg(default_value = "3000", env = "PORT", help = "The port to bind to")]
    port: u16,
}

async fn handle_message<T>(message: Message, connection: &mut Connection, store: &Arc<Mutex<T>>, encrypted: bool, rsp_id: Uuid) -> Option<Message> where T: StoreAble + ACLAble + UserAble + Send + Sync + HashMapAble<String> {
    return match message.content {
        MessageContent::Command(cmd) => {
            log::trace!("Received command: {:?}", cmd);
            /*if !store.acl_is_allowed(&connection.id.to_string(), cmd.to_id()) {
                log::trace!("Command is not allowed for user: {:?}", connection.user.clone().unwrap_or("Not Logged In".to_string()));
                let rsp = Message::new_response(rsp_id, MessageResponse {
                    content: None,
                    status: OperationStatus::NotAllowed,
                    in_reply_to: Some(message.id),
                });
                match to_vec(&rsp) {
                    Ok(rsp) => {
                        match connection.write(&*rsp).await {
                            Ok(_) => {}
                            Err(err) => {
                                log::error!("Error writing response: {}", err);
                                connection.is_closed = true;
                            }
                        }
                    },
                    Err(err) => {
                        log::error!("Error serializing response: {}", err);
                        connection.is_closed = true;
                    }
                }
                continue;
            }
            log::trace!("Command is allowed");*/

            match cmd {
                Command::Get { key, default } => {
                    let store = store.lock().await;
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
                    let mut store = store.lock().await;
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
                    let mut store = store.lock().await;
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
                            let mut store = store.lock().await;
                            store.acl_add(&user, command);
                            let rsp = Message::new_response(rsp_id, MessageResponse {
                                content: None,
                                status: OperationStatus::Success,
                                in_reply_to: Some(message.id),
                            });
                            Some(rsp)
                        }
                        ACLOperation::Remove { user, command } => {
                            let mut store = store.lock().await;
                            store.acl_remove(&user, command);
                            let rsp = Message::new_response(rsp_id, MessageResponse {
                                content: None,
                                status: OperationStatus::Success,
                                in_reply_to: Some(message.id),
                            });
                            Some(rsp)
                        }
                        ACLOperation::List { user } => {
                            let store = store.lock().await;
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
                    let mut hasher = Sha512::new();
                    hasher.update(&password);
                    let result = hasher.finalize();
                    let password = format!("{:x}", result);
                    let store = store.lock().await;
                    let rsp = if store.user_is_valid(&user, &password) {
                        connection.set_user(user.clone());
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
                    let mut store = store.lock().await;
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
                    let store = store.lock().await;
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
                    let mut store = store.lock().await;
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
                    let store = store.lock().await;
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
                    let store = store.lock().await;
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
                    let store = store.lock().await;
                    let rsp = Message::new_response(rsp_id, MessageResponse {
                        content: Some(Bson::Int64(store.hlen(key) as i64)),
                        status: OperationStatus::Success,
                        in_reply_to: Some(message.id),
                    });
                    Some(rsp)
                }
                Command::HVALS { key } => {
                    let store = store.lock().await;
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
                    let store = store.lock().await;
                    let rsp = Message::new_response(rsp_id, MessageResponse {
                        content: Some(Bson::Boolean(store.hcontains(key, field))),
                        status: OperationStatus::Success,
                        in_reply_to: Some(message.id),
                    });
                    Some(rsp)
                }
                Command::HINCRBY { key, field, value } => {
                    let mut store = store.lock().await;
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
                    let store = store.lock().await;
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

async fn worker_loop<T>(mut connection: Connection, store: Arc<Mutex<T>>, key: Identity) where T: StoreAble + ACLAble + UserAble + Send + Sync + HashMapAble<String> {
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

async fn socket_listener<T>(host: IpAddr, port: u16, brotli_effort: u8, store: Arc<Mutex<T>>, key: Identity)
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

#[tokio::main]
async fn main() {
    init_env_logger();

    let cli = Cli::parse();

    let private_key = match std::fs::File::open("server-identity.age") {
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
            match std::fs::write("server-identity.age", key.to_string().expose_secret()) {
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
    let store = Arc::new(Mutex::new(Store::default()));
    socket_listener(cli.host, cli.port, cli.brotli_effort, store, private_key).await;
}
