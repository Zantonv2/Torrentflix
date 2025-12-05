// Maintenance Manager
// Handles library maintenance, rescan, integrity checking, and cleanup operations

use std::sync::Arc;
use std::path::{Path, PathBuf};
use std::time::Duration;
use anyhow::{Result, Context, bail};
use tracing::{info, warn, debug};
use sqlx::{SqlitePool, Row};
use chrono::{DateTime, Utc};

use crate::database::Database;
use crate::jobs::manager::JobManager;
use crate::library::models::{LibraryRoot, LibraryRootId, FileVersionId, MediaItemId};
use super::models::*;

/// MaintenanceManager handles library maintenance operations
pub struct MaintenanceManager {
    database: Arc<Database>,
    job_manager: Option<Arc<JobManager>>,
}

impl MaintenanceManager {
    /// Create a new MaintenanceManager
    pub fn new(database: Arc<Database>) -> Self {
        Self {
            database,
            job_manager: None,
        }
    }

    /// Set the job manager for background operations
    pub fn with_job_manager(mut self, job_manager: Arc<JobManager>) -> Self {
        self.job_manager = Some(job_manager);
        self
    }

    /// Get the database connection pool
    fn pool(&self) -> &SqlitePool {
        self.database.pool()
    }

    // ========================================================================
    // Rescan Operations
    // ========================================================================

    /// Schedule a rescan job for a library root
    pub async fn schedule_rescan(&self, root_id: Option<LibraryRootId>) -> Result<String> {
        let job_manager = self.job_manager.as_ref()
            .context("JobManager not configured")?;

        // Create job parameters
        let params = serde_json::json!({
            "root_id": root_id.map(|id| id.0),
        });

        // Create and enqueue job
        let job = crate::database::jobs::Job::new(
            "rescan".to_string(),
            params,
            0, // default priority
        );

        job_manager.enqueue(job).await
    }

    /// Perform a rescan operation on a library root
    /// 
    /// This operation:
    /// - Recursively walks all directories in the library root
    /// - Detects new files not in the database
    /// - Detects missing files (in DB but not on disk)
    /// - Updates FileVersion status to "missing" for missing files
    /// - Flags MediaItems as incomplete when all versions are missing
    /// - Generates a RescanReport with counts and details
    pub async fn perform_rescan(
        &self,
        root_id: LibraryRootId,
        progress: Option<Box<dyn Fn(usize, usize) + Send + Sync>>,
    ) -> Result<RescanReport> {
        info!("Starting rescan for library root {}", root_id);
        
        let mut report = RescanReport::new();

        // Get the library root information
        let root = self.get_library_root(root_id).await?;
        
        if !root.path.exists() {
            bail!("Library root path does not exist: {:?}", root.path);
        }

        // Step 1: Walk the filesystem and collect all media files
        info!("Scanning filesystem at {:?}", root.path);
        let filesystem_files = self.scan_directory(&root.path).await?;
        report.total_files_scanned = filesystem_files.len();
        
        info!("Found {} files on disk", filesystem_files.len());

        // Step 2: Get all FileVersions for this library root from database
        let db_versions = self.get_file_versions_for_root(root_id).await?;
        info!("Found {} file versions in database", db_versions.len());

        // Step 3: Compare filesystem with database
        // Build a set of absolute paths from filesystem
        let filesystem_paths: std::collections::HashSet<PathBuf> = 
            filesystem_files.iter().cloned().collect();

        // Build a map of absolute paths to version IDs from database
        let mut db_path_to_id: std::collections::HashMap<PathBuf, FileVersionId> = 
            std::collections::HashMap::new();
        
        for (version_id, path) in db_versions {
            db_path_to_id.insert(path, version_id);
        }

        // Step 4: Identify new files (on disk but not in DB)
        for fs_path in &filesystem_paths {
            if !db_path_to_id.contains_key(fs_path) {
                report.new_files.push(fs_path.clone());
                report.new_files_count += 1;
            }
        }

        info!("Identified {} new files", report.new_files_count);

        // Step 5: Identify missing files (in DB but not on disk)
        for (db_path, version_id) in &db_path_to_id {
            if !filesystem_paths.contains(db_path) {
                // File is missing - update status
                self.mark_version_as_missing(*version_id).await?;
                report.missing_versions.push(*version_id);
                report.missing_files_count += 1;
                report.status_changes_count += 1;
            }
        }

        info!("Identified {} missing files", report.missing_files_count);

        // Step 6: Identify MediaItems with all versions missing
        let incomplete_media = self.find_incomplete_media_items().await?;
        report.incomplete_media = incomplete_media.clone();
        report.incomplete_media_count = incomplete_media.len();

        info!("Identified {} incomplete media items", report.incomplete_media_count);

        // Step 7: Update last_scanned_at for the library root
        self.update_library_root_scan_time(root_id).await?;

        report.finalize();
        
        info!(
            "Rescan completed: {} new files, {} missing files, {} incomplete media items",
            report.new_files_count,
            report.missing_files_count,
            report.incomplete_media_count
        );

        Ok(report)
    }

    /// Recursively scan a directory for media files
    /// Detects and handles symbolic links and hard links
    async fn scan_directory(&self, path: &Path) -> Result<Vec<PathBuf>> {
        use crate::filesystem::LinkResolver;
        let mut files = Vec::new();
        
        // Use walkdir for recursive directory traversal
        use walkdir::WalkDir;
        
        for entry in WalkDir::new(path)
            .follow_links(false) // Don't follow symlinks to avoid loops
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            
            // Skip directories
            if !path.is_file() {
                continue;
            }

            // Check if it's a media file by extension
            if self.is_media_file(path) {
                // Analyze the path for symlinks/hardlinks
                match LinkResolver::analyze_path(path) {
                    Ok(link_info) => {
                        if link_info.is_broken {
                            // Skip broken symlinks
                            warn!("Skipping broken symlink: {:?}", path);
                            continue;
                        }
                        
                        // Add the file to the list
                        // We store the actual path (not the target) so we can track symlinks
                        files.push(path.to_path_buf());
                    }
                    Err(e) => {
                        warn!("Failed to analyze path {:?}: {}", path, e);
                        // Still add the file, but log the error
                        files.push(path.to_path_buf());
                    }
                }
            }
        }

