use async_trait::async_trait;
use anyhow::Result;

use super::models::{MediaMetadata, MetadataSearchResult, MediaType};

/// Trait for metadata providers (TMDb, IMDb, Kinopoisk, etc.)
#[async_trait]
pub trait MetadataProvider: Send + Sync {
    /// Provider name for identification
    fn name(&self) -> &str;
    
    /// Whether this provider is enabled/configured
    async fn is_enabled(&self) -> Result<bool>;
    
    /// Search for media by title and optional year
    async fn search(&self, title: &str, year: Option<u32>, media_type: MediaType) -> Result<Vec<MetadataSearchResult>>;
    
    /// Get detailed metadata by provider-specific ID
    async fn get_details(&self, id: &str, media_type: MediaType) -> Result<Option<MediaMetadata>>;
    
    /// Get poster image URL for given path
    fn get_poster_url(&self, path: &str) -> Option<String>;
    
    /// Get backdrop image URL for given path
    fn get_backdrop_url(&self, path: &str) -> Option<String>;
}
