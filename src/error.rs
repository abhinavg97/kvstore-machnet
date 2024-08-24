use std::fmt;
use std::io;

#[derive(Debug)]
pub enum KVStoreError {
    Io(io::Error),
    Serialization(serde_json::Error),
    MachnetError(String),
}

impl fmt::Display for KVStoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            KVStoreError::Io(err) => write!(f, "I/O error: {}", err),
            KVStoreError::Serialization(err) => write!(f, "Serialization error: {}", err),
            KVStoreError::MachnetError(err) => write!(f, "Machnet error: {}", err),
        }
    }
}

impl From<io::Error> for KVStoreError {
    fn from(err: io::Error) -> KVStoreError {
        KVStoreError::Io(err)
    }
}

impl From<serde_json::Error> for KVStoreError {
    fn from(err: serde_json::Error) -> KVStoreError {
        KVStoreError::Serialization(err)
    }
}