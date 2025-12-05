# Library Management System - Developer Guide: Database Schema

## Overview

The Library Management System uses SQLite with WAL (Write-Ahead Logging) mode for concurrent access. This guide documents the complete database schema, relationships, and design decisions.

## Database Configuration

```sql
-- WAL mode for concurrent access
PRAGMA journal_mode = WAL;

-- Foreign key constraints enabled
PRAGMA foreign_keys = ON;

-- Synchronous mode for durability
PRAGMA synchronous = NORMAL;
```

## Core Tables

### media_items

Represents logical media entities (movies or TV series).

```sql
CREATE TABLE media_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    normalized_title TEXT NOT NULL,
    original_title TEXT,
    year INTEGER,
    media_type TEXT NOT NULL CHECK(media_type IN ('movie', 'series')),
    overview TEXT,
    genres TEXT, -- JSON array: ["Action", "Drama"]
    cast TEXT, -- JSON array: [{"name": "Actor", "character": "Role"}]
    director TEXT,
    runtime INTEGER, -- seconds
    poster_url TEXT,
    poster_path TEXT, -- local cached poster path
    backdrop_url TEXT,
    backdrop_path TEXT, -- local cached backdrop path
    external_ids TEXT, -- JSON: {"tmdb_id": 123, "imdb_id": "tt123"}
    user_rating REAL, -- 0.0-10.0
    watch_status TEXT CHECK(watch_status IN ('unwatched', 'in_progress', 'watched')),
    last_watched_at DATETIME,
    custom_notes TEXT,
    metadata_source TEXT, -- 'tmdb', 'kinopoisk', 'manual'
    metadata_fetched_at DATETIME,
    user_edited_fields TEXT, -- JSON array: ["title", "year"]
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_media_items_normalized_title ON media_items(normalized_title);
CREATE INDEX idx_media_items_year ON media_items(year);
CREATE INDEX idx_media_items_watch_status ON media_items(watch_status);
CREATE INDEX idx_media_items_user_rating ON media_items(user_rating);
```

**Key Fields**:
- `normalized_title`: Lowercase, trimmed, special characters handled - used for matching and search
- `genres`, `cast`, `external_ids`: Stored as JSON for flexibility
- `user_edited_fields`: Tracks which fields were manually edited to prevent auto-overwrite
- `watch_status`: Tracks viewing progress (unwatched, in_progress, watched)

**Indexes**:
- `normalized_title`: Fast text search
- `year`: Filtering by release year
- `watch_status`: Filtering by viewing status
- `user_rating`: Sorting by user rating

---

### file_versions

Represents physical files on disk associated with MediaItems.

```sql
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
    container TEXT, -- 'mkv', 'mp4', 'avi'
    resolution_width INTEGER,
    resolution_height INTEGER,
    video_codec TEXT, -- 'h264', 'h265', 'vp9'
    audio_codec TEXT, -- 'aac', 'ac3', 'flac'
    audio_tracks TEXT, -- JSON array: [{"language": "en", "codec": "aac"}]
    subtitle_tracks TEXT, -- JSON array: [{"language": "en", "format": "srt"}]
    duration INTEGER, -- seconds
    bitrate INTEGER, -- bits per second
    framerate REAL, -- frames per second
    
    -- Quality and classification
    quality_label TEXT, -- '1080p BluRay x264', '720p WEB-DL x265'
    release_group TEXT, -- 'GROUP-NAME'
    source_type TEXT, -- 'BluRay', 'WEB-DL', 'HDTV', 'DVD'
    
    -- Fingerprints and hashing
    fast_hash BLOB, -- 32 bytes: first 1MB + last 1MB + size
    full_hash BLOB, -- variable: SHA-256 or BLAKE3
    hash_algorithm TEXT, -- 'sha256', 'blake3'
    hashing_status TEXT CHECK(hashing_status IN ('pending', 'in_progress', 'complete', 'failed')),
    hashed_at DATETIME,
    
    -- Integrity
    checksum BLOB, -- SHA-256 or BLAKE3 for verification
    checksum_algorithm TEXT,
    last_verified_at DATETIME,
    corruption_detected_at DATETIME,
    
    -- Filesystem metadata
    is_symlink BOOLEAN NOT NULL DEFAULT 0,
    symlink_target TEXT, -- resolved target path
    is_hardlink BOOLEAN NOT NULL DEFAULT 0,
    inode INTEGER, -- Linux inode number
    
    -- Timestamps
    added_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_seen_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    trashed_at DATETIME,
    original_path TEXT, -- path before moving to trash
    
    -- Import metadata
    import_source TEXT, -- 'watch_folder', 'manual', 'rescan'
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
```

