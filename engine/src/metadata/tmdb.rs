// TMDb API client using direct reqwest HTTP calls

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

use super::traits::MetadataProvider;
use super::models::{MediaMetadata, MetadataSearchResult, MediaType, ExternalIds};

// TMDB API response models
#[derive(Debug, Deserialize)]
struct TmdbConfiguration {
    images: Option<TmdbImagesConfig>,
}

#[derive(Debug, Deserialize)]
struct TmdbImagesConfig {
    #[serde(rename = "secure_base_url")]
    secure_base_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TmdbSearchResponse<T> {
    page: Option<i32>,
    results: Option<Vec<T>>,
    total_pages: Option<i32>,
    total_results: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct TmdbMovieObject {
    id: Option<i32>,
    title: Option<String>,
    #[serde(rename = "original_title")]
    original_title: Option<String>,
    overview: Option<String>,
    #[serde(rename = "release_date")]
    release_date: Option<String>,
    #[serde(rename = "poster_path")]
    poster_path: Option<String>,
    #[serde(rename = "backdrop_path")]
    backdrop_path: Option<String>,
    #[serde(rename = "vote_average")]
    vote_average: Option<f64>,
    #[serde(rename = "vote_count")]
    vote_count: Option<i32>,
    popularity: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct TmdbTvObject {
    id: Option<i32>,
    name: Option<String>,
    #[serde(rename = "original_name")]
    original_name: Option<String>,
    overview: Option<String>,
    #[serde(rename = "first_air_date")]
    first_air_date: Option<String>,
    #[serde(rename = "poster_path")]
    poster_path: Option<String>,
    #[serde(rename = "backdrop_path")]
    backdrop_path: Option<String>,
    #[serde(rename = "vote_average")]
    vote_average: Option<f64>,
    #[serde(rename = "vote_count")]
    vote_count: Option<i32>,
    popularity: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct TmdbMovieDetails {
    id: Option<i32>,
    title: Option<String>,
    #[serde(rename = "original_title")]
    original_title: Option<String>,
    overview: Option<String>,
    #[serde(rename = "release_date")]
    release_date: Option<String>,
    runtime: Option<i32>,
    genres: Option<Vec<TmdbGenre>>,
    #[serde(rename = "poster_path")]
    poster_path: Option<String>,
    #[serde(rename = "backdrop_path")]
    backdrop_path: Option<String>,
    #[serde(rename = "vote_average")]
    vote_average: Option<f64>,
    #[serde(rename = "vote_count")]
    vote_count: Option<i32>,
    #[serde(rename = "external_ids")]
    external_ids: Option<TmdbExternalIds>,
}

#[derive(Debug, Deserialize)]
struct TmdbTvDetails {
    id: Option<i32>,
    name: Option<String>,
    #[serde(rename = "original_name")]
    original_name: Option<String>,
    overview: Option<String>,
    #[serde(rename = "first_air_date")]
    first_air_date: Option<String>,
    #[serde(rename = "episode_run_time")]
    episode_run_time: Option<Vec<i32>>,
    genres: Option<Vec<TmdbGenre>>,
    #[serde(rename = "poster_path")]
    poster_path: Option<String>,
    #[serde(rename = "backdrop_path")]
    backdrop_path: Option<String>,
    #[serde(rename = "vote_average")]
    vote_average: Option<f64>,
    #[serde(rename = "vote_count")]
    vote_count: Option<i32>,
    #[serde(rename = "external_ids")]
    external_ids: Option<TmdbExternalIds>,
}

#[derive(Debug, Deserialize)]
struct TmdbGenre {
    id: Option<i32>,
    name: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct TmdbExternalIds {
    #[serde(rename = "imdb_id")]
    imdb_id: Option<String>,
    #[serde(rename = "tvdb_id")]
    tvdb_id: Option<i32>,
    #[serde(rename = "facebook_id")]
    facebook_id: Option<String>,
    #[serde(rename = "instagram_id")]
    instagram_id: Option<String>,
    #[serde(rename = "twitter_id")]
    twitter_id: Option<String>,
}

/// TMDb API client for movie and TV show metadata
pub struct TmdbClient {
    api_key: Option<String>,
    client: Client,
    image_base_url: Arc<Mutex<Option<String>>>,
}

impl TmdbClient {
    pub fn new(api_key: Option<String>) -> Self {
        let mut client_builder = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36");
        
        // Configure SOCKS5 proxy if available (using socks5h for DNS resolution through proxy)
        if let Ok(proxy_url) = std::env::var("ADDRESS_PORT") {
            let socks5h_url = format!("socks5h://{}", proxy_url);
            if let Ok(proxy) = reqwest::Proxy::all(&socks5h_url) {
                client_builder = client_builder.proxy(proxy);
            }
        }
        
        let client = client_builder
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            api_key,
            client,
            image_base_url: Arc::new(Mutex::new(None)),
        }
    }

    /// Initialize image base URL from TMDb configuration
    async fn init_image_config(&self) -> Result<()> {
        let mut base_url = self.image_base_url.lock().await;
        if base_url.is_some() {
            return Ok(());
        }

        let api_key = match &self.api_key {
            Some(key) => key.clone(),
            None => {
                *base_url = Some("https://image.tmdb.org/t/p".to_string());
                return Ok(());
            }
        };

        let url = "https://api.themoviedb.org/3/configuration";
        let response = self.client
            .get(url)
            .query(&[("api_key", &api_key)])
            .send()
            .await?;

        if !response.status().is_success() {
            *base_url = Some("https://image.tmdb.org/t/p".to_string());
            return Ok(());
        }

        let config: TmdbConfiguration = response.json().await?;
        if let Some(images) = config.images {
            if let Some(secure_url) = images.secure_base_url {
                *base_url = Some(secure_url);
                debug!("TMDb image base URL configured: {}", base_url.as_ref().unwrap());
            } else {
                *base_url = Some("https://image.tmdb.org/t/p".to_string());
            }
        } else {
            *base_url = Some("https://image.tmdb.org/t/p".to_string());
        }

        Ok(())
    }

    fn extract_year_from_date(date_str: &str) -> Option<u32> {
        date_str.split('-').next()?.parse().ok()
    }

    fn convert_movie_to_metadata(
        movie: TmdbMovieDetails,
        external_ids: Option<TmdbExternalIds>,
        image_base_url: &str,
    ) -> MediaMetadata {
        let id = movie.id.map(|id| id.to_string()).unwrap_or_default();
        let title = movie.title.unwrap_or_default();
        
        let mut metadata = MediaMetadata::new(id.clone(), title, MediaType::Movie);
        metadata.original_title = movie.original_title;
        metadata.overview = movie.overview;
        metadata.year = movie.release_date.as_ref().and_then(|d| Self::extract_year_from_date(d));
        metadata.release_date = movie.release_date;
        metadata.runtime = movie.runtime.map(|r| r as u32);
        metadata.genres = movie.genres
            .unwrap_or_default()
            .into_iter()
            .filter_map(|g| g.name)
            .collect();
        metadata.vote_average = movie.vote_average.map(|v| v as f32);
        metadata.vote_count = movie.vote_count.map(|v| v as u32);
        
        // Set poster and backdrop URLs
        if let Some(poster_path) = movie.poster_path {
            metadata.poster_url = Some(format!("{}/w500{}", image_base_url, poster_path));
        }
        if let Some(backdrop_path) = movie.backdrop_path {
            metadata.backdrop_url = Some(format!("{}/w1280{}", image_base_url, backdrop_path));
        }

        // Extract external IDs
        let imdb_id = external_ids
            .or(movie.external_ids)
            .and_then(|ids| ids.imdb_id);
        
        metadata.imdb_id = imdb_id.clone();
        metadata.external_ids = ExternalIds {
            tmdb_id: Some(id),
            imdb_id,
            kinopoisk_id: None,
        };

        metadata
    }

    fn convert_tv_to_metadata(
        tv: TmdbTvDetails,
        external_ids: Option<TmdbExternalIds>,
        image_base_url: &str,
    ) -> MediaMetadata {
        let id = tv.id.map(|id| id.to_string()).unwrap_or_default();
        let title = tv.name.unwrap_or_default();
        
        let mut metadata = MediaMetadata::new(id.clone(), title, MediaType::Series);
        metadata.original_title = tv.original_name;
        metadata.overview = tv.overview;
        metadata.year = tv.first_air_date.as_ref().and_then(|d| Self::extract_year_from_date(d));
        metadata.release_date = tv.first_air_date;
        // For TV shows, runtime is typically the episode runtime
        metadata.runtime = tv.episode_run_time
            .and_then(|runtimes| runtimes.first().copied())
            .map(|r| r as u32);
        metadata.genres = tv.genres
            .unwrap_or_default()
            .into_iter()
            .filter_map(|g| g.name)
            .collect();
        metadata.vote_average = tv.vote_average.map(|v| v as f32);
        metadata.vote_count = tv.vote_count.map(|v| v as u32);
        
        // Set poster and backdrop URLs
        if let Some(poster_path) = tv.poster_path {
            metadata.poster_url = Some(format!("{}/w500{}", image_base_url, poster_path));
        }
        if let Some(backdrop_path) = tv.backdrop_path {
            metadata.backdrop_url = Some(format!("{}/w1280{}", image_base_url, backdrop_path));
        }

        // Extract external IDs
        let imdb_id = external_ids
            .or(tv.external_ids)
            .and_then(|ids| ids.imdb_id);
        
        metadata.imdb_id = imdb_id.clone();
        metadata.external_ids = ExternalIds {
            tmdb_id: Some(id),
            imdb_id,
            kinopoisk_id: None,
        };

        metadata
    }

    fn convert_movie_to_search_result(
        movie: TmdbMovieObject,
        image_base_url: &str,
    ) -> MetadataSearchResult {
        let id = movie.id.map(|id| id.to_string()).unwrap_or_default();
        let title = movie.title.unwrap_or_default();
        let year = movie.release_date.as_ref().and_then(|d| Self::extract_year_from_date(d));
        let poster_url = movie.poster_path.map(|path| format!("{}/w500{}", image_base_url, path));

        MetadataSearchResult {
            id,
            title,
            year,
            media_type: MediaType::Movie,
            poster_url,
            overview: movie.overview,
        }
    }

    fn convert_tv_to_search_result(
        tv: TmdbTvObject,
        image_base_url: &str,
    ) -> MetadataSearchResult {
        let id = tv.id.map(|id| id.to_string()).unwrap_or_default();
        let title = tv.name.unwrap_or_default();
        let year = tv.first_air_date.as_ref().and_then(|d| Self::extract_year_from_date(d));
        let poster_url = tv.poster_path.map(|path| format!("{}/w500{}", image_base_url, path));

        MetadataSearchResult {
            id,
            title,
            year,
            media_type: MediaType::Series,
            poster_url,
            overview: tv.overview,
        }
    }
}

#[async_trait]
impl MetadataProvider for TmdbClient {
    fn name(&self) -> &str {
        "TMDb"
    }

    async fn is_enabled(&self) -> Result<bool> {
        Ok(self.api_key.is_some())
    }

    async fn search(&self, title: &str, year: Option<u32>, media_type: MediaType) -> Result<Vec<MetadataSearchResult>> {
        // Initialize image config if needed
        self.init_image_config().await.ok();
        let image_base_url = self.image_base_url.lock().await
            .clone()
            .unwrap_or_else(|| "https://image.tmdb.org/t/p".to_string());

        let api_key = match &self.api_key {
            Some(key) => key.clone(),
            None => {
                return Ok(Vec::new());
            }
        };

        debug!("Searching TMDb for: {} ({:?})", title, year);
        
        let search_results: Vec<MetadataSearchResult> = match media_type {
            MediaType::Movie => {
                let url = "https://api.themoviedb.org/3/search/movie";
                let mut query_pairs: Vec<(String, String)> = vec![
                    ("api_key".to_string(), api_key.clone()),
                    ("query".to_string(), title.to_string()),
                    ("page".to_string(), "1".to_string()),
                ];
                
                if let Some(year_val) = year {
                    query_pairs.push(("year".to_string(), year_val.to_string()));
                }
                
                let response = self.client
                    .get(url)
                    .query(&query_pairs.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect::<Vec<_>>())
                    .send()
                    .await
                    .map_err(|e| anyhow!("HTTP request error: {:?}", e))?;

                if !response.status().is_success() {
                    let status = response.status();
                    let text = response.text().await.unwrap_or_default();
                    return Err(anyhow!("TMDB API error: {} - {}", status, text));
                }

                let search_response: TmdbSearchResponse<TmdbMovieObject> = response.json().await
                    .map_err(|e| anyhow!("Failed to parse response: {:?}", e))?;

                if let Some(movies) = search_response.results {
                    movies
                        .into_iter()
                        .map(|m| Self::convert_movie_to_search_result(m, &image_base_url))
                        .collect()
                } else {
                    Vec::new()
                }
            }
            MediaType::Series => {
                let url = "https://api.themoviedb.org/3/search/tv";
                let mut query_pairs: Vec<(String, String)> = vec![
                    ("api_key".to_string(), api_key.clone()),
                    ("query".to_string(), title.to_string()),
                    ("page".to_string(), "1".to_string()),
                ];
                
                if let Some(year_val) = year {
                    query_pairs.push(("first_air_date_year".to_string(), year_val.to_string()));
                }
                
                let response = self.client
                    .get(url)
                    .query(&query_pairs.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect::<Vec<_>>())
                    .send()
                    .await
                    .map_err(|e| anyhow!("HTTP request error: {:?}", e))?;

                if !response.status().is_success() {
                    let status = response.status();
                    let text = response.text().await.unwrap_or_default();
                    return Err(anyhow!("TMDB API error: {} - {}", status, text));
                }

                let search_response: TmdbSearchResponse<TmdbTvObject> = response.json().await
                    .map_err(|e| anyhow!("Failed to parse response: {:?}", e))?;

                if let Some(tv_shows) = search_response.results {
                    tv_shows
                        .into_iter()
                        .map(|t| Self::convert_tv_to_search_result(t, &image_base_url))
                        .collect()
                } else {
                    Vec::new()
                }
            }
            _ => Vec::new(),
        };

        debug!("TMDb returned {} results for: {}", search_results.len(), title);
        Ok(search_results)
    }

    async fn get_details(&self, id: &str, media_type: MediaType) -> Result<Option<MediaMetadata>> {
        // Initialize image config if needed
        self.init_image_config().await.ok();
        let image_base_url = self.image_base_url.lock().await
            .clone()
            .unwrap_or_else(|| "https://image.tmdb.org/t/p".to_string());

        let api_key = match &self.api_key {
            Some(key) => key.clone(),
            None => {
                return Ok(None);
            }
        };

        let id_i32: i32 = id.parse().map_err(|_| anyhow!("Invalid TMDb ID: {}", id))?;

        debug!("Fetching TMDb details for ID: {} ({:?})", id, media_type);

        let metadata = match media_type {
            MediaType::Movie => {
                // Fetch movie details and external IDs in parallel
                let details_url = format!("https://api.themoviedb.org/3/movie/{}", id_i32);
                let external_ids_url = format!("https://api.themoviedb.org/3/movie/{}/external_ids", id_i32);
                
                let (movie_result, external_ids_result) = tokio::join!(
                    async {
                        self.client
                            .get(&details_url)
                            .query(&[("api_key", api_key.as_str()), ("append_to_response", "external_ids")])
                            .send()
                            .await?
                            .json::<TmdbMovieDetails>()
                            .await
                    },
                    async {
                        match self.client
                            .get(&external_ids_url)
                            .query(&[("api_key", api_key.as_str())])
                            .send()
                            .await
                        {
                            Ok(r) => r.json::<TmdbExternalIds>().await.ok(),
                            Err(_) => None,
                        }
                    }
                );

                let movie = movie_result?;
                let external_ids = external_ids_result.or_else(|| movie.external_ids.clone());

                Some(Self::convert_movie_to_metadata(movie, external_ids, &image_base_url))
            }
            MediaType::Series => {
                // Fetch TV details and external IDs in parallel
                let details_url = format!("https://api.themoviedb.org/3/tv/{}", id_i32);
                let external_ids_url = format!("https://api.themoviedb.org/3/tv/{}/external_ids", id_i32);
                
                let (tv_result, external_ids_result) = tokio::join!(
                    async {
                        self.client
                            .get(&details_url)
                            .query(&[("api_key", api_key.as_str()), ("append_to_response", "external_ids")])
                            .send()
                            .await?
                            .json::<TmdbTvDetails>()
                            .await
                    },
                    async {
                        match self.client
                            .get(&external_ids_url)
                            .query(&[("api_key", api_key.as_str())])
                            .send()
                            .await
                        {
                            Ok(r) => r.json::<TmdbExternalIds>().await.ok(),
                            Err(_) => None,
                        }
                    }
                );

                let tv = tv_result?;
                let external_ids = external_ids_result.or_else(|| tv.external_ids.clone());

                Some(Self::convert_tv_to_metadata(tv, external_ids, &image_base_url))
            }
            _ => {
                return Ok(None);
            }
        };

        if let Some(ref meta) = metadata {
            debug!("TMDb details fetched for: {}", meta.title);
        }

        Ok(metadata)
    }

    fn get_poster_url(&self, path: &str) -> Option<String> {
        if path.is_empty() {
            return None;
        }
        // Use standard poster size (fallback if config not loaded)
        Some(format!("https://image.tmdb.org/t/p/w500{}", path))
    }

    fn get_backdrop_url(&self, path: &str) -> Option<String> {
        if path.is_empty() {
            return None;
        }
        // Use standard backdrop size (fallback if config not loaded)
        Some(format!("https://image.tmdb.org/t/p/w1280{}", path))
    }
}

impl TmdbClient {
    /// Test connection to TMDB API with current proxy configuration
    pub async fn test_connection(&self) -> Result<String> {
        let api_key = match &self.api_key {
            Some(key) => key.clone(),
            None => {
                return Err(anyhow!("No API key configured"));
            }
        };

        let config_url = "https://api.themoviedb.org/3/configuration";
        let response = self.client
            .get(config_url)
            .query(&[("api_key", api_key.as_str())])
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("HTTP error: {} - {}", status, text));
        }

        let text = response.text().await?;
        Ok(format!("Connection successful! Status: {}, Response: {} bytes", status, text.len()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenvy;

    #[tokio::test]
    async fn test_tmdb_connection() {
        // Load environment variables from multiple locations (same as main.rs)
        let env_paths = [
            ".env",
            "engine/src/.env",
            "src-tauri/.env",
            "../.env",
            "../engine/src/.env",
        ];
        
        let mut env_loaded = false;
        for path in &env_paths {
            if dotenvy::from_filename(path).is_ok() {
                eprintln!("✅ Test: Loaded .env from: {}", path);
                env_loaded = true;
                break;
            }
        }
        
        if !env_loaded {
            // Try loading from current directory as fallback
            if dotenvy::dotenv().is_ok() {
                eprintln!("✅ Test: Loaded .env from current directory");
            } else {
                eprintln!("⚠️ Test: No .env file found");
            }
        }
        
        let api_key = std::env::var("TMDB_API_KEY").ok();
        if api_key.is_none() {
            eprintln!("⚠️ TMDB_API_KEY not set, skipping test");
            eprintln!("💡 Tip: Make sure .env file exists in one of: {:?}", env_paths);
            return;
        }

        eprintln!("\n🧪 ========== TMDB Connection Test ==========");
        eprintln!("🧪 Testing different proxy configurations...\n");

        // Test 1: No proxy
        eprintln!("🧪 Test 1: No proxy");
        let client_no_proxy = TmdbClient::new(api_key.clone());
        match client_no_proxy.test_connection().await {
            Ok(msg) => eprintln!("✅ Test 1 PASSED: {}", msg),
            Err(e) => eprintln!("❌ Test 1 FAILED: {}", e),
        }

        // Test 2: With proxy (if ADDRESS_PORT is set)
        if let Ok(proxy_url) = std::env::var("ADDRESS_PORT") {
            eprintln!("\n🧪 Test 2: With SOCKS5 proxy: {}", proxy_url);
            std::env::set_var("ADDRESS_PORT", &proxy_url);
            let client_with_proxy = TmdbClient::new(api_key.clone());
            match client_with_proxy.test_connection().await {
                Ok(msg) => eprintln!("✅ Test 2 PASSED: {}", msg),
                Err(e) => eprintln!("❌ Test 2 FAILED: {}", e),
            }
        } else {
            eprintln!("\n⚠️ ADDRESS_PORT not set, skipping proxy test");
        }

        eprintln!("\n🧪 ========== Test Complete ==========\n");
    }

    #[tokio::test]
    async fn test_tmdb_search() {
        // Load environment variables from multiple locations
        let env_paths = [
            ".env",
            "engine/src/.env",
            "src-tauri/.env",
            "../.env",
            "../engine/src/.env",
        ];
        
        for path in &env_paths {
            if dotenvy::from_filename(path).is_ok() {
                break;
            }
        }
        dotenvy::dotenv().ok(); // Fallback to current directory
        
        let api_key = std::env::var("TMDB_API_KEY").ok();
        if api_key.is_none() {
            eprintln!("⚠️ TMDB_API_KEY not set, skipping test");
            return;
        }

        eprintln!("\n🧪 ========== TMDB Search Test ==========");
        
        let client = TmdbClient::new(api_key);
        let results = client.search("The Matrix", Some(1999), MediaType::Movie).await;
        
        match results {
            Ok(movies) => {
                eprintln!("✅ Search successful! Found {} results", movies.len());
                for (i, movie) in movies.iter().take(3).enumerate() {
                    eprintln!("  {}. {} ({:?})", i + 1, movie.title, movie.year);
                }
            }
            Err(e) => {
                eprintln!("❌ Search failed: {}", e);
            }
        }
        
        eprintln!("\n🧪 ========== Test Complete ==========\n");
    }

    #[tokio::test]
    async fn test_tmdb_bearer_token() {
        // Load environment variables
        let env_paths = [
            ".env",
            "engine/src/.env",
            "src-tauri/.env",
            "../.env",
            "../engine/src/.env",
        ];
        
        let mut env_loaded = false;
        for path in &env_paths {
            if dotenvy::from_filename(path).is_ok() {
                eprintln!("✅ Test: Loaded .env from: {}", path);
                env_loaded = true;
                break;
            }
        }
        
        if !env_loaded {
            dotenvy::dotenv().ok();
        }

        // Get Bearer token from environment (TMDB_READ_ACCESS_TOKEN)
        let bearer_token = match std::env::var("TMDB_READ_ACCESS_TOKEN") {
            Ok(token) => token,
            Err(_) => {
                eprintln!("⚠️ TMDB_READ_ACCESS_TOKEN not set, skipping test");
                eprintln!("💡 Set TMDB_READ_ACCESS_TOKEN in .env file to test Bearer token authentication");
                return;
            }
        };

        eprintln!("\n🧪 ========== TMDB Bearer Token Test ==========");
        eprintln!("🧪 Testing authentication endpoint with Bearer token...\n");

        // Create a client with proxy support if available
        let mut client_builder = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36");
        
        if let Ok(proxy_url) = std::env::var("ADDRESS_PORT") {
            let socks5h_url = format!("socks5h://{}", proxy_url);
            if let Ok(proxy) = reqwest::Proxy::all(&socks5h_url) {
                client_builder = client_builder.proxy(proxy);
                eprintln!("✅ Test: SOCKS5 proxy configured (with DNS): {}", socks5h_url);
            }
        }
        
        let client = client_builder.build().expect("Failed to create HTTP client");

        // Test authentication endpoint (like the Python example)
        let url = "https://api.themoviedb.org/3/authentication";
        eprintln!("🧪 Test: GET {}", url);
        
        let start = std::time::Instant::now();
        match client
            .get(url)
            .header("accept", "application/json")
            .header("Authorization", format!("Bearer {}", bearer_token))
            .send()
            .await
        {
            Ok(response) => {
                let elapsed = start.elapsed();
                let status = response.status();
                eprintln!("🧪 Test: Response status: {} (took {:?})", status, elapsed);
                
                match response.text().await {
                    Ok(text) => {
                        eprintln!("✅ Test PASSED! Response:");
                        eprintln!("{}", text);
                    }
                    Err(e) => {
                        eprintln!("❌ Test FAILED: Failed to read response: {}", e);
                    }
                }
            }
            Err(e) => {
                let elapsed = start.elapsed();
                eprintln!("❌ Test FAILED after {:?}: {:?}", elapsed, e);
                if e.is_request() {
                    eprintln!("   Error type: Request error");
                } else if e.is_timeout() {
                    eprintln!("   Error type: Timeout");
                } else if e.is_connect() {
                    eprintln!("   Error type: Connection error");
                }
                if let Some(url) = e.url() {
                    eprintln!("   Failed URL: {}", url);
                }
            }
        }
        
        eprintln!("\n🧪 ========== Test Complete ==========\n");
    }
}
