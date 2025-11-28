// Placeholder download tracker
// TODO: Implement actual download status tracking

use crate::models::DownloadStatus;
use std::collections::HashMap;

pub struct DownloadTracker {
    downloads: HashMap<String, DownloadStatus>,
}

impl DownloadTracker {
    pub fn new() -> Self {
        Self {
            downloads: HashMap::new(),
        }
    }

    pub async fn track_download(&mut self, status: DownloadStatus) {
        self.downloads.insert(status.torrent_info_hash.clone(), status);
    }

    pub async fn get_status(&self, hash: &str) -> Option<&DownloadStatus> {
        self.downloads.get(hash)
    }

    pub async fn get_all_downloads(&self) -> Vec<&DownloadStatus> {
        self.downloads.values().collect()
    }
}

impl Default for DownloadTracker {
    fn default() -> Self {
        Self::new()
    }
}