**Key Fields**:
- `status`: Tracks file state (present, missing, trashed, corrupted)
- `is_preferred`: User-selected preferred version for playback
- `fast_hash`: Quick duplicate detection (first 1MB + last 1MB + size)
- `full_hash`: Exact duplicate detection (full file hash)
- `hashing_status`: Background hashing job status
- `inode`: Linux inode for hardlink detection
- `symlink_target`: Resolved target for symlinks

**Indexes**:
- `media_item_id`: Fast lookup of versions for a media item
- `status`: Filtering by file status
- `fast_hash`, `full_hash`: Duplicate detection
- `quality_label`: Filtering by quality
- `resolution`: Filtering by resolution
- `inode`: Hardlink detection

---

### library_roots

Designates storage locations for media files.

```sql
CREATE TABLE library_roots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT 1,
    is_archive BOOLEAN NOT NULL DEFAULT 0, -- cold storage flag
    filesystem_type TEXT, -- 'ext4', 'btrfs', 'xfs'
    mount_point TEXT, -- mount point for cross-mount detection
    total_space INTEGER, -- bytes
    free_space INTEGER, -- bytes
    last_scanned_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

**Key Fields**:
- `is_archive`: Marks cold storage roots with lower priority for background jobs
- `filesystem_type`: Enables filesystem-specific optimizations (reflinks on btrfs/xfs)
- `mount_point`: Detects cross-mount operations for atomic move fallback

---

### tags

User-defined tags for organizing media.

```sql
CREATE TABLE tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    color TEXT, -- hex color: '#FF0000'
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

---

### media_item_tags

Junction table for many-to-many relationship between MediaItems and Tags.

```sql
CREATE TABLE media_item_tags (
    media_item_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (media_item_id, tag_id),
    FOREIGN KEY (media_item_id) REFERENCES media_items(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

CREATE INDEX idx_media_item_tags_tag ON media_item_tags(tag_id);
```

---

### collections

User-defined groupings of MediaItems (manual or smart).

```sql
CREATE TABLE collections (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT,
    is_smart BOOLEAN NOT NULL DEFAULT 0,
    filter_criteria TEXT, -- JSON for smart collections
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

**Key Fields**:
- `is_smart`: Distinguishes manual collections from smart collections
- `filter_criteria`: JSON object with query criteria for smart collections

**Example filter_criteria**:
```json
{
  "title_pattern": "Marvel",
  "year_range": [2010, 2023],
  "quality_labels": ["1080p", "4K"],
  "tags": ["favorite"],
  "min_rating": 7.0,
  "watch_status": "unwatched"
}
```

---

### collection_items

Junction table for many-to-many relationship between Collections and MediaItems.

```sql
CREATE TABLE collection_items (
    collection_id INTEGER NOT NULL,
    media_item_id INTEGER NOT NULL,
    added_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (collection_id, media_item_id),
    FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE CASCADE,
    FOREIGN KEY (media_item_id) REFERENCES media_items(id) ON DELETE CASCADE
);

CREATE INDEX idx_collection_items_media ON collection_items(media_item_id);
```

---

### watch_folders

Directories monitored for automatic file import.

```sql
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
```

**Key Fields**:
- `recursive`: Whether to monitor subdirectories
- `import_profile_id`: Optional profile for automatic rule application

---

### import_profiles

Automation rules for batch importing files.

```sql
CREATE TABLE import_profiles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT,
    priority INTEGER NOT NULL DEFAULT 0,
    file_pattern TEXT, -- glob or regex pattern
    min_resolution_width INTEGER,
    min_resolution_height INTEGER,
    allowed_codecs TEXT, -- JSON array: ["h264", "h265"]
    auto_tags TEXT, -- JSON array: ["tag1", "tag2"]
    target_collection_id INTEGER,
    target_library_root_id INTEGER,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (target_collection_id) REFERENCES collections(id),
    FOREIGN KEY (target_library_root_id) REFERENCES library_roots(id)
);
```

**Key Fields**:
- `priority`: Higher priority profiles are matched first
- `file_pattern`: Glob or regex for matching filenames
- `auto_tags`: Tags automatically applied to imported files
- `target_collection_id`: Collection to add imported files to

---

### playback_history

Tracks viewing history and resume positions.

```sql
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
```

---

### audit_log

Complete record of all file operations for traceability.

```sql
CREATE TABLE audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    operation_type TEXT NOT NULL, -- 'create', 'move', 'delete', 'update'
    entity_type TEXT NOT NULL, -- 'media_item', 'file_version', 'collection'
    entity_id INTEGER,
    old_value TEXT, -- JSON representation
    new_value TEXT, -- JSON representation
    initiator TEXT NOT NULL, -- 'user' or 'system:job_name'
    error_message TEXT,
    timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_audit_log_timestamp ON audit_log(timestamp);
