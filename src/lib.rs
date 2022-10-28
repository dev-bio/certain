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
//!     certain::blocking::stream(config, |entry| {
//!         println!("{entry:#?}");
//!         true // Continue!
//!     })
//! }
//! ```

use std::{
    
    time::{Duration}, 
    fmt::{Debug},
};

use tokio::runtime::{Runtime};

use futures::{StreamExt};
use reqwest::{Client};

pub mod certificate;
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
            workers: None,
            index: self.index,
            batch: self.batch,
            url: self.url, 
        }
    }

    pub fn workers(self, workers: usize) -> Self {
        StreamConfig { 
            timeout: None,
            workers: Some(workers),
            index: self.index,
            batch: self.batch,
            url: self.url, 
        }
    }

    pub fn index(self, index: usize) -> Self {
        StreamConfig { 
            timeout: self.timeout, 
            workers: None,
            index: Some(index),
            batch: self.batch,
            url: self.url, 
        }
    }

    pub fn batch(self, batch: usize) -> Self {
        StreamConfig { 
            timeout: self.timeout, 
            workers: None,
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

    let size = endpoint::get_log_size(client.clone(), url.clone()).await?;

    let position = index.unwrap_or(size).min(size);
    let workers = workers.unwrap_or(num_cpus::get()).max(1);
    let batch = batch.unwrap_or(1000).max(1);
    
    let timeout = timeout.unwrap_or({
        Duration::from_secs(1)
    });

    let mut iterator = futures::stream::iter((position..)
        .step_by(batch)).map(|start| {

            let client = client.clone();
            let url = url.clone();

            tokio::spawn(async move {
                let mut collection = Vec::with_capacity(batch);
                
                while collection.len() < batch {

                    let start = start + collection.len();
                    let count = batch - collection.len();

                    let entries = endpoint::get_log_entries(client.clone(), url.as_str(), start, count).await?;

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
                }

                Ok(collection)
            })
        }).buffered(workers);

    while let Some(result) = iterator.next().await {
        let entries = result.map_err(|_| StreamError::Concurrency({
            "failed to join task!"
        }))??;

        for entry in entries {
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
        Entry
    };

    use super::{Runtime};
    use super::{Debug};
    
    pub fn stream<U, F>(config : StreamConfig<U>, handler: F) -> Result<(), StreamError>
    where U: AsRef<str> + Clone + Debug, F: FnMut(Entry) -> bool {

        let runtime = Runtime::new().map_err(|_| StreamError::Concurrency({
            "failed to create runtime!"
        }))?;

        runtime.block_on(async {
            super::stream(config, handler).await
        })
    }
}