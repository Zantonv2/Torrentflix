// Library management domain models
// Defines all data structures for media items, file versions, collections, and related entities

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;
use chrono::{DateTime, Utc};

// ============================================================================
// Core Domain Models
// ============================================================================

use std::fmt;

/// Unique identifier for a MediaItem
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MediaItemId(pub i64);

impl fmt::Display for MediaItemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a FileVersion
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FileVersionId(pub i64);

impl fmt::Display for FileVersionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a LibraryRoot
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LibraryRootId(pub i64);

impl fmt::Display for LibraryRootId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a Collection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CollectionId(pub i64);

impl fmt::Display for CollectionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a Tag
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TagId(pub i64);

impl fmt::Display for TagId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for an ImportProfile
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ImportProfileId(pub i64);

impl fmt::Display for ImportProfileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a WatchFolder
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WatchFolderId(pub i64);

impl fmt::Display for WatchFolderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a SavedSearch
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SavedSearchId(pub i64);

impl fmt::Display for SavedSearchId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ============================================================================
// Enums
// ============================================================================

/// Type of media (movie or series)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaType {
    Movie,
    Series,
}

/// Status of a file version
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileStatus {
    Present,
    Missing,
    Trashed,
    Corrupted,
}

/// Watch status of a media item
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WatchStatus {
    Unwatched,
    InProgress,
    Watched,
}

impl std::fmt::Display for WatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WatchStatus::Unwatched => write!(f, "unwatched"),
            WatchStatus::InProgress => write!(f, "in_progress"),
            WatchStatus::Watched => write!(f, "watched"),
        }
    }
}

/// Status of file hashing operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HashingStatus {
    Pending,
    InProgress,
    Complete,
    Failed,
}

/// Source type of a file (quality classification)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceType {
    BluRay,
    WebDl,
    Hdtv,
    Dvd,
    Unknown,
}

/// Type of collection
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CollectionType {
    Manual,
    Smart { criteria: FilterCriteria },
}

// ============================================================================
// Supporting Structures
// ============================================================================

/// Resolution information
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl Resolution {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

/// Video codec and technical information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoInfo {
    pub codec: String,
    pub resolution: Resolution,
    pub framerate: f32,
    pub bitrate: u64,
    pub color_space: Option<String>,
}

/// Audio track information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioTrack {
    pub codec: String,
    pub language: Option<String>,
    pub channels: u32,
    pub bitrate: u64,
}

/// Subtitle track information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleTrack {
    pub language: Option<String>,
    pub codec: String,
}

/// Technical metadata extracted from a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalMetadata {
    pub container: String,
    pub video: Option<VideoInfo>,
    pub audio_tracks: Vec<AudioTrack>,
    pub subtitle_tracks: Vec<SubtitleTrack>,
    pub duration: u64, // seconds
    pub bitrate: u64,
    pub file_size: u64,
}

/// Filesystem-specific information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemInfo {
    pub is_symlink: bool,
    pub symlink_target: Option<PathBuf>,
    pub is_hardlink: bool,
    pub inode: Option<u64>,
}

/// Fast hash for quick duplicate detection
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FastHash(pub Vec<u8>);

/// Full hash for exact duplicate detection
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FullHash {
    pub algorithm: String,
    pub value: Vec<u8>,
}

/// Checksum for integrity verification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Checksum {
    pub algorithm: String,
    pub value: Vec<u8>,
}

/// Quality filters for import profiles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityFilters {
    pub min_resolution: Option<Resolution>,
    pub allowed_codecs: Option<Vec<String>>,
    pub min_bitrate: Option<u64>,
}

/// Filter criteria for smart collections and searches
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilterCriteria {
    pub title_pattern: Option<String>,
    pub year_range: Option<(i32, i32)>,
    pub quality_labels: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub min_rating: Option<f32>,
    pub watch_status: Option<WatchStatus>,
}

/// Cast member information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CastMember {
    pub name: String,
    pub character: Option<String>,
}

/// External IDs from metadata providers
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExternalIds {
    pub tmdb_id: Option<i32>,
    pub imdb_id: Option<String>,
    pub kinopoisk_id: Option<i32>,
}

/// Image data (poster, backdrop, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageData {
    pub url: Option<String>,
    pub local_path: Option<PathBuf>,
}

// ============================================================================
// Main Domain Models
// ============================================================================

