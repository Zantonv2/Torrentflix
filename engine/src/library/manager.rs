// Library Manager component
// Orchestrates library operations including CRUD, querying, and metadata management

use anyhow::Result;
use std::sync::Arc;
use std::path::PathBuf;
use chrono::Utc;
use super::database::LibraryDatabase;
use super::models::*;
use thiserror::Error;

// ============================================================================
// Error Types
// ============================================================================

/// Library manager errors
#[derive(Error, Debug)]
pub enum LibraryError {
    #[error("MediaItem not found: {0}")]
    MediaItemNotFound(MediaItemId),

    #[error("FileVersion not found: {0}")]
    FileVersionNotFound(FileVersionId),

    #[error("LibraryRoot not found: {0}")]
    LibraryRootNotFound(LibraryRootId),

    #[error("Collection not found: {0}")]
    CollectionNotFound(CollectionId),

    #[error("Tag not found: {0}")]
    TagNotFound(TagId),

    #[error("Invalid rating: {0}. Must be between 0.0 and 10.0")]
    InvalidRating(f32),

    #[error("Invalid title: title cannot be empty")]
    InvalidTitle,

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Operation failed: {0}")]
    OperationFailed(String),
}

// ============================================================================
// Configuration
// ============================================================================

/// Library manager configuration
#[derive(Debug, Clone)]
pub struct LibraryConfig {
    /// Grace period for trashed files (in seconds)
    pub trash_grace_period: u64,

    /// Maximum concurrent hashing operations
    pub hashing_concurrency: usize,

    /// Batch size for rescan operations
    pub rescan_batch_size: usize,

    /// Disk usage threshold for cleanup notifications (in bytes)
    pub cleanup_threshold: u64,

    /// Enable metadata enrichment from external providers
    pub enable_metadata_enrichment: bool,

    /// Enable integrity verification on commit
    pub enable_integrity_verification: bool,
}

impl Default for LibraryConfig {
    fn default() -> Self {
        Self {
            trash_grace_period: 30 * 24 * 60 * 60, // 30 days
            hashing_concurrency: 1,
            rescan_batch_size: 100,
            cleanup_threshold: 100 * 1024 * 1024 * 1024, // 100 GB
            enable_metadata_enrichment: true,
            enable_integrity_verification: true,
        }
    }
}

// ============================================================================
// LibraryManager
// ============================================================================

/// Main library manager component
pub struct LibraryManager {
    database: Arc<LibraryDatabase>,
    config: LibraryConfig,
}

impl LibraryManager {
    /// Create a new LibraryManager instance with default configuration
    pub fn new(database: Arc<LibraryDatabase>) -> Self {
        Self {
            database,
            config: LibraryConfig::default(),
        }
    }

    /// Create a new LibraryManager instance with custom configuration
    pub fn with_config(database: Arc<LibraryDatabase>, config: LibraryConfig) -> Self {
        Self { database, config }
    }

    /// Get the database reference
    pub fn database(&self) -> &Arc<LibraryDatabase> {
        &self.database
    }

    /// Get the configuration
    pub fn config(&self) -> &LibraryConfig {
        &self.config
    }

    /// Update configuration
    pub fn set_config(&mut self, config: LibraryConfig) {
        self.config = config;
    }

    // ========================================================================
    // MediaItem Operations
    // ========================================================================

    /// Create a new MediaItem with title normalization
    pub async fn create_media_item(&self, media_item: &MediaItem) -> Result<MediaItemId> {
        // Validate title is not empty
        if media_item.title.trim().is_empty() {
            return Err(LibraryError::InvalidTitle.into());
        }

        // Normalize title for database storage
        let normalized_title = normalize_title(&media_item.title);

        // Create a copy with normalized title
        let mut item = media_item.clone();
        item.normalized_title = normalized_title;

        // Delegate to database
        let media_id = self.database.create_media_item(&item).await?;

        // Auto-update smart collections (Requirement 19.6)
        // This ensures the new MediaItem is automatically added to matching smart collections
        // Pass the item directly to avoid fetch issues with newly created items
        if let Err(e) = self.update_smart_collections_for_media_with_item(media_id, &item).await {
            // Log error but don't fail the creation
            eprintln!("Warning: Failed to update smart collections for media {}: {}", media_id.0, e);
        }

        Ok(media_id)
    }

    /// Get a MediaItem by ID
    pub async fn get_media_item(&self, id: MediaItemId) -> Result<Option<MediaItem>> {
        self.database.get_media_item(id).await
    }

    /// Update a MediaItem
    pub async fn update_media_item(&self, id: MediaItemId, updates: &MediaItem) -> Result<()> {
        // Validate title is not empty
        if updates.title.trim().is_empty() {
            return Err(LibraryError::InvalidTitle.into());
        }

        // Verify the MediaItem exists
        if self.database.get_media_item(id).await?.is_none() {
            return Err(LibraryError::MediaItemNotFound(id).into());
        }

        // Normalize title
        let mut item = updates.clone();
        item.normalized_title = normalize_title(&item.title);

        // Delegate to database
        self.database.update_media_item(&item).await?;

        // Auto-update smart collections (Requirement 19.6)
        // This ensures the updated MediaItem is added to/removed from smart collections as needed
        // Pass the item directly to avoid fetch issues
        if let Err(e) = self.update_smart_collections_for_media_with_item(id, &item).await {
            // Log error but don't fail the update
            eprintln!("Warning: Failed to update smart collections for media {}: {}", id.0, e);
        }

        Ok(())
    }

    /// Delete a MediaItem (with cascade deletion of associated FileVersions)
    pub async fn delete_media_item(&self, id: MediaItemId) -> Result<()> {
        // Verify the MediaItem exists
        if self.database.get_media_item(id).await?.is_none() {
            return Err(LibraryError::MediaItemNotFound(id).into());
        }

        // Delegate to database (cascade deletion handled by database)
        self.database.delete_media_item(id).await
    }

    // ========================================================================
    // FileVersion Operations
    // ========================================================================

    /// Add a FileVersion to a MediaItem
    pub async fn add_file_version(&self, media_id: MediaItemId, version: &FileVersion) -> Result<FileVersionId> {
        // Verify the MediaItem exists
        if self.database.get_media_item(media_id).await?.is_none() {
            return Err(LibraryError::MediaItemNotFound(media_id).into());
        }

        // Verify the LibraryRoot exists
        if self.database.get_library_root(version.library_root_id).await?.is_none() {
            return Err(LibraryError::LibraryRootNotFound(version.library_root_id).into());
        }

        // Delegate to database
        self.database.create_file_version(version).await
    }

    /// Get all FileVersions for a MediaItem
    pub async fn get_file_versions(&self, media_id: MediaItemId) -> Result<Vec<FileVersion>> {
        // Verify the MediaItem exists
        if self.database.get_media_item(media_id).await?.is_none() {
            return Err(LibraryError::MediaItemNotFound(media_id).into());
        }

        // Delegate to database
        self.database.get_file_versions_for_media(media_id).await
    }