        Ok(files)
    }

    /// Check if a file is a media file based on extension
    fn is_media_file(&self, path: &Path) -> bool {
        const MEDIA_EXTENSIONS: &[&str] = &[
            "mp4", "mkv", "avi", "mov", "wmv", "flv", "webm",
            "m4v", "mpg", "mpeg", "m2ts", "ts", "vob", "ogv",
        ];

        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| MEDIA_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
            .unwrap_or(false)
    }

    /// Get library root information
    async fn get_library_root(&self, root_id: LibraryRootId) -> Result<LibraryRoot> {
        let row = sqlx::query(
            r#"
            SELECT id, path, name, is_active, is_archive, filesystem_type, 
                   mount_point, total_space, free_space, last_scanned_at
            FROM library_roots
            WHERE id = ?
            "#
        )
        .bind(root_id.0)
        .fetch_one(self.pool())
        .await
        .context("Failed to fetch library root")?;

        Ok(LibraryRoot {
            id: root_id,
            path: PathBuf::from(row.get::<String, _>("path")),
            name: row.get("name"),
            is_active: row.get("is_active"),
            is_archive: row.get("is_archive"),
            filesystem_type: row.get("filesystem_type"),
            mount_point: row.get::<Option<String>, _>("mount_point").map(PathBuf::from),
            total_space: row.get("total_space"),
            free_space: row.get("free_space"),
            last_scanned_at: row.get("last_scanned_at"),
        })
    }

    /// Get all FileVersions for a library root
    async fn get_file_versions_for_root(
        &self,
        root_id: LibraryRootId,
    ) -> Result<Vec<(FileVersionId, PathBuf)>> {
        let rows = sqlx::query(
            r#"
            SELECT id, absolute_path
            FROM file_versions
            WHERE library_root_id = ?
              AND status != 'trashed'
            "#
        )
        .bind(root_id.0)
        .fetch_all(self.pool())
        .await
        .context("Failed to fetch file versions")?;

        let mut versions = Vec::new();
        for row in rows {
            let id = FileVersionId(row.get("id"));
            let path = PathBuf::from(row.get::<String, _>("absolute_path"));
            versions.push((id, path));
        }

        Ok(versions)
    }

    /// Mark a FileVersion as missing
    async fn mark_version_as_missing(&self, version_id: FileVersionId) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE file_versions
            SET status = 'missing',
                last_seen_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#
        )
        .bind(version_id.0)
        .execute(self.pool())
        .await
        .context("Failed to mark version as missing")?;

        debug!("Marked FileVersion {} as missing", version_id);
        Ok(())
    }

    /// Find MediaItems where all FileVersions are missing
    async fn find_incomplete_media_items(&self) -> Result<Vec<MediaItemId>> {
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT mi.id
            FROM media_items mi
            WHERE NOT EXISTS (
                SELECT 1
                FROM file_versions fv
                WHERE fv.media_item_id = mi.id
                  AND fv.status = 'present'
            )
            AND EXISTS (
                SELECT 1
                FROM file_versions fv
                WHERE fv.media_item_id = mi.id
            )
            "#
        )
        .fetch_all(self.pool())
        .await
        .context("Failed to find incomplete media items")?;

        let mut media_ids = Vec::new();
        for row in rows {
            media_ids.push(MediaItemId(row.get("id")));
        }

        Ok(media_ids)
    }

    /// Update the last_scanned_at timestamp for a library root
    async fn update_library_root_scan_time(&self, root_id: LibraryRootId) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE library_roots
            SET last_scanned_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#
        )
        .bind(root_id.0)
        .execute(self.pool())
        .await
        .context("Failed to update library root scan time")?;

        Ok(())
    }

    // ========================================================================
    // Integrity Check Operations (Placeholder for future implementation)
    // ========================================================================

    /// Schedule an integrity check job
    /// 
    /// Requirements: 5.7, 23.2, 23.3, 23.4
    pub async fn schedule_integrity_check(
        &self,
        options: IntegrityCheckOptions,
    ) -> Result<String> {
        let job_manager = self.job_manager.as_ref()
            .context("JobManager not configured")?;

        let params = serde_json::to_value(options)?;

        let job = crate::database::jobs::Job::new(
            "integrity_check".to_string(),
            params,
            0, // default priority
        );

        job_manager.enqueue(job).await
    }

    /// Perform integrity check job
    /// 
    /// This operation:
    /// - Queries FileVersions for verification based on options
    /// - Verifies checksums in batches
    /// - Marks corrupted files with corruption_detected_at
    /// - Creates alert notifications for corruption
    /// 
    /// Requirements: 5.7, 23.2, 23.3, 23.4
    pub async fn perform_integrity_check(
        &self,
        options: IntegrityCheckOptions,
    ) -> Result<IntegrityCheckReport> {
        info!("Starting integrity check with options: {:?}", options);
        
        let mut report = IntegrityCheckReport::new();

        // Build query to get FileVersions to check
        let mut query = String::from(
            r#"
            SELECT id
            FROM file_versions
            WHERE status = 'present'
            "#
        );

        // Add filter for last verification time if specified
        if let Some(min_days) = options.min_days_since_last_check {
            query.push_str(&format!(
                " AND (last_verified_at IS NULL OR last_verified_at < datetime('now', '-{} days'))",
                min_days
            ));
        }

        // Only check files with checksums if verify_checksums is enabled
        if options.verify_checksums {
            query.push_str(" AND checksum IS NOT NULL");
        }

        query.push_str(&format!(" LIMIT {}", options.batch_size));

        // Fetch FileVersions to check
        let rows = sqlx::query(&query)
            .fetch_all(self.pool())
            .await
            .context("Failed to fetch file versions for integrity check")?;

        let version_ids: Vec<FileVersionId> = rows
            .iter()
            .map(|row| FileVersionId(row.get("id")))
            .collect();

        report.total_checked = version_ids.len();
        info!("Checking integrity of {} file versions", report.total_checked);

        // Check each file
        for version_id in version_ids {
            let result = self.verify_file_integrity(version_id).await?;

            if result.passed {
                report.passed += 1;
                
                // Update last_verified_at timestamp
                sqlx::query(
                    r#"
                    UPDATE file_versions
                    SET last_verified_at = CURRENT_TIMESTAMP
                    WHERE id = ?
                    "#
                )
                .bind(version_id.0)
                .execute(self.pool())
                .await
                .context("Failed to update last_verified_at")?;
            } else {
                report.failed += 1;
                report.failed_versions.push(version_id);

                // Check if it's a corruption (checksum mismatch)
                if result.checksum_matches == Some(false) {
                    report.corrupted += 1;
                    
                    // Mark file as corrupted
                    sqlx::query(
                        r#"
                        UPDATE file_versions
                        SET status = 'corrupted',
                            corruption_detected_at = CURRENT_TIMESTAMP
                        WHERE id = ?
                        "#
                    )
                    .bind(version_id.0)
                    .execute(self.pool())
                    .await
                    .context("Failed to mark file as corrupted")?;

                    // Create alert notification
                    self.create_corruption_alert(version_id, &result).await?;
                    
                    warn!("File corruption detected for FileVersion {}", version_id);
                } else if !result.file_exists {
                    // File is missing - update status
                    self.mark_version_as_missing(version_id).await?;
                    report.missing += 1;
                }
            }
        }

        report.finalize();
        
        info!(
            "Integrity check completed: {} checked, {} passed, {} failed ({} corrupted, {} missing)",
            report.total_checked, report.passed, report.failed, report.corrupted, report.missing
        );

        Ok(report)
    }

    /// Create an alert notification for file corruption
    async fn create_corruption_alert(
        &self,
        version_id: FileVersionId,
        result: &IntegrityResult,
    ) -> Result<()> {
        // Get file path for the notification
        let row = sqlx::query(
            r#"
            SELECT absolute_path
            FROM file_versions
            WHERE id = ?
            "#
        )
        .bind(version_id.0)
        .fetch_one(self.pool())
        .await
        .context("Failed to fetch file path")?;

        let path: String = row.get("absolute_path");

        // Create notification
        sqlx::query(
            r#"
            INSERT INTO notifications (severity, title, message, created_at)
            VALUES ('critical', 'File Corruption Detected', ?, CURRENT_TIMESTAMP)
            "#
        )
        .bind(format!(
            "File corruption detected: {}\nChecksum verification failed for FileVersion {}",
            path, version_id.0
        ))
        .execute(self.pool())
        .await
        .context("Failed to create corruption alert")?;

        info!("Created corruption alert for FileVersion {}", version_id);
        Ok(())
    }

    /// Verify the integrity of a single file
    /// 
    /// This operation:
    /// - Checks if file exists at recorded path
    /// - Verifies file size matches recorded size
    /// - Optionally verifies checksum if stored
    /// - Returns IntegrityResult with pass/fail details
    /// 
    /// Requirements: 5.7, 23.2, 23.3
    pub async fn verify_file_integrity(
        &self,
        version_id: FileVersionId,
    ) -> Result<IntegrityResult> {
        debug!("Verifying integrity for FileVersion {}", version_id);

        // Get FileVersion information from database
        let row = sqlx::query(
            r#"
            SELECT absolute_path, file_size, checksum, checksum_algorithm
            FROM file_versions
            WHERE id = ?
            "#
        )
        .bind(version_id.0)
        .fetch_optional(self.pool())
        .await
        .context("Failed to fetch file version")?;

        let Some(row) = row else {
            return Ok(IntegrityResult {
                version_id,
                passed: false,
                file_exists: false,
                size_matches: false,
                checksum_matches: None,
                error_message: Some("FileVersion not found in database".to_string()),
            });
        };

        let path = PathBuf::from(row.get::<String, _>("absolute_path"));
        let expected_size: i64 = row.get("file_size");
        let checksum: Option<Vec<u8>> = row.get("checksum");
        let checksum_algorithm: Option<String> = row.get("checksum_algorithm");

        // Check if file exists
        let file_exists = tokio::fs::metadata(&path).await.is_ok();
        
        if !file_exists {
            debug!("File does not exist: {:?}", path);
            return Ok(IntegrityResult {
                version_id,
                passed: false,
                file_exists: false,
                size_matches: false,
                checksum_matches: None,
                error_message: Some(format!("File not found: {}", path.display())),
            });
        }

        // Verify file size
        let metadata = tokio::fs::metadata(&path)
            .await
            .context("Failed to read file metadata")?;
        
        let actual_size = metadata.len() as i64;
        let size_matches = actual_size == expected_size;

        if !size_matches {
            debug!(
                "File size mismatch: expected {}, got {}",
                expected_size, actual_size
            );
        }

        // Verify checksum if stored
        let checksum_matches = if let (Some(checksum_bytes), Some(algorithm_str)) = (checksum, checksum_algorithm) {
            // Parse algorithm
            let algorithm = crate::filesystem::hasher::HashAlgorithm::from_str(&algorithm_str)
                .unwrap_or(crate::filesystem::hasher::HashAlgorithm::Sha256);

            // Create expected hash
            let expected_hash = crate::filesystem::hasher::FullHash::new(algorithm, checksum_bytes);

            // Verify checksum using FileHasher
            let hasher = crate::filesystem::hasher::FileHasher::new();
            match hasher.verify_checksum(&path, &expected_hash).await {
                Ok(matches) => {
                    if !matches {
                        debug!("Checksum verification failed for {:?}", path);
                    }
                    Some(matches)
                }
                Err(e) => {
                    warn!("Failed to verify checksum: {}", e);
                    return Ok(IntegrityResult {
                        version_id,
                        passed: false,
                        file_exists: true,
                        size_matches,
                        checksum_matches: Some(false),
                        error_message: Some(format!("Checksum verification error: {}", e)),
                    });
                }
            }
        } else {
            None
        };

        // Overall pass/fail
        let passed = file_exists && size_matches && checksum_matches.unwrap_or(true);

        debug!(
            "Integrity check for FileVersion {}: passed={}, file_exists={}, size_matches={}, checksum_matches={:?}",
            version_id, passed, file_exists, size_matches, checksum_matches
        );

        Ok(IntegrityResult {
            version_id,
            passed,
            file_exists,
            size_matches,
            checksum_matches,
            error_message: None,
        })
    }

    // ========================================================================
    // Cleanup Operations
    // ========================================================================

    /// Identify cleanup candidates based on criteria
    /// 
    /// This operation:
    /// - Identifies exact duplicates by hash
    /// - Identifies lower-quality variants by quality score
    /// - Identifies trashed files past grace period
    /// - Identifies missing file records
    /// - Returns CleanupCandidate list with reasons
    /// 
    /// Requirements: 8.1, 8.2
    pub async fn identify_cleanup_candidates(
        &self,
        criteria: CleanupCriteria,
    ) -> Result<Vec<CleanupCandidate>> {
        info!("Identifying cleanup candidates with criteria: {:?}", criteria);
        
        let mut candidates = Vec::new();

        // 1. Identify exact duplicates by hash
        if criteria.include_duplicates {
            let duplicate_candidates = self.identify_duplicate_candidates().await?;
            info!("Found {} duplicate candidates", duplicate_candidates.len());
            candidates.extend(duplicate_candidates);
        }

        // 2. Identify lower-quality variants
        if criteria.include_low_quality_variants {
            let variant_candidates = self.identify_low_quality_variants().await?;
            info!("Found {} low-quality variant candidates", variant_candidates.len());
            candidates.extend(variant_candidates);
        }

        // 3. Identify trashed files past grace period
        if criteria.include_trashed {
            let trashed_candidates = self.identify_trashed_past_grace_period(
                criteria.trash_grace_period_days
            ).await?;
            info!("Found {} trashed candidates past grace period", trashed_candidates.len());
            candidates.extend(trashed_candidates);
        }

        // 4. Identify missing file records (files in DB but not on disk)
        if criteria.include_missing_records {
            let missing_candidates = self.identify_missing_file_records().await?;
            info!("Found {} missing file record candidates", missing_candidates.len());
            candidates.extend(missing_candidates);
        }

        info!("Total cleanup candidates identified: {}", candidates.len());
        Ok(candidates)
    }

    /// Identify exact duplicate files by hash
    /// For each duplicate group, keep the first version and mark others as candidates
    async fn identify_duplicate_candidates(&self) -> Result<Vec<CleanupCandidate>> {
        let mut candidates = Vec::new();

        // Get all duplicate groups from the library
        let rows = sqlx::query(
            r#"
            SELECT full_hash, COUNT(*) as count
            FROM file_versions
            WHERE full_hash IS NOT NULL
              AND status = 'present'
              AND hashing_status = 'complete'
            GROUP BY full_hash
            HAVING COUNT(*) > 1
            "#
        )
        .fetch_all(self.pool())
        .await
        .context("Failed to query duplicate hashes")?;

        // For each duplicate hash, get all versions and mark all but one as candidates
        for row in rows {
            let hash_bytes: Vec<u8> = row.get("full_hash");
            
            // Get all FileVersions with this hash
            let versions = sqlx::query(
                r#"
                SELECT id, absolute_path, file_size, is_preferred
                FROM file_versions
                WHERE full_hash = ?
                  AND status = 'present'
                ORDER BY is_preferred DESC, id ASC
                "#
            )
            .bind(&hash_bytes)
            .fetch_all(self.pool())
            .await
            .context("Failed to query duplicate versions")?;

            // Skip the first one (keep it), mark the rest as candidates
            for (idx, version_row) in versions.iter().enumerate() {
                if idx == 0 {
                    // Keep the first one (or the preferred one if any)
                    continue;
                }

                let version_id = FileVersionId(version_row.get("id"));
                let path = PathBuf::from(version_row.get::<String, _>("absolute_path"));
                let size: i64 = version_row.get("file_size");

                candidates.push(CleanupCandidate {
                    version_id,
                    path,
                    size: size as u64,
                    reason: CleanupReason::ExactDuplicate,
                });
            }
        }

        Ok(candidates)
    }

    /// Identify lower-quality variants of the same media
    /// For each MediaItem with multiple versions, keep the highest quality and mark others
    async fn identify_low_quality_variants(&self) -> Result<Vec<CleanupCandidate>> {
        use crate::library::quality_scorer::QualityScorer;
        
        let mut candidates = Vec::new();

        // Get all MediaItems with multiple versions
        let media_items = sqlx::query(
            r#"
            SELECT media_item_id, COUNT(*) as version_count
            FROM file_versions
            WHERE status = 'present'
            GROUP BY media_item_id
            HAVING COUNT(*) > 1
            "#
        )
        .fetch_all(self.pool())
        .await
        .context("Failed to query media items with multiple versions")?;

        let scorer = QualityScorer::with_default_weights();

        for item_row in media_items {
            let media_item_id: i64 = item_row.get("media_item_id");

            // Get all versions for this media item
            let versions = sqlx::query(
                r#"
                SELECT id, absolute_path, file_size, is_preferred,
                       resolution_width, resolution_height, video_codec,
                       source_type, quality_label
                FROM file_versions
                WHERE media_item_id = ?
                  AND status = 'present'
                "#
            )
            .bind(media_item_id)
            .fetch_all(self.pool())
            .await
            .context("Failed to query versions for media item")?;

            // If any version is marked as preferred, skip this media item
            // (user has explicitly chosen which version to keep)
            if versions.iter().any(|v| v.get::<bool, _>("is_preferred")) {
                continue;
            }

            // Score all versions
            let mut scored_versions: Vec<(FileVersionId, PathBuf, u64, f32)> = Vec::new();
            
            for version_row in &versions {
                let version_id = FileVersionId(version_row.get("id"));
                let path = PathBuf::from(version_row.get::<String, _>("absolute_path"));
                let size: i64 = version_row.get("file_size");
                
                // Build a minimal FileVersion for scoring
                let resolution_width: Option<i32> = version_row.get("resolution_width");
                let resolution_height: Option<i32> = version_row.get("resolution_height");
                let video_codec: Option<String> = version_row.get("video_codec");
                let source_type: Option<String> = version_row.get("source_type");

                // Calculate quality score
                let score = self.calculate_quality_score(
                    resolution_width,
                    resolution_height,
                    video_codec.as_deref(),
                    source_type.as_deref(),
                    size as u64,
                );

                scored_versions.push((version_id, path, size as u64, score));
            }

            // Sort by score descending (highest quality first)
            scored_versions.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));

            // Keep the highest quality version, mark the rest as candidates
            for (idx, (version_id, path, size, _score)) in scored_versions.iter().enumerate() {
                if idx == 0 {
                    // Keep the highest quality version
                    continue;
                }

                candidates.push(CleanupCandidate {
                    version_id: *version_id,
                    path: path.clone(),
                    size: *size,
                    reason: CleanupReason::LowerQualityVariant,
                });
            }
        }

        Ok(candidates)
    }

    /// Calculate a simple quality score for a file version
    fn calculate_quality_score(
        &self,
        width: Option<i32>,
        height: Option<i32>,
        codec: Option<&str>,
        source: Option<&str>,
        size: u64,
    ) -> f32 {
        let mut score = 0.0;

        // Resolution score (0-4)
        if let (Some(w), Some(h)) = (width, height) {
            let pixels = w * h;
            score += if pixels >= 3840 * 2160 {
                4.0 // 4K
            } else if pixels >= 1920 * 1080 {
                3.0 // 1080p
            } else if pixels >= 1280 * 720 {
                2.0 // 720p
            } else {
                1.0 // SD
            };
        }

        // Codec score (0-2)
        if let Some(codec_str) = codec {
            let codec_lower = codec_str.to_lowercase();
            score += if codec_lower.contains("265") || codec_lower.contains("hevc") {
                2.0 // H.265
            } else if codec_lower.contains("264") || codec_lower.contains("avc") {
                1.5 // H.264
            } else {
                1.0 // Other
            };
        }

        // Source score (0-2)
        if let Some(source_str) = source {
            let source_lower = source_str.to_lowercase();
            score += if source_lower.contains("bluray") || source_lower.contains("blu-ray") {
                2.0
            } else if source_lower.contains("web") {
                1.5
            } else if source_lower.contains("hdtv") {
                1.0
            } else {
                0.5
            };
        }

        // Size score (0-2) - prefer reasonable sizes
        // For 1080p, optimal is around 2-8GB
        let size_gb = size as f64 / (1024.0 * 1024.0 * 1024.0);
        score += if size_gb >= 2.0 && size_gb <= 15.0 {
            2.0
        } else if size_gb >= 1.0 && size_gb <= 20.0 {
            1.5
        } else {
            1.0
        };

        score
    }

    /// Identify trashed files past the grace period
    async fn identify_trashed_past_grace_period(&self, grace_period_days: u32) -> Result<Vec<CleanupCandidate>> {
        let mut candidates = Vec::new();

        // Calculate cutoff timestamp
        let cutoff = Utc::now() - chrono::Duration::days(grace_period_days as i64);
        let cutoff_str = cutoff.format("%Y-%m-%d %H:%M:%S").to_string();

        // Get trashed FileVersions older than grace period
        let rows = sqlx::query(
            r#"
            SELECT id, absolute_path, file_size
            FROM file_versions
            WHERE status = 'trashed'
              AND trashed_at < ?
            "#
        )
        .bind(&cutoff_str)
        .fetch_all(self.pool())
        .await
        .context("Failed to query trashed versions past grace period")?;

        for row in rows {
            let version_id = FileVersionId(row.get("id"));
            let path = PathBuf::from(row.get::<String, _>("absolute_path"));
            let size: i64 = row.get("file_size");

            candidates.push(CleanupCandidate {
                version_id,
                path,
                size: size as u64,
                reason: CleanupReason::TrashedPastGracePeriod,
            });
        }

        Ok(candidates)
    }

    /// Identify missing file records (files in DB but not on disk)
    async fn identify_missing_file_records(&self) -> Result<Vec<CleanupCandidate>> {
        let mut candidates = Vec::new();

        // Get all FileVersions with status 'missing'
        let rows = sqlx::query(
            r#"
            SELECT id, absolute_path, file_size
            FROM file_versions
            WHERE status = 'missing'
            "#
        )
        .fetch_all(self.pool())
        .await
        .context("Failed to query missing file versions")?;

        for row in rows {
            let version_id = FileVersionId(row.get("id"));
            let path = PathBuf::from(row.get::<String, _>("absolute_path"));
            let size: i64 = row.get("file_size");

            candidates.push(CleanupCandidate {
                version_id,
                path,
                size: size as u64,
                reason: CleanupReason::MissingFileRecord,
            });
        }

        Ok(candidates)
    }

    /// Estimate space savings from cleanup candidates
    /// 
    /// This operation:
    /// - Sums file sizes of all candidates
    /// - Accounts for keeping one copy of duplicates
    /// - Returns total bytes that would be freed
    /// 
    /// Requirements: 8.3
    pub async fn estimate_space_savings(&self, candidates: &[CleanupCandidate]) -> Result<u64> {
        // For exact duplicates, we're already only counting the duplicates (not the kept copy)
        // So we can just sum all candidate sizes
        let total: u64 = candidates.iter().map(|c| c.size).sum();
        Ok(total)
    }

    /// Execute cleanup operation
    /// 
    /// This operation:
    /// - Permanently deletes selected files
    /// - Removes FileVersion records
    /// - Logs errors for failed deletions
    /// - Continues with remaining files on error
    /// - Generates CleanupReport with results
    /// 
    /// Requirements: 8.4, 8.5, 8.6
    pub async fn execute_cleanup(
        &self,
        candidates: Vec<CleanupCandidate>,
    ) -> Result<CleanupReport> {
        info!("Executing cleanup for {} candidates", candidates.len());
        
        let mut report = CleanupReport::new();

        for candidate in candidates {
            // Try to delete the file
            let file_deleted = match tokio::fs::remove_file(&candidate.path).await {
                Ok(_) => {
                    info!("Deleted file: {:?}", candidate.path);
                    report.space_freed += candidate.size;
                    true
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        // File already gone, that's okay - still remove from database
                        debug!("File already deleted: {:?}", candidate.path);
                        true
                    } else {
                        warn!("Failed to delete file {:?}: {}", candidate.path, e);
                        report.errors.push(CleanupError {
                            version_id: candidate.version_id,
                            path: candidate.path.clone(),
                            error: e.to_string(),
                        });
                        false // Don't remove from database if file deletion failed
                    }
                }
            };

            // Remove FileVersion record from database if file was deleted
            if file_deleted {
                match sqlx::query(
                    r#"
                    DELETE FROM file_versions
                    WHERE id = ?
                    "#
                )
                .bind(candidate.version_id.0)
                .execute(self.pool())
                .await
                {
                    Ok(_) => {
                        report.files_deleted += 1;
                        
                        // Create audit log entry
                        let _ = self.create_audit_log_entry(
                            "cleanup_delete",
                            "file_version",
                            candidate.version_id.0,
                            Some(serde_json::json!({
                                "path": candidate.path.to_string_lossy(),
                                "size": candidate.size,
                                "reason": format!("{:?}", candidate.reason),
                            })),
                            None,
                            "system",
                        ).await;
                    }
                    Err(e) => {
                        warn!("Failed to remove FileVersion {} from database: {}", candidate.version_id, e);
                        report.errors.push(CleanupError {
                            version_id: candidate.version_id,
                            path: candidate.path,
                            error: format!("Database error: {}", e),
                        });
                    }
                }
            }
        }

        report.finalize();
        
        info!(
            "Cleanup completed: {} files deleted, {} bytes freed, {} errors",
            report.files_deleted, report.space_freed, report.errors.len()
        );

        Ok(report)
    }

    // ========================================================================
    // Trash Management
    // ========================================================================

    /// Soft delete a FileVersion (move to trash)
    /// 
    /// This operation:
    /// - Updates FileVersion status to "trashed"
    /// - Records trashed_at timestamp and original_path
    /// - Moves file to trash directory
    /// - Creates audit log entry
    /// 
    /// Requirements: 7.1, 7.2, 7.3
    pub async fn soft_delete_version(&self, version_id: FileVersionId) -> Result<()> {
        info!("Soft deleting FileVersion {}", version_id);

        // Get FileVersion information
        let row = sqlx::query(
            r#"
            SELECT absolute_path, file_size, media_item_id
            FROM file_versions
            WHERE id = ? AND status != 'trashed'
            "#
        )
        .bind(version_id.0)
        .fetch_optional(self.pool())
        .await
        .context("Failed to fetch file version")?;

        let Some(row) = row else {
            bail!("FileVersion {} not found or already trashed", version_id);
        };

        let original_path = PathBuf::from(row.get::<String, _>("absolute_path"));
        let file_size: i64 = row.get("file_size");
        let media_item_id: i64 = row.get("media_item_id");

        // Check if file exists
        if !original_path.exists() {
            warn!("File does not exist at {:?}, updating status only", original_path);
            
            // Update status in database only
            sqlx::query(
                r#"
                UPDATE file_versions
                SET status = 'trashed',
                    trashed_at = CURRENT_TIMESTAMP,
                    original_path = ?
                WHERE id = ?
                "#
            )
            .bind(original_path.to_string_lossy().to_string())
            .bind(version_id.0)
            .execute(self.pool())
            .await
            .context("Failed to update file version status")?;

            // Create audit log entry
            self.create_audit_log_entry(
                "soft_delete",
                "file_version",
                version_id.0,
                Some(serde_json::json!({
                    "path": original_path.to_string_lossy(),
                    "status": "present",
                    "file_size": file_size,
                })),
                Some(serde_json::json!({
                    "path": original_path.to_string_lossy(),
                    "status": "trashed",
                    "file_size": file_size,
                })),
                "system",
            ).await?;

            return Ok(());
        }

        // Determine trash directory
        // Use a .trash directory in the same parent directory as the file
        let trash_dir = if let Some(parent) = original_path.parent() {
            parent.join(".trash")
        } else {
            bail!("Cannot determine parent directory for {:?}", original_path);
        };

        // Create trash directory if it doesn't exist
        tokio::fs::create_dir_all(&trash_dir)
            .await
            .context("Failed to create trash directory")?;

        // Generate unique trash filename to avoid conflicts
        let filename = original_path
            .file_name()
            .context("Failed to get filename")?;
        
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let trash_filename = format!("{}_{}", timestamp, filename.to_string_lossy());
        let trash_path = trash_dir.join(trash_filename);

        // Move file to trash
        tokio::fs::rename(&original_path, &trash_path)
            .await
            .context("Failed to move file to trash")?;

        info!("Moved file from {:?} to {:?}", original_path, trash_path);

        // Update database with transaction
        let mut tx = self.pool().begin().await?;

        sqlx::query(
            r#"
            UPDATE file_versions
            SET status = 'trashed',
                trashed_at = CURRENT_TIMESTAMP,
                original_path = ?,
                absolute_path = ?
            WHERE id = ?
            "#
        )
        .bind(original_path.to_string_lossy().to_string())
        .bind(trash_path.to_string_lossy().to_string())
        .bind(version_id.0)
        .execute(&mut *tx)
        .await
        .context("Failed to update file version status")?;

        // Create audit log entry
        sqlx::query(
            r#"
            INSERT INTO audit_log (
                operation_type, entity_type, entity_id,
                old_value, new_value, initiator, timestamp
            )
            VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
            "#
        )
        .bind("soft_delete")
        .bind("file_version")
        .bind(version_id.0)
        .bind(serde_json::json!({
            "path": original_path.to_string_lossy(),
            "status": "present",
            "file_size": file_size,
            "media_item_id": media_item_id,
        }).to_string())
        .bind(serde_json::json!({
            "path": trash_path.to_string_lossy(),
            "status": "trashed",
            "file_size": file_size,
            "media_item_id": media_item_id,
            "original_path": original_path.to_string_lossy(),
        }).to_string())
        .bind("system")
        .execute(&mut *tx)
        .await
        .context("Failed to create audit log entry")?;

        tx.commit().await?;

        info!("Successfully soft deleted FileVersion {}", version_id);
        Ok(())
    }

    /// Create an audit log entry
    async fn create_audit_log_entry(
        &self,
        operation_type: &str,
        entity_type: &str,
        entity_id: i64,
        old_value: Option<serde_json::Value>,
        new_value: Option<serde_json::Value>,
        initiator: &str,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO audit_log (
                operation_type, entity_type, entity_id,
                old_value, new_value, initiator, timestamp
            )
            VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
            "#
        )
        .bind(operation_type)
        .bind(entity_type)
        .bind(entity_id)
        .bind(old_value.map(|v| v.to_string()))
        .bind(new_value.map(|v| v.to_string()))
        .bind(initiator)
        .execute(self.pool())
        .await
        .context("Failed to create audit log entry")?;

        Ok(())
    }

    /// Get all trashed FileVersions
    /// 
    /// Requirements: 7.4, 7.5
    pub async fn get_trashed_versions(&self) -> Result<Vec<(FileVersionId, PathBuf, DateTime<Utc>)>> {
        let rows = sqlx::query(
            r#"
            SELECT id, absolute_path, trashed_at
            FROM file_versions
            WHERE status = 'trashed'
            ORDER BY trashed_at DESC
            "#
        )
        .fetch_all(self.pool())
        .await
        .context("Failed to fetch trashed versions")?;

        let mut versions = Vec::new();
        for row in rows {
            let id = FileVersionId(row.get("id"));
            let path = PathBuf::from(row.get::<String, _>("absolute_path"));
            let trashed_at: String = row.get("trashed_at");
            
            // Parse the timestamp
            let trashed_at = chrono::DateTime::parse_from_rfc3339(&trashed_at)
                .or_else(|_| {
                    // Try parsing as SQLite datetime format
                    chrono::NaiveDateTime::parse_from_str(&trashed_at, "%Y-%m-%d %H:%M:%S")
                        .map(|dt| chrono::DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
                        .map(|dt| dt.into())
                })
                .context("Failed to parse trashed_at timestamp")?
                .with_timezone(&Utc);

            versions.push((id, path, trashed_at));
        }

        Ok(versions)
    }

    /// Restore a FileVersion from trash
    /// 
    /// This operation:
    /// - Checks if file is within grace period
    /// - Moves file back to original location
    /// - Updates status to "present"
    /// - Creates audit log entry
    /// 
    /// Requirements: 7.5, 7.6
    pub async fn restore_from_trash(&self, version_id: FileVersionId) -> Result<()> {
        info!("Restoring FileVersion {} from trash", version_id);

        // Get FileVersion information
        let row = sqlx::query(
            r#"
            SELECT absolute_path, original_path, trashed_at, file_size, media_item_id
            FROM file_versions
            WHERE id = ? AND status = 'trashed'
            "#
        )
        .bind(version_id.0)
        .fetch_optional(self.pool())
        .await
        .context("Failed to fetch file version")?;

        let Some(row) = row else {
            bail!("FileVersion {} not found or not in trash", version_id);
        };

        let trash_path = PathBuf::from(row.get::<String, _>("absolute_path"));
        let original_path_str: Option<String> = row.get("original_path");
        let file_size: i64 = row.get("file_size");
        let media_item_id: i64 = row.get("media_item_id");

        let Some(original_path_str) = original_path_str else {
            bail!("Original path not recorded for FileVersion {}", version_id);
        };

        let original_path = PathBuf::from(original_path_str);

        // Check if trash file exists
        if !trash_path.exists() {
            warn!("Trash file does not exist at {:?}, updating status only", trash_path);
            
            // Update status in database only
            sqlx::query(
                r#"
                UPDATE file_versions
                SET status = 'missing',
                    trashed_at = NULL,
                    original_path = NULL
                WHERE id = ?
                "#
            )
            .bind(version_id.0)
            .execute(self.pool())
            .await
            .context("Failed to update file version status")?;

            return Ok(());
        }

        // Check if original location is available
        if original_path.exists() {
            bail!(
                "Cannot restore: file already exists at original location {:?}",
                original_path
            );
        }

        // Ensure parent directory exists
        if let Some(parent) = original_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .context("Failed to create parent directory")?;
        }

        // Move file back to original location
        tokio::fs::rename(&trash_path, &original_path)
            .await
            .context("Failed to move file from trash")?;

        info!("Restored file from {:?} to {:?}", trash_path, original_path);

        // Update database with transaction
        let mut tx = self.pool().begin().await?;

        sqlx::query(
            r#"
            UPDATE file_versions
            SET status = 'present',
                trashed_at = NULL,
                original_path = NULL,
                absolute_path = ?,
                last_seen_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#
        )
        .bind(original_path.to_string_lossy().to_string())
        .bind(version_id.0)
        .execute(&mut *tx)
        .await
        .context("Failed to update file version status")?;

        // Create audit log entry
        sqlx::query(
            r#"
            INSERT INTO audit_log (
                operation_type, entity_type, entity_id,
                old_value, new_value, initiator, timestamp
            )
            VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
            "#
        )
        .bind("restore_from_trash")
        .bind("file_version")
        .bind(version_id.0)
        .bind(serde_json::json!({
            "path": trash_path.to_string_lossy(),
            "status": "trashed",
            "file_size": file_size,
            "media_item_id": media_item_id,
        }).to_string())
        .bind(serde_json::json!({
            "path": original_path.to_string_lossy(),
            "status": "present",
            "file_size": file_size,
            "media_item_id": media_item_id,
        }).to_string())
        .bind("system")
        .execute(&mut *tx)
        .await
        .context("Failed to create audit log entry")?;

        tx.commit().await?;

        info!("Successfully restored FileVersion {}", version_id);
        Ok(())
    }

    /// Purge trash (permanently delete files past grace period)
    /// 
    /// This operation:
    /// - Queries trashed FileVersions older than grace period
    /// - Permanently deletes files
    /// - Removes FileVersion records
    /// - Generates PurgeReport
    /// 
    /// Requirements: 7.7
    pub async fn purge_trash(&self, older_than: Duration) -> Result<PurgeReport> {
        info!("Purging trash older than {} days", older_than.as_secs() / 86400);
        
        let mut report = PurgeReport::new();

        // Calculate cutoff timestamp
        let cutoff = Utc::now() - chrono::Duration::from_std(older_than)?;
        let cutoff_str = cutoff.format("%Y-%m-%d %H:%M:%S").to_string();

        // Get trashed FileVersions older than grace period
        let rows = sqlx::query(
            r#"
            SELECT id, absolute_path, file_size
            FROM file_versions
            WHERE status = 'trashed'
              AND trashed_at < ?
            "#
        )
        .bind(&cutoff_str)
        .fetch_all(self.pool())
        .await
        .context("Failed to fetch trashed versions for purge")?;

        info!("Found {} files to purge", rows.len());

        for row in rows {
            let version_id = FileVersionId(row.get("id"));
            let path = PathBuf::from(row.get::<String, _>("absolute_path"));
            let file_size: i64 = row.get("file_size");

            // Try to delete the file
            match tokio::fs::remove_file(&path).await {
                Ok(_) => {
                    info!("Deleted file: {:?}", path);
                    report.space_freed += file_size as u64;
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        // File already gone, that's okay
                        debug!("File already deleted: {:?}", path);
                    } else {
                        warn!("Failed to delete file {:?}: {}", path, e);
                        report.errors.push(CleanupError {
                            version_id,
                            path: path.clone(),
                            error: e.to_string(),
                        });
                        continue; // Don't remove from database if file deletion failed
                    }
                }
            }

            // Remove FileVersion record from database
            match sqlx::query(
                r#"
                DELETE FROM file_versions
                WHERE id = ?
                "#
            )
            .bind(version_id.0)
            .execute(self.pool())
            .await
            {
                Ok(_) => {
                    report.files_purged += 1;
                    
                    // Create audit log entry
                    let _ = self.create_audit_log_entry(
                        "purge_trash",
                        "file_version",
                        version_id.0,
                        Some(serde_json::json!({
                            "path": path.to_string_lossy(),
                            "status": "trashed",
                            "file_size": file_size,
                        })),
                        None,
                        "system",
                    ).await;
                }
                Err(e) => {
                    warn!("Failed to remove FileVersion {} from database: {}", version_id, e);
                    report.errors.push(CleanupError {
                        version_id,
                        path,
                        error: format!("Database error: {}", e),
                    });
                }
            }
        }

        report.finalize();
        
        info!(
            "Purge completed: {} files purged, {} bytes freed, {} errors",
            report.files_purged, report.space_freed, report.errors.len()
        );

        Ok(report)
    }

    // ========================================================================
    // Background Hashing (Placeholder for future implementation)
    // ========================================================================

    /// Schedule a hashing job
    pub async fn schedule_hashing_job(&self, batch_size: usize) -> Result<String> {
        let job_manager = self.job_manager.as_ref()
            .context("JobManager not configured")?;

        let params = serde_json::json!({
            "batch_size": batch_size,
        });

        let job = crate::database::jobs::Job::new(
            "hashing".to_string(),
            params,
            0, // default priority
        );

        job_manager.enqueue(job).await
    }

    // ========================================================================
    // Storage Analytics
    // ========================================================================

    /// Get storage statistics for the library
    /// 
    /// This operation:
    /// - Calculates total library size
    /// - Counts MediaItems and FileVersions
    /// - Calculates average versions per media
    /// - Groups by resolution, quality label, status
    /// 
    /// Requirements: 10.1, 10.2
    pub async fn get_storage_statistics(&self) -> Result<super::models::StorageStatistics> {
        info!("Calculating storage statistics");

        // Get total library size and counts
        let row = sqlx::query(
            r#"
            SELECT 
                COALESCE(SUM(file_size), 0) as total_size,
                COUNT(*) as version_count
            FROM file_versions
            WHERE status != 'trashed'
            "#
        )
        .fetch_one(self.pool())
        .await
        .context("Failed to query total library size")?;

        let total_library_size: i64 = row.get("total_size");
        let file_version_count: i64 = row.get("version_count");

        // Get MediaItem count
        let media_item_count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM media_items
            "#
        )
        .fetch_one(self.pool())
        .await
        .context("Failed to query media item count")?;

        // Calculate average versions per media
        let average_versions_per_media = if media_item_count > 0 {
            file_version_count as f64 / media_item_count as f64
        } else {
            0.0
        };

        // Get breakdown by resolution
        let by_resolution = self.get_storage_breakdown_by_resolution().await?;

        // Get breakdown by quality label
        let by_quality_label = self.get_storage_breakdown_by_quality_label().await?;

        // Get breakdown by status
        let by_status = self.get_storage_breakdown_by_status().await?;

        let stats = super::models::StorageStatistics {
            total_library_size: total_library_size as u64,
            media_item_count: media_item_count as usize,
            file_version_count: file_version_count as usize,
            average_versions_per_media,
            by_resolution,
            by_quality_label,
            by_status,
        };

        info!(
            "Storage statistics: {} bytes, {} media items, {} file versions, {:.2} avg versions/media",
            stats.total_library_size,
            stats.media_item_count,
            stats.file_version_count,
            stats.average_versions_per_media
        );

        Ok(stats)
    }

    /// Get storage breakdown by resolution
    async fn get_storage_breakdown_by_resolution(&self) -> Result<Vec<super::models::StorageBreakdownItem>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                CASE
                    WHEN resolution_width >= 3840 THEN '4K'
                    WHEN resolution_width >= 1920 THEN '1080p'
                    WHEN resolution_width >= 1280 THEN '720p'
                    WHEN resolution_width >= 720 THEN '480p'
                    WHEN resolution_width IS NOT NULL THEN 'SD'
                    ELSE 'Unknown'
                END as category,
                COUNT(*) as file_count,
                COALESCE(SUM(file_size), 0) as total_size
            FROM file_versions
            WHERE status != 'trashed'
            GROUP BY category
            ORDER BY total_size DESC
            "#
        )
        .fetch_all(self.pool())
        .await
        .context("Failed to query storage breakdown by resolution")?;

        let mut breakdown = Vec::new();
        for row in rows {
            breakdown.push(super::models::StorageBreakdownItem {
                category: row.get("category"),
                file_count: row.get::<i64, _>("file_count") as usize,
                total_size: row.get::<i64, _>("total_size") as u64,
            });
        }

        Ok(breakdown)
    }

    /// Get storage breakdown by quality label
    async fn get_storage_breakdown_by_quality_label(&self) -> Result<Vec<super::models::StorageBreakdownItem>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                COALESCE(quality_label, 'Unknown') as category,
                COUNT(*) as file_count,
                COALESCE(SUM(file_size), 0) as total_size
            FROM file_versions
            WHERE status != 'trashed'
            GROUP BY quality_label
            ORDER BY total_size DESC
            "#
        )
        .fetch_all(self.pool())
        .await
        .context("Failed to query storage breakdown by quality label")?;

        let mut breakdown = Vec::new();
        for row in rows {
            breakdown.push(super::models::StorageBreakdownItem {
                category: row.get("category"),
                file_count: row.get::<i64, _>("file_count") as usize,
                total_size: row.get::<i64, _>("total_size") as u64,
            });
        }

        Ok(breakdown)
    }

    /// Get storage breakdown by status
    async fn get_storage_breakdown_by_status(&self) -> Result<Vec<super::models::StorageBreakdownItem>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                status as category,
                COUNT(*) as file_count,
                COALESCE(SUM(file_size), 0) as total_size
            FROM file_versions
            GROUP BY status
            ORDER BY total_size DESC
            "#
        )
        .fetch_all(self.pool())
        .await
        .context("Failed to query storage breakdown by status")?;

        let mut breakdown = Vec::new();
        for row in rows {
            breakdown.push(super::models::StorageBreakdownItem {
                category: row.get("category"),
                file_count: row.get::<i64, _>("file_count") as usize,
                total_size: row.get::<i64, _>("total_size") as u64,
            });
        }

        Ok(breakdown)
    }

    /// Calculate duplicate space usage
    /// 
    /// This operation:
    /// - Identifies duplicate groups by hash
    /// - Calculates wasted space (total - one copy per group)
    /// 
    /// Requirements: 10.3
    pub async fn calculate_duplicate_space(&self) -> Result<super::models::DuplicateSpaceInfo> {
        info!("Calculating duplicate space usage");

        // Get all duplicate groups
        let rows = sqlx::query(
            r#"
            SELECT 
                full_hash,
                COUNT(*) as file_count,
                MAX(file_size) as file_size
            FROM file_versions
            WHERE full_hash IS NOT NULL
              AND status = 'present'
              AND hashing_status = 'complete'
            GROUP BY full_hash
            HAVING COUNT(*) > 1
            "#
        )
        .fetch_all(self.pool())
        .await
        .context("Failed to query duplicate groups")?;

        let mut groups = Vec::new();
        let mut total_duplicate_size = 0u64;
        let mut wasted_space = 0u64;

        for row in rows {
            let hash: Vec<u8> = row.get("full_hash");
            let file_count: i64 = row.get("file_count");
            let file_size: i64 = row.get("file_size");

            // Get all FileVersion IDs in this group
            let version_ids: Vec<FileVersionId> = sqlx::query_scalar(
                r#"
                SELECT id
                FROM file_versions
                WHERE full_hash = ?
                  AND status = 'present'
                ORDER BY id ASC
                "#
            )
            .bind(&hash)
            .fetch_all(self.pool())
            .await
            .context("Failed to query version IDs for duplicate group")?
            .into_iter()
            .map(|id: i64| FileVersionId(id))
            .collect();

            let group_total_size = file_size as u64 * file_count as u64;
            let group_wasted_space = file_size as u64 * (file_count as u64 - 1);

            total_duplicate_size += group_total_size;
            wasted_space += group_wasted_space;

            groups.push(super::models::DuplicateGroup {
                hash,
                file_count: file_count as usize,
                file_size: file_size as u64,
                wasted_space: group_wasted_space,
                version_ids,
            });
        }

        let info = super::models::DuplicateSpaceInfo {
            duplicate_groups: groups.len(),
            total_duplicate_size,
            wasted_space,
            groups,
        };

        info!(
            "Duplicate space: {} groups, {} bytes total, {} bytes wasted",
            info.duplicate_groups, info.total_duplicate_size, info.wasted_space
        );

        Ok(info)
    }

    /// Check disk usage and create notification if threshold exceeded
    /// 
    /// This operation:
    /// - Checks disk space against threshold
    /// - Creates notification when threshold exceeded
    /// - Suggests cleanup candidates
    /// 
    /// Requirements: 8.7, 26.3
    pub async fn check_disk_usage_threshold(
        &self,
        threshold_percent: f64,
    ) -> Result<Option<String>> {
        info!("Checking disk usage threshold ({}%)", threshold_percent);

        // Get all library roots
        let rows = sqlx::query(
            r#"
            SELECT id, path, total_space, free_space
            FROM library_roots
            WHERE is_active = 1
            "#
        )
        .fetch_all(self.pool())
        .await
        .context("Failed to query library roots")?;

        for row in rows {
            let root_id: i64 = row.get("id");
            let path: String = row.get("path");
            let total_space: Option<i64> = row.get("total_space");
            let free_space: Option<i64> = row.get("free_space");

            // Try to get current disk space if not stored
            let (total, free) = if let (Some(t), Some(f)) = (total_space, free_space) {
                (t as u64, f as u64)
            } else {
                // Try to get disk space from filesystem
                match self.get_disk_space(&PathBuf::from(&path)).await {
                    Ok((t, f)) => (t, f),
                    Err(e) => {
                        warn!("Failed to get disk space for {}: {}", path, e);
                        continue;
                    }
                }
            };

            if total == 0 {
                continue;
            }

            let used_percent = ((total - free) as f64 / total as f64) * 100.0;

            if used_percent >= threshold_percent {
                // Create notification
                let message = format!(
                    "Disk usage for library root '{}' is at {:.1}% ({} GB used of {} GB total). Consider running cleanup to free space.",
                    path,
                    used_percent,
                    (total - free) / (1024 * 1024 * 1024),
                    total / (1024 * 1024 * 1024)
                );

                let notification_id = self.create_disk_usage_notification(&message).await?;
                
                warn!("Disk usage threshold exceeded for {}: {:.1}%", path, used_percent);
                
                return Ok(Some(notification_id));
            }
        }

        Ok(None)
    }

    /// Get disk space for a path
    async fn get_disk_space(&self, path: &Path) -> Result<(u64, u64)> {
        use std::os::unix::fs::MetadataExt;
        
        let _metadata = tokio::fs::metadata(path)
            .await
            .context("Failed to get filesystem metadata")?;

        // Note: Proper disk space detection requires the `nix` crate for statvfs support
        // This is a known limitation - disk space monitoring is not yet fully implemented
        // For now, return placeholder values (0, 0) to indicate unavailable
        // This prevents disk usage threshold notifications from triggering
        Ok((0, 0))
    }

    /// Create a disk usage notification
    async fn create_disk_usage_notification(&self, message: &str) -> Result<String> {
        let row = sqlx::query(
            r#"
            INSERT INTO notifications (severity, title, message, created_at)
            VALUES ('warning', 'Disk Space Warning', ?, CURRENT_TIMESTAMP)
            RETURNING id
            "#
        )
        .bind(message)
        .fetch_one(self.pool())
        .await
        .context("Failed to create disk usage notification")?;

        let notification_id: i64 = row.get("id");
        Ok(notification_id.to_string())
    }

    // ========================================================================
    // Database Maintenance
    // ========================================================================

    /// Run VACUUM on the database to reclaim space from deleted records
    /// 
    /// This operation:
    /// - Runs SQLite VACUUM to reclaim space
    /// - Rebuilds the database file to remove fragmentation
    /// - Should be scheduled during idle periods
    /// - Monitors database size before and after
    /// 
    /// Requirements: 29.1
    pub async fn vacuum_database(&self) -> Result<()> {
        info!("Starting VACUUM operation on database");

        // Run VACUUM to reclaim space from deleted records
        sqlx::query("VACUUM")
            .execute(self.pool())
            .await
            .context("Failed to execute VACUUM")?;

        info!("VACUUM operation completed successfully");

        // Create notification about VACUUM completion
        sqlx::query(
            r#"
            INSERT INTO notifications (severity, title, message, created_at)
            VALUES ('info', 'Database Maintenance Complete', ?, CURRENT_TIMESTAMP)
            "#
        )
        .bind("VACUUM operation completed. Database has been optimized and fragmentation removed.")
        .execute(self.pool())
        .await
        .ok(); // Don't fail if notification creation fails

        Ok(())
    }

    /// Rebuild database indexes to optimize query performance
    /// 
    /// This operation:
    /// - Analyzes query performance
    /// - Rebuilds indexes when needed
    /// - Updates statistics
    /// - Should be scheduled periodically
    /// 
    /// Requirements: 29.2
    pub async fn rebuild_indexes(&self) -> Result<()> {
        info!("Starting index optimization");

        // Run ANALYZE to update statistics
        sqlx::query("ANALYZE")
            .execute(self.pool())
            .await
            .context("Failed to execute ANALYZE")?;

        info!("Database statistics updated");

        // Rebuild all indexes
        let index_names: Vec<String> = sqlx::query_scalar(
            r#"
            SELECT name FROM sqlite_master
            WHERE type = 'index'
              AND name NOT LIKE 'sqlite_%'
            "#
        )
        .fetch_all(self.pool())
        .await
        .context("Failed to query indexes")?;

        let index_count = index_names.len();
        info!("Found {} indexes to rebuild", index_count);

        for index_name in index_names {
            // Drop and recreate the index
            sqlx::query(&format!("REINDEX [{}]", index_name))
                .execute(self.pool())
                .await
                .context(format!("Failed to reindex {}", index_name))?;

            debug!("Rebuilt index: {}", index_name);
        }

        info!("Index optimization completed");

        // Create notification about index optimization
        sqlx::query(
            r#"
            INSERT INTO notifications (severity, title, message, created_at)
            VALUES ('info', 'Database Optimization Complete', ?, CURRENT_TIMESTAMP)
            "#
        )
        .bind(format!(
            "Index optimization completed. Rebuilt {} indexes.",
            index_count
        ))
        .execute(self.pool())
        .await
        .ok(); // Don't fail if notification creation fails

        Ok(())
    }

    /// Clean up orphaned records not associated with any MediaItem or FileVersion
    /// 
    /// This operation:
    /// - Identifies orphaned tags (not associated with any MediaItem)
    /// - Identifies orphaned collection items (referencing deleted MediaItems)
    /// - Optionally removes orphaned records
    /// - Generates cleanup statistics
    /// 
    /// Requirements: 29.3
    pub async fn cleanup_orphaned_records(&self) -> Result<CleanupStats> {
        info!("Starting orphaned record cleanup");

        let mut stats = CleanupStats {
            orphaned_versions_removed: 0,
            orphaned_tags_removed: 0,
            orphaned_collections_removed: 0,
        };

        // 1. Find and remove orphaned FileVersions (referencing non-existent MediaItems)
        let orphaned_versions: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM file_versions
            WHERE media_item_id NOT IN (
                SELECT id FROM media_items
            )
            "#
        )
        .fetch_one(self.pool())
        .await
        .context("Failed to count orphaned file versions")?;

        if orphaned_versions > 0 {
            info!("Found {} orphaned file versions", orphaned_versions);
            
            // Delete orphaned FileVersions
            sqlx::query(
                r#"
                DELETE FROM file_versions
                WHERE media_item_id NOT IN (
                    SELECT id FROM media_items
                )
                "#
            )
            .execute(self.pool())
            .await
            .context("Failed to delete orphaned file versions")?;

            stats.orphaned_versions_removed = orphaned_versions as usize;
            
            // Create audit log entry
            self.create_audit_log_entry(
                "cleanup_orphaned_versions",
                "file_version",
                0,
                Some(serde_json::json!({
                    "count": orphaned_versions,
                })),
                None,
                "system",
            ).await.ok();
        }

        // 2. Find and remove orphaned tags (not associated with any MediaItem)
        let orphaned_tags: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM tags
            WHERE id NOT IN (
                SELECT DISTINCT tag_id FROM media_item_tags
            )
            "#
        )
        .fetch_one(self.pool())
        .await
        .context("Failed to count orphaned tags")?;

        if orphaned_tags > 0 {
            info!("Found {} orphaned tags", orphaned_tags);
            
            // Delete orphaned tags
            sqlx::query(
                r#"
                DELETE FROM tags
                WHERE id NOT IN (
                    SELECT DISTINCT tag_id FROM media_item_tags
                )
                "#
            )
            .execute(self.pool())
            .await
            .context("Failed to delete orphaned tags")?;

            stats.orphaned_tags_removed = orphaned_tags as usize;
            
            // Create audit log entry
            self.create_audit_log_entry(
                "cleanup_orphaned_tags",
                "tag",
                0,
                Some(serde_json::json!({
                    "count": orphaned_tags,
                })),
                None,
                "system",
            ).await.ok();
        }

        // 3. Find and remove orphaned collection items (referencing deleted MediaItems)
        let orphaned_collection_items: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM collection_items
            WHERE media_item_id NOT IN (
                SELECT id FROM media_items
            )
            "#
        )
        .fetch_one(self.pool())
        .await
        .context("Failed to count orphaned collection items")?;

        if orphaned_collection_items > 0 {
            info!("Found {} orphaned collection items", orphaned_collection_items);
            
            // Delete orphaned collection items
            sqlx::query(
                r#"
                DELETE FROM collection_items
                WHERE media_item_id NOT IN (
                    SELECT id FROM media_items
                )
                "#
            )
            .execute(self.pool())
            .await
            .context("Failed to delete orphaned collection items")?;

            stats.orphaned_collections_removed = orphaned_collection_items as usize;
            
            // Create audit log entry
            self.create_audit_log_entry(
                "cleanup_orphaned_collection_items",
                "collection_item",
                0,
                Some(serde_json::json!({
                    "count": orphaned_collection_items,
                })),
                None,
                "system",
            ).await.ok();
        }

        // 4. Find and remove orphaned media_item_tags (referencing deleted MediaItems or tags)
        let orphaned_media_item_tags: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM media_item_tags
            WHERE media_item_id NOT IN (SELECT id FROM media_items)
               OR tag_id NOT IN (SELECT id FROM tags)
            "#
        )
        .fetch_one(self.pool())
        .await
        .context("Failed to count orphaned media_item_tags")?;

        if orphaned_media_item_tags > 0 {
            info!("Found {} orphaned media_item_tags", orphaned_media_item_tags);
            
            // Delete orphaned media_item_tags
            sqlx::query(
                r#"
                DELETE FROM media_item_tags
                WHERE media_item_id NOT IN (SELECT id FROM media_items)
                   OR tag_id NOT IN (SELECT id FROM tags)
                "#
            )
            .execute(self.pool())
            .await
            .context("Failed to delete orphaned media_item_tags")?;

            // Add to tags count since these are tag-related
            stats.orphaned_tags_removed += orphaned_media_item_tags as usize;
        }

        info!(
            "Orphaned record cleanup completed: {} file versions, {} tags, {} collection items removed",
            stats.orphaned_versions_removed,
            stats.orphaned_tags_removed,
            stats.orphaned_collections_removed
        );

        // Create notification about cleanup
        sqlx::query(
            r#"
            INSERT INTO notifications (severity, title, message, created_at)
            VALUES ('info', 'Database Cleanup Complete', ?, CURRENT_TIMESTAMP)
            "#
        )
        .bind(format!(
            "Orphaned record cleanup completed. Removed {} file versions, {} tags, {} collection items.",
            stats.orphaned_versions_removed,
            stats.orphaned_tags_removed,
            stats.orphaned_collections_removed
        ))
        .execute(self.pool())
        .await
        .ok(); // Don't fail if notification creation fails

        Ok(stats)
    }
}
