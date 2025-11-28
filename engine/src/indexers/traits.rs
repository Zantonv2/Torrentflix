use async_trait::async_trait;
use std::collections::HashMap;
use anyhow::Result;
use crate::models::TorrentResult;

#[async_trait]
pub trait Indexer: Send + Sync {
    /// Unique identifier for this indexer
    fn name(&self) -> &str;
    
    /// Human-readable description of what this indexer provides
    fn description(&self) -> &str;
    
    /// Base URL or main domain of the indexer
    fn base_url(&self) -> &str;
    
    /// Whether this indexer is currently enabled/available
    async fn is_enabled(&self) -> Result<bool>;
    
    /// Search for torrents matching the query
    async fn search(&self, query: &SearchQuery) -> Result<Vec<TorrentResult>>;
    
    /// Get detailed information about a specific torrent (if supported)
    async fn get_details(&self, info_hash: &str) -> Result<Option<TorrentResult>>;
    
    /// Test connectivity to the indexer
    async fn test_connection(&self) -> Result<bool>;
    
    /// Get indexer-specific configuration
    fn config(&self) -> &IndexerConfig;
    
    /// Update indexer configuration
    async fn update_config(&mut self, config: IndexerConfig) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub query: String,
    pub media_type: Option<MediaType>,
    pub year: Option<u32>,
    pub season: Option<u32>,
    pub episode: Option<u32>,
    pub min_seeders: Option<u32>,
    pub max_results: Option<usize>,
    pub category: Option<String>,
    pub sort_by: Option<SortBy>,
    pub sort_order: Option<SortOrder>,
}

impl SearchQuery {
    pub fn new(query: String) -> Self {
        Self {
            query,
            media_type: None,
            year: None,
            season: None,
            episode: None,
            min_seeders: None,
            max_results: Some(50),
            category: None,
            sort_by: None,
            sort_order: None,
        }
    }

    pub fn with_media_type(mut self, media_type: MediaType) -> Self {
        self.media_type = Some(media_type);
        self
    }

    pub fn with_year(mut self, year: u32) -> Self {
        self.year = Some(year);
        self
    }

    pub fn with_season(mut self, season: u32) -> Self {
        self.season = Some(season);
        self
    }

    pub fn with_episode(mut self, episode: u32) -> Self {
        self.episode = Some(episode);
        self
    }

    pub fn with_min_seeders(mut self, min_seeders: u32) -> Self {
        self.min_seeders = Some(min_seeders);
        self
    }

    pub fn with_max_results(mut self, max_results: usize) -> Self {
        self.max_results = Some(max_results);
        self
    }

    pub fn with_category(mut self, category: String) -> Self {
        self.category = Some(category);
        self
    }

    pub fn with_sort(mut self, sort_by: SortBy, sort_order: SortOrder) -> Self {
        self.sort_by = Some(sort_by);
        self.sort_order = Some(sort_order);
        self
    }
}

use crate::models::MediaType;

#[derive(Debug, Clone, PartialEq)]
pub enum SortBy {
    Seeders,
    Leechers,
    Size,
    UploadDate,
    Relevance,
    Title,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

#[derive(Debug, Clone)]
pub struct IndexerConfig {
    pub enabled: bool,
    pub timeout_seconds: u64,
    pub rate_limit_ms: u64,
    pub user_agent: Option<String>,
    pub headers: HashMap<String, String>,
    pub cookies: Option<String>,
    pub proxy_url: Option<String>,
    pub api_key: Option<String>,
    pub custom_params: HashMap<String, String>,
}

impl Default for IndexerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            timeout_seconds: 30,
            rate_limit_ms: 1000,
            user_agent: Some("MovieDownloader/1.0".to_string()),
            headers: HashMap::new(),
            cookies: None,
            proxy_url: None,
            api_key: None,
            custom_params: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IndexerCapabilities {
    pub supports_categories: bool,
    pub supports_sorting: bool,
    pub supports_detailed_info: bool,
    pub supports_magnet_links: bool,
    pub supports_torrent_files: bool,
    pub max_results_per_page: Option<usize>,
    pub rate_limit_per_minute: Option<u32>,
}

impl Default for IndexerCapabilities {
    fn default() -> Self {
        Self {
            supports_categories: false,
            supports_sorting: false,
            supports_detailed_info: false,
            supports_magnet_links: true,
            supports_torrent_files: false,
            max_results_per_page: None,
            rate_limit_per_minute: None,
        }
    }
}
