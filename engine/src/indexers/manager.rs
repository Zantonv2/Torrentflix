use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use futures::future::join_all;
use anyhow::{Result, anyhow};
use tracing::{debug, error, info, warn};

use crate::indexers::traits::{Indexer, SearchQuery};
use crate::indexers::registry::IndexerRegistry;
use crate::models::TorrentResult;

pub struct IndexerManager {
    registry: Arc<IndexerRegistry>,
    max_concurrent_searches: usize,
    search_timeout: Duration,
    semaphore: Arc<Semaphore>,
}

impl IndexerManager {
    pub fn new(registry: Arc<IndexerRegistry>) -> Self {
        Self {
            registry,
            max_concurrent_searches: 10,
            search_timeout: Duration::from_secs(30),
            semaphore: Arc::new(Semaphore::new(10)),
        }
    }

    /// Get access to the registry (for engine initialization)
    pub fn registry(&self) -> &Arc<IndexerRegistry> {
        &self.registry
    }

    /// Search across all enabled indexers in parallel
    pub async fn search_all(&self, query: &SearchQuery) -> Result<Vec<TorrentResult>> {
        let enabled_indexers = self.registry.get_enabled().await?;
        
        if enabled_indexers.is_empty() {
            warn!("No enabled indexers available for search");
            return Ok(Vec::new());
        }

        info!("Searching across {} enabled indexers for: {}", enabled_indexers.len(), query.query);
        
        let search_futures: Vec<_> = enabled_indexers
            .into_iter()
            .map(|indexer| self.search_single_indexer(indexer, query.clone()))
            .collect();

        let results = join_all(search_futures).await;
        
        let mut all_results = Vec::new();
        let mut successful_searches = 0;
        let mut failed_searches = 0;

        for result in results {
            match result {
                Ok(indexer_results) => {
                    debug!("Got {} results from indexer", indexer_results.len());
                    all_results.extend(indexer_results);
                    successful_searches += 1;
                }
                Err(e) => {
                    error!("Indexer search failed: {}", e);
                    failed_searches += 1;
                }
            }
        }

        info!(
            "Search completed: {} successful, {} failed, {} total results",
            successful_searches,
            failed_searches,
            all_results.len()
        );

        Ok(all_results)
    }

    /// Search specific indexers by name
    pub async fn search_indexers(
        &self,
        indexer_names: &[String],
        query: &SearchQuery,
    ) -> Result<Vec<TorrentResult>> {
        let mut search_futures = Vec::new();

        for name in indexer_names {
            if let Ok(Some(indexer)) = self.registry.get(name) {
                search_futures.push(self.search_single_indexer(indexer, query.clone()));
            } else {
                warn!("Indexer '{}' not found or not available", name);
            }
        }

        if search_futures.is_empty() {
            warn!("No valid indexers found for search");
            return Ok(Vec::new());
        }

        let results = join_all(search_futures).await;
        let mut all_results = Vec::new();

        for result in results {
            match result {
                Ok(indexer_results) => all_results.extend(indexer_results),
                Err(e) => error!("Indexer search failed: {}", e),
            }
        }

        Ok(all_results)
    }

    /// Search a single indexer with timeout and error handling
    async fn search_single_indexer(
        &self,
        indexer: Arc<dyn Indexer>,
        query: SearchQuery,
    ) -> Result<Vec<TorrentResult>> {
        let _permit = self.semaphore.acquire().await.map_err(|e| {
            anyhow!("Failed to acquire semaphore permit: {}", e)
        })?;

        let indexer_name = indexer.name().to_string();
        debug!("Starting search on indexer: {}", indexer_name);

        let search_future = async {
            indexer.search(&query).await
        };

        let timeout_future = tokio::time::timeout(self.search_timeout, search_future);
        
        match timeout_future.await {
            Ok(Ok(results)) => {
                debug!("Indexer '{}' returned {} results", indexer_name, results.len());
                Ok(results)
            }
            Ok(Err(e)) => {
                error!("Indexer '{}' search error: {}", indexer_name, e);
                Err(anyhow!("Search failed on indexer '{}': {}", indexer_name, e))
            }
            Err(_) => {
                warn!("Indexer '{}' search timed out after {:?}", indexer_name, self.search_timeout);
                Err(anyhow!("Search timed out on indexer '{}'", indexer_name))
            }
        }
    }

    /// Get detailed information about a torrent from any available indexer
    pub async fn get_details(&self, info_hash: &str) -> Result<Option<TorrentResult>> {
        let enabled_indexers = self.registry.get_enabled().await?;
        
        for indexer in enabled_indexers {
            match indexer.get_details(info_hash).await {
                Ok(Some(details)) => {
                    debug!("Found details for {} in indexer {}", info_hash, indexer.name());
                    return Ok(Some(details));
                }
                Ok(_none) => {
                    debug!("No details found for {} in indexer {}", info_hash, indexer.name());
                    continue;
                }
                Err(e) => {
                    warn!("Failed to get details from indexer {}: {}", indexer.name(), e);
                    continue;
                }
            }
        }

        debug!("No details found for {} in any indexer", info_hash);
        Ok(None)
    }

