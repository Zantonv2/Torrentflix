// Watch Folder Module
// Handles automatic file monitoring and import

pub mod monitor;
pub mod models;

// Re-export public types
pub use monitor::WatchFolderMonitor;
pub use models::*;