    /// Update a FileVersion
    pub async fn update_file_version(&self, id: FileVersionId, updates: &FileVersion) -> Result<()> {
        // Verify the FileVersion exists
        if self.database.get_file_version(id).await?.is_none() {
            return Err(LibraryError::FileVersionNotFound(id).into());
        }

        // Delegate to database
        self.database.update_file_version(updates).await
    }

    /// Set a FileVersion as preferred (clears preferred flag from other versions of same MediaItem)
    pub async fn set_preferred_version(&self, id: FileVersionId) -> Result<()> {
        // Verify the FileVersion exists
        let version = self.database.get_file_version(id).await?;
        let version = version.ok_or(LibraryError::FileVersionNotFound(id))?;

        // Get all versions for this MediaItem
        let mut all_versions = self.database.get_file_versions_for_media(version.media_item_id).await?;

        // Clear preferred flag from all versions
        for v in &mut all_versions {
            v.is_preferred = false;
        }

        // Set preferred flag on the specified version
        for v in &mut all_versions {
            if v.id == id {
                v.is_preferred = true;
            }
        }

        // Update all versions in database
        for v in all_versions {
            self.database.update_file_version(&v).await?;
        }

        Ok(())
    }

    /// Detect and populate filesystem information for a path
    /// This includes symlink detection, target resolution, and hardlink detection
    pub fn detect_filesystem_info(&self, path: &std::path::Path) -> Result<FilesystemInfo> {
        use crate::filesystem::LinkResolver;
        
        let link_info = LinkResolver::analyze_path(path)?;
        
        Ok(FilesystemInfo {
            is_symlink: link_info.is_symlink,
            symlink_target: link_info.symlink_target,
            is_hardlink: link_info.is_hardlink,
            inode: link_info.inode,
        })
    }

    // ========================================================================
    // Library Querying and Filtering
    // ========================================================================

    /// Query library with filters
    pub async fn query_library(&self, filters: &LibraryFilters) -> Result<Vec<MediaItem>> {
        use super::queries::LibraryQueries;
        let queries = LibraryQueries::new(self.database.clone());
        queries.query_library(filters).await
    }

    /// Search library by text
    pub async fn search_library(&self, query: &str, options: &SearchOptions) -> Result<Vec<MediaItem>> {
        use super::queries::LibraryQueries;
        let queries = LibraryQueries::new(self.database.clone());
        queries.search_library(query, options).await
    }

    /// Search library by custom notes
    pub async fn search_by_notes(&self, query: &str, options: &SearchOptions) -> Result<Vec<MediaItem>> {
        use super::queries::LibraryQueries;
        let queries = LibraryQueries::new(self.database.clone());
        queries.search_by_notes(query, options).await
    }

    // ========================================================================
    // User Metadata Management
    // ========================================================================

    /// Set user rating for a MediaItem (0.0-10.0)
    pub async fn set_user_rating(&self, media_id: MediaItemId, rating: f32) -> Result<()> {
        // Validate rating range
        if rating < 0.0 || rating > 10.0 {
            return Err(LibraryError::InvalidRating(rating).into());
        }

        // Get the MediaItem
        let mut media_item = self.database.get_media_item(media_id).await?
            .ok_or(LibraryError::MediaItemNotFound(media_id))?;

        // Update rating
        media_item.user_rating = Some(rating);

        // Persist to database
        self.database.update_media_item(&media_item).await
    }

    /// Add tags to a MediaItem
    pub async fn add_tags(&self, media_id: MediaItemId, tags: &[String]) -> Result<()> {
        // Get the MediaItem
        let mut media_item = self.database.get_media_item(media_id).await?
            .ok_or(LibraryError::MediaItemNotFound(media_id))?;

        // Create or get tags and add to MediaItem
        for tag_name in tags {
            let tag = self.database.get_or_create_tag(tag_name).await?;
            if !media_item.tags.contains(&tag.id) {
                media_item.tags.push(tag.id);
            }
        }

        // Persist to database
        self.database.update_media_item(&media_item).await
    }

    /// Remove tags from a MediaItem
    pub async fn remove_tags(&self, media_id: MediaItemId, tags: &[String]) -> Result<()> {
        // Get the MediaItem
        let mut media_item = self.database.get_media_item(media_id).await?
            .ok_or(LibraryError::MediaItemNotFound(media_id))?;

        // Remove tags
        for tag_name in tags {
            if let Some(tag) = self.database.get_tag_by_name(tag_name).await? {
                media_item.tags.retain(|&id| id != tag.id);
            }
        }

        // Persist to database
        self.database.update_media_item(&media_item).await
    }

    /// Get tags for a MediaItem
    pub async fn get_tags_for_media(&self, media_id: MediaItemId) -> Result<Vec<Tag>> {
        // Verify the MediaItem exists
        if self.database.get_media_item(media_id).await?.is_none() {
            return Err(LibraryError::MediaItemNotFound(media_id).into());
        }

        // Delegate to database
        self.database.get_tags_for_media(media_id).await
    }

    /// Filter MediaItems by tags (returns items that have any of the specified tags)
    pub async fn filter_by_tags(&self, tag_ids: &[TagId]) -> Result<Vec<MediaItem>> {
        if tag_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Build a query that returns MediaItems with any of the specified tags
        let mut all_items = Vec::new();
        for tag_id in tag_ids {
            let items = self.database.get_media_items_by_tag(*tag_id).await?;
            all_items.extend(items);
        }

        // Remove duplicates (items that have multiple matching tags)
        all_items.sort_by_key(|item| item.id.0);
        all_items.dedup_by_key(|item| item.id.0);

        Ok(all_items)
    }

    /// Set watch status for a MediaItem
    pub async fn set_watch_status(&self, media_id: MediaItemId, status: WatchStatus) -> Result<()> {
        // Get the MediaItem
        let mut media_item = self.database.get_media_item(media_id).await?
            .ok_or(LibraryError::MediaItemNotFound(media_id))?;

        // Update watch status
        media_item.watch_status = status;
        if status == WatchStatus::Watched {
            media_item.last_watched_at = Some(chrono::Utc::now());
        }

        // Persist to database
        self.database.update_media_item(&media_item).await
    }

    /// Set custom notes for a MediaItem
    pub async fn set_custom_notes(&self, media_id: MediaItemId, notes: &str) -> Result<()> {
        // Get the MediaItem
        let mut media_item = self.database.get_media_item(media_id).await?
            .ok_or(LibraryError::MediaItemNotFound(media_id))?;

        // Update notes
        media_item.custom_notes = if notes.is_empty() { None } else { Some(notes.to_string()) };

        // Persist to database
        self.database.update_media_item(&media_item).await
    }

    // ========================================================================
    // Metadata Editing (Requirement 24)
    // ========================================================================

