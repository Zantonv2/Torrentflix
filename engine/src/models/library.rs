use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use super::{ParsedMedia, MediaType, EnrichedMedia};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LibraryItem {
    pub id: Uuid,
    pub file_path: String,
    pub parsed_media: ParsedMedia,
    pub enriched_media: Option<EnrichedMedia>,
    pub file_size_bytes: u64,
    pub file_hash: Option<String>, // SHA-256 or similar
    pub added_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_watched_at: Option<DateTime<Utc>>,
    pub watch_count: u32,
    pub user_rating: Option<f32>, // 1.0 - 10.0
    pub tags: Vec<String>,
    pub is_favorite: bool,
    pub is_archived: bool,
    pub quality_info: QualityInfo,
}

impl LibraryItem {
    pub fn new(file_path: String, parsed_media: ParsedMedia, file_size_bytes: u64) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            file_path,
            parsed_media,
            enriched_media: None,
            file_size_bytes,
            file_hash: None,
            added_at: now,
            updated_at: now,
            last_watched_at: None,
            watch_count: 0,
            user_rating: None,
            tags: Vec::new(),
            is_favorite: false,
            is_archived: false,
            quality_info: QualityInfo::default(),
        }
    }

    pub fn identity_key(&self) -> String {
        self.parsed_media.identity_key()
    }

    pub fn display_title(&self) -> String {
        if let Some(enriched) = &self.enriched_media {
            enriched.display_title()
        } else if let Some(year) = self.parsed_media.year {
            format!("{} ({})", self.parsed_media.title, year)
        } else {
            self.parsed_media.title.clone()
        }
    }

    pub fn mark_watched(&mut self) {
        self.last_watched_at = Some(Utc::now());
        self.watch_count += 1;
        self.updated_at = Utc::now();
    }

    pub fn set_rating(&mut self, rating: f32) {
        self.user_rating = Some(rating.clamp(1.0, 10.0));
        self.updated_at = Utc::now();
    }

    pub fn toggle_favorite(&mut self) {
        self.is_favorite = !self.is_favorite;
        self.updated_at = Utc::now();
    }

    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.updated_at = Utc::now();
        }
    }

    pub fn remove_tag(&mut self, tag: &str) {
        if let Some(pos) = self.tags.iter().position(|t| t == tag) {
            self.tags.remove(pos);
            self.updated_at = Utc::now();
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QualityInfo {
    pub resolution: Option<String>,
    pub source: Option<String>,
    pub codec: Option<String>,
    pub audio: Option<String>,
    pub bitrate: Option<u64>,
    pub is_hdr: bool,
    pub is_dolby_vision: bool,
    pub is_dolby_atmos: bool,
}

impl QualityInfo {
    pub fn new() -> Self {
        Self {
            resolution: None,
            source: None,
            codec: None,
            audio: None,
            bitrate: None,
            is_hdr: false,
            is_dolby_vision: false,
            is_dolby_atmos: false,
        }
    }

    pub fn quality_score(&self) -> f32 {
        let mut score = 0.0;
        
        // Resolution scoring
        if let Some(res) = &self.resolution {
            score += match res.as_str() {
                "4320p" | "8K" => 100.0,
                "2160p" | "4K" => 80.0,
                "1440p" | "2K" => 60.0,
                "1080p" => 40.0,
                "720p" => 20.0,
                "480p" => 10.0,
                _ => 5.0,
            };
        }
        
        // Source scoring
        if let Some(src) = &self.source {
            score += match src.to_lowercase().as_str() {
                "uhd" | "uhd bluray" => 20.0,
                "bluray" | "bd" => 15.0,
                "web-dl" | "webrip" => 12.0,
                "hdtv" => 8.0,
                "dvd" => 5.0,
                "cam" | "ts" => 1.0,
                _ => 3.0,
            };
        }
        
        // HDR bonus
        if self.is_hdr {
            score += 10.0;
        }
        
        // Dolby Vision bonus
        if self.is_dolby_vision {
            score += 8.0;
        }
        
        // Dolby Atmos bonus
        if self.is_dolby_atmos {
            score += 5.0;
        }
        
        score
    }
}

impl Default for QualityInfo {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LibraryStats {
    pub total_items: u64,
    pub total_size_bytes: u64,
    pub movies_count: u64,
    pub series_count: u64,
    pub episodes_count: u64,
    pub recent_additions: Vec<LibraryItem>,
    pub most_watched: Vec<LibraryItem>,
    pub favorites: Vec<LibraryItem>,
}

impl LibraryStats {
    pub fn new() -> Self {
        Self {
            total_items: 0,
            total_size_bytes: 0,
            movies_count: 0,
            series_count: 0,
            episodes_count: 0,
            recent_additions: Vec::new(),
            most_watched: Vec::new(),
            favorites: Vec::new(),
        }
    }
}

impl Default for LibraryStats {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WatchlistItem {
    pub id: Uuid,
    pub tmdb_id: Option<u32>,
    pub imdb_id: Option<String>,
    pub title: String,
    pub media_type: MediaType,
    pub year: Option<u32>,
    pub added_at: DateTime<Utc>,
    pub notify_on_new: bool,
    pub quality_preference: Option<QualityPreference>,
    pub last_checked: Option<DateTime<Utc>>,
    pub is_complete: bool, // For series - if all episodes are downloaded
}

impl WatchlistItem {
    pub fn new(title: String, media_type: MediaType) -> Self {
        Self {
            id: Uuid::new_v4(),
            tmdb_id: None,
            imdb_id: None,
            title,
            media_type,
            year: None,
            added_at: Utc::now(),
            notify_on_new: true,
            quality_preference: None,
            last_checked: None,
            is_complete: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QualityPreference {
    pub min_resolution: Option<String>,
    pub preferred_sources: Vec<String>,
    pub preferred_codecs: Vec<String>,
    pub max_size_gb: Option<f32>,
    pub require_hdr: bool,
}
