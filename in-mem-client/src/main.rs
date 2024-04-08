use std::net::{IpAddr, SocketAddr};
use age::secrecy::ExposeSecret;
use age::x25519::Identity;
use clap::Parser;
use std::io::Read;
use tokio::net::TcpStream;
use uuid::Uuid;
use common::{command, init_env_logger};
use common::connection::Connection;
use common::message::Message;


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
    let socket = match TcpStream::connect(SocketAddr::new(args.host, args.port)).await {
        Ok(socket) => socket,
        Err(err) => {
            log::error!("Error connecting to {}:{}: {}", args.host, args.port, err);
            return;
        }
    };
    let mut connection = Connection::new(socket, Uuid::new_v4(), 6);
    log::info!("Connected to {}:{}", args.host, args.port);
    let private_key = match std::fs::File::open("identity-client.age") {
        Ok(mut file) => {
            let mut buf = Vec::new();
            match file.read_to_end(&mut buf) {
                Ok(_) => {}
                Err(err) => {
                    log::error!("Error reading identity file: {}", err);
                    std::process::exit(-1);
                }
            }
            Identity::from(std::string::String::from(std::str::from_utf8(&buf).unwrap()).parse().unwrap())
        }
        Err(_) => {
            log::warn!("No identity file found or not readable. Generating new identity file");
            let key = Identity::generate();
            match std::fs::write("identity-client.age", key.to_string().expose_secret()) {
                Ok(_) => {}
                Err(err) => {
                    log::error!("Error writing identity file: {}", err);
                    std::process::exit(-1);
                }
            }
            key
        }
    };
    let server_public_key = match std::fs::File::open("identity-server.age") {
        Ok(mut file) => {
            let mut buf = Vec::new();
            match file.read_to_end(&mut buf) {
                Ok(_) => {}
                Err(err) => {
                    log::error!("Error reading identity file: {}", err);
                    std::process::exit(-1);
                }
            }
            Identity::from(std::string::String::from(std::str::from_utf8(&buf).unwrap()).parse().unwrap()).to_public()
        }
        Err(_) => {
            log::error!("No identity file found or not readable. Generating new identity file");
            std::process::exit(-1);
        }
    };
    let public_key = private_key.to_public();
    log::info!("Public key: \"{}\"", public_key);
    connection.set_pub_key(server_public_key);

    
    
    let heartbeat_message = Message::new_command(Uuid::new_v4(), command::Command::Heartbeat);
    let kex_msg = Message::new_command(Uuid::new_v4(), command::Command::KEYEXCHANGE {pub_key: public_key.clone().to_string() });
    log::debug!("Sending key exchange message");
    match connection.send_message(&kex_msg).await {
        Ok(_) => {}
        Err(err) => {
            log::error!("Error sending heartbeat message size: {}", err);
            std::process::exit(-1);
        }
    }
    log::debug!("Sending first heartbeat message");
    match connection.send_message(&heartbeat_message).await {
        Ok(_) => {
            if connection.read_message(&private_key).await.is_err() {
                log::error!("Error reading heartbeat response");
                std::process::exit(-1);
            }
        }
        Err(_) => {
            log::error!("Connection shut down");
        }
    }
    loop {
        log::debug!("Sending heartbeat message");
        match connection.send_message(&heartbeat_message).await {
            Ok(_) => {
                if connection.read_message(&private_key).await.is_err() {
                    log::error!("Error reading heartbeat response");
                    std::process::exit(-1);
                }
            }
            Err(_) => {
                log::error!("Connection shut down");
            }
        }

        log::trace!("Waiting for input");
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
        let cmd = Message::new_command(Uuid::new_v4(), cmd);
        match connection.send_message(&cmd).await {
            Ok(_) => {}
            Err(err) => {
                log::error!("Error sending message: {}", err);
                continue;
            }
        }
        let message: Message = match connection.read_message(&private_key).await {
            Ok((msg, _)) => msg,
            Err(err) => {
                log::error!("Error parsing Message: {}", err);
                continue;
            }
        };
        log::info!("Response: {}", message);
    }
}
