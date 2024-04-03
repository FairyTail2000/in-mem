use std::collections::TryReserveError;
use common::acl::{ACL, CommandID};

pub trait StoreAble {
    fn get(&self, key: &str) -> Option<&String>;
    fn set(&mut self, key: String, value: String) -> Result<(), TryReserveError>;
    fn remove(&mut self, key: &str) -> Option<String>;
}

pub trait ACLAble {
    fn acl_add(&mut self, user: &str, command: CommandID);
    fn acl_remove(&mut self, user: &str, command: CommandID);
    fn acl_is_allowed(&self, user: &str, command: CommandID) -> bool;
    fn acl_list(&self, user: &str) -> Vec<CommandID>;
}

pub trait UserAble {
    fn user_add(&mut self, user: &str, password: &str);
    fn user_remove(&mut self, user: &str);
    fn user_is_valid(&self, user: &str, password: &str) -> bool;
}

#[derive(Default, Debug)]
pub struct Store {
    map: std::collections::HashMap<String, String>,
    acl: ACL,
    /// UUID -> hashed password
    users: std::collections::HashMap<String, String>,
}

impl StoreAble for Store {
    fn get(&self, key: &str) -> Option<&String> {
        self.map.get(key)
    }

    fn set(&mut self, key: String, value: String) -> Result<(), TryReserveError> {
        match self.map.try_reserve(1) {
            Ok(_) => {
                self.map.insert(key, value);
                Ok(())
            },
            Err(_) => {
                self.map.shrink_to_fit();
                match self.map.try_reserve(1) {
                    Ok(_) => {
                        self.map.insert(key, value);
                        Ok(())
                    }
                    Err(err) => {
                        return Err(err);
                    }
                }
            }
        }
    }

    fn remove(&mut self, key: &str) -> Option<String> {
        self.map.remove(key)
    }
}

impl ACLAble for Store {
    fn acl_add(&mut self, user: &str, command: CommandID) {
        self.acl.add(user, command);
    }

    fn acl_remove(&mut self, user: &str, command: CommandID) {
        self.acl.remove(user, command);
    }

    fn acl_is_allowed(&self, user: &str, command: CommandID) -> bool {
        self.acl.is_allowed(user, command)
    }
    
    fn acl_list(&self, user: &str) -> Vec<CommandID> {
        self.acl.list(user)
    }
}

impl UserAble for Store {
    fn user_add(&mut self, user: &str, password: &str) {
        self.users.insert(user.to_string(), password.to_string());
    }

    fn user_remove(&mut self, user: &str) {
        self.users.remove(user);
    }

    fn user_is_valid(&self, user: &str, password: &str) -> bool {
        match self.users.get(user) {
            Some(p) => {
                p == password
            },
            None => false
        }
    }
}