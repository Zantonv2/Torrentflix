use tauri::State;
use anyhow::Result;
use tracing::{debug, info, error};

use engine::Engine;
use engine::ui::dto::{UiSearchRequest, UiSearchResponse, UiSearchResult};

/// Search for movies/series with Netflix-style results
#[tauri::command]
pub async fn search_movies(
    request: UiSearchRequest,
    engine: State<'_, Engine>,
) -> Result<UiSearchResponse, String> {
    debug!("UI search request: {:?}", request);
    
    let start_time = std::time::Instant::now();
    
    // Convert UI request to engine types
    let media_type = match request.media_type.as_str() {
        "movie" => engine::models::MediaType::Movie,
        "series" => engine::models::MediaType::Series,
        "all" | _ => engine::models::MediaType::Movie, // Default to movies for now
    };
    
    // Perform search through engine
    info!("═══════════════════════════════════════════════════════════");
    info!("🔍 Tauri: search_movies command called from UI");
    info!("🔍 Tauri: Query: '{}', Type: {:?}", request.query, media_type);
    info!("🔍 Tauri: Calling engine.search_by_type()...");
    
    match engine.search_by_type(&request.query, media_type).await {
        Ok(results) => {
            let took_ms = start_time.elapsed().as_millis() as u64;
            
            // Convert engine results to UI DTOs
            let ui_results: Vec<UiSearchResult> = results
                .into_iter()
                .filter_map(UiSearchResult::from_media_search_result)
                .take(request.limit.unwrap_or(50) as usize)
                .collect();
            
            info!("Search completed: {} results in {}ms", ui_results.len(), took_ms);
            
            let total_results = ui_results.len() as u32;
            
            Ok(UiSearchResponse {
                results: ui_results,
                total: total_results,
                took_ms,
            })
        }
        Err(e) => {
            error!("Search failed: {}", e);
            Err(format!("Search failed: {}", e))
        }
    }
}

/// Test command to verify IPC bridge works
#[tauri::command]
pub async fn test_connection() -> Result<String, String> {
    info!("🧪 Tauri: test_connection called");
    Ok("IPC bridge working!".to_string())
}

/// Get feed of recent movies
#[tauri::command]
pub async fn get_feed(
    engine: State<'_, Engine>,
) -> Result<UiSearchResponse, String> {
    info!("═══════════════════════════════════════════════════════════");
    info!("🎬 Tauri: get_feed command called from UI");
    info!("🎬 Tauri: Calling engine.get_feed()...");
    
    match engine.get_feed().await {
        Ok(results) => {
            info!("🎉 Tauri: Engine returned {} results", results.len());
            
            // Convert to UI DTOs
            let ui_results: Vec<UiSearchResult> = results
                .into_iter()
                .filter_map(UiSearchResult::from_media_search_result)
                .collect();
            
            info!("🔄 Tauri: Converted {} results to UI format", ui_results.len());
            
            for (i, result) in ui_results.iter().take(3).enumerate() {
                debug!("📽️ Tauri UI Result {}: '{}' - {}", i, result.title, result.quality_badge);
            }
            
            let total_count = ui_results.len() as u32;
            let response = UiSearchResponse {
                results: ui_results,
                total: total_count,
                took_ms: 0, // Would need to track timing
            };
            
            info!("✅ Tauri: Returning response with {} movies to UI", response.results.len());
            Ok(response)
        }
        Err(e) => {
            error!("❌ Tauri: Feed error: {}", e);
            Err(format!("Failed to fetch feed: {}", e))
        }
    }
}

/// Get torrent magnet link for download
#[tauri::command]
pub async fn get_magnet_link(
    result_id: String,
    _engine: State<'_, Engine>,
) -> Result<String, String> {
    debug!("UI magnet request for: {}", result_id);
    
    // For now, we'll need to implement a way to retrieve the magnet link
    // This is a placeholder - we'll need to add this functionality to the engine
    Err("Magnet link retrieval not implemented yet".to_string())
}

/// Get engine health/status
#[tauri::command]
pub async fn get_engine_status(
    _engine: State<'_, Engine>,
) -> Result<serde_json::Value, String> {
    debug!("UI status request");
    
    let status = serde_json::json!({
        "engine": "healthy",
        "indexers": ["monna2"],
        "metadata": ["tmdb"],
        "scoring": "enabled",
        "version": "0.1.0"
    });
    
    Ok(status)
}

/// Clear IMDb ratings from cache
#[tauri::command]
pub async fn clear_imdb_cache(
    engine: State<'_, Engine>,
) -> Result<usize, String> {
    info!("🧹 Tauri: clear_imdb_cache command called");
    
    match engine.clear_imdb_cache().await {
        Ok(count) => {
            info!("✅ Cleared {} IMDb ratings from cache", count);
            Ok(count)
        }
        Err(e) => {
            error!("❌ Failed to clear IMDb cache: {}", e);
            Err(format!("Failed to clear IMDb cache: {}", e))
        }
    }
}

/// Clear all ratings from cache
#[tauri::command]
pub async fn clear_all_ratings_cache(
    engine: State<'_, Engine>,
) -> Result<usize, String> {
    info!("🧹 Tauri: clear_all_ratings_cache command called");
    
    match engine.clear_all_ratings_cache().await {
        Ok(count) => {
            info!("✅ Cleared {} ratings from cache", count);
            Ok(count)
        }
        Err(e) => {
            error!("❌ Failed to clear ratings cache: {}", e);
            Err(format!("Failed to clear ratings cache: {}", e))
        }
    }
}
