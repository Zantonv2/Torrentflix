# Library Management System Design

## Overview

The Library Management System is a comprehensive media organization solution designed for HDD-based storage on Linux systems with modest hardware. It provides version-aware media tracking, intelligent deduplication, automated import workflows, integrity verification, and safe cleanup operations. The system is built on a database-first architecture that minimizes filesystem I/O, optimizes for sequential access patterns, and provides robust safety mechanisms through staging, soft-deletion, and audit logging.

The design emphasizes:
- **HDD-optimized I/O patterns**: Sequential access, batching, and throttling
- **Linux-native integration**: inotify, POSIX APIs, filesystem features, desktop integration
- **Safety-first operations**: Staging, atomic moves, soft-delete, audit trails
- **Scalability**: Database-backed queries, lazy loading, incremental processing
- **Flexibility**: Multiple versions per media, user metadata, collections, automation rules
- **User-centric workflows**: Intuitive browsing, powerful search/filtering, collections for organization, watch folders for automation
- **Transparency**: Audit logging, notifications, storage analytics, job monitoring

## User Workflows

Based on the user guide, the system supports these primary workflows:

### 1. Library Setup and Configuration
- User adds one or more Library Roots (storage directories)
- User configures settings (trash grace period, hashing concurrency, cleanup thresholds)
- System is ready to import media

### 2. Import Workflow
- **Manual Import**: User selects files → System stages → probes metadata → user commits
- **Watch Folder Import**: User designates watch folder → System monitors → auto-imports on new files
- **Import Profiles**: User creates profiles with rules → System auto-applies tags, collections, quality filters

### 3. Browsing and Discovery
- User opens library view (grid or list)
- User searches by title (case-insensitive, fuzzy matching)
- User applies filters (year, resolution, quality, tags, rating, watch status)
- User saves frequently-used searches
- System displays results with lazy-loaded posters and version counts

### 4. Media Item Management
- User views MediaItem details (metadata, all versions, ratings, watch status)
- User edits metadata (title, year, overview, genres, cast, director)
- User adds custom notes and tags
- User sets watch status (unwatched, in progress, watched)
- User sets user rating (0.0-10.0)
- User uploads custom artwork

### 5. Version Management
- User views all FileVersions for a MediaItem
- User compares versions (resolution, codec, size, quality score)
- User sets preferred version (protected during cleanup)
- User identifies duplicates and variants
- User deletes lower-quality versions

### 6. Collections
- **Manual Collections**: User creates custom groupings (e.g., "Marvel Movies", "Favorites")
- **Smart Collections**: User creates dynamic collections with filter criteria (e.g., "4K Movies", "Unwatched Favorites")
- Collections auto-update as library changes
- User exports collections as file lists

### 7. Storage Management
- User views storage analytics (total size, breakdown by resolution/quality/status)
- User identifies duplicates and variants
- User reviews cleanup candidates (exact duplicates, lower-quality variants, trashed files, missing records)
- User estimates space savings
- User executes cleanup with confirmation

### 8. Maintenance
- User runs rescan to detect new/missing files
- User runs integrity checks to detect corruption
- User views audit logs to trace operations
- User monitors background jobs (rescan, hashing, cleanup)
- User receives notifications (job completion, corruption, disk space warnings, missing files)

### 9. Playback Integration
- User selects FileVersion for playback
- System launches configured media player
- System records playback position for resume
- System tracks watch status and last watched date
- User can manually mark as watched

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         UI Layer (Svelte)                        │
│  Library Browser │ Version Manager │ Collections │ Jobs Dashboard│
└────────────────────────────┬────────────────────────────────────┘
                             │ Tauri Commands
┌────────────────────────────┴────────────────────────────────────┐
│                    Engine Layer (Rust/Tokio)                     │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │   Library    │  │    Import    │  │  Maintenance │          │
│  │   Manager    │  │   Pipeline   │  │   Manager    │          │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘          │
│         │                  │                  │                   │
│  ┌──────┴──────────────────┴──────────────────┴───────┐         │
│  │           Database Layer (SQLite + sqlx)            │         │
│  │  MediaItems │ FileVersions │ Collections │ Jobs    │         │
│  └──────────────────────────┬──────────────────────────┘         │
└─────────────────────────────┴────────────────────────────────────┘
                              │
┌─────────────────────────────┴────────────────────────────────────┐
│                    Filesystem Layer (Linux)                       │
│  Library Roots │ Staging │ Trash │ Watch Folders │ Symlinks     │
└──────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

**Library Manager**
- MediaItem and FileVersion CRUD operations
- Query and filtering with database indexes
- Version comparison and quality scoring
- User metadata management (tags, ratings, notes)
- Collection management (manual and smart)

**Import Pipeline**
- File staging and validation
- Metadata probing (technical and descriptive)
- Fingerprint computation (fast and full hash)
- Atomic commit to library roots
- Watch folder monitoring (inotify)
- Import profile matching and rule application

**Maintenance Manager**
- Rescan and integrity checking
- Duplicate detection and cleanup suggestions
- Soft-delete and trash management
- Background hashing jobs
- Database optimization (VACUUM, index rebuild)
- Audit log management

**Job System Integration**
- Scheduling background operations
- Progress tracking and cancellation
- I/O throttling for HDD optimization
- Job persistence and resumability

**Notification System**
- Event-driven alerts
- Desktop integration (libnotify/D-Bus)
- Priority-based notification queue
- Auto-dismissal of old notifications

## Components and Interfaces

### 1. Library Manager Component

**Purpose**: Core component for managing MediaItems, FileVersions, and library queries.

**Public Interface**:
```rust
pub struct LibraryManager {
    db: Arc<DatabaseConnection>,
    config: LibraryConfig,
}

impl LibraryManager {
    // MediaItem operations
    pub async fn create_media_item(&self, metadata: MediaMetadata) -> Result<MediaItemId>;
    pub async fn get_media_item(&self, id: MediaItemId) -> Result<MediaItem>;
    pub async fn update_media_item(&self, id: MediaItemId, updates: MediaItemUpdate) -> Result<()>;
    pub async fn delete_media_item(&self, id: MediaItemId) -> Result<()>;
    
    // FileVersion operations
    pub async fn add_file_version(&self, media_id: MediaItemId, version: FileVersionData) -> Result<FileVersionId>;
    pub async fn get_file_versions(&self, media_id: MediaItemId) -> Result<Vec<FileVersion>>;
    pub async fn update_file_version(&self, id: FileVersionId, updates: FileVersionUpdate) -> Result<()>;
    pub async fn set_preferred_version(&self, id: FileVersionId) -> Result<()>;
    
    // Query and filtering
    pub async fn query_library(&self, filters: LibraryFilters) -> Result<Vec<MediaItem>>;
    pub async fn search_library(&self, query: &str, options: SearchOptions) -> Result<Vec<MediaItem>>;
    pub async fn get_duplicates(&self, strategy: DuplicateStrategy) -> Result<Vec<DuplicateGroup>>;
    
    // User metadata
    pub async fn set_user_rating(&self, media_id: MediaItemId, rating: f32) -> Result<()>;
    pub async fn add_tags(&self, media_id: MediaItemId, tags: Vec<String>) -> Result<()>;
    pub async fn set_watch_status(&self, media_id: MediaItemId, status: WatchStatus) -> Result<()>;
    
    // Collections
    pub async fn create_collection(&self, name: String, description: Option<String>) -> Result<CollectionId>;
    pub async fn add_to_collection(&self, collection_id: CollectionId, media_ids: Vec<MediaItemId>) -> Result<()>;
    pub async fn create_smart_collection(&self, name: String, criteria: FilterCriteria) -> Result<CollectionId>;
    pub async fn get_collection_items(&self, collection_id: CollectionId) -> Result<Vec<MediaItem>>;
}
```

