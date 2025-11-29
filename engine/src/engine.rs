use std::sync::Arc;
use anyhow::Result;
use tracing::{debug, info, warn, error};

use crate::indexers::{IndexerManager, IndexerRegistry, MonnaIndexer};
use crate::indexers::traits::SearchQuery;
use crate::models::MediaType;
use crate::normalize::Normalizer;
use crate::models::{TorrentResult, MediaSearchResult, EnrichedMedia};
use crate::metadata::{MetadataManager, TmdbClient};
use crate::scoring::{MediaScorer, ScoreWeights};

/// Core engine orchestrator that coordinates all components
pub struct Engine {
    indexer_manager: Arc<IndexerManager>,
    normalizer: Normalizer,
    metadata_manager: MetadataManager,
    scorer: MediaScorer,
}

impl Engine {
    pub fn new() -> Self {
        let registry = Arc::new(IndexerRegistry::new());
        let indexer_manager = Arc::new(IndexerManager::new(registry.clone()));
        let normalizer = Normalizer::default();

        // Initialize metadata manager with TMDb client
        let tmdb_client = Box::new(TmdbClient::new(
            std::env::var("TMDB_API_KEY").ok()
        ));
        let metadata_manager = MetadataManager::new()
            .set_primary(tmdb_client);

        // Initialize scorer with default weights
        let scorer = MediaScorer::with_default_weights();

        Self {
            indexer_manager,
            normalizer,
            metadata_manager,
            scorer,
        }
    }

    pub fn with_api_key(api_key: String) -> Self {
        let registry = Arc::new(IndexerRegistry::new());
        let indexer_manager = Arc::new(IndexerManager::new(registry.clone()));
        let normalizer = Normalizer::default();

        // Initialize metadata manager with TMDb client using provided API key
        let tmdb_client = Box::new(TmdbClient::new(Some(api_key)));
        let metadata_manager = MetadataManager::new()
            .set_primary(tmdb_client);

        // Initialize scorer with default weights
        let scorer = MediaScorer::with_default_weights();

        Self {
            indexer_manager,
            normalizer,
            metadata_manager,
            scorer,
        }
    }

    pub fn with_custom_scorer(mut self, scorer: MediaScorer) -> Self {
        self.scorer = scorer;
        self
    }

    /// Initialize the engine with default indexers
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing MovieDownloader engine");

        // Register default indexers
        self.register_default_indexers().await?;

        // Test connectivity
        let stats = self.indexer_manager.get_stats().await?;
        info!("Engine initialized: {} total indexers, {} enabled, {} connected", 
              stats.total_indexers, stats.enabled_indexers, stats.connected_indexers);

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
        eprintln!("🎬 Engine: get_feed() called");
        info!("🎬 Engine: Fetching movie feed from primary indexer");
        
        // Get the primary indexer (Monna2)
        let indexers = self.indexer_manager.registry().get_all()?;
        if let Some(primary_indexer) = indexers.first() {
            eprintln!("🔍 Engine: Found primary indexer: {}", primary_indexer.name());
            info!("🔍 Engine: Using primary indexer: {}", primary_indexer.name());
            
            // Try to get feed from primary indexer
            let feed_results = if primary_indexer.name() == "monna" {
                eprintln!("🎬 Engine: Calling Monna2 get_feed() method");
                info!("🎬 Engine: Calling Monna2 get_feed() method");
                primary_indexer.get_feed().await?
            } else {
                eprintln!("⚠️ Engine: Non-Monna indexer, falling back to search");
                info!("⚠️ Engine: Non-Monna indexer, falling back to search");
                let search_query = SearchQuery::new("2024".to_string())
                    .with_max_results(20);
                
                self.indexer_manager.search_all(&search_query).await?
            };
            
            eprintln!("📊 Engine: Got {} raw results from indexer", feed_results.len());
            info!("📊 Engine: Got {} raw results from indexer", feed_results.len());
            
            // Normalize the feed results
            let mut normalized_results = Vec::new();
            
            for torrent_result in feed_results {
                eprintln!("🔄 Engine: Normalizing: {}", torrent_result.title);
                info!("🔄 Engine: Normalizing: {}", torrent_result.title);
                match self.normalizer.normalize(&torrent_result) {
                    Ok(parsed_media) => {
                        eprintln!("✅ Engine: Normalized successfully: {} (year: {:?})", parsed_media.title, parsed_media.year);
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
                            eprintln!("👥 Engine: Preserving {} cast members in EnrichedMedia", enriched.cast.len());
                        }
                        if let Some(runtime) = torrent_result.runtime_minutes {
                            enriched.runtime_minutes = Some(runtime);
                        }
                        if !torrent_result.genres.is_empty() {
                            enriched.genres = torrent_result.genres.clone();
                            eprintln!("🎭 Engine: Preserving {} genres in EnrichedMedia", enriched.genres.len());
                        }
                        
                        // Create media search result
                        let mut media_result = MediaSearchResult::new(enriched);
                        media_result.torrent_results = vec![torrent_result.clone()];
                        media_result.best_quality = Some(torrent_result);
                        
                        normalized_results.push(media_result);
                    }
                    Err(e) => {
                        eprintln!("❌ Engine: Failed to normalize '{}': {}", torrent_result.title, e);
                        warn!("❌ Engine: Failed to normalize '{}': {}", torrent_result.title, e);
                        warn!("Failed to normalize feed item '{}': {}", torrent_result.title, e);
                    }
                }
            }
            
            eprintln!("🎉 Engine: Successfully normalized {} movies for UI", normalized_results.len());
            info!("Feed normalization completed: {} results", normalized_results.len());
            Ok(normalized_results)
        } else {
            eprintln!("❌ Engine: No indexers available");
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

        Ok(normalized_results)
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
        Ok(scored_results)
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
        Ok(scored_results)
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
        let engine = Engine::new();
        assert!(engine.initialize().await.is_ok());
    }

    #[tokio::test]
    async fn test_search() {
        let engine = Engine::new();
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
        let engine = Engine::new();
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
        let engine = Engine::new();
        engine.initialize().await.unwrap();

        let results = engine.test_components().await.unwrap();
        
        // Should have at least one working indexer (Monna2)
        assert!(results.working_indexers > 0);
        assert!(results.normalizer_working);
    }
}
