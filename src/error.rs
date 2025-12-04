use thiserror::Error;

#[derive(Error, Debug)]
pub enum KvinError {
    #[error("Key already exists")]
    KeyExists,

    #[error("Key not found")]
    KeyNotFound,

    #[error("Checksum mismatch")]
    ChecksumMismatch,

    #[error("Invalid UTF-8 in text value")]
    InvalidUtf8,

    #[error("Unknown value tag: {0}")]
    UnknownTag(u8),

    #[error("Invalid or incomplete header")]
    InvalidHeader,

    #[error("Entry body is incomplete")]
    IncompleteBody,

    #[error("Malformed or corrupted entry")]
    MalformedEntry,

    #[error("I/O error while accessing persistence files: {0}")]
    Io(#[from] std::io::Error),

    #[error("Corrupted on-disk index")]
    CorruptedIndex,

    #[error("Autosave failed: {0}")]
    AutosaveFailed(String),
}

// A specialized Result type for Kvin operations.
pub type Result<T> = std::result::Result<T, KvinError>;
