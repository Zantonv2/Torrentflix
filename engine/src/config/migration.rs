use anyhow::{Result, Context};
use sqlx::SqlitePool;
use tracing::{info, debug};

/// Configuration migration manager for schema versioning
pub struct ConfigMigration;

impl ConfigMigration {
    /// Initialize the migration system and run pending migrations
    pub async fn initialize(pool: &SqlitePool) -> Result<()> {
        info!("Initializing migration system");
        
        // Create schema_version table if it doesn't exist
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS schema_version (
                id INTEGER PRIMARY KEY,
                version INTEGER NOT NULL UNIQUE,
                description TEXT NOT NULL,
                applied_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(pool)
        .await
        .context("Failed to create schema_version table")?;

        // Get current schema version
        let current_version: i64 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version"
        )
        .fetch_one(pool)
        .await
        .context("Failed to fetch current schema version")?;

        let current_version = current_version as u32;
        debug!("Current schema version: {}", current_version);

        // Run migrations
        Self::run_migrations(pool, current_version).await?;

        info!("Migration system initialized");
        Ok(())
    }

    /// Run all pending migrations
    async fn run_migrations(pool: &SqlitePool, current_version: u32) -> Result<()> {
        let migrations: Vec<(u32, &str)> = vec![
            (1u32, "Create settings table"),
            (2u32, "Create library management tables"),
        ];

        for (version, description) in migrations {
            if version > current_version {
                info!("Running migration {}: {}", version, description);
                
                match version {
                    1 => Self::migration_001(pool).await?,
                    2 => Self::migration_002(pool).await?,
                    _ => anyhow::bail!("Unknown migration version: {}", version),
                }
                
                // Record migration
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .context("Failed to get current time")?
                    .as_secs() as i64;

                sqlx::query(
                    "INSERT INTO schema_version (version, description, applied_at) VALUES (?, ?, ?)"
                )
                .bind(version as i64)
                .bind(description)
                .bind(now)
                .execute(pool)
                .await
                .context(format!("Failed to record migration {}", version))?;

                info!("Migration {} completed", version);
            }
        }

        Ok(())
    }

    /// Migration 001: Create settings table
    async fn migration_001(pool: &SqlitePool) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS settings (
                id INTEGER PRIMARY KEY,
                key TEXT UNIQUE NOT NULL,
                value TEXT NOT NULL,
                type TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(pool)
        .await
        .context("Failed to create settings table in migration")?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_settings_key ON settings(key)
            "#,
        )
        .execute(pool)
        .await
        .context("Failed to create settings index in migration")?;

