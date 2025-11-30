use std::sync::Arc;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use tracing::{info, warn, debug};

use super::manager::WorkerManager;
use crate::ratings::RatingManager;
use crate::models::EnrichedMedia;

/// Worker manager adapter for RatingManager
pub struct RatingWorkerManager {
    rating_manager: Arc<RatingManager>,
}

impl RatingWorkerManager {
    pub fn new(rating_manager: Arc<RatingManager>) -> Self {
        info!("RatingWorkerManager: Creating new RatingWorkerManager");
        Self { rating_manager }
    }
}

#[async_trait]
impl WorkerManager for RatingWorkerManager {
    async fn process(&self, params: Value) -> Result<Value> {
        info!("RatingWorkerManager: process() called - starting rating fetch job");
        debug!("RatingWorkerManager: Params is_array: {}, size: {}", 
               params.is_array(),
               if params.is_array() { 
                   format!("array with {} elements", params.as_array().map(|a| a.len()).unwrap_or(0)) 
               } else { 
                   "not an array".to_string() 
               });

        // Deserialize movies from params
        info!("RatingWorkerManager: Deserializing movies from params...");
        let movies: Vec<EnrichedMedia> = match serde_json::from_value::<Vec<EnrichedMedia>>(params) {
            Ok(m) => {
                info!("RatingWorkerManager: Successfully deserialized {} movies", m.len());
                for (idx, movie) in m.iter().take(3).enumerate() {
                    debug!("RatingWorkerManager: Movie {}: {} ({:?}) - Kinopoisk: {:?}, IMDB: {:?}, TMDB: {:?}", 
                           idx + 1, movie.parsed.title, movie.parsed.year,
                           movie.rating_kinopoisk, movie.rating_imdb, movie.rating_tmdb);
                }
                m
            }
            Err(e) => {
                warn!("RatingWorkerManager: Failed to deserialize movies: {}", e);
                return Err(anyhow::anyhow!("Failed to deserialize movies: {}", e));
            }
        };

        if movies.is_empty() {
            warn!("RatingWorkerManager: Received empty movies list, nothing to process");
            return Ok(serde_json::to_value(Vec::<EnrichedMedia>::new())?);
        }

        info!("RatingWorkerManager: Calling rating_manager.check_and_fetch_ratings() for {} movies", movies.len());

        // Fetch ratings
        let updated_movies = match self.rating_manager.check_and_fetch_ratings(movies).await {
            Ok(movies) => {
                info!("RatingWorkerManager: check_and_fetch_ratings() completed successfully, got {} movies back", movies.len());
                // Log some results
                let with_kinopoisk = movies.iter().filter(|m| m.rating_kinopoisk.is_some()).count();
                let with_imdb = movies.iter().filter(|m| m.rating_imdb.is_some()).count();
                let with_tmdb = movies.iter().filter(|m| m.rating_tmdb.is_some()).count();
                info!("RatingWorkerManager: Results - {} with Kinopoisk, {} with IMDB, {} with TMDB", 
                      with_kinopoisk, with_imdb, with_tmdb);
                movies
            }
            Err(e) => {
                warn!("RatingWorkerManager: check_and_fetch_ratings() failed: {}", e);
                return Err(e);
            }
        };

        info!("RatingWorkerManager: Serializing results...");
        // Serialize results
        let result = match serde_json::to_value(&updated_movies) {
            Ok(r) => {
                info!("RatingWorkerManager: Successfully serialized {} movies to JSON", updated_movies.len());
                r
            }
            Err(e) => {
                warn!("RatingWorkerManager: Failed to serialize results: {}", e);
                return Err(anyhow::anyhow!("Failed to serialize results: {}", e));
            }
        };
        
        info!("RatingWorkerManager: process() completed successfully");
        Ok(result)
    }

    fn job_kind(&self) -> &str {
        "FetchRatings"
    }
}

