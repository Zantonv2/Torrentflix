# Library Management System - Developer Guide: API Interfaces

## Overview

This guide documents the public API interfaces for the Library Management System components. These interfaces define the contract between components and are used by the Tauri command layer to expose functionality to the UI.

## Tauri Commands

Tauri commands are the bridge between the Svelte UI and the Rust engine. All commands are defined in `src-tauri/src/commands.rs` and are async functions that return JSON-serializable results.

### Command Naming Convention

Commands follow the pattern: `{component}_{action}` (e.g., `library_query`, `import_stage_file`)

### Error Handling

All commands return `Result<T, String>` where errors are serialized as JSON error messages:

```rust
#[tauri::command]
pub async fn library_query(
    state: tauri::State<'_, AppState>,
    filters: LibraryFilters,
) -> Result<Vec<MediaItemDTO>, String> {
    state.library_manager
        .query_library(filters)
        .await
        .map_err(|e| e.to_string())
}
```

---

## Library Management Commands

### Query and Search

#### `library_query`
Query the library with filters and pagination.

**Request**:
```json
{
  "title_filter": "Marvel",
  "year_min": 2010,
  "year_max": 2023,
  "quality_labels": ["1080p", "4K"],
  "status": "present",
  "tags": ["favorite"],
  "min_rating": 7.0,
  "sort_by": "title",
  "sort_order": "asc",
  "limit": 50,
  "offset": 0
}
```

**Response**:
```json
[
  {
    "id": 1,
    "title": "The Avengers",
    "year": 2012,
    "media_type": "movie",
    "user_rating": 8.5,
    "watch_status": "watched",
    "version_count": 2,
    "total_size": 5368709120,
    "poster_path": "/path/to/poster.jpg",
    "genres": ["Action", "Adventure"],
    "overview": "Earth's mightiest heroes..."
  }
]
```

**Errors**:
- `DatabaseError`: Query execution failed
- `ValidationError`: Invalid filter values

---

#### `library_search`
Full-text search across title, cast, director, and notes.

**Request**:
```json
{
  "query": "Avengers",
  "search_fields": ["title", "cast", "director"],
  "fuzzy": true,
  "limit": 50
}
```

**Response**:
```json
[
  {
    "id": 1,
    "title": "The Avengers",
    "match_score": 0.95,
    "matched_fields": ["title"]
  }
]
```

---

#### `library_get_media_item`
Get full details for a specific MediaItem.

**Request**:
```json
{
  "id": 1
}
```

**Response**:
```json
{
  "id": 1,
  "title": "The Avengers",
  "normalized_title": "the avengers",
  "original_title": null,
  "year": 2012,
  "media_type": "movie",
  "overview": "Earth's mightiest heroes...",
  "genres": ["Action", "Adventure"],
  "cast": [
    {"name": "Robert Downey Jr.", "character": "Tony Stark"},
    {"name": "Chris Evans", "character": "Steve Rogers"}
  ],
  "director": "Joss Whedon",
  "runtime": 2400,
  "user_rating": 8.5,
  "watch_status": "watched",
  "last_watched_at": "2024-01-15T10:30:00Z",
  "custom_notes": "Great action movie",
  "tags": ["favorite", "mcu"],
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-15T10:30:00Z"
}
```

---

#### `library_get_file_versions`
Get all FileVersions for a MediaItem.

**Request**:
```json
{
  "media_item_id": 1
}
```

