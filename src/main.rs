mod store;

use clap::Parser;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use bson::{from_slice, to_vec};
use uuid::Uuid;
use common::command::Command;
use common::init_env_logger;
use common::message::{Message, MessageContent, MessageResponse, OperationStatus};
use store::Store;
use crate::store::StoreAble;


struct Connection {
    socket: TcpStream,
    is_closed: bool,
    id: Uuid,
}

impl Connection {
    fn new(socket: TcpStream, id: Uuid) -> Self {
        Self {
            socket,
            is_closed: false,
            id
        }
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

async fn worker_loop<T>(sockets: Arc<Mutex<Vec<Connection>>>, mut store: T) where T: StoreAble + Send + Sync {
    loop {
        tokio::time::sleep(std::time::Duration::from_nanos(20)).await;
        let mut blocked = sockets.lock().await;
        for connection in blocked.iter_mut() {
            let mut buf = [0; 1024];
            let socket = &mut connection.socket;
            let rsp_id = Uuid::new_v4();
            match socket.read(&mut buf).await {
                Ok(0) => {
                    log::info!("Connection closed");
                    connection.is_closed = true;
                }
                Ok(_) => {
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
                            socket.write_all(&*rsp).await.unwrap();
                            continue;
                        }
                    };
                    log::debug!("Received message: {:?}", message);
                    match message.content {
                        MessageContent::Command(cmd) => {
                            match cmd {
                                Command::Get { key, default } => {
                                    match store.get(&key) {
                                        None => {
                                            let rsp = Message::new_response(rsp_id, MessageResponse {
                                                content: default,
                                                status: OperationStatus::Failure,
                                                in_reply_to: Some(message.id),
                                            });
                                            let rsp = to_vec(&rsp).unwrap();
                                            socket.write_all(&*rsp).await.unwrap();
                                        }
                                        Some(val) => {
                                            let rsp = Message::new_response(rsp_id, MessageResponse {
                                                content: Some(val.clone()),
                                                status: OperationStatus::Success,
                                                in_reply_to: Some(message.id),
                                            });
                                            let rsp = to_vec(&rsp).unwrap();
                                            socket.write_all(&*rsp).await.unwrap();
                                        }
                                    }
                                }
                                Command::Set { key, value } => {
                                    store.set(key, value);
                                    let rsp = Message::new_response(rsp_id, MessageResponse {
                                        content: None,
                                        status: OperationStatus::Success,
                                        in_reply_to: Some(message.id),
                                    });
                                    let rsp = to_vec(&rsp).unwrap();
                                    socket.write_all(&*rsp).await.unwrap();
                                }
                                Command::Heartbeat => {
                                    let rsp = Message::new_response(rsp_id, MessageResponse {
                                        content: None,
                                        status: OperationStatus::Success,
                                        in_reply_to: Some(message.id),
                                    });
                                    let rsp = to_vec(&rsp).unwrap();
                                    socket.write_all(&*rsp).await.unwrap();
                                }
                                Command::Delete { key } => {
                                    match store.remove(&key) {
                                        None => {
                                            let rsp = Message::new_response(rsp_id, MessageResponse {
                                                content: Some("Key not found".to_string()),
                                                status: OperationStatus::Failure,
                                                in_reply_to: Some(message.id),
                                            });
                                            let rsp = to_vec(&rsp).unwrap();
                                            socket.write_all(&*rsp).await.unwrap();
                                        }
                                        Some(_) => {
                                            let rsp = Message::new_response(rsp_id, MessageResponse {
                                                content: None,
                                                status: OperationStatus::Success,
                                                in_reply_to: Some(message.id),
                                            });
                                            let rsp = to_vec(&rsp).unwrap();
                                            socket.write_all(&*rsp).await.unwrap();
                                        }
                                    };
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