/// A logical media entity (movie or TV series)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaItem {
    pub id: MediaItemId,
    pub title: String,
    pub normalized_title: String,
    pub original_title: Option<String>,
    pub year: Option<i32>,
    pub media_type: MediaType,
    pub overview: Option<String>,
    pub genres: Vec<String>,
    pub cast: Vec<CastMember>,
    pub director: Option<String>,
    pub runtime: Option<u64>, // seconds
    pub poster: Option<ImageData>,
    pub backdrop: Option<ImageData>,
    pub external_ids: ExternalIds,
    pub user_rating: Option<f32>,
    pub watch_status: WatchStatus,
    pub last_watched_at: Option<DateTime<Utc>>,
    pub custom_notes: Option<String>,
    pub tags: Vec<TagId>,
    pub metadata_source: Option<String>,
    pub metadata_fetched_at: Option<DateTime<Utc>>,
    pub user_edited_fields: HashSet<String>,
    pub alternative_titles: Vec<String>, // Additional titles for search matching
    pub custom_poster: Option<Vec<u8>>, // Custom uploaded poster
    pub custom_backdrop: Option<Vec<u8>>, // Custom uploaded backdrop
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A physical file on disk associated with a MediaItem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileVersion {
    pub id: FileVersionId,
    pub media_item_id: MediaItemId,
    pub library_root_id: LibraryRootId,
    pub relative_path: PathBuf,
    pub absolute_path: PathBuf,
    pub file_size: u64,
    pub status: FileStatus,
    pub is_preferred: bool,
    pub technical_metadata: Option<TechnicalMetadata>,
    pub quality_label: Option<String>,
    pub release_group: Option<String>,
    pub source_type: Option<SourceType>,
    pub fast_hash: Option<FastHash>,
    pub full_hash: Option<FullHash>,
    pub hashing_status: HashingStatus,
    pub checksum: Option<Checksum>,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub corruption_detected_at: Option<DateTime<Utc>>,
    pub filesystem_info: FilesystemInfo,
    pub added_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub trashed_at: Option<DateTime<Utc>>,
    pub original_path: Option<PathBuf>,
    pub import_source: Option<String>,
}

/// A designated storage location for media files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryRoot {
    pub id: LibraryRootId,
    pub path: PathBuf,
    pub name: String,
    pub is_active: bool,
    pub is_archive: bool,
    pub filesystem_type: Option<String>,
    pub mount_point: Option<PathBuf>,
    pub total_space: Option<u64>,
    pub free_space: Option<u64>,
    pub last_scanned_at: Option<DateTime<Utc>>,
}

/// A user-defined tag for organizing media
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: TagId,
    pub name: String,
    pub color: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// A user-defined collection of media items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub id: CollectionId,
    pub name: String,
    pub description: Option<String>,
    pub collection_type: CollectionType,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Configuration for automatic import rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportProfile {
    pub id: ImportProfileId,
    pub name: String,
    pub description: Option<String>,
    pub priority: i32,
    pub file_pattern: Option<String>,
    pub quality_filters: QualityFilters,
    pub auto_tags: Vec<String>,
    pub target_collection_id: Option<CollectionId>,
    pub target_library_root_id: Option<LibraryRootId>,
}

/// A directory monitored for new files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchFolder {
    pub id: WatchFolderId,
    pub path: PathBuf,
    pub is_active: bool,
    pub recursive: bool,
    pub import_profile_id: Option<ImportProfileId>,
    pub last_scan_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// A file in the import staging area
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StagedFile {
    pub id: i64,
    pub original_path: PathBuf,
    pub staged_path: PathBuf,
    pub file_size: u64,
    pub status: String, // pending, probing, ready, committing, failed
    pub error_message: Option<String>,
    pub technical_metadata: Option<TechnicalMetadata>,
    pub fast_hash: Option<FastHash>,
    pub matched_profile_id: Option<ImportProfileId>,
    pub created_at: DateTime<Utc>,
}

/// A saved search query with filters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedSearch {
    pub id: SavedSearchId,
    pub name: String,
    pub description: Option<String>,
    pub query: String,
    pub filters: LibraryFilters,
    pub search_options: SearchOptions,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

// ============================================================================
// Query and Result Types
// ============================================================================

