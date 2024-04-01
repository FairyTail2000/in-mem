use std::net::{IpAddr, SocketAddr};
use clap::Parser;
use bson::to_vec;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use uuid::Uuid;
use common::{command, message, init_env_logger};


#[derive(Parser, Debug)]
#[command(name = "in-mem-client", version = "1.0", about = "A demo client to connect to the in-mem-server")]
struct CLI {
    /// The host to bind to
    #[arg(default_value = "127.0.0.1", env = "HOST", help = "The host to connect to")]
    host: IpAddr,
    /// The port to bind to
    #[arg(default_value = "3000", env = "PORT", help = "The port to connect to")]
    port: u16,
}


#[tokio::main]
async fn main() {
    init_env_logger();

    let args = CLI::parse();
    log::trace!("Connecting to {}:{}", args.host, args.port);
    let mut socket = match TcpStream::connect(SocketAddr::new(args.host, args.port)).await {
        Ok(socket) => socket,
        Err(err) => {
            log::error!("Error connecting to {}:{}: {}", args.host, args.port, err);
            return;
        }
    };
    log::info!("Connected to {}:{}", args.host, args.port);
    let mut buf = [0; 1024];
    let heartbeat_message = message::Message::new_command(Uuid::new_v4(), command::Command::Heartbeat);
    let heartbeat_message = to_vec(&heartbeat_message).unwrap();
    loop {
        match socket.write_all(&*heartbeat_message).await {
            Ok(_) => {
                let n = socket.read(&mut buf).await.unwrap();
                if n == 0 {
                    log::error!("Connection closed by server");
                    break;
                }
            }
            Err(_) => {
                log::error!("Connection shut down");
            }
        }


        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        if input.is_empty() {
            continue;
        }
        let cmd = match command::Command::try_from(input) {
            Ok(cmd) => cmd,
            Err(err) => {
                log::error!("Error: {:?}", err);
                continue;
            }
        };
        let cmd = message::Message::new_command(Uuid::new_v4(), cmd);
        let cmd = to_vec(&cmd).unwrap();
        socket.write_all(&*cmd).await.unwrap();
        let n = socket.read(&mut buf).await.unwrap();
        let response = std::str::from_utf8(&buf[..n]).unwrap();
        log::info!("Response: {}", response);
    }
}
