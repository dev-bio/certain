//! Client for listening to certificate transparency logs.
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
//!     let config = StreamConfig::new("https://ct.googleapis.com/logs/argon2023/")
//!         .timeout(Duration::from_secs(1))
//!         .workers(4)
//!         .batch(1);
//! 
//!     certain::blocking::stream(config, |entry| {
//!         println!("{entry:#?}");
//!         true // continue
//!     })
//! }
//! ```

use std::{
    
    time::{Duration}, 
    fmt::{Debug},
};

use endpoint::Response;
use tokio::runtime::{Runtime};

use futures::{StreamExt};
use reqwest::{Client};

pub mod error;

mod endpoint;

pub use endpoint::{Entry};
pub use error::{StreamError};

#[derive(Debug, Clone)]
pub struct StreamConfig<U>
where U: AsRef<str> + Clone + Debug {
    pub timeout: Option<Duration>,
    pub workers: Option<usize>,
    pub index: Option<usize>,
    pub batch: Option<usize>,
    pub url: U,
}

impl<U> StreamConfig<U> 
where U: AsRef<str> + Clone + Debug {
    pub fn new(url: U) -> Self {
        StreamConfig { 

            timeout: None,
            workers: None,
            index: None, 
            batch: None,
            url, 
        }
    }

    pub fn timeout(self, timeout: Duration) -> Self {
        StreamConfig { 
            timeout: Some(timeout), 
            workers: self.workers,
            index: self.index,
            batch: self.batch,
            url: self.url, 
        }
    }

    pub fn workers(self, workers: usize) -> Self {
        StreamConfig { 
            timeout: self.timeout,
            workers: Some(workers),
            index: self.index,
            batch: self.batch,
            url: self.url, 
        }
    }

    pub fn index(self, index: usize) -> Self {
        StreamConfig { 
            timeout: self.timeout, 
            workers: self.workers,
            index: Some(index),
            batch: self.batch,
            url: self.url, 
        }
    }

    pub fn batch(self, batch: usize) -> Self {
        StreamConfig { 
            timeout: self.timeout, 
            workers: self.workers,
            index: self.index,
            batch: Some(batch),
            url: self.url, 
        }
    }
}

pub async fn stream<U, F>(config : StreamConfig<U>, mut handler: F) -> Result<(), StreamError>
where U: AsRef<str> + Clone + Debug, F: FnMut(Entry) -> bool {

    let StreamConfig { 
        timeout, 
        workers,
        index,
        batch,
        url, 
    } = config;

    let client = Client::new();
    let url = String::from({
        url.as_ref()
    });

    let workers = workers.unwrap_or(num_cpus::get()).max(1);
    let batch = batch.unwrap_or(1000).max(1);

    let timeout = timeout.unwrap_or({
        Duration::from_secs(1)
    });

    let size = loop {
        
        let response = endpoint::get_log_size(client.clone(), url.clone()).await?;

        match response {

            Response::Data(size) => {
                break size
            },

            Response::Limited(Some(duration)) => {
                tokio::time::sleep({
                    duration
                }).await;
            },

            Response::Limited(None) => {
                tokio::time::sleep({
                    timeout
                }).await;
            },

            Response::Unhandled(400) => {
                tokio::time::sleep({
                    timeout
                }).await;
            },

            _ => continue,
        }
    };

    let position = index.unwrap_or(size).min(size);

    let mut iterator = futures::stream::iter((position..)
        .step_by(batch)).map(|start| {

            let client = client.clone();
            let url = url.clone();

            tokio::spawn(async move {
                let mut collection = Vec::with_capacity(batch);
                
                loop {

                    let start = start + collection.len();
                    let count = batch - collection.len();

                    let response = match endpoint::get_log_entries(client.clone(), url.as_str(), start, count).await {
                        Err(error) => return Err(error),
                        Ok(response) => response,
                    };

                    match response {

                        Response::Data(entries) => {
                            if entries.is_empty() { 
                                tokio::time::sleep({
                                    timeout
                                }).await;
                            }
        
                            else {
        
                                collection.extend(entries);
                                if collection.len() < batch { continue }
                                    else { break }
                            }
                        },

                        Response::Limited(Some(duration)) => {
                            tokio::time::sleep({
                                duration
                            }).await;
                        },

                        Response::Limited(None) => {
                            tokio::time::sleep({
                                timeout
                            }).await;
                        },

                        Response::Unhandled(400) => {
                            tokio::time::sleep({
                                timeout
                            }).await;
                        },

                        _ => continue,
                    }
                }

                Ok(collection)
            })
        }).buffered(workers);

    while let Some(result) = iterator.next().await {
        for entry in result.map_err(|error| StreamError::Task(error))?? {
            if handler(entry) { continue } else {
                return Ok(())
            }
        }
    }

    Ok(())
}

pub mod blocking {
    
    use super::{

        StreamConfig, 
        StreamError, 
        Entry,
    };

    use super::{Runtime};
    use super::{Debug};
    
    pub fn stream<U, F>(config : StreamConfig<U>, handler: F) -> Result<(), StreamError>
    where U: AsRef<str> + Clone + Debug, F: FnMut(Entry) -> bool {

        let runtime = Runtime::new()?;

        runtime.block_on(async {
            super::stream(config, handler).await
        })
    }
}