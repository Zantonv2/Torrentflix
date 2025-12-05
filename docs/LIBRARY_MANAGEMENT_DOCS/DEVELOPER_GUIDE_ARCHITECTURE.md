# Library Management System - Developer Guide: Component Architecture

## Overview

The Library Management System is built on a modular, layered architecture with clear separation of concerns. This guide documents the component structure, responsibilities, and interactions.

## Architecture Layers

```
┌─────────────────────────────────────────────────────────────────┐
│                         UI Layer (Svelte)                        │
│  LibraryBrowser │ VersionManager │ CollectionManager │ etc.     │
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

## Core Components

### 1. LibraryManager (`engine/src/library/manager.rs`)

**Purpose**: Core component for managing MediaItems, FileVersions, and library queries.

**Key Responsibilities**:
- MediaItem CRUD operations (create, read, update, delete)
- FileVersion management and association
- Library querying with filters and sorting
- User metadata management (ratings, tags, notes, watch status)
- Collection management (manual and smart collections)
- Search functionality (text search, fuzzy matching)

**Public Interface**:
```rust
pub struct LibraryManager {
    db: Arc<DatabaseConnection>,
    config: LibraryConfig,
}

// MediaItem operations
pub async fn create_media_item(&self, metadata: MediaMetadata) -> Result<MediaItemId>;
pub async fn get_media_item(&self, id: MediaItemId) -> Result<MediaItem>;
pub async fn update_media_item(&self, id: MediaItemId, updates: MediaItemUpdate) -> Result<()>;
pub async fn delete_media_item(&self, id: MediaItemId) -> Result<()>;

// FileVersion operations
pub async fn add_file_version(&self, media_id: MediaItemId, version: FileVersionData) -> Result<FileVersionId>;
pub async fn get_file_versions(&self, media_id: MediaItemId) -> Result<Vec<FileVersion>>;
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
```

**Dependencies**:
- `DatabaseConnection` for persistence
- `MetadataProvider` for enrichment
- `QualityScorer` for version ranking

**Key Methods**:
- `create_media_item`: Normalizes title, creates MediaItem record
- `query_library`: Builds dynamic SQL with filters, applies pagination and sorting
- `search_library`: Case-insensitive search with fuzzy matching support
- `get_duplicates`: Groups FileVersions by hash or MediaItem

---

### 2. ImportPipeline (`engine/src/import/pipeline.rs`)

**Purpose**: Handles file import workflow from staging through commit.

**Key Responsibilities**:
- File staging and validation
- Metadata probing (technical and descriptive)
- Fingerprint computation (fast and full hash)
- Atomic commit to library roots
- Watch folder monitoring (inotify)
- Import profile matching and rule application

**Public Interface**:
```rust
pub struct ImportPipeline {
    staging_dir: PathBuf,
    library_roots: Vec<LibraryRoot>,
    db: Arc<DatabaseConnection>,
    metadata_prober: MetadataProber,
    hasher: FileHasher,
}

// Staging operations
pub async fn stage_file(&self, source: PathBuf) -> Result<StagedFile>;
pub async fn probe_staged_file(&self, staged_id: StagedFileId) -> Result<TechnicalMetadata>;
pub async fn compute_fast_fingerprint(&self, staged_id: StagedFileId) -> Result<Fingerprint>;

// Commit operations
pub async fn commit_staged_file(&self, staged_id: StagedFileId, target_root: LibraryRootId) -> Result<FileVersionId>;
pub async fn rollback_staged_file(&self, staged_id: StagedFileId) -> Result<()>;

// Watch folder operations
pub async fn add_watch_folder(&self, path: PathBuf, profile: Option<ImportProfile>) -> Result<WatchFolderId>;
pub async fn process_watch_event(&self, event: InotifyEvent) -> Result<()>;