        Ok(())
    }

    /// Migration 002: Create library management tables
    async fn migration_002(pool: &SqlitePool) -> Result<()> {
        // MediaItems table: logical media entities
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS media_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                normalized_title TEXT NOT NULL,
                original_title TEXT,
                year INTEGER,
                media_type TEXT NOT NULL CHECK(media_type IN ('movie', 'series')),
                overview TEXT,
                genres TEXT,
                cast TEXT,
                director TEXT,
                runtime INTEGER,
                poster_url TEXT,
                poster_path TEXT,
                backdrop_url TEXT,
                backdrop_path TEXT,
                external_ids TEXT,
                user_rating REAL,
                watch_status TEXT CHECK(watch_status IN ('unwatched', 'in_progress', 'watched')),
                last_watched_at INTEGER,
                custom_notes TEXT,
                metadata_source TEXT,
                metadata_fetched_at INTEGER,
                user_edited_fields TEXT,
                alternative_titles TEXT,
                custom_poster BLOB,
                custom_backdrop BLOB,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(pool)
        .await
        .context("Failed to create media_items table")?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_media_items_normalized_title ON media_items(normalized_title)")
            .execute(pool).await.context("Failed to create normalized_title index")?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_media_items_year ON media_items(year)")
            .execute(pool).await.context("Failed to create year index")?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_media_items_watch_status ON media_items(watch_status)")
            .execute(pool).await.context("Failed to create watch_status index")?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_media_items_user_rating ON media_items(user_rating)")
            .execute(pool).await.context("Failed to create user_rating index")?;

        // FileVersions table: physical files on disk
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS file_versions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                media_item_id INTEGER NOT NULL,
                library_root_id INTEGER NOT NULL,
                relative_path TEXT NOT NULL,
                absolute_path TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                status TEXT NOT NULL CHECK(status IN ('present', 'missing', 'trashed', 'corrupted')),
                is_preferred BOOLEAN NOT NULL DEFAULT 0,
                container TEXT,
                resolution_width INTEGER,
                resolution_height INTEGER,
                video_codec TEXT,
                audio_codec TEXT,
                audio_tracks TEXT,
                subtitle_tracks TEXT,
                duration INTEGER,
                bitrate INTEGER,
                framerate REAL,
                quality_label TEXT,
                release_group TEXT,
                source_type TEXT,
                fast_hash BLOB,
                full_hash BLOB,
                hash_algorithm TEXT,
                hashing_status TEXT CHECK(hashing_status IN ('pending', 'in_progress', 'complete', 'failed')),
                hashed_at INTEGER,
                checksum BLOB,
                checksum_algorithm TEXT,
                last_verified_at INTEGER,
                corruption_detected_at INTEGER,
                is_symlink BOOLEAN NOT NULL DEFAULT 0,
                symlink_target TEXT,
                is_hardlink BOOLEAN NOT NULL DEFAULT 0,
                inode INTEGER,
                added_at INTEGER NOT NULL,
                last_seen_at INTEGER NOT NULL,
                trashed_at INTEGER,
                original_path TEXT,
                import_source TEXT,
                import_profile_id INTEGER,
                FOREIGN KEY (media_item_id) REFERENCES media_items(id) ON DELETE CASCADE,
                FOREIGN KEY (library_root_id) REFERENCES library_roots(id),
                FOREIGN KEY (import_profile_id) REFERENCES import_profiles(id)
            )
            "#,
        )
        .execute(pool)
        .await
        .context("Failed to create file_versions table")?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_file_versions_media_item ON file_versions(media_item_id)")
            .execute(pool).await.context("Failed to create media_item index")?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_file_versions_status ON file_versions(status)")
            .execute(pool).await.context("Failed to create status index")?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_file_versions_fast_hash ON file_versions(fast_hash)")
            .execute(pool).await.context("Failed to create fast_hash index")?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_file_versions_full_hash ON file_versions(full_hash)")
            .execute(pool).await.context("Failed to create full_hash index")?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_file_versions_quality_label ON file_versions(quality_label)")
            .execute(pool).await.context("Failed to create quality_label index")?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_file_versions_resolution ON file_versions(resolution_width, resolution_height)")
            .execute(pool).await.context("Failed to create resolution index")?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_file_versions_inode ON file_versions(inode)")
            .execute(pool).await.context("Failed to create inode index")?;

        // LibraryRoots table: storage locations
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS library_roots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                is_active BOOLEAN NOT NULL DEFAULT 1,
                is_archive BOOLEAN NOT NULL DEFAULT 0,
                filesystem_type TEXT,
                mount_point TEXT,
                total_space INTEGER,
                free_space INTEGER,
                last_scanned_at INTEGER,
                created_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(pool)
        .await
        .context("Failed to create library_roots table")?;

        // Tags table: user-defined tags
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                color TEXT,
                created_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(pool)
        .await
        .context("Failed to create tags table")?;

        // MediaItemTags junction table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS media_item_tags (
                media_item_id INTEGER NOT NULL,
                tag_id INTEGER NOT NULL,
                PRIMARY KEY (media_item_id, tag_id),
                FOREIGN KEY (media_item_id) REFERENCES media_items(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(pool)
        .await
        .context("Failed to create media_item_tags table")?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_media_item_tags_tag ON media_item_tags(tag_id)")
            .execute(pool).await.context("Failed to create media_item_tags tag index")?;

        // Collections table: user-defined groupings
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS collections (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                description TEXT,
                is_smart BOOLEAN NOT NULL DEFAULT 0,
                filter_criteria TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(pool)
        .await
        .context("Failed to create collections table")?;

        // CollectionItems junction table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS collection_items (
                collection_id INTEGER NOT NULL,
                media_item_id INTEGER NOT NULL,
                added_at INTEGER NOT NULL,
                PRIMARY KEY (collection_id, media_item_id),
                FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE CASCADE,
                FOREIGN KEY (media_item_id) REFERENCES media_items(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(pool)
        .await
        .context("Failed to create collection_items table")?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_collection_items_media ON collection_items(media_item_id)")
            .execute(pool).await.context("Failed to create collection_items media index")?;

        // WatchFolders table: monitored directories
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS watch_folders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT NOT NULL UNIQUE,
                is_active BOOLEAN NOT NULL DEFAULT 1,
                recursive BOOLEAN NOT NULL DEFAULT 1,
                import_profile_id INTEGER,
                last_scan_at INTEGER,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (import_profile_id) REFERENCES import_profiles(id)
            )
            "#,
        )
        .execute(pool)
        .await
        .context("Failed to create watch_folders table")?;

        // ImportProfiles table: automation rules
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS import_profiles (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                description TEXT,
                priority INTEGER NOT NULL DEFAULT 0,
                file_pattern TEXT,
                min_resolution_width INTEGER,
                min_resolution_height INTEGER,
                allowed_codecs TEXT,
                auto_tags TEXT,
                target_collection_id INTEGER,
                target_library_root_id INTEGER,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (target_collection_id) REFERENCES collections(id),
                FOREIGN KEY (target_library_root_id) REFERENCES library_roots(id)
            )
            "#,
        )
        .execute(pool)
        .await
        .context("Failed to create import_profiles table")?;

        // PlaybackHistory table: watch tracking
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS playback_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_version_id INTEGER NOT NULL,
                started_at INTEGER NOT NULL,
                stopped_at INTEGER,
                playback_position INTEGER,
                duration INTEGER,
                completed BOOLEAN NOT NULL DEFAULT 0,
                FOREIGN KEY (file_version_id) REFERENCES file_versions(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(pool)
        .await
        .context("Failed to create playback_history table")?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_playback_history_version ON playback_history(file_version_id)")
            .execute(pool).await.context("Failed to create playback_history version index")?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_playback_history_started ON playback_history(started_at)")
            .execute(pool).await.context("Failed to create playback_history started index")?;

        // AuditLog table: operation tracking
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS audit_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                operation_type TEXT NOT NULL,
                entity_type TEXT NOT NULL,
                entity_id INTEGER,
                old_value TEXT,
                new_value TEXT,
                initiator TEXT NOT NULL,
                error_message TEXT,
                timestamp INTEGER NOT NULL
            )
            "#,
        )
        .execute(pool)
        .await
        .context("Failed to create audit_log table")?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_log_timestamp ON audit_log(timestamp)")
            .execute(pool).await.context("Failed to create audit_log timestamp index")?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_log_entity ON audit_log(entity_type, entity_id)")
            .execute(pool).await.context("Failed to create audit_log entity index")?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_log_operation ON audit_log(operation_type)")
            .execute(pool).await.context("Failed to create audit_log operation index")?;

        // Notifications table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS notifications (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                severity TEXT NOT NULL CHECK(severity IN ('info', 'warning', 'error', 'critical')),
                title TEXT NOT NULL,
                message TEXT NOT NULL,
                read BOOLEAN NOT NULL DEFAULT 0,
                dismissed BOOLEAN NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL,
                read_at INTEGER,
                dismissed_at INTEGER
            )
            "#,
        )
        .execute(pool)
        .await
        .context("Failed to create notifications table")?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_notifications_read ON notifications(read)")
            .execute(pool).await.context("Failed to create notifications read index")?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_notifications_created ON notifications(created_at)")
            .execute(pool).await.context("Failed to create notifications created index")?;

        // StagedFiles table: import staging
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS staged_files (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                original_path TEXT NOT NULL,
                staged_path TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                status TEXT NOT NULL CHECK(status IN ('pending', 'probing', 'ready', 'committing', 'failed')),
                error_message TEXT,
                technical_metadata TEXT,
                fast_hash BLOB,
                matched_profile_id INTEGER,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (matched_profile_id) REFERENCES import_profiles(id)
            )
            "#,
        )
        .execute(pool)
        .await
        .context("Failed to create staged_files table")?;

        // SavedSearches table: saved search queries
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS saved_searches (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                description TEXT,
                query TEXT NOT NULL,
                filters TEXT NOT NULL,
                search_options TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                last_used_at INTEGER
            )
            "#,
        )
        .execute(pool)
        .await
        .context("Failed to create saved_searches table")?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_saved_searches_name ON saved_searches(name)")
            .execute(pool).await.context("Failed to create saved_searches name index")?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_saved_searches_last_used ON saved_searches(last_used_at)")
            .execute(pool).await.context("Failed to create saved_searches last_used index")?;

        Ok(())
    }

    /// Get current schema version
    pub async fn get_version(pool: &SqlitePool) -> Result<u32> {
        let version: i64 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version"
        )
        .fetch_one(pool)
        .await
        .context("Failed to fetch schema version")?;

        Ok(version as u32)
    }
}