**Dependencies**:
- DatabaseConnection for persistence
- MetadataProvider for enrichment
- QualityScorer for version ranking

### 2. Import Pipeline Component

**Purpose**: Handles file import workflow from staging through commit.

**Public Interface**:
```rust
pub struct ImportPipeline {
    staging_dir: PathBuf,
    library_roots: Vec<LibraryRoot>,
    db: Arc<DatabaseConnection>,
    metadata_prober: MetadataProber,
    hasher: FileHasher,
}

impl ImportPipeline {
    // Staging operations
    pub async fn stage_file(&self, source: PathBuf) -> Result<StagedFile>;
    pub async fn probe_staged_file(&self, staged_id: StagedFileId) -> Result<TechnicalMetadata>;
    pub async fn compute_fast_fingerprint(&self, staged_id: StagedFileId) -> Result<Fingerprint>;
    
    // Commit operations
    pub async fn commit_staged_file(&self, staged_id: StagedFileId, target_root: LibraryRootId) -> Result<FileVersionId>;
    pub async fn rollback_staged_file(&self, staged_id: StagedFileId) -> Result<()>;
    
    // Watch folder operations
    pub async fn add_watch_folder(&self, path: PathBuf, profile: Option<ImportProfile>) -> Result<WatchFolderId>;
    pub async fn remove_watch_folder(&self, id: WatchFolderId) -> Result<()>;
    pub async fn process_watch_event(&self, event: InotifyEvent) -> Result<()>;
    
    // Import profiles
    pub async fn create_import_profile(&self, profile: ImportProfile) -> Result<ImportProfileId>;
    pub async fn match_import_profile(&self, file_path: &Path) -> Result<Option<ImportProfile>>;
    pub async fn apply_import_rules(&self, file: &StagedFile, profile: &ImportProfile) -> Result<ImportActions>;
}
```

**Dependencies**:
- LibraryManager for MediaItem/FileVersion creation
- MetadataProber (ffprobe wrapper) for technical metadata
- FileHasher for fingerprint computation
- InotifyWatcher for watch folder monitoring

### 3. Maintenance Manager Component

**Purpose**: Handles library maintenance, integrity, and cleanup operations.

**Public Interface**:
```rust
pub struct MaintenanceManager {
    db: Arc<DatabaseConnection>,
    library_roots: Vec<LibraryRoot>,
    job_manager: Arc<JobManager>,
}

impl MaintenanceManager {
    // Rescan operations
    pub async fn schedule_rescan(&self, root_id: Option<LibraryRootId>) -> Result<JobId>;
    pub async fn perform_rescan(&self, root_id: LibraryRootId, progress: ProgressReporter) -> Result<RescanReport>;
    
    // Integrity operations
    pub async fn schedule_integrity_check(&self, options: IntegrityCheckOptions) -> Result<JobId>;
    pub async fn verify_file_integrity(&self, version_id: FileVersionId) -> Result<IntegrityResult>;
    
    // Cleanup operations
    pub async fn identify_cleanup_candidates(&self, criteria: CleanupCriteria) -> Result<Vec<CleanupCandidate>>;
    pub async fn estimate_space_savings(&self, candidates: &[CleanupCandidate]) -> Result<u64>;
    pub async fn execute_cleanup(&self, candidates: Vec<CleanupCandidate>) -> Result<CleanupReport>;
    
    // Trash management
    pub async fn soft_delete_version(&self, version_id: FileVersionId) -> Result<()>;
    pub async fn restore_from_trash(&self, version_id: FileVersionId) -> Result<()>;
    pub async fn purge_trash(&self, older_than: Duration) -> Result<PurgeReport>;
    
    // Background hashing
    pub async fn schedule_hashing_job(&self, batch_size: usize) -> Result<JobId>;
    pub async fn compute_full_hash(&self, version_id: FileVersionId) -> Result<Hash>;
    
    // Database maintenance
    pub async fn vacuum_database(&self) -> Result<()>;
    pub async fn rebuild_indexes(&self) -> Result<()>;
    pub async fn cleanup_orphaned_records(&self) -> Result<CleanupStats>;
}
```

**Dependencies**:
- JobManager for background task scheduling
- FileHasher for integrity verification
- AuditLogger for operation tracking

### 4. Metadata Prober Component

**Purpose**: Extracts technical metadata from media files.

**Public Interface**:
```rust
pub struct MetadataProber {
    ffprobe_path: PathBuf,
}

impl MetadataProber {
    pub async fn probe_file(&self, path: &Path) -> Result<TechnicalMetadata>;
    pub async fn extract_video_info(&self, path: &Path) -> Result<VideoInfo>;
    pub async fn extract_audio_info(&self, path: &Path) -> Result<Vec<AudioTrack>>;
    pub async fn extract_subtitle_info(&self, path: &Path) -> Result<Vec<SubtitleTrack>>;
    pub async fn get_duration(&self, path: &Path) -> Result<Duration>;
}

pub struct TechnicalMetadata {
    pub container: String,
    pub video: Option<VideoInfo>,
    pub audio_tracks: Vec<AudioTrack>,
    pub subtitle_tracks: Vec<SubtitleTrack>,
    pub duration: Duration,
    pub bitrate: u64,
    pub file_size: u64,
}

pub struct VideoInfo {
    pub codec: String,
    pub resolution: Resolution,
    pub framerate: f32,
    pub bitrate: u64,
    pub color_space: Option<String>,
}
```

**Dependencies**:
- ffprobe binary (external)
- serde_json for parsing ffprobe output

### 5. File Hasher Component

**Purpose**: Computes file fingerprints for duplicate detection and integrity verification.

**Public Interface**:
```rust
pub struct FileHasher {
    fast_hash_size: usize, // bytes to read for fast hash
    buffer_size: usize,    // I/O buffer size (HDD-optimized)
}

impl FileHasher {
    pub async fn compute_fast_hash(&self, path: &Path) -> Result<FastHash>;
    pub async fn compute_full_hash(&self, path: &Path, algorithm: HashAlgorithm) -> Result<FullHash>;
    pub async fn verify_checksum(&self, path: &Path, expected: &FullHash) -> Result<bool>;
}

pub enum HashAlgorithm {
    Sha256,
    Blake3,
}

pub struct FastHash(pub [u8; 32]); // First 1MB + last 1MB + size
pub struct FullHash {
    pub algorithm: HashAlgorithm,
    pub value: Vec<u8>,
}
```

**Dependencies**:
- sha2 or blake3 crate for hashing
- tokio::fs for async file I/O

### 6. Quality Scorer Component

**Purpose**: Ranks FileVersions by quality for duplicate resolution and preferred version selection.

