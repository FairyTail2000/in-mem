use std::collections::{HashMap, TryReserveError};
use std::num::ParseIntError;
use age::x25519::Recipient;

use common::acl::ACL;
use common::command::CommandID;

#[derive(Debug, Clone)]
enum Type {
    String(String),
    HashMap(HashMap<String, String>),
    List(Vec<String>),
    User((String, Option<Recipient>)),
}

pub enum ErrorType {
    TryReserveError(TryReserveError),
    ParseIntError(ParseIntError),
}

impl From<TryReserveError> for ErrorType {
    fn from(value: TryReserveError) -> Self {
        Self::TryReserveError(value)
    }
}

impl From<ParseIntError> for ErrorType {
    fn from(value: ParseIntError) -> Self {
        Self::ParseIntError(value)
    }
}

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
    fn user_add(&mut self, user: &str, password: &str, public_key: Option<Recipient>);
    /// Removes a user from the store. Returns true if the user was removed. Which means it was found in the store
    fn user_remove(&mut self, user: &str) -> bool;
    fn user_is_valid(&self, user: &str, password: &str) -> bool;
    fn verify_key(&self, user: &str, key: &Recipient) -> bool;
    fn user_has_key(&self, user: &str) -> bool;
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
    fn hincrby(&mut self, map_key: String, key: String, value: i64) -> Result<i64, ErrorType>;
}


pub trait ListAble {
    fn llen(&self, list_key: String) -> usize;
    fn lindex(&self, list_key: String, value: String) -> Option<usize>;
    fn lmove(&mut self, src_key: String, dest_key: String, left_right: String, right_left: String) -> Option<String>;
    /// Removes and returns the first element(s) of the list stored at key. Count has a default of 1
    fn lpop(&mut self, list_key: String, count: Option<usize>) -> Result<Option<Vec<String>>, TryReserveError>;
    /// Actually, I don't understand the redis docs at all for this. I'm just going to implement it as I see fit. Since I'm not going to implement redis I'm allowed to do that.
    fn lpos(&self, list_key: String, value: String, rank: Option<isize>, count: Option<usize>, max_len: Option<usize>) -> Result<Option<Vec<usize>>, TryReserveError>;
    fn lpush(&mut self, list_key: String, values: Vec<String>) -> Result<(), TryReserveError>;
    /// Only inserts when the list already exists, otherwise it does nothing
    fn lpushx(&mut self, list_key: String, values: Vec<String>) -> Result<(), TryReserveError>;
    fn lrange(&self, list_key: String, start: isize, stop: isize) -> Result<Vec<String>, TryReserveError>;
    fn lrem(&mut self, list_key: String, count: isize, value: String) -> usize;
    fn lset(&mut self, list_key: String, index: isize, value: String) -> bool;
    fn ltrim(&mut self, list_key: String, start: isize, stop: isize) -> bool;

    fn rpop(&mut self, list_key: String, count: Option<usize>) -> Option<Vec<String>>;
    fn rpush(&mut self, list_key: String, values: Vec<String>) -> Result<(), TryReserveError>;
    fn rpushx(&mut self, list_key: String, values: Vec<String>) -> Result<(), TryReserveError>;
}

#[derive(Default, Debug, Clone)]
pub struct Store {
    acl: ACL,
    values: HashMap<String, Type>,
}

impl StoreAble for Store {
    fn get(&self, key: &str) -> Option<&String> {
        match self.values.get(key) {
            Some(Type::String(s)) => Some(s),
            _ => None
        }
    }

