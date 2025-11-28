use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use anyhow::{Result, anyhow};
use tracing::{debug, info, warn};

use crate::indexers::traits::{Indexer, IndexerConfig};

pub struct IndexerRegistry {
    indexers: Arc<RwLock<HashMap<String, Arc<dyn Indexer>>>>,
}

impl IndexerRegistry {
    pub fn new() -> Self {
        Self {
            indexers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register an indexer with the registry
    pub fn register(&self, indexer: Arc<dyn Indexer>) -> Result<()> {
        let name = indexer.name().to_string();
        let mut indexers = self.indexers.write().map_err(|e| {
            anyhow!("Failed to acquire write lock for indexer registration: {}", e)
        })?;

        if indexers.contains_key(&name) {
            warn!("Indexer '{}' is already registered, skipping", name);
            return Ok(());
        }

        info!("Registering indexer: {}", name);
        indexers.insert(name, indexer);
        Ok(())
    }

    /// Unregister an indexer by name
    pub fn unregister(&self, name: &str) -> Result<()> {
        let mut indexers = self.indexers.write().map_err(|e| {
            anyhow!("Failed to acquire write lock for indexer unregistration: {}", e)
        })?;

        if indexers.remove(name).is_some() {
            info!("Unregistered indexer: {}", name);
            Ok(())
        } else {
            warn!("Attempted to unregister non-existent indexer: {}", name);
            Err(anyhow!("Indexer '{}' not found", name))
        }
    }

    /// Get an indexer by name
    pub fn get(&self, name: &str) -> Result<Option<Arc<dyn Indexer>>> {
        let indexers = self.indexers.read().map_err(|e| {
            anyhow!("Failed to acquire read lock for indexer lookup: {}", e)
        })?;
        Ok(indexers.get(name).cloned())
    }

    /// Get all registered indexers
    pub fn get_all(&self) -> Result<Vec<Arc<dyn Indexer>>> {
        let indexers = self.indexers.read().map_err(|e| {
            anyhow!("Failed to acquire read lock for getting all indexers: {}", e)
        })?;
        Ok(indexers.values().cloned().collect())
    }

    /// Get only enabled indexers
    pub async fn get_enabled(&self) -> Result<Vec<Arc<dyn Indexer>>> {
        let all_indexers = self.get_all()?;
        let mut enabled = Vec::new();

        for indexer in all_indexers {
            match indexer.is_enabled().await {
                Ok(true) => {
                    debug!("Indexer '{}' is enabled", indexer.name());
                    enabled.push(indexer);
                }
                Ok(false) => {
                    debug!("Indexer '{}' is disabled", indexer.name());
                }
                Err(e) => {
                    warn!("Failed to check enabled status for indexer '{}': {}", indexer.name(), e);
                }
            }
        }

        Ok(enabled)
    }

    /// Get indexer names
    pub fn get_names(&self) -> Result<Vec<String>> {
        let indexers = self.indexers.read().map_err(|e| {
            anyhow!("Failed to acquire read lock for getting indexer names: {}", e)
        })?;
        Ok(indexers.keys().cloned().collect())
    }

    /// Check if an indexer is registered
    pub fn is_registered(&self, name: &str) -> Result<bool> {
        let indexers = self.indexers.read().map_err(|e| {
            anyhow!("Failed to acquire read lock for checking registration: {}", e)
        })?;
        Ok(indexers.contains_key(name))
    }

    /// Get the count of registered indexers
    pub fn count(&self) -> Result<usize> {
        let indexers = self.indexers.read().map_err(|e| {
            anyhow!("Failed to acquire read lock for counting indexers: {}", e)
        })?;
        Ok(indexers.len())
    }

    /// Test all registered indexers
    pub async fn test_all(&self) -> HashMap<String, Result<bool>> {
        let mut results = HashMap::new();
        
        if let Ok(indexers) = self.get_all() {
            for indexer in indexers {
                let name = indexer.name().to_string();
                let result = indexer.test_connection().await;
                results.insert(name, result);
            }
        }

        results
    }
}

impl Default for IndexerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Macro for easy indexer registration
#[macro_export]
macro_rules! register_indexer {
    ($registry:expr, $indexer_type:ty, $config:expr) => {
        {
            let indexer = Arc::new(<$indexer_type>::new($config));
            $registry.register(indexer)
        }
    };
}

/// Trait for indexer factories to enable dynamic creation
pub trait IndexerFactory: Send + Sync {
    fn create(&self, config: IndexerConfig) -> Result<Arc<dyn Indexer>>;
    fn indexer_name(&self) -> &str;
    fn default_config(&self) -> IndexerConfig;
}

/// Registry for indexer factories
pub struct IndexerFactoryRegistry {
    factories: Arc<RwLock<HashMap<String, Arc<dyn IndexerFactory>>>>,
}

impl IndexerFactoryRegistry {
    pub fn new() -> Self {
        Self {
            factories: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn register_factory(&self, factory: Arc<dyn IndexerFactory>) -> Result<()> {
        let name = factory.indexer_name().to_string();
        let mut factories = self.factories.write().map_err(|e| {
            anyhow!("Failed to acquire write lock for factory registration: {}", e)
        })?;

        if factories.contains_key(&name) {
            warn!("Indexer factory '{}' is already registered", name);
            return Ok(());
        }

        info!("Registering indexer factory: {}", name);
        factories.insert(name, factory);
        Ok(())
    }

    pub fn create_indexer(&self, name: &str, config: IndexerConfig) -> Result<Arc<dyn Indexer>> {
        let factories = self.factories.read().map_err(|e| {
            anyhow!("Failed to acquire read lock for factory lookup: {}", e)
        })?;

        if let Some(factory) = factories.get(name) {
            factory.create(config)
        } else {
            Err(anyhow!("Indexer factory '{}' not found", name))
        }
    }

    pub fn get_available_indexers(&self) -> Result<Vec<String>> {
        let factories = self.factories.read().map_err(|e| {
            anyhow!("Failed to acquire read lock for getting available indexers: {}", e)
        })?;
        Ok(factories.keys().cloned().collect())
    }
}

impl Default for IndexerFactoryRegistry {
    fn default() -> Self {
        Self::new()
    }
}
