use std::collections::HashMap;
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
            BorrowedValue::Text(s) => Value::Text((*s).to_string()),
            BorrowedValue::Bool(b) => Value::Bool(*b),
            BorrowedValue::Blob(b) => Value::Blob(b.to_vec()),
        }
    }
}

/// On-disk type tags (1 byte)
const TAG_INT: u8 = 1;
const TAG_TEXT: u8 = 2;
const TAG_BOOL: u8 = 3;
const TAG_BLOB: u8 = 4;

/// Entry header layout (little-endian):
/// [total_len: u64][tag: u8]  followed by the tagged payload:
/// - Int:  [i64: 8]
/// - Bool: [u8: 1]
/// - Text: [len: u64][bytes: len]
/// - Blob: [len: u64][bytes: len]
///
/// `total_len` counts bytes after the u64 length field itself (i.e. tag + payload).
#[inline]
fn write_u64_le(dst: &mut Vec<u8>, v: u64) {
    dst.extend_from_slice(&v.to_le_bytes());
}
#[inline]
fn read_u64_le(src: &[u8]) -> u64 {
    let mut a = [0u8; 8];
    a.copy_from_slice(&src[..8]);
    u64::from_le_bytes(a)
}
#[inline]
fn write_i64_le(dst: &mut Vec<u8>, v: i64) {
    dst.extend_from_slice(&v.to_le_bytes());
}
#[inline]
fn read_i64_le(src: &[u8]) -> i64 {
    let mut a = [0u8; 8];
    a.copy_from_slice(&src[..8]);
    i64::from_le_bytes(a)
}

/// Trait that defines the Kvin API.
pub trait KvStore {
    /// Insert a key-value pair.
    fn put(&mut self, key: Key, value: Value) -> bool;
    /// Return a zero-copy value by key.
    fn get(&self, key: &Key) -> Option<BorrowedValue<'_>>;
    /// Delete a key and return its value if it existed.
    fn delete(&mut self, key: &Key) -> Option<Value>;
    /// Update the value for an existing key.
    fn update(&mut self, key: &Key, new_value: Value) -> bool;
    /// Return an owned copy of a value by key.
    fn get_owned(&self, key: &Key) -> Option<Value>;
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

pub fn serialize_value(buf: &mut Vec<u8>, value: &Value) -> usize {
    let start = buf.len();
    // Reserve room: [len:u64][tag:u8] + payload
    // First, push a placeholder for len to fill later:
    let len_pos = buf.len();
    write_u64_le(buf, 0); // placeholder
    // tag
    match value {
        Value::Int(i) => {
            buf.push(TAG_INT);
            write_i64_le(buf, *i);
        }
        Value::Bool(b) => {
            buf.push(TAG_BOOL);
            buf.push(if *b { 1 } else { 0 });
        }
        Value::Text(s) => {
            buf.push(TAG_TEXT);
            let bytes = s.as_bytes();
            write_u64_le(buf, bytes.len() as u64);
            buf.extend_from_slice(bytes);
        }
        Value::Blob(bytes) => {
            buf.push(TAG_BLOB);
            write_u64_le(buf, bytes.len() as u64);
            buf.extend_from_slice(bytes);
        }
    }

    // total_len = (current_len - after_len_field)
    let after_len = len_pos + 8;
    let total_len = (buf.len() - after_len) as u64;
    // write it back
    buf[len_pos..len_pos + 8].copy_from_slice(&total_len.to_le_bytes());

    start
}

/// Returns (borrowed_value, total_entry_len_bytes)
pub fn deserialize_borrowed<'a>(src: &'a [u8]) -> Option<(BorrowedValue<'a>, usize)> {
    // need at least 8 (len) + 1 (tag)
    if src.len() < 9 {
        return None;
    }

    let total_len = read_u64_le(&src[..8]) as usize;
    // total_len counts tag + payload, so full entry size is 8 + total_len
    if total_len == 0 {
        return None;
    } // invalid: must contain at least the tag
    if src.len() < 8 + total_len {
        return None;
    } // ensure full entry is available

    let tag = src[8];
    let payload_len = total_len - 1; // exclude tag
    let payload = &src[9..9 + payload_len];

    match tag {
        TAG_INT => {
            if payload.len() < 8 {
                return None;
            }
            Some((
                BorrowedValue::Int(read_i64_le(&payload[..8])),
                8 + total_len,
            ))
        }
        TAG_BOOL => {
            if payload.is_empty() {
                return None;
            }
            Some((BorrowedValue::Bool(payload[0] != 0), 8 + total_len))
        }
        TAG_TEXT | TAG_BLOB => {
            if payload.len() < 8 {
                return None;
            }
            let n = read_u64_le(&payload[..8]) as usize;
            if payload.len() < 8 + n {
                return None;
            }
            let bytes = &payload[8..8 + n];
            if tag == TAG_TEXT {
                let s = std::str::from_utf8(bytes).ok()?;
                Some((BorrowedValue::Text(s), 8 + total_len))
            } else {
                Some((BorrowedValue::Blob(bytes), 8 + total_len))
            }
        }
        _ => None,
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

    fn get(&self, _key: &Key) -> Option<BorrowedValue<'_>> {
        self.get_borrowed(_key)
    }

    fn delete(&mut self, key: &Key) -> Option<Value> {
        let off = self.index.remove(key)?;
        let (bv, _len) = deserialize_borrowed(&self.buf[off..])?;
        Some(bv.to_owned())
    }

    fn update(&mut self, key: &Key, new_value: Value) -> bool {
        if !self.index.contains_key(key) {
            return false;
        }
        let off = serialize_value(&mut self.buf, &new_value);
        self.index.insert(key.clone(), off);
        true
    }

    fn get_owned(&self, key: &Key) -> Option<Value> {
        self.get_borrowed(key).map(|bv| bv.to_owned())
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
