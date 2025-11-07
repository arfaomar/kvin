use std::borrow::ToOwned;
use std::collections::HashMap;
use crate::serialization::{deserialize_borrowed, serialize_value};
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

/// Borrowed view of values, zero-copy from the byte buffer.
#[derive(Debug, PartialEq)]
pub enum BorrowedValue<'a> {
    Int(i64),
    Text(&'a str),
    Bool(bool),
    Blob(&'a [u8]),
}

impl<'a> BorrowedValue<'a> {
    pub fn to_owned(&self) -> Value {
        match self {
            BorrowedValue::Int(i) => Value::Int(*i),
            BorrowedValue::Text(s) => Value::Text((*s).to_owned()),
            BorrowedValue::Bool(b) => Value::Bool(*b),
            BorrowedValue::Blob(b) => Value::Blob((*b).to_owned()),
        }
    }
}


/// Trait that defines the Kvin API.
pub trait KvStore {
    /// Insert a key-value pair.
    fn put(&mut self, key: Key, value: Value) -> bool;

    /// Update the value for an existing key.
    fn update(&mut self, key: &Key, new_value: Value) -> bool;

    /// Return a zero-copy value by key.
    fn get<K>(&self, key: K) -> Option<BorrowedValue<'_>>
    where
        K: Into<Key>;

    /// Delete a key and return its value if it existed.
    fn delete<K>(&mut self, key: K) -> Option<Value>
    where
        K: Into<Key>;

    /// Return an owned copy of a value by key.
    fn get_owned<K>(&self, key: K) -> Option<Value>
    where
        K: Into<Key>;
}

/// The actual store implementation.
#[derive(Debug, Default)]
pub struct Kvin {
    /// Append-only bytes (our “log”).
    buf: Vec<u8>,
    /// Maps a key to the start offset of its most recent serialized value in `buf`.
    index: HashMap<Key, usize>,
}

impl Kvin {
    /// Create a new, empty Kvin store.
    pub fn new() -> Self {
        Self {
            buf: Vec::new(),
            index: HashMap::new(),
        }
    }
    /// zero-copy borrowed get method.
    pub fn get_borrowed<'a>(&'a self, key: &Key) -> Option<BorrowedValue<'a>> {
        let off = *self.index.get(key)?;
        let (bv, _len) = deserialize_borrowed(&self.buf[off..])?;
        Some(bv)
    }
}



impl KvStore for Kvin {
    fn put(&mut self, key: Key, value: Value) -> bool {
        if self.index.contains_key(&key) {
            // do not overwrite existing key (test expects false)
            return false;
        }
        let off = serialize_value(&mut self.buf, &value);
        self.index.insert(key, off);
        true
    }

    fn update(&mut self, key: &Key, new_value: Value) -> bool {
        if !self.index.contains_key(key) {
            return false;
        }
        let off = serialize_value(&mut self.buf, &new_value);
        self.index.insert(key.clone(), off);
        true
    }

    fn get<K>(&self, key: K) -> Option<BorrowedValue<'_>>
    where
        K: Into<Key>,
    {
        let key = key.into();
        self.get_borrowed(&key)
    }

   fn delete<K>(&mut self, key: K) -> Option<Value>
    where
        K: Into<Key>,
    {
        let key = key.into();
        let off = self.index.remove(&key)?;
        let (bv, _len) = deserialize_borrowed(&self.buf[off..])?;
        Some(bv.to_owned())
    }

    fn get_owned<K>(&self, key: K) -> Option<Value>
    where
        K: Into<Key>,
    {
        let key = key.into();
        self.get_borrowed(&key).map(|bv| bv.to_owned())
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
