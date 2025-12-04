// Library database interface and operations
// Provides database access for media items, file versions, and related entities

use anyhow::{Result, Context};
use sqlx::{SqlitePool, Row};
use super::models::*;
use chrono::Utc;
use serde_json;

/// Database interface for library operations
pub struct LibraryDatabase {
    pool: SqlitePool,
}

impl LibraryDatabase {
    /// Create a new LibraryDatabase instance
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Get the connection pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    // ========================================================================
    // MediaItem Operations
    // ========================================================================

    /// Create a new MediaItem
    pub async fn create_media_item(&self, media_item: &MediaItem) -> Result<MediaItemId> {
        let user_edited_fields = serde_json::to_string(&media_item.user_edited_fields)
            .context("Failed to serialize user_edited_fields")?;
        let genres = serde_json::to_string(&media_item.genres)
            .context("Failed to serialize genres")?;
        let cast = serde_json::to_string(&media_item.cast)
            .context("Failed to serialize cast")?;
        let external_ids = serde_json::to_string(&media_item.external_ids)
            .context("Failed to serialize external_ids")?;
        let alternative_titles = serde_json::to_string(&media_item.alternative_titles)
            .context("Failed to serialize alternative_titles")?;

        let result = sqlx::query(
            "INSERT INTO media_items (
                title, normalized_title, original_title, year, media_type, overview,
                genres, cast, director, runtime, poster_url, poster_path, backdrop_url,
                backdrop_path, external_ids, user_rating, watch_status, last_watched_at,
                custom_notes, metadata_source, metadata_fetched_at, user_edited_fields,
                alternative_titles, custom_poster, custom_backdrop,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&media_item.title)
        .bind(&media_item.normalized_title)
        .bind(&media_item.original_title)
        .bind(media_item.year)
        .bind(match media_item.media_type {
            MediaType::Movie => "movie",
            MediaType::Series => "series",
        })
        .bind(&media_item.overview)
        .bind(&genres)
        .bind(&cast)
        .bind(&media_item.director)
        .bind(media_item.runtime.map(|r| r as i64))
        .bind(media_item.poster.as_ref().and_then(|p| p.url.clone()))
        .bind(media_item.poster.as_ref().and_then(|p| p.local_path.as_ref().map(|p| p.to_string_lossy().to_string())))
        .bind(media_item.backdrop.as_ref().and_then(|b| b.url.clone()))
        .bind(media_item.backdrop.as_ref().and_then(|b| b.local_path.as_ref().map(|p| p.to_string_lossy().to_string())))
        .bind(&external_ids)
        .bind(media_item.user_rating)
        .bind(match media_item.watch_status {
            WatchStatus::Unwatched => "unwatched",
            WatchStatus::InProgress => "in_progress",
            WatchStatus::Watched => "watched",
        })
        .bind(media_item.last_watched_at)
        .bind(&media_item.custom_notes)
        .bind(&media_item.metadata_source)
        .bind(media_item.metadata_fetched_at)
        .bind(&user_edited_fields)
        .bind(&alternative_titles)
        .bind(&media_item.custom_poster)
        .bind(&media_item.custom_backdrop)
        .bind(media_item.created_at)
        .bind(media_item.updated_at)
        .execute(&self.pool)
        .await
        .context("Failed to insert media item")?;

        Ok(MediaItemId(result.last_insert_rowid()))
    }

