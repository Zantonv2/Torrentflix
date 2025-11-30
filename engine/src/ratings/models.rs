use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Result from a rating worker
#[derive(Debug, Clone)]
pub struct RatingWorkerResult {
    pub movie_id: String,  // Identity key from ParsedMedia
    pub rating: Option<f32>,
    pub source: RatingSource,
    pub fetched_at: DateTime<Utc>,
}

/// Source of the rating
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RatingSource {
    Kinopoisk,
    Imdb,
}

/// Result from TMDB Worker 1 (metadata fetcher)
#[derive(Debug, Clone)]
pub struct TMDBMetadataResult {
    pub movie_id: String,
    pub tmdb_id: Option<String>,
    pub imdb_id: Option<String>,
    pub episodes: Option<u32>,
    pub seasons: Option<u32>,
    pub fetched_at: DateTime<Utc>,
}

/// Cached TMDB metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedTMDBMetadata {
    pub imdb_id: Option<String>,
    pub episodes: Option<u32>,
    pub seasons: Option<u32>,
    pub cached_at: DateTime<Utc>,
}

/// Cached rating
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedRating {
    pub rating: Option<f32>,
    pub cached_at: DateTime<Utc>,
}

