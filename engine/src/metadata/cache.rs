use std::collections::HashMap;
use std::time::{Duration, Instant};
use anyhow::Result;
use async_trait::async_trait;

use super::traits::MetadataProvider;
use super::models::{MediaMetadata, MetadataSearchResult, MediaType};

/// Simple in-memory LRU cache for metadata responses
pub struct MetadataCache {
    provider: Box<dyn MetadataProvider>,
    search_cache: HashMap<String, CachedSearchResult>,
    details_cache: HashMap<String, CachedMetadata>,
    cache_ttl: Duration,
    name: String,
}

/// Cached search result with timestamp
#[derive(Debug, Clone)]
struct CachedSearchResult {
    results: Vec<MetadataSearchResult>,
    cached_at: Instant,
}

/// Cached metadata with timestamp
#[derive(Debug, Clone)]
struct CachedMetadata {
    metadata: MediaMetadata,
    cached_at: Instant,
}

impl MetadataCache {
    pub fn new(provider: Box<dyn MetadataProvider>, cache_ttl_seconds: u64) -> Self {
        let name = format!("Cached({})", provider.name());
        Self {
            provider,
            search_cache: HashMap::new(),
            details_cache: HashMap::new(),
            cache_ttl: Duration::from_secs(cache_ttl_seconds),
            name,
        }
    }

    /// Generate cache key for search queries
    fn search_cache_key(title: &str, year: Option<u32>, media_type: &MediaType) -> String {
        format!("{}:{}:{}", title, year.unwrap_or(0), serde_json::to_string(media_type).unwrap_or_default())
    }

    /// Clean expired entries from search cache
    fn clean_expired_search(&mut self) {
        let now = Instant::now();
        self.search_cache.retain(|_, cached| now.duration_since(cached.cached_at) < self.cache_ttl);
    }

    /// Clean expired entries from details cache
    fn clean_expired_details(&mut self) {
        let now = Instant::now();
        self.details_cache.retain(|_, cached| now.duration_since(cached.cached_at) < self.cache_ttl);
    }

    /// Check if cached search result is still valid
    fn is_search_valid(&self, cached: &CachedSearchResult) -> bool {
        Instant::now().duration_since(cached.cached_at) < self.cache_ttl
    }

    /// Check if cached metadata is still valid
    fn is_metadata_valid(&self, cached: &CachedMetadata) -> bool {
        Instant::now().duration_since(cached.cached_at) < self.cache_ttl
    }
}

#[async_trait]
impl MetadataProvider for MetadataCache {
    fn name(&self) -> &str {
        &self.name
    }

    async fn is_enabled(&self) -> Result<bool> {
        self.provider.is_enabled().await
    }

    async fn search(&self, title: &str, year: Option<u32>, media_type: MediaType) -> Result<Vec<MetadataSearchResult>> {
        let cache_key = Self::search_cache_key(title, year, &media_type);
        
        // Check cache first (mutable borrow for cleanup)
        if let Some(cached) = self.search_cache.get(&cache_key) {
            if self.is_search_valid(cached) {
                return Ok(cached.results.clone());
            }
        }

        // Cache miss or expired - fetch from provider
        let results = self.provider.search(title, year, media_type).await?;
        
        // Cache the results (note: this requires interior mutability in a real implementation)
        // For now, we'll return results directly without caching
        // TODO: Implement proper interior mutability with Arc<Mutex<>> or similar
        
        Ok(results)
    }

    async fn get_details(&self, id: &str, media_type: MediaType) -> Result<Option<MediaMetadata>> {
        // Check cache first
        if let Some(cached) = self.details_cache.get(id) {
            if self.is_metadata_valid(cached) {
                return Ok(Some(cached.metadata.clone()));
            }
        }

        // Cache miss or expired - fetch from provider
        let metadata = self.provider.get_details(id, media_type).await?;
        
        // Cache the result (note: this requires interior mutability in a real implementation)
        // TODO: Implement proper interior mutability with Arc<Mutex<>> or similar
        
        Ok(metadata)
    }

    fn get_poster_url(&self, path: &str) -> Option<String> {
        self.provider.get_poster_url(path)
    }

    fn get_backdrop_url(&self, path: &str) -> Option<String> {
        self.provider.get_backdrop_url(path)
    }
}

/// Metadata manager that coordinates multiple providers
pub struct MetadataManager {
    providers: Vec<Box<dyn MetadataProvider>>,
    primary_provider: Option<Box<dyn MetadataProvider>>,
}

impl MetadataManager {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
            primary_provider: None,
        }
    }

    pub fn add_provider(mut self, provider: Box<dyn MetadataProvider>) -> Self {
        self.providers.push(provider);
        self
    }

    pub fn set_primary(mut self, provider: Box<dyn MetadataProvider>) -> Self {
        self.primary_provider = Some(provider);
        self
    }

    /// Search using primary provider, fallback to others if needed
    pub async fn search(&self, title: &str, year: Option<u32>, media_type: MediaType) -> Result<Vec<MetadataSearchResult>> {
        // Try primary provider first
        if let Some(primary) = &self.primary_provider {
            if let Ok(results) = primary.search(title, year, media_type.clone()).await {
                if !results.is_empty() {
                    return Ok(results);
                }
            }
        }

        // Try other providers as fallback
        for provider in &self.providers {
            if let Ok(results) = provider.search(title, year, media_type.clone()).await {
                if !results.is_empty() {
                    return Ok(results);
                }
            }
        }

        Ok(Vec::new())
    }

    /// Get details using primary provider, fallback to others
    pub async fn get_details(&self, id: &str, media_type: MediaType) -> Result<Option<MediaMetadata>> {
        // Try primary provider first
        if let Some(primary) = &self.primary_provider {
            if let Ok(Some(metadata)) = primary.get_details(id, media_type.clone()).await {
                return Ok(Some(metadata));
            }
        }

        // Try other providers as fallback
        for provider in &self.providers {
            if let Ok(Some(metadata)) = provider.get_details(id, media_type.clone()).await {
                return Ok(Some(metadata));
            }
        }

        Ok(None)
    }
}

impl Default for MetadataManager {
    fn default() -> Self {
        Self::new()
    }
}
