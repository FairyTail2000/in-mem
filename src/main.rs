mod store;

use clap::Parser;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use bson::{from_slice, to_vec};
use uuid::Uuid;
use common::command::{ACLOperation, Command};
use common::init_env_logger;
use common::message::{Message, MessageContent, MessageResponse, OperationStatus};
use store::Store;
use crate::store::{ACLAble, StoreAble, UserAble};
use brotli2::read::BrotliDecoder;
use brotli2::write::BrotliEncoder;
use brotli2::CompressParams;
use std::io::prelude::{Read, Write};
use sha2::{Digest, Sha512};


struct Connection {
    socket: TcpStream,
    is_closed: bool,
    id: Uuid,
    user: Option<String>,
}

impl Connection {
    fn new(socket: TcpStream, id: Uuid) -> Self {
        Self {
            socket,
            is_closed: false,
            id,
            user: None
        }
    }

    /// Abstracts the read operation on the socket, returning the read buffer to abstract the operation to add compression later
    async fn read(&mut self) -> std::io::Result<Vec<u8>> {
        let mut buf = vec![0; 1024];
        return match self.socket.read(&mut buf).await {
            Ok(read) => {
                if read == 0 {
                    return Err(std::io::Error::from(std::io::ErrorKind::ConnectionAborted));
                }
                buf.truncate(read);
                let mut d = BrotliDecoder::new(&buf[..]);
                let mut decompressed_buf = Vec::new();
                d.read_to_end(&mut decompressed_buf)?;
                Ok(decompressed_buf)
            }
            Err(err) => {
                Err(err)
            }
        };
    }

    /// Abstracts the write operation on the socket, returning the read buffer to abstract the operation to add compression later
    async fn write(&mut self, buf: &[u8]) -> std::io::Result<()> {
        let mut params = CompressParams::new();
        params.quality(6);
        let mut e = BrotliEncoder::from_params(Vec::new(), &params);
        e.write_all(buf)?;
        let compressed_buf = e.finish()?;
        return self.socket.write_all(&compressed_buf).await;
    }
}

#[derive(Parser, Debug)]
#[command(name = "in-mem-server", version = "1.0", about = "")]
struct CLI {
    /// The host to bind to
    #[arg(default_value = "127.0.0.1", env = "HOST", help = "The host to bind to")]
    host: IpAddr,
    /// The port to bind to
    #[arg(default_value = "3000", env = "PORT", help = "The port to bind to")]
    port: u16,
}

