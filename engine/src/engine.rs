use std::sync::Arc;
use anyhow::Result;
use tracing::{debug, info, warn, error};
use tokio::sync::mpsc;

use crate::indexers::{IndexerManager, IndexerRegistry, MonnaIndexer};
use crate::indexers::traits::SearchQuery;
use crate::models::MediaType;
use crate::normalize::Normalizer;
use crate::models::{TorrentResult, MediaSearchResult, EnrichedMedia};
use crate::metadata::{MetadataManager, TmdbClient};
use crate::scoring::{MediaScorer, ScoreWeights};
use crate::jobs::JobManager;
use crate::database::Database;
use crate::ratings::RatingManager;

/// Rating update message sent to UI
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RatingUpdate {
    pub movie_id: String,
    pub rating_kinopoisk: Option<f32>,
    pub rating_imdb: Option<f32>,
    pub rating_tmdb: Option<f32>,
}

/// Core engine orchestrator that coordinates all components
pub struct Engine {
    indexer_manager: Arc<IndexerManager>,
    normalizer: Normalizer,
    metadata_manager: MetadataManager,
    scorer: MediaScorer,
    job_manager: Option<Arc<JobManager>>,
    rating_manager: Option<Arc<crate::ratings::RatingManager>>,
    rating_update_tx: Option<mpsc::UnboundedSender<RatingUpdate>>,
}

impl Engine {
    pub fn new() -> Self {
        let registry = Arc::new(IndexerRegistry::new());
        let indexer_manager = Arc::new(IndexerManager::new(registry.clone()));
        let normalizer = Normalizer::default();

        // Initialize metadata manager with TMDb client
        let metadata_manager = MetadataManager::new()
            .set_primary(Box::new(TmdbClient::new(
                std::env::var("TMDB_API_KEY").ok()
            )));

        // Initialize scorer with default weights
        let scorer = MediaScorer::with_default_weights();

        Self {
            indexer_manager,
            normalizer,
            metadata_manager,
            scorer,
            job_manager: None, // Will be initialized in initialize() method
            rating_manager: None, // Will be set when RatingManager is created
            rating_update_tx: None,
        }
    }

    pub fn with_api_key(api_key: String) -> Self {
        let registry = Arc::new(IndexerRegistry::new());
        let indexer_manager = Arc::new(IndexerManager::new(registry.clone()));
        let normalizer = Normalizer::default();

        // Initialize metadata manager with TMDb client using provided API key
        let metadata_manager = MetadataManager::new()
            .set_primary(Box::new(TmdbClient::new(Some(api_key))));

        // Initialize scorer with default weights
        let scorer = MediaScorer::with_default_weights();

        Self {
            indexer_manager,
            normalizer,
            metadata_manager,
            scorer,
            job_manager: None, // Will be initialized in initialize() method
            rating_manager: None, // Will be set when RatingManager is created
            rating_update_tx: None,
        }
    }

    pub fn with_custom_scorer(mut self, scorer: MediaScorer) -> Self {
        self.scorer = scorer;
        self
    }

    /// Set the channel for rating updates (used to notify UI when ratings are fetched)
    pub fn set_rating_update_channel(&mut self, tx: mpsc::UnboundedSender<RatingUpdate>) {
        self.rating_update_tx = Some(tx.clone());
        
        // Also set it on RatingManager if it exists
        // Try to get mutable reference (will only work if we have exclusive access)
        if let Some(ref mut rating_manager) = self.rating_manager {
            if let Some(rating_manager_mut) = Arc::get_mut(rating_manager) {
                rating_manager_mut.set_rating_update_channel(tx);
                info!("Engine: ✅ Set rating update channel on existing RatingManager");
            } else {
                warn!("Engine: RatingManager is shared, channel stored but not set on RatingManager yet");
                warn!("Engine: Channel will be used via stored reference in RatingManager");
            }
        } else {
            info!("Engine: RatingManager not created yet, channel will be set when RatingManager is initialized");
        }
    }

    /// Initialize the engine with default indexers and job manager
    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing MovieDownloader engine");

        // Register default indexers
        self.register_default_indexers().await?;

        // Initialize database and job manager
        self.initialize_job_manager().await?;

        // Test connectivity
        let stats = self.indexer_manager.get_stats().await?;
        info!("Engine initialized: {} total indexers, {} enabled, {} connected", 
              stats.total_indexers, stats.enabled_indexers, stats.connected_indexers);