**Response**:
```json
[
  {
    "id": 1,
    "media_item_id": 1,
    "library_root_id": 1,
    "relative_path": "movies/The Avengers (2012).mkv",
    "absolute_path": "/media/movies/The Avengers (2012).mkv",
    "file_size": 5368709120,
    "status": "present",
    "is_preferred": true,
    "container": "mkv",
    "resolution_width": 1920,
    "resolution_height": 1080,
    "video_codec": "h264",
    "audio_codec": "aac",
    "audio_tracks": [
      {"language": "en", "codec": "aac", "channels": 6}
    ],
    "subtitle_tracks": [
      {"language": "en", "format": "srt"},
      {"language": "es", "format": "srt"}
    ],
    "duration": 2400,
    "bitrate": 5000000,
    "framerate": 23.976,
    "quality_label": "1080p BluRay x264",
    "release_group": "GROUP",
    "source_type": "BluRay",
    "hashing_status": "complete",
    "is_symlink": false,
    "is_hardlink": false,
    "added_at": "2024-01-01T00:00:00Z",
    "last_seen_at": "2024-01-15T10:30:00Z"
  }
]
```

---

### User Metadata

#### `library_set_user_rating`
Set user rating for a MediaItem.

**Request**:
```json
{
  "media_item_id": 1,
  "rating": 8.5
}
```

**Response**:
```json
{
  "success": true,
  "media_item_id": 1,
  "rating": 8.5
}
```

**Validation**:
- Rating must be between 0.0 and 10.0

---

#### `library_add_tags`
Add tags to a MediaItem.

**Request**:
```json
{
  "media_item_id": 1,
  "tags": ["favorite", "mcu"]
}
```

**Response**:
```json
{
  "success": true,
  "media_item_id": 1,
  "tags": ["favorite", "mcu"]
}
```

---

#### `library_remove_tags`
Remove tags from a MediaItem.

**Request**:
```json
{
  "media_item_id": 1,
  "tags": ["favorite"]
}
```

**Response**:
```json
{
  "success": true,
  "media_item_id": 1,
  "remaining_tags": ["mcu"]
}
```

---

#### `library_set_watch_status`
Update watch status for a MediaItem.

**Request**:
```json
{
  "media_item_id": 1,
  "status": "watched"
}
```

**Response**:
```json
{
  "success": true,
  "media_item_id": 1,
  "status": "watched",
  "last_watched_at": "2024-01-15T10:30:00Z"
}
```

**Valid statuses**: `unwatched`, `in_progress`, `watched`

---

#### `library_set_custom_notes`
Set custom notes for a MediaItem.

**Request**:
```json
{
  "media_item_id": 1,
  "notes": "Great action movie with excellent cast"
}
```

**Response**:
```json
{
  "success": true,
  "media_item_id": 1,
  "notes": "Great action movie with excellent cast"
}
```

---

## Import Commands

### File Staging

#### `import_stage_file`
Stage a file for import.

**Request**:
```json
{
  "source_path": "/home/user/Downloads/movie.mkv"
}
```

**Response**:
```json
{
  "staged_file_id": 1,
  "original_path": "/home/user/Downloads/movie.mkv",
  "staged_path": "/media/staging/movie_abc123.mkv",
  "file_size": 5368709120,
  "status": "pending"
}
```

---

#### `import_get_staged_files`
Get list of staged files.

**Request**:
```json
{
  "status_filter": "ready"
}
```

**Response**:
```json
[
  {
    "staged_file_id": 1,
    "original_path": "/home/user/Downloads/movie.mkv",
    "file_size": 5368709120,
    "status": "ready",
    "technical_metadata": {
      "container": "mkv",
      "resolution": "1920x1080",
      "video_codec": "h264",
      "duration": 2400
    },
    "created_at": "2024-01-15T10:00:00Z"
  }
]
```

---

#### `import_commit_staged_file`
Commit a staged file to the library.

**Request**:
```json
{
  "staged_file_id": 1,
  "library_root_id": 1,
  "media_metadata": {
    "title": "The Avengers",
    "year": 2012
  }
}
```

**Response**:
```json
{
  "success": true,
  "media_item_id": 1,
  "file_version_id": 1,
  "absolute_path": "/media/movies/The Avengers (2012).mkv"
}
```

---

#### `import_rollback_staged_file`
Rollback a failed import.

**Request**:
```json
{
  "staged_file_id": 1
}
```

**Response**:
```json
{
  "success": true,
  "staged_file_id": 1,
  "status": "failed"
}
```

---

