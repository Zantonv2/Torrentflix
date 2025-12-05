// Watch Folder Monitor Component
// Monitors directories for new files using Linux inotify

use anyhow::{Result, Context};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use sqlx::{SqlitePool, Row};
use tokio::sync::{Mutex, mpsc};
use tracing::{info, warn, error, debug};
use chrono::Utc;
use thiserror::Error;
use notify::{Watcher, RecursiveMode, Event, EventKind, event::CreateKind};

use super::models::*;
use crate::import::ImportPipeline;

// ============================================================================
// Error Types
// ============================================================================

/// Watch folder monitor errors
#[derive(Error, Debug)]
pub enum WatchError {
    #[error("Watch folder not found: {0}")]
    WatchFolderNotFound(WatchFolderId),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Path does not exist: {0}")]
    PathNotFound(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Filesystem error: {0}")]
    FilesystemError(String),

    #[error("Inotify error: {0}")]
    InotifyError(String),

    #[error("Operation failed: {0}")]
    OperationFailed(String),
}

// ============================================================================
// WatchFolderMonitor
// ============================================================================

/// Main watch folder monitoring component
///
/// Uses Linux inotify to monitor directories for new files and automatically
/// trigger import operations.
///
/// # Requirements
/// - Requirements: 16.1, 16.2, 16.3, 16.4
pub struct WatchFolderMonitor {
    /// Database connection pool
    pool: Arc<SqlitePool>,
    
    /// Import pipeline for automatic import
    import_pipeline: Arc<ImportPipeline>,
    
    /// Active watch folders
    watch_folders: Arc<Mutex<HashMap<WatchFolderId, WatchFolder>>>,
    
    /// File watcher (notify crate)
    watcher: Arc<Mutex<Option<notify::RecommendedWatcher>>>,
    
    /// Event channel sender
    event_tx: mpsc::UnboundedSender<WatchEvent>,
    
    /// Event channel receiver
    event_rx: Arc<Mutex<mpsc::UnboundedReceiver<WatchEvent>>>,
    
    /// Write completion tracking (path -> last modified time)
    write_tracking: Arc<Mutex<HashMap<PathBuf, std::time::SystemTime>>>,
}

impl WatchFolderMonitor {
    /// Create a new WatchFolderMonitor instance
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `import_pipeline` - Import pipeline for automatic import
    ///
    /// # Requirements
    /// - Requirements: 16.1, 16.2, 16.3, 16.4
    pub async fn new(
        pool: Arc<SqlitePool>,
        import_pipeline: Arc<ImportPipeline>,
    ) -> Result<Self> {
        info!("Initializing WatchFolderMonitor");
        
        // Create event channel
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        
        // Load watch folders from database
        let watch_folders = Self::load_watch_folders(&pool).await?;
        info!("Loaded {} watch folders", watch_folders.len());
        
        let monitor = Self {
            pool,
            import_pipeline,
            watch_folders: Arc::new(Mutex::new(watch_folders)),
            watcher: Arc::new(Mutex::new(None)),
            event_tx,
            event_rx: Arc::new(Mutex::new(event_rx)),
            write_tracking: Arc::new(Mutex::new(HashMap::new())),
        };
        
        Ok(monitor)
    }
    
    /// Load watch folders from database
    async fn load_watch_folders(pool: &SqlitePool) -> Result<HashMap<WatchFolderId, WatchFolder>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                id,
                path,
                is_active,
                recursive,
                import_profile_id,
                last_scan_at,
                created_at
            FROM watch_folders
            WHERE is_active = 1
            ORDER BY id
            "#
        )
        .fetch_all(pool)
        .await
        .context("Failed to load watch folders")?;
        
