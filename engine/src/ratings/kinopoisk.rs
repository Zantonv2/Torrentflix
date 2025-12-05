use reqwest::Client;
use serde::Deserialize;
use anyhow::{Result, anyhow};
use tracing::{debug, warn};
use std::time::Duration;

use crate::normalize::RatingTitleInfo;
use crate::models::EnrichedMedia;

/// Kinopoisk API Unofficial client for fetching ratings
/// API Documentation: https://kinopoiskapiunofficial.tech/documentation/api/
pub struct KinopoiskClient {
    client: Client,
    api_key: String,
    base_url: String,
}

/// Kinopoisk search response - simplified to match working scraper
#[derive(Debug, Deserialize)]
struct KinopoiskSearchResponse {
    #[serde(rename = "films", default)]
    films: Vec<KinopoiskSearchItem>,
}

/// Simplified search item - only extract filmId from search
#[derive(Debug, Deserialize)]
struct KinopoiskSearchItem {
    #[serde(rename = "filmId")]
    film_id: u32,
}

/// Kinopoisk movie details response - simplified to match working scraper
#[derive(Debug, Deserialize)]
struct KinopoiskMovieDetails {
    #[serde(rename = "kinopoiskId")]
    kinopoisk_id: u32,
    #[serde(rename = "imdbId", default)]
    imdb_id: Option<String>,
    #[serde(rename = "ratingKinopoisk", default)]
    rating_kinopoisk: Option<f64>,
    #[serde(rename = "ratingKinopoiskVoteCount", default)]
    rating_kinopoisk_vote_count: Option<u32>,
    #[serde(rename = "nameRu", default)]
    name_ru: Option<String>,
    #[serde(rename = "nameEn", default)]
    name_en: Option<String>,
    #[serde(rename = "year", default)]
    year: Option<i32>,
}

impl KinopoiskClient {
    /// Create a new Kinopoisk client
    /// Loads API token from KINOPOISK_API_TOKEN environment variable
    /// 
    /// Note: This method is provided for backward compatibility and testing.
    /// In production, use `with_api_token()` with the token from settings.
    pub fn new() -> Result<Self> {
        let api_key = std::env::var("KINOPOISK_API_TOKEN")
            .map_err(|_| anyhow!("KINOPOISK_API_TOKEN not found in environment variables"))?;

        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?;

        Ok(Self {
            client,
            api_key,
            base_url: "https://kinopoiskapiunofficial.tech/api".to_string(),
        })
    }

    /// Create a new Kinopoisk client with provided API token
    /// This allows using API token from settings instead of environment variables
    pub fn with_api_token(api_token: String) -> Result<Self> {
        debug!("Kinopoisk Client: Initializing with API token from settings");
        
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?;

        debug!("Kinopoisk Client: ✅ Client initialized successfully");
        Ok(Self {
            client,
            api_key: api_token,
            base_url: "https://kinopoiskapiunofficial.tech/api".to_string(),
        })
    }