CREATE INDEX idx_audit_log_entity ON audit_log(entity_type, entity_id);
CREATE INDEX idx_audit_log_operation ON audit_log(operation_type);
```

**Key Fields**:
- `operation_type`: Type of operation performed
- `old_value`, `new_value`: JSON snapshots for change tracking
- `initiator`: Distinguishes user actions from system jobs
- `error_message`: Captured if operation failed

---

### notifications

User notifications and alerts.

```sql
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
```

---

### staged_files

Temporary storage for files being imported.

```sql
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

**Key Fields**:
- `status`: Tracks import progress (pending → probing → ready → committing)
- `technical_metadata`: Cached metadata from probing
- `fast_hash`: Computed during staging for quick duplicate detection

---

## Relationships

### One-to-Many
- `media_items` → `file_versions` (one MediaItem has many FileVersions)
- `library_roots` → `file_versions` (one root has many versions)
- `collections` → `collection_items` (one collection has many items)
- `file_versions` → `playback_history` (one version has many playback records)

### Many-to-Many
- `media_items` ↔ `tags` (via `media_item_tags`)
- `collections` ↔ `media_items` (via `collection_items`)

### Cascading Deletes
- Deleting a `media_item` cascades to `file_versions` and `media_item_tags`
- Deleting a `collection` cascades to `collection_items`
- Deleting a `file_version` cascades to `playback_history`

---

## Query Patterns

### Find all versions of a media item
```sql
SELECT * FROM file_versions
WHERE media_item_id = ? AND status != 'trashed'
ORDER BY is_preferred DESC, added_at DESC;
```

### Find duplicates by hash
```sql
SELECT full_hash, COUNT(*) as count, GROUP_CONCAT(id) as version_ids
FROM file_versions
WHERE full_hash IS NOT NULL AND status = 'present'
GROUP BY full_hash
HAVING count > 1;
```

### Find cleanup candidates
```sql
SELECT * FROM file_versions
WHERE status IN ('trashed', 'missing', 'corrupted')
   OR (status = 'present' AND trashed_at < datetime('now', '-30 days'))
ORDER BY file_size DESC;
```

### Search media by title
```sql
SELECT * FROM media_items
WHERE normalized_title LIKE ? OR title LIKE ?
ORDER BY year DESC, title ASC;
```

### Get collection items with filters
```sql
SELECT m.* FROM media_items m
JOIN collection_items ci ON m.id = ci.media_item_id
WHERE ci.collection_id = ?
  AND (? IS NULL OR m.year >= ?)
  AND (? IS NULL OR m.year <= ?)
ORDER BY m.title ASC;
```

### Get smart collection items
```sql
SELECT m.* FROM media_items m
WHERE (? IS NULL OR m.normalized_title LIKE ?)
  AND (? IS NULL OR m.year >= ?)
  AND (? IS NULL OR m.year <= ?)
  AND (? IS NULL OR m.user_rating >= ?)
ORDER BY m.title ASC;
```

---

## Performance Considerations

### Indexes
- Indexes on frequently filtered columns (title, year, status, hash)
- Composite indexes for common filter combinations
- Indexes on foreign keys for join performance

### Query Optimization
- Use `LIMIT` for pagination to avoid loading entire result sets
- Use `COUNT(*)` with `GROUP BY` for aggregations
- Lazy load large fields (posters, metadata) only when needed

### Concurrency
- WAL mode allows concurrent reads and writes
- Transactions for multi-step operations ensure atomicity
- Foreign key constraints prevent orphaned records

### Storage
- BLOB fields for hashes and checksums (efficient binary storage)
- JSON fields for flexible metadata (genres, cast, external_ids)
- TEXT fields for searchable content (title, notes)

---

## Maintenance

### VACUUM
Reclaims space from deleted records:
```sql
VACUUM;
```

### ANALYZE
Updates query optimizer statistics:
```sql
ANALYZE;
```

### Integrity Check
Verifies database consistency:
```sql
PRAGMA integrity_check;
```

### Backup
SQLite database files can be backed up by copying:
- `.data/torrentflix.db` (main database)
- `.data/torrentflix.db-wal` (write-ahead log)
- `.data/torrentflix.db-shm` (shared memory)

---

## Migration Strategy

Schema changes are handled through migrations in `engine/src/config/migration.rs`:

1. New tables are created if they don't exist
2. Columns are added with `ALTER TABLE` if missing
3. Indexes are created if missing
4. Data is migrated with `UPDATE` statements if needed

This allows the application to evolve the schema without manual intervention.

