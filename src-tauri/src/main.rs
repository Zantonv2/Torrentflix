#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri::Manager;

mod commands;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Initialize the engine
    let engine = engine::Engine::new();
    if let Err(e) = engine.initialize().await {
        eprintln!("Failed to initialize engine: {}", e);
        return;
    }

    tauri::Builder::default()
        .manage(engine)
        .invoke_handler(tauri::generate_handler![
            commands::search_movies,
            commands::get_magnet_link,
            commands::get_engine_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