        Ok(())
    }

    /// Initialize job manager with database and register worker managers
    async fn initialize_job_manager(&mut self) -> Result<()> {
        // Get database URL from environment or use default
        // sqlx expects sqlite:// format for SQLite connections
        // Store database outside src-tauri/ to avoid triggering rebuilds1231
        let database_url = match std::env::var("DATABASE_URL") {
            Ok(url) => {
                // Strip sqlite:// prefix if present for path checking
                let path_check = url.strip_prefix("sqlite://").unwrap_or(&url);
                
                // Check if it's an absolute root path (like /torrent_aggregator.db)
                // These require root permissions and are usually mistakes
                if path_check.starts_with("/") && !path_check.starts_with("/home") && !path_check.starts_with("/tmp") && path_check.len() > 1 {
                    warn!("DATABASE_URL is set to root filesystem path: {}", url);
                    warn!("This requires root permissions. Using default .data/torrentflix.db instead.");
                    // Fall through to default
                    None
                } else if url.starts_with("sqlite://") || url.starts_with("sqlite:") {
                    Some(url)
                } else {
                    // Relative or absolute path - convert to sqlite:// format
                    Some(format!("sqlite://{}", url))
                }
            }
            Err(_) => None,
        }
        .unwrap_or_else(|| {
            // Use a data directory outside the source tree
            let db_path = ".data/torrentflix.db";
            format!("sqlite://{}", db_path)
        });

        info!("Initializing database: {}", database_url);
        let database = Arc::new(Database::new(&database_url).await?);

        // Initialize rating cache database
        let rating_cache_db = Arc::new(crate::database::ratings::RatingCacheDatabase::new(database.clone()));
        rating_cache_db.initialize().await?;
        info!("Rating cache database initialized");

        info!("Initializing JobManager");
        let mut job_manager = JobManager::new(database.clone()).await?;

        // Initialize RatingManager with database cache and register it as a worker
        info!("Engine: Initializing RatingManager...");
        let tmdb_client = Arc::new(TmdbClient::new(
            std::env::var("TMDB_API_KEY").ok()
        ));
        
        info!("Engine: Creating RatingManager instance...");
        match RatingManager::new(tmdb_client) {
            Ok(rating_manager) => {
                info!("Engine: RatingManager created successfully");
                info!("Engine: Attaching database cache to RatingManager...");
            let rating_manager_with_db = rating_manager.with_db_cache(rating_cache_db);
            let mut rating_manager_arc = Arc::new(rating_manager_with_db);
            
                // Store reference to RatingManager so we can set the channel later
                self.rating_manager = Some(Arc::clone(&rating_manager_arc));
                
                // Set channel if it was already set (try to get mutable reference before cloning)
                // We need to set it before storing the clone, so we can get a mutable reference
                if let Some(ref tx) = self.rating_update_tx {
                    if let Some(rating_manager_mut) = Arc::get_mut(&mut rating_manager_arc) {
                        rating_manager_mut.set_rating_update_channel(tx.clone());
                        info!("Engine: ✅ Set rating update channel on RatingManager");
                    }
                }
                
                // Now store the clone (this creates another reference, so we can't mutate after this)
                self.rating_manager = Some(Arc::clone(&rating_manager_arc));
                
                info!("Engine: Registering RatingManager as worker in JobManager...");
            job_manager.register_rating_worker(rating_manager_arc);
                info!("Engine: ✅ Registered RatingManager as worker in JobManager (with database cache)");
            }
            Err(e) => {
                warn!("Engine: ❌ RatingManager initialization failed: {}", e);
                warn!("Engine: This may be due to missing API keys (KINOPOISK_API_KEY)");
                warn!("Engine: Rating fetching will not work until RatingManager is properly initialized");
            }
        }

        self.job_manager = Some(Arc::new(job_manager));
        info!("✅ Engine: JobManager initialized successfully");
        
        // Verify it's set
        if self.job_manager.is_some() {
            info!("✅ Engine: Verified job_manager is Some()");
        } else {
            warn!("❌ Engine: ERROR - job_manager is None after initialization!");
        }

        Ok(())
    }

    async fn register_default_indexers(&self) -> Result<()> {
        // Register Monna2 indexer as primary
        let monna_indexer = Arc::new(MonnaIndexer::new());
        
        if let Err(e) = self.indexer_manager.registry().register(monna_indexer) {
            warn!("Failed to register Monna2 indexer: {}", e);
        } else {
            info!("Registered Monna2 indexer as primary");
        }

        Ok(())
    }

    /// Get feed of recent movies from primary indexer
    pub async fn get_feed(&self) -> Result<Vec<MediaSearchResult>> {
        info!("🎬 Engine: Fetching movie feed from primary indexer");
        
        // Get the primary indexer (Monna2)
        let indexers = self.indexer_manager.registry().get_all()?;
        if let Some(primary_indexer) = indexers.first() {
            info!("🔍 Engine: Using primary indexer: {}", primary_indexer.name());
            
            // Try to get feed from primary indexer
            let feed_results = if primary_indexer.name() == "monna" {
                info!("🎬 Engine: Calling Monna2 get_feed() method");
                primary_indexer.get_feed().await?
            } else {
                info!("⚠️ Engine: Non-Monna indexer, falling back to search");
                let search_query = SearchQuery::new("2024".to_string())
                    .with_max_results(20);
                
                self.indexer_manager.search_all(&search_query).await?
            };
            
            info!("📊 Engine: Got {} raw results from indexer", feed_results.len());
            
            // Normalize the feed results
            let mut normalized_results = Vec::new();
            
            for torrent_result in feed_results {
                info!("🔄 Engine: Normalizing: {}", torrent_result.title);
                match self.normalizer.normalize(&torrent_result) {
                    Ok(parsed_media) => {
                        debug!("✅ Engine: Normalized successfully: {}", parsed_media.title);
                        // Create enriched media (without external metadata for feed)
                        let mut enriched = EnrichedMedia::new(parsed_media);
                        
                        // Preserve metadata from TorrentResult (poster_url, description, cast, runtime, genres)
                        if let Some(poster_url) = torrent_result.poster_url.clone() {
                            enriched.poster_url = Some(poster_url);
                        }
                        if let Some(description) = torrent_result.description.clone() {
                            enriched.overview = Some(description);
                        }
                        if !torrent_result.cast.is_empty() {
                            enriched.cast = torrent_result.cast.clone();
                        }
                        if let Some(runtime) = torrent_result.runtime_minutes {
                            enriched.runtime_minutes = Some(runtime);
                        }
                        if !torrent_result.genres.is_empty() {
                            enriched.genres = torrent_result.genres.clone();
                        }
                        
                        // Create media search result
                        let mut media_result = MediaSearchResult::new(enriched);
                        media_result.torrent_results = vec![torrent_result.clone()];
                        media_result.best_quality = Some(torrent_result);
                        
                        normalized_results.push(media_result);
                    }
                    Err(e) => {
                        warn!("❌ Engine: Failed to normalize '{}': {}", torrent_result.title, e);
                        warn!("Failed to normalize feed item '{}': {}", torrent_result.title, e);
                    }
                }
            }
            
            info!("Feed normalization completed: {} results", normalized_results.len());
            
            // Fetch ratings for all movies
            info!("⭐ Engine: About to call fetch_ratings_for_results() with {} normalized results", normalized_results.len());
            let results_with_ratings = self.fetch_ratings_for_results(normalized_results).await?;
            info!("⭐ Engine: fetch_ratings_for_results() returned {} results", results_with_ratings.len());
            Ok(results_with_ratings)
        } else {
            warn!("No indexers available for feed");
            Ok(Vec::new())
        }
    }

    /// Search for media across all indexers and normalize results
    pub async fn search(&self, query: &str) -> Result<Vec<MediaSearchResult>> {
        info!("Starting search for: {}", query);

        // Create search query
        let search_query = SearchQuery::new(query.to_string())
            .with_max_results(50);

        // Search across all indexers
        let torrent_results = self.indexer_manager.search_all(&search_query).await?;
        
        if torrent_results.is_empty() {
            info!("No torrent results found for: {}", query);
            return Ok(Vec::new());
        }

        info!("Found {} raw torrent results, normalizing...", torrent_results.len());

        // Normalize each torrent result
        let mut normalized_results = Vec::new();
        let mut normalization_errors = 0;

        for torrent_result in torrent_results {
            match self.normalizer.normalize(&torrent_result) {
                Ok(parsed_media) => {
                    debug!("Normalized: {} -> {}", torrent_result.title, parsed_media.title);
                    
                    // Create enriched media (without external metadata for now)
                    let enriched = EnrichedMedia::new(parsed_media);
                    
                    // Create media search result
                    let mut media_result = MediaSearchResult::new(enriched);
                    media_result.torrent_results = vec![torrent_result.clone()];
                    media_result.best_quality = Some(torrent_result);
                    
                    normalized_results.push(media_result);
                }
                Err(e) => {
                    warn!("Failed to normalize '{}': {}", torrent_result.title, e);
                    normalization_errors += 1;
                }
            }
        }

        info!("Normalization completed: {} successful, {} errors", 
              normalized_results.len(), normalization_errors);

        // TODO: Add deduplication and scoring here
        // For now, return all normalized results

        // Fetch ratings for all movies
        let results_with_ratings = self.fetch_ratings_for_results(normalized_results).await?;
        Ok(results_with_ratings)
    }

    /// Search for specific media type
    pub async fn search_by_type(&self, query: &str, media_type: MediaType) -> Result<Vec<MediaSearchResult>> {
        info!("Starting search for: {} (type: {:?})", query, media_type);

        let search_query = SearchQuery::new(query.to_string())
            .with_media_type(media_type.clone())
            .with_max_results(50);

        let torrent_results = self.indexer_manager.search_all(&search_query).await?;
        
        if torrent_results.is_empty() {
            return Ok(Vec::new());
        }

        let mut normalized_results = Vec::new();

        for torrent_result in torrent_results {
            if let Ok(parsed_media) = self.normalizer.normalize(&torrent_result) {
                // Only include results matching the requested media type
                if parsed_media.media_type == media_type {
                    // Enrich with metadata from TMDb
                    let mut enriched = self.enrich_with_metadata(parsed_media).await?;
                    
                    // Preserve metadata from TorrentResult (poster_url, description, cast, runtime, genres)
                    // Only override if TMDB didn't provide it
                    if enriched.poster_url.is_none() {
                        enriched.poster_url = torrent_result.poster_url.clone();
                    }
                    if enriched.overview.is_none() {
                        enriched.overview = torrent_result.description.clone();
                    }
                    if enriched.cast.is_empty() && !torrent_result.cast.is_empty() {
                        enriched.cast = torrent_result.cast.clone();
                    }
                    if enriched.runtime_minutes.is_none() {
                        enriched.runtime_minutes = torrent_result.runtime_minutes;
                    }
                    // Merge genres: prefer TMDB, but add indexer genres if not present
                    if enriched.genres.is_empty() && !torrent_result.genres.is_empty() {
                        enriched.genres = torrent_result.genres.clone();
                    } else if !torrent_result.genres.is_empty() {
                        // Merge unique genres from both sources
                        let mut merged = enriched.genres.clone();
                        for genre in &torrent_result.genres {
                            if !merged.contains(genre) {
                                merged.push(genre.clone());
                            }
                        }
                        enriched.genres = merged;
                    }
                    
                    let mut media_result = MediaSearchResult::new(enriched);
                    media_result.torrent_results = vec![torrent_result.clone()];
                    media_result.best_quality = Some(torrent_result);
                    normalized_results.push(media_result);
                }
            }
        }

        // Apply scoring and ranking
        let scored_results = self.scorer.score_results(normalized_results);

        info!("Returning {} scored results", scored_results.len());
        
        // Fetch ratings for all movies
        info!("search_by_type: Calling fetch_ratings_for_results()...");
        let results_with_ratings = self.fetch_ratings_for_results(scored_results).await?;
        Ok(results_with_ratings)
    }

    /// Search with custom scoring weights
    pub async fn search_with_scoring(&self, query: &str, media_type: MediaType, weights: ScoreWeights) -> Result<Vec<MediaSearchResult>> {
        info!("Starting scored search for: {} (type: {:?})", query, media_type);

        // Create custom scorer
        let custom_scorer = MediaScorer::new(weights);

        let search_query = SearchQuery::new(query.to_string())
            .with_media_type(media_type.clone())
            .with_max_results(50);

        let torrent_results = self.indexer_manager.search_all(&search_query).await?;
        
        if torrent_results.is_empty() {
            return Ok(Vec::new());
        }

        let mut normalized_results = Vec::new();

        for torrent_result in torrent_results {
            if let Ok(parsed_media) = self.normalizer.normalize(&torrent_result) {
                if parsed_media.media_type == media_type {
                    let enriched = self.enrich_with_metadata(parsed_media).await?;
                    let mut media_result = MediaSearchResult::new(enriched);
                    media_result.torrent_results = vec![torrent_result.clone()];
                    media_result.best_quality = Some(torrent_result);
                    normalized_results.push(media_result);
                }
            }
        }

        // Apply custom scoring and ranking
        let scored_results = custom_scorer.score_results(normalized_results);

        info!("Returning {} custom scored results", scored_results.len());
        
        // Fetch ratings for all movies
        let results_with_ratings = self.fetch_ratings_for_results(scored_results).await?;
        Ok(results_with_ratings)
    }


    /// Fetch ratings for movies using JobManager
    /// Returns results immediately (with cached ratings if available)
    /// Fetches missing ratings in the background
    async fn fetch_ratings_for_results(&self, results: Vec<MediaSearchResult>) -> Result<Vec<MediaSearchResult>> {
        info!("Engine: fetch_ratings_for_results() called with {} results", results.len());
        
        if let Some(job_manager) = &self.job_manager {
            // Extract all enriched media with their IDs for matching
            let media_with_ids: Vec<(String, EnrichedMedia)> = results.iter()
                .map(|r| (r.enriched.parsed.identity_key(), r.enriched.clone()))
                .collect();
            let enriched_media: Vec<EnrichedMedia> = media_with_ids.iter().map(|(_, m)| m.clone()).collect();
            
            // Log current rating status (from cache or existing)
            let with_kinopoisk = enriched_media.iter().filter(|m| m.rating_kinopoisk.is_some()).count();
            let with_imdb = enriched_media.iter().filter(|m| m.rating_imdb.is_some()).count();
            let with_tmdb = enriched_media.iter().filter(|m| m.rating_tmdb.is_some()).count();
            info!("Engine: Current rating status (cached) - Kinopoisk: {}/{}, IMDB: {}/{}, TMDB: {}/{}", 
                  with_kinopoisk, enriched_media.len(), with_imdb, enriched_media.len(), with_tmdb, enriched_media.len());
            
            // Spawn background task to fetch missing ratings (non-blocking)
            let job_manager_clone = Arc::clone(job_manager);
            let rating_tx = self.rating_update_tx.clone();
            tokio::spawn(async move {
                info!("Engine: Background task: Starting to fetch missing ratings for {} movies", enriched_media.len());
                match job_manager_clone.fetch_ratings_immediate(enriched_media).await {
                    Ok(updated_media) => {
                        info!("Engine: Background task: Fetched ratings for {} movies", updated_media.len());
                        
                        // Send rating updates to UI via channel
                        if let Some(tx) = rating_tx {
                            for updated in updated_media {
                                let update = RatingUpdate {
                                    movie_id: updated.parsed.identity_key(),
                                    rating_kinopoisk: updated.rating_kinopoisk,
                                    rating_imdb: updated.rating_imdb,
                                    rating_tmdb: updated.rating_tmdb,
                                };
                                if let Err(e) = tx.send(update) {
                                    warn!("Engine: Failed to send rating update to UI: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Engine: Background task: Failed to fetch ratings: {}", e);
                    }
                }
            });
            
            info!("Engine: Spawned background task for rating fetch, returning results immediately");
        } else {
            warn!("Engine: JobManager is NOT initialized, skipping rating fetch");
        }
        
        info!("Engine: fetch_ratings_for_results() returning {} results immediately", results.len());
        
        // Count current ratings (from cache)
        let mut with_kinopoisk = 0;
        let mut with_imdb = 0;
        let mut with_tmdb = 0;
        let mut no_ratings = 0;
        
        for result in &results {
            let kp = result.enriched.rating_kinopoisk.is_some();
            let imdb = result.enriched.rating_imdb.is_some();
            let tmdb = result.enriched.rating_tmdb.is_some();
            
            if kp { with_kinopoisk += 1; }
            if imdb { with_imdb += 1; }
            if tmdb { with_tmdb += 1; }
            if !kp && !imdb && !tmdb { 
                no_ratings += 1; 
            }
        }
        
        info!("📊 Engine: Returning results with ratings (cached) - Kinopoisk: {}/{}, IMDB: {}/{}, TMDB: {}/{}, No ratings: {}", 
              with_kinopoisk, results.len(), with_imdb, results.len(), with_tmdb, results.len(), no_ratings);
        
        Ok(results)
    }

    /// Enrich parsed media with metadata from TMDb
    async fn enrich_with_metadata(&self, parsed_media: crate::models::ParsedMedia) -> Result<EnrichedMedia> {
        // Convert media type for metadata search
        let metadata_type = match parsed_media.media_type {
            MediaType::Movie => crate::metadata::MetadataMediaType::Movie,
            MediaType::Series => crate::metadata::MetadataMediaType::Series,
            MediaType::Documentary => crate::metadata::MetadataMediaType::Movie, // Treat as movie for now
            MediaType::Anime => crate::metadata::MetadataMediaType::Movie, // Treat as movie for now
            MediaType::Other => crate::metadata::MetadataMediaType::Movie, // Default to movie
        };

        // Search for metadata using title and year
        let search_results = self.metadata_manager.search(
            &parsed_media.title, 
            parsed_media.year, 
            metadata_type.clone()
        ).await?;

        if let Some(best_match) = search_results.first() {
            // Get detailed metadata for the best match
            if let Ok(Some(metadata)) = self.metadata_manager.get_details(&best_match.id, metadata_type).await {
                let mut enriched = EnrichedMedia::new(parsed_media);
                // Map MediaMetadata fields to EnrichedMedia with correct field names
                enriched.tmdb_id = metadata.external_ids.tmdb_id.and_then(|s| s.parse::<u32>().ok());
                enriched.imdb_id = metadata.external_ids.imdb_id;
                enriched.overview = metadata.overview;
                enriched.poster_url = metadata.poster_url;
                enriched.backdrop_url = metadata.backdrop_url;
                enriched.genres = metadata.genres;
                enriched.rating_tmdb = metadata.vote_average;
                enriched.runtime_minutes = metadata.runtime;
                return Ok(enriched);
            }
        }

        // No metadata found, return enriched media without metadata
        Ok(EnrichedMedia::new(parsed_media))
    }

    /// Get engine statistics
    pub async fn get_stats(&self) -> Result<EngineStats> {
        let indexer_stats = self.indexer_manager.get_stats().await?;
        
        Ok(EngineStats {
            indexer_stats,
            total_searches: 0, // TODO: Implement search tracking
            successful_normalizations: 0, // TODO: Implement normalization tracking
        })
    }

    /// Test all components
    pub async fn test_components(&self) -> Result<ComponentTestResults> {
        info!("Testing engine components");

        // Test indexers
        let connectivity_results = self.indexer_manager.test_connectivity().await?;
        
        let mut working_indexers = 0;
        let mut failed_indexers = 0;

        for (name, is_connected) in &connectivity_results {
            if *is_connected {
                working_indexers += 1;
                info!("✓ {} - Connected", name);
            } else {
                failed_indexers += 1;
                error!("✗ {} - Failed to connect", name);
            }
        }

        // Test normalizer with sample data
        let sample_torrent = TorrentResult::new(
            "Inception.2010.1080p.BluRay.x264-SPARKS".to_string(),
            "magnet:?xt=urn:btih:test".to_string(),
            1_500_000_000,
            100,
            50,
            "Test".to_string(),
        );

        let normalizer_working = self.normalizer.normalize(&sample_torrent).is_ok();

        if normalizer_working {
            info!("✓ Normalizer - Working");
        } else {
            error!("✗ Normalizer - Failed");
        }

        Ok(ComponentTestResults {
            working_indexers,
            failed_indexers,
            normalizer_working,
        })
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct EngineStats {
    pub indexer_stats: crate::indexers::manager::IndexerStats,
    pub total_searches: u64,
    pub successful_normalizations: u64,
}

#[derive(Debug, Clone)]
pub struct ComponentTestResults {
    pub working_indexers: usize,
    pub failed_indexers: usize,
    pub normalizer_working: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_engine_creation() {
        let mut engine = Engine::new();
        assert!(engine.initialize().await.is_ok());
    }

    #[tokio::test]
    async fn test_search() {
        let mut engine = Engine::new();
        engine.initialize().await.unwrap();

        let results = engine.search("inception").await.unwrap();
        
        // Should find some results for "inception"
        assert!(!results.is_empty());
        
        for result in results {
            assert!(!result.enriched.parsed.title.is_empty());
            assert!(!result.torrent_results.is_empty());
            assert!(result.best_quality.is_some());
        }
    }

    #[tokio::test]
    async fn test_search_by_type() {
        let mut engine = Engine::new();
        engine.initialize().await.unwrap();

        let results = engine.search_by_type("inception", MediaType::Movie).await.unwrap();
        
        // Should find movie results for "inception"
        assert!(!results.is_empty());
        
        for result in results {
            assert_eq!(result.enriched.parsed.media_type, MediaType::Movie);
        }
    }

    #[tokio::test]
    async fn test_component_tests() {
        let mut engine = Engine::new();
        engine.initialize().await.unwrap();

        let results = engine.test_components().await.unwrap();
        
        // Should have at least one working indexer (Monna2)
        assert!(results.working_indexers > 0);
        assert!(results.normalizer_working);
    }
}
