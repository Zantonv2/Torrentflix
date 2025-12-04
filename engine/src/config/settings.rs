use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Library configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryConfig {
    /// Grace period for trashed files before permanent deletion (in seconds)
    pub trash_grace_period_secs: u64,
    
    /// Maximum number of concurrent hashing operations
    pub hashing_concurrency_limit: u32,
    
    /// Batch size for rescan operations
    pub rescan_batch_size: u32,
    
    /// Disk usage threshold for cleanup warnings (in bytes)
    pub cleanup_threshold_bytes: u64,
    
    /// Quality preference policy for version selection
    pub quality_preference: QualityPreference,
    
    /// Version retention policy
    pub version_retention: VersionRetention,
}

/// Quality preference policy for automatic version selection
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum QualityPreference {
    /// Prefer highest resolution
    HighestResolution,
    /// Prefer smallest file size
    SmallestSize,
    /// Prefer specific codec (H.265 > H.264)
    PreferCodec,
    /// Balanced approach
    Balanced,
}

/// Version retention policy
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum VersionRetention {
    /// Keep only the best version
    BestOnly,
    /// Keep best plus one backup
    BestPlusBakup,
    /// Keep all versions
    KeepAll,
}

impl LibraryConfig {
    /// Create default library configuration optimized for HDD storage
    pub fn new() -> Self {
        Self {
            trash_grace_period_secs: 30 * 24 * 60 * 60, // 30 days
            hashing_concurrency_limit: 1,
            rescan_batch_size: 100,
            cleanup_threshold_bytes: 10 * 1024 * 1024 * 1024, // 10 GB
            quality_preference: QualityPreference::Balanced,
            version_retention: VersionRetention::BestPlusBakup,
        }
    }

    /// Get trash grace period as Duration
    pub fn trash_grace_period(&self) -> Duration {
        Duration::from_secs(self.trash_grace_period_secs)
    }
}

impl Default for LibraryConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Application settings (legacy, kept for compatibility)
pub struct AppSettings {
    pub download_path: String,
    pub max_concurrent_downloads: u32,
    pub enabled_indexers: Vec<String>,
}

impl AppSettings {
    pub fn new() -> Self {
        Self {
            download_path: "./downloads".to_string(),
            max_concurrent_downloads: 3,
            enabled_indexers: vec!["monna".to_string()],
        }
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self::new()
    }
}
