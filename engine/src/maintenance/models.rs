// Maintenance-related data models

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::{DateTime, Utc};

use crate::library::models::{MediaItemId, FileVersionId};

// ============================================================================
// Rescan Models
// ============================================================================

/// Report generated after a rescan operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RescanReport {
    /// Number of new files found on disk but not in database
    pub new_files_count: usize,
    /// Number of files in database but missing from disk
    pub missing_files_count: usize,
    /// Number of FileVersion status changes
    pub status_changes_count: usize,
    /// Number of MediaItems flagged as incomplete
    pub incomplete_media_count: usize,
    /// Total files scanned
    pub total_files_scanned: usize,
    /// List of new file paths found
    pub new_files: Vec<PathBuf>,
    /// List of missing FileVersion IDs
    pub missing_versions: Vec<FileVersionId>,
    /// List of incomplete MediaItem IDs
    pub incomplete_media: Vec<MediaItemId>,
    /// Timestamp when rescan started
    pub started_at: DateTime<Utc>,
    /// Timestamp when rescan completed
    pub completed_at: DateTime<Utc>,
}

impl RescanReport {
    /// Create a new empty rescan report
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            new_files_count: 0,
            missing_files_count: 0,
            status_changes_count: 0,
            incomplete_media_count: 0,
            total_files_scanned: 0,
            new_files: Vec::new(),
            missing_versions: Vec::new(),
            incomplete_media: Vec::new(),
            started_at: now,
            completed_at: now,
        }
    }

    /// Finalize the report with completion timestamp
    pub fn finalize(&mut self) {
        self.completed_at = Utc::now();
    }
}

impl Default for RescanReport {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Integrity Check Models
// ============================================================================

/// Result of an integrity check on a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityResult {
    pub version_id: FileVersionId,
    pub passed: bool,
    pub file_exists: bool,
    pub size_matches: bool,
    pub checksum_matches: Option<bool>,
    pub error_message: Option<String>,
}

/// Options for integrity check operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityCheckOptions {
    /// Verify checksums (slower but more thorough)
    pub verify_checksums: bool,
    /// Maximum number of files to check per batch
    pub batch_size: usize,
    /// Only check files not verified within this duration (in days)
    pub min_days_since_last_check: Option<u32>,
}

impl Default for IntegrityCheckOptions {
    fn default() -> Self {
        Self {
            verify_checksums: false,
            batch_size: 100,
            min_days_since_last_check: Some(30),
        }
    }
}

/// Report generated after an integrity check operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityCheckReport {
    /// Total number of files checked
    pub total_checked: usize,
    /// Number of files that passed integrity check
    pub passed: usize,
    /// Number of files that failed integrity check
    pub failed: usize,
    /// Number of files with corruption detected
    pub corrupted: usize,
    /// Number of files that are missing
    pub missing: usize,
    /// List of failed FileVersion IDs
    pub failed_versions: Vec<FileVersionId>,
    /// Timestamp when check started
    pub started_at: DateTime<Utc>,
    /// Timestamp when check completed
    pub completed_at: DateTime<Utc>,
}

impl IntegrityCheckReport {
    /// Create a new empty integrity check report
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            total_checked: 0,
            passed: 0,
            failed: 0,
            corrupted: 0,
            missing: 0,
            failed_versions: Vec::new(),
            started_at: now,
            completed_at: now,
        }
    }

    /// Finalize the report with completion timestamp
    pub fn finalize(&mut self) {
        self.completed_at = Utc::now();
    }
}

impl Default for IntegrityCheckReport {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Cleanup Models
// ============================================================================

/// A file identified as a candidate for cleanup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupCandidate {
    pub version_id: FileVersionId,
    pub path: PathBuf,
    pub size: u64,
    pub reason: CleanupReason,
}

/// Reason why a file is a cleanup candidate
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CleanupReason {
    ExactDuplicate,
    LowerQualityVariant,
    TrashedPastGracePeriod,
    MissingFileRecord,
}

/// Criteria for identifying cleanup candidates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupCriteria {
    pub include_duplicates: bool,
    pub include_low_quality_variants: bool,
    pub include_trashed: bool,
    pub include_missing_records: bool,
    pub trash_grace_period_days: u32,
}

impl Default for CleanupCriteria {
    fn default() -> Self {
        Self {
            include_duplicates: true,
            include_low_quality_variants: true,
            include_trashed: true,
            include_missing_records: true,
            trash_grace_period_days: 30,
        }
    }
}

/// Report generated after a cleanup operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupReport {
    pub files_deleted: usize,
    pub space_freed: u64,
    pub errors: Vec<CleanupError>,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
}

impl CleanupReport {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            files_deleted: 0,
            space_freed: 0,
            errors: Vec::new(),
            started_at: now,
            completed_at: now,
        }
    }

    pub fn finalize(&mut self) {
        self.completed_at = Utc::now();
    }
}

impl Default for CleanupReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Error that occurred during cleanup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupError {
    pub version_id: FileVersionId,
    pub path: PathBuf,
    pub error: String,
}

// ============================================================================
// Trash Management Models
// ============================================================================

/// Report generated after purging trash
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurgeReport {
    pub files_purged: usize,
    pub space_freed: u64,
    pub errors: Vec<CleanupError>,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
}

impl PurgeReport {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            files_purged: 0,
            space_freed: 0,
            errors: Vec::new(),
            started_at: now,
            completed_at: now,
        }
    }

    pub fn finalize(&mut self) {
        self.completed_at = Utc::now();
    }
}

impl Default for PurgeReport {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Database Maintenance Models
// ============================================================================

/// Statistics about database cleanup operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupStats {
    pub orphaned_versions_removed: usize,
    pub orphaned_tags_removed: usize,
    pub orphaned_collections_removed: usize,
}

// ============================================================================
// Storage Analytics Models
// ============================================================================

/// Storage statistics for the library
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStatistics {
    /// Total library size in bytes
    pub total_library_size: u64,
    /// Number of MediaItems
    pub media_item_count: usize,
    /// Number of FileVersions
    pub file_version_count: usize,
    /// Average versions per media item
    pub average_versions_per_media: f64,
    /// Breakdown by resolution
    pub by_resolution: Vec<StorageBreakdownItem>,
    /// Breakdown by quality label
    pub by_quality_label: Vec<StorageBreakdownItem>,
    /// Breakdown by status
    pub by_status: Vec<StorageBreakdownItem>,
}

/// A single item in a storage breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageBreakdownItem {
    /// Category name (e.g., "1080p", "present", "BluRay")
    pub category: String,
    /// Number of files in this category
    pub file_count: usize,
    /// Total size in bytes for this category
    pub total_size: u64,
}

/// Information about duplicate space usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateSpaceInfo {
    /// Number of duplicate groups
    pub duplicate_groups: usize,
    /// Total size of all duplicate files
    pub total_duplicate_size: u64,
    /// Size that would be saved by keeping only one copy per group
    pub wasted_space: u64,
    /// List of duplicate groups with details
    pub groups: Vec<DuplicateGroup>,
}

/// A group of duplicate files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateGroup {
    /// Hash value identifying this group
    pub hash: Vec<u8>,
    /// Number of files in this group
    pub file_count: usize,
    /// Size of one file (all are identical)
    pub file_size: u64,
    /// Total wasted space (size * (count - 1))
    pub wasted_space: u64,
    /// FileVersion IDs in this group
    pub version_ids: Vec<FileVersionId>,
}
