use tauri::State;
use anyhow::Result;
use tracing::{debug, info, error, warn};

use engine::Engine;
use engine::ui::dto::{UiSearchRequest, UiSearchResponse, UiSearchResult};
use engine::settings::models::Settings;

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

/// Start downloading a torrent
#[tauri::command]
pub async fn start_download(
    magnet_link: String,
    title: String,
    engine: State<'_, Engine>,
) -> Result<String, String> {
    info!("📥 Tauri: start_download called for: {}", title);
    
    match engine.start_download(&magnet_link, &title).await {
        Ok(hash) => {
            info!("✅ Download started: {}", hash);
            Ok(hash)
        }
        Err(e) => {
            error!("❌ Failed to start download: {}", e);
            Err(format!("Failed to start download: {}", e))
        }
    }
}

/// Get all active downloads
#[tauri::command]
pub async fn get_active_downloads(
    engine: State<'_, Engine>,
) -> Result<Vec<engine::models::DownloadStatus>, String> {
    debug!("📊 Tauri: get_active_downloads called");
    
    match engine.get_active_downloads().await {
        Ok(downloads) => Ok(downloads),
        Err(e) => {
            error!("❌ Failed to get downloads: {}", e);
            Err(format!("Failed to get downloads: {}", e))
        }
    }
}

/// Pause a download
#[tauri::command]
pub async fn pause_download(
    hash: String,
    engine: State<'_, Engine>,
) -> Result<(), String> {
    info!("⏸️ Tauri: pause_download called for: {}", hash);
    
    match engine.pause_download(&hash).await {
        Ok(_) => {
            info!("✅ Download paused: {}", hash);
            Ok(())
        }
        Err(e) => {
            error!("❌ Failed to pause download: {}", e);
            Err(format!("Failed to pause download: {}", e))
        }
    }
}

/// Resume a download
#[tauri::command]
pub async fn resume_download(
    hash: String,
    engine: State<'_, Engine>,
) -> Result<(), String> {
    info!("▶️ Tauri: resume_download called for: {}", hash);
    
    match engine.resume_download(&hash).await {
        Ok(_) => {
            info!("✅ Download resumed: {}", hash);
            Ok(())
        }
        Err(e) => {
            error!("❌ Failed to resume download: {}", e);
            Err(format!("Failed to resume download: {}", e))
        }
    }
}

/// Delete a download
#[tauri::command]
pub async fn delete_download(
    hash: String,
    delete_files: bool,
    engine: State<'_, Engine>,
) -> Result<(), String> {
    info!("🗑️ Tauri: delete_download called for: {} (delete_files: {})", hash, delete_files);
    
    match engine.delete_download(&hash, delete_files).await {
        Ok(_) => {
            info!("✅ Download deleted: {}", hash);
            Ok(())
        }
        Err(e) => {
            error!("❌ Failed to delete download: {}", e);
            Err(format!("Failed to delete download: {}", e))
        }
    }
}

/// Get current settings from database
/// 
/// This command always succeeds by falling back to default settings if loading fails.
/// This ensures the UI can always display the settings form.
#[tauri::command]
pub async fn get_settings(
    engine: State<'_, Engine>,
) -> Result<Settings, String> {
    debug!("⚙️ Tauri: get_settings command called");
    
    match engine.get_settings().await {
        Ok(settings) => {
            info!("✅ Settings loaded successfully");
            Ok(settings)
        }
        Err(e) => {
            // This should rarely happen since engine.get_settings() now returns defaults on error
            error!("❌ Failed to load settings: {}", e);
            warn!("⚠️ Returning default settings to UI");
            Ok(Settings::default())
        }
    }
}

/// Save settings to database with validation
/// 
/// Returns user-friendly error messages for common failure scenarios:
/// - Validation errors: Specific field-level errors
/// - Database errors: Generic "unable to save" message
/// - Connection errors: Specific connection failure details
#[tauri::command]
pub async fn save_settings(
    settings: Settings,
    engine: State<'_, Engine>,
) -> Result<(), String> {
    info!("💾 Tauri: save_settings command called");
    
    match engine.save_settings(&settings).await {
        Ok(_) => {
            info!("✅ Settings saved successfully");
            Ok(())
        }
        Err(e) => {
            error!("❌ Failed to save settings: {}", e);
            
            // Provide user-friendly error messages
            let error_msg = e.to_string();
            
            // Check if it's a validation error
            if error_msg.contains("validation failed") || error_msg.contains("must be") {
                // Return the validation error as-is (it's already user-friendly)
                Err(error_msg)
            } else if error_msg.contains("Failed to save") {
                // Database error
                Err("Unable to save settings to database. Please check that the application has write permissions and try again.".to_string())
            } else {
                // Generic error
                Err(format!("Failed to save settings: {}", error_msg))
            }
        }
    }
}

/// Test qBittorrent connection with provided credentials
/// 
/// Returns user-friendly error messages for common connection failures:
/// - Connection timeout
/// - Connection refused
/// - Invalid credentials
/// - Invalid URL
#[tauri::command]
pub async fn test_qbittorrent(
    url: String,
    username: String,
    password: String,
    engine: State<'_, Engine>,
) -> Result<(), String> {
    info!("🧪 Tauri: test_qbittorrent command called for: {}", url);
    
    match engine.test_qbittorrent_connection(&url, &username, &password).await {
        Ok(_) => {
            info!("✅ qBittorrent connection successful");
            Ok(())
        }
        Err(e) => {
            error!("❌ qBittorrent connection failed: {}", e);
            
            // The error messages from test_qbittorrent_connection are already user-friendly
            // Just pass them through
            let error_msg = e.to_string();
            Err(error_msg)
        }
    }
}

/// Open native file picker for folder selection
#[tauri::command]
pub async fn pick_folder(app: tauri::AppHandle) -> Result<Option<String>, String> {
    debug!("📁 Tauri: pick_folder command called");
    
    // Use the dialog plugin to pick a folder
    let folder = tauri_plugin_dialog::DialogExt::dialog(&app)
        .file()
        .blocking_pick_folder();
    
    match folder {
        Some(path) => {
            let path_str = path.to_string();
            info!("✅ Folder selected: {}", path_str);
            Ok(Some(path_str))
        }
        None => {
            debug!("📁 Tauri: User cancelled folder selection");
            Ok(None)
        }
    }
}
