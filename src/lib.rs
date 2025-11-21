pub mod kvin;
mod serialization;
mod error;

pub use kvin::{BorrowedEntry, BorrowedValue, Key, KvStore, Kvin, StoreIter, Value};

pub use crate::serialization::{
    RAW_HEADER_SIZE, RawHeader, deserialize_borrowed, deserialize_unsafe, serialize_unsafe,
    serialize_value,
};

pub use crate::error::Result;
