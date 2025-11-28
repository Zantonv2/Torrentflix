use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use tracing::{debug, error, info, warn};

use crate::indexers::traits::{Indexer, SearchQuery, IndexerConfig, SortBy, SortOrder};
use crate::models::{TorrentResult, MediaType};

/// YTS (YIFY) indexer implementation
pub struct YtsIndexer {
    client: Client,
    config: IndexerConfig,
    base_url: String,
}

impl YtsIndexer {
    pub fn new(config: IndexerConfig) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .user_agent(config.user_agent.as_deref().unwrap_or("MovieDownloader/1.0"))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            config,
            base_url: "https://yts.mx/api/v2".to_string(),
        }
    }

    async fn search_movies(&self, query: &str, limit: usize) -> Result<Vec<YtsMovie>> {
        let url = format!("{}/list_movies.json", self.base_url);
        
        let params = [
            ("query_term", query),
            ("limit", &limit.to_string()),
            ("sort_by", "date_added"),
            ("order_by", "desc"),
        ];

        let response = self.client
            .get(&url)
            .query(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("YTS API returned status: {}", response.status()));
        }

        let yts_response: YtsApiResponse = response.json().await?;
        
        if yts_response.status != "ok" {
            return Err(anyhow!("YTS API returned error status: {}", yts_response.status));
        }

        Ok(yts_response.data.movies.unwrap_or_default())
    }

    fn yts_movie_to_torrent_result(&self, movie: &YtsMovie, torrent: &YtsTorrent) -> TorrentResult {
        let title = format!("{} {} {}", 
            movie.title, 
            movie.year.unwrap_or(0), 
            torrent.quality.clone()
        );

        TorrentResult::new(
            title,
            torrent.url.clone(),
            torrent.size_bytes.unwrap_or(0),
            torrent.seeds.unwrap_or(0),
            torrent.peers.unwrap_or(0),
            "YTS".to_string(),
        )
        .with_category("movie".to_string())
    }
}

#[async_trait]
impl Indexer for YtsIndexer {
    fn name(&self) -> &str {
        "YTS"
    }

    fn description(&self) -> &str {
        "YTS (YIFY) movie torrents - high quality movies with small file sizes"
    }

    fn base_url(&self) -> &str {
        &self.base_url
    }

    async fn is_enabled(&self) -> Result<bool> {
        Ok(self.config.enabled)
    }

    async fn search(&self, query: &SearchQuery) -> Result<Vec<TorrentResult>> {
        if !self.config.enabled {
            return Ok(Vec::new());
        }

        // YTS only supports movies
        if let Some(media_type) = &query.media_type {
            if !matches!(media_type, MediaType::Movie) {
                debug!("YTS only supports movies, skipping search for {:?}", media_type);
                return Ok(Vec::new());
            }
        }

        info!("Searching YTS for: {}", query.query);
        
        let limit = query.max_results.unwrap_or(50);
        let movies = self.search_movies(&query.query, limit).await?;

        let mut results = Vec::new();
        
        for movie in movies {
            for torrent in &movie.torrents {
                let result = self.yts_movie_to_torrent_result(&movie, torrent);
                results.push(result);
            }
        }

        info!("YTS returned {} results", results.len());
        Ok(results)
    }

    async fn get_details(&self, _info_hash: &str) -> Result<Option<TorrentResult>> {
        // YTS doesn't support getting details by info hash
        Ok(None)
    }

    async fn test_connection(&self) -> Result<bool> {
        let url = format!("{}/list_movies.json", self.base_url);
        
        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(e) => {
                error!("YTS connection test failed: {}", e);
                Ok(false)
            }
        }
    }

    fn config(&self) -> &IndexerConfig {
        &self.config
    }

    async fn update_config(&mut self, config: IndexerConfig) -> Result<()> {
        self.config = config;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct YtsApiResponse {
    status: String,
    status_message: Option<String>,
    data: YtsApiData,
}

#[derive(Debug, Serialize, Deserialize)]
struct YtsApiData {
    movies: Option<Vec<YtsMovie>>,
    limit: Option<u32>,
    page_number: Option<u32>,
    movies_count: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct YtsMovie {
    id: u32,
    url: String,
    title: String,
    year: Option<u32>,
    rating: Option<f32>,
    runtime: Option<u32>,
    summary: Option<String>,
    synopsis: Option<String>,
    yt_trailer_code: Option<String>,
    language: Option<String>,
    mpa_rating: Option<String>,
    background_image: Option<String>,
    background_image_original: Option<String>,
    small_cover_image: Option<String>,
    medium_cover_image: Option<String>,
    large_cover_image: Option<String>,
    state: Option<String>,
    torrents: Vec<YtsTorrent>,
    date_uploaded: Option<String>,
    date_uploaded_unix: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct YtsTorrent {
    url: String,
    hash: String,
    quality: String,
    size: String,
    size_bytes: Option<u64>,
    date_uploaded: Option<String>,
    date_uploaded_unix: Option<u64>,
    seeds: Option<u32>,
    peers: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_yts_indexer_creation() {
        let config = IndexerConfig::default();
        let indexer = YtsIndexer::new(config);
        
        assert_eq!(indexer.name(), "YTS");
        assert!(indexer.description().contains("YTS"));
        assert_eq!(indexer.base_url(), "https://yts.mx/api/v2");
    }

    #[tokio::test]
    async fn test_yts_search() {
        let config = IndexerConfig::default();
        let indexer = YtsIndexer::new(config);
        
        let query = SearchQuery::new("inception".to_string())
            .with_media_type(MediaType::Movie)
            .with_max_results(5);
        
        let results = indexer.search(&query).await.unwrap();
        
        // YTS should return some results for "inception"
        assert!(!results.is_empty());
        
        for result in results {
            assert_eq!(result.source, "YTS");
            assert!(result.title.contains("inception") || result.title.contains("Inception"));
            assert!(result.magnet_link.starts_with("magnet:"));
        }
    }

    #[tokio::test]
    async fn test_yts_connection() {
        let config = IndexerConfig::default();
        let indexer = YtsIndexer::new(config);
        
        let is_connected = indexer.test_connection().await.unwrap();
        assert!(is_connected, "YTS should be accessible");
    }
}