async fn worker_loop<T>(sockets: Arc<Mutex<Vec<Connection>>>, mut store: T) where T: StoreAble + ACLAble + UserAble + Send + Sync {
    loop {
        tokio::time::sleep(std::time::Duration::from_nanos(20)).await;
        let mut blocked = sockets.lock().await;
        for connection in blocked.iter_mut() {
            let rsp_id = Uuid::new_v4();
            match connection.read().await {
                Ok(buf) => {
                    log::trace!("Read from socket: {}", connection.id);
                    let message: Message = match from_slice(&buf) {
                        Ok(msg) => msg,
                        Err(err) => {
                            log::error!("Error parsing Message: {}", err);
                            connection.is_closed = true;
                            let rsp = Message::new_response(rsp_id, MessageResponse {
                                content: Some(err.to_string()),
                                status: OperationStatus::Failure,
                                in_reply_to: None,
                            });
                            let rsp = to_vec(&rsp).unwrap();
                            connection.write(&*rsp).await.unwrap();
                            continue;
                        }
                    };
                    match message.content {
                        MessageContent::Command(cmd) => {
                            log::trace!("Received command: {:?}", cmd);
                            if !store.acl_is_allowed(&connection.id.to_string(), cmd.to_id()) {
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
                            log::trace!("Command is allowed");
                            
                            match cmd {
                                Command::Get { key, default } => {
                                    let rsp = match store.get(&key) {
                                        None => {
                                            Message::new_response(rsp_id, MessageResponse {
                                                content: default,
                                                status: OperationStatus::Failure,
                                                in_reply_to: Some(message.id),
                                            }).to_vec().unwrap()
                                        }
                                        Some(val) => {
                                            Message::new_response(rsp_id, MessageResponse {
                                                content: Some(val.clone()),
                                                status: OperationStatus::Success,
                                                in_reply_to: Some(message.id),
                                            }).to_vec().unwrap()
                                        }
                                    };
                                    connection.write(&*rsp).await.unwrap();
                                }
                                Command::Set { key, value } => {
                                    let rsp = match store.set(key, value) {
                                        Ok(_) => {
                                            Message::new_response(rsp_id, MessageResponse {
                                                content: None,
                                                status: OperationStatus::Success,
                                                in_reply_to: Some(message.id),
                                            })
                                        },
                                        Err(err) => {
                                            Message::new_response(rsp_id, MessageResponse {
                                                content: Some(err.to_string()),
                                                status: OperationStatus::Failure,
                                                in_reply_to: Some(message.id),
                                            })
                                        }
                                    }.to_vec().unwrap();
                                    connection.write(&*rsp).await.unwrap();
                                }
                                Command::Heartbeat => {
                                    let rsp = Message::new_response(rsp_id, MessageResponse {
                                        content: None,
                                        status: OperationStatus::Success,
                                        in_reply_to: Some(message.id),
                                    }).to_vec().unwrap();
                                    connection.write(&*rsp).await.unwrap();
                                }
                                Command::Delete { key } => {
                                    let rsp = match store.remove(&key) {
                                        None => {
                                            Message::new_response(rsp_id, MessageResponse {
                                                content: None,
                                                status: OperationStatus::NotFound,
                                                in_reply_to: Some(message.id),
                                            }).to_vec().unwrap()
                                        }
                                        Some(_) => {
                                            Message::new_response(rsp_id, MessageResponse {
                                                content: None,
                                                status: OperationStatus::Success,
                                                in_reply_to: Some(message.id),
                                            }).to_vec().unwrap()
                                        }
                                    };
                                    connection.write(&*rsp).await.unwrap();
                                }
                                Command::ACL { op } => {
                                    match op {
                                        ACLOperation::Set { user, command } => {
                                            store.acl_add(&user, command);
                                        }
                                        ACLOperation::Remove { user, command } => {
                                            store.acl_remove(&user, command);
                                        }
                                        ACLOperation::List { user } => {
                                            let commands = store.acl_list(&user);
                                            
                                            let rsp = Message::new_response(rsp_id, MessageResponse {
                                                content: Some(commands.iter().map(|cmd| cmd.to_string()).collect::<Vec<String>>().join(", ").to_string()),
                                                status: OperationStatus::Success,
                                                in_reply_to: Some(message.id),
                                            }).to_vec().unwrap();
                                            connection.write(&*rsp).await.unwrap();
                                        }
                                    }
                                },
                                Command::Login { user, password } => {
                                    let mut hasher = Sha512::new();
                                    hasher.update(&password);
                                    let result = hasher.finalize();
                                    let password = format!("{:x}", result);
                                    let rsp = if store.user_is_valid(&user, &password) {
                                        connection.user = Some(user.clone());
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
                                    }.to_vec().unwrap();
                                    connection.write(&*rsp).await.unwrap();
                                }
                            }
                        }
                        MessageContent::Response(_) => {
                            log::error!("Received unexpected response from client: {}", connection.id);
                            connection.is_closed = true;
                        }
                    }
                }
                Err(err) => {
                    log::error!("Error reading from socket: {}", err);
                    connection.is_closed = true;
                }
            }
        }
        blocked.retain(|connection| !connection.is_closed);
    }
}

async fn socket_listener(sockets: Arc<Mutex<Vec<Connection>>>) {
    let args = CLI::parse();

    let addr = SocketAddr::from((args.host, args.port));
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
        sockets.lock().await.push(Connection::new(socket, Uuid::new_v4()));
    }
}

#[tokio::main]
async fn main() {
    init_env_logger();
    let sockets: Arc<Mutex<Vec<Connection>>> = Arc::new(Mutex::new(Vec::new()));
    let cloned_sockets = sockets.clone();
    std::thread::spawn(move || {
        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(err) => {
                log::error!("Error creating runtime: {}", err);
                std::process::exit(-1);
            }
        };
        let sockets = cloned_sockets.clone();
        rt.block_on(async move {
            let store = Store::default();
            worker_loop(sockets, store).await;
        });
    });
    socket_listener(sockets).await;
}
