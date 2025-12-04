// Import Pipeline Component
// Handles file staging, metadata probing, and commit operations

use anyhow::{Result, Context, bail};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use sqlx::{SqlitePool, Row};
use tracing::{info, warn, error, debug};
use chrono::{DateTime, Utc};
use thiserror::Error;
use uuid;

use super::models::*;
use super::metadata_prober::{MetadataProber, TechnicalMetadata};
use crate::filesystem::hasher::{FileHasher, FastHash};

// ============================================================================
// Error Types
// ============================================================================

/// Import pipeline errors
#[derive(Error, Debug)]
pub enum ImportError {
    #[error("Staged file not found: {0}")]
    StagedFileNotFound(StagedFileId),

    #[error("Library root not found: {0}")]
    LibraryRootNotFound(LibraryRootId),

    #[error("Invalid file path: {0}")]
    InvalidFilePath(String),

    #[error("File does not exist: {0}")]
    FileNotFound(String),

    #[error("Staging directory not configured")]
    StagingDirectoryNotConfigured,

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Filesystem error: {0}")]
    FilesystemError(String),

    #[error("Operation failed: {0}")]
    OperationFailed(String),
}

// ============================================================================
// ImportPipeline
// ============================================================================

/// Main import pipeline component
pub struct ImportPipeline {
    /// Database connection pool
    pool: Arc<SqlitePool>,
    
    /// Staging directory path
    staging_dir: PathBuf,
    
    /// Library roots (loaded from database)
    library_roots: Vec<LibraryRoot>,
    
    /// Metadata prober for extracting technical metadata
    metadata_prober: MetadataProber,
    
    /// File hasher for computing fingerprints
    file_hasher: FileHasher,
}

impl ImportPipeline {
    /// Create a new ImportPipeline instance
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `staging_dir` - Path to staging directory
    ///
    /// # Requirements
    /// - Requirements: 2.1, 2.2, 2.3, 2.4
    pub async fn new(pool: Arc<SqlitePool>, staging_dir: PathBuf) -> Result<Self> {
        info!("Initializing ImportPipeline with staging directory: {:?}", staging_dir);
        
        // Ensure staging directory exists
        if !staging_dir.exists() {
            info!("Creating staging directory: {:?}", staging_dir);
            std::fs::create_dir_all(&staging_dir)
                .with_context(|| format!("Failed to create staging directory: {:?}", staging_dir))?;
        }
        
        // Verify staging directory is writable
        if !staging_dir.is_dir() {
            bail!("Staging path is not a directory: {:?}", staging_dir);
        }
        
        // Load library roots from database
        let library_roots = Self::load_library_roots(&pool).await?;
        info!("Loaded {} library roots", library_roots.len());
        
        // Initialize metadata prober
        let metadata_prober = MetadataProber::new(None);
        
        // Initialize file hasher
        let file_hasher = FileHasher::new();
        
        Ok(Self {
            pool,
            staging_dir,
            library_roots,
            metadata_prober,
            file_hasher,
        })
    }
    
