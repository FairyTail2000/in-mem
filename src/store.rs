use std::collections::{HashMap, TryReserveError};
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

// Now I understand why redis used h in front of all the hashmap commands. It's to avoid name conflicts.
pub trait HashMapAble<T> {
    fn hadd(&mut self, map_key: String, key: String, value: T) -> Result<(), TryReserveError>;
    fn hremove(&mut self, map_key: String, key: String) -> bool;
    fn hcontains(&self, map_key: String, key: String) -> bool;
    fn hget(&self, map_key: String, key: String) -> Option<&T>;
    fn hget_all(&self, map_key: String) -> Result<HashMap<String, T>, TryReserveError>;
    fn hget_all_values(&self, map_key: String) -> Result<Vec<T>, TryReserveError>;
    fn hkeys(&self, map_key: String) -> Result<Vec<String>, TryReserveError>;
    fn hlen(&self, map_key: String) -> usize;
    fn hupsert(&mut self, map_key: String, key: String, value: T) -> Result<(), TryReserveError>;
    fn hstr_len(&self, map_key: String, key: String) -> Option<usize>;
    fn hincrby(&mut self, map_key: String, key: String, value: i64) -> Result<i64, TryReserveError>;
}

#[derive(Default, Debug)]
pub struct Store {
    map: HashMap<String, String>,
    acl: ACL,
    /// UUID -> hashed password
    users: HashMap<String, String>,
    hash_maps: HashMap<String, HashMap<String, String>>,
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

impl HashMapAble<String> for Store {
    fn hadd(&mut self, map_key: String, key: String, value: String) -> Result<(), TryReserveError> {
        self.hash_maps.try_reserve(1)?;
        let map = self.hash_maps.entry(map_key).or_insert(HashMap::new());
        map.try_reserve(1)?;
        map.insert(key, value);
        Ok(())
    }

    fn hremove(&mut self, map_key: String, key: String) -> bool {
        match self.hash_maps.get_mut(&map_key) {
            Some(map) => {
                map.remove(&key);
                true
            },
            None => false
        }
    }

    fn hcontains(&self, map_key: String, key: String) -> bool {
        match self.hash_maps.get(&map_key) {
            Some(map) => {
                map.contains_key(&key)
            },
            None => false
        }
    }

    fn hget(&self, map_key: String, key: String) -> Option<&String> {
        match self.hash_maps.get(&map_key) {
            Some(map) => {
                map.get(&key)
            },
            None => None
        }
    }

    fn hget_all(&self, map_key: String) -> Result<HashMap<String, String>, TryReserveError> {
        match self.hash_maps.get(&map_key) {
            Some(map) => {
                let mut new_map = HashMap::new();
                new_map.try_reserve(map.len())?;
                for (k, v) in map.iter() {
                    new_map.insert(k.clone(), v.clone());
                }
                Ok(new_map)
            },
            None => Ok(HashMap::new())
        }
    }

    fn hget_all_values(&self, map_key: String) -> Result<Vec<String>, TryReserveError> {
        match self.hash_maps.get(&map_key) {
            Some(map) => {
                let mut values = Vec::new();
                values.try_reserve_exact(map.len())?;
                for v in map.values() {
                    values.push(v.clone());
                }
                Ok(values)
            },
            None => Ok(Vec::new())
        }
    }

    fn hkeys(&self, map_key: String) -> Result<Vec<String>, TryReserveError> {
        match self.hash_maps.get(&map_key) {
            Some(map) => {
                let mut keys = Vec::new();
                keys.try_reserve_exact(map.len())?;
                for k in map.keys() {
                    keys.push(k.clone());
                }
                Ok(keys)
            },
            None => Ok(Vec::new())
        }
    }

    fn hlen(&self, map_key: String) -> usize {
        match self.hash_maps.get(&map_key) {
            Some(map) => map.len(),
            None => 0
        }
    }

    fn hupsert(&mut self, map_key: String, key: String, value: String) -> Result<(), TryReserveError> {
        self.hash_maps.try_reserve(1)?;
        let map = self.hash_maps.entry(map_key).or_insert(HashMap::new());
        map.try_reserve(1)?;
        map.insert(key, value);
        Ok(())
    }

    fn hstr_len(&self, map_key: String, key: String) -> Option<usize> {
        match self.hash_maps.get(&map_key) {
            Some(map) => {
                match map.get(&key) {
                    Some(v) => Some(v.len()),
                    None => None
                }
            },
            None => None
        }
    }

    fn hincrby(&mut self, map_key: String, key: String, value: i64) -> Result<i64, TryReserveError> {
        self.hash_maps.try_reserve(1)?;
        let map = self.hash_maps.entry(map_key).or_insert(HashMap::new());
        map.try_reserve(1)?;
        let new_value = match map.get(&key) {
            Some(v) => {
                let new_value = v.parse::<i64>().unwrap() + value;
                map.insert(key, new_value.to_string());
                new_value
            },
            None => {
                map.insert(key.clone(), value.to_string());
                value
            }
        };
        Ok(new_value)
    }
}