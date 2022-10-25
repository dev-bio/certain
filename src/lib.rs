//! Lightweight utility for listening to certificate transparency logs.
//!
//! ## Example
//! ```
//! use std::time::{Duration};
//! 
//! use certain::{
//!     
//!     StreamConfig,
//!     StreamError, 
//! };
//! 
//! fn main() -> Result<(), StreamError> {
//!     let config = StreamConfig::new("https://ct.googleapis.com/logs/argon2022/")
//!         .timeout(Duration::from_secs(1));
//! 
//!     certain::stream(config, |entry| {
//!         println!("{entry:#?}");
//!         true
//!     })?;
//! 
//!     Ok(())
//! }
//! ```

use std::time::{Duration};

use reqwest::blocking::{Client};

mod certificate_log;
pub mod certificate;
pub mod error;

pub use certificate_log::{Entry};
pub use error::{StreamError};

pub struct StreamConfig<E: AsRef<str>> {
    pub endpoint: E,
    pub timeout: Option<Duration>,
    pub index: Option<usize>,
}

impl <E: AsRef<str>> StreamConfig<E> {
    pub fn new(endpoint: E) -> Self {
        StreamConfig { 

            endpoint: endpoint, 
            timeout: Some(Duration::from_secs(1)),
            index: None, 
        }
    }

    pub fn timeout(self, timeout: Duration) -> Self {
        StreamConfig { 
            endpoint: self.endpoint, 
            timeout: Some(timeout), 
            index: self.index,
        }
    }

    pub fn index(self, index: usize) -> Self {
        StreamConfig { 
            endpoint: self.endpoint, 
            timeout: self.timeout, 
            index: Some(index),
        }
    }
}

pub fn stream<E, F>(config : StreamConfig<E>, mut handler: F) -> Result<(), StreamError>
where E: AsRef<str>, F: FnMut(Entry) -> bool {

    let client = Client::new();

    let StreamConfig { 
        endpoint, 
        timeout, 
        index,
    } = config;

    let mut current = certificate_log::get_log_size(&client, endpoint.as_ref())?;

    if let Some(index) = index {
        current = if index < current { index } 
            else { current }
    }
    
    loop {

        let entries = certificate_log::get_log_entries(&client, endpoint.as_ref(), current, 10000)?;

        if entries.is_empty() {
            if let Some(timeout) = timeout {
                std::thread::sleep(timeout);
            }
        }

        else {

            current += entries.len();

            for entry in entries {
                if handler(entry) { continue } else {
                    return Ok(())
                }
            }     
        }       
    }
}