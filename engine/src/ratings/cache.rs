use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::models::{CachedTMDBMetadata, CachedRating};

/// Cache for rating manager responses
#[derive(Debug, Clone)]
pub struct RatingCache {
    // TMDB Worker 1 cache: Full metadata responses
    tmdb_metadata: Arc<Mutex<HashMap<String, CachedTMDBMetadata>>>,
    
    // TMDB Worker 2 cache: IMDb ratings
    imdb_ratings: Arc<Mutex<HashMap<String, CachedRating>>>,
    
    // Kinopoisk Worker cache: Kinopoisk ratings
    kinopoisk_ratings: Arc<Mutex<HashMap<String, CachedRating>>>,
}

impl RatingCache {
    pub fn new() -> Self {
        Self {
            tmdb_metadata: Arc::new(Mutex::new(HashMap::new())),
            imdb_ratings: Arc::new(Mutex::new(HashMap::new())),
            kinopoisk_ratings: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get TMDB metadata from cache
    pub async fn get_tmdb_metadata(&self, key: &str) -> Option<CachedTMDBMetadata> {
        self.tmdb_metadata.lock().await.get(key).cloned()
    }

    /// Store TMDB metadata in cache
    pub async fn set_tmdb_metadata(&self, key: String, metadata: CachedTMDBMetadata) {
        self.tmdb_metadata.lock().await.insert(key, metadata);
    }

    /// Get IMDb rating from cache
    pub async fn get_imdb_rating(&self, imdb_id: &str) -> Option<CachedRating> {
        let cache_key = format!("imdb_rating:{}", imdb_id);
        self.imdb_ratings.lock().await.get(&cache_key).cloned()
    }

    /// Store IMDb rating in cache
    pub async fn set_imdb_rating(&self, imdb_id: String, rating: CachedRating) {
        let cache_key = format!("imdb_rating:{}", imdb_id);
        self.imdb_ratings.lock().await.insert(cache_key, rating);
    }

    /// Get Kinopoisk rating from cache
    pub async fn get_kinopoisk_rating(&self, key: &str) -> Option<CachedRating> {
        self.kinopoisk_ratings.lock().await.get(key).cloned()
    }

    /// Store Kinopoisk rating in cache
    pub async fn set_kinopoisk_rating(&self, key: String, rating: CachedRating) {
        self.kinopoisk_ratings.lock().await.insert(key, rating);
    }

    /// Generate cache key for Kinopoisk
    pub fn kinopoisk_cache_key(kinopoisk_id: Option<u32>, title: &str, year: Option<u32>) -> String {
        if let Some(id) = kinopoisk_id {
            format!("kinopoisk_rating:{}", id)
        } else if let Some(y) = year {
            format!("kinopoisk_rating:{}:{}", title, y)
        } else {
            format!("kinopoisk_rating:{}", title)
        }
    }

    /// Generate cache key for TMDB metadata
    pub fn tmdb_metadata_cache_key(tmdb_id: Option<&str>, title: &str, year: Option<u32>) -> String {
        if let Some(id) = tmdb_id {
            format!("tmdb_metadata:{}", id)
        } else if let Some(y) = year {
            format!("tmdb_metadata:{}:{}", title, y)
        } else {
            format!("tmdb_metadata:{}", title)
        }
    }
}

impl Default for RatingCache {
    fn default() -> Self {
        Self::new()
    }
}

