use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Result;
use chrono::Utc;
use tracing::{info, warn, debug};
use tokio::sync::mpsc;
use std::sync::Mutex;

use crate::models::EnrichedMedia;
use crate::normalize::RatingTitleInfo;
use crate::metadata::{TmdbClient, MetadataProvider};
use crate::database::ratings::RatingCacheDatabase;
use crate::engine::RatingUpdate;

use super::cache::RatingCache;
use super::models::{RatingWorkerResult, RatingSource, TMDBMetadataResult, CachedTMDBMetadata, CachedRating};
use super::kinopoisk::KinopoiskClient;
use super::imdb::ImdbClient;

/// Rating Manager coordinates fetching ratings from Kinopoisk and IMDb
/// Uses 3 workers: Kinopoisk (independent), TMDB Worker 1 (metadata), TMDB Worker 2 (IMDb scraper)
pub struct RatingManager {
    cache: Arc<RatingCache>,
    db_cache: Option<Arc<RatingCacheDatabase>>,
    kinopoisk_client: Option<Arc<KinopoiskClient>>,
    imdb_client: Arc<ImdbClient>,
    tmdb_client: Arc<TmdbClient>,
    rating_update_tx: Arc<Mutex<Option<mpsc::UnboundedSender<RatingUpdate>>>>,
}

impl RatingManager {
    /// Create a new Rating Manager
    /// Kinopoisk is optional - if KINOPOISK_API_TOKEN is not set, RatingManager will still work but without Kinopoisk ratings
    pub fn new(tmdb_client: Arc<TmdbClient>) -> Result<Self> {
        // Kinopoisk is optional - create client only if API token is available
        let kinopoisk_client = match KinopoiskClient::new() {
            Ok(client) => {
                info!("Kinopoisk client initialized successfully");
                Some(Arc::new(client))
            },
            Err(e) => {
                warn!("Kinopoisk client initialization failed: {}. Ratings from Kinopoisk will not be available.", e);
                None
            }
        };
        let imdb_client = Arc::new(ImdbClient::new());

        Ok(Self {
            cache: Arc::new(RatingCache::new()),
            db_cache: None, // Will be set via set_db_cache()
            kinopoisk_client,
            imdb_client,
            tmdb_client,
            rating_update_tx: Arc::new(Mutex::new(None)),
        })
    }

    /// Set database cache for persistent rating storage
    pub fn with_db_cache(mut self, db_cache: Arc<RatingCacheDatabase>) -> Self {
        self.db_cache = Some(db_cache);
        self
    }

    /// Set rating update channel for sending updates to UI
    /// Can be called even when RatingManager is in an Arc (uses interior mutability)
    pub fn set_rating_update_channel(&self, tx: mpsc::UnboundedSender<RatingUpdate>) {
        if let Ok(mut guard) = self.rating_update_tx.lock() {
            *guard = Some(tx);
            eprintln!("🔵 Rating Manager: ✅ Rating update channel set (via Mutex)");
            info!("Rating Manager: ✅ Rating update channel set");
        } else {
            warn!("Rating Manager: Failed to lock Mutex to set rating update channel");
        }
    }

    /// Send rating update to UI if channel is available
    fn send_rating_update(&self, rating_tx: &Option<mpsc::UnboundedSender<RatingUpdate>>, movie_id: String, rating_kinopoisk: Option<f32>, rating_imdb: Option<f32>, rating_tmdb: Option<f32>) {
        if let Some(ref tx) = rating_tx {
            let update = RatingUpdate {
                movie_id: movie_id.clone(),
                rating_kinopoisk,
                rating_imdb,
                rating_tmdb,
            };
            if let Err(e) = tx.send(update) {
                eprintln!("🔵 Rating Manager: ❌ Failed to send rating update to UI for {}: {}", movie_id, e);
                warn!("Rating Manager: ❌ Failed to send rating update to UI for {}: {}", movie_id, e);
            } else {
                eprintln!("🔵 Rating Manager: ✅ Sent rating update to UI for {} - Kinopoisk: {:?}, IMDb: {:?}, TMDB: {:?}", 
                      movie_id, rating_kinopoisk, rating_imdb, rating_tmdb);
                info!("Rating Manager: ✅ Sent rating update to UI for {} - Kinopoisk: {:?}, IMDb: {:?}, TMDB: {:?}", 
                      movie_id, rating_kinopoisk, rating_imdb, rating_tmdb);
            }
        } else {
            eprintln!("🔵 Rating Manager: ⚠️  Channel not set, skipping update for {}", movie_id);
            debug!("Rating Manager: Channel not set, skipping update for {}", movie_id);
        }
    }

