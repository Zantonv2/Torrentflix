// Placeholder TMDb client
// TODO: Implement actual TMDb API integration

use reqwest::Client;
use serde::Deserialize;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use tracing::debug;

use super::traits::MetadataProvider;
use super::models::{MediaMetadata, MetadataSearchResult, MediaType, ExternalIds};

/// TMDb API client for movie metadata
pub struct TmdbClient {
    client: Client,
    api_key: Option<String>,
    base_url: String,
    image_base_url: Option<String>,
}

/// TMDb movie search response
#[derive(Debug, Deserialize)]
struct TmdbSearchResponse {
    results: Vec<TmdbMovieResult>,
    total_results: u32,
    total_pages: u32,
}

/// TMDb movie result
#[derive(Debug, Deserialize)]
struct TmdbMovieResult {
    id: u32,
    title: String,
    original_title: Option<String>,
    overview: Option<String>,
    release_date: Option<String>,
    poster_path: Option<String>,
    backdrop_path: Option<String>,
    genre_ids: Vec<u32>,
    vote_average: Option<f32>,
    vote_count: Option<u32>,
    popularity: f32,
}

/// TMDb movie details response
#[derive(Debug, Deserialize)]
struct TmdbMovieDetails {
    id: u32,
    title: String,
    original_title: Option<String>,
    overview: Option<String>,
    release_date: Option<String>,
    runtime: Option<u32>,
    poster_path: Option<String>,
    backdrop_path: Option<String>,
    genres: Vec<TmdbGenre>,
    vote_average: Option<f32>,
    vote_count: Option<u32>,
    imdb_id: Option<String>,
}

/// TMDb genre
#[derive(Debug, Deserialize)]
struct TmdbGenre {
    id: u32,
    name: String,
}

/// TMDb configuration response
#[derive(Debug, Deserialize)]
struct TmdbConfigResponse {
    images: TmdbImageConfig,
}

#[derive(Debug, Deserialize)]
struct TmdbImageConfig {
    base_url: String,
    secure_base_url: String,
    poster_sizes: Vec<String>,
    backdrop_sizes: Vec<String>,
}

impl TmdbClient {
    pub fn new(api_key: Option<String>) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.themoviedb.org/3".to_string(),
            image_base_url: None,
        }
    }

    /// Initialize image base URL from TMDb configuration
    async fn init_image_config(&mut self) -> Result<()> {
        if self.image_base_url.is_some() {
            return Ok(());
        }

        let config_url = format!("{}/configuration", self.base_url);
        let response = self.client
            .get(&config_url)
            .query(&[("api_key", self.api_key.as_ref().ok_or_else(|| anyhow!("TMDb API key not configured"))?)])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to fetch TMDb configuration: {}", response.status()));
        }

        let config: TmdbConfigResponse = response.json().await?;
        self.image_base_url = Some(config.images.secure_base_url);

        debug!("TMDb image base URL configured: {}", self.image_base_url.as_ref().unwrap());
        Ok(())
    }

    fn extract_year_from_date(date_str: &str) -> Option<u32> {
        date_str.split('-').next()?.parse().ok()
    }

    fn convert_tmdb_to_metadata(tmdb: TmdbMovieDetails) -> MediaMetadata {
        let mut metadata = MediaMetadata::new(tmdb.id.to_string(), tmdb.title, MediaType::Movie);
        metadata.original_title = tmdb.original_title;
        metadata.overview = tmdb.overview;
        metadata.year = tmdb.release_date.as_ref().and_then(|d| Self::extract_year_from_date(d));
        metadata.release_date = tmdb.release_date;
        metadata.runtime = tmdb.runtime;
        metadata.genres = tmdb.genres.into_iter().map(|g| g.name).collect();
        metadata.vote_average = tmdb.vote_average;
        metadata.vote_count = tmdb.vote_count;
        metadata.imdb_id = tmdb.imdb_id.clone();
        metadata.external_ids = ExternalIds {
            tmdb_id: Some(tmdb.id.to_string()),
            imdb_id: tmdb.imdb_id,
            kinopoisk_id: None,
        };
        metadata
    }

    fn convert_tmdb_to_search_result(tmdb: TmdbMovieResult) -> MetadataSearchResult {
        MetadataSearchResult {
            id: tmdb.id.to_string(),
            title: tmdb.title,
            year: tmdb.release_date.as_ref().and_then(|d| Self::extract_year_from_date(d)),
            media_type: MediaType::Movie,
            poster_url: tmdb.poster_path.clone(),
            overview: tmdb.overview,
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
        if self.api_key.is_none() {
            return Err(anyhow!("TMDb API key not configured"));
        }

        // TMDb primarily supports movies for now
        if !matches!(media_type, MediaType::Movie) {
            return Ok(Vec::new());
        }

        let search_url = format!("{}/search/movie", self.base_url);
        let title_string = title.to_string();
        let year_string = year.map(|y| y.to_string());
        
        let mut query_params = vec![
            ("api_key", self.api_key.as_ref().unwrap()),
            ("query", &title_string),
        ];

        if let Some(ref year_str) = year_string {
            query_params.push(("year", year_str));
        }

        debug!("Searching TMDb for: {} ({})", title, year.unwrap_or(0));

        let response = self.client
            .get(&search_url)
            .query(&query_params)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("TMDb search failed: {}", response.status()));
        }

        let search_response: TmdbSearchResponse = response.json().await?;
        let results: Vec<MetadataSearchResult> = search_response.results
            .into_iter()
            .map(TmdbClient::convert_tmdb_to_search_result)
            .collect();

        debug!("TMDb returned {} results for: {}", results.len(), title);
        Ok(results)
    }

    async fn get_details(&self, id: &str, media_type: MediaType) -> Result<Option<MediaMetadata>> {
        if self.api_key.is_none() {
            return Err(anyhow!("TMDb API key not configured"));
        }

        if !matches!(media_type, MediaType::Movie) {
            return Ok(None);
        }

        let details_url = format!("{}/movie/{}", self.base_url, id);
        
        debug!("Fetching TMDb details for ID: {}", id);

        let response = self.client
            .get(&details_url)
            .query(&[("api_key", self.api_key.as_ref().unwrap())])
            .send()
            .await?;

        if !response.status().is_success() {
            if response.status() == 404 {
                return Ok(None);
            }
            return Err(anyhow!("TMDb details failed: {}", response.status()));
        }

        let movie_details: TmdbMovieDetails = response.json().await?;
        let metadata = TmdbClient::convert_tmdb_to_metadata(movie_details);

        debug!("TMDb details fetched for: {}", metadata.title);
        Ok(Some(metadata))
    }

    fn get_poster_url(&self, path: &str) -> Option<String> {
        if path.is_empty() {
            return None;
        }
        // Use standard poster size
        Some(format!("https://image.tmdb.org/t/p/w500{}", path))
    }

    fn get_backdrop_url(&self, path: &str) -> Option<String> {
        if path.is_empty() {
            return None;
        }
        // Use standard backdrop size
        Some(format!("https://image.tmdb.org/t/p/w1280{}", path))
    }
}