/// Filters for library queries
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LibraryFilters {
    pub title: Option<String>,
    pub year: Option<i32>,
    pub resolution: Option<Resolution>,
    pub quality_label: Option<String>,
    pub status: Option<FileStatus>,
    pub tags: Option<Vec<TagId>>,
    pub min_rating: Option<f32>,
    pub watch_status: Option<WatchStatus>,
    pub sort_by: Option<SortField>,
    pub sort_ascending: bool,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// Options for library search
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchOptions {
    pub fuzzy: bool,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// Options for sorting library results
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortField {
    Title,
    Year,
    AddedDate,
    Size,
    VersionCount,
    UserRating,
}

/// A group of duplicate files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateGroup {
    pub hash: FullHash,
    pub versions: Vec<FileVersionId>,
}

/// A file identified as a cleanup candidate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupCandidate {
    pub version_id: FileVersionId,
    pub file_path: PathBuf,
    pub file_size: u64,
    pub reason: String,
}

/// Report from a rescan operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RescanReport {
    pub new_files_found: u32,
    pub missing_files_detected: u32,
    pub status_changes: u32,
    pub errors: Vec<String>,
}

/// Report from a cleanup operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupReport {
    pub files_deleted: u32,
    pub space_freed: u64,
    pub errors: Vec<String>,
}

/// Result of integrity verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityResult {
    pub passed: bool,
    pub details: String,
}

/// Quality score for a file version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityScore {
    pub total: f32,
    pub resolution_score: f32,
    pub codec_score: f32,
    pub source_score: f32,
    pub size_score: f32,
}

// ============================================================================
// Batch Operation Models
// ============================================================================

/// Result of a batch operation on a single item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchItemResult {
    pub media_item_id: MediaItemId,
    pub success: bool,
    pub error: Option<String>,
}

/// Result of a batch tag operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchTagResult {
    pub total_items: usize,
    pub successful: usize,
    pub failed: usize,
    pub results: Vec<BatchItemResult>,
}

/// Result of a batch rating operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRatingResult {
    pub total_items: usize,
    pub successful: usize,
    pub failed: usize,
    pub results: Vec<BatchItemResult>,
}

/// Result of a batch collection assignment operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCollectionResult {
    pub total_items: usize,
    pub successful: usize,
    pub failed: usize,
    pub results: Vec<BatchItemResult>,
}

/// Result of a batch deletion operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchDeletionResult {
    pub total_items: usize,
    pub total_versions: usize,
    pub successful_versions: usize,
    pub failed_versions: usize,
    pub results: Vec<BatchItemResult>,
}

/// Progress callback for batch operations
pub type BatchProgressCallback = Box<dyn Fn(usize, usize) + Send + Sync>;

// ============================================================================
// Export and Backup Models
// ============================================================================

/// Format for export files
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Csv,
}

/// Export profile for filtering what to export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportProfile {
    pub name: String,
    pub description: Option<String>,
    pub filters: Option<LibraryFilters>,
    pub include_collections: bool,
    pub include_tags: bool,
    pub include_ratings: bool,
    pub include_notes: bool,
    pub include_audit_logs: bool,
    pub include_artwork: bool,
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
}

impl Default for ExportProfile {
    fn default() -> Self {
        Self {
            name: "Full Export".to_string(),
            description: None,
            filters: None,
            include_collections: true,
            include_tags: true,
            include_ratings: true,
            include_notes: true,
            include_audit_logs: false,
            include_artwork: false,
            date_range: None,
        }
    }
}

/// Complete library export data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryExport {
    pub version: String,
    pub exported_at: DateTime<Utc>,
    pub media_items: Vec<MediaItem>,
    pub file_versions: Vec<FileVersion>,
    pub collections: Option<Vec<Collection>>,
    pub tags: Option<Vec<Tag>>,
    pub audit_logs: Option<Vec<AuditLogEntry>>,
}

/// Audit log entry for export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: i64,
    pub operation_type: String,
    pub entity_type: String,
    pub entity_id: Option<i64>,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub initiator: String,
    pub error_message: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// Result of an export operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    pub file_path: PathBuf,
    pub format: ExportFormat,
    pub media_items_count: usize,
    pub file_versions_count: usize,
    pub collections_count: usize,
    pub tags_count: usize,
    pub audit_logs_count: usize,
    pub file_size_bytes: u64,
    pub exported_at: DateTime<Utc>,
}

/// Options for import merge behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeStrategy {
    /// Skip items that already exist
    Skip,
    /// Update existing items with imported data
    Update,
    /// Keep existing items, don't update
    KeepExisting,
    /// Fail on conflicts
    Fail,
}