// Import profiles
pub async fn create_import_profile(&self, profile: ImportProfile) -> Result<ImportProfileId>;
pub async fn match_import_profile(&self, file_path: &Path) -> Result<Option<ImportProfile>>;
pub async fn apply_import_rules(&self, file: &StagedFile, profile: &ImportProfile) -> Result<ImportActions>;
```

**Dependencies**:
- `LibraryManager` for MediaItem/FileVersion creation
- `MetadataProber` (ffprobe wrapper) for technical metadata
- `FileHasher` for fingerprint computation
- `InotifyWatcher` for watch folder monitoring

**Key Methods**:
- `stage_file`: Copies/moves file to staging, creates StagedFile record
- `commit_staged_file`: Atomic transaction - creates MediaItem/FileVersion, moves file to library root
- `probe_staged_file`: Calls ffprobe, extracts and stores technical metadata
- `compute_fast_fingerprint`: Computes hash from first 1MB + last 1MB + file size

---

### 3. MaintenanceManager (`engine/src/maintenance/manager.rs`)

**Purpose**: Handles library maintenance, integrity, and cleanup operations.

**Key Responsibilities**:
- Rescan and integrity checking
- Duplicate detection and cleanup suggestions
- Soft-delete and trash management
- Background hashing jobs
- Database optimization (VACUUM, index rebuild)
- Audit log management

**Public Interface**:
```rust
pub struct MaintenanceManager {
    db: Arc<DatabaseConnection>,
    library_roots: Vec<LibraryRoot>,
    job_manager: Arc<JobManager>,
}

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
```

**Dependencies**:
- `JobManager` for background task scheduling
- `FileHasher` for integrity verification
- `AuditLogger` for operation tracking

**Key Methods**:
- `perform_rescan`: Walks library roots, detects new/missing files, updates database
- `identify_cleanup_candidates`: Groups duplicates, identifies low-quality variants, trashed files
- `execute_cleanup`: Permanently deletes files, removes records, logs errors
- `soft_delete_version`: Moves file to trash, updates status, creates audit log

---

### 4. MetadataProber (`engine/src/import/metadata_prober.rs`)

**Purpose**: Extracts technical metadata from media files using ffprobe.

**Key Responsibilities**:
- Execute ffprobe on media files
- Parse JSON output
- Extract video, audio, subtitle information
- Handle errors gracefully

**Public Interface**:
```rust
pub struct MetadataProber {
    ffprobe_path: PathBuf,
}

pub async fn probe_file(&self, path: &Path) -> Result<TechnicalMetadata>;
pub async fn extract_video_info(&self, path: &Path) -> Result<VideoInfo>;
pub async fn extract_audio_info(&self, path: &Path) -> Result<Vec<AudioTrack>>;
pub async fn extract_subtitle_info(&self, path: &Path) -> Result<Vec<SubtitleTrack>>;
pub async fn get_duration(&self, path: &Path) -> Result<Duration>;
```

**Dependencies**:
- ffprobe binary (external)
- serde_json for parsing

**Key Methods**:
- `probe_file`: Executes ffprobe, parses output, returns complete TechnicalMetadata
- `extract_video_info`: Extracts codec, resolution, framerate, bitrate
- `extract_audio_info`: Extracts audio tracks with language and codec info

---

### 5. FileHasher (`engine/src/filesystem/hasher.rs`)

**Purpose**: Computes file fingerprints for duplicate detection and integrity verification.

**Key Responsibilities**:
- Compute fast hashes (first 1MB + last 1MB + size)
- Compute full-file hashes (SHA-256, BLAKE3)
- Verify checksums
- HDD-optimized I/O patterns

**Public Interface**:
```rust
pub struct FileHasher {
    fast_hash_size: usize,
    buffer_size: usize,
}

pub async fn compute_fast_hash(&self, path: &Path) -> Result<FastHash>;
pub async fn compute_full_hash(&self, path: &Path, algorithm: HashAlgorithm) -> Result<FullHash>;
pub async fn verify_checksum(&self, path: &Path, expected: &FullHash) -> Result<bool>;
```

**Dependencies**:
- sha2 or blake3 crate for hashing
- tokio::fs for async file I/O

**Key Methods**:
- `compute_fast_hash`: Reads first 1MB, last 1MB, combines with file size
- `compute_full_hash`: Reads entire file in 1MB chunks, computes hash
- `verify_checksum`: Recomputes hash, compares with expected value

---

### 6. QualityScorer (`engine/src/library/quality_scorer.rs`)

**Purpose**: Ranks FileVersions by quality for duplicate resolution and preferred version selection.

**Key Responsibilities**:
- Score versions based on resolution, codec, source, size
- Compare versions
- Rank versions by quality
- Respect user-set preferred flag

**Public Interface**:
```rust
pub struct QualityScorer {
    weights: QualityWeights,
}