    /// Search for a movie/series by title
    /// Returns the best match or None if not found
    /// Simplified to match working scraper - just get first result's filmId
    pub async fn search_by_title(&self, title_info: &RatingTitleInfo) -> Result<Option<KinopoiskRatingResult>> {
        let keyword = urlencoding::encode(&title_info.cleaned_title);
        let mut url = format!("{}/v2.1/films/search-by-keyword?keyword={}", self.base_url, keyword);
        
        // Add year if available (like working scraper)
        if let Some(year) = title_info.year {
            url.push_str(&format!("&year={}", year));
        }

        debug!("Kinopoisk: Searching for '{}'", title_info.cleaned_title);

        let response = self.client
            .get(&url)
            .header("X-API-KEY", &self.api_key)
            .send()
            .await
            .map_err(|e| {
                warn!("Kinopoisk: HTTP request failed for '{}': {}", title_info.cleaned_title, e);
                anyhow!("HTTP request error: {}", e)
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            warn!("Kinopoisk API error ({}): {}", status, error_text);
            return Err(anyhow!("Kinopoisk API error: {}", status));
        }

        let search_response: KinopoiskSearchResponse = response.json().await.map_err(|e| {
            warn!("Kinopoisk: Failed to parse search response for '{}': {}", title_info.cleaned_title, e);
            anyhow!("error decoding response body: {}", e)
        })?;

        if search_response.films.is_empty() {
            debug!("Kinopoisk: No results found for '{}'", title_info.cleaned_title);
            return Ok(None);
        }

        // Use first result's filmId (like working scraper)
        let film_id = search_response.films[0].film_id;
        debug!("Kinopoisk: Found {} films for '{}', using first: {}", 
               search_response.films.len(), title_info.cleaned_title, film_id);

        // Fetch full details to get accurate rating
        self.get_rating_by_id(film_id).await
    }

    /// Get rating by Kinopoisk ID
    pub async fn get_rating_by_id(&self, kinopoisk_id: u32) -> Result<Option<KinopoiskRatingResult>> {
        let url = format!("{}/v2.2/films/{}", self.base_url, kinopoisk_id);

        debug!("Kinopoisk: Fetching details for ID {}", kinopoisk_id);

        let response = self.client
            .get(&url)
            .header("X-API-KEY", &self.api_key)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            if status == 404 {
                debug!("Kinopoisk: Movie with ID {} not found", kinopoisk_id);
                return Ok(None);
            }
            let error_text = response.text().await.unwrap_or_default();
            warn!("Kinopoisk API error ({}): {}", status, error_text);
            return Err(anyhow!("Kinopoisk API error: {}", status));
        }

        let details: KinopoiskMovieDetails = response.json().await?;

        Ok(Some(KinopoiskRatingResult {
            kinopoisk_id: details.kinopoisk_id,
            rating: details.rating_kinopoisk.map(|r| r as f32), // Convert f64 to f32
            rating_vote_count: details.rating_kinopoisk_vote_count,
            imdb_id: details.imdb_id,
        }))
    }

    /// Fetch rating for a movie/series using normalized title info
    /// This is the main entry point for the rating worker
    pub async fn fetch_rating(&self, title_info: &RatingTitleInfo) -> Result<Option<f32>> {
        match self.search_by_title(title_info).await {
            Ok(Some(result)) => Ok(result.rating),
            Ok(None) => {
                debug!("Kinopoisk: No rating found for '{}'", title_info.cleaned_title);
                Ok(None)
            }
            Err(e) => {
                warn!("Kinopoisk: Error fetching rating for '{}': {}", title_info.cleaned_title, e);
                Err(e)
            }
        }
    }

    /// Fetch rating and update EnrichedMedia if found
    pub async fn fetch_and_update_rating(
        &self,
        media: &mut EnrichedMedia,
        title_info: &RatingTitleInfo,
    ) -> Result<bool> {
        match self.search_by_title(title_info).await {
            Ok(Some(result)) => {
                media.rating_kinopoisk = result.rating;
                if media.kinopoisk_id.is_none() {
                    media.kinopoisk_id = Some(result.kinopoisk_id);
                }
                if media.imdb_id.is_none() && result.imdb_id.is_some() {
                    media.imdb_id = result.imdb_id;
                }
                Ok(true)
            }
            Ok(None) => {
                debug!("Kinopoisk: No rating found for '{}'", title_info.cleaned_title);
                Ok(false)
            }
            Err(e) => {
                warn!("Kinopoisk: Error fetching rating for '{}': {}", title_info.cleaned_title, e);
                Err(e)
            }
        }
    }
}

/// Result from Kinopoisk rating fetch
#[derive(Debug, Clone)]
pub struct KinopoiskRatingResult {
    pub kinopoisk_id: u32,
    pub rating: Option<f32>,
    pub rating_vote_count: Option<u32>,
    pub imdb_id: Option<String>,
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kinopoisk_client_creation() {
        // This will fail if KINOPOISK_API_TOKEN is not set, which is expected
        // In actual usage, the token should be in .env file
        let _ = KinopoiskClient::new();
    }
}
