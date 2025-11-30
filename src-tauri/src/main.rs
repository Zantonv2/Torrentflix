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
            env_loaded = true;
            break;
        }
    }
    
    if !env_loaded {
        dotenvy::dotenv().ok();
    }

    // Configure proxy environment variables for HTTP clients
    if let Ok(proxy_url) = std::env::var("ADDRESS_PORT") {
        let socks5_url = format!("socks5://{}", proxy_url);
        std::env::set_var("ALL_PROXY", &socks5_url);
        std::env::set_var("HTTP_PROXY", &socks5_url);
        std::env::set_var("HTTPS_PROXY", &socks5_url);
    }

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
            commands::clear_all_ratings_cache
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
