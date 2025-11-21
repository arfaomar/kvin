use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum KvinError {

    #[error("Checksum mismatch")]
    ChecksumMismatch,

    #[error("Text value is not valid UTF-8")]
    InvalidUtf8,

    #[error("Unknown value tag: {0}")]
    UnknownTag(u8),

    #[error("Incomplete or malformed entry in buffer")]
    MalformedEntry,
}

// A specialized Result type for Kvin operations.
pub type Result<T> = std::result::Result<T, KvinError>;