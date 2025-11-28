use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::Semaphore;
use tracing::{debug, error, info, warn};
use crate::models::TorrentResult;

pub mod yts;
pub mod monna;
pub mod registry;
pub mod manager;
pub mod traits;

pub use traits::Indexer;
pub use manager::IndexerManager;
pub use registry::IndexerRegistry;
pub use yts::YtsIndexer;
pub use monna::MonnaIndexer;