**Public Interface**:
```rust
pub struct QualityScorer {
    weights: QualityWeights,
}

impl QualityScorer {
    pub fn score_version(&self, version: &FileVersion) -> QualityScore;
    pub fn compare_versions(&self, a: &FileVersion, b: &FileVersion) -> Ordering;
    pub fn rank_versions(&self, versions: &[FileVersion]) -> Vec<(FileVersionId, QualityScore)>;
}

pub struct QualityScore {
    pub total: f32,
    pub resolution_score: f32,
    pub codec_score: f32,
    pub source_score: f32,
    pub size_score: f32,
}

pub struct QualityWeights {
    pub resolution: f32,
    pub codec: f32,
    pub source: f32,
    pub size: f32,
}
```

**Dependencies**: None (pure computation)

### 7. Watch Folder Monitor Component

**Purpose**: Monitors directories for new files using Linux inotify.

**Public Interface**:
```rust
pub struct WatchFolderMonitor {
    inotify: Inotify,
    watch_folders: HashMap<WatchFolderId, WatchFolder>,
    import_pipeline: Arc<ImportPipeline>,
}

impl WatchFolderMonitor {
    pub async fn add_watch(&self, path: PathBuf, profile: Option<ImportProfile>) -> Result<WatchFolderId>;
    pub async fn remove_watch(&self, id: WatchFolderId) -> Result<()>;
    pub async fn start_monitoring(&self) -> Result<()>;
    pub async fn handle_event(&self, event: InotifyEvent) -> Result<()>;
}

struct WatchFolder {
    id: WatchFolderId,
    path: PathBuf,
    profile: Option<ImportProfile>,
    recursive: bool,
}
```

**Dependencies**:
- inotify crate for Linux filesystem events
- ImportPipeline for automatic import

### 8. Notification Manager Component

**Purpose**: Manages user notifications and desktop integration.

**Public Interface**:
```rust
pub struct NotificationManager {
    backend: NotificationBackend,
    notifications: Arc<Mutex<Vec<Notification>>>,
}

impl NotificationManager {
    pub async fn send_notification(&self, notification: Notification) -> Result<NotificationId>;
    pub async fn get_unread_notifications(&self) -> Result<Vec<Notification>>;
    pub async fn mark_as_read(&self, id: NotificationId) -> Result<()>;
    pub async fn dismiss_notification(&self, id: NotificationId) -> Result<()>;
    pub async fn cleanup_old_notifications(&self, older_than: Duration) -> Result<usize>;
}

pub struct Notification {
    pub id: NotificationId,
    pub severity: NotificationSeverity,
    pub title: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub read: bool,
}

pub enum NotificationSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

enum NotificationBackend {
    Libnotify,
    DBus,
    Fallback,
}
```

**Dependencies**:
- notify-rust or dbus crate for Linux desktop notifications

## UI/UX Design Principles

Based on user workflows, the UI should implement:

### 1. Library Browsing
- **Grid View**: Poster thumbnails with title, year, version count, total size
- **List View**: Detailed rows with sortable columns
- **Lazy Loading**: Posters load as user scrolls
- **Sorting**: By title, year, added date, size, version count, user rating
- **Pagination**: Efficient loading of large libraries

### 2. Search and Filtering
- **Quick Search**: Case-insensitive title search with fuzzy matching
- **Advanced Filters**: Year range, resolution, quality label, status, tags, rating, watch status
- **Filter Combinations**: AND logic for multiple filters, OR logic within a filter
- **Saved Searches**: Quick access to frequently-used search combinations
- **Search Highlighting**: Matching terms highlighted in results

### 3. Detail Panel
- **Metadata Display**: Title, year, overview, genres, cast, director, runtime, ratings
- **Editable Fields**: User can edit title, year, overview, genres, cast, director
- **User Metadata**: Rating (0-10), watch status, custom notes, tags
- **Version List**: All FileVersions with path, size, resolution, codec, status, preferred flag
- **Version Comparison**: Side-by-side quality scores and technical details
- **Duplicate Detection**: Exact duplicates and variants highlighted

### 4. Collections
- **Collection List**: All manual and smart collections
- **Collection Details**: Items in collection with standard library browsing
- **Smart Collection Editor**: Visual filter builder for criteria
- **Collection Export**: Download as JSON or CSV

### 5. Storage Analytics
- **Statistics Dashboard**: Total size, item count, version count, average versions per media
- **Breakdown Charts**: By resolution, quality label, status
- **Duplicate Analysis**: Exact duplicates and variants with space savings
- **Largest Items**: Top media items by total size
- **Cleanup Suggestions**: Candidates with space savings estimate

### 6. Jobs Dashboard
- **Job List**: Running, completed, and failed jobs
- **Progress Bars**: Visual progress for long-running jobs
- **Job Details**: Type, status, progress percentage, timestamps
- **Job Control**: Cancel running jobs
- **Job Results**: Summary of completed jobs

### 7. Notifications
- **Notification Center**: List of unread notifications
- **Severity Levels**: Info, warning, error, critical
- **Auto-dismiss**: Old notifications automatically dismissed
- **Desktop Integration**: Native notifications for important events

## Data Models

### Database Schema

