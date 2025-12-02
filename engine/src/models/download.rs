use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DownloadStatus {
    pub hash: String,
    pub title: String,
    pub progress: f32,
    pub speed: u64,
    pub eta: i64,
    pub seeders: i32,
    pub leechers: i32,
    pub state: String,
    pub size: u64,
}

impl DownloadStatus {
    pub fn new(hash: String, title: String) -> Self {
        Self {
            hash,
            title,
            progress: 0.0,
            speed: 0,
            eta: -1,
            seeders: 0,
            leechers: 0,
            state: "queued".to_string(),
            size: 0,
        }
    }

    pub fn is_active(&self) -> bool {
        !matches!(self.state.as_str(), "pausedDL" | "error" | "missingFiles")
    }

    pub fn is_completed(&self) -> bool {
        matches!(self.state.as_str(), "uploading" | "pausedUP" | "queuedUP" | "stalledUP" | "checkingUP" | "forcedUP")
    }

    pub fn is_downloading(&self) -> bool {
        matches!(self.state.as_str(), "downloading" | "metaDL" | "allocating" | "stalledDL" | "queuedDL" | "checkingDL" | "forcedDL")
    }
}

