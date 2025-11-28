use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TorrentResult {
    pub title: String,
    pub magnet_link: String,
    pub info_hash: Option<String>,
    pub size_bytes: u64,
    pub seeders: u32,
    pub leechers: u32,
    pub source: String, // Name of the indexer
    pub upload_date: Option<DateTime<Utc>>,
    pub category: Option<String>, // movie, series, etc.
    pub tracker: Option<String>,
    pub poster_url: Option<String>, // URL to movie poster image
}

impl TorrentResult {
    pub fn new(
        title: String,
        magnet_link: String,
        size_bytes: u64,
        seeders: u32,
        leechers: u32,
        source: String,
    ) -> Self {
        Self {
            title,
            magnet_link,
            info_hash: None,
            size_bytes,
            seeders,
            leechers,
            source,
            upload_date: None,
            category: None,
            tracker: None,
            poster_url: None,
        }
    }

    pub fn with_info_hash(mut self, info_hash: String) -> Self {
        self.info_hash = Some(info_hash);
        self
    }

    pub fn with_upload_date(mut self, upload_date: DateTime<Utc>) -> Self {
        self.upload_date = Some(upload_date);
        self
    }

    pub fn with_category(mut self, category: String) -> Self {
        self.category = Some(category);
        self
    }

    pub fn with_tracker(mut self, tracker: String) -> Self {
        self.tracker = Some(tracker);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TorrentQuality {
    pub resolution: Option<String>, // 720p, 1080p, 2160p, etc.
    pub source: Option<String>,     // BluRay, WEB-DL, HDRip, etc.
    pub codec: Option<String>,      // H.264, H.265, x264, x265, etc.
    pub audio: Option<String>,      // AC3, DTS, AAC, etc.
    pub release_group: Option<String>,
}

impl TorrentQuality {
    pub fn new() -> Self {
        Self {
            resolution: None,
            source: None,
            codec: None,
            audio: None,
            release_group: None,
        }
    }
}

impl Default for TorrentQuality {
    fn default() -> Self {
        Self::new()
    }
}
