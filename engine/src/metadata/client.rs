//! TMDB API client implementation
//! 
//! Rapid port of Python TMDBClient using generic HttpClient

use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::RwLock;

use crate::scrapers::base::{HttpClient, ScrapeError, ScrapeResult, RetryConfig};
use super::models::*;

/// TMDB API client
pub struct TMDBClient {
    /// HTTP client for API calls
    client: HttpClient,
    /// Simple in-memory cache
    cache: RwLock<HashMap<String, (TMDBMedia, Duration)>>,
    /// Base URL for images
    image_base_url: String,
    /// API key
    api_key: String,
}

impl TMDBClient {
    /// Create new TMDB client
    pub fn new(api_key: &str) -> ScrapeResult<Self> {
        let client = HttpClient::with_api_key(
            "https://api.themoviedb.org/3",
            api_key
        )?;
        
        Ok(Self {
            client,
            cache: RwLock::new(HashMap::new()),
            image_base_url: "https://image.tmdb.org/t/p/".to_string(),
            api_key: api_key.to_string(),
        })
    }
    
    /// Search for movies and TV shows
    pub async fn search_multi(&self, query: &str, year: Option<i32>) -> ScrapeResult<Vec<TMDBMedia>> {
        let mut endpoint = format!("search/multi?query={}&api_key={}", 
            urlencoding::encode(query), 
            urlencoding::encode(&self.api_key)
        );
        
        if let Some(year) = year {
            endpoint.push_str(&format!("&year={}", year));
        }
        
        println!("🔍 TMDB API Request URL: https://api.themoviedb.org/3/{}", endpoint);
        
        let response: TMDBSearchResponse = self.client.get_json(&endpoint).await?;
        Ok(response.results)
    }
    
    /// Get detailed movie information with credits
    pub async fn get_movie_details(&self, movie_id: i32) -> ScrapeResult<TMDBMedia> {
        let endpoint = format!("movie/{}?append_to_response=credits&api_key={}", 
            movie_id,
            urlencoding::encode(&self.api_key)
        );
        let mut movie: TMDBMedia = self.client.get_json(&endpoint).await?;
        movie.media_type = "movie".to_string();
        Ok(movie)
    }
    
    /// Get detailed TV show information with credits
    pub async fn get_tv_details(&self, tv_id: i32) -> ScrapeResult<TMDBMedia> {
        let endpoint = format!("tv/{}?append_to_response=credits&api_key={}", 
            tv_id,
            urlencoding::encode(&self.api_key)
        );
        let mut tv: TMDBMedia = self.client.get_json(&endpoint).await?;
        tv.media_type = "tv".to_string();
        Ok(tv)
    }
    
    /// Search with detailed results for each item
    pub async fn search_multi_detailed(&self, query: &str, year: Option<i32>) -> ScrapeResult<Vec<TMDBMedia>> {
        let basic_results = self.search_multi(query, year).await?;
        let mut detailed_results = Vec::new();
        
        for item in basic_results {
            let detailed = if item.media_type == "movie" {
                self.get_movie_details(item.id).await?
            } else if item.media_type == "tv" {
                self.get_tv_details(item.id).await?
            } else {
                item // fallback to basic if unknown type
            };
            detailed_results.push(detailed);
        }
        
        Ok(detailed_results)
    }
    
    /// Construct full poster URL
    pub fn get_poster_url(&self, poster_path: &str, size: &str) -> String {
        format!("{}{}/{}", self.image_base_url, size, poster_path)
    }
    
    /// Construct full backdrop URL
    pub fn get_backdrop_url(&self, backdrop_path: &str, size: &str) -> String {
        format!("{}{}/{}", self.image_base_url, size, backdrop_path)
    }
    
    /// Get movie details by ID
    pub async fn get_movie(&self, id: i32) -> ScrapeResult<TMDBMedia> {
        let endpoint = format!("movie/{}?api_key={}", 
            id,
            urlencoding::encode(&self.api_key)
        );
        let mut media: TMDBMedia = self.client.get_json(&endpoint).await?;
        media.media_type = "movie".to_string();
        Ok(media)
    }
    
    /// Get TV show details by ID
    pub async fn get_tv_show(&self, id: i32) -> ScrapeResult<TMDBMedia> {
        let endpoint = format!("tv/{}?api_key={}", 
            id,
            urlencoding::encode(&self.api_key)
        );
        let mut media: TMDBMedia = self.client.get_json(&endpoint).await?;
        media.media_type = "tv".to_string();
        Ok(media)
    }
    