/// Result of an import operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub total_media_items: usize,
    pub imported_media_items: usize,
    pub skipped_media_items: usize,
    pub updated_media_items: usize,
    pub failed_media_items: usize,
    pub total_file_versions: usize,
    pub imported_file_versions: usize,
    pub skipped_file_versions: usize,
    pub errors: Vec<String>,
    pub imported_at: DateTime<Utc>,
}

/// Backup options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupOptions {
    pub include_database: bool,
    pub include_artwork: bool,
    pub include_metadata_cache: bool,
    pub compress: bool,
}

impl Default for BackupOptions {
    fn default() -> Self {
        Self {
            include_database: true,
            include_artwork: false,
            include_metadata_cache: false,
            compress: true,
        }
    }
}

/// Result of a backup operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupResult {
    pub backup_path: PathBuf,
    pub database_size_bytes: u64,
    pub artwork_size_bytes: u64,
    pub total_size_bytes: u64,
    pub compressed: bool,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// Metadata Editing
// ============================================================================

/// Metadata fields that can be edited by the user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataEdit {
    pub title: Option<String>,
    pub original_title: Option<Option<String>>, // Option<Option> to distinguish between "don't change" and "set to None"
    pub year: Option<Option<i32>>,
    pub overview: Option<Option<String>>,
    pub genres: Option<Vec<String>>,
    pub cast: Option<Vec<CastMember>>,
    pub director: Option<Option<String>>,
    pub runtime: Option<Option<u64>>,
    pub alternative_titles: Option<Vec<String>>, // Additional titles for search matching
}

impl MetadataEdit {
    /// Create an empty edit (no changes)
    pub fn new() -> Self {
        Self {
            title: None,
            original_title: None,
            year: None,
            overview: None,
            genres: None,
            cast: None,
            director: None,
            runtime: None,
            alternative_titles: None,
        }
    }

    /// Check if any fields are being edited
    pub fn has_changes(&self) -> bool {
        self.title.is_some()
            || self.original_title.is_some()
            || self.year.is_some()
            || self.overview.is_some()
            || self.genres.is_some()
            || self.cast.is_some()
            || self.director.is_some()
            || self.runtime.is_some()
            || self.alternative_titles.is_some()
    }

    /// Get list of field names being edited
    pub fn edited_field_names(&self) -> Vec<String> {
        let mut fields = Vec::new();
        if self.title.is_some() {
            fields.push("title".to_string());
        }
        if self.original_title.is_some() {
            fields.push("original_title".to_string());
        }
        if self.year.is_some() {
            fields.push("year".to_string());
        }
        if self.overview.is_some() {
            fields.push("overview".to_string());
        }
        if self.genres.is_some() {
            fields.push("genres".to_string());
        }
        if self.cast.is_some() {
            fields.push("cast".to_string());
        }
        if self.director.is_some() {
            fields.push("director".to_string());
        }
        if self.runtime.is_some() {
            fields.push("runtime".to_string());
        }
        if self.alternative_titles.is_some() {
            fields.push("alternative_titles".to_string());
        }
        fields
    }
}

impl Default for MetadataEdit {
    fn default() -> Self {
        Self::new()
    }
}

/// Custom artwork for a MediaItem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomArtwork {
    pub poster: Option<Vec<u8>>, // Raw image data
    pub backdrop: Option<Vec<u8>>,
}

/// Alternative titles for improved search matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeTitle {
    pub title: String,
    pub language: Option<String>,
}

// ============================================================================
// Playback Integration Models
// ============================================================================

/// Unique identifier for a PlaybackHistory record
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlaybackHistoryId(pub i64);

impl fmt::Display for PlaybackHistoryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A playback history record for a file version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackHistory {
    pub id: PlaybackHistoryId,
    pub file_version_id: FileVersionId,
    pub started_at: DateTime<Utc>,
    pub stopped_at: Option<DateTime<Utc>>,
    pub playback_position: Option<u64>, // seconds
    pub duration: Option<u64>, // seconds
    pub completed: bool,
}

/// Playback event for recording playback activity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackEvent {
    pub file_version_id: FileVersionId,
    pub started_at: DateTime<Utc>,
    pub stopped_at: Option<DateTime<Utc>>,
    pub playback_position: Option<u64>, // seconds
    pub duration: Option<u64>, // seconds
    pub completed: bool,
}

/// Resume information for a file version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeInfo {
    pub file_version_id: FileVersionId,
    pub last_position: u64, // seconds
    pub last_watched_at: DateTime<Utc>,
}
