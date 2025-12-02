// qBittorrent WebUI API client
// API Reference: https://github.com/qbittorrent/qBittorrent/wiki/WebUI-API-(qBittorrent-4.1)

use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn, debug};

use crate::models::DownloadStatus;

pub struct TorrentClient {
    base_url: String,
    username: String,
    password: String,
    client: Client,
    cookie: Arc<Mutex<Option<String>>>,
}

#[derive(Debug, Deserialize)]
struct QBTorrentInfo {
    hash: String,
    name: String,
    progress: f32,
    dlspeed: u64,
    eta: i64,
    num_seeds: i32,
    num_leechs: i32,
    state: String,
    size: u64,
}

impl TorrentClient {
    pub fn new(url: String, username: String, password: String) -> Self {
        Self {
            base_url: url.trim_end_matches('/').to_string(),
            username,
            password,
            client: Client::builder()
                .cookie_store(true)
                .build()
                .expect("Failed to create HTTP client"),
            cookie: Arc::new(Mutex::new(None)),
        }
    }

    /// Login to qBittorrent WebUI
    async fn ensure_logged_in(&self) -> Result<()> {
        let cookie = self.cookie.lock().await;
        if cookie.is_some() {
            return Ok(());
        }
        drop(cookie);

        info!("Logging in to qBittorrent at {}", self.base_url);
        
        let response = self.client
            .post(&format!("{}/api/v2/auth/login", self.base_url))
            .form(&[
                ("username", self.username.as_str()),
                ("password", self.password.as_str()),
            ])
            .send()
            .await?;

        if response.status().is_success() {
            let text = response.text().await?;
            if text == "Ok." {
                info!("Successfully logged in to qBittorrent");
                Ok(())
            } else {
                Err(anyhow!("Login failed: {}", text))
            }
        } else {
            Err(anyhow!("Login failed with status: {}", response.status()))
        }
    }

    /// Add torrent via magnet link
    pub async fn add_torrent(&self, magnet_url: &str) -> Result<String> {
        self.ensure_logged_in().await?;

        info!("Adding torrent: {}", &magnet_url[..60.min(magnet_url.len())]);

        let response = self.client
            .post(&format!("{}/api/v2/torrents/add", self.base_url))
            .form(&[("urls", magnet_url)])
            .send()
            .await?;

        if response.status().is_success() {
            let text = response.text().await?;
            if text == "Ok." {
                // Extract hash from magnet link
                let hash = extract_hash_from_magnet(magnet_url)?;
                info!("Torrent added successfully: {}", hash);
                Ok(hash)
            } else {
                Err(anyhow!("Failed to add torrent: {}", text))
            }
        } else {
            Err(anyhow!("Failed to add torrent: {}", response.status()))
        }
    }

    /// Get download status for a specific torrent
    pub async fn get_download_status(&self, hash: &str) -> Result<DownloadStatus> {
        self.ensure_logged_in().await?;

        debug!("Getting status for torrent: {}", hash);

        let response = self.client
            .get(&format!("{}/api/v2/torrents/info", self.base_url))
            .query(&[("hashes", hash)])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to get torrent info: {}", response.status()));
        }

        let torrents: Vec<QBTorrentInfo> = response.json().await?;
        
        let torrent = torrents.into_iter().next()
            .ok_or_else(|| anyhow!("Torrent not found: {}", hash))?;

        Ok(DownloadStatus {
            hash: torrent.hash,
            title: torrent.name,
            progress: torrent.progress,
            speed: torrent.dlspeed,
            eta: torrent.eta,
            seeders: torrent.num_seeds,
            leechers: torrent.num_leechs,
            state: torrent.state,
            size: torrent.size,
        })
    }

    /// Get all active downloads
    pub async fn get_all_downloads(&self) -> Result<Vec<DownloadStatus>> {
        self.ensure_logged_in().await?;

        debug!("Getting all torrents");

        let response = self.client
            .get(&format!("{}/api/v2/torrents/info", self.base_url))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to get torrents: {}", response.status()));
        }

        let torrents: Vec<QBTorrentInfo> = response.json().await?;
        
        Ok(torrents.into_iter().map(|t| DownloadStatus {
            hash: t.hash,
            title: t.name,
            progress: t.progress,
            speed: t.dlspeed,
            eta: t.eta,
            seeders: t.num_seeds,
            leechers: t.num_leechs,
            state: t.state,
            size: t.size,
        }).collect())
    }

    /// Pause a download
    pub async fn pause_download(&self, hash: &str) -> Result<()> {
        self.ensure_logged_in().await?;

        info!("Pausing torrent: {}", hash);

        let response = self.client
            .get(&format!("{}/api/v2/torrents/pause", self.base_url))
            .query(&[("hashes", hash)])
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow!("Failed to pause torrent: {}", response.status()))
        }
    }

    /// Resume a download
    pub async fn resume_download(&self, hash: &str) -> Result<()> {
        self.ensure_logged_in().await?;

        info!("Resuming torrent: {}", hash);

        let response = self.client
            .get(&format!("{}/api/v2/torrents/resume", self.base_url))
            .query(&[("hashes", hash)])
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow!("Failed to resume torrent: {}", response.status()))
        }
    }

    /// Remove a download
    pub async fn remove_download(&self, hash: &str, delete_files: bool) -> Result<()> {
        self.ensure_logged_in().await?;

        info!("Removing torrent: {} (delete_files: {})", hash, delete_files);

        let endpoint = if delete_files {
            "torrents/delete"
        } else {
            "torrents/delete"
        };

        let response = self.client
            .post(&format!("{}/api/v2/{}", self.base_url, endpoint))
            .form(&[
                ("hashes", hash),
                ("deleteFiles", if delete_files { "true" } else { "false" }),
            ])
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow!("Failed to remove torrent: {}", response.status()))
        }
    }

    /// Test connection to qBittorrent
    pub async fn test_connection(&self) -> Result<()> {
        self.ensure_logged_in().await?;
        
        // Try to get API version
        let response = self.client
            .get(&format!("{}/api/v2/app/version", self.base_url))
            .send()
            .await?;

        if response.status().is_success() {
            let version = response.text().await?;
            info!("Connected to qBittorrent version: {}", version);
            Ok(())
        } else {
            Err(anyhow!("Connection test failed: {}", response.status()))
        }
    }
}

/// Extract hash from magnet link
fn extract_hash_from_magnet(magnet: &str) -> Result<String> {
    // Magnet format: magnet:?xt=urn:btih:HASH&...
    let hash = magnet
        .split("btih:")
        .nth(1)
        .and_then(|s| s.split('&').next())
        .ok_or_else(|| anyhow!("Invalid magnet link: no btih found"))?;
    
    Ok(hash.to_lowercase())
}