    /// Load library roots from database
    async fn load_library_roots(pool: &SqlitePool) -> Result<Vec<LibraryRoot>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                id,
                path,
                name,
                is_active,
                is_archive,
                filesystem_type,
                mount_point,
                total_space,
                free_space,
                last_scanned_at,
                created_at
            FROM library_roots
            WHERE is_active = 1
            ORDER BY id
            "#
        )
        .fetch_all(pool)
        .await
        .context("Failed to load library roots")?;
        
        let mut roots = Vec::new();
        for row in rows {
            let root = LibraryRoot {
                id: row.get("id"),
                path: PathBuf::from(row.get::<String, _>("path")),
                name: row.get("name"),
                is_active: row.get("is_active"),
                is_archive: row.get("is_archive"),
                filesystem_type: row.get("filesystem_type"),
                mount_point: row.get::<Option<String>, _>("mount_point").map(PathBuf::from),
                total_space: row.get::<Option<i64>, _>("total_space").map(|s| s as u64),
                free_space: row.get::<Option<i64>, _>("free_space").map(|s| s as u64),
                last_scanned_at: row.get::<Option<String>, _>("last_scanned_at").and_then(|s| {
                    chrono::DateTime::parse_from_rfc3339(&s)
                        .ok()
                        .map(|dt| dt.with_timezone(&Utc))
                }),
                created_at: row.get::<String, _>("created_at")
                    .parse::<chrono::DateTime<Utc>>()
                    .unwrap_or_else(|_| Utc::now()),
            };
            roots.push(root);
        }
        
        Ok(roots)
    }
    
    /// Get the staging directory path
    pub fn staging_dir(&self) -> &Path {
        &self.staging_dir
    }
    
    /// Get library roots
    pub fn library_roots(&self) -> &[LibraryRoot] {
        &self.library_roots
    }
    
    /// Reload library roots from database
    pub async fn reload_library_roots(&mut self) -> Result<()> {
        self.library_roots = Self::load_library_roots(&self.pool).await?;
        info!("Reloaded {} library roots", self.library_roots.len());
        Ok(())
    }
    
    /// Get a library root by ID
    pub fn get_library_root(&self, id: LibraryRootId) -> Option<&LibraryRoot> {
        self.library_roots.iter().find(|r| r.id == id)
    }
    
    // ========================================================================
    // Staging Operations
    // ========================================================================
    
    /// Stage a file for import
    ///
    /// Copies or moves the file to the staging directory and creates a database record.
    /// Generates a unique staging filename to avoid conflicts.
    ///
    /// # Arguments
    /// * `source` - Path to the source file
    /// * `move_file` - If true, move the file; if false, copy it
    ///
    /// # Returns
    /// The staged file record
    ///
    /// # Requirements
    /// - Requirements: 2.1
    ///
    /// # Errors
    /// Returns an error if:
    /// - Source file does not exist
    /// - File cannot be read
    /// - Staging directory is not writable
    /// - Database operation fails
    pub async fn stage_file(&self, source: &Path, move_file: bool) -> Result<StagedFile> {
        info!("Staging file: {:?} (move: {})", source, move_file);
        
        // Validate source file exists
        if !source.exists() {
            return Err(ImportError::FileNotFound(source.display().to_string()).into());
        }
        
        if !source.is_file() {
            return Err(ImportError::InvalidFilePath(
                format!("Path is not a file: {}", source.display())
            ).into());
        }
        
        // Get file size
        let metadata = std::fs::metadata(source)
            .with_context(|| format!("Failed to read file metadata: {}", source.display()))?;
        let file_size = metadata.len();
        
        // Generate unique staging filename
        let staged_filename = self.generate_staging_filename(source)?;
        let staged_path = self.staging_dir.join(&staged_filename);
        
        // Ensure no collision (should be extremely rare with UUID)
        if staged_path.exists() {
            warn!("Staging filename collision detected, regenerating: {:?}", staged_path);
            let staged_filename = self.generate_staging_filename(source)?;
            let staged_path = self.staging_dir.join(&staged_filename);
            
            if staged_path.exists() {
                return Err(ImportError::OperationFailed(
                    format!("Failed to generate unique staging filename after retry")
                ).into());
            }
        }
        
        // Copy or move file to staging
        if move_file {
            debug!("Moving file to staging: {:?} -> {:?}", source, staged_path);
            if let Err(e) = std::fs::rename(source, &staged_path) {
                // If rename fails (cross-filesystem), fall back to copy + delete
                debug!("Rename failed ({}), falling back to copy + delete", e);
                std::fs::copy(source, &staged_path)
                    .with_context(|| format!("Failed to copy file to staging: {}", source.display()))?;
                std::fs::remove_file(source)
                    .with_context(|| format!("Failed to remove source file: {}", source.display()))?;
            }
        } else {
            debug!("Copying file to staging: {:?} -> {:?}", source, staged_path);
            std::fs::copy(source, &staged_path)
                .with_context(|| format!("Failed to copy file to staging: {}", source.display()))?;
        }
        
        // Create database record
        let now = Utc::now();
        let staged_file = self.create_staged_file_record(
            source,
            &staged_path,
            file_size,
            now,
        ).await?;
        
        info!("File staged successfully: {:?} (ID: {})", staged_path, staged_file.id);
        Ok(staged_file)
    }
    
    /// Generate a unique staging filename
    fn generate_staging_filename(&self, source: &Path) -> Result<String> {
        // Use UUID to ensure uniqueness
        let uuid = uuid::Uuid::new_v4();
        
        // Preserve original extension if present
        let extension = source.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        
        let filename = if extension.is_empty() {
            format!("staged_{}", uuid)
        } else {
            format!("staged_{}.{}", uuid, extension)
        };
        
        Ok(filename)
    }
    
    /// Create a staged file database record
    async fn create_staged_file_record(
        &self,
        original_path: &Path,
        staged_path: &Path,
        file_size: u64,
        created_at: DateTime<Utc>,
    ) -> Result<StagedFile> {
        let original_path_str = original_path.display().to_string();
        let staged_path_str = staged_path.display().to_string();
        let created_at_str = created_at.to_rfc3339();
        let status = StagingStatus::Pending.as_str();
        
        let result = sqlx::query(
            r#"
            INSERT INTO staged_files (
                original_path,
                staged_path,
                file_size,
                status,
                created_at
            ) VALUES (?, ?, ?, ?, ?)
            "#
        )
        .bind(&original_path_str)
        .bind(&staged_path_str)
        .bind(file_size as i64)
        .bind(status)
        .bind(&created_at_str)
        .execute(&*self.pool)
        .await
        .context("Failed to insert staged file record")?;
        
        let id = result.last_insert_rowid();
        
        Ok(StagedFile {
            id,
            original_path: original_path.to_path_buf(),
            staged_path: staged_path.to_path_buf(),
            file_size,
            status: StagingStatus::Pending,
            error_message: None,
            technical_metadata: None,
            fast_hash: None,
            matched_profile_id: None,
            created_at,
        })
    }
    
    /// Get a staged file by ID
    pub async fn get_staged_file(&self, id: StagedFileId) -> Result<StagedFile> {
        let row = sqlx::query(
            r#"
            SELECT 
                id,
                original_path,
                staged_path,
                file_size,
                status,
                error_message,
                technical_metadata,
                fast_hash,
                matched_profile_id,
                created_at
            FROM staged_files
            WHERE id = ?
            "#
        )
        .bind(id)
        .fetch_optional(&*self.pool)
        .await
        .context("Failed to query staged file")?
        .ok_or_else(|| ImportError::StagedFileNotFound(id))?;
        
        Ok(Self::row_to_staged_file(row)?)
    }
    
    /// Convert a database row to a StagedFile
    fn row_to_staged_file(row: sqlx::sqlite::SqliteRow) -> Result<StagedFile> {
        let status_str: String = row.get("status");
        let status = match status_str.as_str() {
            "pending" => StagingStatus::Pending,
            "probing" => StagingStatus::Probing,
            "ready" => StagingStatus::Ready,
            "committing" => StagingStatus::Committing,
            "failed" => StagingStatus::Failed,
            _ => StagingStatus::Pending,
        };
        
        let created_at_str: String = row.get("created_at");
        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
            .ok()
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);
        
        Ok(StagedFile {
            id: row.get("id"),
            original_path: PathBuf::from(row.get::<String, _>("original_path")),
            staged_path: PathBuf::from(row.get::<String, _>("staged_path")),
            file_size: row.get::<i64, _>("file_size") as u64,
            status,
            error_message: row.get("error_message"),
            technical_metadata: row.get("technical_metadata"),
            fast_hash: row.get("fast_hash"),
            matched_profile_id: row.get("matched_profile_id"),
            created_at,
        })
    }
    
    // ========================================================================
    // Metadata Probing Operations
    // ========================================================================
    
    /// Probe a staged file for technical metadata
    ///
    /// Calls MetadataProber to extract video, audio, and subtitle information,
    /// stores the metadata in the staged_files table, and updates status to "ready".
    ///
    /// # Arguments
    /// * `staged_id` - ID of the staged file to probe
    ///
    /// # Returns
    /// The extracted technical metadata
    ///
    /// # Requirements
    /// - Requirements: 2.2
    ///
    /// # Errors
    /// Returns an error if:
    /// - Staged file not found
    /// - File does not exist on disk
    /// - ffprobe fails to parse the file
    /// - Database update fails
    pub async fn probe_staged_file(&self, staged_id: StagedFileId) -> Result<TechnicalMetadata> {
        info!("Probing staged file: {}", staged_id);
        
        // Get staged file record
        let staged_file = self.get_staged_file(staged_id).await?;
        
        // Verify file exists
        if !staged_file.staged_path.exists() {
            return Err(ImportError::FileNotFound(
                staged_file.staged_path.display().to_string()
            ).into());
        }
        
        // Update status to "probing"
        self.update_staged_file_status(staged_id, StagingStatus::Probing, None).await?;
        
        // Probe the file
        let metadata = match self.metadata_prober.probe_file(&staged_file.staged_path).await {
            Ok(metadata) => metadata,
            Err(e) => {
                // Update status to "failed" with error message
                let error_msg = format!("Metadata probing failed: {}", e);
                warn!("{}", error_msg);
                self.update_staged_file_status(
                    staged_id,
                    StagingStatus::Failed,
                    Some(error_msg)
                ).await?;
                return Err(e);
            }
        };
        
        // Serialize metadata to JSON
        let metadata_json = serde_json::to_string(&metadata)
            .context("Failed to serialize metadata to JSON")?;
        
        // Update database with metadata and status "ready"
        sqlx::query(
            r#"
            UPDATE staged_files
            SET 
                status = ?,
                technical_metadata = ?
            WHERE id = ?
            "#
        )
        .bind(StagingStatus::Ready.as_str())
        .bind(&metadata_json)
        .bind(staged_id)
        .execute(&*self.pool)
        .await
        .context("Failed to update staged file with metadata")?;
        
        info!("Staged file probed successfully: {} (status: ready)", staged_id);
        Ok(metadata)
    }
    
    /// Update staged file status
    async fn update_staged_file_status(
        &self,
        staged_id: StagedFileId,
        status: StagingStatus,
        error_message: Option<String>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE staged_files
            SET 
                status = ?,
                error_message = ?
            WHERE id = ?
            "#
        )
        .bind(status.as_str())
        .bind(error_message)
        .bind(staged_id)
        .execute(&*self.pool)
        .await
        .context("Failed to update staged file status")?;
        
        Ok(())
    }
    
    // ========================================================================
    // Fingerprinting Operations
    // ========================================================================
    
    /// Compute fast fingerprint for a staged file
    ///
    /// Calls FileHasher to compute a fast hash (first 1MB + last 1MB + size)
    /// and stores it in the staged_files table.
    ///
    /// # Arguments
    /// * `staged_id` - ID of the staged file to fingerprint
    ///
    /// # Returns
    /// The computed fast hash
    ///
    /// # Requirements
    /// - Requirements: 2.3
    ///
    /// # Errors
    /// Returns an error if:
    /// - Staged file not found
    /// - File does not exist on disk
    /// - Hashing fails
    /// - Database update fails
    pub async fn compute_fast_fingerprint(&self, staged_id: StagedFileId) -> Result<FastHash> {
        info!("Computing fast fingerprint for staged file: {}", staged_id);
        
        // Get staged file record
        let staged_file = self.get_staged_file(staged_id).await?;
        
        // Verify file exists
        if !staged_file.staged_path.exists() {
            return Err(ImportError::FileNotFound(
                staged_file.staged_path.display().to_string()
            ).into());
        }
        
        // Compute fast hash
        let fast_hash = self.file_hasher.compute_fast_hash(&staged_file.staged_path).await
            .with_context(|| format!("Failed to compute fast hash for: {}", staged_file.staged_path.display()))?;
        
        // Store hash in database
        sqlx::query(
            r#"
            UPDATE staged_files
            SET fast_hash = ?
            WHERE id = ?
            "#
        )
        .bind(&fast_hash.value)
        .bind(staged_id)
        .execute(&*self.pool)
        .await
        .context("Failed to update staged file with fast hash")?;
        
        info!("Fast fingerprint computed and stored for staged file: {}", staged_id);
        Ok(fast_hash)
    }
    
    // ========================================================================
    // Commit Operations
    // ========================================================================
    
    /// Commit a staged file to the library
    ///
    /// This operation is atomic - either the file is successfully moved to the library
    /// with all database records created, or the file remains in staging with no changes.
    ///
    /// The commit process:
    /// 1. Begin database transaction
    /// 2. Create or find MediaItem by normalized title/year
    /// 3. Generate target path with consistent naming scheme
    /// 4. Atomically move file to library root (rename within same filesystem)
    /// 5. Create FileVersion record with status "present"
    /// 6. Commit transaction
    /// 7. Remove from staging on success
    ///
    /// # Arguments
    /// * `staged_id` - ID of the staged file to commit
    /// * `target_root_id` - ID of the library root to commit to
    ///
    /// # Returns
    /// The ID of the created FileVersion
    ///
    /// # Requirements
    /// - Requirements: 2.4, 2.6, 2.7
    ///
    /// # Errors
    /// Returns an error if:
    /// - Staged file not found
    /// - Library root not found
    /// - File does not exist on disk
    /// - Database transaction fails
    /// - File move operation fails
    pub async fn commit_staged_file(
        &self,
        staged_id: StagedFileId,
        target_root_id: LibraryRootId,
    ) -> Result<crate::library::models::FileVersionId> {
        use crate::library::models::{MediaItem, MediaType, FileVersion, FileStatus, HashingStatus, FilesystemInfo, WatchStatus, ExternalIds};
        
        info!("Committing staged file {} to library root {}", staged_id, target_root_id);
        
        // Get staged file record
        let staged_file = self.get_staged_file(staged_id).await?;
        
        // Verify file exists
        if !staged_file.staged_path.exists() {
            return Err(ImportError::FileNotFound(
                staged_file.staged_path.display().to_string()
            ).into());
        }
        
        // Get library root
        let library_root = self.get_library_root(target_root_id)
            .ok_or_else(|| ImportError::LibraryRootNotFound(target_root_id))?;
        
        // Verify library root exists and is writable
        if !library_root.path.exists() {
            return Err(ImportError::FilesystemError(
                format!("Library root does not exist: {}", library_root.path.display())
            ).into());
        }
        
        // Update status to "committing"
        self.update_staged_file_status(staged_id, StagingStatus::Committing, None).await?;
        
        // Parse technical metadata if available
        let technical_metadata: Option<crate::import::metadata_prober::TechnicalMetadata> = 
            staged_file.technical_metadata.as_ref()
                .and_then(|json| serde_json::from_str(json).ok());
        
        // Extract title and year from filename (simplified - in production would use guessit)
        let filename = staged_file.original_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown");
        
        // For now, use a simple title extraction (in production, use guessit parser)
        let title = filename.to_string();
        let year: Option<i32> = None; // Would be extracted by guessit parser
        
        // Normalize title (simple lowercase and trim for now)
        let normalized_title = title.to_lowercase().trim().to_string();
        
        // Begin database transaction
        let mut tx = self.pool.begin().await
            .context("Failed to begin transaction")?;
        
        // Try to find existing MediaItem by normalized title and year
        let media_item_id = match self.find_media_item_by_title_year(&mut tx, &normalized_title, year).await? {
            Some(id) => {
                info!("Found existing MediaItem: {}", id);
                id
            }
            None => {
                // Create new MediaItem
                info!("Creating new MediaItem for: {} (normalized: {})", title, normalized_title);
                let media_item = MediaItem {
                    id: crate::library::models::MediaItemId(0), // Will be assigned by database
                    title: title.clone(),
                    normalized_title: normalized_title.clone(),
                    original_title: None,
                    year,
                    media_type: MediaType::Movie, // Default to movie
                    overview: None,
                    genres: Vec::new(),
                    cast: Vec::new(),
                    director: None,
                    runtime: technical_metadata.as_ref().map(|m| m.duration),
                    poster: None,
                    backdrop: None,
                    external_ids: ExternalIds {
                        tmdb_id: None,
                        imdb_id: None,
                        kinopoisk_id: None,
                    },
                    user_rating: None,
                    watch_status: WatchStatus::Unwatched,
                    last_watched_at: None,
                    custom_notes: None,
                    tags: Vec::new(),
                    metadata_source: None,
                    metadata_fetched_at: None,
                    user_edited_fields: std::collections::HashSet::new(),
                    alternative_titles: Vec::new(),
                    custom_poster: None,
                    custom_backdrop: None,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };
                
                self.create_media_item_in_tx(&mut tx, &media_item).await?
            }
        };
        
        // Generate target path with consistent naming scheme
        let target_filename = self.generate_target_filename(&title, year, &staged_file.staged_path)?;
        let relative_path = PathBuf::from(&target_filename);
        let absolute_path = library_root.path.join(&target_filename);
        
        // Ensure target directory exists
        if let Some(parent) = absolute_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create target directory: {}", parent.display()))?;
            }
        }
        
        // Atomically move file to library root
        // Use rename for atomic operation within same filesystem
        if let Err(e) = std::fs::rename(&staged_file.staged_path, &absolute_path) {
            // If rename fails (cross-filesystem), fall back to copy + delete
            warn!("Rename failed ({}), falling back to copy + delete", e);
            std::fs::copy(&staged_file.staged_path, &absolute_path)
                .with_context(|| format!("Failed to copy file to library: {}", absolute_path.display()))?;
            
            // Only delete source after successful copy
            std::fs::remove_file(&staged_file.staged_path)
                .with_context(|| format!("Failed to remove staged file: {}", staged_file.staged_path.display()))?;
        }
        
        info!("File moved to library: {}", absolute_path.display());
        
        // Create FileVersion record
        let file_version = FileVersion {
            id: crate::library::models::FileVersionId(0), // Will be assigned by database
            media_item_id,
            library_root_id: crate::library::models::LibraryRootId(target_root_id),
            relative_path: relative_path.clone(),
            absolute_path: absolute_path.clone(),
            file_size: staged_file.file_size,
            status: FileStatus::Present,
            is_preferred: false,
            technical_metadata: technical_metadata.clone(),
            quality_label: None, // Would be determined by quality scorer
            release_group: None,
            source_type: None,
            fast_hash: staged_file.fast_hash.map(|h| crate::library::models::FastHash(h)),
            full_hash: None,
            hashing_status: HashingStatus::Pending,
            checksum: None,
            last_verified_at: None,
            corruption_detected_at: None,
            filesystem_info: FilesystemInfo {
                is_symlink: false,
                symlink_target: None,
                is_hardlink: false,
                inode: None,
            },
            added_at: Utc::now(),
            last_seen_at: Utc::now(),
            trashed_at: None,
            original_path: Some(staged_file.original_path.clone()),
            import_source: Some("import_pipeline".to_string()),
        };
        
        let file_version_id = self.create_file_version_in_tx(&mut tx, &file_version).await?;
        
        // Commit transaction
        tx.commit().await
            .context("Failed to commit transaction")?;
        
        info!("Transaction committed successfully");
        
        // Remove from staging (best effort - don't fail if this fails)
        if let Err(e) = self.remove_staged_file(staged_id).await {
            warn!("Failed to remove staged file record: {}", e);
        }
        
        info!("Staged file {} committed successfully as FileVersion {}", staged_id, file_version_id);
        Ok(file_version_id)
    }
    
    /// Rollback a staged file commit
    ///
    /// Handles commit failures by keeping the file in staging with error status.
    /// The database transaction is automatically rolled back by the transaction guard.
    ///
    /// # Arguments
    /// * `staged_id` - ID of the staged file to rollback
    /// * `error_message` - Error message describing the failure
    ///
    /// # Requirements
    /// - Requirements: 2.5
    ///
    /// # Errors
    /// Returns an error if database update fails
    pub async fn rollback_staged_file(&self, staged_id: StagedFileId, error_message: String) -> Result<()> {
        warn!("Rolling back staged file {}: {}", staged_id, error_message);
        
        // Update status to "failed" with error message
        self.update_staged_file_status(
            staged_id,
            StagingStatus::Failed,
            Some(error_message.clone())
        ).await?;
        
        info!("Staged file {} rolled back to failed status", staged_id);
        Ok(())
    }
    
    // ========================================================================
    // Helper Methods
    // ========================================================================
    
    /// Find a MediaItem by normalized title and year
    async fn find_media_item_by_title_year(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        normalized_title: &str,
        year: Option<i32>,
    ) -> Result<Option<crate::library::models::MediaItemId>> {
        let row = if let Some(y) = year {
            sqlx::query(
                r#"
                SELECT id
                FROM media_items
                WHERE normalized_title = ? AND year = ?
                LIMIT 1
                "#
            )
            .bind(normalized_title)
            .bind(y)
            .fetch_optional(&mut **tx)
            .await
            .context("Failed to query media item by title and year")?
        } else {
            sqlx::query(
                r#"
                SELECT id
                FROM media_items
                WHERE normalized_title = ? AND year IS NULL
                LIMIT 1
                "#
            )
            .bind(normalized_title)
            .fetch_optional(&mut **tx)
            .await
            .context("Failed to query media item by title")?
        };
        
        Ok(row.map(|r| crate::library::models::MediaItemId(r.get("id"))))
    }
    
    /// Create a MediaItem within a transaction
    async fn create_media_item_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        media_item: &crate::library::models::MediaItem,
    ) -> Result<crate::library::models::MediaItemId> {
        let media_type = match media_item.media_type {
            crate::library::models::MediaType::Movie => "movie",
            crate::library::models::MediaType::Series => "series",
        };
        
        let watch_status = match media_item.watch_status {
            crate::library::models::WatchStatus::Unwatched => "unwatched",
            crate::library::models::WatchStatus::InProgress => "in_progress",
            crate::library::models::WatchStatus::Watched => "watched",
        };
        
        let genres_json = serde_json::to_string(&media_item.genres)
            .context("Failed to serialize genres")?;
        let cast_json = serde_json::to_string(&media_item.cast)
            .context("Failed to serialize cast")?;
        let external_ids_json = serde_json::to_string(&media_item.external_ids)
            .context("Failed to serialize external_ids")?;
        let user_edited_fields_json = serde_json::to_string(&media_item.user_edited_fields)
            .context("Failed to serialize user_edited_fields")?;
        
        let result = sqlx::query(
            r#"
            INSERT INTO media_items (
                title,
                normalized_title,
                original_title,
                year,
                media_type,
                overview,
                genres,
                cast,
                director,
                runtime,
                external_ids,
                user_rating,
                watch_status,
                last_watched_at,
                custom_notes,
                metadata_source,
                metadata_fetched_at,
                user_edited_fields,
                created_at,
                updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&media_item.title)
        .bind(&media_item.normalized_title)
        .bind(&media_item.original_title)
        .bind(media_item.year)
        .bind(media_type)
        .bind(&media_item.overview)
        .bind(&genres_json)
        .bind(&cast_json)
        .bind(&media_item.director)
        .bind(media_item.runtime.map(|r| r as i64))
        .bind(&external_ids_json)
        .bind(media_item.user_rating)
        .bind(watch_status)
        .bind(media_item.last_watched_at.map(|dt| dt.to_rfc3339()))
        .bind(&media_item.custom_notes)
        .bind(&media_item.metadata_source)
        .bind(media_item.metadata_fetched_at.map(|dt| dt.to_rfc3339()))
        .bind(&user_edited_fields_json)
        .bind(media_item.created_at.to_rfc3339())
        .bind(media_item.updated_at.to_rfc3339())
        .execute(&mut **tx)
        .await
        .context("Failed to insert media item")?;
        
        Ok(crate::library::models::MediaItemId(result.last_insert_rowid()))
    }
    
    /// Create a FileVersion within a transaction
    async fn create_file_version_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        file_version: &crate::library::models::FileVersion,
    ) -> Result<crate::library::models::FileVersionId> {
        let status = match file_version.status {
            crate::library::models::FileStatus::Present => "present",
            crate::library::models::FileStatus::Missing => "missing",
            crate::library::models::FileStatus::Trashed => "trashed",
            crate::library::models::FileStatus::Corrupted => "corrupted",
        };
        
        let hashing_status = match file_version.hashing_status {
            crate::library::models::HashingStatus::Pending => "pending",
            crate::library::models::HashingStatus::InProgress => "in_progress",
            crate::library::models::HashingStatus::Complete => "complete",
            crate::library::models::HashingStatus::Failed => "failed",
        };
        
        let audio_tracks_json = file_version.technical_metadata.as_ref()
            .map(|m| serde_json::to_string(&m.audio_tracks))
            .transpose()
            .context("Failed to serialize audio tracks")?;
        
        let subtitle_tracks_json = file_version.technical_metadata.as_ref()
            .map(|m| serde_json::to_string(&m.subtitle_tracks))
            .transpose()
            .context("Failed to serialize subtitle tracks")?;
        
        let result = sqlx::query(
            r#"
            INSERT INTO file_versions (
                media_item_id,
                library_root_id,
                relative_path,
                absolute_path,
                file_size,
                status,
                is_preferred,
                container,
                resolution_width,
                resolution_height,
                video_codec,
                audio_codec,
                audio_tracks,
                subtitle_tracks,
                duration,
                bitrate,
                framerate,
                quality_label,
                release_group,
                source_type,
                fast_hash,
                full_hash,
                hash_algorithm,
                hashing_status,
                is_symlink,
                is_hardlink,
                added_at,
                last_seen_at,
                original_path,
                import_source
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(file_version.media_item_id.0)
        .bind(file_version.library_root_id.0)
        .bind(file_version.relative_path.to_str())
        .bind(file_version.absolute_path.to_str())
        .bind(file_version.file_size as i64)
        .bind(status)
        .bind(file_version.is_preferred)
        .bind(file_version.technical_metadata.as_ref().map(|m| &m.container))
        .bind(file_version.technical_metadata.as_ref().and_then(|m| m.video.as_ref()).map(|v| v.resolution.width as i64))
        .bind(file_version.technical_metadata.as_ref().and_then(|m| m.video.as_ref()).map(|v| v.resolution.height as i64))
        .bind(file_version.technical_metadata.as_ref().and_then(|m| m.video.as_ref()).map(|v| &v.codec))
        .bind(file_version.technical_metadata.as_ref().and_then(|m| m.audio_tracks.first()).map(|a| &a.codec))
        .bind(audio_tracks_json)
        .bind(subtitle_tracks_json)
        .bind(file_version.technical_metadata.as_ref().map(|m| m.duration as i64))
        .bind(file_version.technical_metadata.as_ref().map(|m| m.bitrate as i64))
        .bind(file_version.technical_metadata.as_ref().and_then(|m| m.video.as_ref()).map(|v| v.framerate as f64))
        .bind(&file_version.quality_label)
        .bind(&file_version.release_group)
        .bind(file_version.source_type.as_ref().map(|s| match s {
            crate::library::models::SourceType::BluRay => "BluRay",
            crate::library::models::SourceType::WebDl => "WEB-DL",
            crate::library::models::SourceType::Hdtv => "HDTV",
            crate::library::models::SourceType::Dvd => "DVD",
            crate::library::models::SourceType::Unknown => "Unknown",
        }))
        .bind(file_version.fast_hash.as_ref().map(|h| &h.0))
        .bind(file_version.full_hash.as_ref().map(|h| &h.value))
        .bind(file_version.full_hash.as_ref().map(|h| &h.algorithm))
        .bind(hashing_status)
        .bind(file_version.filesystem_info.is_symlink)
        .bind(file_version.filesystem_info.is_hardlink)
        .bind(file_version.added_at.to_rfc3339())
        .bind(file_version.last_seen_at.to_rfc3339())
        .bind(file_version.original_path.as_ref().and_then(|p| p.to_str()))
        .bind(&file_version.import_source)
        .execute(&mut **tx)
        .await
        .context("Failed to insert file version")?;
        
        Ok(crate::library::models::FileVersionId(result.last_insert_rowid()))
    }
    
    /// Generate a target filename with consistent naming scheme
    fn generate_target_filename(&self, title: &str, year: Option<i32>, source_path: &Path) -> Result<String> {
        // Preserve original extension
        let extension = source_path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("mkv");
        
        // Generate filename: Title (Year).ext or Title.ext
        let filename = if let Some(y) = year {
            format!("{} ({}).{}", title, y, extension)
        } else {
            format!("{}.{}", title, extension)
        };
        
        // Sanitize filename (remove invalid characters)
        let sanitized = filename
            .replace('/', "_")
            .replace('\\', "_")
            .replace(':', "_")
            .replace('*', "_")
            .replace('?', "_")
            .replace('"', "_")
            .replace('<', "_")
            .replace('>', "_")
            .replace('|', "_");
        
        Ok(sanitized)
    }
    
    /// Remove a staged file record from the database
    async fn remove_staged_file(&self, staged_id: StagedFileId) -> Result<()> {
        sqlx::query(
            r#"
            DELETE FROM staged_files
            WHERE id = ?
            "#
        )
        .bind(staged_id)
        .execute(&*self.pool)
        .await
        .context("Failed to delete staged file record")?;
        
        Ok(())
    }
    
    // ========================================================================
    // Import Profile Operations
    // ========================================================================
    
    /// Create a new import profile
    ///
    /// Stores profile configuration in the database with file patterns,
    /// quality filters, auto-tags, and target collections.
    ///
    /// # Arguments
    /// * `profile` - Import profile configuration
    ///
    /// # Returns
    /// The ID of the created profile
    ///
    /// # Requirements
    /// - Requirements: 28.1, 28.2, 28.3, 28.4, 28.5
    ///
    /// # Errors
    /// Returns an error if database operation fails
    pub async fn create_import_profile(&self, profile: &super::models::ImportProfile) -> Result<ImportProfileId> {
        info!("Creating import profile: {}", profile.name);
        
        // Serialize quality filters and auto-tags to JSON
        let allowed_codecs_json = profile.quality_filters.allowed_codecs.as_ref()
            .map(|codecs| serde_json::to_string(codecs))
            .transpose()
            .context("Failed to serialize allowed codecs")?;
        
        let auto_tags_json = serde_json::to_string(&profile.auto_tags)
            .context("Failed to serialize auto tags")?;
        
        let result = sqlx::query(
            r#"
            INSERT INTO import_profiles (
                name,
                description,
                priority,
                file_pattern,
                min_resolution_width,
                min_resolution_height,
                allowed_codecs,
                auto_tags,
                target_collection_id,
                target_library_root_id,
                created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&profile.name)
        .bind(&profile.description)
        .bind(profile.priority)
        .bind(&profile.file_pattern)
        .bind(profile.quality_filters.min_resolution_width)
        .bind(profile.quality_filters.min_resolution_height)
        .bind(allowed_codecs_json)
        .bind(&auto_tags_json)
        .bind(profile.target_collection_id)
        .bind(profile.target_library_root_id)
        .bind(profile.created_at.to_rfc3339())
        .execute(&*self.pool)
        .await
        .context("Failed to insert import profile")?;
        
        let id = result.last_insert_rowid();
        info!("Import profile created with ID: {}", id);
        Ok(id)
    }
    
    /// Get an import profile by ID
    ///
    /// # Arguments
    /// * `id` - Import profile ID
    ///
    /// # Returns
    /// The import profile if found
    ///
    /// # Errors
    /// Returns an error if profile not found or database operation fails
    pub async fn get_import_profile(&self, id: ImportProfileId) -> Result<super::models::ImportProfile> {
        let row = sqlx::query(
            r#"
            SELECT 
                id,
                name,
                description,
                priority,
                file_pattern,
                min_resolution_width,
                min_resolution_height,
                allowed_codecs,
                auto_tags,
                target_collection_id,
                target_library_root_id,
                created_at
            FROM import_profiles
            WHERE id = ?
            "#
        )
        .bind(id)
        .fetch_optional(&*self.pool)
        .await
        .context("Failed to query import profile")?
        .ok_or_else(|| ImportError::OperationFailed(format!("Import profile not found: {}", id)))?;
        
        Self::row_to_import_profile(row)
    }
    
    /// Get all import profiles ordered by priority (highest first)
    ///
    /// # Returns
    /// List of all import profiles
    ///
    /// # Errors
    /// Returns an error if database operation fails
    pub async fn get_all_import_profiles(&self) -> Result<Vec<super::models::ImportProfile>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                id,
                name,
                description,
                priority,
                file_pattern,
                min_resolution_width,
                min_resolution_height,
                allowed_codecs,
                auto_tags,
                target_collection_id,
                target_library_root_id,
                created_at
            FROM import_profiles
            ORDER BY priority DESC, id ASC
            "#
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to query import profiles")?;
        
        let mut profiles = Vec::new();
        for row in rows {
            profiles.push(Self::row_to_import_profile(row)?);
        }
        
        Ok(profiles)
    }
    
    /// Convert a database row to an ImportProfile
    fn row_to_import_profile(row: sqlx::sqlite::SqliteRow) -> Result<super::models::ImportProfile> {
        use super::models::{ImportProfile, QualityFilters};
        
        let allowed_codecs: Option<Vec<String>> = row.get::<Option<String>, _>("allowed_codecs")
            .map(|json| serde_json::from_str(&json))
            .transpose()
            .context("Failed to deserialize allowed codecs")?;
        
        let auto_tags: Vec<String> = row.get::<String, _>("auto_tags")
            .parse::<String>()
            .ok()
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default();
        
        let created_at_str: String = row.get("created_at");
        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
            .ok()
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);
        
        Ok(ImportProfile {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
            priority: row.get("priority"),
            file_pattern: row.get("file_pattern"),
            quality_filters: QualityFilters {
                min_resolution_width: row.get("min_resolution_width"),
                min_resolution_height: row.get("min_resolution_height"),
                allowed_codecs,
                min_bitrate: None, // Not stored in database yet
            },
            auto_tags,
            target_collection_id: row.get("target_collection_id"),
            target_library_root_id: row.get("target_library_root_id"),
            created_at,
        })
    }
    
    /// Update an import profile
    ///
    /// # Arguments
    /// * `id` - Import profile ID
    /// * `profile` - Updated profile configuration
    ///
    /// # Errors
    /// Returns an error if profile not found or database operation fails
    pub async fn update_import_profile(&self, id: ImportProfileId, profile: &super::models::ImportProfile) -> Result<()> {
        info!("Updating import profile: {}", id);
        
        // Serialize quality filters and auto-tags to JSON
        let allowed_codecs_json = profile.quality_filters.allowed_codecs.as_ref()
            .map(|codecs| serde_json::to_string(codecs))
            .transpose()
            .context("Failed to serialize allowed codecs")?;
        
        let auto_tags_json = serde_json::to_string(&profile.auto_tags)
            .context("Failed to serialize auto tags")?;
        
        sqlx::query(
            r#"
            UPDATE import_profiles
            SET 
                name = ?,
                description = ?,
                priority = ?,
                file_pattern = ?,
                min_resolution_width = ?,
                min_resolution_height = ?,
                allowed_codecs = ?,
                auto_tags = ?,
                target_collection_id = ?,
                target_library_root_id = ?
            WHERE id = ?
            "#
        )
        .bind(&profile.name)
        .bind(&profile.description)
        .bind(profile.priority)
        .bind(&profile.file_pattern)
        .bind(profile.quality_filters.min_resolution_width)
        .bind(profile.quality_filters.min_resolution_height)
        .bind(allowed_codecs_json)
        .bind(&auto_tags_json)
        .bind(profile.target_collection_id)
        .bind(profile.target_library_root_id)
        .bind(id)
        .execute(&*self.pool)
        .await
        .context("Failed to update import profile")?;
        
        info!("Import profile updated: {}", id);
        Ok(())
    }
    
    /// Delete an import profile
    ///
    /// # Arguments
    /// * `id` - Import profile ID
    ///
    /// # Errors
    /// Returns an error if database operation fails
    pub async fn delete_import_profile(&self, id: ImportProfileId) -> Result<()> {
        info!("Deleting import profile: {}", id);
        
        sqlx::query(
            r#"
            DELETE FROM import_profiles
            WHERE id = ?
            "#
        )
        .bind(id)
        .execute(&*self.pool)
        .await
        .context("Failed to delete import profile")?;
        
        info!("Import profile deleted: {}", id);
        Ok(())
    }
    
    /// Match a file path against import profiles
    ///
    /// Matches the file path against all import profiles' patterns and returns
    /// the best matching profile based on priority and pattern specificity.
    ///
    /// # Arguments
    /// * `file_path` - Path to the file to match
    ///
    /// # Returns
    /// The matched import profile, or None if no profile matches
    ///
    /// # Requirements
    /// - Requirements: 28.2, 28.6
    ///
    /// # Errors
    /// Returns an error if database operation fails
    pub async fn match_import_profile(&self, file_path: &Path) -> Result<Option<super::models::ImportProfile>> {
        debug!("Matching import profile for file: {:?}", file_path);
        
        // Get all profiles ordered by priority (highest first)
        let profiles = self.get_all_import_profiles().await?;
        
        // Convert file path to string for pattern matching
        let file_path_str = file_path.to_string_lossy();
        let file_name = file_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        
        // Try to match each profile in priority order
        for profile in profiles {
            if let Some(ref pattern) = profile.file_pattern {
                // Try glob pattern matching first
                if Self::matches_glob_pattern(&file_path_str, &file_name, pattern) {
                    info!("File matched import profile '{}' (ID: {}) with pattern: {}", 
                          profile.name, profile.id, pattern);
                    return Ok(Some(profile));
                }
                
                // Try regex pattern matching if glob didn't match
                if Self::matches_regex_pattern(&file_path_str, pattern) {
                    info!("File matched import profile '{}' (ID: {}) with regex: {}", 
                          profile.name, profile.id, pattern);
                    return Ok(Some(profile));
                }
            } else {
                // Profile with no pattern matches all files
                info!("File matched import profile '{}' (ID: {}) with no pattern (matches all)", 
                      profile.name, profile.id);
                return Ok(Some(profile));
            }
        }
        
        debug!("No import profile matched for file: {:?}", file_path);
        Ok(None)
    }
    
    /// Check if a file path matches a glob pattern
    fn matches_glob_pattern(file_path: &str, file_name: &str, pattern: &str) -> bool {
        // Simple glob pattern matching
        // Supports: * (any characters), ? (single character), ** (recursive)
        
        // Try matching against full path
        if Self::glob_match(file_path, pattern) {
            return true;
        }
        
        // Try matching against just the filename
        if Self::glob_match(file_name, pattern) {
            return true;
        }
        
        false
    }
    
    /// Simple glob pattern matching
    fn glob_match(text: &str, pattern: &str) -> bool {
        // Handle ** for recursive matching
        if pattern.contains("**") {
            // Convert ** to .* for regex matching
            let regex_pattern = pattern
                .replace(".", "\\.")
                .replace("**", ".*")
                .replace("*", "[^/]*")
                .replace("?", ".");
            
            if let Ok(re) = regex::Regex::new(&format!("^{}$", regex_pattern)) {
                return re.is_match(text);
            }
        }
        
        // Simple glob matching without **
        let regex_pattern = pattern
            .replace(".", "\\.")
            .replace("*", ".*")
            .replace("?", ".");
        
        if let Ok(re) = regex::Regex::new(&format!("^{}$", regex_pattern)) {
            return re.is_match(text);
        }
        
        false
    }
    
    /// Check if a file path matches a regex pattern
    fn matches_regex_pattern(file_path: &str, pattern: &str) -> bool {
        // Try to compile and match as regex
        // Regex patterns typically start with ^ or contain regex-specific syntax
        if pattern.starts_with('^') || pattern.contains('(') || pattern.contains('[') {
            if let Ok(re) = regex::Regex::new(pattern) {
                return re.is_match(file_path);
            }
        }
        
        false
    }
    
    /// Apply import rules from a profile to a staged file
    ///
    /// Evaluates quality filters and determines actions to take:
    /// - Auto-tags to apply
    /// - Target collection assignment
    /// - Target library root
    /// - Whether to reject the file based on quality
    ///
    /// # Arguments
    /// * `staged_file` - The staged file to evaluate
    /// * `profile` - The import profile to apply
    ///
    /// # Returns
    /// ImportActions describing what actions to take
    ///
    /// # Requirements
    /// - Requirements: 28.3, 28.4, 28.5
    ///
    /// # Errors
    /// Returns an error if metadata cannot be parsed
    pub async fn apply_import_rules(
        &self,
        staged_file: &StagedFile,
        profile: &super::models::ImportProfile,
    ) -> Result<super::models::ImportActions> {
        use super::models::ImportActions;
        
        info!("Applying import rules from profile '{}' to staged file {}", profile.name, staged_file.id);
        
        // Parse technical metadata if available
        let technical_metadata: Option<TechnicalMetadata> = staged_file.technical_metadata.as_ref()
            .and_then(|json| serde_json::from_str(json).ok());
        
        // Check quality filters
        let (should_reject, rejection_reason) = self.check_quality_filters(
            &technical_metadata,
            &profile.quality_filters,
        );
        
        if should_reject {
            info!("File rejected by quality filters: {}", rejection_reason.as_ref().unwrap());
            return Ok(ImportActions {
                tags_to_apply: Vec::new(),
                collection_id: None,
                library_root_id: None,
                should_reject: true,
                rejection_reason,
            });
        }
        
        // File passes quality filters, apply profile actions
        info!("File passes quality filters, applying profile actions");
        
        Ok(ImportActions {
            tags_to_apply: profile.auto_tags.clone(),
            collection_id: profile.target_collection_id,
            library_root_id: profile.target_library_root_id,
            should_reject: false,
            rejection_reason: None,
        })
    }
    
    /// Check if a file meets quality filter requirements
    ///
    /// # Arguments
    /// * `metadata` - Technical metadata of the file
    /// * `filters` - Quality filters to check against
    ///
    /// # Returns
    /// Tuple of (should_reject, rejection_reason)
    fn check_quality_filters(
        &self,
        metadata: &Option<TechnicalMetadata>,
        filters: &super::models::QualityFilters,
    ) -> (bool, Option<String>) {
        // If no metadata available, cannot check quality filters
        let metadata = match metadata {
            Some(m) => m,
            None => {
                // If filters are specified but no metadata, reject
                if filters.min_resolution_width.is_some() 
                    || filters.min_resolution_height.is_some()
                    || filters.allowed_codecs.is_some() {
                    return (true, Some("No metadata available to check quality filters".to_string()));
                }
                return (false, None);
            }
        };
        
        // Check minimum resolution width
        if let Some(min_width) = filters.min_resolution_width {
            if let Some(ref video) = metadata.video {
                if (video.resolution.width as i32) < min_width {
                    return (
                        true,
                        Some(format!(
                            "Resolution width {} is below minimum {}",
                            video.resolution.width, min_width
                        ))
                    );
                }
            } else {
                return (true, Some("No video stream found".to_string()));
            }
        }
        
        // Check minimum resolution height
        if let Some(min_height) = filters.min_resolution_height {
            if let Some(ref video) = metadata.video {
                if (video.resolution.height as i32) < min_height {
                    return (
                        true,
                        Some(format!(
                            "Resolution height {} is below minimum {}",
                            video.resolution.height, min_height
                        ))
                    );
                }
            } else {
                return (true, Some("No video stream found".to_string()));
            }
        }
        
        // Check allowed codecs
        if let Some(ref allowed_codecs) = filters.allowed_codecs {
            if !allowed_codecs.is_empty() {
                if let Some(ref video) = metadata.video {
                    let codec_lower = video.codec.to_lowercase();
                    let is_allowed = allowed_codecs.iter()
                        .any(|c| codec_lower.contains(&c.to_lowercase()));
                    
                    if !is_allowed {
                        return (
                            true,
                            Some(format!(
                                "Codec '{}' is not in allowed list: {:?}",
                                video.codec, allowed_codecs
                            ))
                        );
                    }
                } else {
                    return (true, Some("No video stream found".to_string()));
                }
            }
        }
        
        // All checks passed
        (false, None)
    }
}