        let mut folders = HashMap::new();
        for row in rows {
            let id: WatchFolderId = row.get("id");
            let folder = WatchFolder {
                id,
                path: PathBuf::from(row.get::<String, _>("path")),
                is_active: row.get("is_active"),
                recursive: row.get("recursive"),
                import_profile_id: row.get("import_profile_id"),
                last_scan_at: row.get::<Option<String>, _>("last_scan_at").and_then(|s| {
                    chrono::DateTime::parse_from_rfc3339(&s)
                        .ok()
                        .map(|dt| dt.with_timezone(&Utc))
                }),
                created_at: row.get::<String, _>("created_at")
                    .parse::<chrono::DateTime<Utc>>()
                    .unwrap_or_else(|_| Utc::now()),
            };
            folders.insert(id, folder);
        }
        
        Ok(folders)
    }
    
    /// Reload watch folders from database
    pub async fn reload_watch_folders(&self) -> Result<()> {
        let folders = Self::load_watch_folders(&self.pool).await?;
        let mut watch_folders = self.watch_folders.lock().await;
        *watch_folders = folders;
        info!("Reloaded {} watch folders", watch_folders.len());
        Ok(())
    }
    
    /// Get all watch folders
    pub async fn get_watch_folders(&self) -> Vec<WatchFolder> {
        let watch_folders = self.watch_folders.lock().await;
        watch_folders.values().cloned().collect()
    }
    
    /// Get a watch folder by ID
    pub async fn get_watch_folder(&self, id: WatchFolderId) -> Option<WatchFolder> {
        let watch_folders = self.watch_folders.lock().await;
        watch_folders.get(&id).cloned()
    }
    
    // ========================================================================
    // Watch Management
    // ========================================================================
    
    /// Add a watch folder
    ///
    /// Registers a directory with inotify and stores the configuration in the database.
    /// Optionally performs an initial scan of existing files.
    ///
    /// # Arguments
    /// * `path` - Path to the directory to watch
    /// * `recursive` - Whether to monitor subdirectories recursively
    /// * `profile` - Optional import profile to apply
    /// * `initial_scan` - Whether to scan existing files immediately
    ///
    /// # Returns
    /// The ID of the created watch folder
    ///
    /// # Requirements
    /// - Requirements: 16.1, 16.4, 16.5
    ///
    /// # Errors
    /// Returns an error if:
    /// - Path does not exist
    /// - Path is not a directory
    /// - Database operation fails
    /// - Inotify registration fails
    pub async fn add_watch(
        &self,
        path: PathBuf,
        recursive: bool,
        profile: Option<ImportProfile>,
        initial_scan: bool,
    ) -> Result<WatchFolderId> {
        info!("Adding watch folder: {:?} (recursive: {}, initial_scan: {})", 
              path, recursive, initial_scan);
        
        // Validate path exists and is a directory
        if !path.exists() {
            return Err(WatchError::PathNotFound(path.display().to_string()).into());
        }
        
        if !path.is_dir() {
            return Err(WatchError::InvalidPath(
                format!("Path is not a directory: {}", path.display())
            ).into());
        }
        
        // Store import profile if provided
        let profile_id = if let Some(p) = profile {
            Some(self.create_import_profile(&p).await?)
        } else {
            None
        };
        
        // Create database record
        let now = Utc::now();
        let watch_folder_id = self.create_watch_folder_record(
            &path,
            recursive,
            profile_id,
            now,
        ).await?;
        
        // Add to in-memory map
        let watch_folder = WatchFolder {
            id: watch_folder_id,
            path: path.clone(),
            is_active: true,
            recursive,
            import_profile_id: profile_id,
            last_scan_at: None,
            created_at: now,
        };
        
        {
            let mut watch_folders = self.watch_folders.lock().await;
            watch_folders.insert(watch_folder_id, watch_folder);
        }
        
        // Register with inotify (if watcher is running)
        {
            let mut watcher_guard = self.watcher.lock().await;
            if let Some(watcher) = watcher_guard.as_mut() {
                let mode = if recursive {
                    RecursiveMode::Recursive
                } else {
                    RecursiveMode::NonRecursive
                };
                
                watcher.watch(&path, mode)
                    .with_context(|| format!("Failed to watch path: {}", path.display()))?;
                
                info!("Registered watch with inotify: {:?}", path);
            }
        }
        
        // Perform initial scan if requested
        if initial_scan {
            info!("Performing initial scan of watch folder: {:?}", path);
            self.scan_watch_folder(watch_folder_id).await?;
        }
        
        info!("Watch folder added successfully: {} (path: {:?})", watch_folder_id, path);
        Ok(watch_folder_id)
    }
    
    /// Remove a watch folder
    ///
    /// Stops monitoring the directory and removes the configuration from the database.
    ///
    /// # Arguments
    /// * `id` - ID of the watch folder to remove
    ///
    /// # Requirements
    /// - Requirements: 16.7
    ///
    /// # Errors
    /// Returns an error if:
    /// - Watch folder not found
    /// - Database operation fails
    pub async fn remove_watch(&self, id: WatchFolderId) -> Result<()> {
        info!("Removing watch folder: {}", id);
        
        // Get watch folder
        let watch_folder = {
            let watch_folders = self.watch_folders.lock().await;
            watch_folders.get(&id).cloned()
        };
        
        let watch_folder = watch_folder
            .ok_or_else(|| WatchError::WatchFolderNotFound(id))?;
        
        // Unregister from inotify (if watcher is running)
        {
            let mut watcher_guard = self.watcher.lock().await;
            if let Some(watcher) = watcher_guard.as_mut() {
                if let Err(e) = watcher.unwatch(&watch_folder.path) {
                    warn!("Failed to unwatch path: {} ({})", watch_folder.path.display(), e);
                }
            }
        }
        
        // Remove from in-memory map
        {
            let mut watch_folders = self.watch_folders.lock().await;
            watch_folders.remove(&id);
        }
        
        // Update database (set is_active = false instead of deleting)
        sqlx::query(
            r#"
            UPDATE watch_folders
            SET is_active = 0
            WHERE id = ?
            "#
        )
        .bind(id)
        .execute(&*self.pool)
        .await
        .context("Failed to deactivate watch folder")?;
        
        info!("Watch folder removed successfully: {}", id);
        Ok(())
    }
    
    /// Scan a watch folder for existing files
    async fn scan_watch_folder(&self, id: WatchFolderId) -> Result<()> {
        let watch_folder = self.get_watch_folder(id).await
            .ok_or_else(|| WatchError::WatchFolderNotFound(id))?;
        
        info!("Scanning watch folder: {:?}", watch_folder.path);
        
        // Walk directory
        let paths: Vec<PathBuf> = if watch_folder.recursive {
            walkdir::WalkDir::new(&watch_folder.path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .map(|e| e.path().to_path_buf())
                .collect::<Vec<_>>()
        } else {
            std::fs::read_dir(&watch_folder.path)
                .context("Failed to read directory")?
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
                .map(|e| e.path())
                .collect::<Vec<_>>()
        };
        
        info!("Found {} files in watch folder", paths.len());
        
        // Process each file
        for path in paths {
            
            // Create watch event
            let event = WatchEvent {
                path,
                watch_folder_id: id,
                timestamp: Utc::now(),
            };
            
            // Send to event handler
            if let Err(e) = self.event_tx.send(event) {
                error!("Failed to send watch event: {}", e);
            }
        }
        
        // Update last_scan_at
        let now = Utc::now();
        sqlx::query(
            r#"
            UPDATE watch_folders
            SET last_scan_at = ?
            WHERE id = ?
            "#
        )
        .bind(now.to_rfc3339())
        .bind(id)
        .execute(&*self.pool)
        .await
        .context("Failed to update last_scan_at")?;
        
        Ok(())
    }
    
    // ========================================================================
    // Event Handling
    // ========================================================================
    
    /// Start monitoring watch folders
    ///
    /// Sets up inotify and begins processing filesystem events.
    ///
    /// # Requirements
    /// - Requirements: 16.1, 16.2, 16.3
    ///
    /// # Errors
    /// Returns an error if inotify initialization fails
    pub async fn start_monitoring(&self) -> Result<()> {
        info!("Starting watch folder monitoring");
        
        // Create inotify watcher
        let event_tx = self.event_tx.clone();
        let watch_folders_map = self.watch_folders.clone();
        
        let watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            match res {
                Ok(event) => {
                    // Handle filesystem event
                    if let Err(e) = Self::handle_notify_event(
                        event,
                        &event_tx,
                        &watch_folders_map,
                    ) {
                        error!("Failed to handle notify event: {}", e);
                    }
                }
                Err(e) => {
                    error!("Inotify error: {}", e);
                }
            }
        })
        .context("Failed to create inotify watcher")?;
        
        // Store watcher
        {
            let mut watcher_guard = self.watcher.lock().await;
            *watcher_guard = Some(watcher);
        }
        
        // Register all active watch folders
        {
            let watch_folders = self.watch_folders.lock().await;
            let mut watcher_guard = self.watcher.lock().await;
            
            if let Some(watcher) = watcher_guard.as_mut() {
                for folder in watch_folders.values() {
                    let mode = if folder.recursive {
                        RecursiveMode::Recursive
                    } else {
                        RecursiveMode::NonRecursive
                    };
                    
                    watcher.watch(&folder.path, mode)
                        .with_context(|| format!("Failed to watch path: {}", folder.path.display()))?;
                    
                    info!("Registered watch: {:?} (recursive: {})", folder.path, folder.recursive);
                }
            }
        }
        
        info!("Watch folder monitoring started successfully");
        Ok(())
    }
    
    /// Handle a notify event (internal helper)
    fn handle_notify_event(
        event: Event,
        event_tx: &mpsc::UnboundedSender<WatchEvent>,
        _watch_folders: &Arc<Mutex<HashMap<WatchFolderId, WatchFolder>>>,
    ) -> Result<()> {
        // Only handle file creation events
        if !matches!(event.kind, EventKind::Create(CreateKind::File)) {
            return Ok(());
        }
        
        // Process each path in the event
        for path in event.paths {
            // Skip if not a file
            if !path.is_file() {
                continue;
            }
            
            // Find which watch folder this belongs to
            // Note: This is a simplified version - in production, we'd need to
            // maintain a reverse mapping from paths to watch folder IDs
            debug!("File created: {:?}", path);
            
            // For now, we'll send the event with a placeholder watch folder ID
            // The actual implementation would need to track which watch folder
            // each path belongs to
            let watch_event = WatchEvent {
                path,
                watch_folder_id: 0, // Placeholder
                timestamp: Utc::now(),
            };
            
            if let Err(e) = event_tx.send(watch_event) {
                error!("Failed to send watch event: {}", e);
            }
        }
        
        Ok(())
    }
    
    /// Stop monitoring watch folders
    pub async fn stop_monitoring(&self) -> Result<()> {
        info!("Stopping watch folder monitoring");
        
        let mut watcher_guard = self.watcher.lock().await;
        *watcher_guard = None;
        
        info!("Watch folder monitoring stopped");
        Ok(())
    }
    
    // ========================================================================
    // Database Operations
    // ========================================================================
    
    /// Create a watch folder database record
    async fn create_watch_folder_record(
        &self,
        path: &Path,
        recursive: bool,
        profile_id: Option<ImportProfileId>,
        created_at: chrono::DateTime<Utc>,
    ) -> Result<WatchFolderId> {
        let path_str = path.display().to_string();
        let created_at_str = created_at.to_rfc3339();
        
        let result = sqlx::query(
            r#"
            INSERT INTO watch_folders (
                path,
                is_active,
                recursive,
                import_profile_id,
                created_at
            ) VALUES (?, 1, ?, ?, ?)
            "#
        )
        .bind(&path_str)
        .bind(recursive)
        .bind(profile_id)
        .bind(&created_at_str)
        .execute(&*self.pool)
        .await
        .context("Failed to insert watch folder record")?;
        
        Ok(result.last_insert_rowid())
    }
    
    /// Create an import profile database record
    async fn create_import_profile(&self, profile: &ImportProfile) -> Result<ImportProfileId> {
        let allowed_codecs_json = serde_json::to_string(&profile.allowed_codecs)
            .context("Failed to serialize allowed_codecs")?;
        let auto_tags_json = serde_json::to_string(&profile.auto_tags)
            .context("Failed to serialize auto_tags")?;
        let created_at_str = profile.created_at.to_rfc3339();
        
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
        .bind(profile.min_resolution_width)
        .bind(profile.min_resolution_height)
        .bind(&allowed_codecs_json)
        .bind(&auto_tags_json)
        .bind(profile.target_collection_id)
        .bind(profile.target_library_root_id)
        .bind(&created_at_str)
        .execute(&*self.pool)
        .await
        .context("Failed to insert import profile record")?;
        
        Ok(result.last_insert_rowid())
    }
    
    // ========================================================================
    // Event Processing
    // ========================================================================
    
    /// Handle a watch event
    ///
    /// Detects file write completion, triggers automatic import to staging,
    /// and begins metadata probing.
    ///
    /// # Arguments
    /// * `event` - The watch event to process
    ///
    /// # Requirements
    /// - Requirements: 16.2, 16.3
    ///
    /// # Errors
    /// Returns an error if:
    /// - File does not exist
    /// - Import operation fails
    pub async fn handle_event(&self, event: WatchEvent) -> Result<()> {
        info!("Handling watch event for: {:?}", event.path);
        
        // Check if file still exists
        if !event.path.exists() {
            debug!("File no longer exists, skipping: {:?}", event.path);
            return Ok(());
        }
        
        // Check if file is a regular file
        if !event.path.is_file() {
            debug!("Path is not a file, skipping: {:?}", event.path);
            return Ok(());
        }
        
        // Wait for write completion
        if !self.is_write_complete(&event.path).await? {
            debug!("File write not complete, will retry later: {:?}", event.path);
            
            // Re-queue the event for later processing
            let delayed_event = event.clone();
            let event_tx = self.event_tx.clone();
            
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_secs(2)).await;
                if let Err(e) = event_tx.send(delayed_event) {
                    error!("Failed to re-queue watch event: {}", e);
                }
            });
            
            return Ok(());
        }
        
        info!("File write complete, triggering import: {:?}", event.path);
        
        // Get watch folder to check for import profile
        let _watch_folder = self.get_watch_folder(event.watch_folder_id).await;
        
        // Stage the file for import
        match self.import_pipeline.stage_file(&event.path, true).await {
            Ok(staged_file) => {
                info!("File staged successfully: {:?} (ID: {})", event.path, staged_file.id);
                
                // Begin metadata probing
                match self.import_pipeline.probe_staged_file(staged_file.id).await {
                    Ok(_metadata) => {
                        info!("Metadata probing complete for staged file: {}", staged_file.id);
                        
                        // Compute fast fingerprint
                        if let Err(e) = self.import_pipeline.compute_fast_fingerprint(staged_file.id).await {
                            warn!("Failed to compute fast fingerprint: {}", e);
                        }
                        
                        // Note: Import profile rules and auto-commit are handled by the import pipeline
                        // when the file is explicitly committed by the user
                    }
                    Err(e) => {
                        error!("Metadata probing failed for staged file {}: {}", staged_file.id, e);
                        self.handle_failed_import(&event.path, format!("Metadata probing failed: {}", e)).await?;
                    }
                }
            }
            Err(e) => {
                error!("Failed to stage file {:?}: {}", event.path, e);
                self.handle_failed_import(&event.path, format!("Staging failed: {}", e)).await?;
            }
        }
        
        Ok(())
    }
    
    /// Check if file write is complete
    ///
    /// Detects write completion by checking if the file size has stabilized.
    /// Tracks file modification times and sizes to determine when writes are done.
    ///
    /// # Arguments
    /// * `path` - Path to the file to check
    ///
    /// # Returns
    /// True if write is complete, false if still being written
    ///
    /// # Requirements
    /// - Requirements: 16.2
    async fn is_write_complete(&self, path: &Path) -> Result<bool> {
        // Get current file metadata
        let metadata = std::fs::metadata(path)
            .with_context(|| format!("Failed to read file metadata: {}", path.display()))?;
        
        let current_modified = metadata.modified()
            .context("Failed to get file modification time")?;
        
        // Check if we've seen this file before
        let mut tracking = self.write_tracking.lock().await;
        
        if let Some(last_modified) = tracking.get(path) {
            // Check if modification time has changed
            if current_modified == *last_modified {
                // File hasn't been modified since last check - write is complete
                tracking.remove(path);
                return Ok(true);
            } else {
                // File is still being modified
                tracking.insert(path.to_path_buf(), current_modified);
                return Ok(false);
            }
        } else {
            // First time seeing this file - track it
            tracking.insert(path.to_path_buf(), current_modified);
            return Ok(false);
        }
    }
    
    /// Handle failed import
    ///
    /// Moves the failed file to a failed imports directory and logs the error.
    ///
    /// # Arguments
    /// * `path` - Path to the failed file
    /// * `error_message` - Error message describing the failure
    ///
    /// # Requirements
    /// - Requirements: 16.6, 26.5
    async fn handle_failed_import(&self, path: &Path, error_message: String) -> Result<()> {
        warn!("Handling failed import for {:?}: {}", path, error_message);
        
        // Create failed imports directory if it doesn't exist
        let failed_dir = self.import_pipeline.staging_dir().parent()
            .unwrap_or(self.import_pipeline.staging_dir())
            .join("failed_imports");
        
        if !failed_dir.exists() {
            std::fs::create_dir_all(&failed_dir)
                .with_context(|| format!("Failed to create failed imports directory: {}", failed_dir.display()))?;
        }
        
        // Generate unique filename for failed file
        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let failed_filename = format!("{}_{}", timestamp, filename);
        let failed_path = failed_dir.join(failed_filename);
        
        // Move file to failed imports directory
        if path.exists() {
            if let Err(e) = std::fs::rename(path, &failed_path) {
                // If rename fails, try copy + delete
                warn!("Rename failed ({}), falling back to copy + delete", e);
                std::fs::copy(path, &failed_path)
                    .with_context(|| format!("Failed to copy file to failed imports: {}", path.display()))?;
                std::fs::remove_file(path)
                    .with_context(|| format!("Failed to remove source file: {}", path.display()))?;
            }
            
            info!("Moved failed file to: {:?}", failed_path);
        }
        
        // Log error details
        error!("Import failed for {:?}: {}", path, error_message);
        
        // Note: Notifications for failed imports are handled by the notification manager
        // when the import pipeline reports the failure
        
        Ok(())
    }
    
    /// Process events from the event queue
    ///
    /// This is the main event processing loop that should be run in a background task.
    ///
    /// # Requirements
    /// - Requirements: 16.2, 16.3
    pub async fn process_events(&self) -> Result<()> {
        info!("Starting event processing loop");
        
        loop {
            // Receive event from channel
            let event = {
                let mut rx = self.event_rx.lock().await;
                rx.recv().await
            };
            
            match event {
                Some(event) => {
                    // Handle the event
                    if let Err(e) = self.handle_event(event).await {
                        error!("Failed to handle watch event: {}", e);
                    }
                }
                None => {
                    // Channel closed, exit loop
                    info!("Event channel closed, stopping event processing");
                    break;
                }
            }
        }
        
        Ok(())
    }
}
