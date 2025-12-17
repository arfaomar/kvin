use crate::{
    KvinError, Result, deserialize_borrowed, deserialize_key, serialize_key, serialize_value,
};
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
    fn compact(&mut self) -> Result<()>;
}

/// The actual store implementation.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Kvin {
    /// Append-only bytes (our “log”).
    pub buf: Vec<u8>,

    /// Maps a key to the start offset of its most recent serialized value in `buf`.
    pub index: HashMap<Key, usize>,

    /// Optional directory for automatic persistence.
    /// None => purely in-memory store, only manual save_to_dir/load_from_dir.
    pub persistence_dir: Option<std::path::PathBuf>,

    /// Number of mutating operations since the last automatic save.
    dirty_ops: u64,

    /// Number of mutating operations before an automatic save is triggered.
    pub autosave_threshold: Option<u64>,

    /// Ratio of live entries to total buffer size that triggers automatic compaction after an autosave.
    pub compaction_ratio: Option<u64>,
}

impl Kvin {
    /// Create a new, empty Kvin store.
    pub fn new() -> Self {
        Self {
            buf: Vec::new(),
            index: HashMap::new(),
            persistence_dir: None,
            dirty_ops: 0,
            autosave_threshold: None, // in-memory
            compaction_ratio: None,   // no automatic compaction
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
        use std::fs::File;
        use std::io::Write;

        let dir = dir.as_ref();
        fs::create_dir_all(dir)?; // may create it if it doesn't exist

        let values_path = dir.join("kvin.values");
        let index_path = dir.join("kvin.index");

        let values_tmp = dir.join("kvin.values.tmp");
        let index_tmp = dir.join("kvin.index.tmp");

        // 1) Write the raw value buffer to a temp file
        {
            let mut f = File::create(&values_tmp)?;
            f.write_all(&self.buf)?;
            f.sync_all()?; // flush to disk
        }

        // 2) Serialize the index into a buffer and write to temp
        let mut idx_buf = Vec::new();
        write_u64_le(&mut idx_buf, self.index.len() as u64);
        for (key, &off) in &self.index {
            serialize_key(&mut idx_buf, key);
            write_u64_le(&mut idx_buf, off as u64);
        }

        {
            let mut f = File::create(&index_tmp)?;
            f.write_all(&idx_buf)?;
            f.sync_all()?;
        }

        // 3) Atomically replace the main files with the temp files
        fs::rename(&values_tmp, &values_path)?;
        fs::rename(&index_tmp, &index_path)?;

        Ok(())
    }

    /// Load a Kvin store from `dir/kvin.values` and `dir/kvin.index`.
    /// If the files do not exist, returns an empty store.
    pub fn load_from_dir<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let dir = dir.as_ref();
        let values_path = dir.join("kvin.values");
        let index_path = dir.join("kvin.index");

        let values_tmp = dir.join("kvin.values.tmp");
        let index_tmp = dir.join("kvin.index.tmp");

        let main_exists = values_path.exists() && index_path.exists();
        let tmp_exists = values_tmp.exists() && index_tmp.exists();

        // Helper closure: load from arbitrary value + index paths
        fn load_from_paths(values: &Path, index: &Path) -> Result<Kvin> {
            let buf = fs::read(values)?;

            let idx_bytes = fs::read(index)?;
            if idx_bytes.len() < 8 {
                return Err(KvinError::MalformedEntry);
            }

            let mut pos = 0;
            let count = read_u64_le(&idx_bytes[pos..pos + 8]) as usize;
            pos += 8;

            let mut index = HashMap::with_capacity(count);

            for _ in 0..count {
                let (key, consumed) = deserialize_key(&idx_bytes[pos..])?;
                pos += consumed;

                if idx_bytes.len() < pos + 8 {
                    return Err(KvinError::IncompleteBody);
                }
                let off = read_u64_le(&idx_bytes[pos..pos + 8]) as usize;
                pos += 8;

                if off >= buf.len() {
                    return Err(KvinError::CorruptedIndex);
                }

                let _ = deserialize_borrowed(&buf[off..])?;

                index.insert(key, off);
            }

            Ok(Kvin {
                buf,
                index,
                persistence_dir: None,
                dirty_ops: 0,
                autosave_threshold: None,
                compaction_ratio: None,
            })
        }

        if main_exists {
            // Normal case: just load the main files
            return load_from_paths(&values_path, &index_path);
        }

        if tmp_exists {
            // Recovery case: try to recover from temp files
            let store = load_from_paths(&values_tmp, &index_tmp)?;

            // If that worked, promote .tmp to main (best-effort)
            let _ = fs::rename(&values_tmp, &values_path);
            let _ = fs::rename(&index_tmp, &index_path);

            return Ok(store);
        }

        // First run: nothing there yet
        Ok(Kvin::new())
    }

