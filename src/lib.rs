pub mod kvin;
mod serialization;

pub use kvin::{BorrowedValue, Key, KvStore, Kvin, Value};

// Used in tests.rs
// We shouldn't test it, instead we should test KvStore methods that use it. (Black-Box)
pub use serialization::{deserialize_borrowed, serialize_value};
