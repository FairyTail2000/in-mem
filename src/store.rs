pub trait StoreAble {
    fn get(&self, key: &str) -> Option<&String>;
    fn set(&mut self, key: String, value: String);
    fn remove(&mut self, key: &str) -> Option<String>;
}

#[derive(Default, Debug)]
pub struct Store {
    map: std::collections::HashMap<String, String>,
}

impl StoreAble for Store {
    fn get(&self, key: &str) -> Option<&String> {
        self.map.get(key)
    }

    fn set(&mut self, key: String, value: String) {
        self.map.insert(key, value);
    }

    fn remove(&mut self, key: &str) -> Option<String> {
        self.map.remove(key)
    }
}