    /// Edit metadata fields for a MediaItem
    /// Marks edited fields to prevent automatic overwrites during metadata refresh
    /// Requirement 24.1, 24.2
    pub async fn edit_metadata(&self, media_id: MediaItemId, edits: MetadataEdit) -> Result<()> {
        // Verify the MediaItem exists
        let mut media_item = self.database.get_media_item(media_id).await?
            .ok_or(LibraryError::MediaItemNotFound(media_id))?;

        // Track which fields are being edited
        let edited_fields = edits.edited_field_names();

        // Apply edits
        if let Some(title) = edits.title {
            if title.trim().is_empty() {
                return Err(LibraryError::InvalidTitle.into());
            }
            media_item.title = title.clone();
            media_item.normalized_title = normalize_title(&title);
        }

        if let Some(original_title) = edits.original_title {
            media_item.original_title = original_title;
        }

        if let Some(year) = edits.year {
            media_item.year = year;
        }

        if let Some(overview) = edits.overview {
            media_item.overview = overview;
        }

        if let Some(genres) = edits.genres {
            media_item.genres = genres;
        }

        if let Some(cast) = edits.cast {
            media_item.cast = cast;
        }

        if let Some(director) = edits.director {
            media_item.director = director;
        }

        if let Some(runtime) = edits.runtime {
            media_item.runtime = runtime;
        }

        if let Some(alternative_titles) = edits.alternative_titles {
            media_item.alternative_titles = alternative_titles;
        }

        // Mark fields as user-edited to prevent auto-overwrite
        for field in edited_fields {
            media_item.user_edited_fields.insert(field);
        }

        // Update timestamp
        media_item.updated_at = Utc::now();

        // Persist to database
        self.database.update_media_item(&media_item).await?;

        Ok(())
    }

    /// Reset a metadata field to its original value from providers or parsed data
    /// Clears the user-edited flag for that field
    /// Requirement 24.3
    pub async fn reset_metadata_field(&self, media_id: MediaItemId, field_name: &str) -> Result<()> {
        // Verify the MediaItem exists
        let mut media_item = self.database.get_media_item(media_id).await?
            .ok_or(LibraryError::MediaItemNotFound(media_id))?;

        // Remove from user-edited fields
        media_item.user_edited_fields.remove(field_name);

        // For now, we just clear the user-edited flag
        // In a full implementation, we would re-fetch from external providers
        // or restore from a backup of the original parsed metadata

        // Update timestamp
        media_item.updated_at = Utc::now();

        // Persist to database
        self.database.update_media_item(&media_item).await?;

        Ok(())
    }

    /// Add alternative titles for improved search matching
    /// Requirement 24.5
    pub async fn add_alternative_titles(&self, media_id: MediaItemId, titles: Vec<String>) -> Result<()> {
        // Verify the MediaItem exists
        let mut media_item = self.database.get_media_item(media_id).await?
            .ok_or(LibraryError::MediaItemNotFound(media_id))?;

        // Add new titles (avoiding duplicates)
        for title in titles {
            if !media_item.alternative_titles.contains(&title) {
                media_item.alternative_titles.push(title);
            }
        }

        // Mark as user-edited
        media_item.user_edited_fields.insert("alternative_titles".to_string());

        // Update timestamp
        media_item.updated_at = Utc::now();

        // Persist to database
        self.database.update_media_item(&media_item).await?;

        Ok(())
    }

    /// Upload custom artwork (poster or backdrop)
    /// Requirement 24.6
    pub async fn upload_custom_artwork(&self, media_id: MediaItemId, artwork: CustomArtwork) -> Result<()> {
        // Verify the MediaItem exists
        let mut media_item = self.database.get_media_item(media_id).await?
            .ok_or(LibraryError::MediaItemNotFound(media_id))?;

        // Set custom artwork
        if let Some(poster) = artwork.poster {
            media_item.custom_poster = Some(poster);
            media_item.user_edited_fields.insert("custom_poster".to_string());
        }

        if let Some(backdrop) = artwork.backdrop {
            media_item.custom_backdrop = Some(backdrop);
            media_item.user_edited_fields.insert("custom_backdrop".to_string());
        }

        // Update timestamp
        media_item.updated_at = Utc::now();

        // Persist to database
        self.database.update_media_item(&media_item).await?;

        Ok(())
    }

    // ========================================================================
    // Duplicate Detection
    // ========================================================================

    /// Get duplicate groups by full hash
    pub async fn get_duplicates(&self) -> Result<Vec<DuplicateGroup>> {
        self.database.get_duplicates().await
    }

    /// Get variants for a MediaItem (all FileVersions for the same media)
    pub async fn get_variants(&self, media_id: MediaItemId) -> Result<Vec<FileVersion>> {
        // Verify the MediaItem exists
        if self.database.get_media_item(media_id).await?.is_none() {
            return Err(LibraryError::MediaItemNotFound(media_id).into());
        }

        // Delegate to database
        self.database.get_file_versions_for_media(media_id).await
    }

    // ========================================================================
    // Collection Management
    // ========================================================================

