// Placeholder torrent controller
// TODO: Implement actual torrent control logic

use anyhow::Result;
use crate::models::TorrentResult;

pub struct TorrentController {
    client: super::client::TorrentClient,
}

impl TorrentController {
    pub fn new(client: super::client::TorrentClient) -> Self {
        Self { client }
    }

    pub async fn start_download(&self, torrent: &TorrentResult) -> Result<String> {
        self.client.add_torrent(&torrent.magnet_link).await
    }

    pub async fn pause_download(&self, hash: &str) -> Result<()> {
        self.client.pause_download(hash).await
    }

    pub async fn resume_download(&self, hash: &str) -> Result<()> {
        self.client.resume_download(hash).await
    }

    pub async fn remove_download(&self, hash: &str, delete_files: bool) -> Result<()> {
        self.client.remove_download(hash, delete_files).await
    }
}