    fn set(&mut self, key: String, value: String) -> Result<(), TryReserveError> {
        match self.values.try_reserve(1) {
            Ok(_) => {
                self.values.insert(key, Type::String(value));
                Ok(())
            }
            Err(_) => {
                self.values.shrink_to_fit();
                match self.values.try_reserve(1) {
                    Ok(_) => {
                        self.values.insert(key, Type::String(value));
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
        match self.values.get(key) {
            None => {
                None
            }
            Some(value) => {
                match value {
                    Type::String(_) => {
                        self.values.remove(key).map(|v| {
                            match v {
                                Type::String(s) => s,
                                _ => unreachable!("Value was not a string, although is was a string when checked previously")
                            }
                        })
                    }
                    _ => None,
                }
            }
        }
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
    fn user_add(&mut self, user: &str, password: &str, public_key: Option<Recipient>) {
        if self.values.contains_key(user) {
            return;
        } else {
            self.values.insert(user.to_string(), Type::User((password.to_string(), public_key)));
        }
    }

    fn user_remove(&mut self, user: &str) -> bool {
        match self.values.get(user) {
            Some(Type::User(_)) => {
                self.values.remove(user);
                true
            }
            _ => false
        }
    }

    fn user_is_valid(&self, user: &str, password: &str) -> bool {
        match self.values.get(user) {
            Some(Type::User((p, _))) => {
                p == password
            }
            _ => false
        }
    }

    fn verify_key(&self, user: &str, key: &Recipient) -> bool {
        match self.values.get(user) {
            Some(Type::User((_, Some(k)))) => {
                k == key
            }
            _ => false
        }
    }

    fn user_has_key(&self, user: &str) -> bool {
        match self.values.get(user) {
            Some(Type::User((_, Some(_)))) => true,
            _ => false
        }
    }
}

impl HashMapAble<String> for Store {
    fn hadd(&mut self, map_key: String, key: String, value: String) -> Result<(), TryReserveError> {
        self.values.try_reserve(1)?;
        if let Type::HashMap(ref mut map) = self.values.entry(map_key).or_insert(Type::HashMap(HashMap::new())) {
            map.try_reserve(1)?;
            map.insert(key, value);
        }
        Ok(())
    }

    fn hremove(&mut self, map_key: String, key: String) -> bool {
        match self.values.get_mut(&map_key) {
            Some(Type::HashMap(map)) => {
                map.remove(&key);
                true
            }
            _ => false
        }
    }

    fn hcontains(&self, map_key: String, key: String) -> bool {
        match self.values.get(&map_key) {
            Some(Type::HashMap(map)) => {
                map.contains_key(&key)
            }
            _ => false
        }
    }

    fn hget(&self, map_key: String, key: String) -> Option<&String> {
        match self.values.get(&map_key) {
            Some(Type::HashMap(map)) => {
                map.get(&key)
            }
            _ => None
        }
    }

    fn hget_all(&self, map_key: String) -> Result<HashMap<String, String>, TryReserveError> {
        match self.values.get(&map_key) {
            Some(Type::HashMap(map)) => {
                let mut new_map = HashMap::new();
                new_map.try_reserve(map.len())?;
                for (k, v) in map.iter() {
                    new_map.insert(k.clone(), v.clone());
                }
                Ok(new_map)
            }
            // Something that is not a hashmap is in the place of the hashmap
            // TODO: handle this better
            Some(_) => Ok(HashMap::new()),
            None => Ok(HashMap::new())
        }
    }

    fn hget_all_values(&self, map_key: String) -> Result<Vec<String>, TryReserveError> {
        match self.values.get(&map_key) {
            Some(Type::HashMap(map)) => {
                let mut values = Vec::new();
                values.try_reserve_exact(map.len())?;
                for v in map.values() {
                    values.push(v.clone());
                }
                Ok(values)
            }
            // Same as above, but for values
            Some(_) => Ok(Vec::new()),
            None => Ok(Vec::new())
        }
    }

    fn hkeys(&self, map_key: String) -> Result<Vec<String>, TryReserveError> {
        match self.values.get(&map_key) {
            Some(Type::HashMap(map)) => {
                let mut keys = Vec::new();
                keys.try_reserve_exact(map.len())?;
                for k in map.keys() {
                    keys.push(k.clone());
                }
                Ok(keys)
            }
            // Same as above, but for values
            Some(_) => Ok(Vec::new()),
            None => Ok(Vec::new())
        }
    }

    fn hlen(&self, map_key: String) -> usize {
        match self.values.get(&map_key) {
            Some(Type::HashMap(map)) => map.len(),
            // This also captures the case where the key does exist, but has a different type
            _ => 0
        }
    }

    fn hupsert(&mut self, map_key: String, key: String, value: String) -> Result<(), TryReserveError> {
        self.values.try_reserve(1)?;
        if let Type::HashMap(ref mut map) = self.values.entry(map_key).or_insert(Type::HashMap(HashMap::new())) {
            map.try_reserve(1)?;
            map.insert(key, value);
        }
        Ok(())
    }

    fn hstr_len(&self, map_key: String, key: String) -> Option<usize> {
        match self.values.get(&map_key) {
            Some(map) => {
                match map {
                    Type::HashMap(map) => {
                        match map.get(&key) {
                            Some(str) => Some(str.len()),
                            None => None
                        }
                    }
                    _ => return None
                }
            }
            None => None
        }
    }

    fn hincrby(&mut self, map_key: String, key: String, value: i64) -> Result<i64, ErrorType> {
        self.values.try_reserve(1)?;
        if let Type::HashMap(ref mut map) = self.values.entry(map_key).or_insert(Type::HashMap(HashMap::new())) {
            map.try_reserve(1)?;
            let new_value = match map.get(&key) {
                Some(v) => {
                    let new_value = v.parse::<i64>()?.checked_add(value).unwrap_or(0);
                    map.insert(key, new_value.to_string());
                    new_value
                }
                None => {
                    map.insert(key.clone(), value.to_string());
                    value
                }
            };
            Ok(new_value)
        } else {
            // This should never happen as the entry is always a hashmap since we are inserting a hashmap
            unreachable!("This should never happen, because a just inserted hashmap is not a hashmap, which is nonsensical")
        }
    }
}

impl ListAble for Store {
    fn llen(&self, list_key: String) -> usize {
        match self.values.get(&list_key) {
            Some(list) => {
                match list {
                    Type::List(l) => l.len(),
                    _ => 0
                }
            }
            None => 0
        }
    }

    fn lindex(&self, list_key: String, value: String) -> Option<usize> {
        match self.values.get(&list_key) {
            Some(list) => {
                match list {
                    Type::List(l) => {
                        l.iter().position(|x| x == &value)
                    }
                    _ => None
                }
            }
            None => None
        }
    }

    fn lmove(&mut self, src_key: String, dest_key: String, left_right: String, right_left: String) -> Option<String> {
        // left_right needs to be either "left" or "right"
        if !left_right.eq_ignore_ascii_case("right") && !left_right.eq_ignore_ascii_case("left") {
            return None;
        }
        // right_left needs to be either "right" or "left"
        if !right_left.eq_ignore_ascii_case("right") && !right_left.eq_ignore_ascii_case("left") {
            return None;
        }
        let mut src = match self.values.remove(&src_key) {
            Some(Type::List(src_list)) => {
                Some(src_list)
            }
            _ => None
        };

        if src.is_none() {
            return None;
        }
        let src_list = src.as_mut().unwrap();
        let ret = match self.values.get_mut(&dest_key) {
            Some(Type::List(dest_list)) => {
                if left_right.eq_ignore_ascii_case("left") {
                    if right_left.eq_ignore_ascii_case("right") {
                        dest_list.push(src_list.remove(0));
                    } else {
                        dest_list.insert(0, src_list.remove(0));
                    }
                } else {
                    if right_left.eq_ignore_ascii_case("right") {
                        dest_list.push(src_list.pop().unwrap());
                    } else {
                        dest_list.insert(0, src_list.pop().unwrap());
                    }
                }
                Some(dest_list.last().unwrap().clone())
            }
            _ => None
        };
        self.values.insert(src_key, Type::List(src_list.clone()));

        ret
    }

    fn lpop(&mut self, list_key: String, count: Option<usize>) -> Result<Option<Vec<String>>, TryReserveError> {
        let count = count.unwrap_or(1);
        match self.values.get_mut(&list_key) {
            Some(Type::List(list)) => {
                let mut popped = Vec::new();
                popped.try_reserve_exact(count)?;
                for _ in 0..count {
                    if let Some(v) = list.pop() {
                        popped.push(v);
                    } else {
                        break;
                    }
                }
                Ok(Some(popped))
            }
            _ => Ok(None)
        }
    }

    fn lpos(&self, list_key: String, value: String, rank: Option<isize>, count: Option<usize>, max_len: Option<usize>) -> Result<Option<Vec<usize>>, TryReserveError> {
        let list = match self.values.get(&list_key) {
            Some(Type::List(l)) => l,
            _ => return Ok(None),
        };

        let rank = rank.unwrap_or(1);
        let count = count.unwrap_or(1);
        let max_len = max_len.unwrap_or(0);

        let mut matches = Vec::new();
        let mut current_rank = if rank > 0 { 1 } else { -1 };
        let mut comparisons = 0;

        matches.try_reserve_exact(list.len())?;


        let iter: Box<dyn Iterator<Item=(usize, &String)>> = if rank > 0 {
            Box::new(list.iter().enumerate())
        } else {
            Box::new(list.iter().enumerate().rev())
        };

        for (index, item) in iter {
            if max_len > 0 && comparisons >= max_len {
                break;
            }
            comparisons += 1;

            if item == &value {
                if current_rank == rank {
                    matches.push(index);
                    if matches.len() == count {
                        break;
                    }
                }
                current_rank += if rank > 0 { 1 } else { -1 };
            }
        }

        if matches.is_empty() {
            Ok(None)
        } else {
            Ok(Some(matches))
        }
    }

    fn lpush(&mut self, list_key: String, values: Vec<String>) -> Result<(), TryReserveError> {
        self.values.try_reserve(1)?;
        if let Type::List(ref mut list) = self.values.entry(list_key).or_insert(Type::List(Vec::new())) {
            list.try_reserve(values.len())?;
            list.extend(values.into_iter());
        }
        Ok(())
    }

    fn lpushx(&mut self, list_key: String, values: Vec<String>) -> Result<(), TryReserveError> {
        match self.values.get_mut(&list_key) {
            Some(Type::List(list)) => {
                list.try_reserve(values.len())?;
                list.extend(values.into_iter());
                Ok(())
            }
            _ => Ok(())
        }
    }

    fn lrange(&self, list_key: String, start: isize, stop: isize) -> Result<Vec<String>, TryReserveError> {
        match self.values.get(&list_key) {
            Some(Type::List(list)) => {
                let mut new_list = Vec::new();
                new_list.try_reserve_exact(list.len())?;
                if start.is_negative() {
                    if stop.is_negative() {
                        for i in (start + list.len() as isize)..(stop + list.len() as isize) {
                            if i > list.len() as isize - 1 {
                                break;
                            }
                            new_list.push(list[i as usize].clone());
                        }
                    } else {
                        for i in (start + list.len() as isize)..stop {
                            if i > list.len() as isize - 1 {
                                break;
                            }
                            new_list.push(list[i as usize].clone());
                        }
                    }
                } else {
                    if stop.is_negative() {
                        for i in start..(stop + list.len() as isize) {
                            if i > list.len() as isize - 1 {
                                break;
                            }
                            new_list.push(list[i as usize].clone());
                        }
                    } else {
                        for i in start..stop {
                            if i > list.len() as isize - 1 {
                                break;
                            }
                            new_list.push(list[i as usize].clone());
                        }
                    }
                }
                Ok(new_list)
            }
            _ => Ok(Vec::new())
        }
    }

    fn lrem(&mut self, list_key: String, count: isize, value: String) -> usize {
        match self.values.get_mut(&list_key) {
            Some(Type::List(list)) => {
                let mut removed = 0;
                let mut indicies = Vec::new();
                for (i, v) in list.iter().enumerate() {
                    if v == &value {
                        indicies.push(i);
                    }
                }
                if count.is_negative() {
                    for i in indicies.iter().rev() {
                        list.remove(*i);
                        removed += 1;
                        if removed == count.abs() as usize {
                            break;
                        }
                    }
                } else {
                    for i in indicies.iter() {
                        list.remove(*i);
                        removed += 1;
                        if removed == count.abs() as usize {
                            break;
                        }
                    }
                }
                removed
            }
            _ => 0
        }
    }

    fn lset(&mut self, list_key: String, index: isize, value: String) -> bool {
        match self.values.get_mut(&list_key) {
            Some(Type::List(list)) => {
                if index.is_negative() {
                    let i = index + list.len() as isize;
                    if i < 0 {
                        return false;
                    }
                    list[i as usize] = value;
                    true
                } else {
                    if index as usize > list.len() - 1 {
                        return false;
                    }
                    list[index as usize] = value;
                    true
                }
            }
            _ => false
        }
    }

    fn ltrim(&mut self, list_key: String, start: isize, stop: isize) -> bool {
        match self.values.get_mut(&list_key) {
            Some(Type::List(list)) => {
                let len = list.len() as isize;
                let start = if start < 0 { len + start } else { start };
                let stop = if stop < 0 { len + stop } else { stop };

                if start >= len || stop < 0 || start > stop {
                    list.clear();
                } else {
                    let start = start.max(0) as usize;
                    let stop = stop.min(len - 1) as usize;
                    list.drain(..start);
                    list.drain((stop - start + 1)..);
                }
                true
            }
            _ => false,
        }
    }

    fn rpop(&mut self, list_key: String, count: Option<usize>) -> Option<Vec<String>> {
        let count = count.unwrap_or(1);
        match self.values.get_mut(&list_key) {
            Some(Type::List(list)) => {
                let mut popped = Vec::new();
                for _ in 0..count {
                    if let Some(v) = list.pop() {
                        popped.push(v);
                    } else {
                        break;
                    }
                }
                if popped.is_empty() {
                    None
                } else {
                    Some(popped)
                }
            }
            _ => None,
        }
    }

    fn rpush(&mut self, list_key: String, values: Vec<String>) -> Result<(), TryReserveError> {
        self.values.try_reserve(1)?;
        if let Type::List(ref mut list) = self.values.entry(list_key).or_insert(Type::List(Vec::new())) {
            list.try_reserve(values.len())?;
            list.extend(values.into_iter());
        }
        Ok(())
    }

    fn rpushx(&mut self, list_key: String, values: Vec<String>) -> Result<(), TryReserveError> {
        match self.values.get_mut(&list_key) {
            Some(Type::List(list)) => {
                list.try_reserve(values.len())?;
                list.extend(values.into_iter());
                Ok(())
            }
            _ => Ok(()),
        }
    }
}