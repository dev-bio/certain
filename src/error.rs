pub use tokio::task::{JoinError as TaskError};
pub use reqwest::{Error as RequestError};
pub use url::{ParseError as UrlError};

use thiserror::{Error as ThisError};

use std::io::{Error as IoError};

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
pub enum ResponseError {
    #[error("Client error, code: {0}")]
    Client(u16),
    #[error("Server error, code: {0}")]
    Server(u16),
}

#[derive(ThisError, Debug)]
pub enum StreamError {
    #[error("Endpoint error.")]
    Endpoint(#[from] UrlError),
    #[error("Request error.")]
    Request(#[from] RequestError),
    #[error("Response error.")]
    Response(#[from] ResponseError),
    #[error("Runtime error.")]
    Runtime(#[from] IoError),
    #[error("Task error.")]
    Task(#[from] TaskError),
    #[error("Log error.")]
    Log(#[from] LogError),
}