```sql
-- MediaItems table: logical media entities
CREATE TABLE media_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    normalized_title TEXT NOT NULL,
    original_title TEXT,
    year INTEGER,
    media_type TEXT NOT NULL CHECK(media_type IN ('movie', 'series')),
    overview TEXT,
    genres TEXT, -- JSON array
    cast TEXT, -- JSON array
    director TEXT,
    runtime INTEGER, -- seconds
    poster_url TEXT,
    poster_path TEXT, -- local cached poster
    backdrop_url TEXT,
    backdrop_path TEXT,
    external_ids TEXT, -- JSON object (tmdb_id, imdb_id, etc.)
    user_rating REAL,
    watch_status TEXT CHECK(watch_status IN ('unwatched', 'in_progress', 'watched')),
    last_watched_at DATETIME,
    custom_notes TEXT,
    metadata_source TEXT,
    metadata_fetched_at DATETIME,
    user_edited_fields TEXT, -- JSON array of field names
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_media_items_normalized_title ON media_items(normalized_title);
CREATE INDEX idx_media_items_year ON media_items(year);
CREATE INDEX idx_media_items_watch_status ON media_items(watch_status);
CREATE INDEX idx_media_items_user_rating ON media_items(user_rating);

-- FileVersions table: physical files on disk
CREATE TABLE file_versions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    media_item_id INTEGER NOT NULL,
    library_root_id INTEGER NOT NULL,
    relative_path TEXT NOT NULL,
    absolute_path TEXT NOT NULL, -- computed from root + relative
    file_size INTEGER NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('present', 'missing', 'trashed', 'corrupted')),
    is_preferred BOOLEAN NOT NULL DEFAULT 0,
    
    -- Technical metadata
    container TEXT,
    resolution_width INTEGER,
    resolution_height INTEGER,
    video_codec TEXT,
    audio_codec TEXT,
    audio_tracks TEXT, -- JSON array
    subtitle_tracks TEXT, -- JSON array
    duration INTEGER, -- seconds
    bitrate INTEGER,
    framerate REAL,
    
    -- Quality and classification
    quality_label TEXT,
    release_group TEXT,
    source_type TEXT, -- BluRay, WEB-DL, HDTV, etc.
    
    -- Fingerprints and hashing
    fast_hash BLOB,
    full_hash BLOB,
    hash_algorithm TEXT,
    hashing_status TEXT CHECK(hashing_status IN ('pending', 'in_progress', 'complete', 'failed')),
    hashed_at DATETIME,
    
    -- Integrity
    checksum BLOB,
    checksum_algorithm TEXT,
    last_verified_at DATETIME,
    corruption_detected_at DATETIME,
    
    -- Filesystem metadata
    is_symlink BOOLEAN NOT NULL DEFAULT 0,
    symlink_target TEXT,
    is_hardlink BOOLEAN NOT NULL DEFAULT 0,
    inode INTEGER,
    
    -- Timestamps
    added_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_seen_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    trashed_at DATETIME,
    original_path TEXT, -- path before moving to trash
    
    -- Import metadata
    import_source TEXT,
    import_profile_id INTEGER,
    
    FOREIGN KEY (media_item_id) REFERENCES media_items(id) ON DELETE CASCADE,
    FOREIGN KEY (library_root_id) REFERENCES library_roots(id),
    FOREIGN KEY (import_profile_id) REFERENCES import_profiles(id)
);

CREATE INDEX idx_file_versions_media_item ON file_versions(media_item_id);
CREATE INDEX idx_file_versions_status ON file_versions(status);
CREATE INDEX idx_file_versions_fast_hash ON file_versions(fast_hash);
CREATE INDEX idx_file_versions_full_hash ON file_versions(full_hash);
CREATE INDEX idx_file_versions_quality_label ON file_versions(quality_label);
CREATE INDEX idx_file_versions_resolution ON file_versions(resolution_width, resolution_height);
CREATE INDEX idx_file_versions_inode ON file_versions(inode);

-- LibraryRoots table: storage locations
CREATE TABLE library_roots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT 1,
    is_archive BOOLEAN NOT NULL DEFAULT 0, -- cold storage flag
    filesystem_type TEXT,
    mount_point TEXT,
    total_space INTEGER,
    free_space INTEGER,
    last_scanned_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Tags table: user-defined tags
CREATE TABLE tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    color TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- MediaItemTags junction table
CREATE TABLE media_item_tags (
    media_item_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (media_item_id, tag_id),
    FOREIGN KEY (media_item_id) REFERENCES media_items(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

CREATE INDEX idx_media_item_tags_tag ON media_item_tags(tag_id);

-- Collections table: user-defined groupings
CREATE TABLE collections (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT,
    is_smart BOOLEAN NOT NULL DEFAULT 0,
    filter_criteria TEXT, -- JSON object for smart collections
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- CollectionItems junction table
CREATE TABLE collection_items (
    collection_id INTEGER NOT NULL,
    media_item_id INTEGER NOT NULL,
    added_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (collection_id, media_item_id),
    FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE CASCADE,
    FOREIGN KEY (media_item_id) REFERENCES media_items(id) ON DELETE CASCADE
);

CREATE INDEX idx_collection_items_media ON collection_items(media_item_id);

-- WatchFolders table: monitored directories
CREATE TABLE watch_folders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL UNIQUE,
    is_active BOOLEAN NOT NULL DEFAULT 1,
    recursive BOOLEAN NOT NULL DEFAULT 1,
    import_profile_id INTEGER,
    last_scan_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (import_profile_id) REFERENCES import_profiles(id)
);

-- ImportProfiles table: automation rules
CREATE TABLE import_profiles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT,
    priority INTEGER NOT NULL DEFAULT 0,
    file_pattern TEXT, -- glob or regex pattern
    min_resolution_width INTEGER,
    min_resolution_height INTEGER,
    allowed_codecs TEXT, -- JSON array
    auto_tags TEXT, -- JSON array of tag names
    target_collection_id INTEGER,
    target_library_root_id INTEGER,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (target_collection_id) REFERENCES collections(id),
    FOREIGN KEY (target_library_root_id) REFERENCES library_roots(id)
);

-- PlaybackHistory table: watch tracking
CREATE TABLE playback_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_version_id INTEGER NOT NULL,
    started_at DATETIME NOT NULL,
    stopped_at DATETIME,
    playback_position INTEGER, -- seconds
    duration INTEGER, -- seconds
    completed BOOLEAN NOT NULL DEFAULT 0,
    FOREIGN KEY (file_version_id) REFERENCES file_versions(id) ON DELETE CASCADE
);

CREATE INDEX idx_playback_history_version ON playback_history(file_version_id);
CREATE INDEX idx_playback_history_started ON playback_history(started_at);

-- AuditLog table: operation tracking
CREATE TABLE audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    operation_type TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id INTEGER,
    old_value TEXT, -- JSON
    new_value TEXT, -- JSON
    initiator TEXT NOT NULL, -- 'user' or 'system:job_name'
    error_message TEXT,
    timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_audit_log_timestamp ON audit_log(timestamp);
CREATE INDEX idx_audit_log_entity ON audit_log(entity_type, entity_id);
CREATE INDEX idx_audit_log_operation ON audit_log(operation_type);

-- Notifications table
CREATE TABLE notifications (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    severity TEXT NOT NULL CHECK(severity IN ('info', 'warning', 'error', 'critical')),
    title TEXT NOT NULL,
    message TEXT NOT NULL,
    read BOOLEAN NOT NULL DEFAULT 0,
    dismissed BOOLEAN NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    read_at DATETIME,
    dismissed_at DATETIME
);

CREATE INDEX idx_notifications_read ON notifications(read);
CREATE INDEX idx_notifications_created ON notifications(created_at);

-- StagedFiles table: import staging
CREATE TABLE staged_files (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    original_path TEXT NOT NULL,
    staged_path TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('pending', 'probing', 'ready', 'committing', 'failed')),
    error_message TEXT,
    technical_metadata TEXT, -- JSON
    fast_hash BLOB,
    matched_profile_id INTEGER,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (matched_profile_id) REFERENCES import_profiles(id)
);
```

### Rust Domain Models