    /// Test connectivity to all enabled indexers
    pub async fn test_connectivity(&self) -> Result<Vec<(String, bool)>> {
        let enabled_indexers = self.registry.get_enabled().await?;
        let mut results = Vec::new();

        for indexer in enabled_indexers {
            let name = indexer.name().to_string();
            let is_connected = match indexer.test_connection().await {
                Ok(connected) => connected,
                Err(e) => {
                    error!("Connection test failed for indexer {}: {}", name, e);
                    false
                }
            };
            results.push((name, is_connected));
        }

        Ok(results)
    }

    /// Get statistics about available indexers
    pub async fn get_stats(&self) -> Result<IndexerStats> {
        let all_indexers = self.registry.get_all()?;
        let enabled_indexers = self.registry.get_enabled().await?;
        let connectivity_results = self.test_connectivity().await?;

        let connected_count = connectivity_results.iter().filter(|(_, connected)| *connected).count();

        Ok(IndexerStats {
            total_indexers: all_indexers.len(),
            enabled_indexers: enabled_indexers.len(),
            connected_indexers: connected_count,
            max_concurrent_searches: self.max_concurrent_searches,
            search_timeout_seconds: self.search_timeout.as_secs(),
        })
    }

    /// Update configuration for a specific indexer
    pub async fn update_indexer_config(
        &self,
        indexer_name: &str,
        config: crate::indexers::traits::IndexerConfig,
    ) -> Result<()> {
        if let Ok(Some(indexer)) = self.registry.get(indexer_name) {
            // Note: This requires the indexer to be mutable, which might need
            // a different approach in a real implementation (e.g., using Arc<Mutex<>>
            // or recreating the indexer with new config)
            warn!("Indexer config update not fully implemented for {}", indexer_name);
            Ok(())
        } else {
            Err(anyhow!("Indexer '{}' not found", indexer_name))
        }
    }
}

#[derive(Debug, Clone)]
pub struct IndexerStats {
    pub total_indexers: usize,
    pub enabled_indexers: usize,
    pub connected_indexers: usize,
    pub max_concurrent_searches: usize,
    pub search_timeout_seconds: u64,
}

impl Default for IndexerManager {
    fn default() -> Self {
        Self::new(Arc::new(IndexerRegistry::default()))
    }
}

#[cfg(test)]
mod tests {
    use anyhow::{Result, anyhow};
    use crate::indexers::{IndexerRegistry, Indexer, IndexerManager};
    use crate::indexers::traits::{SearchQuery, IndexerConfig};
    use crate::models::TorrentResult;
    use std::sync::Arc;

    // Mock indexer for testing
    struct MockIndexer {
        name: String,
        config: IndexerConfig,
        should_fail: bool,
    }

    impl MockIndexer {
        fn new(name: String, should_fail: bool) -> Self {
            Self {
                name,
                config: IndexerConfig::default(),
                should_fail,
            }
        }
    }

    #[async_trait::async_trait]
    impl Indexer for MockIndexer {
        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            "Mock indexer for testing"
        }

        fn base_url(&self) -> &str {
            "https://mock.example.com"
        }

        async fn is_enabled(&self) -> Result<bool> {
            Ok(true) // Mock indexers are always enabled
        }

        async fn search(&self, _query: &SearchQuery) -> Result<Vec<TorrentResult>> {
            if self.should_fail {
                Err(anyhow!("Mock indexer configured to fail"))
            } else {
                Ok(vec![
                    TorrentResult::new(
                        "Mock Movie 2023".to_string(),
                        "magnet:?xt=urn:btih:mock".to_string(),
                        1_000_000_000,
                        100,
                        50,
                        self.name.clone(),
                    )
                ])
            }
        }

        async fn get_details(&self, _info_hash: &str) -> Result<Option<TorrentResult>> {
            Ok(None)
        }

        async fn test_connection(&self) -> Result<bool> {
            Ok(!self.should_fail)
        }

        fn config(&self) -> &IndexerConfig {
            &self.config
        }

        async fn update_config(&mut self, _config: IndexerConfig) -> Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_search_all_indexers() {
        let registry = Arc::new(IndexerRegistry::new());
        
        // Register mock indexers
        registry.register(Arc::new(MockIndexer::new("mock1".to_string(), false))).unwrap();
        registry.register(Arc::new(MockIndexer::new("mock2".to_string(), true))).unwrap();
        
        let manager = IndexerManager::new(registry);
        let query = SearchQuery::new("test movie".to_string());
        
        let results = manager.search_all(&query).await.unwrap();
        
        // Should get results from the successful indexer only
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].source, "mock1");
    }

    #[tokio::test]
    async fn test_get_stats() {
        let registry = Arc::new(IndexerRegistry::new());
        registry.register(Arc::new(MockIndexer::new("mock1".to_string(), false))).unwrap();
        
        let manager = IndexerManager::new(registry);
        let stats = manager.get_stats().await.unwrap();
        
        assert_eq!(stats.total_indexers, 1);
        assert_eq!(stats.enabled_indexers, 1);
        assert_eq!(stats.connected_indexers, 1);
    }
}