    /// Open a persistent Kvin store rooted at `dir` that remembers the directory for automatic persistence and Drop.
    pub fn open<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let dir_ref = dir.as_ref();
        let mut store = Kvin::load_from_dir(dir_ref)?;
        store.persistence_dir = Some(dir.as_ref().to_path_buf());
        store.dirty_ops = 0;
        store.autosave_threshold = Some(10);
        store.compaction_ratio = Some(8);

        Ok(store)
    }

    fn mark_dirty(&mut self) -> Result<()> {
        // Only autosave if persistent storage is active
        let threshold = match (self.persistence_dir.as_ref(), self.autosave_threshold) {
            (Some(_), Some(n)) => n,
            _ => return Ok(()),
        };

        self.dirty_ops += 1;

        if self.dirty_ops >= threshold {
            if let Some(dir) = &self.persistence_dir {
                match self.save_to_dir(dir) {
                    Ok(_) => {
                        if let Some(ratio) = self.compaction_ratio {
                            let live = self.index.len();
                            let threshold = live.saturating_mul(ratio as usize);

                            if self.buf.len() > threshold {
                                self.do_compact()?;
                            }
                        }

                        self.dirty_ops = 0;
                    }
                    Err(e) => {
                        return Err(KvinError::AutosaveFailed(format!(
                            "Could not autosave to {}: {}",
                            dir.display(),
                            e
                        )));
                    }
                }
            }
        }

        Ok(())
    }

    pub fn set_autosave_threshold(&mut self, new_threshold: Option<u64>) -> () {
        self.autosave_threshold = new_threshold;
    }

    pub fn set_compaction_ratio(&mut self, ratio: Option<u64>) {
        self.compaction_ratio = ratio;
    }

    fn do_compact(&mut self) -> Result<()> {
        let mut new_buf = Vec::new();
        let mut new_index = HashMap::new();

        for (key, &old_off) in &self.index {
            let (bv, _) = deserialize_borrowed(&self.buf[old_off..])?;
            let new_off = serialize_value(&mut new_buf, &bv.to_owned());
            new_index.insert(key.clone(), new_off);
        }

        self.buf = new_buf;
        self.index = new_index;

        Ok(())
    }
}

impl KvStore for Kvin {
    fn put(&mut self, key: Key, value: Value) -> Result<()> {
        if self.index.contains_key(&key) {
            return Err(KvinError::KeyExists);
        }
        let off = serialize_value(&mut self.buf, &value);
        self.index.insert(key, off);
        self.mark_dirty()?;
        Ok(())
    }

    fn update(&mut self, key: &Key, new_value: Value) -> Result<()> {
        if !self.index.contains_key(key) {
            return Err(KvinError::KeyNotFound);
        }
        let off = serialize_value(&mut self.buf, &new_value);
        self.index.insert(key.clone(), off);
        self.mark_dirty()?;
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
        let result = Some(bv.to_owned());
        self.mark_dirty()?;
        Ok(result)
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

    fn compact(&mut self) -> Result<()> {
        self.do_compact()
    }
}

impl Drop for Kvin {
    fn drop(&mut self) {
        if let (Some(dir), Some(_threshold)) = (&self.persistence_dir, self.autosave_threshold) {
            if self.dirty_ops > 0 {
                let _ = self.save_to_dir(dir);
            }
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

impl From<&Key> for Key {
    fn from(k: &Key) -> Self {
        k.clone()
    }
}