## Maintenance Commands

### Rescan and Integrity

#### `maintenance_schedule_rescan`
Schedule a library rescan job.

**Request**:
```json
{
  "library_root_id": null
}
```

**Response**:
```json
{
  "job_id": 1,
  "status": "queued",
  "operation": "rescan"
}
```

---

#### `maintenance_get_cleanup_candidates`
Get list of cleanup candidates.

**Request**:
```json
{
  "include_duplicates": true,
  "include_trashed": true,
  "include_missing": true,
  "include_low_quality": true
}
```

**Response**:
```json
[
  {
    "file_version_id": 1,
    "path": "/media/movies/old_version.mkv",
    "file_size": 2684354560,
    "reason": "duplicate",
    "quality_score": 0.6,
    "space_savings": 2684354560
  }
]
```

---

#### `maintenance_execute_cleanup`
Execute cleanup on selected candidates.

**Request**:
```json
{
  "file_version_ids": [1, 2, 3]
}
```

**Response**:
```json
{
  "success": true,
  "deleted_count": 3,
  "freed_space": 8053063680,
  "errors": []
}
```

---

#### `maintenance_get_storage_analytics`
Get storage statistics and breakdown.

**Request**:
```json
{}
```

**Response**:
```json
{
  "total_size": 536870912000,
  "media_item_count": 1000,
  "file_version_count": 1500,
  "average_versions_per_media": 1.5,
  "breakdown_by_resolution": {
    "4K": 107374182400,
    "1080p": 322122547200,
    "720p": 107374182400
  },
  "breakdown_by_status": {
    "present": 483183820800,
    "trashed": 53687091200,
    "missing": 0
  },
  "duplicate_space": 26843545600,
  "largest_media_items": [
    {
      "media_item_id": 1,
      "title": "4K Movie",
      "total_size": 107374182400
    }
  ]
}
```

---

## Collection Commands

### Collection Management

#### `collection_create`
Create a new collection.

**Request**:
```json
{
  "name": "Favorites",
  "description": "My favorite movies",
  "is_smart": false
}
```

**Response**:
```json
{
  "collection_id": 1,
  "name": "Favorites",
  "description": "My favorite movies",
  "is_smart": false,
  "created_at": "2024-01-15T10:30:00Z"
}
```

---

#### `collection_add_items`
Add MediaItems to a collection.

**Request**:
```json
{
  "collection_id": 1,
  "media_item_ids": [1, 2, 3]
}
```

**Response**:
```json
{
  "success": true,
  "collection_id": 1,
  "added_count": 3
}
```

---

#### `collection_get_items`
Get items in a collection.

**Request**:
```json
{
  "collection_id": 1,
  "limit": 50,
  "offset": 0
}
```

**Response**:
```json
[
  {
    "id": 1,
    "title": "The Avengers",
    "year": 2012,
    "added_at": "2024-01-15T10:30:00Z"
  }
]
```

---

#### `collection_create_smart`
Create a smart collection with filter criteria.

**Request**:
```json
{
  "name": "Recent 4K Movies",
  "description": "4K movies added in the last month",
  "filter_criteria": {
    "quality_labels": ["4K"],
    "year_range": [2020, 2024],
    "min_rating": 7.0
  }
}
```

**Response**:
```json
{
  "collection_id": 2,
  "name": "Recent 4K Movies",
  "is_smart": true,
  "filter_criteria": {
    "quality_labels": ["4K"],
    "year_range": [2020, 2024],
    "min_rating": 7.0
  },
  "item_count": 42
}
```

---

## Watch Folder Commands

### Watch Folder Management

#### `watch_add_folder`
Add a directory to watch for automatic import.

**Request**:
```json
{
  "path": "/home/user/Downloads",
  "recursive": true,
  "import_profile_id": null
}
```

**Response**:
```json
{
  "watch_folder_id": 1,
  "path": "/home/user/Downloads",
  "recursive": true,
  "is_active": true,
  "created_at": "2024-01-15T10:30:00Z"
}
```

