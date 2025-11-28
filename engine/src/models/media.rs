use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParsedMedia {
    pub title: String,
    pub year: Option<u32>,
    pub season: Option<u32>,
    pub episode: Option<u32>,
    pub resolution: Option<String>,
    pub source: Option<String>,
    pub codec: Option<String>,
    pub audio: Option<String>,
    pub release_group: Option<String>,
    pub media_type: MediaType,
    pub language: Option<String>,
    pub subtitles: Option<Vec<String>>,
}

impl ParsedMedia {
    pub fn new(title: String, media_type: MediaType) -> Self {
        Self {
            title,
            year: None,
            season: None,
            episode: None,
            resolution: None,
            source: None,
            codec: None,
            audio: None,
            release_group: None,
            media_type,
            language: None,
            subtitles: None,
        }
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

    pub fn identity_key(&self) -> String {
        match self.media_type {
            MediaType::Movie => {
                format!("movie:{}:{}", self.title, self.year.unwrap_or(0))
            }
            MediaType::Series => {
                if let (Some(season), Some(episode)) = (self.season, self.episode) {
                    format!("series:{}:S{:02}E{:02}", self.title, season, episode)
                } else {
                    format!("series:{}", self.title)
                }
            }
            MediaType::Documentary | MediaType::Anime | MediaType::Other => {
                format!("{}:{}:{}", self.media_type, self.title, self.year.unwrap_or(0))
            }
        }
    }

    pub fn release_signature(&self) -> String {
        let mut parts = Vec::new();
        
        if let Some(resolution) = &self.resolution {
            parts.push(resolution.clone());
        }
        if let Some(source) = &self.source {
            parts.push(source.clone());
        }
        if let Some(codec) = &self.codec {
            parts.push(codec.clone());
        }
        if let Some(audio) = &self.audio {
            parts.push(audio.clone());
        }
        
        parts.join(".")
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MediaType {
    Movie,
    Series,
    Documentary,
    Anime,
    Other,
}

impl std::fmt::Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaType::Movie => write!(f, "movie"),
            MediaType::Series => write!(f, "series"),
            MediaType::Documentary => write!(f, "documentary"),
            MediaType::Anime => write!(f, "anime"),
            MediaType::Other => write!(f, "other"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnrichedMedia {
    pub parsed: ParsedMedia,
    pub tmdb_id: Option<u32>,
    pub imdb_id: Option<String>,
    pub kinopoisk_id: Option<u32>,
    pub overview: Option<String>,
    pub poster_url: Option<String>,
    pub backdrop_url: Option<String>,
    pub runtime_minutes: Option<u32>,
    pub genres: Vec<String>,
    pub cast: Vec<String>,
    pub director: Option<String>,
    pub rating_imdb: Option<f32>,
    pub rating_tmdb: Option<f32>,
    pub rating_kinopoisk: Option<f32>,
    pub release_date: Option<DateTime<Utc>>,
    pub external_urls: HashMap<String, String>, // Key: source, Value: URL
}

impl EnrichedMedia {
    pub fn new(parsed: ParsedMedia) -> Self {
        Self {
            parsed,
            tmdb_id: None,
            imdb_id: None,
            kinopoisk_id: None,
            overview: None,
            poster_url: None,
            backdrop_url: None,
            runtime_minutes: None,
            genres: Vec::new(),
            cast: Vec::new(),
            director: None,
            rating_imdb: None,
            rating_tmdb: None,
            rating_kinopoisk: None,
            release_date: None,
            external_urls: HashMap::new(),
        }
    }

    pub fn primary_rating(&self) -> Option<f32> {
        self.rating_tmdb
            .or(self.rating_imdb)
            .or(self.rating_kinopoisk)
    }

    pub fn display_title(&self) -> String {
        if let Some(year) = self.parsed.year {
            format!("{} ({})", self.parsed.title, year)
        } else {
            self.parsed.title.clone()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MediaSearchResult {
    pub enriched: EnrichedMedia,
    pub torrent_results: Vec<TorrentResult>,
    pub best_quality: Option<TorrentResult>,
    pub alternative_qualities: Vec<TorrentResult>,
    pub score: f32,
}

impl MediaSearchResult {
    pub fn new(enriched: EnrichedMedia) -> Self {
        Self {
            enriched,
            torrent_results: Vec::new(),
            best_quality: None,
            alternative_qualities: Vec::new(),
            score: 0.0,
        }
    }
}

// Re-export TorrentResult for convenience
pub use super::torrent::TorrentResult;
