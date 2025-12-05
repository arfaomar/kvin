mod error;
pub mod kvin;
mod serialization;

pub use kvin::{BorrowedEntry, BorrowedValue, Key, KvStore, Kvin, StoreIter, Value};

pub use crate::serialization::{
    RAW_HEADER_SIZE, RawHeader, deserialize_borrowed, deserialize_key, deserialize_unsafe,
    serialize_key, serialize_unsafe, serialize_value,
};

pub use crate::error::{KvinError, Result};
