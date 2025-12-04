// Watch Folder Data Models

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::{DateTime, Utc};

// ============================================================================
// Type Aliases
// ============================================================================

pub type WatchFolderId = i64;
pub type ImportProfileId = i64;

// ============================================================================
// Watch Folder
// ============================================================================

/// Represents a monitored directory for automatic import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchFolder {
    /// Unique identifier
    pub id: WatchFolderId,
    
    /// Path to the watched directory
    pub path: PathBuf,
    
    /// Whether monitoring is active
    pub is_active: bool,
    
    /// Whether to monitor subdirectories recursively
    pub recursive: bool,
    
    /// Optional import profile to apply
    pub import_profile_id: Option<ImportProfileId>,
    
    /// Last scan timestamp
    pub last_scan_at: Option<DateTime<Utc>>,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// Import Profile
// ============================================================================

/// Automation rules for file import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportProfile {
    /// Unique identifier
    pub id: ImportProfileId,
    
    /// Profile name
    pub name: String,
    
    /// Description
    pub description: Option<String>,
    
    /// Priority (higher = more important)
    pub priority: i32,
    
    /// File pattern (glob or regex)
    pub file_pattern: Option<String>,
    
    /// Minimum resolution width
    pub min_resolution_width: Option<i32>,
    
    /// Minimum resolution height
    pub min_resolution_height: Option<i32>,
    
    /// Allowed codecs (JSON array)
    pub allowed_codecs: Vec<String>,
    
    /// Auto-tags to apply
    pub auto_tags: Vec<String>,
    
    /// Target collection ID
    pub target_collection_id: Option<i64>,
    
    /// Target library root ID
    pub target_library_root_id: Option<i64>,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// Watch Event
// ============================================================================

/// Represents a filesystem event from inotify
#[derive(Debug, Clone)]
pub struct WatchEvent {
    /// Path to the file that triggered the event
    pub path: PathBuf,
    
    /// Watch folder ID
    pub watch_folder_id: WatchFolderId,
    
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
}
