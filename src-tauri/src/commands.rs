use tauri::{State, Manager};
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

/// Get torrent magnet link for download
#[tauri::command]
pub async fn get_magnet_link(
    result_id: String,
    engine: State<'_, Engine>,
) -> Result<String, String> {
    debug!("UI magnet request for: {}", result_id);
    
    // For now, we'll need to implement a way to retrieve the magnet link
    // This is a placeholder - we'll need to add this functionality to the engine
    Err("Magnet link retrieval not implemented yet".to_string())
}

/// Get engine health/status
#[tauri::command]
pub async fn get_engine_status(
    engine: State<'_, Engine>,
) -> Result<serde_json::Value, String> {
    debug!("UI status request");
    
    let status = serde_json::json!({
        "engine": "healthy",
        "indexers": ["yts"],
        "metadata": ["tmdb"],
        "scoring": "enabled",
        "version": "0.1.0"
    });
    
    Ok(status)
}