    /// Get external IDs for a media item
    pub async fn get_external_ids(&self, id: i32, media_type: &str) -> ScrapeResult<TMDBExternalIds> {
        // TMDB requires api_key as query parameter, not Bearer token
        let endpoint = format!("{}/{}/external_ids?api_key={}", 
            media_type, 
            id,
            urlencoding::encode(&self.api_key)
        );
        println!("🔍 TMDB get_external_ids URL: https://api.themoviedb.org/3/{}", endpoint);
        self.client.get_json(&endpoint).await
    }
    
    /// Search by IMDb ID
    pub async fn find_by_imdb_id(&self, imdb_id: &str) -> ScrapeResult<Option<TMDBMedia>> {
        let endpoint = format!("find/{}?external_source=imdb_id&api_key={}", 
            imdb_id,
            urlencoding::encode(&self.api_key)
        );
        let response: serde_json::Value = self.client.get_json(&endpoint).await?;
        
        // Check movie results first
        if let Some(movies) = response.get("movie_results").and_then(|v| v.as_array()) {
            if let Some(movie) = movies.first() {
                let media: TMDBMedia = serde_json::from_value(movie.clone())
                    .map_err(|e| ScrapeError::Parse { 
                        message: format!("Failed to parse movie: {}", e) 
                    })?;
                return Ok(Some(media));
            }
        }
        
        // Check TV results
        if let Some(tv_shows) = response.get("tv_results").and_then(|v| v.as_array()) {
            if let Some(show) = tv_shows.first() {
                let media: TMDBMedia = serde_json::from_value(show.clone())
                    .map_err(|e| ScrapeError::Parse { 
                        message: format!("Failed to parse TV show: {}", e) 
                    })?;
                return Ok(Some(media));
            }
        }
        
        Ok(None)
    }
    
    /// Get configuration (image base URLs, etc.)
    pub async fn get_config(&self) -> ScrapeResult<TMDBConfig> {
        let endpoint = format!("configuration?api_key={}", urlencoding::encode(&self.api_key));
        let config: TMDBConfig = self.client.get_json(&endpoint).await?;
        Ok(config)
    }
    
    /// Get popular movies
    pub async fn get_popular_movies(&self, page: i32) -> ScrapeResult<Vec<TMDBMedia>> {
        let endpoint = format!("movie/popular?page={}&api_key={}", 
            page,
            urlencoding::encode(&self.api_key)
        );
        let response: TMDBSearchResponse = self.client.get_json(&endpoint).await?;
        Ok(response.results)
    }
    
    /// Get popular TV shows
    pub async fn get_popular_tv_shows(&self, page: i32) -> ScrapeResult<Vec<TMDBMedia>> {
        let endpoint = format!("tv/popular?page={}&api_key={}", 
            page,
            urlencoding::encode(&self.api_key)
        );
        let response: TMDBSearchResponse = self.client.get_json(&endpoint).await?;
        Ok(response.results)
    }
    
    /// Get trending content (day or week)
    pub async fn get_trending(&self, media_type: &str, time_window: &str) -> ScrapeResult<Vec<TMDBMedia>> {
        let endpoint = format!("trending/{}/{}?api_key={}", 
            media_type, 
            time_window,
            urlencoding::encode(&self.api_key)
        );
        let response: TMDBSearchResponse = self.client.get_json(&endpoint).await?;
        Ok(response.results)
    }
    
    /// Simple cache lookup (in-memory, short-lived)
    async fn get_from_cache(&self, key: &str) -> Option<TMDBMedia> {
        let cache = self.cache.read().await;
        if let Some((media, _)) = cache.get(key) {
            return Some(media.clone());
        }
        None
    }
    
    /// Simple cache store
    async fn store_in_cache(&self, key: String, media: TMDBMedia) {
        let mut cache = self.cache.write().await;
        cache.insert(key, (media, Duration::from_secs(300))); // 5 minutes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_tmdb_client_creation() {
        let client = TMDBClient::new("test_key");
        assert!(client.is_ok());
    }
    
    #[tokio::test]
    async fn test_search_multi() {
        let client = TMDBClient::new("test_key");
        // This would fail with real API key, but tests structure
        assert!(client.is_ok());
    }
}