pub fn score_version(&self, version: &FileVersion) -> QualityScore;
pub fn compare_versions(&self, a: &FileVersion, b: &FileVersion) -> Ordering;
pub fn rank_versions(&self, versions: &[FileVersion]) -> Vec<(FileVersionId, QualityScore)>;
```

**Dependencies**: None (pure computation)

**Key Methods**:
- `score_version`: Calculates weighted score from resolution, codec, source, size
- `rank_versions`: Sorts versions by score, respects preferred flag

---

### 7. WatchFolderMonitor (`engine/src/watch/monitor.rs`)

**Purpose**: Monitors directories for new files using Linux inotify.

**Key Responsibilities**:
- Register directories with inotify
- Detect file write completion
- Trigger automatic import
- Handle failed imports

**Public Interface**:
```rust
pub struct WatchFolderMonitor {
    inotify: Inotify,
    watch_folders: HashMap<WatchFolderId, WatchFolder>,
    import_pipeline: Arc<ImportPipeline>,
}

pub async fn add_watch(&self, path: PathBuf, profile: Option<ImportProfile>) -> Result<WatchFolderId>;
pub async fn remove_watch(&self, id: WatchFolderId) -> Result<()>;
pub async fn start_monitoring(&self) -> Result<()>;
pub async fn handle_event(&self, event: InotifyEvent) -> Result<()>;
```

**Dependencies**:
- inotify crate for Linux filesystem events
- ImportPipeline for automatic import

**Key Methods**:
- `add_watch`: Registers directory with inotify, stores configuration
- `handle_event`: Detects write completion, triggers import
- `start_monitoring`: Main event loop, processes inotify events

---

### 8. NotificationManager (`engine/src/notifications/manager.rs`)

**Purpose**: Manages user notifications and desktop integration.

**Key Responsibilities**:
- Create and store notifications
- Send to desktop notification system
- Manage notification lifecycle
- Support priority levels

**Public Interface**:
```rust
pub struct NotificationManager {
    backend: NotificationBackend,
    notifications: Arc<Mutex<Vec<Notification>>>,
}

pub async fn send_notification(&self, notification: Notification) -> Result<NotificationId>;
pub async fn get_unread_notifications(&self) -> Result<Vec<Notification>>;
pub async fn mark_as_read(&self, id: NotificationId) -> Result<()>;
pub async fn dismiss_notification(&self, id: NotificationId) -> Result<()>;
pub async fn cleanup_old_notifications(&self, older_than: Duration) -> Result<usize>;
```

**Dependencies**:
- notify-rust or dbus crate for Linux desktop notifications

**Key Methods**:
- `send_notification`: Creates record, sends to desktop system
- `cleanup_old_notifications`: Removes old notifications from database

---

### 9. AuditLogger (`engine/src/audit/logger.rs`)

**Purpose**: Logs all file operations for traceability and debugging.

**Key Responsibilities**:
- Record operation type, entity, old/new values
- Store initiator (user or system job)
- Query logs with filters
- Archive old entries

**Public Interface**:
```rust
pub struct AuditLogger {
    db: Arc<DatabaseConnection>,
}

pub async fn log_operation(&self, operation: AuditOperation) -> Result<()>;
pub async fn query_logs(&self, filters: AuditFilters) -> Result<Vec<AuditLogEntry>>;
pub async fn archive_old_entries(&self, older_than: DateTime<Utc>) -> Result<usize>;
```

**Dependencies**: DatabaseConnection

**Key Methods**:
- `log_operation`: Stores operation details with timestamp
- `query_logs`: Returns entries in reverse chronological order

---

## Component Interactions

### Import Workflow
```
User selects file
    ↓
ImportPipeline::stage_file
    ↓ (creates StagedFile record)
MetadataProber::probe_file
    ↓ (extracts technical metadata)
FileHasher::compute_fast_hash
    ↓ (computes fingerprint)
ImportPipeline::commit_staged_file
    ↓ (atomic transaction)
LibraryManager::create_media_item (if new)
    ↓
LibraryManager::add_file_version
    ↓
AuditLogger::log_operation
    ↓
File moved to library root
```

### Cleanup Workflow
```
User requests cleanup
    ↓
MaintenanceManager::identify_cleanup_candidates
    ↓ (groups duplicates, identifies low-quality)
MaintenanceManager::estimate_space_savings
    ↓ (calculates freed space)
User confirms
    ↓
MaintenanceManager::execute_cleanup
    ↓ (deletes files, removes records)
AuditLogger::log_operation
    ↓
NotificationManager::send_notification
```

### Watch Folder Workflow
```
File appears in watch folder
    ↓
WatchFolderMonitor::handle_event (inotify)
    ↓ (detects write completion)
ImportPipeline::stage_file
    ↓
MetadataProber::probe_file
    ↓
ImportPipeline::match_import_profile
    ↓ (applies rules)
ImportPipeline::commit_staged_file
    ↓
