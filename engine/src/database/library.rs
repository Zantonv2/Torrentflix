use std::sync::Arc;
use std::collections::HashSet;
use std::path::PathBuf;
use anyhow::Result;
use sqlx::{SqlitePool, Row};
use tracing::info;
use chrono::{DateTime, Utc};

use super::connection::Database;
use crate::library::models::*;

/// Database connection for library operations
pub struct LibraryDatabase {
    database: Arc<Database>,
}

impl LibraryDatabase {
    /// Create a new library database connection
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    /// Initialize library schema
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing library schema");
        // TODO: Create library tables (media_items, file_versions, etc.)
        // For now, this is a placeholder
        Ok(())
    }

    /// Get the connection pool
    pub fn pool(&self) -> &SqlitePool {
        self.database.pool()
    }

    // ========================================================================
    // Collection Operations
    // ========================================================================

    /// Create a new collection
    pub async fn create_collection(&self, collection: &Collection) -> Result<CollectionId> {
        let criteria_json = match &collection.collection_type {
            CollectionType::Manual => None,
            CollectionType::Smart { criteria } => Some(serde_json::to_string(criteria)?),
        };

        let is_smart = matches!(collection.collection_type, CollectionType::Smart { .. });

        let result = sqlx::query(
            r#"
            INSERT INTO collections (name, description, is_smart, filter_criteria, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&collection.name)
        .bind(&collection.description)
        .bind(is_smart)
        .bind(criteria_json)
        .bind(collection.created_at)
        .bind(collection.updated_at)
        .execute(self.pool())
        .await?;

        Ok(CollectionId(result.last_insert_rowid()))
    }

    /// Get a collection by ID
    pub async fn get_collection(&self, id: CollectionId) -> Result<Option<Collection>> {
        let row = sqlx::query(
            r#"
            SELECT id, name, description, is_smart, filter_criteria, created_at, updated_at
            FROM collections
            WHERE id = ?
            "#
        )
        .bind(id.0)
        .fetch_optional(self.pool())
        .await?;

        match row {
            Some(row) => {
                let is_smart: bool = row.get("is_smart");
                let collection_type = if is_smart {
                    let criteria_json: Option<String> = row.get("filter_criteria");
                    let criteria = criteria_json
                        .map(|json| serde_json::from_str(&json))
                        .transpose()?
                        .unwrap_or_else(|| FilterCriteria {
                            title_pattern: None,
                            year_range: None,
                            quality_labels: None,
                            tags: None,
                            min_rating: None,
                            watch_status: None,
                        });
                    CollectionType::Smart { criteria }
                } else {
                    CollectionType::Manual
                };

                Ok(Some(Collection {
                    id: CollectionId(row.get("id")),
                    name: row.get("name"),
                    description: row.get("description"),
                    collection_type,
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                }))
            }
            None => Ok(None),
        }
    }

    /// Add MediaItems to a collection
    pub async fn add_to_collection(&self, collection_id: CollectionId, media_ids: &[MediaItemId]) -> Result<()> {
        let mut tx = self.pool().begin().await?;

        for media_id in media_ids {
            // Use INSERT OR IGNORE to avoid errors if the association already exists
            sqlx::query(
                r#"
                INSERT OR IGNORE INTO collection_items (collection_id, media_item_id, added_at)
                VALUES (?, ?, ?)
                "#
            )
            .bind(collection_id.0)
            .bind(media_id.0)
            .bind(Utc::now())
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    /// Get MediaItems in a collection
    pub async fn get_collection_items(&self, collection_id: CollectionId) -> Result<Vec<MediaItem>> {
        let rows = sqlx::query(
            r#"
            SELECT m.*
            FROM media_items m
            INNER JOIN collection_items ci ON m.id = ci.media_item_id
            WHERE ci.collection_id = ?
            ORDER BY ci.added_at DESC
            "#
        )
        .bind(collection_id.0)
        .fetch_all(self.pool())
        .await?;

        let mut items = Vec::new();
        for row in rows {
            items.push(self.row_to_media_item(&row)?);
        }

        Ok(items)
    }

    /// Get all collections
    pub async fn get_all_collections(&self) -> Result<Vec<Collection>> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, description, is_smart, filter_criteria, created_at, updated_at
            FROM collections
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(self.pool())
        .await?;

        let mut collections = Vec::new();
        for row in rows {
            let is_smart: bool = row.get("is_smart");
            let collection_type = if is_smart {
                let criteria_json: Option<String> = row.get("filter_criteria");
                let criteria = criteria_json
                    .map(|json| serde_json::from_str(&json))
                    .transpose()?
                    .unwrap_or_else(|| FilterCriteria {
                        title_pattern: None,
                        year_range: None,
                        quality_labels: None,
                        tags: None,
                        min_rating: None,
                        watch_status: None,
                    });
                CollectionType::Smart { criteria }
            } else {
                CollectionType::Manual
            };

            collections.push(Collection {
                id: CollectionId(row.get("id")),
                name: row.get("name"),
                description: row.get("description"),
                collection_type,
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }

        Ok(collections)
    }

    // ========================================================================
    // MediaItem Operations (Stubs for now)
    // ========================================================================

    pub async fn create_media_item(&self, _media_item: &MediaItem) -> Result<MediaItemId> {
        // Stub implementation
        Ok(MediaItemId(1))
    }

    pub async fn get_media_item(&self, id: MediaItemId) -> Result<Option<MediaItem>> {
        let row = sqlx::query(
            r#"
            SELECT id, title, normalized_title, original_title, year, media_type,
                   overview, genres, cast, director, runtime, poster_url, poster_path,
                   backdrop_url, backdrop_path, external_ids, user_rating, watch_status,
                   last_watched_at, custom_notes, metadata_source, metadata_fetched_at,
                   user_edited_fields, alternative_titles, custom_poster, custom_backdrop,
                   created_at, updated_at
            FROM media_items
            WHERE id = ?
            "#,
        )
        .bind(id.0)
        .fetch_optional(self.pool())
        .await?;

        if let Some(row) = row {
            let genres_json: String = row.try_get("genres").unwrap_or_else(|_| "[]".to_string());
            let genres: Vec<String> = serde_json::from_str(&genres_json).unwrap_or_default();

            let cast_json: String = row.try_get("cast").unwrap_or_else(|_| "[]".to_string());
            let cast: Vec<CastMember> = serde_json::from_str(&cast_json).unwrap_or_default();

            let external_ids_json: String = row.try_get("external_ids").unwrap_or_else(|_| "{}".to_string());
            let external_ids: ExternalIds = serde_json::from_str(&external_ids_json).unwrap_or_default();

            let user_edited_fields_json: String = row.try_get("user_edited_fields").unwrap_or_else(|_| "[]".to_string());
            let user_edited_fields: HashSet<String> = serde_json::from_str(&user_edited_fields_json).unwrap_or_default();

            let alternative_titles_json: Option<String> = row.try_get("alternative_titles").ok();
            let alternative_titles: Vec<String> = alternative_titles_json
                .and_then(|json| serde_json::from_str(&json).ok())
                .unwrap_or_default();

            let watch_status_str: String = row.try_get("watch_status").unwrap_or_else(|_| "unwatched".to_string());
            let watch_status = match watch_status_str.as_str() {
                "in_progress" => WatchStatus::InProgress,
                "watched" => WatchStatus::Watched,
                _ => WatchStatus::Unwatched,
            };

            let media_type_str: String = row.try_get("media_type")?;
            let media_type = match media_type_str.as_str() {
                "series" => MediaType::Series,
                _ => MediaType::Movie,
            };

            let created_at_ts: i64 = row.try_get("created_at")?;
            let updated_at_ts: i64 = row.try_get("updated_at")?;

            let last_watched_at = row.try_get::<Option<i64>, _>("last_watched_at")
                .ok()
                .flatten()
                .map(|ts| DateTime::from_timestamp(ts, 0).unwrap_or_else(|| Utc::now()));

            let metadata_fetched_at = row.try_get::<Option<i64>, _>("metadata_fetched_at")
                .ok()
                .flatten()
                .map(|ts| DateTime::from_timestamp(ts, 0).unwrap_or_else(|| Utc::now()));

            Ok(Some(MediaItem {
                id,
                title: row.try_get("title")?,
                normalized_title: row.try_get("normalized_title")?,
                original_title: row.try_get("original_title").ok(),
                year: row.try_get("year").ok(),
                media_type,
                overview: row.try_get("overview").ok(),
                genres,
                cast,
                director: row.try_get("director").ok(),
                runtime: row.try_get::<Option<i64>, _>("runtime").ok().flatten().map(|r| r as u64),
                poster: None, // TODO: Load from poster_path
                backdrop: None, // TODO: Load from backdrop_path
                external_ids,
                user_rating: row.try_get("user_rating").ok(),
                watch_status,
                last_watched_at,
                custom_notes: row.try_get("custom_notes").ok(),
                tags: Vec::new(), // TODO: Load from junction table
                metadata_source: row.try_get("metadata_source").ok(),
                metadata_fetched_at,
                user_edited_fields,
                alternative_titles,
                custom_poster: row.try_get("custom_poster").ok(),
                custom_backdrop: row.try_get("custom_backdrop").ok(),
                created_at: DateTime::from_timestamp(created_at_ts, 0).unwrap_or_else(|| Utc::now()),
                updated_at: DateTime::from_timestamp(updated_at_ts, 0).unwrap_or_else(|| Utc::now()),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn update_media_item(&self, media_item: &MediaItem) -> Result<()> {
        let user_edited_fields_json = serde_json::to_string(&media_item.user_edited_fields)?;
        let genres_json = serde_json::to_string(&media_item.genres)?;
        let cast_json = serde_json::to_string(&media_item.cast)?;
        let external_ids_json = serde_json::to_string(&media_item.external_ids)?;
        let alternative_titles_json = serde_json::to_string(&media_item.alternative_titles)?;

        sqlx::query(
            r#"
            UPDATE media_items
            SET title = ?,
                normalized_title = ?,
                original_title = ?,
                year = ?,
                overview = ?,
                genres = ?,
                cast = ?,
                director = ?,
                runtime = ?,
                user_rating = ?,
                watch_status = ?,
                last_watched_at = ?,
                custom_notes = ?,
                user_edited_fields = ?,
                alternative_titles = ?,
                custom_poster = ?,
                custom_backdrop = ?,
                updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&media_item.title)
        .bind(&media_item.normalized_title)
        .bind(&media_item.original_title)
        .bind(media_item.year)
        .bind(&media_item.overview)
        .bind(&genres_json)
        .bind(&cast_json)
        .bind(&media_item.director)
        .bind(media_item.runtime.map(|r| r as i64))
        .bind(media_item.user_rating)
        .bind(media_item.watch_status.to_string())
        .bind(media_item.last_watched_at.map(|dt| dt.timestamp()))
        .bind(&media_item.custom_notes)
        .bind(&user_edited_fields_json)
        .bind(&alternative_titles_json)
        .bind(&media_item.custom_poster)
        .bind(&media_item.custom_backdrop)
        .bind(media_item.updated_at.timestamp())
        .bind(media_item.id.0)
        .execute(self.pool())
        .await?;

        Ok(())
    }

    pub async fn delete_media_item(&self, _id: MediaItemId) -> Result<()> {
        // Stub implementation
        Ok(())
    }

    // ========================================================================
    // FileVersion Operations (Stubs for now)
    // ========================================================================

    pub async fn create_file_version(&self, _version: &FileVersion) -> Result<FileVersionId> {
        // Stub implementation
        Ok(FileVersionId(1))
    }

    pub async fn get_file_version(&self, _id: FileVersionId) -> Result<Option<FileVersion>> {
        // Stub implementation
        Ok(None)
    }

    pub async fn get_file_versions_for_media(&self, _media_id: MediaItemId) -> Result<Vec<FileVersion>> {
        // Stub implementation
        Ok(Vec::new())
    }

    pub async fn update_file_version(&self, _version: &FileVersion) -> Result<()> {
        // Stub implementation
        Ok(())
    }

    // ========================================================================
    // LibraryRoot Operations
    // ========================================================================

    /// Create a new library root
    pub async fn create_library_root(&self, root: &LibraryRoot) -> Result<LibraryRootId> {
        let result = sqlx::query(
            r#"
            INSERT INTO library_roots (path, name, is_active, is_archive, filesystem_type, mount_point, total_space, free_space, last_scanned_at, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(root.path.to_string_lossy().to_string())
        .bind(&root.name)
        .bind(root.is_active)
        .bind(root.is_archive)
        .bind(&root.filesystem_type)
        .bind(root.mount_point.as_ref().map(|p| p.to_string_lossy().to_string()))
        .bind(root.total_space.map(|s| s as i64))
        .bind(root.free_space.map(|s| s as i64))
        .bind(root.last_scanned_at.map(|dt| dt.timestamp()))
        .bind(Utc::now().timestamp())
        .execute(self.pool())
        .await?;

        Ok(LibraryRootId(result.last_insert_rowid()))
    }

    /// Get a library root by ID
    pub async fn get_library_root(&self, id: LibraryRootId) -> Result<Option<LibraryRoot>> {
        let row = sqlx::query(
            r#"
            SELECT id, path, name, is_active, is_archive, filesystem_type, mount_point, total_space, free_space, last_scanned_at, created_at
            FROM library_roots
            WHERE id = ?
            "#
        )
        .bind(id.0)
        .fetch_optional(self.pool())
        .await?;

        if let Some(row) = row {
            let path_str: String = row.get("path");
            let mount_point_str: Option<String> = row.get("mount_point");
            let total_space: Option<i64> = row.get("total_space");
            let free_space: Option<i64> = row.get("free_space");
            let last_scanned_at_ts: Option<i64> = row.get("last_scanned_at");

            Ok(Some(LibraryRoot {
                id,
                path: PathBuf::from(path_str),
                name: row.get("name"),
                is_active: row.get("is_active"),
                is_archive: row.get("is_archive"),
                filesystem_type: row.get("filesystem_type"),
                mount_point: mount_point_str.map(PathBuf::from),
                total_space: total_space.map(|s| s as u64),
                free_space: free_space.map(|s| s as u64),
                last_scanned_at: last_scanned_at_ts.and_then(|ts| DateTime::from_timestamp(ts, 0)),
            }))
        } else {
            Ok(None)
        }
    }

    /// Get all library roots
    pub async fn get_all_library_roots(&self) -> Result<Vec<LibraryRoot>> {
        let rows = sqlx::query(
            r#"
            SELECT id, path, name, is_active, is_archive, filesystem_type, mount_point, total_space, free_space, last_scanned_at, created_at
            FROM library_roots
            ORDER BY created_at ASC
            "#
        )
        .fetch_all(self.pool())
        .await?;

        let mut roots = Vec::new();
        for row in rows {
            let id: i64 = row.get("id");
            let path_str: String = row.get("path");
            let mount_point_str: Option<String> = row.get("mount_point");
            let total_space: Option<i64> = row.get("total_space");
            let free_space: Option<i64> = row.get("free_space");
            let last_scanned_at_ts: Option<i64> = row.get("last_scanned_at");

            roots.push(LibraryRoot {
                id: LibraryRootId(id),
                path: PathBuf::from(path_str),
                name: row.get("name"),
                is_active: row.get("is_active"),
                is_archive: row.get("is_archive"),
                filesystem_type: row.get("filesystem_type"),
                mount_point: mount_point_str.map(PathBuf::from),
                total_space: total_space.map(|s| s as u64),
                free_space: free_space.map(|s| s as u64),
                last_scanned_at: last_scanned_at_ts.and_then(|ts| DateTime::from_timestamp(ts, 0)),
            });
        }

        Ok(roots)
    }

    /// Get active library roots
    pub async fn get_active_library_roots(&self) -> Result<Vec<LibraryRoot>> {
        let rows = sqlx::query(
            r#"
            SELECT id, path, name, is_active, is_archive, filesystem_type, mount_point, total_space, free_space, last_scanned_at, created_at
            FROM library_roots
            WHERE is_active = 1
            ORDER BY created_at ASC
            "#
        )
        .fetch_all(self.pool())
        .await?;

        let mut roots = Vec::new();
        for row in rows {
            let id: i64 = row.get("id");
            let path_str: String = row.get("path");
            let mount_point_str: Option<String> = row.get("mount_point");
            let total_space: Option<i64> = row.get("total_space");
            let free_space: Option<i64> = row.get("free_space");
            let last_scanned_at_ts: Option<i64> = row.get("last_scanned_at");

            roots.push(LibraryRoot {
                id: LibraryRootId(id),
                path: PathBuf::from(path_str),
                name: row.get("name"),
                is_active: row.get("is_active"),
                is_archive: row.get("is_archive"),
                filesystem_type: row.get("filesystem_type"),
                mount_point: mount_point_str.map(PathBuf::from),
                total_space: total_space.map(|s| s as u64),
                free_space: free_space.map(|s| s as u64),
                last_scanned_at: last_scanned_at_ts.and_then(|ts| DateTime::from_timestamp(ts, 0)),
            });
        }

        Ok(roots)
    }

    /// Update a library root
    pub async fn update_library_root(&self, root: &LibraryRoot) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE library_roots
            SET path = ?, name = ?, is_active = ?, is_archive = ?, filesystem_type = ?, mount_point = ?, total_space = ?, free_space = ?, last_scanned_at = ?
            WHERE id = ?
            "#
        )
        .bind(root.path.to_string_lossy().to_string())
        .bind(&root.name)
        .bind(root.is_active)
        .bind(root.is_archive)
        .bind(&root.filesystem_type)
        .bind(root.mount_point.as_ref().map(|p| p.to_string_lossy().to_string()))
        .bind(root.total_space.map(|s| s as i64))
        .bind(root.free_space.map(|s| s as i64))
        .bind(root.last_scanned_at.map(|dt| dt.timestamp()))
        .bind(root.id.0)
        .execute(self.pool())
        .await?;

        Ok(())
    }

    /// Delete a library root
    pub async fn delete_library_root(&self, id: LibraryRootId) -> Result<()> {
        sqlx::query("DELETE FROM library_roots WHERE id = ?")
            .bind(id.0)
            .execute(self.pool())
            .await?;

        Ok(())
    }

    /// Get library root by path
    pub async fn get_library_root_by_path(&self, path: &std::path::Path) -> Result<Option<LibraryRoot>> {
        let path_str = path.to_string_lossy().to_string();
        let row = sqlx::query(
            r#"
            SELECT id, path, name, is_active, is_archive, filesystem_type, mount_point, total_space, free_space, last_scanned_at, created_at
            FROM library_roots
            WHERE path = ?
            "#
        )
        .bind(&path_str)
        .fetch_optional(self.pool())
        .await?;

        if let Some(row) = row {
            let id: i64 = row.get("id");
            let mount_point_str: Option<String> = row.get("mount_point");
            let total_space: Option<i64> = row.get("total_space");
            let free_space: Option<i64> = row.get("free_space");
            let last_scanned_at_ts: Option<i64> = row.get("last_scanned_at");

            Ok(Some(LibraryRoot {
                id: LibraryRootId(id),
                path: PathBuf::from(path_str),
                name: row.get("name"),
                is_active: row.get("is_active"),
                is_archive: row.get("is_archive"),
                filesystem_type: row.get("filesystem_type"),
                mount_point: mount_point_str.map(PathBuf::from),
                total_space: total_space.map(|s| s as u64),
                free_space: free_space.map(|s| s as u64),
                last_scanned_at: last_scanned_at_ts.and_then(|ts| DateTime::from_timestamp(ts, 0)),
            }))
        } else {
            Ok(None)
        }
    }

    // ========================================================================
    // Tag Operations (Stubs for now)
    // ========================================================================

    pub async fn get_or_create_tag(&self, _name: &str) -> Result<Tag> {
        // Stub implementation
        Ok(Tag {
            id: TagId(1),
            name: "stub".to_string(),
            color: None,
            created_at: Utc::now(),
        })
    }

    pub async fn get_tag_by_name(&self, _name: &str) -> Result<Option<Tag>> {
        // Stub implementation
        Ok(None)
    }

    pub async fn get_tags_for_media(&self, _media_id: MediaItemId) -> Result<Vec<Tag>> {
        // Stub implementation
        Ok(Vec::new())
    }

    pub async fn get_media_items_by_tag(&self, _tag_id: TagId) -> Result<Vec<MediaItem>> {
        // Stub implementation
        Ok(Vec::new())
    }

    // ========================================================================
    // Duplicate Detection (Stubs for now)
    // ========================================================================

    pub async fn get_duplicates(&self) -> Result<Vec<DuplicateGroup>> {
        // Stub implementation
        Ok(Vec::new())
    }

    // ========================================================================
    // Playback History Operations
    // ========================================================================

    /// Record a playback event
    pub async fn record_playback_event(&self, event: &PlaybackEvent) -> Result<PlaybackHistoryId> {
        let result = sqlx::query(
            r#"
            INSERT INTO playback_history (file_version_id, started_at, stopped_at, playback_position, duration, completed)
            VALUES (?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(event.file_version_id.0)
        .bind(event.started_at.timestamp())
        .bind(event.stopped_at.map(|dt| dt.timestamp()))
        .bind(event.playback_position.map(|p| p as i64))
        .bind(event.duration.map(|d| d as i64))
        .bind(event.completed)
        .execute(self.pool())
        .await?;

        Ok(PlaybackHistoryId(result.last_insert_rowid()))
    }

    /// Get playback history for a file version
    pub async fn get_playback_history(&self, file_version_id: FileVersionId) -> Result<Vec<PlaybackHistory>> {
        let rows = sqlx::query(
            r#"
            SELECT id, file_version_id, started_at, stopped_at, playback_position, duration, completed
            FROM playback_history
            WHERE file_version_id = ?
            ORDER BY started_at DESC
            "#
        )
        .bind(file_version_id.0)
        .fetch_all(self.pool())
        .await?;

        let mut history = Vec::new();
        for row in rows {
            let started_at_ts: i64 = row.get("started_at");
            let stopped_at_ts: Option<i64> = row.get("stopped_at");
            let playback_position: Option<i64> = row.get("playback_position");
            let duration: Option<i64> = row.get("duration");

            history.push(PlaybackHistory {
                id: PlaybackHistoryId(row.get("id")),
                file_version_id: FileVersionId(row.get("file_version_id")),
                started_at: DateTime::from_timestamp(started_at_ts, 0).unwrap_or_else(|| Utc::now()),
                stopped_at: stopped_at_ts.and_then(|ts| DateTime::from_timestamp(ts, 0)),
                playback_position: playback_position.map(|p| p as u64),
                duration: duration.map(|d| d as u64),
                completed: row.get("completed"),
            });
        }

        Ok(history)
    }

    /// Get the last playback position for a file version (for resume)
    pub async fn get_resume_info(&self, file_version_id: FileVersionId) -> Result<Option<ResumeInfo>> {
        let row = sqlx::query(
            r#"
            SELECT playback_position, started_at
            FROM playback_history
            WHERE file_version_id = ? AND playback_position IS NOT NULL
            ORDER BY started_at DESC
            LIMIT 1
            "#
        )
        .bind(file_version_id.0)
        .fetch_optional(self.pool())
        .await?;

        if let Some(row) = row {
            let playback_position: i64 = row.get("playback_position");
            let started_at_ts: i64 = row.get("started_at");

            Ok(Some(ResumeInfo {
                file_version_id,
                last_position: playback_position as u64,
                last_watched_at: DateTime::from_timestamp(started_at_ts, 0).unwrap_or_else(|| Utc::now()),
            }))
        } else {
            Ok(None)
        }
    }

    /// Update watch status based on playback
    pub async fn update_watch_status_from_playback(&self, media_item_id: MediaItemId, watch_status: WatchStatus) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE media_items
            SET watch_status = ?, last_watched_at = ?, updated_at = ?
            WHERE id = ?
            "#
        )
        .bind(watch_status.to_string())
        .bind(Utc::now().timestamp())
        .bind(Utc::now().timestamp())
        .bind(media_item_id.0)
        .execute(self.pool())
        .await?;

        Ok(())
    }

    /// Get all playback history for a media item (across all versions)
    pub async fn get_media_playback_history(&self, media_item_id: MediaItemId) -> Result<Vec<PlaybackHistory>> {
        let rows = sqlx::query(
            r#"
            SELECT ph.id, ph.file_version_id, ph.started_at, ph.stopped_at, ph.playback_position, ph.duration, ph.completed
            FROM playback_history ph
            INNER JOIN file_versions fv ON ph.file_version_id = fv.id
            WHERE fv.media_item_id = ?
            ORDER BY ph.started_at DESC
            "#
        )
        .bind(media_item_id.0)
        .fetch_all(self.pool())
        .await?;

        let mut history = Vec::new();
        for row in rows {
            let started_at_ts: i64 = row.get("started_at");
            let stopped_at_ts: Option<i64> = row.get("stopped_at");
            let playback_position: Option<i64> = row.get("playback_position");
            let duration: Option<i64> = row.get("duration");

            history.push(PlaybackHistory {
                id: PlaybackHistoryId(row.get("id")),
                file_version_id: FileVersionId(row.get("file_version_id")),
                started_at: DateTime::from_timestamp(started_at_ts, 0).unwrap_or_else(|| Utc::now()),
                stopped_at: stopped_at_ts.and_then(|ts| DateTime::from_timestamp(ts, 0)),
                playback_position: playback_position.map(|p| p as u64),
                duration: duration.map(|d| d as u64),
                completed: row.get("completed"),
            });
        }

        Ok(history)
    }

    // ========================================================================
    // Helper Methods
    // ========================================================================

    fn row_to_media_item(&self, row: &sqlx::sqlite::SqliteRow) -> Result<MediaItem> {
        // Stub implementation - parse row into MediaItem
        Ok(MediaItem {
            id: MediaItemId(row.get("id")),
            title: row.get("title"),
            normalized_title: row.get("normalized_title"),
            original_title: row.get("original_title"),
            year: row.get("year"),
            media_type: MediaType::Movie, // TODO: parse from string
            overview: row.get("overview"),
            genres: Vec::new(), // TODO: parse from JSON
            cast: Vec::new(), // TODO: parse from JSON
            director: row.get("director"),
            runtime: row.get("runtime"),
            poster: None, // TODO: parse from JSON
            backdrop: None, // TODO: parse from JSON
            external_ids: ExternalIds {
                tmdb_id: None,
                imdb_id: None,
                kinopoisk_id: None,
            }, // TODO: parse from JSON
            user_rating: row.get("user_rating"),
            watch_status: WatchStatus::Unwatched, // TODO: parse from string
            last_watched_at: row.get("last_watched_at"),
            custom_notes: row.get("custom_notes"),
            tags: Vec::new(), // TODO: load from junction table
            metadata_source: row.get("metadata_source"),
            metadata_fetched_at: row.get("metadata_fetched_at"),
            user_edited_fields: std::collections::HashSet::new(), // TODO: parse from JSON
            alternative_titles: Vec::new(), // TODO: parse from JSON
            custom_poster: None, // TODO: load from database
            custom_backdrop: None, // TODO: load from database
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }
}

