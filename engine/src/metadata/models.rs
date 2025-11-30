use serde::{Deserialize, Serialize};

/// Standardized metadata model from various sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaMetadata {
    pub id: String,
    pub title: String,
    pub original_title: Option<String>,
    pub overview: Option<String>,
    pub media_type: MediaType,
    pub year: Option<u32>,
    pub release_date: Option<String>,
    pub runtime: Option<u32>,
    pub poster_url: Option<String>,
    pub backdrop_url: Option<String>,
    pub genres: Vec<String>,
    pub vote_average: Option<f32>,
    pub vote_count: Option<u32>,
    pub imdb_id: Option<String>,
    pub external_ids: ExternalIds,
    pub number_of_seasons: Option<u32>,
    pub number_of_episodes: Option<u32>,
}

/// External IDs from various sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalIds {
    pub tmdb_id: Option<String>,
    pub imdb_id: Option<String>,
    pub kinopoisk_id: Option<String>,
}

/// Media type for metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MediaType {
    Movie,
    Series,
    Episode,
}

/// Search result from metadata provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataSearchResult {
    pub id: String,
    pub title: String,
    pub year: Option<u32>,
    pub media_type: MediaType,
    pub poster_url: Option<String>,
    pub overview: Option<String>,
}

impl MediaMetadata {
    pub fn new(id: String, title: String, media_type: MediaType) -> Self {
        Self {
            id,
            title,
            original_title: None,
            overview: None,
            media_type,
            year: None,
            release_date: None,
            runtime: None,
            poster_url: None,
            backdrop_url: None,
            genres: Vec::new(),
            vote_average: None,
            vote_count: None,
            imdb_id: None,
            external_ids: ExternalIds {
                tmdb_id: None,
                imdb_id: None,
                kinopoisk_id: None,
            },
            number_of_seasons: None,
            number_of_episodes: None,
        }
    }
}
