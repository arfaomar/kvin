use std::collections::HashMap;
use std::hash::{Hash};

/// Represents possible key types.
/// Must implement Eq + Hash for use in HashMap.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Key {
    Int(i64),
    Text(String),
    Bool(bool),
}

/// Represents possible value types.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Text(String),
    Bool(bool),
    Blob(Vec<u8>),
}

/// Trait that defines the Kvin API.
pub trait KvStore {
    /// Insert a key-value pair.
    fn put(&mut self, key: Key, value: Value) -> bool;
    /// Retrieve a value by key.
    fn get(&self, key: &Key) -> Option<&Value>;
    /// Delete a key and return its value if it existed.
    fn delete(&mut self, key: &Key) -> Option<Value>;
    /// Update the value for an existing key.
    fn update(&mut self, key: &Key, new_value: Value) -> bool;
}

/// The actual store implementation.
#[derive(Debug, Default)]
pub struct Kvin {
    data: HashMap<Key, Value>,
}

impl Kvin {
    /// Create a new, empty Kvin store.
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

impl KvStore for Kvin {
    fn put(&mut self, key: Key, value: Value) -> bool {
        if self.data.contains_key(&key) {
            false
        } else {
            self.data.insert(key, value);
            true
        }
    }

    fn get(&self, key: &Key) -> Option<&Value> {
        self.data.get(key)
    }

    fn delete(&mut self, key: &Key) -> Option<Value> {
        self.data.remove(key)
    }

    fn update(&mut self, key: &Key, new_value: Value) -> bool {
        if let Some(v) = self.data.get_mut(key) {
            *v = new_value;
            true
        } else {
            false
        }
    }
}

/// Optional helper conversions for ergonomics
impl From<&str> for Key {
    fn from(s: &str) -> Self {
        Key::Text(s.to_string())
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::Text(s.to_string())
    }
}

impl From<i64> for Key {
    fn from(i: i64) -> Self {
        Key::Int(i)
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Value::Int(i)
    }
}

impl From<bool> for Key {
    fn from(b: bool) -> Self {
        Key::Bool(b)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl From<i32> for Key {
    fn from(i: i32) -> Self {
        Key::Int(i as i64)
    }
}

impl From<i32> for Value {
    fn from(i: i32) -> Self {
        Value::Int(i as i64)
    }
}
