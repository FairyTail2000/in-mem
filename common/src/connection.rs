use uuid::Uuid;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use age::x25519::{Identity, Recipient};
use age::Decryptor;
use brotli2::CompressParams;
use brotli2::read::BrotliDecoder;
use brotli2::write::BrotliEncoder;
use std::io::prelude::{Read, Write};
use crate::message::Message;

pub struct Connection {
    socket: TcpStream,
    is_closed: bool,
    id: Uuid,
    user: Option<String>,
    pub_key: Option<Recipient>,
    brotli_effort: u8
}

impl Connection {
    pub fn new(socket: TcpStream, id: Uuid, brotli_effort: u8) -> Self {
        Self {
            socket,
            is_closed: false,
            id,
            user: None,
            pub_key: None,
            brotli_effort
        }
    }

    /// Decrypts the buffer with the private key of the server, if the first bytes are age-encrypt
    fn decrypt(&self, buf: &[u8], key: &Identity) -> std::io::Result<Option<Vec<u8>>> {
        let encrypted = match std::str::from_utf8(&buf[..11]) {
            Ok(header) => {
                header == "age-encrypt"
            },
            Err(_) => false
        };
        if encrypted {
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
            return Ok(Some(decrypted));
        }
        return Ok(None);
    }

    fn encrypt(&self, buf: &[u8]) -> std::io::Result<Vec<u8>> {
        return match self.pub_key.as_ref() {
            Some(key) => {
                let mut encrypted = Vec::new();
                let e = age::Encryptor::with_recipients(vec![Box::new(key.clone())]).unwrap();
                let mut writer = e.wrap_output(&mut encrypted).unwrap();
                writer.write_all(buf)?;
                writer.finish()?;
                Ok(encrypted)
            }
            None => {
                Ok(buf.to_vec())
            }
        };
    }

    fn compress(&self, buf: &[u8]) -> std::io::Result<Vec<u8>> {
        let mut params = CompressParams::new();
        params.quality(self.brotli_effort as u32);
        let mut e = BrotliEncoder::from_params(Vec::new(), &params);
        e.write_all(buf)?;
        let compressed_buf = e.finish()?;
        return Ok(compressed_buf);
    }

    fn decompress(&self, buf: &[u8]) -> std::io::Result<Vec<u8>> {
        let mut d = BrotliDecoder::new(&buf[..]);
        let mut decompressed_buf = Vec::new();
        d.read_to_end(&mut decompressed_buf)?;
        return Ok(decompressed_buf);
    }

    /// Read -> decompress -> decrypt
    pub async fn read(&mut self, key: &Identity) -> std::io::Result<(Vec<u8>, bool)> {
        let size = self.socket.read_u32().await?;
        let mut buf = vec![0; size as usize];
        return match self.socket.read(&mut buf).await {
            Ok(read) => {
                if read == 0 {
                    return Err(std::io::Error::from(std::io::ErrorKind::ConnectionAborted));
                }
                buf.truncate(read);
                log::trace!("Read {} bytes from socket, decompressing", read);
                let decompressed_buf = self.decompress(&buf)?;
                log::trace!("Decompressed {} bytes, decrypting", decompressed_buf.len());
                match self.decrypt(&decompressed_buf, key)? {
                    Some(decrypted) => {
                        log::trace!("Decrypted {} bytes", decrypted.len());
                        Ok((decrypted, true))
                    },
                    None => {
                        log::trace!("No public key present, returning decompressed buffer");
                        Ok((decompressed_buf, false))
                    }
                }
            }
            Err(err) => {
                Err(err)
            }
        };
    }

    /// Encrypt -> compress -> write
    pub async fn write(&mut self, buf: &[u8]) -> std::io::Result<()> {
        let maybe_encrypted = self.encrypt(&buf)?;
        let compressed_buf = self.compress(&maybe_encrypted)?;
        let len_bytes = (compressed_buf.len() as u32).to_be_bytes();
        self.socket.write_all(&len_bytes).await?;
        // Maybe encrypt because we might not have a public key. And thus need to send unencrypted
        return self.socket.write_all(&compressed_buf).await;
    }
    
    pub async fn send_message(&mut self, msg: &Message) -> std::io::Result<()> {
        let msg = msg.to_vec().unwrap();
        let msg = self.encrypt(&msg).unwrap();
        let msg = self.compress(&msg).unwrap();
        let msg_size_bytes = (msg.len() as u32).to_be_bytes();
        log::trace!("Sending message of size {}bytes", msg.len());
        self.socket.write_all(&msg_size_bytes).await?;
        self.socket.write_all(&*msg).await
    }
    
    // Boolean flag indicates that the message was encrypted
    pub async fn read_message(&mut self, key: &Identity) -> std::io::Result<(Message, bool)> {
        let mut len_bytes = [0u8; 4];
        self.socket.read_exact(&mut len_bytes).await?;
        let msg_size = u32::from_be_bytes(len_bytes); // Convert from big endian
        
        log::trace!("Reading message of size {}bytes", msg_size);
        let mut buf = vec![0; msg_size as usize];
        self.socket.read_exact(&mut buf).await?;
        let buf = self.decompress(&buf)?;
        let before = buf.len();
        let buf = self.decrypt(&buf, key)?.unwrap();
        let after = buf.len();
        return Ok((Message::from_slice(&buf).unwrap(), before != after));
    }
    
    /// Important. Does not actually close the connection, just sets a flag closed flag
    pub fn close(&mut self) {
        self.is_closed = true;
    }
    
    pub fn is_closed(&self) -> bool {
        self.is_closed
    }
    
    pub fn get_id(&self) -> Uuid {
        self.id
    }
    
    pub fn get_user(&self) -> Option<String> {
        self.user.clone()
    }
    
    pub fn set_user(&mut self, user: String) {
        self.user = Some(user);
    }
    
    pub fn set_pub_key(&mut self, key: Recipient) {
        self.pub_key = Some(key);
    }
    
    pub fn get_pub_key(&self) -> Option<Recipient> {
        self.pub_key.clone()
    }
}