---

#### `watch_remove_folder`
Stop watching a directory.

**Request**:
```json
{
  "watch_folder_id": 1
}
```

**Response**:
```json
{
  "success": true,
  "watch_folder_id": 1
}
```

---

## Job Management Commands

### Job Monitoring

#### `jobs_get_status`
Get status of a background job.

**Request**:
```json
{
  "job_id": 1
}
```

**Response**:
```json
{
  "job_id": 1,
  "operation": "rescan",
  "status": "running",
  "progress_percent": 45,
  "started_at": "2024-01-15T10:00:00Z",
  "estimated_completion": "2024-01-15T10:30:00Z"
}
```

---

#### `jobs_cancel`
Cancel a running job.

**Request**:
```json
{
  "job_id": 1
}
```

**Response**:
```json
{
  "success": true,
  "job_id": 1,
  "status": "cancelled"
}
```

---

## Data Transfer Objects (DTOs)

DTOs are used for serialization between Rust and JSON. They are defined in `engine/src/ui/dto.rs`.

### MediaItemDTO
```rust
pub struct MediaItemDTO {
    pub id: i64,
    pub title: String,
    pub normalized_title: String,
    pub original_title: Option<String>,
    pub year: Option<i32>,
    pub media_type: String,
    pub overview: Option<String>,
    pub genres: Vec<String>,
    pub cast: Vec<CastMemberDTO>,
    pub director: Option<String>,
    pub runtime: Option<i32>,
    pub user_rating: Option<f32>,
    pub watch_status: String,
    pub last_watched_at: Option<String>,
    pub custom_notes: Option<String>,
    pub tags: Vec<String>,
    pub version_count: i32,
    pub total_size: i64,
    pub poster_path: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
```

### FileVersionDTO
```rust
pub struct FileVersionDTO {
    pub id: i64,
    pub media_item_id: i64,
    pub library_root_id: i64,
    pub relative_path: String,
    pub absolute_path: String,
    pub file_size: i64,
    pub status: String,
    pub is_preferred: bool,
    pub container: Option<String>,
    pub resolution_width: Option<i32>,
    pub resolution_height: Option<i32>,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub audio_tracks: Vec<AudioTrackDTO>,
    pub subtitle_tracks: Vec<SubtitleTrackDTO>,
    pub duration: Option<i32>,
    pub bitrate: Option<i32>,
    pub framerate: Option<f32>,
    pub quality_label: Option<String>,
    pub release_group: Option<String>,
    pub source_type: Option<String>,
    pub hashing_status: String,
    pub is_symlink: bool,
    pub is_hardlink: bool,
    pub added_at: String,
    pub last_seen_at: String,
}
```

---

## Error Responses

All error responses follow this format:

```json
{
  "error": "Error message describing what went wrong"
}
```

### Common Error Types

- `ValidationError`: Invalid input parameters
- `DatabaseError`: Database operation failed
- `FileSystemError`: File operation failed
- `NotFoundError`: Resource not found
- `ConflictError`: Operation conflicts with existing state
- `InternalError`: Unexpected server error

---

## Pagination

Commands that return lists support pagination:

```json
{
  "limit": 50,
  "offset": 0
}
```

- `limit`: Maximum number of items to return (default: 50, max: 1000)
- `offset`: Number of items to skip (default: 0)

---

## Filtering

Most query commands support filtering:

```json
{
  "title_filter": "search term",
  "year_min": 2010,
  "year_max": 2023,
  "quality_labels": ["1080p", "4K"],
  "status": "present",
  "tags": ["favorite"],
  "min_rating": 7.0
}
```

All filter fields are optional. When multiple filters are specified, they are combined with AND logic.

---

## Sorting

Query commands support sorting:

```json
{
  "sort_by": "title",
  "sort_order": "asc"
}
```

**Valid sort fields**: `title`, `year`, `added_date`, `size`, `version_count`, `rating`

**Valid sort orders**: `asc`, `desc`