    /// Get a MediaItem by ID
    pub async fn get_media_item(&self, id: MediaItemId) -> Result<Option<MediaItem>> {
        let row = sqlx::query(
            "SELECT * FROM media_items WHERE id = ?"
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch media item")?;

        match row {
            Some(row) => {
                let mut media_item = self.row_to_media_item(&row)?;
                // Load tags from junction table
                media_item.tags = self.get_tag_ids_for_media(id).await.unwrap_or_default();
                Ok(Some(media_item))
            }
            None => Ok(None),
        }
    }

    /// Get all MediaItems
    pub async fn get_all_media_items(&self) -> Result<Vec<MediaItem>> {
        let rows = sqlx::query("SELECT * FROM media_items ORDER BY title")
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch all media items")?;

        let mut items = Vec::new();
        for row in rows {
            let mut media_item = self.row_to_media_item(&row)?;
            // Load tags for each item
            media_item.tags = self.get_tag_ids_for_media(media_item.id).await.unwrap_or_default();
            items.push(media_item);
        }

        Ok(items)
    }

    /// Update a MediaItem
    pub async fn update_media_item(&self, media_item: &MediaItem) -> Result<()> {
        let user_edited_fields = serde_json::to_string(&media_item.user_edited_fields)
            .context("Failed to serialize user_edited_fields")?;
        let genres = serde_json::to_string(&media_item.genres)
            .context("Failed to serialize genres")?;
        let cast = serde_json::to_string(&media_item.cast)
            .context("Failed to serialize cast")?;
        let external_ids = serde_json::to_string(&media_item.external_ids)
            .context("Failed to serialize external_ids")?;
        let alternative_titles = serde_json::to_string(&media_item.alternative_titles)
            .context("Failed to serialize alternative_titles")?;

        sqlx::query(
            r#"
            UPDATE media_items SET
                title = ?, normalized_title = ?, original_title = ?, year = ?, media_type = ?,
                overview = ?, genres = ?, cast = ?, director = ?, runtime = ?,
                poster_url = ?, poster_path = ?, backdrop_url = ?, backdrop_path = ?,
                external_ids = ?, user_rating = ?, watch_status = ?, last_watched_at = ?,
                custom_notes = ?, metadata_source = ?, metadata_fetched_at = ?,
                user_edited_fields = ?, alternative_titles = ?, custom_poster = ?, custom_backdrop = ?,
                updated_at = ?
            WHERE id = ?
            "#
        )
        .bind(&media_item.title)
        .bind(&media_item.normalized_title)
        .bind(&media_item.original_title)
        .bind(media_item.year)
        .bind(match media_item.media_type {
            MediaType::Movie => "movie",
            MediaType::Series => "series",
        })
        .bind(&media_item.overview)
        .bind(&genres)
        .bind(&cast)
        .bind(&media_item.director)
        .bind(media_item.runtime.map(|r| r as i64))
        .bind(media_item.poster.as_ref().and_then(|p| p.url.clone()))
        .bind(media_item.poster.as_ref().and_then(|p| p.local_path.as_ref().map(|p| p.to_string_lossy().to_string())))
        .bind(media_item.backdrop.as_ref().and_then(|b| b.url.clone()))
        .bind(media_item.backdrop.as_ref().and_then(|b| b.local_path.as_ref().map(|p| p.to_string_lossy().to_string())))
        .bind(&external_ids)
        .bind(media_item.user_rating)
        .bind(match media_item.watch_status {
            WatchStatus::Unwatched => "unwatched",
            WatchStatus::InProgress => "in_progress",
            WatchStatus::Watched => "watched",
        })
        .bind(media_item.last_watched_at)
        .bind(&media_item.custom_notes)
        .bind(&media_item.metadata_source)
        .bind(media_item.metadata_fetched_at)
        .bind(&user_edited_fields)
        .bind(&alternative_titles)
        .bind(&media_item.custom_poster)
        .bind(&media_item.custom_backdrop)
        .bind(media_item.updated_at)
        .bind(media_item.id.0)
        .execute(&self.pool)
        .await
        .context("Failed to update media item")?;

        // Update tag associations
        self.sync_media_item_tags(media_item.id, &media_item.tags).await?;

        Ok(())
    }

    /// Sync media_item_tags junction table with the tags list
    async fn sync_media_item_tags(&self, media_id: MediaItemId, tag_ids: &[TagId]) -> Result<()> {
        // Delete all existing associations
        sqlx::query("DELETE FROM media_item_tags WHERE media_item_id = ?")
            .bind(media_id.0)
            .execute(&self.pool)
            .await
            .context("Failed to delete existing tag associations")?;

        // Insert new associations
        for tag_id in tag_ids {
            sqlx::query(
                r#"
                INSERT INTO media_item_tags (media_item_id, tag_id)
                VALUES (?, ?)
                "#
            )
            .bind(media_id.0)
            .bind(tag_id.0)
            .execute(&self.pool)
            .await
            .context("Failed to create tag association")?;
        }

        Ok(())
    }

    /// Delete a MediaItem
    pub async fn delete_media_item(&self, id: MediaItemId) -> Result<()> {
        sqlx::query("DELETE FROM media_items WHERE id = ?")
            .bind(id.0)
            .execute(&self.pool)
            .await
            .context("Failed to delete media item")?;

        Ok(())
    }

    // Helper function to convert database row to MediaItem
    fn row_to_media_item(&self, row: &sqlx::sqlite::SqliteRow) -> Result<MediaItem> {
        use sqlx::Row;

        let user_edited_fields_str: String = row.get("user_edited_fields");
        let user_edited_fields = serde_json::from_str(&user_edited_fields_str)
            .context("Failed to deserialize user_edited_fields")?;

        let genres_str: String = row.get("genres");
        let genres = serde_json::from_str(&genres_str)
            .context("Failed to deserialize genres")?;

        let cast_str: String = row.get("cast");
        let cast = serde_json::from_str(&cast_str)
            .context("Failed to deserialize cast")?;

        let external_ids_str: String = row.get("external_ids");
        let external_ids = serde_json::from_str(&external_ids_str)
            .context("Failed to deserialize external_ids")?;

        let media_type_str: String = row.get("media_type");
        let media_type = match media_type_str.as_str() {
            "movie" => MediaType::Movie,
            "series" => MediaType::Series,
            _ => MediaType::Movie,
        };

        let watch_status_str: String = row.get("watch_status");
        let watch_status = match watch_status_str.as_str() {
            "unwatched" => WatchStatus::Unwatched,
            "in_progress" => WatchStatus::InProgress,
            "watched" => WatchStatus::Watched,
            _ => WatchStatus::Unwatched,
        };

        Ok(MediaItem {
            id: MediaItemId(row.get("id")),
            title: row.get("title"),
            normalized_title: row.get("normalized_title"),
            original_title: row.get("original_title"),
            year: row.get("year"),
            media_type,
            overview: row.get("overview"),
            genres,
            cast,
            director: row.get("director"),
            runtime: row.get("runtime"),
            poster: {
                let url: Option<String> = row.get("poster_url");
                let local_path: Option<String> = row.get("poster_path");
                if url.is_some() || local_path.is_some() {
                    Some(ImageData {
                        url,
                        local_path: local_path.map(std::path::PathBuf::from),
                    })
                } else {
                    None
                }
            },
            backdrop: {
                let url: Option<String> = row.get("backdrop_url");
                let local_path: Option<String> = row.get("backdrop_path");
                if url.is_some() || local_path.is_some() {
                    Some(ImageData {
                        url,
                        local_path: local_path.map(std::path::PathBuf::from),
                    })
                } else {
                    None
                }
            },
            external_ids,
            user_rating: row.get("user_rating"),
            watch_status,
            last_watched_at: row.get("last_watched_at"),
            custom_notes: row.get("custom_notes"),
            tags: vec![], // Will be loaded separately
            metadata_source: row.get("metadata_source"),
            metadata_fetched_at: row.get("metadata_fetched_at"),
            user_edited_fields,
            alternative_titles: {
                let alt_titles_str: Option<String> = row.try_get("alternative_titles").ok().flatten();
                alt_titles_str
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_default()
            },
            custom_poster: row.try_get("custom_poster").ok().flatten(),
            custom_backdrop: row.try_get("custom_backdrop").ok().flatten(),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    // Helper function to convert database row to FileVersion
    fn row_to_file_version(&self, row: &sqlx::sqlite::SqliteRow) -> Result<FileVersion> {
        use sqlx::Row;

        let status_str: String = row.get("status");
        let status = match status_str.as_str() {
            "present" => FileStatus::Present,
            "missing" => FileStatus::Missing,
            "trashed" => FileStatus::Trashed,
            "corrupted" => FileStatus::Corrupted,
            _ => FileStatus::Present,
        };

        let hashing_status_str: String = row.get("hashing_status");
        let hashing_status = match hashing_status_str.as_str() {
            "pending" => HashingStatus::Pending,
            "in_progress" => HashingStatus::InProgress,
            "complete" => HashingStatus::Complete,
            "failed" => HashingStatus::Failed,
            _ => HashingStatus::Pending,
        };

        let source_type_str: Option<String> = row.get("source_type");
        let source_type = source_type_str.and_then(|st| match st.as_str() {
            "bluray" => Some(SourceType::BluRay),
            "webdl" => Some(SourceType::WebDl),
            "hdtv" => Some(SourceType::Hdtv),
            "dvd" => Some(SourceType::Dvd),
            "unknown" => Some(SourceType::Unknown),
            _ => None,
        });

        let audio_tracks_str: Option<String> = row.get("audio_tracks");
        let audio_tracks = audio_tracks_str
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        let subtitle_tracks_str: Option<String> = row.get("subtitle_tracks");
        let subtitle_tracks = subtitle_tracks_str
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        let fast_hash_bytes: Option<Vec<u8>> = row.get("fast_hash");
        let fast_hash = fast_hash_bytes.map(FastHash);

        let full_hash_bytes: Option<Vec<u8>> = row.get("full_hash");
        let hash_algorithm: Option<String> = row.get("hash_algorithm");
        let full_hash = full_hash_bytes.zip(hash_algorithm).map(|(value, algorithm)| FullHash {
            algorithm,
            value,
        });

        let checksum_bytes: Option<Vec<u8>> = row.get("checksum");
        let checksum_algorithm: Option<String> = row.get("checksum_algorithm");
        let checksum = checksum_bytes.zip(checksum_algorithm).map(|(value, algorithm)| Checksum {
            algorithm,
            value,
        });

        let symlink_target_str: Option<String> = row.get("symlink_target");
        let symlink_target = symlink_target_str.map(std::path::PathBuf::from);

        let inode: Option<i64> = row.get("inode");

        let technical_metadata = {
            let container: Option<String> = row.get("container");
            let resolution_width: Option<i32> = row.get("resolution_width");
            let resolution_height: Option<i32> = row.get("resolution_height");
            let video_codec: Option<String> = row.get("video_codec");
            let duration: Option<i64> = row.get("duration");
            let bitrate: Option<i64> = row.get("bitrate");
            let framerate: Option<f32> = row.get("framerate");

            if let Some(container) = container {
                let video = if let (Some(width), Some(height), Some(codec), Some(fps)) = 
                    (resolution_width, resolution_height, video_codec, framerate) {
                    Some(VideoInfo {
                        codec,
                        resolution: Resolution {
                            width: width as u32,
                            height: height as u32,
                        },
                        framerate: fps,
                        bitrate: 0, // Not stored separately
                        color_space: None,
                    })
                } else {
                    None
                };

                Some(TechnicalMetadata {
                    container,
                    video,
                    audio_tracks,
                    subtitle_tracks,
                    duration: duration.unwrap_or(0) as u64,
                    bitrate: bitrate.unwrap_or(0) as u64,
                    file_size: row.get::<i64, _>("file_size") as u64,
                })
            } else {
                None
            }
        };

        Ok(FileVersion {
            id: FileVersionId(row.get("id")),
            media_item_id: MediaItemId(row.get("media_item_id")),
            library_root_id: LibraryRootId(row.get("library_root_id")),
            relative_path: {
                let path_str: String = row.get("relative_path");
                std::path::PathBuf::from(path_str)
            },
            absolute_path: {
                let path_str: String = row.get("absolute_path");
                std::path::PathBuf::from(path_str)
            },
            file_size: row.get::<i64, _>("file_size") as u64,
            status,
            is_preferred: row.get("is_preferred"),
            technical_metadata,
            quality_label: row.get("quality_label"),
            release_group: row.get("release_group"),
            source_type,
            fast_hash,
            full_hash,
            hashing_status,
            checksum,
            last_verified_at: row.get("last_verified_at"),
            corruption_detected_at: row.get("corruption_detected_at"),
            filesystem_info: FilesystemInfo {
                is_symlink: row.get("is_symlink"),
                symlink_target,
                is_hardlink: row.get("is_hardlink"),
                inode: inode.map(|i| i as u64),
            },
            added_at: row.get("added_at"),
            last_seen_at: row.get("last_seen_at"),
            trashed_at: row.get("trashed_at"),
            original_path: {
                let path_str: Option<String> = row.get("original_path");
                path_str.map(std::path::PathBuf::from)
            },
            import_source: row.get("import_source"),
        })
    }

    // ========================================================================
    // FileVersion Operations
    // ========================================================================

    /// Create a new FileVersion
    pub async fn create_file_version(&self, file_version: &FileVersion) -> Result<FileVersionId> {
        let status_str = match file_version.status {
            FileStatus::Present => "present",
            FileStatus::Missing => "missing",
            FileStatus::Trashed => "trashed",
            FileStatus::Corrupted => "corrupted",
        };

        let hashing_status_str = match file_version.hashing_status {
            HashingStatus::Pending => "pending",
            HashingStatus::InProgress => "in_progress",
            HashingStatus::Complete => "complete",
            HashingStatus::Failed => "failed",
        };

        let source_type_str = file_version.source_type.map(|st| match st {
            SourceType::BluRay => "bluray",
            SourceType::WebDl => "webdl",
            SourceType::Hdtv => "hdtv",
            SourceType::Dvd => "dvd",
            SourceType::Unknown => "unknown",
        });

        let audio_tracks = file_version.technical_metadata.as_ref()
            .map(|tm| serde_json::to_string(&tm.audio_tracks).unwrap_or_default());
        let subtitle_tracks = file_version.technical_metadata.as_ref()
            .map(|tm| serde_json::to_string(&tm.subtitle_tracks).unwrap_or_default());

        let result = sqlx::query(
            r#"
            INSERT INTO file_versions (
                media_item_id, library_root_id, relative_path, absolute_path, file_size,
                status, is_preferred, container, resolution_width, resolution_height,
                video_codec, audio_codec, audio_tracks, subtitle_tracks, duration, bitrate,
                framerate, quality_label, release_group, source_type, fast_hash, full_hash,
                hash_algorithm, hashing_status, hashed_at, checksum, checksum_algorithm, last_verified_at,
                corruption_detected_at, is_symlink, symlink_target, is_hardlink, inode,
                added_at, last_seen_at, trashed_at, original_path, import_source, import_profile_id
            ) VALUES (
                ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
            )
            "#
        )
        .bind(file_version.media_item_id.0)
        .bind(file_version.library_root_id.0)
        .bind(file_version.relative_path.to_string_lossy().to_string())
        .bind(file_version.absolute_path.to_string_lossy().to_string())
        .bind(file_version.file_size as i64)
        .bind(status_str)
        .bind(file_version.is_preferred)
        .bind(file_version.technical_metadata.as_ref().map(|tm| &tm.container))
        .bind(file_version.technical_metadata.as_ref().and_then(|tm| tm.video.as_ref().map(|v| v.resolution.width as i32)))
        .bind(file_version.technical_metadata.as_ref().and_then(|tm| tm.video.as_ref().map(|v| v.resolution.height as i32)))
        .bind(file_version.technical_metadata.as_ref().and_then(|tm| tm.video.as_ref().map(|v| &v.codec)))
        .bind(file_version.technical_metadata.as_ref().and_then(|tm| tm.audio_tracks.first().map(|a| &a.codec)))
        .bind(audio_tracks)
        .bind(subtitle_tracks)
        .bind(file_version.technical_metadata.as_ref().map(|tm| tm.duration as i64))
        .bind(file_version.technical_metadata.as_ref().map(|tm| tm.bitrate as i64))
        .bind(file_version.technical_metadata.as_ref().and_then(|tm| tm.video.as_ref().map(|v| v.framerate)))
        .bind(&file_version.quality_label)
        .bind(&file_version.release_group)
        .bind(source_type_str)
        .bind(file_version.fast_hash.as_ref().map(|h| h.0.as_slice()))
        .bind(file_version.full_hash.as_ref().map(|h| h.value.as_slice()))
        .bind(file_version.full_hash.as_ref().map(|h| &h.algorithm))
        .bind(hashing_status_str)
        .bind::<Option<i64>>(None) // hashed_at - set when hashing completes
        .bind(file_version.checksum.as_ref().map(|c| c.value.as_slice()))
        .bind(file_version.checksum.as_ref().map(|c| &c.algorithm))
        .bind(file_version.last_verified_at)
        .bind(file_version.corruption_detected_at)
        .bind(file_version.filesystem_info.is_symlink)
        .bind(file_version.filesystem_info.symlink_target.as_ref().map(|p| p.to_string_lossy().to_string()))
        .bind(file_version.filesystem_info.is_hardlink)
        .bind(file_version.filesystem_info.inode.map(|i| i as i64))
        .bind(file_version.added_at)
        .bind(file_version.last_seen_at)
        .bind(file_version.trashed_at)
        .bind(file_version.original_path.as_ref().map(|p| p.to_string_lossy().to_string()))
        .bind(&file_version.import_source)
        .bind::<Option<i64>>(None) // import_profile_id - not set during creation
        .execute(&self.pool)
        .await
        .context("Failed to insert file version")?;

        Ok(FileVersionId(result.last_insert_rowid()))
    }

    /// Get a FileVersion by ID
    pub async fn get_file_version(&self, _id: FileVersionId) -> Result<Option<FileVersion>> {
        // TODO: Implement database query
        unimplemented!("get_file_version")
    }

    /// Get all FileVersions for a MediaItem
    pub async fn get_file_versions_for_media(&self, media_id: MediaItemId) -> Result<Vec<FileVersion>> {
        let rows = sqlx::query("SELECT * FROM file_versions WHERE media_item_id = ? ORDER BY added_at DESC")
            .bind(media_id.0)
            .fetch_all(&self.pool)
            .await
            .context("Failed to fetch file versions for media")?;

        let mut versions = Vec::new();
        for row in rows {
            versions.push(self.row_to_file_version(&row)?);
        }

        Ok(versions)
    }

    /// Update a FileVersion
    pub async fn update_file_version(&self, _file_version: &FileVersion) -> Result<()> {
        // TODO: Implement database update
        unimplemented!("update_file_version")
    }

    /// Update FileVersion status
    pub async fn update_file_version_status(
        &self,
        id: FileVersionId,
        status: FileStatus,
    ) -> Result<()> {
        let status_str = match status {
            FileStatus::Present => "present",
            FileStatus::Missing => "missing",
            FileStatus::Trashed => "trashed",
            FileStatus::Corrupted => "corrupted",
        };

        sqlx::query(
            r#"
            UPDATE file_versions
            SET status = ?,
                trashed_at = CASE WHEN ? = 'trashed' THEN CURRENT_TIMESTAMP ELSE trashed_at END
            WHERE id = ?
            "#,
        )
        .bind(status_str)
        .bind(status_str)
        .bind(id.0)
        .execute(&self.pool)
        .await
        .context("Failed to update file version status")?;

        Ok(())
    }

    /// Delete a FileVersion
    pub async fn delete_file_version(&self, _id: FileVersionId) -> Result<()> {
        // TODO: Implement database deletion
        unimplemented!("delete_file_version")
    }

    // ========================================================================
    // LibraryRoot Operations
    // ========================================================================

    /// Create a new LibraryRoot
    pub async fn create_library_root(&self, root: &LibraryRoot) -> Result<LibraryRootId> {
        let id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO library_roots (
                path, name, is_active, is_archive, filesystem_type, mount_point,
                total_space, free_space, last_scanned_at, created_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
            RETURNING id
            "#,
        )
        .bind(root.path.to_string_lossy().to_string())
        .bind(&root.name)
        .bind(root.is_active)
        .bind(root.is_archive)
        .bind(&root.filesystem_type)
        .bind(root.mount_point.as_ref().map(|p| p.to_string_lossy().to_string()))
        .bind(root.total_space.map(|s| s as i64))
        .bind(root.free_space.map(|s| s as i64))
        .bind(root.last_scanned_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(LibraryRootId(id))
    }

    /// Get a LibraryRoot by ID
    pub async fn get_library_root(&self, id: LibraryRootId) -> Result<Option<LibraryRoot>> {
        use sqlx::Row;
        
        let row = sqlx::query(
            "SELECT id, path, name, is_active, is_archive, filesystem_type, mount_point, total_space, free_space, last_scanned_at, created_at FROM library_roots WHERE id = ?"
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let path_str: String = row.get("path");
            let mount_point_str: Option<String> = row.get("mount_point");
            let total_space: Option<i64> = row.get("total_space");
            let free_space: Option<i64> = row.get("free_space");
            let last_scanned_at_ts: Option<i64> = row.get("last_scanned_at");

            Ok(Some(LibraryRoot {
                id,
                path: std::path::PathBuf::from(path_str),
                name: row.get("name"),
                is_active: row.get("is_active"),
                is_archive: row.get("is_archive"),
                filesystem_type: row.get("filesystem_type"),
                mount_point: mount_point_str.map(std::path::PathBuf::from),
                total_space: total_space.map(|s| s as u64),
                free_space: free_space.map(|s| s as u64),
                last_scanned_at: last_scanned_at_ts.and_then(|ts| chrono::DateTime::from_timestamp(ts, 0)),
            }))
        } else {
            Ok(None)
        }
    }

    /// Get all LibraryRoots
    pub async fn get_all_library_roots(&self) -> Result<Vec<LibraryRoot>> {
        use sqlx::Row;
        
        let rows = sqlx::query(
            "SELECT id, path, name, is_active, is_archive, filesystem_type, mount_point, total_space, free_space, last_scanned_at, created_at FROM library_roots ORDER BY created_at ASC"
        )
        .fetch_all(&self.pool)
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
                path: std::path::PathBuf::from(path_str),
                name: row.get("name"),
                is_active: row.get("is_active"),
                is_archive: row.get("is_archive"),
                filesystem_type: row.get("filesystem_type"),
                mount_point: mount_point_str.map(std::path::PathBuf::from),
                total_space: total_space.map(|s| s as u64),
                free_space: free_space.map(|s| s as u64),
                last_scanned_at: last_scanned_at_ts.and_then(|ts| chrono::DateTime::from_timestamp(ts, 0)),
            });
        }

        Ok(roots)
    }

    /// Update a LibraryRoot
    pub async fn update_library_root(&self, root: &LibraryRoot) -> Result<()> {
        sqlx::query(
            "UPDATE library_roots SET path = ?, name = ?, is_active = ?, is_archive = ?, filesystem_type = ?, mount_point = ?, total_space = ?, free_space = ?, last_scanned_at = ? WHERE id = ?"
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
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Delete a LibraryRoot
    pub async fn delete_library_root(&self, id: LibraryRootId) -> Result<()> {
        sqlx::query("DELETE FROM library_roots WHERE id = ?")
            .bind(id.0)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // ========================================================================
    // Collection Operations
    // ========================================================================

    /// Create a new Collection
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
        .execute(&self.pool)
        .await?;

        Ok(CollectionId(result.last_insert_rowid()))
    }

    /// Get a Collection by ID
    pub async fn get_collection(&self, id: CollectionId) -> Result<Option<Collection>> {
        use sqlx::Row;
        
        let row = sqlx::query(
            r#"
            SELECT id, name, description, is_smart, filter_criteria, created_at, updated_at
            FROM collections
            WHERE id = ?
            "#
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
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

    /// Get all Collections
    pub async fn get_all_collections(&self) -> Result<Vec<Collection>> {
        use sqlx::Row;
        
        let rows = sqlx::query(
            r#"
            SELECT id, name, description, is_smart, filter_criteria, created_at, updated_at
            FROM collections
            ORDER BY name
            "#
        )
        .fetch_all(&self.pool)
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

    /// Update a Collection
    pub async fn update_collection(&self, _collection: &Collection) -> Result<()> {
        // TODO: Implement database update
        unimplemented!("update_collection")
    }

    /// Delete a Collection
    pub async fn delete_collection(&self, _id: CollectionId) -> Result<()> {
        // TODO: Implement database deletion
        unimplemented!("delete_collection")
    }

    // ========================================================================
    // Tag Operations
    // ========================================================================

    /// Create a new Tag
    pub async fn create_tag(&self, tag: &Tag) -> Result<TagId> {
        let result = sqlx::query(
            r#"
            INSERT INTO tags (name, color, created_at)
            VALUES (?, ?, ?)
            "#
        )
        .bind(&tag.name)
        .bind(&tag.color)
        .bind(tag.created_at)
        .execute(&self.pool)
        .await
        .context("Failed to create tag")?;

        Ok(TagId(result.last_insert_rowid()))
    }

    /// Get a Tag by ID
    pub async fn get_tag(&self, id: TagId) -> Result<Option<Tag>> {
        let row = sqlx::query(
            r#"
            SELECT id, name, color, created_at
            FROM tags WHERE id = ?
            "#
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch tag")?;

        match row {
            Some(row) => {
                use sqlx::Row;
                Ok(Some(Tag {
                    id: TagId(row.get("id")),
                    name: row.get("name"),
                    color: row.get("color"),
                    created_at: row.get("created_at"),
                }))
            }
            None => Ok(None),
        }
    }

    /// Get all Tags
    pub async fn get_all_tags(&self) -> Result<Vec<Tag>> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, color, created_at
            FROM tags
            ORDER BY name
            "#
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch all tags")?;

        let tags = rows.into_iter().map(|row| {
            use sqlx::Row;
            Tag {
                id: TagId(row.get("id")),
                name: row.get("name"),
                color: row.get("color"),
                created_at: row.get("created_at"),
            }
        }).collect();

        Ok(tags)
    }

    /// Delete a Tag
    pub async fn delete_tag(&self, id: TagId) -> Result<()> {
        sqlx::query("DELETE FROM tags WHERE id = ?")
            .bind(id.0)
            .execute(&self.pool)
            .await
            .context("Failed to delete tag")?;

        Ok(())
    }

    // ========================================================================
    // ImportProfile Operations
    // ========================================================================

    /// Create a new ImportProfile
    pub async fn create_import_profile(&self, _profile: &ImportProfile) -> Result<ImportProfileId> {
        // TODO: Implement database insertion
        unimplemented!("create_import_profile")
    }

    /// Get an ImportProfile by ID
    pub async fn get_import_profile(&self, _id: ImportProfileId) -> Result<Option<ImportProfile>> {
        // TODO: Implement database query
        unimplemented!("get_import_profile")
    }

    /// Get all ImportProfiles
    pub async fn get_all_import_profiles(&self) -> Result<Vec<ImportProfile>> {
        // TODO: Implement database query
        unimplemented!("get_all_import_profiles")
    }

    /// Update an ImportProfile
    pub async fn update_import_profile(&self, _profile: &ImportProfile) -> Result<()> {
        // TODO: Implement database update
        unimplemented!("update_import_profile")
    }

    /// Delete an ImportProfile
    pub async fn delete_import_profile(&self, _id: ImportProfileId) -> Result<()> {
        // TODO: Implement database deletion
        unimplemented!("delete_import_profile")
    }

    // ========================================================================
    // WatchFolder Operations
    // ========================================================================

    /// Create a new WatchFolder
    pub async fn create_watch_folder(&self, _folder: &WatchFolder) -> Result<WatchFolderId> {
        // TODO: Implement database insertion
        unimplemented!("create_watch_folder")
    }

    /// Get a WatchFolder by ID
    pub async fn get_watch_folder(&self, _id: WatchFolderId) -> Result<Option<WatchFolder>> {
        // TODO: Implement database query
        unimplemented!("get_watch_folder")
    }

    /// Get all WatchFolders
    pub async fn get_all_watch_folders(&self) -> Result<Vec<WatchFolder>> {
        // TODO: Implement database query
        unimplemented!("get_all_watch_folders")
    }

    /// Update a WatchFolder
    pub async fn update_watch_folder(&self, _folder: &WatchFolder) -> Result<()> {
        // TODO: Implement database update
        unimplemented!("update_watch_folder")
    }

    /// Delete a WatchFolder
    pub async fn delete_watch_folder(&self, _id: WatchFolderId) -> Result<()> {
        // TODO: Implement database deletion
        unimplemented!("delete_watch_folder")
    }

    // ========================================================================
    // Additional Query Operations
    // ========================================================================

    /// Get duplicate groups by full hash
    /// Returns groups of FileVersions that have identical full_hash values
    /// Each group represents exact duplicates (same file content)
    pub async fn get_duplicates(&self) -> Result<Vec<DuplicateGroup>> {
        use sqlx::Row;
        
        // Query to find FileVersions grouped by full_hash where count > 1
        // This identifies exact duplicates (same hash = same content)
        let rows = sqlx::query(
            r#"
            SELECT full_hash, hash_algorithm
            FROM file_versions
            WHERE full_hash IS NOT NULL
              AND hashing_status = 'complete'
              AND status != 'trashed'
            GROUP BY full_hash, hash_algorithm
            HAVING COUNT(*) > 1
            "#
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to query duplicate hashes")?;

        let mut duplicate_groups = Vec::new();

        // For each duplicate hash, get all FileVersions with that hash
        for row in rows {
            let hash_bytes: Vec<u8> = row.get("full_hash");
            let algorithm: String = row.get("hash_algorithm");

            // Get all FileVersions with this hash
            let versions = sqlx::query(
                r#"
                SELECT id
                FROM file_versions
                WHERE full_hash = ?
                  AND hash_algorithm = ?
                  AND hashing_status = 'complete'
                  AND status != 'trashed'
                "#
            )
            .bind(&hash_bytes)
            .bind(&algorithm)
            .fetch_all(&self.pool)
            .await
            .context("Failed to query duplicate versions")?;

            let version_ids: Vec<FileVersionId> = versions
                .iter()
                .map(|row| FileVersionId(row.get("id")))
                .collect();

            if version_ids.len() > 1 {
                duplicate_groups.push(DuplicateGroup {
                    hash: FullHash {
                        algorithm,
                        value: hash_bytes,
                    },
                    versions: version_ids,
                });
            }
        }

        Ok(duplicate_groups)
    }

    /// Get or create a tag by name
    pub async fn get_or_create_tag(&self, name: &str) -> Result<Tag> {
        // Try to get existing tag
        if let Some(tag) = self.get_tag_by_name(name).await? {
            return Ok(tag);
        }

        // Create new tag
        let now = Utc::now();
        let result = sqlx::query(
            r#"
            INSERT INTO tags (name, created_at)
            VALUES (?, ?)
            "#
        )
        .bind(name)
        .bind(now)
        .execute(&self.pool)
        .await
        .context("Failed to create tag")?;

        Ok(Tag {
            id: TagId(result.last_insert_rowid()),
            name: name.to_string(),
            color: None,
            created_at: now,
        })
    }

    /// Get a tag by name
    pub async fn get_tag_by_name(&self, name: &str) -> Result<Option<Tag>> {
        let row = sqlx::query(
            r#"
            SELECT id, name, color, created_at
            FROM tags WHERE name = ?
            "#
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch tag by name")?;

        match row {
            Some(row) => {
                use sqlx::Row;
                Ok(Some(Tag {
                    id: TagId(row.get("id")),
                    name: row.get("name"),
                    color: row.get("color"),
                    created_at: row.get("created_at"),
                }))
            }
            None => Ok(None),
        }
    }

    /// Get tag IDs for a MediaItem
    async fn get_tag_ids_for_media(&self, media_id: MediaItemId) -> Result<Vec<TagId>> {
        let rows = sqlx::query(
            r#"
            SELECT tag_id
            FROM media_item_tags
            WHERE media_item_id = ?
            "#
        )
        .bind(media_id.0)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch tag IDs")?;

        Ok(rows.iter().map(|row| TagId(row.get("tag_id"))).collect())
    }

    /// Get tags for a MediaItem
    pub async fn get_tags_for_media(&self, media_id: MediaItemId) -> Result<Vec<Tag>> {
        let rows = sqlx::query(
            r#"
            SELECT t.id, t.name, t.color, t.created_at
            FROM tags t
            INNER JOIN media_item_tags mit ON t.id = mit.tag_id
            WHERE mit.media_item_id = ?
            ORDER BY t.name
            "#
        )
        .bind(media_id.0)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch tags for media item")?;

        let tags = rows.into_iter().map(|row| {
            use sqlx::Row;
            Tag {
                id: TagId(row.get("id")),
                name: row.get("name"),
                color: row.get("color"),
                created_at: row.get("created_at"),
            }
        }).collect();

        Ok(tags)
    }

    /// Add MediaItems to a Collection
    pub async fn add_to_collection(&self, collection_id: CollectionId, media_ids: &[MediaItemId]) -> Result<()> {
        let mut tx = self.pool.begin().await?;

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

    /// Get MediaItems in a Collection
    pub async fn get_collection_items(&self, collection_id: CollectionId) -> Result<Vec<MediaItem>> {
        let rows = sqlx::query(
            r#"
            SELECT m.id, m.title, m.normalized_title, m.original_title, m.year, m.media_type, m.overview,
                   m.genres, m.cast, m.director, m.runtime, m.poster_url, m.poster_path, m.backdrop_url,
                   m.backdrop_path, m.external_ids, m.user_rating, m.watch_status, m.last_watched_at,
                   m.custom_notes, m.metadata_source, m.metadata_fetched_at, m.user_edited_fields,
                   m.created_at, m.updated_at
            FROM media_items m
            INNER JOIN collection_items ci ON m.id = ci.media_item_id
            WHERE ci.collection_id = ?
            ORDER BY ci.added_at DESC
            "#
        )
        .bind(collection_id.0)
        .fetch_all(&self.pool)
        .await?;

        let mut items = Vec::new();
        for row in rows {
            items.push(self.row_to_media_item(&row)?);
        }

        Ok(items)
    }

    /// Execute a custom query to retrieve MediaItems
    pub async fn query_media_items(&self, query: &str) -> Result<Vec<MediaItem>> {
        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .context("Failed to execute media items query")?;

        let mut media_items = Vec::new();
        for row in rows {
            let media_item = self.row_to_media_item(&row)?;
            media_items.push(media_item);
        }

        Ok(media_items)
    }

    /// Get MediaItems by tag
    pub async fn get_media_items_by_tag(&self, tag_id: TagId) -> Result<Vec<MediaItem>> {
        let query = format!(
            r#"
            SELECT DISTINCT m.id, m.title, m.normalized_title, m.original_title, m.year, m.media_type, m.overview,
                   m.genres, m.cast, m.director, m.runtime, m.poster_url, m.poster_path, m.backdrop_url,
                   m.backdrop_path, m.external_ids, m.user_rating, m.watch_status, m.last_watched_at,
                   m.custom_notes, m.metadata_source, m.metadata_fetched_at, m.user_edited_fields,
                   m.created_at, m.updated_at
            FROM media_items m
            INNER JOIN media_item_tags mit ON m.id = mit.media_item_id
            WHERE mit.tag_id = {}
            ORDER BY m.title
            "#,
            tag_id.0
        );

        self.query_media_items(&query).await
    }

    // ========================================================================
    // Saved Search Operations
    // ========================================================================

    /// Create a new saved search
    pub async fn create_saved_search(&self, saved_search: &SavedSearch) -> Result<SavedSearchId> {
        let filters_json = serde_json::to_string(&saved_search.filters)
            .context("Failed to serialize filters")?;
        let options_json = serde_json::to_string(&saved_search.search_options)
            .context("Failed to serialize search_options")?;

        let result = sqlx::query(
            r#"
            INSERT INTO saved_searches (name, description, query, filters, search_options, created_at, last_used_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&saved_search.name)
        .bind(&saved_search.description)
        .bind(&saved_search.query)
        .bind(filters_json)
        .bind(options_json)
        .bind(saved_search.created_at)
        .bind(saved_search.last_used_at)
        .execute(&self.pool)
        .await
        .context("Failed to create saved search")?;

        Ok(SavedSearchId(result.last_insert_rowid()))
    }

    /// Get a saved search by ID
    pub async fn get_saved_search(&self, id: SavedSearchId) -> Result<Option<SavedSearch>> {
        let row = sqlx::query(
            r#"
            SELECT id, name, description, query, filters, search_options, created_at, last_used_at
            FROM saved_searches
            WHERE id = ?
            "#
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to get saved search")?;

        match row {
            Some(row) => {
                let filters_json: String = row.get("filters");
                let options_json: String = row.get("search_options");

                let filters: LibraryFilters = serde_json::from_str(&filters_json)
                    .context("Failed to deserialize filters")?;
                let search_options: SearchOptions = serde_json::from_str(&options_json)
                    .context("Failed to deserialize search_options")?;

                Ok(Some(SavedSearch {
                    id: SavedSearchId(row.get("id")),
                    name: row.get("name"),
                    description: row.get("description"),
                    query: row.get("query"),
                    filters,
                    search_options,
                    created_at: row.get("created_at"),
                    last_used_at: row.get("last_used_at"),
                }))
            }
            None => Ok(None),
        }
    }

    /// Get all saved searches
    pub async fn get_all_saved_searches(&self) -> Result<Vec<SavedSearch>> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, description, query, filters, search_options, created_at, last_used_at
            FROM saved_searches
            ORDER BY last_used_at DESC NULLS LAST, created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to get all saved searches")?;

        let mut saved_searches = Vec::new();
        for row in rows {
            let filters_json: String = row.get("filters");
            let options_json: String = row.get("search_options");

            let filters: LibraryFilters = serde_json::from_str(&filters_json)
                .context("Failed to deserialize filters")?;
            let search_options: SearchOptions = serde_json::from_str(&options_json)
                .context("Failed to deserialize search_options")?;

            saved_searches.push(SavedSearch {
                id: SavedSearchId(row.get("id")),
                name: row.get("name"),
                description: row.get("description"),
                query: row.get("query"),
                filters,
                search_options,
                created_at: row.get("created_at"),
                last_used_at: row.get("last_used_at"),
            });
        }

        Ok(saved_searches)
    }

    /// Update a saved search
    pub async fn update_saved_search(&self, saved_search: &SavedSearch) -> Result<()> {
        let filters_json = serde_json::to_string(&saved_search.filters)
            .context("Failed to serialize filters")?;
        let options_json = serde_json::to_string(&saved_search.search_options)
            .context("Failed to serialize search_options")?;

        sqlx::query(
            r#"
            UPDATE saved_searches
            SET name = ?, description = ?, query = ?, filters = ?, search_options = ?, last_used_at = ?
            WHERE id = ?
            "#
        )
        .bind(&saved_search.name)
        .bind(&saved_search.description)
        .bind(&saved_search.query)
        .bind(filters_json)
        .bind(options_json)
        .bind(saved_search.last_used_at)
        .bind(saved_search.id.0)
        .execute(&self.pool)
        .await
        .context("Failed to update saved search")?;

        Ok(())
    }

    /// Delete a saved search
    pub async fn delete_saved_search(&self, id: SavedSearchId) -> Result<()> {
        sqlx::query(
            r#"
            DELETE FROM saved_searches
            WHERE id = ?
            "#
        )
        .bind(id.0)
        .execute(&self.pool)
        .await
        .context("Failed to delete saved search")?;

        Ok(())
    }

    // ========================================================================
    // Audit Log Operations
    // ========================================================================

    /// Create an audit log entry
    pub async fn create_audit_log_entry(
        &self,
        operation_type: &str,
        entity_type: &str,
        entity_id: Option<i64>,
        old_value: Option<&str>,
        new_value: Option<&str>,
        initiator: &str,
    ) -> Result<i64> {
        let result = sqlx::query(
            r#"
            INSERT INTO audit_log (
                operation_type, entity_type, entity_id,
                old_value, new_value, initiator, timestamp
            )
            VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
            "#,
        )
        .bind(operation_type)
        .bind(entity_type)
        .bind(entity_id)
        .bind(old_value)
        .bind(new_value)
        .bind(initiator)
        .execute(&self.pool)
        .await
        .context("Failed to create audit log entry")?;

        Ok(result.last_insert_rowid())
    }

    /// Get all audit logs
    pub async fn get_all_audit_logs(&self) -> Result<Vec<AuditLogEntry>> {
        use sqlx::Row;
        
        let rows = sqlx::query(
            r#"
            SELECT id, operation_type, entity_type, entity_id,
                   old_value, new_value, initiator, error_message, timestamp
            FROM audit_log
            ORDER BY timestamp DESC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch audit logs")?;

        let mut logs = Vec::new();
        for row in rows {
            logs.push(AuditLogEntry {
                id: row.get("id"),
                operation_type: row.get("operation_type"),
                entity_type: row.get("entity_type"),
                entity_id: row.get("entity_id"),
                old_value: row.get("old_value"),
                new_value: row.get("new_value"),
                initiator: row.get("initiator"),
                error_message: row.get("error_message"),
                timestamp: row.get("timestamp"),
            });
        }

        Ok(logs)
    }

    // ========================================================================
    // Export/Import Helper Methods
    // ========================================================================

    /// Find a media item by normalized title and year
    pub async fn find_media_item_by_title_year(
        &self,
        normalized_title: &str,
        year: Option<i32>,
    ) -> Result<Option<MediaItem>> {
        use sqlx::Row;
        
        let row = sqlx::query(
            r#"
            SELECT * FROM media_items
            WHERE normalized_title = ? AND year = ?
            LIMIT 1
            "#
        )
        .bind(normalized_title)
        .bind(year)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to find media item by title and year")?;

        if let Some(row) = row {
            Ok(Some(self.row_to_media_item(&row)?))
        } else {
            Ok(None)
        }
    }

    /// Find a file version by absolute path
    pub async fn find_file_version_by_path(
        &self,
        _path: &std::path::Path,
    ) -> Result<Option<FileVersion>> {
        // TODO: Implement when file version queries are fully implemented
        Ok(None)
    }

    /// Get the database file path
    pub fn get_database_path(&self) -> std::path::PathBuf {
        // This is a simplified implementation
        // In a real implementation, we'd store the path when creating the connection
        std::path::PathBuf::from(".data/torrentflix.db")
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
                started_at: chrono::DateTime::from_timestamp(started_at_ts, 0).unwrap_or_else(|| Utc::now()),
                stopped_at: stopped_at_ts.and_then(|ts| chrono::DateTime::from_timestamp(ts, 0)),
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
                last_watched_at: chrono::DateTime::from_timestamp(started_at_ts, 0).unwrap_or_else(|| Utc::now()),
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
                started_at: chrono::DateTime::from_timestamp(started_at_ts, 0).unwrap_or_else(|| Utc::now()),
                stopped_at: stopped_at_ts.and_then(|ts| chrono::DateTime::from_timestamp(ts, 0)),
                playback_position: playback_position.map(|p| p as u64),
                duration: duration.map(|d| d as u64),
                completed: row.get("completed"),
            });
        }

        Ok(history)
    }
}