```rust
// Core domain models
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
    pub runtime: Option<Duration>,
    pub poster: Option<ImageData>,
    pub backdrop: Option<ImageData>,
    pub external_ids: ExternalIds,
    pub user_rating: Option<f32>,
    pub watch_status: WatchStatus,
    pub last_watched_at: Option<DateTime<Utc>>,
    pub custom_notes: Option<String>,
    pub tags: Vec<Tag>,
    pub metadata_source: Option<String>,
    pub metadata_fetched_at: Option<DateTime<Utc>>,
    pub user_edited_fields: HashSet<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct FileVersion {
    pub id: FileVersionId,
    pub media_item_id: MediaItemId,
    pub library_root_id: LibraryRootId,
    pub relative_path: PathBuf,
    pub absolute_path: PathBuf,
    pub file_size: u64,
    pub status: FileStatus,
    pub is_preferred: bool,
    pub technical_metadata: TechnicalMetadata,
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

pub struct Collection {
    pub id: CollectionId,
    pub name: String,
    pub description: Option<String>,
    pub collection_type: CollectionType,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum CollectionType {
    Manual,
    Smart { criteria: FilterCriteria },
}

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

// Enums
pub enum MediaType {
    Movie,
    Series,
}

pub enum FileStatus {
    Present,
    Missing,
    Trashed,
    Corrupted,
}

pub enum WatchStatus {
    Unwatched,
    InProgress,
    Watched,
}

pub enum HashingStatus {
    Pending,
    InProgress,
    Complete,
    Failed,
}

pub enum SourceType {
    BluRay,
    WebDl,
    Hdtv,
    Dvd,
    Unknown,
}

// Supporting structures
pub struct FilesystemInfo {
    pub is_symlink: bool,
    pub symlink_target: Option<PathBuf>,
    pub is_hardlink: bool,
    pub inode: Option<u64>,
}

pub struct QualityFilters {
    pub min_resolution: Option<Resolution>,
    pub allowed_codecs: Option<Vec<String>>,
    pub min_bitrate: Option<u64>,
}

pub struct FilterCriteria {
    pub title_pattern: Option<String>,
    pub year_range: Option<(i32, i32)>,
    pub quality_labels: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub min_rating: Option<f32>,
    pub watch_status: Option<WatchStatus>,
}
```

## Corr
ectness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system—essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Media Item and Version Management Properties

Property 1: MediaItem creation with normalized metadata
*For any* file import with title and year metadata, creating a MediaItem should result in a record with properly normalized title (lowercase, trimmed, special characters handled) and year value stored.
**Validates: Requirements 1.1**

Property 2: FileVersion completeness
*For any* committed file, the FileVersion record should contain all required fields: absolute path, file size, resolution, codec, container format, quality label, fingerprint, added timestamp, and status set to "present".
**Validates: Requirements 1.2**

Property 3: Version-to-MediaItem association
*For any* set of files with identical normalized title and year, all resulting FileVersion records should reference the same MediaItemId.
**Validates: Requirements 1.3**

Property 4: Version retrieval completeness
*For any* MediaItem, querying its FileVersions should return all associated versions with complete technical metadata and status information.
**Validates: Requirements 1.4**

Property 5: Preferred flag persistence
*For any* FileVersion marked as preferred, retrieving that version should show the preferred flag as true, and it should persist across application restarts.
**Validates: Requirements 1.5**

### Import Pipeline Properties

Property 6: Staging directory isolation
*For any* file selected for import, the file should be moved or copied to the staging directory before any processing begins.
**Validates: Requirements 2.1**

Property 7: Metadata probing completeness
*For any* staged file, probing should extract and store container format, resolution, codec, duration, and audio track information.
**Validates: Requirements 2.2**

Property 8: Fast fingerprint computation
*For any* staged file after metadata probing, a fast fingerprint should be computed and stored.
**Validates: Requirements 2.3**

Property 9: Atomic commit operation
*For any* staged file commit, either the file is successfully moved to the library root with database records created, or the file remains in staging with no database changes (atomicity).
**Validates: Requirements 2.4, 2.5**

Property 10: Post-commit database state
*For any* successfully committed file, the database should contain a MediaItem (new or existing) and a FileVersion with status "present".
**Validates: Requirements 2.6**

Property 11: Staging cleanup
*For any* completed commit operation (success or failure), the staging directory should not contain the processed file.
**Validates: Requirements 2.7**

### Hashing and Fingerprinting Properties

Property 12: Initial hashing status
*For any* newly created FileVersion, the hashing_status field should be set to "pending".
**Validates: Requirements 3.1**

Property 13: Hash computation for pending versions
*For any* FileVersion with hashing_status "pending", when a hashing job runs, the full-file hash should be computed and stored.
**Validates: Requirements 3.2**

Property 14: Hashing concurrency limit
*For any* time period during hashing operations, at most one file should be actively hashed to prevent I/O overload.
**Validates: Requirements 3.3**

Property 15: Hash storage and status update
*For any* FileVersion after successful hash computation, the full_hash field should contain the hash value and hashing_status should be "complete".
**Validates: Requirements 3.4**

Property 16: Duplicate identification by hash
*For any* two FileVersions with identical full_hash values, they should be identified as exact duplicates.
**Validates: Requirements 3.6**

### Library Browsing and Filtering Properties

Property 17: Library query completeness
*For any* library view request, the returned MediaItems should include title, year, poster thumbnail, version count, and total size for each item.
**Validates: Requirements 4.1**

Property 18: Filter application correctness
*For any* set of filters applied, the returned MediaItems should match all specified criteria (title, year, resolution, quality label, status, tags, rating).
**Validates: Requirements 4.2**

Property 19: Case-insensitive search
*For any* text search query, results should include MediaItems where the normalized title matches the query regardless of case.
**Validates: Requirements 4.3**

Property 20: Version listing completeness
*For any* expanded MediaItem, all associated FileVersions should be returned with path, resolution, codec, size, status, preferred flag, and timestamps.
**Validates: Requirements 4.4**

Property 21: Sort order correctness
*For any* sort field (title, year, added date, size, version count, user rating), the returned MediaItems should be ordered correctly by that field.
**Validates: Requirements 4.5**


### Rescan and Integrity Properties

Property 22: Rescan directory traversal
*For any* rescan job on a library root, all subdirectories should be recursively visited and processed.
**Validates: Requirements 5.1**

Property 23: New file detection
*For any* file present on disk but not in the database, a rescan should flag it as a new import candidate.
**Validates: Requirements 5.2**

Property 24: Missing file detection
*For any* FileVersion record whose path no longer exists on disk, a rescan should update its status to "missing".
**Validates: Requirements 5.3**

Property 25: Incomplete MediaItem flagging
*For any* MediaItem where all FileVersions have status "missing", the MediaItem should be flagged as incomplete.
**Validates: Requirements 5.4**

Property 26: Rescan report accuracy
*For any* completed rescan job, the summary report should accurately count new files, missing files, and status changes.
**Validates: Requirements 5.6**

Property 27: Integrity check verification
*For any* FileVersion during integrity check, if the file exists and its size matches the recorded value, verification should pass; otherwise it should fail.
**Validates: Requirements 5.7**

### Duplicate Detection and Version Management Properties

Property 28: Exact duplicate detection
*For any* two FileVersions with identical full_hash values, duplicate detection should identify them as exact duplicates.
**Validates: Requirements 6.1**

Property 29: Variant detection
*For any* MediaItem with multiple FileVersions, variant detection should identify all versions as variants of the same media.
**Validates: Requirements 6.2**

Property 30: Comparison data completeness
*For any* duplicate or variant comparison, the displayed data should include resolution, codec, size, quality label, and hash values for each version.
**Validates: Requirements 6.3**

Property 31: Quality score calculation
*For any* MediaItem with multiple versions, each FileVersion should have a calculated quality score based on resolution, codec, source, and file size.
**Validates: Requirements 6.4**

Property 32: Default version recommendation
*For any* MediaItem without a user-set preferred version, the version with the highest quality score should be recommended as default.
**Validates: Requirements 6.5**

Property 33: Preferred version override
*For any* MediaItem where a user marks a version as preferred, that version should be prioritized regardless of its quality score.
**Validates: Requirements 6.6**