NotificationManager::send_notification
```

---

## Data Flow

### Query Flow
```
UI (Svelte)
    ↓ Tauri command
src-tauri/commands.rs
    ↓ delegates to
LibraryManager::query_library
    ↓ builds SQL query
DatabaseConnection
    ↓ executes query
SQLite
    ↓ returns results
LibraryManager
    ↓ returns Vec<MediaItem>
Tauri
    ↓ serializes to JSON
UI (displays results)
```

### Mutation Flow
```
UI (Svelte)
    ↓ Tauri command
src-tauri/commands.rs
    ↓ delegates to
LibraryManager::add_tags (or other mutation)
    ↓ begins transaction
DatabaseConnection
    ↓ executes INSERT/UPDATE
SQLite
    ↓ commits transaction
AuditLogger::log_operation
    ↓ records change
Tauri
    ↓ returns success
UI (updates display)
```

---

## Module Organization

```
engine/src/
├── library/
│   ├── manager.rs          # LibraryManager (main component)
│   ├── models.rs           # Domain models (MediaItem, FileVersion, etc.)
│   ├── queries.rs          # Database queries
│   ├── database.rs         # Database operations
│   └── mod.rs              # Module exports
├── import/
│   ├── pipeline.rs         # ImportPipeline (main component)
│   ├── metadata_prober.rs  # MetadataProber
│   ├── models.rs           # Import-related models
│   └── mod.rs              # Module exports
├── maintenance/
│   ├── manager.rs          # MaintenanceManager (main component)
│   ├── models.rs           # Maintenance-related models
│   └── mod.rs              # Module exports
├── filesystem/
│   ├── hasher.rs           # FileHasher
│   ├── atomic.rs           # Atomic operations
│   ├── links.rs            # Symlink/hardlink handling
│   ├── posix.rs            # POSIX path handling
│   └── mod.rs              # Module exports
├── watch/
│   ├── monitor.rs          # WatchFolderMonitor
│   ├── models.rs           # Watch folder models
│   └── mod.rs              # Module exports
├── notifications/
│   ├── manager.rs          # NotificationManager
│   ├── models.rs           # Notification models
│   └── mod.rs              # Module exports
├── audit/
│   ├── logger.rs           # AuditLogger
│   ├── models.rs           # Audit log models
│   └── mod.rs              # Module exports
├── database/
│   ├── connection.rs       # Database connection pool
│   ├── library.rs          # Library queries
│   ├── jobs.rs             # Job queries
│   └── mod.rs              # Module exports
├── jobs/
│   ├── manager.rs          # Job scheduling and execution
│   ├── workers.rs          # Background workers
│   └── mod.rs              # Module exports
├── config/
│   ├── loader.rs           # Configuration loading
│   ├── validation.rs       # Configuration validation
│   └── mod.rs              # Module exports
└── engine.rs               # Main Engine orchestrator
```

---

## Error Handling

All components use `anyhow::Result<T>` for error propagation:

```rust
pub async fn some_operation(&self) -> Result<SomeType> {
    // Operations that can fail use ? operator
    let data = self.db.query_something().await?;
    let processed = process_data(data)?;
    Ok(processed)
}
```

Errors are logged with context and propagated to the caller. The Tauri command layer converts errors to JSON responses for the UI.

---

## Testing

Each component has associated property-based tests in `engine/tests/`:

- `library_management_proptest.rs` - LibraryManager tests
- `import_pipeline_proptest.rs` - ImportPipeline tests
- `maintenance_proptest.rs` - MaintenanceManager tests
- etc.

Tests use the `proptest` crate to generate random inputs and verify correctness properties.

---

## Performance Considerations

### HDD Optimization
- Sequential access patterns for directory scanning
- Large read buffers (1MB+) for file hashing
- Batched I/O operations to minimize seeks
- Throttled concurrent operations to avoid disk thrashing

### Database Optimization
- Indexes on frequently queried columns (title, year, status, hash)
- WAL mode for concurrent access
- Transactions for multi-step operations
- Lazy loading of large fields (posters, metadata)

### Async/Concurrency
- All I/O operations are async using Tokio
- Background jobs run in separate tasks
- Shared state protected with Arc<Mutex<T>>
- Channels for producer-consumer patterns

---

## Future Extensions

The component architecture supports easy addition of:
- New metadata providers (implement MetadataProvider trait)
- New rating sources (implement RatingSource trait)
- New indexers (implement Indexer trait)
- New notification backends (implement NotificationBackend trait)
- New filesystem optimizations (extend FileHasher, AtomicOps)

