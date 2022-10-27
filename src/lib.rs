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

use std::{
    
    time::{Duration}, 
    fmt::{Debug},
};

use rayon::prelude::{

    IntoParallelIterator,
    ParallelIterator,
};

use reqwest::blocking::{Client};

mod certificate_log;
pub mod certificate;
pub mod error;

pub use certificate_log::{Entry};
pub use error::{StreamError};

#[derive(Debug, Clone)]
pub struct StreamConfig<E>
where E: AsRef<str> + Debug {
    pub endpoint: E,
    pub timeout: Option<Duration>,
    pub workers: Option<usize>,
    pub index: Option<usize>,
}

impl<E> StreamConfig<E> 
where E: AsRef<str> + Debug {
    pub fn new(endpoint: E) -> Self {
        StreamConfig { 

            endpoint: endpoint, 
            timeout: Some(Duration::from_secs(1)),
            workers: None,
            index: None, 
        }
    }

    pub fn timeout(self, timeout: Duration) -> Self {
        StreamConfig { 
            endpoint: self.endpoint, 
            timeout: Some(timeout), 
            workers: self.workers,
            index: self.index,
        }
    }

    pub fn workers(self, threads: usize) -> Self {
        StreamConfig { 
            endpoint: self.endpoint, 
            timeout: self.timeout, 
            workers: Some(threads.max(2)),
            index: self.index,
        }
    }

    pub fn index(self, index: usize) -> Self {
        StreamConfig { 
            endpoint: self.endpoint, 
            timeout: self.timeout, 
            workers: self.workers,
            index: Some(index),
        }
    }
}

pub fn stream<E, F, R>(config : StreamConfig<E>, mut handler: F) -> Result<R, StreamError>
where E: AsRef<str> + Debug, F: FnMut(Entry) -> Option<R> {

    let client = Client::new();

    let StreamConfig { 
        endpoint, 
        timeout, 
        workers,
        index,
    } = config;

    let endpoint = endpoint.as_ref();

    let (size, batch) = {

        let size = certificate_log::get_log_size(&client, endpoint)?;
        let batch = certificate_log::get_log_entries(&client, endpoint, 0, size)?.len();

        (size, batch)
    };

    let mut position = if let Some(index) = index {
        if index < size { index } else { size } 
    } else { 0 };

    if let Some(workers) = workers {
        type WorkResult = Result<Vec<Entry>, StreamError>;

        loop {

            let collection: Vec<WorkResult> = (0..(workers)).into_par_iter()
                .map(|offset| -> WorkResult {

                    let start = position + (offset * batch);
                    let mut collection = Vec::with_capacity(batch);
                    
                    loop {

                        let start = start + collection.len();
                        let count = batch - collection.len();

                        let entries = certificate_log::get_log_entries(&client, endpoint, start, count)?;

                        if entries.is_empty() {
                            if let Some(timeout) = timeout {
                                std::thread::sleep(timeout);
                            }
                        }

                        else {

                            collection.extend(entries);
                            
                            if collection.len() < batch { continue }
                                else { return Ok(collection) }
                        }
                    }
                }).collect();

            for result in collection {
                match result {

                    Err(error) => {

                        return Err(error)
                    },

                    Ok(entries) => {

                        position = position + {
                            entries.len()
                        };

                        for entry in entries {
                            if let Some(result) = handler(entry) {
                                return Ok(result)
                            }
                        }
                    },
                }
            }
        }
    }

    else {
    
        loop {

            let entries = certificate_log::get_log_entries(&client, endpoint, position, batch)?;

            if entries.is_empty() {
                if let Some(timeout) = timeout {
                    std::thread::sleep(timeout);
                }
            }

            else {

                position = position + {
                    entries.len()
                };

                for entry in entries {
                    if let Some(result) = handler(entry) {
                        return Ok(result)
                    }
                }  
            }       
        }
    }
}