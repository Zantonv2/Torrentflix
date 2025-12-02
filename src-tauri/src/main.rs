#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri::{Manager, Emitter};
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

    // Initialize the engine
    let mut engine = engine::Engine::new();
    if let Err(e) = engine.initialize().await {
        warn!("Failed to initialize engine: {}", e);
        warn!("Application will start but engine features may be unavailable");
    }
    
    // Create channel for rating updates
    let (rating_tx, mut rating_rx) = mpsc::unbounded_channel::<engine::RatingUpdate>();
    engine.set_rating_update_channel(rating_tx);
    
    tauri::Builder::default()
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
            commands::delete_download
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
