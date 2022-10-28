use thiserror::{Error as ThisError};

#[derive(ThisError, Debug)]
pub enum LogError {
    #[error("Unsupported version, got: {0}")]
    UnsupportedVersion(u8),
    #[error("Unsupported leaf, got: {0}")]
    UnsupportedLeaf(u8),
    #[error("Unsupported entry, got: {0}")]
    UnsupportedEntry(u16),
    #[error("Parsing failed, info: {0}")]
    Parse(&'static str),
}

#[derive(ThisError, Debug)]
pub enum StreamError {
    #[error("Invalid log endpoint.")]
    InvalidEndpoint,
    #[error("Connection error, info: {0}")]
    Connection(&'static str),
    #[error("Response error, info: {0}")]
    Response(&'static str),
    #[error("Concurrency error, info: {0}")]
    Concurrency(&'static str),
    #[error("Parse error.")]
    Parse(LogError),
}