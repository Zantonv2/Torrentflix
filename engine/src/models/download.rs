use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DownloadStatus {
    pub torrent_info_hash: String,
    pub title: String,
    pub state: DownloadState,
    pub progress: f32,
    pub download_speed: u64,
    pub upload_speed: u64,
    pub seeders: u32,
    pub leechers: u32,
    pub total_size: u64,
    pub downloaded_size: u64,
    pub uploaded_size: u64,
    pub eta_seconds: Option<u64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub file_path: Option<String>, // Final location after completion
    pub temp_path: Option<String>, // Temporary download location
}

impl DownloadStatus {
    pub fn new(torrent_info_hash: String, title: String) -> Self {
        let now = Utc::now();
        Self {
            torrent_info_hash,
            title,
            state: DownloadState::Queued,
            progress: 0.0,
            download_speed: 0,
            upload_speed: 0,
            seeders: 0,
            leechers: 0,
            total_size: 0,
            downloaded_size: 0,
            uploaded_size: 0,
            eta_seconds: None,
            created_at: now,
            updated_at: now,
            started_at: None,
            completed_at: None,
            error_at: None,
            error_message: None,
            file_path: None,
            temp_path: None,
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self.state, DownloadState::Downloading | DownloadState::Paused | DownloadState::Queued)
    }

    pub fn is_completed(&self) -> bool {
        matches!(self.state, DownloadState::Completed | DownloadState::Seeding)
    }

    pub fn is_failed(&self) -> bool {
        matches!(self.state, DownloadState::Failed | DownloadState::Stopped)
    }

    pub fn update_progress(&mut self, progress: f32, downloaded_bytes: u64, total_bytes: u64) {
        self.progress = progress.clamp(0.0, 100.0);
        self.downloaded_size = downloaded_bytes;
        self.total_size = total_bytes;
        self.updated_at = Utc::now();
    }

    pub fn update_speed(&mut self, download_speed: u64, upload_speed: u64) {
        self.download_speed = download_speed;
        self.upload_speed = upload_speed;
        self.updated_at = Utc::now();
    }

    pub fn update_peers(&mut self, seeders: u32, leechers: u32) {
        self.seeders = seeders;
        self.leechers = leechers;
        self.updated_at = Utc::now();
    }

    pub fn set_state(&mut self, state: DownloadState) {
        let old_state = self.state.clone();
        self.state = state.clone();
        
        // Update timestamp based on state change
        if matches!(state, DownloadState::Completed | DownloadState::Seeding) {
            self.completed_at = Some(Utc::now());
        } else if matches!(state, DownloadState::Failed) {
            self.error_at = Some(Utc::now());
        } else if matches!(state, DownloadState::Downloading) && !matches!(old_state, DownloadState::Downloading) {
            self.started_at = Some(Utc::now());
        }
        self.updated_at = Utc::now();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DownloadState {
    Queued,
    Downloading,
    Paused,
    Completed,
    Seeding,
    Failed,
    Stopped,
    Checking, // qBittorrent specific states
    MetaDL,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DownloadConfig {
    pub download_path: String,
    pub temp_path: String,
    pub max_active_downloads: u32,
    pub auto_start: bool,
    pub seed_ratio_limit: Option<f32>,
    pub seed_time_limit: Option<u64>, // in seconds
    pub max_upload_speed: Option<u64>, // bytes per second
    pub max_download_speed: Option<u64>, // bytes per second
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            download_path: "./downloads".to_string(),
            temp_path: "./temp".to_string(),
            max_active_downloads: 3,
            auto_start: true,
            seed_ratio_limit: Some(1.0),
            seed_time_limit: Some(3600), // 1 hour
            max_upload_speed: None,
            max_download_speed: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DownloadEvent {
    pub download_id: Uuid,
    pub event_type: DownloadEventType,
    pub timestamp: DateTime<Utc>,
    pub message: Option<String>,
}

impl DownloadEvent {
    pub fn new(download_id: Uuid, event_type: DownloadEventType) -> Self {
        Self {
            download_id,
            event_type,
            timestamp: Utc::now(),
            message: None,
        }
    }

    pub fn with_message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DownloadEventType {
    Started,
    Paused,
    Resumed,
    Completed,
    Failed,
    Stopped,
    FileMoved,
    FileRenamed,
    Error,
}
