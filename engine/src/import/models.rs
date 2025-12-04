// Import Pipeline Data Models

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::{DateTime, Utc};

// ============================================================================
// Type Aliases
// ============================================================================

pub type StagedFileId = i64;
pub type LibraryRootId = i64;

// ============================================================================
// Enums
// ============================================================================

/// Status of a staged file
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StagingStatus {
    /// File is pending processing
    Pending,
    /// File is being probed for metadata
    Probing,
    /// File is ready for commit
    Ready,
    /// File is being committed to library
    Committing,
    /// File processing failed
    Failed,
}

impl StagingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            StagingStatus::Pending => "pending",
            StagingStatus::Probing => "probing",
            StagingStatus::Ready => "ready",
            StagingStatus::Committing => "committing",
            StagingStatus::Failed => "failed",
        }
    }
}

impl std::fmt::Display for StagingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ============================================================================
// Staged File
// ============================================================================

/// Represents a file in the staging directory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StagedFile {
    /// Unique identifier
    pub id: StagedFileId,
    
    /// Original file path before staging
    pub original_path: PathBuf,
    
    /// Path in staging directory
    pub staged_path: PathBuf,
    
    /// File size in bytes
    pub file_size: u64,
    
    /// Current status
    pub status: StagingStatus,
    
    /// Error message if status is Failed
    pub error_message: Option<String>,
    
    /// Technical metadata (JSON)
    pub technical_metadata: Option<String>,
    
    /// Fast hash for quick duplicate detection
    pub fast_hash: Option<Vec<u8>>,
    
    /// Matched import profile ID
    pub matched_profile_id: Option<i64>,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// Library Root
// ============================================================================

/// Represents a library root directory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryRoot {
    /// Unique identifier
    pub id: LibraryRootId,
    
    /// Absolute path to the root directory
    pub path: PathBuf,
    
    /// Human-readable name
    pub name: String,
    
    /// Whether this root is active
    pub is_active: bool,
    
    /// Whether this is archive/cold storage
    pub is_archive: bool,
    
    /// Filesystem type (ext4, btrfs, xfs, etc.)
    pub filesystem_type: Option<String>,
    
    /// Mount point
    pub mount_point: Option<PathBuf>,
    
    /// Total space in bytes
    pub total_space: Option<u64>,
    
    /// Free space in bytes
    pub free_space: Option<u64>,
    
    /// Last scan timestamp
    pub last_scanned_at: Option<DateTime<Utc>>,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// Import Profile
// ============================================================================

pub type ImportProfileId = i64;
pub type CollectionId = i64;

/// Quality filters for import profiles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityFilters {
    /// Minimum resolution width
    pub min_resolution_width: Option<i32>,
    
    /// Minimum resolution height
    pub min_resolution_height: Option<i32>,
    
    /// Allowed codecs (e.g., ["h264", "h265"])
    pub allowed_codecs: Option<Vec<String>>,
    
    /// Minimum bitrate in bits per second
    pub min_bitrate: Option<u64>,
}

/// Import profile for automation rules
///
/// Defines rules for automatically processing files during import based on
/// file patterns, quality requirements, and automatic actions.
///
/// # Requirements
/// - Requirements: 28.1, 28.2, 28.3, 28.4, 28.5
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportProfile {
    /// Unique identifier
    pub id: ImportProfileId,
    
    /// Profile name
    pub name: String,
    
    /// Optional description
    pub description: Option<String>,
    
    /// Priority for profile matching (higher = higher priority)
    pub priority: i32,
    
    /// File pattern (glob or regex) for matching files
    /// Examples: "*.mkv", "**/*1080p*", "^.*\\.mp4$"
    pub file_pattern: Option<String>,
    
    /// Quality filters
    pub quality_filters: QualityFilters,
    
    /// Tags to automatically apply to imported media
    pub auto_tags: Vec<String>,
    
    /// Target collection ID for automatic assignment
    pub target_collection_id: Option<CollectionId>,
    
    /// Target library root ID for file placement
    pub target_library_root_id: Option<LibraryRootId>,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

/// Actions to be taken based on import profile rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportActions {
    /// Tags to apply to the media item
    pub tags_to_apply: Vec<String>,
    
    /// Collection ID to add the media item to
    pub collection_id: Option<CollectionId>,
    
    /// Library root ID to use for file placement
    pub library_root_id: Option<LibraryRootId>,
    
    /// Whether the file should be rejected based on quality filters
    pub should_reject: bool,
    
    /// Rejection reason if should_reject is true
    pub rejection_reason: Option<String>,
}
