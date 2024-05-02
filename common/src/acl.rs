use std::collections::{HashMap, HashSet};
use crate::command::CommandID;


#[derive(Debug, Default, Clone)]
pub struct ACL {
    map: HashMap<String, HashSet<CommandID>>,
}

impl ACL {
    pub fn add(&mut self, user: &str, command: CommandID) {
        self.map.entry(user.to_string()).or_default().insert(command);
    }

    pub fn remove(&mut self, user: &str, command: CommandID) {
        if let Some(set) = self.map.get_mut(user) {
            set.remove(&command);
        }
    }

    pub fn is_allowed(&self, user: &str, command: CommandID) -> bool {
        if command == CommandID::KEYEXCHANGE || command == CommandID::Login || command == CommandID::Heartbeat {
            return true;
        }

        self.map.get(user).map_or(false, |set| set.contains(&command))
    }

    pub fn list(&self, user: &str) -> Vec<CommandID> {
        self.map.get(user).map_or(Vec::new(), |set| set.iter().copied().collect())
    }
}
