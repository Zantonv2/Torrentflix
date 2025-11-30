use serde::{Deserialize, Serialize};

/// Minimal UI DTO for search results (Netflix-style card)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSearchResult {
    pub id: String,
    pub title: String,
    pub year: Option<u32>,
    pub poster_url: Option<String>,
    pub backdrop_url: Option<String>,
    pub quality_badge: String, // "1080p", "4K", etc.
    pub rating: Option<f32>, // Primary rating (fallback: TMDB -> IMDb -> Kinopoisk)
    pub rating_kinopoisk: Option<f32>,
    pub rating_imdb: Option<f32>,
    pub runtime_minutes: Option<u32>,
    pub category: Option<String>,
    pub description: Option<String>,
    pub cast: Vec<String>,
    pub genres: Vec<String>,
    pub torrent_info: UiTorrentInfo,
}

/// Minimal torrent information for UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiTorrentInfo {
    pub seeders: u32,
    pub size_gb: f32,
    pub magnet_link: String,
}

/// Search request from UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSearchRequest {
    pub query: String,
    pub media_type: String, // "movie", "series", "all"
    pub limit: Option<u32>,
}

/// Search response to UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSearchResponse {
    pub results: Vec<UiSearchResult>,
    pub total: u32,
    pub took_ms: u64,
}

impl UiSearchResult {
    pub fn from_media_search_result(result: crate::models::MediaSearchResult) -> Option<Self> {
        let best_quality = result.best_quality?;
        
        // Extract quality from normalized torrent info
        let quality_badge = result.enriched.parsed.resolution.clone()
            .unwrap_or_else(|| "Unknown".to_string());
        
        Some(Self {
            id: result.enriched.parsed.identity_key(),
            title: result.enriched.parsed.title.clone(),
            year: result.enriched.parsed.year,
            poster_url: result.enriched.poster_url.clone(),
            backdrop_url: result.enriched.backdrop_url.clone(),
            quality_badge,
            rating: result.enriched.primary_rating(),
            rating_kinopoisk: result.enriched.rating_kinopoisk,
            rating_imdb: result.enriched.rating_imdb,
            runtime_minutes: result.enriched.runtime_minutes,
            category: best_quality.category.clone(),
            description: result.enriched.overview.clone(),
            cast: result.enriched.cast.clone(),
            genres: result.enriched.genres.clone(),
            torrent_info: UiTorrentInfo {
                seeders: best_quality.seeders,
                size_gb: best_quality.size_bytes as f32 / (1024.0 * 1024.0 * 1024.0),
                magnet_link: best_quality.magnet_link.clone(),
            },
        })
    }
    
    pub fn display_title(&self) -> String {
        if let Some(year) = self.year {
            format!("{} ({})", self.title, year)
        } else {
            self.title.clone()
        }
    }
    
    pub fn display_size(&self) -> String {
        if self.torrent_info.size_gb < 1.0 {
            format!("{:.0} MB", self.torrent_info.size_gb * 1024.0)
        } else {
            format!("{:.1} GB", self.torrent_info.size_gb)
        }
    }
}
