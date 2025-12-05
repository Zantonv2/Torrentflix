#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri::Emitter;
use tracing::{warn, info};
use tokio::sync::mpsc;

mod commands;

#[tokio::main]
async fn main() {
    // Set up panic handler to log panics
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("💥 PANIC: {:?}", panic_info);
        if let Some(location) = panic_info.location() {
            eprintln!("   at {}:{}:{}", location.file(), location.line(), location.column());
        }
    }));
    // Load environment variables from .env file
    // Try multiple locations: project root, engine/src/, src-tauri/
    let env_paths = [
        ".env",
        "engine/src/.env",
        "src-tauri/.env",
        "../.env",
        "../engine/src/.env",
    ];
    
    let mut env_loaded = false;
    for path in &env_paths {
        if let Ok(_) = dotenvy::from_filename(path) {
            eprintln!("✅ Loaded .env from: {}", path);
            env_loaded = true;
            // Verify ADDRESS_PORT is loaded
            if let Ok(addr) = std::env::var("ADDRESS_PORT") {
                eprintln!("✅ ADDRESS_PORT is set: {}", addr);
            } else {
                eprintln!("⚠️ ADDRESS_PORT not found in environment after loading {}", path);
            }
            break;
        }
    }
    
    if !env_loaded {
        eprintln!("⚠️ No .env file found in any of the checked paths, trying dotenv()");
        dotenvy::dotenv().ok();
        if let Ok(addr) = std::env::var("ADDRESS_PORT") {
            eprintln!("✅ ADDRESS_PORT found after dotenv(): {}", addr);
        } else {
            eprintln!("❌ ADDRESS_PORT still not found after dotenv()");
        }
    }

    // Note: Proxy configuration is handled directly in each HTTP client (e.g., TmdbClient)
    // We don't set ALL_PROXY/HTTP_PROXY/HTTPS_PROXY here to avoid conflicts with direct proxy configuration
    // Each client that needs a proxy should read ADDRESS_PORT and configure it explicitly

    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create channel for rating updates BEFORE initializing engine
    let (rating_tx, mut rating_rx) = mpsc::unbounded_channel::<engine::RatingUpdate>();
    
    // Initialize the engine
    let mut engine = engine::Engine::new();
    
    // Set the rating update channel BEFORE initialize() so it's available during setup
    engine.set_rating_update_channel(rating_tx);
    
    if let Err(e) = engine.initialize().await {
        warn!("Failed to initialize engine: {}", e);
        warn!("Application will start but engine features may be unavailable");
    }
    
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(move |app| {
            let app_handle = app.handle().clone();
            
            // Spawn task to listen for rating updates and emit Tauri events
            tokio::spawn(async move {
                eprintln!("🎬 Tauri: Started listening for rating updates");
                info!("🎬 Tauri: Started listening for rating updates");
                while let Some(update) = rating_rx.recv().await {
                    eprintln!("🎬 Tauri: Received rating update for movie_id: {} - Kinopoisk: {:?}, IMDb: {:?}, TMDB: {:?}", 
                              update.movie_id, update.rating_kinopoisk, update.rating_imdb, update.rating_tmdb);
                    info!("🎬 Tauri: Received rating update for movie_id: {}", update.movie_id);
                    match app_handle.emit("rating-update", &update) {
                        Ok(_) => {
                            eprintln!("🎬 Tauri: ✅ Successfully emitted rating-update event for {}", update.movie_id);
                        }
                        Err(e) => {
                            eprintln!("🎬 Tauri: ❌ Failed to emit rating-update event for {}: {}", update.movie_id, e);
                            warn!("Failed to emit rating-update event: {}", e);
                        }
                    }
                }
                eprintln!("🎬 Tauri: Rating update channel closed");
            });
            
            Ok(())
        })
        .manage(engine)
        .invoke_handler(tauri::generate_handler![
            commands::search_movies,
            commands::get_feed,
            commands::get_magnet_link,
            commands::get_engine_status,
            commands::test_connection,
            commands::clear_imdb_cache,
            commands::clear_all_ratings_cache,
            commands::start_download,
            commands::get_active_downloads,
            commands::pause_download,
            commands::resume_download,
            commands::delete_download,
            commands::get_settings,
            commands::save_settings,
            commands::test_qbittorrent,
            commands::pick_folder,
            // Library management commands
            commands::query_library,
            commands::search_library,
            commands::get_media_item,
            commands::get_file_versions,
            // Import commands
            commands::stage_file,
            commands::commit_staged_file,
            commands::get_staged_files,
            // Maintenance commands
            commands::schedule_rescan,
            commands::get_cleanup_candidates,
            commands::execute_cleanup,
            commands::get_storage_analytics,
            // Collection commands
            commands::create_collection,
            commands::add_to_collection,
            commands::get_collection_items,
            commands::create_smart_collection,
            // User metadata commands
            commands::set_user_rating,
            commands::add_tags,
            commands::set_watch_status,
            commands::set_custom_notes,
            // Desktop integration commands
            commands::process_dropped_files,
            commands::register_file_associations,
            commands::is_file_associations_registered,
            commands::unregister_file_associations,
            commands::initialize_system_tray,
            commands::update_tray_status,
            commands::get_tray_tooltip,
            commands::initialize_power_management,
            commands::get_power_state,
            commands::should_pause_operations
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