### Soft Delete and Trash Management Properties

Property 34: Soft delete operation
*For any* FileVersion deletion, the status should be updated to "trashed" and the file should be moved to the trash directory (not permanently deleted).
**Validates: Requirements 7.1**

Property 35: Trash metadata preservation
*For any* trashed FileVersion, the deletion timestamp and original path should be recorded in the FileVersion metadata.
**Validates: Requirements 7.2**

Property 36: Audit log creation for deletion
*For any* file moved to trash, an audit log entry should be created with operation type, timestamp, file path, and initiator.
**Validates: Requirements 7.3**

Property 37: Trashed item exclusion
*For any* default library query, FileVersions with status "trashed" should be excluded from results.
**Validates: Requirements 7.4**

Property 38: Trash view completeness
*For any* trash view request, all FileVersions with status "trashed" should be returned with their deletion timestamps.
**Validates: Requirements 7.5**

Property 39: Restore within grace period
*For any* FileVersion trashed for less than the configured grace period, restoration should move the file back to its original location and update status to "present".
**Validates: Requirements 7.6**

Property 40: Cleanup candidate identification
*For any* FileVersion that has been trashed longer than the grace period, it should be identified as a cleanup candidate.
**Validates: Requirements 7.7**

### Cleanup and Space Management Properties

Property 41: Cleanup candidate types
*For any* cleanup suggestion request, identified candidates should include exact duplicates, lower-quality variants, trashed files past grace period, and missing file records.
**Validates: Requirements 8.1**

Property 42: Candidate information completeness
*For any* cleanup candidate, the displayed information should include file path, size, reason for candidacy, and estimated space savings.
**Validates: Requirements 8.2**

Property 43: Space savings calculation
*For any* set of selected cleanup candidates, the total space savings should equal the sum of all candidate file sizes.
**Validates: Requirements 8.3**

Property 44: Cleanup execution completeness
*For any* confirmed cleanup operation, all selected files should be permanently deleted and their FileVersion records removed from the database.
**Validates: Requirements 8.4**

Property 45: Cleanup error handling
*For any* cleanup operation where some files fail to delete, the operation should log errors, skip failed files, and continue with remaining files.
**Validates: Requirements 8.5**

Property 46: Cleanup report accuracy
*For any* completed cleanup operation, the summary report should accurately list deleted files, freed space, and any errors encountered.
**Validates: Requirements 8.6**

Property 47: Disk usage threshold notification
*For any* time when disk usage exceeds the configured threshold, a notification should be created suggesting cleanup.
**Validates: Requirements 8.7**

### User Metadata Properties

Property 48: User metadata persistence
*For any* user metadata modification (rating, watch status, notes), the changes should be immediately persisted to the database.
**Validates: Requirements 9.2**

Property 49: Tag storage and retrieval
*For any* tags added to a MediaItem, the tags should be stored as a list and retrievable with the MediaItem.
**Validates: Requirements 9.3**

Property 50: Tag filtering correctness
*For any* tag filter applied, the results should include all MediaItems that have at least one of the specified tags.
**Validates: Requirements 9.4**

Property 51: Notes search functionality
*For any* text search on custom notes, results should include MediaItems whose notes contain the search text.
**Validates: Requirements 9.5**

Property 52: Rating display completeness
*For any* MediaItem with both user rating and external ratings, both should be displayed together.
**Validates: Requirements 9.6**


### Storage Analytics Properties

Property 53: Analytics calculation accuracy
*For any* storage analytics request, the displayed totals (library size, MediaItem count, FileVersion count, average versions per media) should match the actual database state.
**Validates: Requirements 10.1**

Property 54: Storage breakdown accuracy
*For any* storage breakdown by category (resolution, quality label, status), the sum of all category sizes should equal the total library size.
**Validates: Requirements 10.2**

Property 55: Duplicate space calculation
*For any* duplicate space calculation, the identified space should equal the total size of all duplicate FileVersions minus the size of one copy of each unique file.
**Validates: Requirements 10.3**

Property 56: Largest media sorting
*For any* per-media storage display, MediaItems should be sorted by total size (sum of all versions) in descending order.
**Validates: Requirements 10.4**

Property 57: Space savings estimation
*For any* space savings estimate, the calculated value should equal the sum of sizes for all duplicates, trashed files, and low-quality variants that would be removed.
**Validates: Requirements 10.5**

### Background Job Management Properties

Property 58: Job listing completeness
*For any* jobs dashboard request, all jobs (scheduled, running, completed) should be returned with type, status, progress, and timestamps.
**Validates: Requirements 11.1**

Property 59: Progress update monotonicity
*For any* running background job, progress percentage should increase monotonically over time until completion.
**Validates: Requirements 11.2**

Property 60: Job enqueueing
*For any* job start request with valid prerequisites, the job should be enqueued for execution.
**Validates: Requirements 11.3**

Property 61: Job cancellation
*For any* running job cancellation request, the job status should be updated to "cancelled" and the job should stop gracefully.
**Validates: Requirements 11.4**

Property 62: Job failure logging
*For any* failed job, the error details should be logged and the job status should be "failed" with an error message.
**Validates: Requirements 11.5**

Property 63: Job completion state
*For any* successfully completed job, the status should be "completed" and any result summary should be stored.
**Validates: Requirements 11.6**

### Multi-Root Library Properties

Property 64: Library root creation
*For any* new library root added, it should be stored with an absolute path and assigned a unique identifier.
**Validates: Requirements 12.1**

Property 65: Path storage decomposition
*For any* FileVersion, both the library root identifier and relative path within that root should be stored.
**Validates: Requirements 12.2**

Property 66: Root path update preservation
*For any* library root path update, all FileVersion associations should be preserved by using the root identifier.
**Validates: Requirements 12.3**

Property 67: Multi-root scanning
*For any* rescan operation, all configured library roots should be recursively scanned.
**Validates: Requirements 12.4**

Property 68: Archive root priority marking
*For any* library root designated as archive/cold storage, associated FileVersions should be marked with lower priority for background operations.
**Validates: Requirements 12.5**

Property 69: Path resolution correctness
*For any* FileVersion, the absolute path should equal the concatenation of the library root path and the relative path.
**Validates: Requirements 12.6**

### Metadata Enrichment Properties

Property 70: External metadata fetching
*For any* MediaItem creation with metadata enrichment enabled, external providers should be queried for overview, genres, cast, runtime, and poster URL.
**Validates: Requirements 13.1**

Property 71: Metadata caching
*For any* fetched external metadata, subsequent requests for the same MediaItem should use cached data without making new API calls.
**Validates: Requirements 13.2**

Property 72: Graceful degradation on provider failure
*For any* external metadata provider failure, the import should continue using parsed metadata without blocking.
**Validates: Requirements 13.3**

Property 73: Metadata refresh update
*For any* metadata refresh request, external providers should be re-queried and cached metadata should be updated.
**Validates: Requirements 13.4**

Property 74: Image caching
*For any* metadata with poster or backdrop URLs, the images should be downloaded and cached locally.
**Validates: Requirements 13.5**

Property 75: Offline metadata access
*For any* MediaItem display, cached metadata and images should be used without requiring network access.
**Validates: Requirements 13.6**

### Audit Logging Properties

