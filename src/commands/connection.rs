use std::str::FromStr;
use std::sync::Arc;
use age::x25519::Recipient;
use async_trait::async_trait;
use bson::Bson;
use sha2::{Digest, Sha512};
use tokio::sync::RwLock;
use common::command_input::{KeyExchangeCommandInput, LoginCommandInput};
use common::connection::Connection;
use common::message::{Message, MessageResponse, OperationStatus};
use crate::commands::Command;
use crate::store::{Store, UserAble};

#[derive(Default)]
pub struct LoginCommand {
    encrypted: bool,
    already_logged_in: bool,
    recipient: Option<Recipient>,
    /// When the login succeeds the user is stored here to be used in the post_exec to update the connection
    login: Option<String>,
}

#[async_trait]
impl Command for LoginCommand {
    async fn pre_exec(&mut self, connection: &Connection, encrypted: bool) -> bool {
        self.encrypted = encrypted;
        self.already_logged_in = connection.get_user().is_some();
        self.recipient = connection.get_pub_key();
        true
    }

    async fn execute(&mut self, store: Arc<RwLock<Store>>, args: Bson, message: &Message) -> Option<MessageResponse> {
        let store = store.read().await;
        let args: LoginCommandInput = match args.as_document() {
            None => {
                return None;
            }
            Some(doc) => {
                match bson::from_bson(Bson::Document(doc.clone())) {
                    Ok(val) => val,
                    Err(_) => {
                        return None;
                    }
                }
            }
        };

        if !self.encrypted {
            log::error!("Received unencrypted login message for user {}", args.user);
            return None;
        }
        if self.already_logged_in {
            log::error!("User {} is already logged in", args.user);
            return None;
        }

        let mut hasher = Sha512::new();
        hasher.update(&args.password);

        let result = hasher.finalize();
        let password = format!("{:x}", result);

        let rsp = if store.user_is_valid(&args.user, &password) {
            if store.user_has_key(&args.user) {
                let rcp = self.recipient.as_ref().unwrap();
                if !store.verify_key(&args.user, rcp) {
                    log::error!("User {} has a public key but it's not valid. Therefor login will be denied", args.user);
                    return None;
                }

                return Some(MessageResponse {
                    content: None,
                    status: OperationStatus::Success,
                    in_reply_to: Some(message.id),
                });
            } else {
                log::warn!("User {} has no public key. Continuing anyway", args.user);
            }

            MessageResponse {
                content: None,
                status: OperationStatus::Success,
                in_reply_to: Some(message.id),
            }
        } else {
            MessageResponse {
                content: None,
                status: OperationStatus::Failure,
                in_reply_to: Some(message.id),
            }
        };
        Some(rsp)
    }

    async fn post_exec(&mut self, connection: &mut Connection, _: Option<&MessageResponse>) {
        self.encrypted = false;
        self.already_logged_in = false;
        self.recipient = None;
        self.login.as_ref().map(|user| {
            connection.set_user(user.clone());
        });
        self.login = None;
    }
}

#[derive(Default)]
pub struct KeyExchangeCommand {
    encrypted: bool,
    recipient: Option<Recipient>,
}

#[async_trait]
impl Command for KeyExchangeCommand {
    async fn pre_exec(&mut self, _: &Connection, encrypted: bool) -> bool {
        self.encrypted = encrypted;
        true
    }

    async fn execute(&mut self, _: Arc<RwLock<Store>>, args: Bson, message: &Message) -> Option<MessageResponse> {
        let args: KeyExchangeCommandInput = match args.as_document() {
            None => {
                return None;
            }
            Some(doc) => {
                match bson::from_bson(Bson::Document(doc.clone())) {
                    Ok(val) => val,
                    Err(_) => {
                        return None;
                    }
                }
            }
        };

        if !self.encrypted {
            log::error!("Received unencrypted key exchange message");
            return None;
        }
        match age::x25519::Recipient::from_str(&*args.pub_key) {
            Ok(key) => {
                self.recipient = Some(key);
            }
            Err(err) => {
                log::error!("Error parsing public key: {}", err);
                let rsp = MessageResponse {
                    content: Some(Bson::String(err.to_string())),
                    status: OperationStatus::Failure,
                    in_reply_to: Some(message.id),
                };
                return Some(rsp);
            }
        };
        let rsp = MessageResponse {
            content: None,
            status: OperationStatus::Success,
            in_reply_to: Some(message.id),
        };
        Some(rsp)
    }

    async fn post_exec(&mut self, connection: &mut Connection, _: Option<&MessageResponse>) {
        self.encrypted = false;
        self.recipient.as_ref().map(|pub_key| {
            connection.set_pub_key(pub_key.clone());
        });
        self.recipient = None;
    }
}