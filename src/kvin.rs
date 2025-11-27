use crate::{KvinError, Result, deserialize_borrowed, serialize_key, serialize_value};
use std::borrow::ToOwned;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

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
#[derive(Debug, PartialEq, Clone)]
pub enum BorrowedValue<'a> {
    Int(i64),
    Text(&'a str),
    Bool(bool),
    Blob(&'a [u8]),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BorrowedEntry<'a> {
    pub key: &'a Key,
    pub value: BorrowedValue<'a>,
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

fn write_u64_le(dst: &mut Vec<u8>, v: u64) {
    dst.extend_from_slice(&v.to_le_bytes());
}

fn read_u64_le(src: &[u8]) -> u64 {
    let mut a = [0u8; 8];
    a.copy_from_slice(&src[..8]);
    u64::from_le_bytes(a)
}

#[derive(Debug)]
pub struct StoreIter<'a> {
    buf: &'a [u8],
    inner: std::collections::hash_map::Iter<'a, Key, usize>,
}

impl<'a> Iterator for StoreIter<'a> {
    type Item = Result<BorrowedEntry<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        let (key, &off) = self.inner.next()?;
        match deserialize_borrowed(&self.buf[off..]) {
            Ok((value, _)) => Some(Ok(BorrowedEntry { key, value })),
            Err(e) => Some(Err(e)),
        }
    }
}

impl<'a> IntoIterator for &'a Kvin {
    type Item = Result<BorrowedEntry<'a>>;
    type IntoIter = StoreIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> std::iter::FusedIterator for StoreIter<'a> {}

/// Trait that defines the Kvin API.
pub trait KvStore {
    /// Insert a key-value pair.
    fn put(&mut self, key: Key, value: Value) -> Result<()>;

    /// Update the value for an existing key.
    fn update(&mut self, key: &Key, new_value: Value) -> Result<()>;

    /// Return a zero-copy value by key.
    fn get<K>(&self, key: K) -> Result<Option<BorrowedValue<'_>>>
    where
        K: Into<Key>;

    /// Delete a key and return its value if it existed.
    fn delete<K>(&mut self, key: K) -> Result<Option<Value>>
    where
        K: Into<Key>;

    /// Return an owned copy of a value by key.
    fn get_owned<K>(&self, key: K) -> Result<Option<Value>>
    where
        K: Into<Key>;
}

/// The actual store implementation.
#[derive(Debug, Clone, Default, PartialEq)]
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
    pub fn get_borrowed<'a>(&'a self, key: &Key) -> Result<Option<BorrowedValue<'a>>> {
        let off = match self.index.get(key) {
            None => return Ok(None),
            Some(&off) => off,
        };

        let (bv, _) = deserialize_borrowed(&self.buf[off..])?;
        Ok(Some(bv))
    }

    pub fn iter(&self) -> StoreIter<'_> {
        StoreIter {
            buf: &self.buf,
            inner: self.index.iter(),
        }
    }

    pub fn keys<'a>(&'a self) -> impl Iterator<Item = Result<&'a Key>> + 'a {
        self.iter().map(|res| res.map(|e| e.key))
    }

    pub fn values<'a>(&'a self) -> impl Iterator<Item = Result<BorrowedValue<'a>>> + 'a {
        self.iter().map(|res| res.map(|e| e.value))
    }

    /// Persist the current store to `dir/kvin.values` and `dir/kvin.index`.
    pub fn save_to_dir<P: AsRef<Path>>(&self, dir: P) -> Result<()> {
        let dir = dir.as_ref();
        fs::create_dir_all(dir)?; // may create it if it doesn't exist

        let values_path = dir.join("kvin.values");
        let index_path = dir.join("kvin.index");

        // 1) Write the raw value buffer
        fs::write(&values_path, &self.buf)?;

        // 2) Serialize the index: [count: u64][entries...]
        let mut idx_buf = Vec::new();

        write_u64_le(&mut idx_buf, self.index.len() as u64);
        for (key, &off) in &self.index {
            // serialize key
            serialize_key(&mut idx_buf, key);
            // serialize offset as u64
            write_u64_le(&mut idx_buf, off as u64);
        }

        fs::write(&index_path, &idx_buf)?;

        Ok(())
    }

    /// Load a Kvin store from `dir/kvin.values` and `dir/kvin.index`.
    ///
    /// If the files do not exist, returns an empty store.
    pub fn load_from_dir<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let dir = dir.as_ref();
        let values_path = dir.join("kvin.values");
        let index_path = dir.join("kvin.index");

        // First-run case: no persistence yet -> empty store
        if !values_path.exists() || !index_path.exists() {
            return Ok(Kvin::new());
        }

        // 1) Read value buffer
        let buf = fs::read(&values_path)?; // KvinError::Io on failure

        // 2) Read and parse the index
        let idx_bytes = fs::read(&index_path)?;
        if idx_bytes.len() < 8 {
            return Err(KvinError::MalformedEntry);
        }

        let mut pos = 0;
        let count = read_u64_le(&idx_bytes[pos..pos + 8]) as usize;
        pos += 8;

        let mut index = HashMap::with_capacity(count);

        for _ in 0..count {
            // deserialize key
            let (key, consumed) = crate::deserialize_key(&idx_bytes[pos..])?;
            pos += consumed;

            // read offset
            if idx_bytes.len() < pos + 8 {
                return Err(KvinError::IncompleteBody);
            }
            let off = read_u64_le(&idx_bytes[pos..pos + 8]) as usize;
            pos += 8;

            // basic structural check: offset must be inside buf
            if off >= buf.len() {
                return Err(KvinError::CorruptedIndex);
            }

            // optional: try to parse the value to catch checksum errors early
            let _ = crate::deserialize_borrowed(&buf[off..])?;

            index.insert(key, off);
        }

        Ok(Kvin { buf, index })
    }
}

impl KvStore for Kvin {
    fn put(&mut self, key: Key, value: Value) -> Result<()> {
        if self.index.contains_key(&key) {
            return Err(KvinError::KeyExists);
        }
        let off = serialize_value(&mut self.buf, &value);
        self.index.insert(key, off);
        Ok(())
    }

    fn update(&mut self, key: &Key, new_value: Value) -> Result<()> {
        if !self.index.contains_key(key) {
            return Err(KvinError::KeyNotFound);
        }
        let off = serialize_value(&mut self.buf, &new_value);
        self.index.insert(key.clone(), off);
        Ok(())
    }

    fn get<K>(&self, key: K) -> Result<Option<BorrowedValue<'_>>>
    where
        K: Into<Key>,
    {
        self.get_borrowed(&key.into())
    }

    fn delete<K>(&mut self, key: K) -> Result<Option<Value>>
    where
        K: Into<Key>,
    {
        let key = key.into();
        let off = match self.index.remove(&key) {
            None => return Ok(None),
            Some(off) => off,
        };

        let (bv, _) = deserialize_borrowed(&self.buf[off..])?;
        Ok(Some(bv.to_owned()))
    }

    fn get_owned<K>(&self, key: K) -> Result<Option<Value>>
    where
        K: Into<Key>,
    {
        match self.get_borrowed(&key.into())? {
            None => Ok(None),
            Some(bv) => Ok(Some(bv.to_owned())),
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