Property 76: Audit log entry completeness
*For any* file operation, an audit log entry should be created with operation type, timestamp, file path, old status, new status, and initiator.
**Validates: Requirements 14.1**

Property 77: Move operation logging
*For any* FileVersion move operation, both the old path and new path should be logged.
**Validates: Requirements 14.2**

Property 78: Deletion logging
*For any* FileVersion deletion, the log entry should include file size and deletion reason.
**Validates: Requirements 14.3**

Property 79: Audit log query ordering
*For any* audit log view request, entries should be returned in reverse chronological order (newest first).
**Validates: Requirements 14.4**

Property 80: Error logging
*For any* failed operation, error details and stack trace should be logged in the audit log.
**Validates: Requirements 14.5**

Property 81: Audit log archival
*For any* audit log that exceeds the configured size threshold, old entries should be archived to prevent unbounded growth.
**Validates: Requirements 14.6**

### Watch Folder and Automation Properties

Property 82: Watch folder monitoring setup
*For any* directory designated as a watch folder, Linux inotify should be configured to monitor that directory for new files.
**Validates: Requirements 16.1**

Property 83: Write completion detection
*For any* new file appearing in a watch folder, import should not begin until file write operations are complete.
**Validates: Requirements 16.2**

Property 84: Automatic import trigger
*For any* completed file write in a watch folder, the file should be automatically moved to staging and metadata probing should begin.
**Validates: Requirements 16.3**

Property 85: Recursive watch monitoring
*For any* watch folder with subdirectories, all subdirectories should be recursively monitored for new files.
**Validates: Requirements 16.4**

Property 86: Initial scan on watch folder addition
*For any* newly added watch folder with initial scan enabled, existing files should be imported.
**Validates: Requirements 16.5**

Property 87: Failed import handling
*For any* file in a watch folder that fails import validation, the file should be moved to a failed imports directory and an error should be logged.
**Validates: Requirements 16.6**

Property 88: Watch folder disable
*For any* disabled watch folder, monitoring should stop without affecting existing library content.
**Validates: Requirements 16.7**


### Symbolic Link and Filesystem Properties

Property 89: Symlink target resolution
*For any* symbolic link encountered during scanning, the link target should be resolved and used for the FileVersion path.
**Validates: Requirements 17.1**

Property 90: Symlink path storage
*For any* FileVersion created from a symbolic link, both the link path and resolved target path should be stored.
**Validates: Requirements 17.2**

Property 91: Hard link inode detection
*For any* hard link to a library file, the shared inode should be detected and both paths should be associated with the same FileVersion.
**Validates: Requirements 17.3**

Property 92: Broken symlink handling
*For any* symbolic link whose target does not exist, the FileVersion should be marked as "missing" and a broken link warning should be logged.
**Validates: Requirements 17.4**

Property 93: Filesystem type indication
*For any* FileVersion display, the path type (symbolic link, hard link, or regular file) should be correctly indicated.
**Validates: Requirements 17.5**

Property 94: POSIX path handling
*For any* file path operation, Linux path separators, hidden files (dot-prefixed), and case-sensitive filenames should be correctly processed.
**Validates: Requirements 18.2**

Property 95: Atomic rename within filesystem
*For any* file operation requiring atomicity within the same filesystem, the rename system call should be used to ensure atomicity.
**Validates: Requirements 18.5**

Property 96: Cross-mount-point detection
*For any* file move operation, if source and destination are on different mount points, a copy-then-delete operation should be used instead of rename.
**Validates: Requirements 18.6**

### Collection Management Properties

Property 97: Collection creation
*For any* new collection created, it should be stored with name, description, and creation timestamp.
**Validates: Requirements 19.1**

Property 98: Collection association
*For any* MediaItems added to a collection, associations should be created between the collection and all selected MediaItems.
**Validates: Requirements 19.2**

Property 99: Collection item retrieval
*For any* collection view request, all associated MediaItems should be returned.
**Validates: Requirements 19.3**

Property 100: Smart collection criteria storage
*For any* smart collection created, the filter criteria should be stored including title patterns, year ranges, quality labels, tags, ratings, and version counts.
**Validates: Requirements 19.4**

Property 101: Smart collection dynamic querying
*For any* smart collection view, MediaItems should be dynamically queried to match the stored filter criteria.
**Validates: Requirements 19.5**

Property 102: Smart collection auto-update
*For any* new MediaItem added to the library, it should automatically appear in all smart collections whose criteria it matches.
**Validates: Requirements 19.6**

Property 103: Multi-collection membership
*For any* MediaItem matching multiple smart collection criteria, it should appear in all matching collections.
**Validates: Requirements 19.7**

Property 104: Collection export completeness
*For any* collection export, the generated file should contain paths and metadata for all MediaItems in the collection.
**Validates: Requirements 19.8**

### Advanced Search Properties

Property 105: Multi-field search
*For any* text search, results should include MediaItems where the query matches title, original title, alternative titles, cast names, director names, or custom notes.
**Validates: Requirements 20.1**

Property 106: Multi-filter AND logic
*For any* multiple filters applied simultaneously, results should include only MediaItems matching all filter criteria (AND logic).
**Validates: Requirements 20.2**

Property 107: Saved search persistence
*For any* saved search created, the query and filter combination should be stored for reuse.
**Validates: Requirements 20.3**

Property 108: File property filtering
*For any* file property filter (size range, codec, audio language, subtitle language, container), results should match the specified criteria.
**Validates: Requirements 20.4**

Property 109: Date range filtering
*For any* date filter (added date, release year, last modified, last watched), results should fall within the specified date range.
**Validates: Requirements 20.5**

Property 110: Fuzzy search matching
*For any* fuzzy search query, results should include MediaItems with approximate title matches based on edit distance.
**Validates: Requirements 20.6**

### Batch Operation Properties

Property 111: Batch tag application
*For any* batch of selected MediaItems, applying tags should add the specified tags to all selected items.
**Validates: Requirements 21.2**

Property 112: Batch rating assignment
*For any* batch of selected MediaItems, assigning a rating should update the user rating for all selected items.
**Validates: Requirements 21.3**

Property 113: Batch collection assignment
*For any* batch of selected MediaItems, moving to a collection should create associations for all selected items.
**Validates: Requirements 21.4**

Property 114: Batch deletion
*For any* batch deletion, all FileVersions associated with selected MediaItems should be soft-deleted with audit log entries created for each.
**Validates: Requirements 21.5**

Property 115: Batch operation error handling
*For any* batch operation where some items fail, errors should be logged, failed items skipped, and remaining items processed.
**Validates: Requirements 21.7**

### Export and Backup Properties

Property 116: Metadata export completeness
*For any* metadata export, the generated file should contain all MediaItem and FileVersion records with complete metadata.
**Validates: Requirements 22.1**

Property 117: Full library export
*For any* full library export, collections, smart collections, user tags, ratings, notes, and audit logs should be included.
**Validates: Requirements 22.2**

Property 118: Backup creation
*For any* backup operation, the SQLite database file and optionally cached artwork and metadata should be exported.
**Validates: Requirements 22.3**

Property 119: Export profile filtering
*For any* export with a profile, only MediaItems, collections, or date ranges matching the profile filters should be included.
**Validates: Requirements 22.4**