    /// Check and fetch missing ratings for all movies
    /// Returns updated EnrichedMedia with ratings filled in
    /// If rating_update_tx is provided, sends updates to UI as ratings are fetched
    pub async fn check_and_fetch_ratings(&self, movies: Vec<EnrichedMedia>) -> Result<Vec<EnrichedMedia>> {
        // Use stored channel if available (clone from Mutex)
        let rating_tx = self.rating_update_tx.lock()
            .ok()
            .and_then(|guard| guard.clone());
        if rating_tx.is_some() {
            eprintln!("🔵 Rating Manager: ✅ Rating update channel is SET - will send updates to UI");
            info!("Rating Manager: ✅ Rating update channel is SET - will send updates to UI");
        } else {
            eprintln!("🔵 Rating Manager: ⚠️  Rating update channel is NOT SET - updates will NOT be sent to UI");
            warn!("Rating Manager: ⚠️  Rating update channel is NOT SET - updates will NOT be sent to UI");
        }
        info!("═══════════════════════════════════════════════════════════");
        info!("Rating Manager: check_and_fetch_ratings() STARTED");
        info!("Rating Manager: Checking ratings for {} movies", movies.len());
        
        if movies.is_empty() {
            warn!("Rating Manager: Received empty movies list, returning early");
            return Ok(Vec::new());
        }

        // Identify missing ratings
        let mut movies_missing_kinopoisk = Vec::new();
        let mut movies_missing_imdb = Vec::new();

        for (idx, movie) in movies.iter().enumerate() {
            if movie.rating_kinopoisk.is_none() {
                movies_missing_kinopoisk.push((idx, movie.clone()));
            }
            if movie.rating_imdb.is_none() {
                movies_missing_imdb.push((idx, movie.clone()));
            }
        }

        eprintln!("🔵 Rating Manager: Found {} movies missing Kinopoisk ratings", movies_missing_kinopoisk.len());
        eprintln!("🔵 Rating Manager: Found {} movies missing IMDB ratings", movies_missing_imdb.len());
        info!("Rating Manager: Found {} movies missing Kinopoisk ratings", movies_missing_kinopoisk.len());
        info!("Rating Manager: Found {} movies missing IMDB ratings", movies_missing_imdb.len());
        
        // Log sample movies
        for (idx, (_, movie)) in movies_missing_kinopoisk.iter().take(3).enumerate() {
            eprintln!("  Missing Kinopoisk {}: '{}' ({:?})", idx + 1, movie.parsed.title, movie.parsed.year);
        }
        for (idx, (_, movie)) in movies_missing_imdb.iter().take(3).enumerate() {
            eprintln!("  Missing IMDB {}: '{}' ({:?})", idx + 1, movie.parsed.title, movie.parsed.year);
        }

        // Check cache first
        eprintln!("🔵 Rating Manager: Checking cache...");
        let (kinopoisk_from_cache, kinopoisk_to_fetch) = self.check_kinopoisk_cache(&movies_missing_kinopoisk).await;
        let (tmdb_metadata_from_cache, tmdb_metadata_to_fetch) = self.check_tmdb_metadata_cache(&movies_missing_imdb).await;

        eprintln!("🔵 Rating Manager: Cache lookup: {} Kinopoisk ratings found in cache, {} to fetch", 
                 kinopoisk_from_cache.len(), kinopoisk_to_fetch.len());
        eprintln!("🔵 Rating Manager: Cache lookup: {} TMDB metadata found in cache, {} to fetch", 
                 tmdb_metadata_from_cache.len(), tmdb_metadata_to_fetch.len());
        info!("Rating Manager: Cache lookup: {} Kinopoisk ratings found in cache", kinopoisk_from_cache.len());
        info!("Rating Manager: Cache lookup: {} TMDB metadata found in cache", tmdb_metadata_from_cache.len());

        // Spawn workers
        info!("Rating Manager: 🚀 SPAWNING WORKERS...");
        let kinopoisk_handle = if !kinopoisk_to_fetch.is_empty() && self.kinopoisk_client.is_some() {
            eprintln!("🟢 Rating Manager: Spawning Kinopoisk worker for {} movies", kinopoisk_to_fetch.len());
            info!("Rating Manager: 🟢 Spawning Kinopoisk worker ({} movies to fetch, {} from cache)", 
                  kinopoisk_to_fetch.len(), kinopoisk_from_cache.len());
            Some(self.spawn_kinopoisk_worker(kinopoisk_to_fetch))
        } else if !kinopoisk_to_fetch.is_empty() && self.kinopoisk_client.is_none() {
            eprintln!("⚪ Rating Manager: Kinopoisk client not available, skipping Kinopoisk ratings");
            info!("Rating Manager: ⚪ Kinopoisk client not available, skipping Kinopoisk ratings");
            None
        } else {
            eprintln!("⚪ Rating Manager: No movies to fetch for Kinopoisk (all from cache or already have ratings)");
            info!("Rating Manager: ⚪ Kinopoisk: All ratings from cache, no worker needed");
            None
        };

        let tmdb_metadata_handle = if !tmdb_metadata_to_fetch.is_empty() {
            eprintln!("🟢 Rating Manager: Spawning TMDB metadata worker for {} movies", tmdb_metadata_to_fetch.len());
            info!("Rating Manager: 🟢 Spawning TMDB metadata worker ({} movies to fetch)", tmdb_metadata_to_fetch.len());
            info!("Rating Manager: ⚠️  NOTE: This will BLOCK until TMDB metadata is fetched before IMDb can start");
            Some(self.spawn_tmdb_metadata_worker(tmdb_metadata_to_fetch))
        } else {
            eprintln!("⚪ Rating Manager: No movies to fetch for TMDB metadata (all from cache or already have ratings)");
            info!("Rating Manager: ⚪ TMDB metadata: All from cache, no worker needed");
            None
        };

        // Wait for TMDB Worker 1 to complete first (if needed)
        eprintln!("🔵 Rating Manager: ⏳ WAITING for TMDB Metadata Worker to complete...");
        info!("Rating Manager: ⏳ WAITING for TMDB Metadata Worker to complete...");
        let tmdb_metadata_results = if let Some(handle) = tmdb_metadata_handle {
            match handle.await? {
                Ok(results) => {
                    eprintln!("🔵 Rating Manager: ✅ TMDB Metadata Worker COMPLETED: Fetched metadata for {} movies", results.len());
                    info!("Rating Manager: ✅ TMDB Metadata Worker COMPLETED: Fetched metadata for {} movies", results.len());
                    info!("Rating Manager: Caching TMDB metadata for {} movies", results.len());
                    
                    // Cache the results (in-memory and database)
                    for result in &results {
                        // In-memory cache
                        let cache_key = RatingCache::tmdb_metadata_cache_key(
                            result.tmdb_id.as_deref(),
                            &result.movie_id,
                            None, // TODO: Get year from movie
                        );
                        let cached = CachedTMDBMetadata {
                            imdb_id: result.imdb_id.clone(),
                            episodes: result.episodes,
                            seasons: result.seasons,
                            cached_at: Utc::now(),
                        };
                        self.cache.set_tmdb_metadata(cache_key, cached).await;
                        
                        // Database cache - we'll update ratings when they're fetched
                        // TMDB metadata (imdb_id) will be stored when ratings are saved
                    }
                    
                    Some(results)
                }
                Err(e) => {
                    warn!("TMDB Metadata Worker failed: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // Merge cached TMDB metadata with fetched results
        info!("Rating Manager: 🔄 Merging TMDB metadata (cached + fetched)...");
        let all_tmdb_metadata: HashMap<String, TMDBMetadataResult> = {
            let mut map = HashMap::new();
            for (movie_id, result) in tmdb_metadata_from_cache {
                map.insert(movie_id, result);
            }
            if let Some(ref results) = tmdb_metadata_results {
                for result in results {
                    map.insert(result.movie_id.clone(), result.clone());
                }
            }
            info!("Rating Manager: ✅ Merged TMDB metadata: {} total movies with metadata", map.len());
            map
        };

        // Check IMDb cache
        info!("Rating Manager: 🔍 Checking IMDb cache...");
        eprintln!("🔵 Rating Manager: Checking IMDb cache for {} movies with TMDB metadata", all_tmdb_metadata.len());
        
        // Debug: show which movies have IMDb IDs
        let movies_with_imdb_id_count = all_tmdb_metadata.iter()
            .filter(|(_, m)| m.imdb_id.is_some())
            .count();
        eprintln!("🔵 Rating Manager: Movies with IMDb ID in metadata: {} (out of {} total)", 
                 movies_with_imdb_id_count, all_tmdb_metadata.len());
        
        let (imdb_from_cache, imdb_to_fetch) = self.check_imdb_cache(&all_tmdb_metadata).await;
        eprintln!("🔵 Rating Manager: ✅ IMDb cache lookup: {} found in cache, {} to fetch", imdb_from_cache.len(), imdb_to_fetch.len());
        info!("Rating Manager: ✅ IMDb cache lookup: {} found in cache, {} to fetch", imdb_from_cache.len(), imdb_to_fetch.len());

        // Cache movies without TMDB metadata (no IMDb ID) as None
        // These movies can't get IMDb ratings, so we cache None to avoid retrying
        info!("Rating Manager: 🔍 Identifying movies without IMDb ID (no TMDB metadata or imdb_id is None)...");
        let mut movies_without_imdb_id = Vec::new();
        for (_idx, movie) in &movies_missing_imdb {
            let movie_id = movie.parsed.identity_key();
            // Check if this movie has TMDB metadata with an IMDb ID
            let has_imdb_id = all_tmdb_metadata.get(&movie_id)
                .and_then(|m| m.imdb_id.as_ref())
                .is_some();
            
            // If movie doesn't have IMDb ID (either no TMDB metadata or imdb_id is None)
            // and isn't already cached, cache it as None
            if !has_imdb_id && !imdb_from_cache.contains_key(&movie_id) {
                movies_without_imdb_id.push((movie_id.clone(), movie.clone()));
            }
        }
        
        if !movies_without_imdb_id.is_empty() {
            info!("Rating Manager: 📦 Caching {} movies without IMDb ID as None (no TMDB metadata)", movies_without_imdb_id.len());
            // Cache these movies as having None IMDb rating in database
            if let Some(ref db_cache) = self.db_cache {
                for (movie_id, _movie) in &movies_without_imdb_id {
                    // Get existing record or create new
                    let mut record = match db_cache.get_rating(movie_id).await? {
                        Some(existing) => existing,
                        None => crate::database::ratings::CachedRatingRecord::new(movie_id.clone()),
                    };
                    // Cache as None (means we checked and movie doesn't have IMDb ID, so can't get rating)
                    record.rating_imdb = None;
                    eprintln!("🔵 Rating Manager: Caching movie without IMDb ID as None: movie_id={}", movie_id);
                    if let Err(e) = db_cache.set_rating(&record).await {
                        warn!("Failed to store 'no IMDb ID' cache for movie_id {}: {}", movie_id, e);
                    }
                }
            }
        }

        // Spawn IMDb scraper worker
        let imdb_handle = if !imdb_to_fetch.is_empty() {
            eprintln!("🔵 Rating Manager: 🟢 Spawning IMDb scraper worker ({} movies with imdb_id, {} from cache)", 
                  imdb_to_fetch.len(), imdb_from_cache.len());
            info!("Rating Manager: 🟢 Spawning IMDb scraper worker ({} movies with imdb_id, {} from cache)", 
                  imdb_to_fetch.len(), imdb_from_cache.len());
            Some(self.spawn_imdb_worker(imdb_to_fetch))
        } else {
            eprintln!("🔵 Rating Manager: ⚪ IMDb: All ratings from cache, no worker needed");
            eprintln!("🔵 Rating Manager: ⚠️  DEBUG: imdb_to_fetch is empty! Movies with IMDb ID: {}, From cache: {}", 
                     movies_with_imdb_id_count, imdb_from_cache.len());
            info!("Rating Manager: ⚪ IMDb: All ratings from cache, no worker needed");
            None
        };

        // Wait for all workers to complete
        eprintln!("🔵 Rating Manager: ⏳ WAITING for Kinopoisk Worker to complete...");
        info!("Rating Manager: ⏳ WAITING for Kinopoisk Worker to complete...");
        let kinopoisk_results = if let Some(handle) = kinopoisk_handle {
            match handle.await? {
                Ok(results) => {
                    eprintln!("🔵 Rating Manager: ✅ Kinopoisk Worker COMPLETED: Fetched {} ratings", results.len());
                    info!("Rating Manager: ✅ Kinopoisk Worker COMPLETED: Fetched {} ratings", results.len());
                    results
                }
                Err(e) => {
                    warn!("Rating Manager: ❌ Kinopoisk Worker FAILED: {}", e);
                    Vec::new()
                }
            }
        } else {
            info!("Rating Manager: ⚪ Kinopoisk Worker: No handle (all from cache)");
            Vec::new()
        };

        eprintln!("🔵 Rating Manager: ⏳ WAITING for IMDb Scraper Worker to complete...");
        info!("Rating Manager: ⏳ WAITING for IMDb Scraper Worker to complete...");
        let imdb_results = if let Some(handle) = imdb_handle {
            match handle.await? {
                Ok(results) => {
                    eprintln!("🔵 Rating Manager: ✅ IMDb Scraper Worker COMPLETED: Fetched {} ratings", results.len());
                    info!("Rating Manager: ✅ IMDb Scraper Worker COMPLETED: Fetched {} ratings", results.len());
                    results
                }
                Err(e) => {
                    warn!("Rating Manager: ❌ IMDb Scraper Worker FAILED: {}", e);
                    Vec::new()
                }
            }
        } else {
            info!("Rating Manager: ⚪ IMDb Scraper Worker: No handle (all from cache)");
            Vec::new()
        };
        
        eprintln!("🔵 Rating Manager: ✅ ALL WORKERS COMPLETED - Kinopoisk: {} results, IMDb: {} results", 
              kinopoisk_results.len(), imdb_results.len());
        info!("Rating Manager: ✅ ALL WORKERS COMPLETED - Kinopoisk: {} results, IMDb: {} results", 
              kinopoisk_results.len(), imdb_results.len());

        // Cache Kinopoisk results (in-memory and database)
        for result in &kinopoisk_results {
            let movie = movies.iter().find(|m| m.parsed.identity_key() == result.movie_id);
            if let Some(m) = movie {
                // In-memory cache
                let cache_key = RatingCache::kinopoisk_cache_key(
                    m.kinopoisk_id,
                    &m.parsed.title,
                    m.parsed.year,
                );
                let cached = CachedRating {
                    rating: result.rating,
                    cached_at: Utc::now(),
                };
                self.cache.set_kinopoisk_rating(cache_key, cached).await;
                
                // Database cache
                if let Some(ref db_cache) = self.db_cache {
                    let record = crate::database::ratings::CachedRatingRecord::new(result.movie_id.clone())
                        .with_kinopoisk(result.rating);
                    if let Err(e) = db_cache.set_rating(&record).await {
                        warn!("Failed to store Kinopoisk rating in DB cache: {}", e);
                    }
                }
            }
        }

        // Cache IMDb results (in-memory and database) - cache ALL results including None
        for result in &imdb_results {
            // In-memory cache - cache even if rating is None (means we checked and it doesn't exist)
            let cached = CachedRating {
                rating: result.rating,
                cached_at: Utc::now(),
            };
            // For IMDb results, result.movie_id is the movie's identity_key
            // Use all_tmdb_metadata to get the imdb_id (more reliable than looking up the movie)
            if let Some(metadata) = all_tmdb_metadata.get(&result.movie_id) {
                if let Some(imdb_id) = &metadata.imdb_id {
                    eprintln!("🔵 Rating Manager: Caching IMDb rating for imdb_id={}, rating={:?}", imdb_id, result.rating);
                    self.cache.set_imdb_rating(imdb_id.clone(), cached).await;
                } else {
                    eprintln!("🔵 Rating Manager: ⚠️  No imdb_id in metadata for movie_id={}, cannot cache IMDb rating", result.movie_id);
                }
            } else {
                eprintln!("🔵 Rating Manager: ⚠️  No metadata found for movie_id={}, cannot cache IMDb rating", result.movie_id);
            }
            
            // Database cache - use identity_key directly (result.movie_id is the identity_key)
            if let Some(ref db_cache) = self.db_cache {
                let movie_id = &result.movie_id;
                // Get existing record or create new
                let mut record = match db_cache.get_rating(movie_id).await? {
                    Some(existing) => existing,
                    None => crate::database::ratings::CachedRatingRecord::new(movie_id.clone()),
                };
                // Cache the rating even if it's None (means we checked and it doesn't exist)
                record.rating_imdb = result.rating;
                eprintln!("🔵 Rating Manager: Caching IMDb rating in DB for movie_id={}, rating={:?}", movie_id, result.rating);
                if let Err(e) = db_cache.set_rating(&record).await {
                    warn!("Failed to store IMDb rating in DB cache: {}", e);
                }
            }
        }

        // Merge all results back into movies
        let mut updated_movies = movies;
        
        // Apply Kinopoisk ratings (from cache and worker) and send updates
        eprintln!("🔵 Rating Manager: 📤 Sending Kinopoisk rating updates to UI...");
        eprintln!("🔵 Rating Manager: Channel set? {}", rating_tx.is_some());
        eprintln!("🔵 Rating Manager: Kinopoisk from cache: {} movies", kinopoisk_from_cache.len());
        eprintln!("🔵 Rating Manager: Kinopoisk results: {} movies", kinopoisk_results.len());
        info!("Rating Manager: 📤 Sending Kinopoisk rating updates to UI...");
        for (idx, movie) in kinopoisk_from_cache {
            if let Some(m) = updated_movies.get_mut(idx) {
                let movie_id = m.parsed.identity_key();
                m.rating_kinopoisk = movie.rating_kinopoisk;
                eprintln!("🔵 Rating Manager: Sending update for cached Kinopoisk: {}", movie_id);
                // Send update with current ratings
                self.send_rating_update(&rating_tx, movie_id, m.rating_kinopoisk, m.rating_imdb, m.rating_tmdb);
            }
        }
        for result in kinopoisk_results {
            if let Some(movie) = updated_movies.iter_mut().find(|m| m.parsed.identity_key() == result.movie_id) {
                let movie_id = movie.parsed.identity_key();
                movie.rating_kinopoisk = result.rating;
                eprintln!("🔵 Rating Manager: Sending update for Kinopoisk result: {}", movie_id);
                // Send update immediately when Kinopoisk rating is ready
                self.send_rating_update(&rating_tx, movie_id, movie.rating_kinopoisk, movie.rating_imdb, movie.rating_tmdb);
                if result.rating.is_none() {
                    warn!("Rating Manager: Movie \"{}\" missing Kinopoisk rating", movie.parsed.title);
                }
            } else {
                eprintln!("🔵 Rating Manager: ⚠️  Could not find movie with ID: {}", result.movie_id);
            }
        }

        // Apply IMDb ratings (from cache and worker) and send updates
        eprintln!("🔵 Rating Manager: 📤 Sending IMDb rating updates to UI...");
        eprintln!("🔵 Rating Manager: IMDb from cache: {} movies", imdb_from_cache.len());
        eprintln!("🔵 Rating Manager: IMDb results: {} movies", imdb_results.len());
        info!("Rating Manager: 📤 Sending IMDb rating updates to UI...");
        for (movie_id, cached_result) in imdb_from_cache {
            if let Some(movie) = updated_movies.iter_mut().find(|m| m.parsed.identity_key() == movie_id) {
                movie.rating_imdb = cached_result.rating;
                eprintln!("🔵 Rating Manager: Sending update for cached IMDb: {}", movie_id);
                // Send update with current ratings
                self.send_rating_update(&rating_tx, movie_id, movie.rating_kinopoisk, movie.rating_imdb, movie.rating_tmdb);
            } else {
                eprintln!("🔵 Rating Manager: ⚠️  Could not find movie with ID: {}", movie_id);
            }
        }
        for result in imdb_results {
            // For IMDb results, result.movie_id is actually the movie's identity_key (not imdb_id)
            // This is because spawn_imdb_worker receives (movie_id, imdb_id) and uses movie_id
            if let Some(movie) = updated_movies.iter_mut().find(|m| m.parsed.identity_key() == result.movie_id) {
                let movie_id = movie.parsed.identity_key();
                movie.rating_imdb = result.rating;
                eprintln!("🔵 Rating Manager: Sending update for IMDb result: {} (rating: {:?})", movie_id, result.rating);
                // Send update immediately when IMDb rating is ready
                self.send_rating_update(&rating_tx, movie_id, movie.rating_kinopoisk, movie.rating_imdb, movie.rating_tmdb);
                if result.rating.is_none() {
                    warn!("Rating Manager: Movie \"{}\" missing IMDB rating", movie.parsed.title);
                }
            } else {
                eprintln!("🔵 Rating Manager: ⚠️  Could not find movie with identity_key: {}", result.movie_id);
            }
        }

        // Apply TMDB metadata (episodes, seasons, imdb_id)
        for (movie_id, metadata) in all_tmdb_metadata {
            if let Some(movie) = updated_movies.iter_mut().find(|m| m.parsed.identity_key() == movie_id) {
                if movie.imdb_id.is_none() && metadata.imdb_id.is_some() {
                    movie.imdb_id = metadata.imdb_id;
                }
                // TODO: Store episodes and seasons in EnrichedMedia if we add those fields
            }
        }

        let updated_count = updated_movies.iter()
            .filter(|m| m.rating_kinopoisk.is_some() || m.rating_imdb.is_some())
            .count();
        info!("Rating Manager: ✅ FINAL RESULT: Updated {} movies with new ratings and metadata", updated_count);
        info!("Rating Manager: 📊 Rating breakdown:");
        let kinopoisk_count = updated_movies.iter().filter(|m| m.rating_kinopoisk.is_some()).count();
        let imdb_count = updated_movies.iter().filter(|m| m.rating_imdb.is_some()).count();
        let tmdb_count = updated_movies.iter().filter(|m| m.rating_tmdb.is_some()).count();
        info!("Rating Manager:   - Kinopoisk: {}/{}", kinopoisk_count, updated_movies.len());
        info!("Rating Manager:   - IMDb: {}/{}", imdb_count, updated_movies.len());
        info!("Rating Manager:   - TMDB: {}/{}", tmdb_count, updated_movies.len());
        info!("═══════════════════════════════════════════════════════════");
        info!("Rating Manager: check_and_fetch_ratings() COMPLETED");

        Ok(updated_movies)
    }

    /// Check Kinopoisk cache and return (cached_movies, movies_to_fetch)
    async fn check_kinopoisk_cache(&self, movies: &[(usize, EnrichedMedia)]) -> (Vec<(usize, EnrichedMedia)>, Vec<(usize, EnrichedMedia)>) {
        let mut cached = Vec::new();
        let mut to_fetch = Vec::new();

        for (idx, movie) in movies {
            let movie_id = movie.parsed.identity_key();
            let mut found = false;

            // Check in-memory cache first
            let cache_key = RatingCache::kinopoisk_cache_key(
                movie.kinopoisk_id,
                &movie.parsed.title,
                movie.parsed.year,
            );

            // Check in-memory cache first - use cached result even if rating is None
            // (None means we already checked and the movie doesn't have a rating)
            if let Some(cached_rating) = self.cache.get_kinopoisk_rating(&cache_key).await {
                let mut updated_movie = movie.clone();
                updated_movie.rating_kinopoisk = cached_rating.rating;
                cached.push((*idx, updated_movie));
                found = true;
            } else if let Some(ref db_cache) = self.db_cache {
                // Check database cache - use cached result even if rating is None
                // (None means we already checked and the movie doesn't have a rating)
                // Note: We can't perfectly distinguish "never checked" from "checked and got None" in DB,
                // but if a record exists, we'll trust it (the record wouldn't exist unless we processed it)
                if let Ok(Some(db_record)) = db_cache.get_rating(&movie_id).await {
                    let mut updated_movie = movie.clone();
                    updated_movie.rating_kinopoisk = db_record.rating_kinopoisk;
                    // Also update in-memory cache for faster access
                    let cached_rating = CachedRating {
                        rating: db_record.rating_kinopoisk,
                        cached_at: db_record.cached_at,
                    };
                    self.cache.set_kinopoisk_rating(cache_key, cached_rating).await;
                    cached.push((*idx, updated_movie));
                    found = true;
                }
            }

            if !found {
                to_fetch.push((*idx, movie.clone()));
            }
        }

        (cached, to_fetch)
    }

    /// Check TMDB metadata cache and return (cached_metadata, movies_to_fetch)
    async fn check_tmdb_metadata_cache(&self, movies: &[(usize, EnrichedMedia)]) -> (HashMap<String, TMDBMetadataResult>, Vec<(usize, EnrichedMedia)>) {
        let mut cached = HashMap::new();
        let mut to_fetch = Vec::new();

        for (idx, movie) in movies {
            let tmdb_id_str = movie.tmdb_id.map(|id| id.to_string());
            let cache_key = RatingCache::tmdb_metadata_cache_key(
                tmdb_id_str.as_deref(),
                &movie.parsed.title,
                movie.parsed.year,
            );

            if let Some(cached_metadata) = self.cache.get_tmdb_metadata(&cache_key).await {
                let movie_id = movie.parsed.identity_key();
                let result = TMDBMetadataResult {
                    movie_id: movie_id.clone(),
                    tmdb_id: movie.tmdb_id.map(|id| id.to_string()),
                    imdb_id: cached_metadata.imdb_id,
                    episodes: cached_metadata.episodes,
                    seasons: cached_metadata.seasons,
                    fetched_at: cached_metadata.cached_at,
                };
                cached.insert(movie_id, result);
            } else {
                to_fetch.push((*idx, movie.clone()));
            }
        }

        (cached, to_fetch)
    }

    /// Check IMDb cache and return (cached_movies, movies_to_fetch)
    async fn check_imdb_cache(&self, tmdb_metadata: &HashMap<String, TMDBMetadataResult>) -> (HashMap<String, RatingWorkerResult>, Vec<(String, String)>) {
        let mut cached = HashMap::new();
        let mut to_fetch = Vec::new();

        eprintln!("🔵 Rating Manager: check_imdb_cache: Checking {} movies", tmdb_metadata.len());
        
        for (movie_id, metadata) in tmdb_metadata {
            if let Some(imdb_id) = &metadata.imdb_id {
                let mut found = false;

                // Check in-memory cache first - use cached result even if rating is None
                // (None means we already checked and the movie doesn't have a rating)
                if let Some(cached_rating) = self.cache.get_imdb_rating(imdb_id).await {
                    eprintln!("🔵 Rating Manager: Found in-memory cache: movie_id={}, imdb_id={}, rating={:?}", 
                             movie_id, imdb_id, cached_rating.rating);
                    cached.insert(movie_id.clone(), RatingWorkerResult {
                        movie_id: movie_id.clone(),
                        rating: cached_rating.rating,
                        source: RatingSource::Imdb,
                        fetched_at: cached_rating.cached_at,
                    });
                    found = true;
                } else {
                    eprintln!("🔵 Rating Manager: Not in in-memory cache: movie_id={}, imdb_id={}", movie_id, imdb_id);
                }

                // Check database cache if not found in memory - use cached result even if rating is None
                // (None means we already checked and the movie doesn't have a rating)
                // Note: We can't perfectly distinguish "never checked" from "checked and got None" in DB.
                // We use a heuristic: only trust the database cache if the record has at least one rating set
                // (meaning we've processed this movie) OR if the record was recently updated (within last hour).
                // This prevents us from using stale "never checked" records.
                if !found {
                    if let Some(ref db_cache) = self.db_cache {
                        if let Ok(Some(db_record)) = db_cache.get_rating(movie_id).await {
                            // Check if record indicates we've processed this movie:
                            // 1. Has at least one rating set (any rating means we processed it)
                            // 2. OR was updated recently (within last hour, meaning we just processed it)
                            let has_any_rating = db_record.rating_kinopoisk.is_some() 
                                || db_record.rating_imdb.is_some() 
                                || db_record.rating_tmdb.is_some();
                            let recently_updated = (Utc::now() - db_record.updated_at).num_seconds() < 3600;
                            
                            if has_any_rating || recently_updated {
                                // Record indicates we've processed this movie - trust the IMDb rating (even if None)
                                eprintln!("🔵 Rating Manager: Found in DB cache: movie_id={}, imdb_id={}, rating={:?}, has_any_rating={}, recently_updated={}", 
                                         movie_id, imdb_id, db_record.rating_imdb, has_any_rating, recently_updated);
                                // Update in-memory cache for faster access
                                let cached_rating = CachedRating {
                                    rating: db_record.rating_imdb,
                                    cached_at: db_record.cached_at,
                                };
                                self.cache.set_imdb_rating(imdb_id.clone(), cached_rating).await;
                                
                                cached.insert(movie_id.clone(), RatingWorkerResult {
                                    movie_id: movie_id.clone(),
                                    rating: db_record.rating_imdb,
                                    source: RatingSource::Imdb,
                                    fetched_at: db_record.cached_at,
                                });
                                found = true;
                            } else {
                                // Record exists but looks stale (no ratings, not recently updated)
                                // Don't use it - we'll fetch fresh data
                                eprintln!("🔵 Rating Manager: Ignoring stale DB record: movie_id={}, imdb_id={} (no ratings, not recently updated)", 
                                         movie_id, imdb_id);
                                debug!("Rating Manager: Ignoring stale database record for movie_id={} (no ratings, not recently updated)", movie_id);
                            }
                        }
                    }
                }

                if !found {
                    eprintln!("🔵 Rating Manager: Will fetch: movie_id={}, imdb_id={}", movie_id, imdb_id);
                    to_fetch.push((movie_id.clone(), imdb_id.clone()));
                }
            } else {
                eprintln!("🔵 Rating Manager: Movie has no IMDb ID in metadata: movie_id={}", movie_id);
            }
        }

        eprintln!("🔵 Rating Manager: check_imdb_cache result: {} cached, {} to fetch", cached.len(), to_fetch.len());
        (cached, to_fetch)
    }

    /// Spawn Kinopoisk worker
    fn spawn_kinopoisk_worker(&self, movies: Vec<(usize, EnrichedMedia)>) -> tokio::task::JoinHandle<Result<Vec<RatingWorkerResult>>> {
        let client = match &self.kinopoisk_client {
            Some(client) => Arc::clone(client),
            None => {
                // Return a task that immediately returns empty results if Kinopoisk is not available
                return tokio::spawn(async move {
                    warn!("Kinopoisk client not available, skipping Kinopoisk ratings");
                    Ok(Vec::new())
                });
            }
        };
        
        tokio::spawn(async move {
            info!("🟡 Kinopoisk Worker: 🚀 STARTING, processing {} movies", movies.len());
            eprintln!("🟡 Kinopoisk Worker: Starting, processing {} movies", movies.len());
            let start_time = std::time::Instant::now();
            let mut results = Vec::new();

            for (idx, (_orig_idx, movie)) in movies.iter().enumerate() {
                eprintln!("🟡 Kinopoisk Worker: Processing movie {}: '{}' ({:?})", idx + 1, movie.parsed.title, movie.parsed.year);
                
                let title_info = RatingTitleInfo {
                    cleaned_title: movie.parsed.title.clone(),
                    year: movie.parsed.year,
                    is_show: matches!(movie.parsed.media_type, crate::models::MediaType::Series),
                    season: movie.parsed.season,
                };

                eprintln!("🟡 Kinopoisk Worker: Fetching rating for '{}' (year: {:?}, is_show: {})", 
                         title_info.cleaned_title, title_info.year, title_info.is_show);

                match client.fetch_rating(&title_info).await {
                    Ok(rating) => {
                        eprintln!("🟡 Kinopoisk Worker: Got rating for '{}': {:?}", movie.parsed.title, rating);
                        results.push(RatingWorkerResult {
                            movie_id: movie.parsed.identity_key(),
                            rating,
                            source: RatingSource::Kinopoisk,
                            fetched_at: Utc::now(),
                        });
                    }
                    Err(e) => {
                        eprintln!("🔴 Kinopoisk Worker: Error fetching rating for '{}': {}", movie.parsed.title, e);
                        eprintln!("🔴 Kinopoisk Worker: Error details - {:?}", e);
                        eprintln!("🔴 Kinopoisk Worker: Error chain:");
                        let mut current = e.source();
                        let mut depth = 0;
                        while let Some(err) = current {
                            eprintln!("🔴   {}: {}", depth, err);
                            current = err.source();
                            depth += 1;
                            if depth > 5 { break; }
                        }
                        warn!("Kinopoisk Worker: Error fetching rating for '{}': {}", movie.parsed.title, e);
                        results.push(RatingWorkerResult {
                            movie_id: movie.parsed.identity_key(),
                            rating: None,
                            source: RatingSource::Kinopoisk,
                            fetched_at: Utc::now(),
                        });
                    }
                }
            }

            let elapsed = start_time.elapsed();
            info!("🟡 Kinopoisk Worker: ✅ COMPLETED in {:.2}s, returning {} results", elapsed.as_secs_f64(), results.len());
            eprintln!("🟡 Kinopoisk Worker: Completed, returning {} results", results.len());
            Ok(results)
        })
    }

    /// Spawn TMDB metadata worker (Worker 1)
    fn spawn_tmdb_metadata_worker(&self, movies: Vec<(usize, EnrichedMedia)>) -> tokio::task::JoinHandle<Result<Vec<TMDBMetadataResult>>> {
        let client = Arc::clone(&self.tmdb_client);
        
        tokio::spawn(async move {
            info!("🟡 TMDB Metadata Worker: 🚀 STARTING, processing {} movies", movies.len());
            eprintln!("🟡 TMDB Metadata Worker: Starting, processing {} movies", movies.len());
            info!("🟡 TMDB Metadata Worker: ⚠️  This worker BLOCKS IMDb worker from starting");
            let start_time = std::time::Instant::now();
            let mut results = Vec::new();

            for (idx, (_orig_idx, movie)) in movies.iter().enumerate() {
                eprintln!("🟡 TMDB Metadata Worker: Processing movie {}: '{}' ({:?})", idx + 1, movie.parsed.title, movie.parsed.year);
                
                // Use TMDB client to get metadata
                // For now, we'll search by title and get the first result's details
                use crate::metadata::MetadataMediaType;
                let media_type = match movie.parsed.media_type {
                    crate::models::MediaType::Movie => MetadataMediaType::Movie,
                    crate::models::MediaType::Series => MetadataMediaType::Series,
                    _ => MetadataMediaType::Movie,
                };

                eprintln!("🟡 TMDB Metadata Worker: Searching TMDB for '{}' (year: {:?}, type: {:?})", 
                         movie.parsed.title, movie.parsed.year, media_type);

                match client.search(&movie.parsed.title, movie.parsed.year, media_type.clone()).await {
                    Ok(search_results) => {
                        eprintln!("🟡 TMDB Metadata Worker: Search returned {} results for '{}'", 
                                 search_results.len(), movie.parsed.title);
                        if let Some(first_result) = search_results.first() {
                            eprintln!("🟡 TMDB Metadata Worker: Getting details for TMDB ID: {}", first_result.id);
                            match client.get_details(&first_result.id, media_type.clone()).await {
                                Ok(Some(metadata)) => {
                                    // Extract episodes and seasons from metadata
                                    // Note: TMDB client returns this in the metadata, but we need to extract it
                                    // For now, we'll just get the external_ids
                                    let tmdb_id = metadata.external_ids.tmdb_id.clone();
                                    let imdb_id = metadata.external_ids.imdb_id.clone();
                                    
                                    eprintln!("🟡 TMDB Metadata Worker: Got metadata for '{}' - TMDB ID: {:?}, IMDB ID: {:?}", 
                                             movie.parsed.title, tmdb_id, imdb_id);
                                    
                                    results.push(TMDBMetadataResult {
                                        movie_id: movie.parsed.identity_key(),
                                        tmdb_id,
                                        imdb_id,
                                        episodes: None, // TODO: Extract from TV show details
                                        seasons: None, // TODO: Extract from TV show details
                                        fetched_at: Utc::now(),
                                    });
                                }
                                Ok(None) => {
                                    eprintln!("🔴 TMDB Metadata Worker: No details found for '{}'", movie.parsed.title);
                                    debug!("TMDB Metadata Worker: No details found for '{}'", movie.parsed.title);
                                }
                                Err(e) => {
                                    eprintln!("🔴 TMDB Metadata Worker: Error getting details for '{}': {}", movie.parsed.title, e);
                                    warn!("TMDB Metadata Worker: Error getting details for '{}': {}", movie.parsed.title, e);
                                }
                            }
                        } else {
                            eprintln!("🔴 TMDB Metadata Worker: No search results for '{}'", movie.parsed.title);
                        }
                    }
                    Err(e) => {
                        eprintln!("🔴 TMDB Metadata Worker: Error searching for '{}': {}", movie.parsed.title, e);
                        eprintln!("🔴 TMDB Metadata Worker: Error details - {:?}", e);
                        eprintln!("🔴 TMDB Metadata Worker: Error chain:");
                        let mut current = e.source();
                        let mut depth = 0;
                        while let Some(err) = current {
                            eprintln!("🔴   {}: {}", depth, err);
                            current = err.source();
                            depth += 1;
                            if depth > 5 { break; }
                        }
                        warn!("TMDB Metadata Worker: Error searching for '{}': {}", movie.parsed.title, e);
                    }
                }
            }

            let elapsed = start_time.elapsed();
            info!("🟡 TMDB Metadata Worker: ✅ COMPLETED in {:.2}s, returning {} results", elapsed.as_secs_f64(), results.len());
            eprintln!("🟡 TMDB Metadata Worker: Completed, returning {} results", results.len());
            Ok(results)
        })
    }

    /// Spawn IMDb scraper worker (Worker 2)
    fn spawn_imdb_worker(&self, movies: Vec<(String, String)>) -> tokio::task::JoinHandle<Result<Vec<RatingWorkerResult>>> {
        let client = Arc::clone(&self.imdb_client);
        
        tokio::spawn(async move {
            info!("🟡 IMDb Scraper Worker: 🚀 STARTING, processing {} movies in parallel", movies.len());
            eprintln!("🟡 IMDb Scraper Worker: Starting, processing {} movies in parallel", movies.len());
            let start_time = std::time::Instant::now();
            
            // Process all requests in parallel with limited concurrency (max 10 concurrent)
            use futures::stream::{self, StreamExt};
            let results: Vec<RatingWorkerResult> = stream::iter(movies.into_iter())
                .map(|(movie_id, imdb_id)| {
                    let client = Arc::clone(&client);
                    
                    async move {
                        eprintln!("🟡 IMDb Scraper Worker: Processing movie_id='{}', imdb_id='{}'", 
                                 movie_id, imdb_id);
                        
                        match client.get_rating(&imdb_id).await {
                            Ok(rating) => {
                                eprintln!("🟡 IMDb Scraper Worker: Got rating for '{}': {:?}", imdb_id, rating);
                                RatingWorkerResult {
                                    movie_id,
                                    rating,
                                    source: RatingSource::Imdb,
                                    fetched_at: Utc::now(),
                                }
                            }
                            Err(e) => {
                                eprintln!("🔴 IMDb Scraper Worker: Error fetching rating for '{}': {}", imdb_id, e);
                                warn!("IMDb Scraper Worker: Error fetching rating for '{}': {}", imdb_id, e);
                                RatingWorkerResult {
                                    movie_id,
                                    rating: None,
                                    source: RatingSource::Imdb,
                                    fetched_at: Utc::now(),
                                }
                            }
                        }
                    }
                })
                .buffer_unordered(10) // Process up to 10 requests concurrently
                .collect()
                .await;
            
            let elapsed = start_time.elapsed();
            info!("🟡 IMDb Scraper Worker: ✅ COMPLETED in {:.2}s, returning {} results", elapsed.as_secs_f64(), results.len());
            eprintln!("🟡 IMDb Scraper Worker: Completed, returning {} results", results.len());
            Ok(results)
        })
    }

    /// Clear all IMDb ratings from cache
    pub async fn clear_imdb_cache(&self) -> Result<usize> {
        // Clear in-memory cache
        // Note: We can't easily clear specific IMDb entries from in-memory cache
        // without iterating, so we'll just clear the database cache
        // The in-memory cache will be repopulated on next fetch
        
        if let Some(ref db_cache) = self.db_cache {
            db_cache.clear_imdb_ratings().await
        } else {
            warn!("Rating Manager: No database cache available to clear");
            Ok(0)
        }
    }

    /// Clear all ratings from cache
    pub async fn clear_all_cache(&self) -> Result<usize> {
        // Clear in-memory cache
        // Note: We can't easily clear the in-memory cache without exposing internal structure
        // The in-memory cache will be repopulated on next fetch
        
        if let Some(ref db_cache) = self.db_cache {
            db_cache.clear_all_ratings().await
        } else {
            warn!("Rating Manager: No database cache available to clear");
            Ok(0)
        }
    }
}