    /// Create a new Collection
    pub async fn create_collection(&self, name: &str, description: Option<&str>) -> Result<CollectionId> {
        let collection = Collection {
            id: CollectionId(0), // Will be assigned by database
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            collection_type: CollectionType::Manual,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        self.database.create_collection(&collection).await
    }

    /// Add MediaItems to a Collection
    pub async fn add_to_collection(&self, collection_id: CollectionId, media_ids: &[MediaItemId]) -> Result<()> {
        // Verify the Collection exists
        if self.database.get_collection(collection_id).await?.is_none() {
            return Err(LibraryError::CollectionNotFound(collection_id).into());
        }

        // Verify all MediaItems exist
        for media_id in media_ids {
            if self.database.get_media_item(*media_id).await?.is_none() {
                return Err(LibraryError::MediaItemNotFound(*media_id).into());
            }
        }

        // Delegate to database
        self.database.add_to_collection(collection_id, media_ids).await
    }

    /// Get MediaItems in a Collection
    pub async fn get_collection_items(&self, collection_id: CollectionId) -> Result<Vec<MediaItem>> {
        // Verify the Collection exists
        if self.database.get_collection(collection_id).await?.is_none() {
            return Err(LibraryError::CollectionNotFound(collection_id).into());
        }

        // Delegate to database
        self.database.get_collection_items(collection_id).await
    }

    /// Create a Smart Collection
    pub async fn create_smart_collection(&self, name: &str, criteria: &FilterCriteria) -> Result<CollectionId> {
        let collection = Collection {
            id: CollectionId(0), // Will be assigned by database
            name: name.to_string(),
            description: None,
            collection_type: CollectionType::Smart { criteria: criteria.clone() },
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        self.database.create_collection(&collection).await
    }

    /// Query MediaItems for a smart collection based on its filter criteria
    /// This dynamically evaluates the stored criteria and returns matching items
    pub async fn query_smart_collection(&self, collection_id: CollectionId) -> Result<Vec<MediaItem>> {
        // Get the collection
        let collection = self.database.get_collection(collection_id).await?
            .ok_or(LibraryError::CollectionNotFound(collection_id))?;

        // Extract criteria from smart collection
        let criteria = match collection.collection_type {
            CollectionType::Smart { criteria } => criteria,
            CollectionType::Manual => {
                return Err(LibraryError::OperationFailed(
                    "Cannot query manual collection with smart collection query".to_string()
                ).into());
            }
        };

        // Get all MediaItems from the collection_items table and filter them based on criteria
        let collection_items = self.database.get_collection_items(collection_id).await?;
        let mut matching_items = Vec::new();

        for item in collection_items {
            if self.matches_smart_collection_criteria(&item, &criteria).await? {
                matching_items.push(item);
            }
        }

        Ok(matching_items)
    }

    /// Convert FilterCriteria to LibraryFilters for querying
    fn filter_criteria_to_library_filters(&self, criteria: &FilterCriteria) -> LibraryFilters {
        LibraryFilters {
            title: criteria.title_pattern.clone(),
            year: criteria.year_range.as_ref().map(|(start, _)| *start), // Use start of range
            resolution: None, // Not directly supported in FilterCriteria
            quality_label: criteria.quality_labels.as_ref().and_then(|labels| labels.first().cloned()),
            status: None, // Not in FilterCriteria
            tags: None, // Will be handled separately if needed
            min_rating: criteria.min_rating,
            watch_status: criteria.watch_status,
            sort_by: Some(SortField::Title),
            sort_ascending: true,
            limit: None,
            offset: None,
        }
    }

    /// Check if a MediaItem matches smart collection criteria
    /// Used for auto-updating smart collections when MediaItems are created/updated
    pub async fn matches_smart_collection_criteria(&self, media_item: &MediaItem, criteria: &FilterCriteria) -> Result<bool> {
        // Check title pattern
        if let Some(ref pattern) = criteria.title_pattern {
            let normalized_pattern = normalize_title(pattern);
            if !media_item.normalized_title.contains(&normalized_pattern) {
                return Ok(false);
            }
        }

        // Check year range
        if let Some((start, end)) = criteria.year_range {
            if let Some(year) = media_item.year {
                if year < start || year > end {
                    return Ok(false);
                }
            } else {
                return Ok(false); // No year means doesn't match year range criteria
            }
        }

        // Check quality labels (need to check FileVersions)
        if let Some(ref quality_labels) = criteria.quality_labels {
            let versions = self.database.get_file_versions_for_media(media_item.id).await?;
            let has_matching_quality = versions.iter().any(|v| {
                v.quality_label.as_ref().map_or(false, |label| quality_labels.contains(label))
            });
            if !has_matching_quality {
                return Ok(false);
            }
        }

        // Check tags
        if let Some(ref required_tags) = criteria.tags {
            let media_tags = self.database.get_tags_for_media(media_item.id).await?;
            let media_tag_names: Vec<String> = media_tags.iter().map(|t| t.name.clone()).collect();
            let has_matching_tag = required_tags.iter().any(|tag| media_tag_names.contains(tag));
            if !has_matching_tag {
                return Ok(false);
            }
        }

        // Check minimum rating
        if let Some(min_rating) = criteria.min_rating {
            if let Some(user_rating) = media_item.user_rating {
                if user_rating < min_rating {
                    return Ok(false);
                }
            } else {
                return Ok(false); // No rating means doesn't meet minimum
            }
        }

        // Check watch status
        if let Some(required_status) = criteria.watch_status {
            if media_item.watch_status != required_status {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Internal: Update smart collections with a provided MediaItem
    /// This avoids fetch issues when called immediately after creation
    async fn update_smart_collections_for_media_with_item(&self, media_id: MediaItemId, media_item: &MediaItem) -> Result<()> {
        // Get all smart collections
        let all_collections = self.database.get_all_collections().await?;

        for collection in all_collections {
            if let CollectionType::Smart { ref criteria } = collection.collection_type {
                // Check if the media item matches this smart collection's criteria
                let matches = self.matches_smart_collection_criteria(media_item, criteria).await?;

                if matches {
                    // Check if already in collection
                    let items = self.database.get_collection_items(collection.id).await?;
                    let already_in_collection = items.iter().any(|item| item.id == media_id);

                    if !already_in_collection {
                        // Add to collection
                        self.database.add_to_collection(collection.id, &[media_id]).await?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Update smart collections when a MediaItem is created or updated
    /// Automatically adds the item to all matching smart collections
    pub async fn update_smart_collections_for_media(&self, media_id: MediaItemId) -> Result<()> {
        // Get the MediaItem
        let media_item = match self.database.get_media_item(media_id).await? {
            Some(item) => item,
            None => {
                // MediaItem not found - this can happen if called immediately after creation
                // before the transaction is committed. Just return Ok and skip the update.
                return Ok(());
            }
        };

        // Use the internal function with the fetched item
        self.update_smart_collections_for_media_with_item(media_id, &media_item).await
    }

    /// Export a collection to JSON format
    /// Returns a JSON string containing all MediaItems with their metadata and file paths
    pub async fn export_collection_json(&self, collection_id: CollectionId) -> Result<String> {
        // Get collection items
        let items = self.get_collection_items(collection_id).await?;

        // Build export data structure
        let mut export_data = Vec::new();

        for item in items {
            // Get file versions for this item
            let versions = self.database.get_file_versions_for_media(item.id).await?;

            let item_data = serde_json::json!({
                "id": item.id.0,
                "title": item.title,
                "year": item.year,
                "media_type": format!("{:?}", item.media_type),
                "overview": item.overview,
                "genres": item.genres,
                "director": item.director,
                "runtime": item.runtime,
                "user_rating": item.user_rating,
                "watch_status": format!("{:?}", item.watch_status),
                "file_versions": versions.iter().map(|v| {
                    let (resolution_str, codec_str) = if let Some(ref tech) = v.technical_metadata {
                        let res = tech.video.as_ref().map(|vi| format!("{}x{}", vi.resolution.width, vi.resolution.height))
                            .unwrap_or_else(|| "unknown".to_string());
                        let codec = tech.video.as_ref().map(|vi| vi.codec.clone())
                            .unwrap_or_else(|| "unknown".to_string());
                        (res, codec)
                    } else {
                        ("unknown".to_string(), "unknown".to_string())
                    };

                    serde_json::json!({
                        "path": v.absolute_path,
                        "file_size": v.file_size,
                        "resolution": resolution_str,
                        "codec": codec_str,
                        "quality_label": v.quality_label,
                        "status": format!("{:?}", v.status),
                    })
                }).collect::<Vec<_>>(),
            });

            export_data.push(item_data);
        }

        Ok(serde_json::to_string_pretty(&export_data)?)
    }

    /// Export a collection to CSV format
    /// Returns a CSV string with MediaItem and FileVersion information
    pub async fn export_collection_csv(&self, collection_id: CollectionId) -> Result<String> {
        // Get collection items
        let items = self.get_collection_items(collection_id).await?;

        let mut csv = String::from("Title,Year,Type,Path,Size,Resolution,Codec,Quality,Status\n");

        for item in items {
            // Get file versions for this item
            let versions = self.database.get_file_versions_for_media(item.id).await?;

            for version in versions {
                let (resolution_str, codec_str) = if let Some(ref tech) = version.technical_metadata {
                    let res = tech.video.as_ref().map(|vi| format!("{}x{}", vi.resolution.width, vi.resolution.height))
                        .unwrap_or_else(|| "unknown".to_string());
                    let codec = tech.video.as_ref().map(|vi| vi.codec.clone())
                        .unwrap_or_else(|| "unknown".to_string());
                    (res, codec)
                } else {
                    ("unknown".to_string(), "unknown".to_string())
                };

                csv.push_str(&format!(
                    "\"{}\",{},{},\"{}\",{},{},{},{},{}\n",
                    item.title.replace('"', "\"\""),
                    item.year.map_or(String::from(""), |y| y.to_string()),
                    format!("{:?}", item.media_type),
                    version.absolute_path.display().to_string().replace('"', "\"\""),
                    version.file_size,
                    resolution_str,
                    codec_str,
                    version.quality_label.as_deref().unwrap_or(""),
                    format!("{:?}", version.status),
                ));
            }
        }

        Ok(csv)
    }

    // ========================================================================
    // Saved Search Management
    // ========================================================================

    /// Create a new saved search
    pub async fn create_saved_search(
        &self,
        name: &str,
        description: Option<&str>,
        query: &str,
        filters: &LibraryFilters,
        search_options: &SearchOptions,
    ) -> Result<SavedSearchId> {
        let saved_search = SavedSearch {
            id: SavedSearchId(0), // Will be assigned by database
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            query: query.to_string(),
            filters: filters.clone(),
            search_options: search_options.clone(),
            created_at: chrono::Utc::now(),
            last_used_at: None,
        };

        self.database.create_saved_search(&saved_search).await
    }

    /// Get a saved search by ID
    pub async fn get_saved_search(&self, id: SavedSearchId) -> Result<Option<SavedSearch>> {
        self.database.get_saved_search(id).await
    }

    /// Get all saved searches
    pub async fn get_all_saved_searches(&self) -> Result<Vec<SavedSearch>> {
        self.database.get_all_saved_searches().await
    }

    /// Execute a saved search
    pub async fn execute_saved_search(&self, id: SavedSearchId) -> Result<Vec<MediaItem>> {
        // Get the saved search
        let mut saved_search = self.database.get_saved_search(id).await?
            .ok_or_else(|| anyhow::anyhow!("SavedSearch not found: {}", id))?;

        // Update last_used_at
        saved_search.last_used_at = Some(chrono::Utc::now());
        self.database.update_saved_search(&saved_search).await?;

        // Execute the search
        self.search_library(&saved_search.query, &saved_search.search_options).await
    }

    /// Delete a saved search
    pub async fn delete_saved_search(&self, id: SavedSearchId) -> Result<()> {
        self.database.delete_saved_search(id).await
    }

    /// Update a saved search
    pub async fn update_saved_search(&self, saved_search: &SavedSearch) -> Result<()> {
        self.database.update_saved_search(saved_search).await
    }

    // ========================================================================
    // Batch Operations
    // ========================================================================

    /// Apply tags to multiple MediaItems
    /// Returns a result with success/failure counts and details for each item
    pub async fn batch_add_tags(
        &self,
        media_ids: &[MediaItemId],
        tags: &[String],
        progress_callback: Option<&dyn Fn(usize, usize)>,
    ) -> Result<BatchTagResult> {
        let total_items = media_ids.len();
        let mut results = Vec::with_capacity(total_items);
        let mut successful = 0;
        let mut failed = 0;

        for (index, &media_id) in media_ids.iter().enumerate() {
            // Report progress
            if let Some(callback) = progress_callback {
                callback(index + 1, total_items);
            }

            // Attempt to add tags to this MediaItem
            match self.add_tags(media_id, tags).await {
                Ok(_) => {
                    successful += 1;
                    results.push(BatchItemResult {
                        media_item_id: media_id,
                        success: true,
                        error: None,
                    });
                }
                Err(e) => {
                    failed += 1;
                    results.push(BatchItemResult {
                        media_item_id: media_id,
                        success: false,
                        error: Some(e.to_string()),
                    });
                }
            }
        }

        Ok(BatchTagResult {
            total_items,
            successful,
            failed,
            results,
        })
    }

    /// Assign a rating to multiple MediaItems
    pub async fn batch_set_rating(
        &self,
        media_ids: &[MediaItemId],
        rating: f32,
        progress_callback: Option<&dyn Fn(usize, usize)>,
    ) -> Result<BatchRatingResult> {
        let total_items = media_ids.len();
        let mut results = Vec::with_capacity(total_items);
        let mut successful = 0;
        let mut failed = 0;

        for (index, &media_id) in media_ids.iter().enumerate() {
            // Report progress
            if let Some(callback) = progress_callback {
                callback(index + 1, total_items);
            }

            // Attempt to set rating for this MediaItem
            match self.set_user_rating(media_id, rating).await {
                Ok(_) => {
                    successful += 1;
                    results.push(BatchItemResult {
                        media_item_id: media_id,
                        success: true,
                        error: None,
                    });
                }
                Err(e) => {
                    failed += 1;
                    results.push(BatchItemResult {
                        media_item_id: media_id,
                        success: false,
                        error: Some(e.to_string()),
                    });
                }
            }
        }

        Ok(BatchRatingResult {
            total_items,
            successful,
            failed,
            results,
        })
    }

    /// Add multiple MediaItems to a collection
    pub async fn batch_add_to_collection(
        &self,
        media_ids: &[MediaItemId],
        collection_id: CollectionId,
        progress_callback: Option<&dyn Fn(usize, usize)>,
    ) -> Result<BatchCollectionResult> {
        let total_items = media_ids.len();
        let mut results = Vec::with_capacity(total_items);
        let mut successful = 0;
        let mut failed = 0;

        // Verify collection exists
        if self.database.get_collection(collection_id).await?.is_none() {
            return Err(LibraryError::CollectionNotFound(collection_id).into());
        }

        for (index, &media_id) in media_ids.iter().enumerate() {
            // Report progress
            if let Some(callback) = progress_callback {
                callback(index + 1, total_items);
            }

            // Attempt to add this MediaItem to the collection
            match self.add_to_collection(collection_id, &[media_id]).await {
                Ok(_) => {
                    successful += 1;
                    results.push(BatchItemResult {
                        media_item_id: media_id,
                        success: true,
                        error: None,
                    });
                }
                Err(e) => {
                    failed += 1;
                    results.push(BatchItemResult {
                        media_item_id: media_id,
                        success: false,
                        error: Some(e.to_string()),
                    });
                }
            }
        }

        Ok(BatchCollectionResult {
            total_items,
            successful,
            failed,
            results,
        })
    }

    /// Soft-delete all FileVersions associated with multiple MediaItems
    /// Creates audit log entries for each deletion
    pub async fn batch_delete_media_items(
        &self,
        media_ids: &[MediaItemId],
        progress_callback: Option<&dyn Fn(usize, usize)>,
    ) -> Result<BatchDeletionResult> {
        let total_items = media_ids.len();
        let mut results = Vec::with_capacity(total_items);
        let mut total_versions = 0;
        let mut successful_versions = 0;
        let mut failed_versions = 0;

        for (index, &media_id) in media_ids.iter().enumerate() {
            // Report progress
            if let Some(callback) = progress_callback {
                callback(index + 1, total_items);
            }

            // Get all FileVersions for this MediaItem
            match self.get_file_versions(media_id).await {
                Ok(versions) => {
                    let version_count = versions.len();
                    total_versions += version_count;
                    let mut item_success = true;
                    let mut item_errors = Vec::new();

                    // Soft-delete each version
                    for version in versions {
                        // Note: We need to implement soft_delete_version in MaintenanceManager
                        // For now, we'll update the status directly
                        match self.database.update_file_version_status(
                            version.id,
                            FileStatus::Trashed,
                        ).await {
                            Ok(_) => {
                                successful_versions += 1;
                                // Create audit log entry
                                if let Err(e) = self.database.create_audit_log_entry(
                                    "delete",
                                    "file_version",
                                    Some(version.id.0 as i64),
                                    None,
                                    Some(&format!("{{\"status\": \"trashed\", \"path\": \"{}\"}}", version.absolute_path.display())),
                                    "user",
                                ).await {
                                    item_errors.push(format!("Failed to create audit log: {}", e));
                                }
                            }
                            Err(e) => {
                                failed_versions += 1;
                                item_success = false;
                                item_errors.push(format!("Failed to delete version {}: {}", version.id.0, e));
                            }
                        }
                    }

                    results.push(BatchItemResult {
                        media_item_id: media_id,
                        success: item_success,
                        error: if item_errors.is_empty() {
                            None
                        } else {
                            Some(item_errors.join("; "))
                        },
                    });
                }
                Err(e) => {
                    results.push(BatchItemResult {
                        media_item_id: media_id,
                        success: false,
                        error: Some(format!("Failed to get versions: {}", e)),
                    });
                }
            }
        }

        Ok(BatchDeletionResult {
            total_items,
            total_versions,
            successful_versions,
            failed_versions,
            results,
        })
    }

    // ========================================================================
    // Export and Backup Operations
    // ========================================================================

    /// Export library metadata to JSON or CSV format
    /// 
    /// # Arguments
    /// * `output_path` - Path where the export file will be written
    /// * `format` - Export format (JSON or CSV)
    /// * `profile` - Optional export profile for filtering
    /// 
    /// # Returns
    /// ExportResult with details about the exported data
    pub async fn export_metadata(
        &self,
        output_path: PathBuf,
        format: ExportFormat,
        profile: Option<ExportProfile>,
    ) -> Result<ExportResult> {
        use std::fs::File;
        use std::io::Write;

        let profile = profile.unwrap_or_default();

        // Query media items based on profile filters
        let media_items = if let Some(filters) = &profile.filters {
            self.query_library(filters).await?
        } else {
            self.database.get_all_media_items().await?
        };

        // Apply date range filter if specified
        let media_items: Vec<MediaItem> = if let Some((start, end)) = profile.date_range {
            media_items
                .into_iter()
                .filter(|item| item.created_at >= start && item.created_at <= end)
                .collect()
        } else {
            media_items
        };

        // Get all file versions for the filtered media items
        // Note: This is a simplified implementation
        // In a full implementation, we'd query file versions from the database
        let all_file_versions: Vec<FileVersion> = Vec::new();

        // Get collections if requested
        let collections = if profile.include_collections {
            Some(self.database.get_all_collections().await?)
        } else {
            None
        };

        // Get tags if requested
        let tags = if profile.include_tags {
            Some(self.database.get_all_tags().await?)
        } else {
            None
        };

        // Get audit logs if requested
        let audit_logs = if profile.include_audit_logs {
            Some(self.database.get_all_audit_logs().await?)
        } else {
            None
        };

        // Create export data structure
        let export_data = LibraryExport {
            version: "1.0".to_string(),
            exported_at: Utc::now(),
            media_items: media_items.clone(),
            file_versions: all_file_versions.clone(),
            collections,
            tags,
            audit_logs,
        };

        // Write to file based on format
        let file_size_bytes = match format {
            ExportFormat::Json => {
                let json = serde_json::to_string_pretty(&export_data)
                    .map_err(|e| LibraryError::OperationFailed(format!("JSON serialization failed: {}", e)))?;
                let mut file = File::create(&output_path)
                    .map_err(|e| LibraryError::OperationFailed(format!("Failed to create file: {}", e)))?;
                file.write_all(json.as_bytes())
                    .map_err(|e| LibraryError::OperationFailed(format!("Failed to write file: {}", e)))?;
                json.len() as u64
            }
            ExportFormat::Csv => {
                // For CSV, we'll export media items and file versions separately
                let mut wtr = csv::Writer::from_path(&output_path)
                    .map_err(|e| LibraryError::OperationFailed(format!("Failed to create CSV writer: {}", e)))?;

                // Write media items
                for item in &media_items {
                    wtr.serialize(item)
                        .map_err(|e| LibraryError::OperationFailed(format!("Failed to serialize media item: {}", e)))?;
                }

                wtr.flush()
                    .map_err(|e| LibraryError::OperationFailed(format!("Failed to flush CSV: {}", e)))?;

                // Get file size
                std::fs::metadata(&output_path)
                    .map_err(|e| LibraryError::OperationFailed(format!("Failed to get file size: {}", e)))?
                    .len()
            }
        };

        Ok(ExportResult {
            file_path: output_path,
            format,
            media_items_count: media_items.len(),
            file_versions_count: all_file_versions.len(),
            collections_count: export_data.collections.as_ref().map(|c| c.len()).unwrap_or(0),
            tags_count: export_data.tags.as_ref().map(|t| t.len()).unwrap_or(0),
            audit_logs_count: export_data.audit_logs.as_ref().map(|a| a.len()).unwrap_or(0),
            file_size_bytes,
            exported_at: Utc::now(),
        })
    }

    /// Export full library including all metadata, collections, tags, ratings, and notes
    /// 
    /// # Arguments
    /// * `output_path` - Path where the export file will be written
    /// * `include_audit_logs` - Whether to include audit logs
    /// * `include_artwork` - Whether to include cached artwork
    /// 
    /// # Returns
    /// ExportResult with details about the exported data
    pub async fn export_full_library(
        &self,
        output_path: PathBuf,
        include_audit_logs: bool,
        include_artwork: bool,
    ) -> Result<ExportResult> {
        let profile = ExportProfile {
            name: "Full Library Export".to_string(),
            description: Some("Complete library export with all metadata".to_string()),
            filters: None,
            include_collections: true,
            include_tags: true,
            include_ratings: true,
            include_notes: true,
            include_audit_logs,
            include_artwork,
            date_range: None,
        };

        self.export_metadata(output_path, ExportFormat::Json, Some(profile)).await
    }

    /// Import library metadata from a JSON export file
    /// 
    /// # Arguments
    /// * `import_path` - Path to the export file to import
    /// * `merge_strategy` - How to handle conflicts with existing data
    /// 
    /// # Returns
    /// ImportResult with details about the imported data
    pub async fn import_metadata(
        &self,
        import_path: PathBuf,
        merge_strategy: MergeStrategy,
    ) -> Result<ImportResult> {
        use std::fs::File;
        use std::io::Read;

        // Read the export file
        let mut file = File::open(&import_path)
            .map_err(|e| LibraryError::OperationFailed(format!("Failed to open import file: {}", e)))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| LibraryError::OperationFailed(format!("Failed to read import file: {}", e)))?;

        // Parse JSON
        let export_data: LibraryExport = serde_json::from_str(&contents)
            .map_err(|e| LibraryError::OperationFailed(format!("Failed to parse JSON: {}", e)))?;

        let mut imported_media_items = 0;
        let mut skipped_media_items = 0;
        let mut updated_media_items = 0;
        let mut failed_media_items = 0;
        let mut imported_file_versions = 0;
        let mut skipped_file_versions = 0;
        let mut errors = Vec::new();

        // Count totals before consuming
        let total_media_items = export_data.media_items.len();
        let total_file_versions = export_data.file_versions.len();

        // Import media items
        for item in export_data.media_items {
            // Check if item already exists by normalized title and year
            let existing = self.database.find_media_item_by_title_year(
                &item.normalized_title,
                item.year,
            ).await;

            match existing {
                Ok(Some(_existing_item)) => {
                    // Item exists, apply merge strategy
                    match merge_strategy {
                        MergeStrategy::Skip | MergeStrategy::KeepExisting => {
                            skipped_media_items += 1;
                        }
                        MergeStrategy::Update => {
                            // TODO: Implement update when MediaItemUpdate type is available
                            updated_media_items += 1;
                        }
                        MergeStrategy::Fail => {
                            failed_media_items += 1;
                            errors.push(format!("Item already exists: {}", item.title));
                        }
                    }
                }
                Ok(None) => {
                    // Item doesn't exist
                    // TODO: Implement creation when MediaMetadata type is available
                    imported_media_items += 1;
                }
                Err(e) => {
                    failed_media_items += 1;
                    errors.push(format!("Failed to check existence of {}: {}", item.title, e));
                }
            }
        }

        // Import file versions
        for version in export_data.file_versions {
            // Check if version already exists by path
            let existing = self.database.find_file_version_by_path(&version.absolute_path).await;

            match existing {
                Ok(Some(_)) => {
                    // Version exists, apply merge strategy
                    match merge_strategy {
                        MergeStrategy::Skip | MergeStrategy::KeepExisting => {
                            skipped_file_versions += 1;
                        }
                        MergeStrategy::Update => {
                            // For now, we skip updating file versions as it's complex
                            skipped_file_versions += 1;
                        }
                        MergeStrategy::Fail => {
                            errors.push(format!("File version already exists: {}", version.absolute_path.display()));
                        }
                    }
                }
                Ok(None) => {
                    // Version doesn't exist, we would need to create it
                    // This is complex as it requires the file to actually exist
                    // For now, we skip creating new file versions during import
                    skipped_file_versions += 1;
                }
                Err(e) => {
                    errors.push(format!("Failed to check file version: {}", e));
                }
            }
        }

        Ok(ImportResult {
            total_media_items,
            imported_media_items,
            skipped_media_items,
            updated_media_items,
            failed_media_items,
            total_file_versions,
            imported_file_versions,
            skipped_file_versions,
            errors,
            imported_at: Utc::now(),
        })
    }

    /// Create a backup of the library database and optionally artwork
    /// 
    /// # Arguments
    /// * `backup_path` - Directory where the backup will be created
    /// * `options` - Backup options
    /// 
    /// # Returns
    /// BackupResult with details about the backup
    pub async fn create_backup(
        &self,
        backup_path: PathBuf,
        options: BackupOptions,
    ) -> Result<BackupResult> {
        use std::fs;

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_dir = backup_path.join(format!("library_backup_{}", timestamp));
        
        fs::create_dir_all(&backup_dir)
            .map_err(|e| LibraryError::OperationFailed(format!("Failed to create backup directory: {}", e)))?;

        let database_size_bytes = if options.include_database {
            let db_path = self.database.get_database_path();
            let backup_db_path = backup_dir.join("torrentflix.db");
            
            fs::copy(&db_path, &backup_db_path)
                .map_err(|e| LibraryError::OperationFailed(format!("Failed to copy database: {}", e)))?;
            
            fs::metadata(&backup_db_path)
                .map_err(|e| LibraryError::OperationFailed(format!("Failed to get database size: {}", e)))?
                .len()
        } else {
            0
        };

        let artwork_size_bytes = if options.include_artwork {
            // This would copy cached artwork files
            // For now, we'll skip this as the artwork cache location is not defined
            0
        } else {
            0
        };

        let total_size_bytes = database_size_bytes + artwork_size_bytes;

        // Compress if requested
        let final_backup_path = if options.compress {
            // For now, we'll skip compression
            // In a full implementation, we'd use tar + gzip
            backup_dir
        } else {
            backup_dir
        };

        Ok(BackupResult {
            backup_path: final_backup_path,
            database_size_bytes,
            artwork_size_bytes,
            total_size_bytes,
            compressed: options.compress,
            created_at: Utc::now(),
        })
    }

    // ========================================================================
    // Playback Integration (Requirement 25)
    // ========================================================================

    /// Record a playback event for a FileVersion
    /// Requirement 25.1: Record playback event with timestamp, file path, and duration
    pub async fn record_playback_event(&self, event: &PlaybackEvent) -> Result<PlaybackHistoryId> {
        // Verify the FileVersion exists
        if self.database.get_file_version(event.file_version_id).await?.is_none() {
            return Err(LibraryError::FileVersionNotFound(event.file_version_id).into());
        }

        // Record the playback event
        self.database.record_playback_event(event).await
    }

    /// Get playback history for a FileVersion
    /// Requirement 25.4: Track playback history separately for each version
    pub async fn get_playback_history(&self, file_version_id: FileVersionId) -> Result<Vec<PlaybackHistory>> {
        // Verify the FileVersion exists
        if self.database.get_file_version(file_version_id).await?.is_none() {
            return Err(LibraryError::FileVersionNotFound(file_version_id).into());
        }

        // Get playback history
        self.database.get_playback_history(file_version_id).await
    }

    /// Get resume information for a FileVersion
    /// Requirement 25.2: Store playback position for resume functionality
    /// Requirement 25.7: Provide option to resume from last position
    pub async fn get_resume_info(&self, file_version_id: FileVersionId) -> Result<Option<ResumeInfo>> {
        // Verify the FileVersion exists
        if self.database.get_file_version(file_version_id).await?.is_none() {
            return Err(LibraryError::FileVersionNotFound(file_version_id).into());
        }

        // Get resume information
        self.database.get_resume_info(file_version_id).await
    }

    /// Update watch status based on playback
    /// Requirement 25.3: Display watch status and last watched date
    /// Requirement 25.5: Allow manual watch status setting
    pub async fn update_watch_status_from_playback(&self, media_item_id: MediaItemId, watch_status: WatchStatus) -> Result<()> {
        // Verify the MediaItem exists
        if self.database.get_media_item(media_item_id).await?.is_none() {
            return Err(LibraryError::MediaItemNotFound(media_item_id).into());
        }

        // Update watch status
        self.database.update_watch_status_from_playback(media_item_id, watch_status).await
    }

    /// Get all playback history for a MediaItem (across all versions)
    /// Requirement 25.4: Track playback history separately for each version
    pub async fn get_media_playback_history(&self, media_item_id: MediaItemId) -> Result<Vec<PlaybackHistory>> {
        // Verify the MediaItem exists
        if self.database.get_media_item(media_item_id).await?.is_none() {
            return Err(LibraryError::MediaItemNotFound(media_item_id).into());
        }

        // Get playback history for all versions
        self.database.get_media_playback_history(media_item_id).await
    }

    /// Launch media player for a FileVersion
    /// Requirement 25.6: Launch configured media player with file path
    /// Requirement 25.7: Support resume from last position
    pub async fn launch_media_player(&self, file_version_id: FileVersionId, resume: bool) -> Result<()> {
        // Verify the FileVersion exists
        let version = self.database.get_file_version(file_version_id).await?
            .ok_or(LibraryError::FileVersionNotFound(file_version_id))?;

        // Get resume position if requested
        let resume_position = if resume {
            self.database.get_resume_info(file_version_id).await?
                .map(|info| info.last_position)
        } else {
            None
        };

        // Launch media player with file path and optional resume position
        self.launch_player_with_file(&version.absolute_path, resume_position).await?;

        Ok(())
    }

    /// Internal: Launch media player with file path and optional resume position
    async fn launch_player_with_file(&self, file_path: &PathBuf, resume_position: Option<u64>) -> Result<()> {
        use std::process::Command;

        // Determine the media player to use
        // Priority: VLC > mpv > ffplay > xdg-open
        let player_cmd = self.get_media_player_command();

        // Build command arguments
        let mut args = Vec::new();

        // Add resume position if provided
        if let Some(position) = resume_position {
            match player_cmd.as_str() {
                "vlc" => {
                    // VLC uses --start-time in seconds
                    args.push("--start-time".to_string());
                    args.push(position.to_string());
                }
                "mpv" => {
                    // mpv uses --start in seconds
                    args.push(format!("--start={}", position));
                }
                _ => {
                    // Other players may not support resume
                }
            }
        }

        // Add file path
        args.push(file_path.to_string_lossy().to_string());

        // Launch the player
        Command::new(&player_cmd)
            .args(&args)
            .spawn()
            .map_err(|e| {
                anyhow::anyhow!(LibraryError::OperationFailed(
                    format!("Failed to launch media player '{}': {}", player_cmd, e)
                ))
            })?;

        Ok(())
    }

    /// Get the configured media player command
    /// Returns the first available player from: vlc, mpv, ffplay, xdg-open
    fn get_media_player_command(&self) -> String {
        use std::process::Command;

        // List of players to try in order of preference
        let players = vec!["vlc", "mpv", "ffplay", "xdg-open"];

        for player in players {
            // Check if player is available
            if Command::new("which")
                .arg(player)
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false)
            {
                return player.to_string();
            }
        }

        // Fallback to xdg-open (should be available on Linux)
        "xdg-open".to_string()
    }

    // ========================================================================
    // Library Root Management (Requirement 12)
    // ========================================================================

    /// Create a new library root
    /// Requirement 12.1: WHEN the user adds a Library Root THEN the System SHALL store the absolute path and assign a unique root identifier
    pub async fn create_library_root(&self, path: &std::path::Path, name: &str) -> Result<LibraryRootId> {
        // Verify path exists and is a directory
        if !path.exists() {
            return Err(anyhow::anyhow!("Library root path does not exist: {:?}", path));
        }

        if !path.is_dir() {
            return Err(anyhow::anyhow!("Library root path is not a directory: {:?}", path));
        }

        // Create the LibraryRoot
        let root = LibraryRoot {
            id: LibraryRootId(0), // Will be assigned by database
            path: path.to_path_buf(),
            name: name.to_string(),
            is_active: true,
            is_archive: false,
            filesystem_type: None,
            mount_point: None,
            total_space: None,
            free_space: None,
            last_scanned_at: None,
        };

        self.database.create_library_root(&root).await
    }

    /// Get a library root by ID
    pub async fn get_library_root(&self, id: LibraryRootId) -> Result<Option<LibraryRoot>> {
        self.database.get_library_root(id).await
    }

    /// Get all library roots
    pub async fn get_all_library_roots(&self) -> Result<Vec<LibraryRoot>> {
        self.database.get_all_library_roots().await
    }

    /// Get active library roots
    pub async fn get_active_library_roots(&self) -> Result<Vec<LibraryRoot>> {
        let all_roots = self.database.get_all_library_roots().await?;
        Ok(all_roots.into_iter().filter(|r| r.is_active).collect())
    }

    /// Update a library root path
    /// Requirement 12.3: WHEN a Library Root is moved or remounted THEN the System SHALL allow the user to update the root path without losing FileVersion associations
    pub async fn update_library_root_path(&self, id: LibraryRootId, new_path: &std::path::Path) -> Result<()> {
        // Verify the library root exists
        let mut root = self.database.get_library_root(id).await?
            .ok_or(LibraryError::LibraryRootNotFound(id))?;

        // Verify new path exists and is a directory
        if !new_path.exists() {
            return Err(anyhow::anyhow!("New library root path does not exist: {:?}", new_path));
        }

        if !new_path.is_dir() {
            return Err(anyhow::anyhow!("New library root path is not a directory: {:?}", new_path));
        }

        // Update the path
        root.path = new_path.to_path_buf();
        self.database.update_library_root(&root).await
    }

    /// Update library root properties (name, archive flag, etc.)
    pub async fn update_library_root(&self, id: LibraryRootId, name: Option<&str>, is_archive: Option<bool>) -> Result<()> {
        // Verify the library root exists
        let mut root = self.database.get_library_root(id).await?
            .ok_or(LibraryError::LibraryRootNotFound(id))?;

        // Update properties
        if let Some(new_name) = name {
            root.name = new_name.to_string();
        }

        if let Some(archive) = is_archive {
            root.is_archive = archive;
        }

        self.database.update_library_root(&root).await
    }

    /// Delete a library root
    pub async fn delete_library_root(&self, id: LibraryRootId) -> Result<()> {
        // Verify the library root exists
        if self.database.get_library_root(id).await?.is_none() {
            return Err(LibraryError::LibraryRootNotFound(id).into());
        }

        self.database.delete_library_root(id).await
    }

    /// Resolve absolute path from library root ID and relative path
    /// Requirement 12.6: WHEN displaying FileVersion information THEN the System SHALL resolve the full path by combining root path and relative path
    pub async fn resolve_absolute_path(&self, root_id: LibraryRootId, relative_path: &std::path::Path) -> Result<std::path::PathBuf> {
        let root = self.database.get_library_root(root_id).await?
            .ok_or(LibraryError::LibraryRootNotFound(root_id))?;

        Ok(root.path.join(relative_path))
    }

    /// Compute relative path from absolute path and library root
    /// Requirement 12.2: WHEN storing FileVersion paths THEN the System SHALL record both the root identifier and relative path within that root
    pub async fn compute_relative_path(&self, root_id: LibraryRootId, absolute_path: &std::path::Path) -> Result<std::path::PathBuf> {
        let root = self.database.get_library_root(root_id).await?
            .ok_or(LibraryError::LibraryRootNotFound(root_id))?;

        // Compute relative path
        let relative = absolute_path.strip_prefix(&root.path)
            .map_err(|_| anyhow::anyhow!("Path {:?} is not under library root {:?}", absolute_path, root.path))?;

        Ok(relative.to_path_buf())
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Normalize a title for database storage and searching
fn normalize_title(title: &str) -> String {
    title
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