Property 120: Import merge behavior
*For any* metadata import, imported records should be merged with existing library data with conflicts resolved based on user preference.
**Validates: Requirements 22.5**

### Integrity Verification Properties

Property 121: Checksum storage on commit
*For any* FileVersion committed with integrity verification enabled, a cryptographic checksum should be computed and stored.
**Validates: Requirements 23.1**

Property 122: Integrity verification detection
*For any* FileVersion integrity check, if the recomputed checksum does not match the stored value, the version should be marked as corrupted.
**Validates: Requirements 23.2, 23.3**

Property 123: Corruption alert creation
*For any* detected corrupted file, an alert notification should be created with file path and checksum mismatch details.
**Validates: Requirements 23.3, 23.4**

Property 124: Corruption replacement suggestion
*For any* corrupted FileVersion where other intact versions exist for the same MediaItem, the system should suggest replacing with an intact version.
**Validates: Requirements 23.5**

### Metadata Editing Properties

Property 125: Metadata edit persistence
*For any* manually edited metadata field, the change should be persisted and the field marked as user-edited to prevent automatic overwrites.
**Validates: Requirements 24.2**

Property 126: Metadata field reset
*For any* user-edited field reset, the original value from external providers or parsed metadata should be restored.
**Validates: Requirements 24.3**

Property 127: Alternative title storage
*For any* alternative titles added by the user, all title variants should be stored and used for search matching.
**Validates: Requirements 24.5**

Property 128: Custom artwork storage
*For any* custom artwork uploaded by the user, the image should be stored locally and used instead of provider-sourced posters.
**Validates: Requirements 24.6**

### Playback Integration Properties

Property 129: Playback event recording
*For any* FileVersion playback, a playback event should be recorded with timestamp, file path, and duration.
**Validates: Requirements 25.1**

Property 130: Playback position storage
*For any* playback stop event, the current playback position should be stored for resume functionality.
**Validates: Requirements 25.2**

Property 131: Watch status display
*For any* MediaItem view, the watch status (unwatched, in progress, watched) and last watched date should be displayed.
**Validates: Requirements 25.3**

Property 132: Per-version playback tracking
*For any* MediaItem with multiple FileVersions, playback history should be tracked separately for each version.
**Validates: Requirements 25.4**

### Notification Properties

Property 133: Job completion notification
*For any* completed background job, a notification should be created with job type, status, and summary results.
**Validates: Requirements 26.1**

Property 134: Corruption alert priority
*For any* file integrity check that detects corruption, a high-priority alert notification should be created.
**Validates: Requirements 26.2**

Property 135: Disk space warning
*For any* time when disk space falls below the configured threshold, a warning notification should be created with usage and cleanup suggestions.
**Validates: Requirements 26.3**

Property 136: Missing file notification
*For any* rescan that detects missing files, a notification should be created listing the count and affected MediaItems.
**Validates: Requirements 26.4**

Property 137: Watch folder failure notification
*For any* watch folder import failure, a notification should be created with the failed file path and error reason.
**Validates: Requirements 26.5**

### Import Profile Properties

Property 138: Import profile pattern matching
*For any* file that matches an import profile pattern, the profile's rules should be automatically applied during import.
**Validates: Requirements 28.2**

Property 139: Automatic tagging from profile
*For any* import profile with automatic tagging configured, the specified tags should be added to imported MediaItems.
**Validates: Requirements 28.3**

Property 140: Quality filtering from profile
*For any* import profile with quality thresholds, files below the specified quality should be rejected.
**Validates: Requirements 28.4**

Property 141: Profile collection assignment
*For any* import profile with a target collection, imported MediaItems should be automatically added to that collection.
**Validates: Requirements 28.5**

Property 142: Profile priority resolution
*For any* file matching multiple import profiles, the profile with the highest priority or most specific pattern should be applied.
**Validates: Requirements 28.6**

### Database Maintenance Properties

Property 143: VACUUM space reclamation
*For any* VACUUM operation on the database, freed space from deleted records should be reclaimed.
**Validates: Requirements 29.1**

Property 144: Index optimization
*For any* index rebuild operation, query performance should be optimized by analyzing and rebuilding indexes.
**Validates: Requirements 29.2**

Property 145: Orphaned record cleanup
*For any* orphaned record cleanup, records not associated with any MediaItem or FileVersion should be identified and optionally removed.
**Validates: Requirements 29.3**

Property 146: Corruption recovery
*For any* detected database corruption, automatic recovery should be attempted using SQLite integrity checks and backups.
**Validates: Requirements 29.5**

## Error Handling

The system employs a comprehensive error handling strategy:

**Import Pipeline Errors**:
- File access errors: Retry with exponential backoff, then move to failed imports
- Metadata probing failures: Use fallback parsing, log warning, continue import
- Commit failures: Rollback transaction, keep file in staging, notify user

**Background Job Errors**:
- Transient errors: Retry up to 3 times with backoff
- Permanent errors: Mark job as failed, log details, notify user
- Resource exhaustion: Throttle operations, reschedule for later

**Database Errors**:
- Connection failures: Retry with connection pool
- Constraint violations: Log error, skip operation, continue processing
- Corruption: Attempt recovery, restore from backup if available

**Filesystem Errors**:
- Permission denied: Log error, skip file, continue scanning
- Disk full: Notify user, pause operations, suggest cleanup
- I/O errors: Retry operation, mark file as problematic if persistent

**External API Errors**:
- Network failures: Use cached data, retry with backoff
- Rate limiting: Respect limits, queue requests, use exponential backoff
- Invalid responses: Log error, use fallback data, continue operation

All errors are logged to the audit log with full context for debugging and recovery.

## Testing Strategy

### Unit Testing

Unit tests will cover:
- Individual component methods (LibraryManager, ImportPipeline, MaintenanceManager)
- Data model validation and serialization
- Quality scoring algorithms
- Path resolution and filesystem operations
- Database query correctness
- Filter and search logic

### Property-Based Testing

Property-based tests will be implemented using the `proptest` crate for Rust. Each correctness property listed above will have a corresponding property-based test that:
- Generates random valid inputs (files, metadata, configurations)
- Executes the operation
- Verifies the property holds across all generated inputs
- Runs a minimum of 100 iterations per property

Property tests will use smart generators that:
- Generate realistic file paths, sizes, and metadata
- Create valid database states
- Produce edge cases (empty strings, large numbers, special characters)
- Respect filesystem constraints (path length, valid characters)

### Integration Testing

Integration tests will verify:
- End-to-end import workflow (staging → probing → commit)
- Rescan and integrity check operations
- Cleanup and trash management
- Watch folder monitoring and automatic import
- Collection management and smart collection updates
- Export and import round-trip

### Performance Testing

Performance tests will validate:
- Library browsing with 10,000+ MediaItems
- Rescan performance on large directory trees
- Hashing throughput on HDD storage
- Query performance with complex filters
- Batch operation scalability

### Linux-Specific Testing

Linux integration tests will verify:
- inotify watch folder monitoring
- Symbolic and hard link handling
- POSIX path operations
- Mount point detection
- Desktop notification integration

All tests will be automated and run in CI/CD pipelines to ensure correctness across changes.
