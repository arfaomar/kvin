pub mod kvin;
mod serialization;

pub use kvin::{
    BorrowedEntry, BorrowedKey, BorrowedValue, Key, KvStore, Kvin,  Value,
};

pub use serialization::{
    RAW_HEADER_SIZE, RawHeader,
    deserialize_borrowed, deserialize_unsafe, serialize_unsafe, serialize_value,
};