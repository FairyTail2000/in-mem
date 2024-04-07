use std::net::{IpAddr, SocketAddr};
use age::secrecy::ExposeSecret;
use age::x25519::{Identity, Recipient};
use clap::Parser;
use bson::{from_slice, to_vec};
use std::io::{Read, Write};
use std::str::FromStr;
use age::Decryptor;
use brotli2::CompressParams;
use brotli2::read::BrotliDecoder;
use brotli2::write::BrotliEncoder;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use uuid::Uuid;
use common::{command, init_env_logger};
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

fn decrypt(buf: &[u8], key: &Identity) -> std::io::Result<Vec<u8>> {
    let dec = match Decryptor::new(&buf[..]) {
        Ok(dec) => {
            match dec {
                Decryptor::Recipients(d) => {d},
                Decryptor::Passphrase(_) => {
                    return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Received passphrase encrypted message, expected public key encrypted message"));
                }
            }
        },
        Err(err) => {
            let formatted = format!("Error creating decryptor: {}", err);
            log::error!("{}", formatted);
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, formatted));
        }
    };
    let mut reader = match dec.decrypt(vec![key as &dyn age::Identity].into_iter()) {
        Ok(reader) => reader,
        Err(err) => {
            let formatted = format!("Error decrypting message: {}", err);
            log::error!("{}", formatted);
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, formatted));
        }
    };
    let mut decrypted = Vec::new();
    match reader.read_to_end(&mut decrypted) {
        Ok(_) => {}
        Err(err) => {
            let formatted = format!("Error reading decrypted message: {}", err);
            log::error!("{}", formatted);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, formatted));
        }
    }
    return Ok(decrypted);
}

fn encrypt(buf: &[u8], key: &Recipient) -> std::io::Result<Vec<u8>> {
    let mut encrypted = Vec::new();
    let e = age::Encryptor::with_recipients(vec![Box::new(key.clone())]).unwrap();
    let mut writer = e.wrap_output(&mut encrypted).unwrap();
    writer.write_all(buf)?;
    writer.finish()?;
    Ok(encrypted)
}

fn compress(buf: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut params = CompressParams::new();
    params.quality(6);
    let mut e = BrotliEncoder::from_params(Vec::new(), &params);
    e.write_all(buf)?;
    let compressed_buf = e.finish()?;
    return Ok(compressed_buf);
}

fn decompress(buf: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut d = BrotliDecoder::new(&buf[..]);
    let mut decompressed_buf = Vec::new();
    d.read_to_end(&mut decompressed_buf)?;
    return Ok(decompressed_buf);
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
            match std::fs::write("identity2.age", key.to_string().expose_secret()) {
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
    let server_public_key = Recipient::from_str( "age1jzh3n3x83jm3e77zlwkxh28uekg4emjfu962uwkzp04remlnngls52d6ve").unwrap();
    log::info!("Public key: \"{}\"", public_key);
    
    
    
    let mut buf = [0; 1024];
    let heartbeat_message = Message::new_command(Uuid::new_v4(), command::Command::Heartbeat).to_vec().unwrap();
    let heartbeat_message = compress(&heartbeat_message).unwrap();
    let kex_msg = Message::new_command(Uuid::new_v4(), command::Command::KEYEXCHANGE {pub_key: public_key.clone().to_string() }).to_vec().unwrap();
    log::trace!("Unencrypted len: {}", kex_msg.len());
    let kex_msg = encrypt(&kex_msg, &server_public_key).unwrap();
    log::trace!("Encrypted len: {}", kex_msg.len());
    let kex_msg = compress(&kex_msg).unwrap();
    log::trace!("Compressed len: {}", kex_msg.len());
    log::trace!("Sending key exchange message with size: {}", kex_msg.len());
    match socket.write_all(&*kex_msg).await {
        Ok(_) => {
            let n = socket.read(&mut buf).await.unwrap();
            if n == 0 {
                log::error!("Connection closed by server before sending key!");
                std::process::exit(-1);
            }
        }
        Err(_) => {
            log::error!("Connection shut down");
        }
    }
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
        let cmd = Message::new_command(Uuid::new_v4(), cmd);
        log::debug!("Sending command: {}", cmd);
        let cmd = to_vec(&cmd).unwrap();
        let cmd = encrypt(&cmd, &server_public_key).unwrap();
        let cmd = compress(&cmd).unwrap();
        socket.write_all(&*cmd).await.unwrap();
        socket.read(&mut buf).await.unwrap();
        let buf = decompress(&buf).unwrap();
        let buf = decrypt(&buf, &private_key).unwrap();
        let message: Message = match from_slice(&buf) {
            Ok(msg) => msg,
            Err(err) => {
                log::error!("Error parsing Message: {}", err);
                continue;
            }
        };
        log::info!("Response: {}", message);
    }
}
