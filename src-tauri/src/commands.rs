use tauri::State;
use anyhow::Result;
use tracing::{debug, info, error, warn};

use engine::Engine;
use engine::ui::dto::{UiSearchRequest, UiSearchResponse, UiSearchResult};
use engine::settings::models::Settings;
use engine::library::models::{
    MediaItem, FileVersion, LibraryFilters, SearchOptions, 
    CollectionId, MediaItemId, FilterCriteria, WatchStatus
};

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
    debug!("Tauri: search_movies command - Query: '{}', Type: {:?}", request.query, media_type);
    
    match engine.search_by_type(&request.query, media_type).await {
        Ok(results) => {
            let took_ms = start_time.elapsed().as_millis() as u64;
            
            // Convert engine results to UI DTOs
            let ui_results: Vec<UiSearchResult> = results
                .into_iter()
                .filter_map(UiSearchResult::from_media_search_result)
                .take(request.limit.unwrap_or(50) as usize)
                .collect();
            
            debug!("Search completed: {} results in {}ms", ui_results.len(), took_ms);
            
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
    debug!("Tauri: test_connection called");
    Ok("IPC bridge working!".to_string())
}

/// Get feed of recent movies
#[tauri::command]
pub async fn get_feed(
    engine: State<'_, Engine>,
) -> Result<UiSearchResponse, String> {
    debug!("Tauri: get_feed command called");
    
    match engine.get_feed().await {
        Ok(results) => {
            debug!("Tauri: Engine returned {} results", results.len());
            
            // Convert to UI DTOs
            let ui_results: Vec<UiSearchResult> = results
                .into_iter()
                .filter_map(UiSearchResult::from_media_search_result)
                .collect();
            
            let total_count = ui_results.len() as u32;
            let response = UiSearchResponse {
                results: ui_results,
                total: total_count,
                took_ms: 0, // Would need to track timing
            };
            
            debug!("Tauri: Returning response with {} movies to UI", response.results.len());
            Ok(response)
        }
        Err(e) => {
            error!("Tauri: Feed error: {}", e);
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
    debug!("Tauri: clear_imdb_cache command called");
    
    match engine.clear_imdb_cache().await {
        Ok(count) => {
            debug!("Cleared {} IMDb ratings from cache", count);
            Ok(count)
        }
        Err(e) => {
            error!("Failed to clear IMDb cache: {}", e);
            Err(format!("Failed to clear IMDb cache: {}", e))
        }
    }
}

/// Clear all ratings from cache
#[tauri::command]
pub async fn clear_all_ratings_cache(
    engine: State<'_, Engine>,
) -> Result<usize, String> {
    debug!("Tauri: clear_all_ratings_cache command called");
    
    match engine.clear_all_ratings_cache().await {
        Ok(count) => {
            debug!("Cleared {} ratings from cache", count);
            Ok(count)
        }
        Err(e) => {
            error!("Failed to clear ratings cache: {}", e);
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
    debug!("Tauri: start_download called for: {}", title);
    
    match engine.start_download(&magnet_link, &title).await {
        Ok(hash) => {
            debug!("Download started: {}", hash);
            Ok(hash)
        }
        Err(e) => {
            error!("Failed to start download: {}", e);
            Err(format!("Failed to start download: {}", e))
        }
    }
}

/// Get all active downloads
#[tauri::command]
pub async fn get_active_downloads(
    engine: State<'_, Engine>,
) -> Result<Vec<engine::models::DownloadStatus>, String> {
    debug!("Tauri: get_active_downloads called");
    
    match engine.get_active_downloads().await {
        Ok(downloads) => Ok(downloads),
        Err(e) => {
            error!("Failed to get downloads: {}", e);
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
    debug!("Tauri: pause_download called for: {}", hash);
    
    match engine.pause_download(&hash).await {
        Ok(_) => {
            debug!("Download paused: {}", hash);
            Ok(())
        }
        Err(e) => {
            error!("Failed to pause download: {}", e);
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
    debug!("Tauri: resume_download called for: {}", hash);
    
    match engine.resume_download(&hash).await {
        Ok(_) => {
            debug!("Download resumed: {}", hash);
            Ok(())
        }
        Err(e) => {
            error!("Failed to resume download: {}", e);
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

// ============================================================================
// Library Management Commands (Task 48)
// ============================================================================

/// Query library with filters
/// Requirements: 4.1, 4.2, 4.3, 4.4
#[tauri::command]
pub async fn query_library(
    filters: LibraryFilters,
    engine: State<'_, Engine>,
) -> Result<Vec<MediaItem>, String> {
    info!("📚 Tauri: query_library command called");
    
    match engine.query_library(&filters).await {
        Ok(items) => {
            info!("✅ Query returned {} items", items.len());
            Ok(items)
        }
        Err(e) => {
            error!("❌ Query failed: {}", e);
            Err(format!("Failed to query library: {}", e))
        }
    }
}

/// Search library by text
/// Requirements: 4.3, 20.1, 20.6
#[tauri::command]
pub async fn search_library(
    query: String,
    options: SearchOptions,
    engine: State<'_, Engine>,
) -> Result<Vec<MediaItem>, String> {
    info!("🔍 Tauri: search_library command called with query: '{}'", query);
    
    match engine.search_library(&query, &options).await {
        Ok(items) => {
            info!("✅ Search returned {} items", items.len());
            Ok(items)
        }
        Err(e) => {
            error!("❌ Search failed: {}", e);
            Err(format!("Failed to search library: {}", e))
        }
    }
}

/// Get a specific MediaItem by ID
/// Requirements: 4.4
#[tauri::command]
pub async fn get_media_item(
    id: i64,
    engine: State<'_, Engine>,
) -> Result<Option<MediaItem>, String> {
    debug!("📖 Tauri: get_media_item command called for id: {}", id);
    
    let media_id = MediaItemId(id);
    match engine.get_media_item(media_id).await {
        Ok(item) => {
            if item.is_some() {
                info!("✅ MediaItem found: {}", id);
            } else {
                debug!("⚠️ MediaItem not found: {}", id);
            }
            Ok(item)
        }
        Err(e) => {
            error!("❌ Failed to get MediaItem: {}", e);
            Err(format!("Failed to get MediaItem: {}", e))
        }
    }
}

/// Get all FileVersions for a MediaItem
/// Requirements: 4.4
#[tauri::command]
pub async fn get_file_versions(
    media_id: i64,
    engine: State<'_, Engine>,
) -> Result<Vec<FileVersion>, String> {
    debug!("📁 Tauri: get_file_versions command called for media_id: {}", media_id);
    
    let id = MediaItemId(media_id);
    match engine.get_file_versions(id).await {
        Ok(versions) => {
            info!("✅ Retrieved {} file versions", versions.len());
            Ok(versions)
        }
        Err(e) => {
            error!("❌ Failed to get file versions: {}", e);
            Err(format!("Failed to get file versions: {}", e))
        }
    }
}

// ============================================================================
// Import Commands (Task 48.2)
// ============================================================================

/// Stage a file for import
/// Requirements: 2.1, 2.4
#[tauri::command]
pub async fn stage_file(
    file_path: String,
    engine: State<'_, Engine>,
) -> Result<i64, String> {
    info!("📥 Tauri: stage_file command called for: {}", file_path);
    
    match engine.stage_file(&file_path).await {
        Ok(staged_id) => {
            info!("✅ File staged successfully: {}", staged_id);
            Ok(staged_id)
        }
        Err(e) => {
            error!("❌ Failed to stage file: {}", e);
            Err(format!("Failed to stage file: {}", e))
        }
    }
}

/// Commit a staged file to the library
/// Requirements: 2.1, 2.4
#[tauri::command]
pub async fn commit_staged_file(
    staged_id: i64,
    library_root_id: i64,
    engine: State<'_, Engine>,
) -> Result<i64, String> {
    info!("✅ Tauri: commit_staged_file command called for staged_id: {}", staged_id);
    
    match engine.commit_staged_file(staged_id, library_root_id).await {
        Ok(version_id) => {
            info!("✅ File committed successfully: {}", version_id);
            Ok(version_id)
        }
        Err(e) => {
            error!("❌ Failed to commit file: {}", e);
            Err(format!("Failed to commit file: {}", e))
        }
    }
}

/// Get all staged files
/// Requirements: 2.1, 2.4
#[tauri::command]
pub async fn get_staged_files(
    engine: State<'_, Engine>,
) -> Result<Vec<serde_json::Value>, String> {
    debug!("📋 Tauri: get_staged_files command called");
    
    match engine.get_staged_files().await {
        Ok(files) => {
            info!("✅ Retrieved {} staged files", files.len());
            Ok(files)
        }
        Err(e) => {
            error!("❌ Failed to get staged files: {}", e);
            Err(format!("Failed to get staged files: {}", e))
        }
    }
}

// ============================================================================
// Maintenance Commands (Task 48.3)
// ============================================================================

/// Schedule a rescan operation
/// Requirements: 5.1
#[tauri::command]
pub async fn schedule_rescan(
    library_root_id: Option<i64>,
    engine: State<'_, Engine>,
) -> Result<i64, String> {
    info!("🔄 Tauri: schedule_rescan command called");
    
    match engine.schedule_rescan(library_root_id).await {
        Ok(job_id) => {
            info!("✅ Rescan scheduled: job_id={}", job_id);
            Ok(job_id)
        }
        Err(e) => {
            error!("❌ Failed to schedule rescan: {}", e);
            Err(format!("Failed to schedule rescan: {}", e))
        }
    }
}

/// Get cleanup candidates
/// Requirements: 8.1, 8.2
#[tauri::command]
pub async fn get_cleanup_candidates(
    engine: State<'_, Engine>,
) -> Result<Vec<serde_json::Value>, String> {
    info!("🧹 Tauri: get_cleanup_candidates command called");
    
    match engine.get_cleanup_candidates().await {
        Ok(candidates) => {
            info!("✅ Found {} cleanup candidates", candidates.len());
            Ok(candidates)
        }
        Err(e) => {
            error!("❌ Failed to get cleanup candidates: {}", e);
            Err(format!("Failed to get cleanup candidates: {}", e))
        }
    }
}

/// Execute cleanup operation
/// Requirements: 8.4, 8.5, 8.6
#[tauri::command]
pub async fn execute_cleanup(
    candidate_ids: Vec<i64>,
    engine: State<'_, Engine>,
) -> Result<serde_json::Value, String> {
    info!("🗑️ Tauri: execute_cleanup command called for {} candidates", candidate_ids.len());
    
    match engine.execute_cleanup(candidate_ids).await {
        Ok(report) => {
            info!("✅ Cleanup completed");
            Ok(report)
        }
        Err(e) => {
            error!("❌ Cleanup failed: {}", e);
            Err(format!("Cleanup failed: {}", e))
        }
    }
}

/// Get storage analytics
/// Requirements: 10.1, 10.2, 10.3
#[tauri::command]
pub async fn get_storage_analytics(
    engine: State<'_, Engine>,
) -> Result<serde_json::Value, String> {
    info!("📊 Tauri: get_storage_analytics command called");
    
    match engine.get_storage_analytics().await {
        Ok(analytics) => {
            info!("✅ Storage analytics retrieved");
            Ok(analytics)
        }
        Err(e) => {
            error!("❌ Failed to get storage analytics: {}", e);
            Err(format!("Failed to get storage analytics: {}", e))
        }
    }
}

// ============================================================================
// Collection Commands (Task 48.4)
// ============================================================================

/// Create a new collection
/// Requirements: 19.1, 19.4
#[tauri::command]
pub async fn create_collection(
    name: String,
    description: Option<String>,
    engine: State<'_, Engine>,
) -> Result<i64, String> {
    info!("📚 Tauri: create_collection command called: '{}'", name);
    
    match engine.create_collection(&name, description.as_deref()).await {
        Ok(collection_id) => {
            info!("✅ Collection created: {}", collection_id);
            Ok(collection_id)
        }
        Err(e) => {
            error!("❌ Failed to create collection: {}", e);
            Err(format!("Failed to create collection: {}", e))
        }
    }
}

/// Add MediaItems to a collection
/// Requirements: 19.2
#[tauri::command]
pub async fn add_to_collection(
    collection_id: i64,
    media_ids: Vec<i64>,
    engine: State<'_, Engine>,
) -> Result<(), String> {
    info!("➕ Tauri: add_to_collection command called for collection: {}", collection_id);
    
    let coll_id = CollectionId(collection_id);
    let ids: Vec<MediaItemId> = media_ids.into_iter().map(MediaItemId).collect();
    
    match engine.add_to_collection(coll_id, &ids).await {
        Ok(_) => {
            info!("✅ Added {} items to collection", ids.len());
            Ok(())
        }
        Err(e) => {
            error!("❌ Failed to add items to collection: {}", e);
            Err(format!("Failed to add items to collection: {}", e))
        }
    }
}

/// Get MediaItems in a collection
/// Requirements: 19.3
#[tauri::command]
pub async fn get_collection_items(
    collection_id: i64,
    engine: State<'_, Engine>,
) -> Result<Vec<MediaItem>, String> {
    debug!("📖 Tauri: get_collection_items command called for collection: {}", collection_id);
    
    let coll_id = CollectionId(collection_id);
    match engine.get_collection_items(coll_id).await {
        Ok(items) => {
            info!("✅ Retrieved {} items from collection", items.len());
            Ok(items)
        }
        Err(e) => {
            error!("❌ Failed to get collection items: {}", e);
            Err(format!("Failed to get collection items: {}", e))
        }
    }
}

/// Create a smart collection
/// Requirements: 19.4, 19.5
#[tauri::command]
pub async fn create_smart_collection(
    name: String,
    criteria: FilterCriteria,
    engine: State<'_, Engine>,
) -> Result<i64, String> {
    info!("🧠 Tauri: create_smart_collection command called: '{}'", name);
    
    match engine.create_smart_collection(&name, &criteria).await {
        Ok(collection_id) => {
            info!("✅ Smart collection created: {}", collection_id);
            Ok(collection_id)
        }
        Err(e) => {
            error!("❌ Failed to create smart collection: {}", e);
            Err(format!("Failed to create smart collection: {}", e))
        }
    }
}

// ============================================================================
// User Metadata Commands (Task 48.5)
// ============================================================================

/// Set user rating for a MediaItem
/// Requirements: 9.2, 9.6
#[tauri::command]
pub async fn set_user_rating(
    media_id: i64,
    rating: f32,
    engine: State<'_, Engine>,
) -> Result<(), String> {
    info!("⭐ Tauri: set_user_rating command called for media_id: {}, rating: {}", media_id, rating);
    
    let id = MediaItemId(media_id);
    match engine.set_user_rating(id, rating).await {
        Ok(_) => {
            info!("✅ Rating set successfully");
            Ok(())
        }
        Err(e) => {
            error!("❌ Failed to set rating: {}", e);
            Err(format!("Failed to set rating: {}", e))
        }
    }
}

/// Add tags to a MediaItem
/// Requirements: 9.3, 9.4
#[tauri::command]
pub async fn add_tags(
    media_id: i64,
    tags: Vec<String>,
    engine: State<'_, Engine>,
) -> Result<(), String> {
    info!("🏷️ Tauri: add_tags command called for media_id: {}", media_id);
    
    let id = MediaItemId(media_id);
    match engine.add_tags(id, &tags).await {
        Ok(_) => {
            info!("✅ Tags added successfully");
            Ok(())
        }
        Err(e) => {
            error!("❌ Failed to add tags: {}", e);
            Err(format!("Failed to add tags: {}", e))
        }
    }
}

/// Set watch status for a MediaItem
/// Requirements: 9.2, 25.5
#[tauri::command]
pub async fn set_watch_status(
    media_id: i64,
    status: String,
    engine: State<'_, Engine>,
) -> Result<(), String> {
    info!("👁️ Tauri: set_watch_status command called for media_id: {}, status: {}", media_id, status);
    
    let watch_status = match status.as_str() {
        "unwatched" => WatchStatus::Unwatched,
        "in_progress" => WatchStatus::InProgress,
        "watched" => WatchStatus::Watched,
        _ => {
            error!("❌ Invalid watch status: {}", status);
            return Err(format!("Invalid watch status: {}", status));
        }
    };
    
    let id = MediaItemId(media_id);
    match engine.set_watch_status(id, watch_status).await {
        Ok(_) => {
            info!("✅ Watch status set successfully");
            Ok(())
        }
        Err(e) => {
            error!("❌ Failed to set watch status: {}", e);
            Err(format!("Failed to set watch status: {}", e))
        }
    }
}

/// Set custom notes for a MediaItem
/// Requirements: 9.5, 25.5
#[tauri::command]
pub async fn set_custom_notes(
    media_id: i64,
    notes: String,
    engine: State<'_, Engine>,
) -> Result<(), String> {
    info!("📝 Tauri: set_custom_notes command called for media_id: {}", media_id);
    
    let id = MediaItemId(media_id);
    match engine.set_custom_notes(id, &notes).await {
        Ok(_) => {
            info!("✅ Custom notes set successfully");
            Ok(())
        }
        Err(e) => {
            error!("❌ Failed to set custom notes: {}", e);
            Err(format!("Failed to set custom notes: {}", e))
        }
    }
}

// ============================================================================
// Desktop Integration Commands (Task 51)
// ============================================================================

/// Process dropped files for import
/// Requirements: 30.4
#[tauri::command]
pub async fn process_dropped_files(
    files: Vec<String>,
    _engine: State<'_, Engine>,
) -> Result<Vec<String>, String> {
    info!("📥 Tauri: process_dropped_files command called with {} files", files.len());
    
    let file_paths: Vec<std::path::PathBuf> = files
        .into_iter()
        .map(std::path::PathBuf::from)
        .collect();
    
    match engine::desktop::DragDropHandler::process_dropped_files(file_paths).await {
        Ok(valid_files) => {
            let file_strings: Vec<String> = valid_files
                .into_iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
            info!("✅ Processed {} valid files from drag-and-drop", file_strings.len());
            Ok(file_strings)
        }
        Err(e) => {
            error!("❌ Failed to process dropped files: {}", e);
            Err(format!("Failed to process dropped files: {}", e))
        }
    }
}

/// Register the application with the desktop environment
/// Requirements: 30.1, 30.2
#[tauri::command]
pub async fn register_file_associations(
    app_name: String,
    app_exec: String,
    app_icon: Option<String>,
) -> Result<(), String> {
    info!("📋 Tauri: register_file_associations command called");
    
    let manager = engine::desktop::FileAssociationManager::new(
        app_name,
        std::path::PathBuf::from(app_exec),
        app_icon.map(std::path::PathBuf::from),
    );
    
    match manager.register_with_desktop().await {
        Ok(_) => {
            info!("✅ File associations registered successfully");
            Ok(())
        }
        Err(e) => {
            error!("❌ Failed to register file associations: {}", e);
            Err(format!("Failed to register file associations: {}", e))
        }
    }
}

/// Check if the application is registered with the desktop environment
/// Requirements: 30.1, 30.2
#[tauri::command]
pub async fn is_file_associations_registered(
    app_name: String,
) -> Result<bool, String> {
    debug!("🔍 Tauri: is_file_associations_registered command called");
    
    let manager = engine::desktop::FileAssociationManager::new(
        app_name,
        std::path::PathBuf::from("/usr/bin/moviedownloader"),
        None,
    );
    
    Ok(manager.is_registered().await)
}

/// Unregister the application from the desktop environment
/// Requirements: 30.1, 30.2
#[tauri::command]
pub async fn unregister_file_associations(
    app_name: String,
) -> Result<(), String> {
    info!("🗑️ Tauri: unregister_file_associations command called");
    
    let manager = engine::desktop::FileAssociationManager::new(
        app_name,
        std::path::PathBuf::from("/usr/bin/moviedownloader"),
        None,
    );
    
    match manager.unregister_from_desktop().await {
        Ok(_) => {
            info!("✅ File associations unregistered successfully");
            Ok(())
        }
        Err(e) => {
            error!("❌ Failed to unregister file associations: {}", e);
            Err(format!("Failed to unregister file associations: {}", e))
        }
    }
}

/// Initialize system tray
/// Requirements: 30.5
#[tauri::command]
pub async fn initialize_system_tray(
    icon_path: Option<String>,
) -> Result<(), String> {
    info!("🎯 Tauri: initialize_system_tray command called");
    
    let manager = engine::desktop::SystemTrayManager::new(icon_path);
    
    match manager.initialize().await {
        Ok(_) => {
            info!("✅ System tray initialized successfully");
            Ok(())
        }
        Err(e) => {
            error!("❌ Failed to initialize system tray: {}", e);
            Err(format!("Failed to initialize system tray: {}", e))
        }
    }
}

/// Update system tray status
/// Requirements: 30.5
#[tauri::command]
pub async fn update_tray_status(
    is_running: bool,
    active_jobs: usize,
    status_message: String,
) -> Result<(), String> {
    debug!("🎯 Tauri: update_tray_status command called");
    
    let manager = engine::desktop::SystemTrayManager::new(None);
    let status = engine::desktop::system_tray::TrayStatus {
        is_running,
        active_jobs,
        status_message,
    };
    
    match manager.update_status(status).await {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("❌ Failed to update tray status: {}", e);
            Err(format!("Failed to update tray status: {}", e))
        }
    }
}

/// Get system tray tooltip
/// Requirements: 30.5
#[tauri::command]
pub async fn get_tray_tooltip() -> Result<String, String> {
    debug!("🎯 Tauri: get_tray_tooltip command called");
    
    let manager = engine::desktop::SystemTrayManager::new(None);
    Ok(manager.get_tooltip().await)
}

/// Initialize power management monitoring
/// Requirements: 30.6
#[tauri::command]
pub async fn initialize_power_management() -> Result<(), String> {
    info!("⚡ Tauri: initialize_power_management command called");
    
    let handler = engine::desktop::PowerManagementHandler::new();
    
    match handler.initialize().await {
        Ok(_) => {
            info!("✅ Power management monitoring initialized");
            Ok(())
        }
        Err(e) => {
            error!("❌ Failed to initialize power management: {}", e);
            Err(format!("Failed to initialize power management: {}", e))
        }
    }
}

/// Get current power state
/// Requirements: 30.6
#[tauri::command]
pub async fn get_power_state() -> Result<String, String> {
    debug!("⚡ Tauri: get_power_state command called");
    
    let handler = engine::desktop::PowerManagementHandler::new();
    let state = handler.get_power_state().await;
    
    let state_str = match state {
        engine::desktop::power_management::PowerState::Running => "running",
        engine::desktop::power_management::PowerState::SuspendPending => "suspend_pending",
        engine::desktop::power_management::PowerState::Suspended => "suspended",
        engine::desktop::power_management::PowerState::Resuming => "resuming",
        engine::desktop::power_management::PowerState::HibernationPending => "hibernation_pending",
        engine::desktop::power_management::PowerState::Hibernating => "hibernating",
    };
    
    Ok(state_str.to_string())
}

/// Check if operations should be paused due to power state
/// Requirements: 30.6
#[tauri::command]
pub async fn should_pause_operations() -> Result<bool, String> {
    debug!("⚡ Tauri: should_pause_operations command called");
    
    let handler = engine::desktop::PowerManagementHandler::new();
    Ok(handler.should_pause_operations().await)
}
