// Library query operations
// Provides high-level query interface for searching, filtering, and retrieving library data

use anyhow::Result;
use std::sync::Arc;
use super::database::LibraryDatabase;
use super::models::*;

/// Query builder for constructing dynamic SQL queries
pub struct QueryBuilder {
    select_clause: String,
    where_clauses: Vec<String>,
    join_clauses: Vec<String>,
    order_by_clause: Option<String>,
    limit_clause: Option<String>,
}

impl QueryBuilder {
    /// Create a new query builder for MediaItems
    pub fn new() -> Self {
        Self {
            select_clause: "SELECT DISTINCT m.* FROM media_items m".to_string(),
            where_clauses: Vec::new(),
            join_clauses: Vec::new(),
            order_by_clause: None,
            limit_clause: None,
        }
    }

    /// Add a WHERE clause condition
    pub fn where_condition(mut self, condition: &str) -> Self {
        self.where_clauses.push(condition.to_string());
        self
    }

    /// Add a JOIN clause
    pub fn join(mut self, join: &str) -> Self {
        self.join_clauses.push(join.to_string());
        self
    }

    /// Add ORDER BY clause
    pub fn order_by(mut self, field: &str, ascending: bool) -> Self {
        let direction = if ascending { "ASC" } else { "DESC" };
        self.order_by_clause = Some(format!("ORDER BY {} {}", field, direction));
        self
    }

    /// Add LIMIT and OFFSET
    pub fn limit(mut self, limit: u32, offset: u32) -> Self {
        self.limit_clause = Some(format!("LIMIT {} OFFSET {}", limit, offset));
        self
    }

    /// Build the final SQL query
    pub fn build(self) -> String {
        let mut query = self.select_clause;

        // Add joins
        for join in self.join_clauses {
            query.push(' ');
            query.push_str(&join);
        }

        // Add WHERE clauses
        if !self.where_clauses.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&self.where_clauses.join(" AND "));
        }

        // Add ORDER BY
        if let Some(order_by) = self.order_by_clause {
            query.push(' ');
            query.push_str(&order_by);
        }

        // Add LIMIT
        if let Some(limit) = self.limit_clause {
            query.push(' ');
            query.push_str(&limit);
        }

        query
    }
}

impl Default for QueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Query interface for library operations
pub struct LibraryQueries {
    database: Arc<LibraryDatabase>,
}

impl LibraryQueries {
    /// Create a new LibraryQueries instance
    pub fn new(database: Arc<LibraryDatabase>) -> Self {
        Self { database }
    }

    // ========================================================================
    // Library Browsing and Filtering
    // ========================================================================

    /// Query library with filters, pagination, and sorting
    /// 
    /// By default, excludes trashed FileVersions from results (Requirement 7.4)
    pub async fn query_library(&self, filters: &LibraryFilters) -> Result<Vec<MediaItem>> {
        let mut builder = QueryBuilder::new();

        // Exclude trashed items by default (Requirement 7.4)
        // Only include MediaItems that have at least one non-trashed FileVersion
        // Use a subquery to handle cases where there are no file versions
        builder = builder
            .where_condition("(SELECT COUNT(*) FROM file_versions fv WHERE fv.media_item_id = m.id AND fv.status != 'trashed') > 0 OR (SELECT COUNT(*) FROM file_versions fv WHERE fv.media_item_id = m.id) = 0");

        // Add title filter
        if let Some(ref title) = filters.title {
            let normalized = normalize_title(title);
            builder = builder.where_condition(&format!("m.normalized_title LIKE '%{}%'", normalized));
        }

        // Add year filter
        if let Some(year) = filters.year {
            builder = builder.where_condition(&format!("m.year = {}", year));
        }

        // Add resolution filter
        if let Some(resolution) = filters.resolution {
            builder = builder
                .join("LEFT JOIN file_versions fv ON m.id = fv.media_item_id")
                .where_condition(&format!(
                    "fv.resolution_width = {} AND fv.resolution_height = {}",
                    resolution.width, resolution.height
                ));
        }

        // Add quality label filter
        if let Some(ref quality) = filters.quality_label {
            builder = builder
                .join("LEFT JOIN file_versions fv ON m.id = fv.media_item_id")
                .where_condition(&format!("fv.quality_label = '{}'", quality));
        }

        // Add status filter
        if let Some(status) = filters.status {
            let status_str = match status {
                FileStatus::Present => "present",
                FileStatus::Missing => "missing",
                FileStatus::Trashed => "trashed",
                FileStatus::Corrupted => "corrupted",
            };
            builder = builder
                .join("LEFT JOIN file_versions fv ON m.id = fv.media_item_id")
                .where_condition(&format!("fv.status = '{}'", status_str));
        }

        // Add tags filter
        if let Some(ref tags) = filters.tags {
            if !tags.is_empty() {
                let tag_ids: Vec<String> = tags.iter().map(|t| t.0.to_string()).collect();
                builder = builder
                    .join("LEFT JOIN media_item_tags mit ON m.id = mit.media_item_id")
                    .where_condition(&format!("mit.tag_id IN ({})", tag_ids.join(",")));
            }
        }

        // Add rating filter
        if let Some(min_rating) = filters.min_rating {
            builder = builder.where_condition(&format!("m.user_rating >= {}", min_rating));
        }

        // Add watch status filter
        if let Some(watch_status) = filters.watch_status {
            let status_str = match watch_status {
                WatchStatus::Unwatched => "unwatched",
                WatchStatus::InProgress => "in_progress",
                WatchStatus::Watched => "watched",
            };
            builder = builder.where_condition(&format!("m.watch_status = '{}'", status_str));
        }

        // Add sorting
        if let Some(sort_field) = filters.sort_by {
            let sort_column = match sort_field {
                SortField::Title => "m.title",
                SortField::Year => "m.year",
                SortField::AddedDate => "m.created_at",
                SortField::Size => "(SELECT SUM(fv.file_size) FROM file_versions fv WHERE fv.media_item_id = m.id)",
                SortField::VersionCount => "(SELECT COUNT(*) FROM file_versions fv WHERE fv.media_item_id = m.id)",
                SortField::UserRating => "m.user_rating",
            };
            builder = builder.order_by(sort_column, filters.sort_ascending);
        }

        // Add pagination
        let limit = filters.limit.unwrap_or(100);
        let offset = filters.offset.unwrap_or(0);
        builder = builder.limit(limit, offset);

        let query = builder.build();
        self.database.query_media_items(&query).await
    }

    /// Search library by text with optional fuzzy matching
    /// Searches across title, original_title, cast names, director names, and custom notes
    pub async fn search_library(&self, query: &str, options: &SearchOptions) -> Result<Vec<MediaItem>> {
        // Normalize the search query
        let normalized_query = normalize_title(query);

        // If fuzzy matching is enabled, we need to fetch more results and filter by edit distance
        if options.fuzzy {
            // For fuzzy search, get all items (or a large subset) and filter by edit distance
            let mut builder = QueryBuilder::new();
            
            // Add pagination with a larger limit for fuzzy search
            let limit = options.limit.unwrap_or(100).max(100); // At least 100 for fuzzy
            let offset = options.offset.unwrap_or(0);
            builder = builder.limit(limit, offset);
            
            let query_str = builder.build();
            let all_results = self.database.query_media_items(&query_str).await?;
            
            // Calculate similarity scores for all results
            let mut scored_results: Vec<(MediaItem, usize, f32)> = all_results
                .into_iter()
                .map(|item| {
                    let dist = levenshtein_distance(&normalize_title(&item.title), &normalized_query);
                    // Calculate similarity score (0.0 = no match, 1.0 = perfect match)
                    let max_len = item.title.len().max(query.len());
                    let similarity = if max_len == 0 {
                        1.0
                    } else {
                        1.0 - (dist as f32 / max_len as f32)
                    };
                    (item, dist, similarity)
                })
                .collect();

            // Filter out results with very low similarity (< 0.3)
            scored_results.retain(|(_, _, similarity)| *similarity >= 0.3);

            // Sort by distance (lower is better)
            scored_results.sort_by(|a, b| a.1.cmp(&b.1));

            // Extract just the MediaItems and apply the original limit
            let results: Vec<MediaItem> = scored_results
                .into_iter()
                .take(options.limit.unwrap_or(100) as usize)
                .map(|(item, _, _)| item)
                .collect();
            
            Ok(results)
        } else {
            // For non-fuzzy search, use LIKE queries
            let mut builder = QueryBuilder::new();

            // Build search condition for multiple fields
            let search_pattern = format!("%{}%", normalized_query);
            let case_insensitive_pattern = format!("%{}%", query.to_lowercase());

            // Search across multiple fields:
            // - normalized_title (normalized search)
            // - original_title (case-insensitive)
            // - cast (JSON field, case-insensitive)
            // - director (case-insensitive)
            // - custom_notes (case-insensitive)
            // - overview (case-insensitive)
            builder = builder.where_condition(&format!(
                "(m.normalized_title LIKE '{}' OR \
                 LOWER(m.original_title) LIKE '{}' OR \
                 LOWER(m.cast) LIKE '{}' OR \
                 LOWER(m.director) LIKE '{}' OR \
                 LOWER(m.custom_notes) LIKE '{}' OR \
                 LOWER(m.overview) LIKE '{}')",
                search_pattern, 
                case_insensitive_pattern, 
                case_insensitive_pattern, 
                case_insensitive_pattern, 
                case_insensitive_pattern,
                case_insensitive_pattern
            ));

            // Add sorting by relevance (title matches first)
            builder = builder.order_by("m.normalized_title", true);

            // Add pagination
            let limit = options.limit.unwrap_or(100);
            let offset = options.offset.unwrap_or(0);
            builder = builder.limit(limit, offset);

            let query_str = builder.build();
            self.database.query_media_items(&query_str).await
        }
    }

    /// Get all MediaItems
    pub async fn get_all_media_items(&self) -> Result<Vec<MediaItem>> {
        // TODO: Implement query
        unimplemented!("get_all_media_items")
    }

    /// Get MediaItems by tag
    pub async fn get_media_items_by_tag(&self, _tag_id: TagId) -> Result<Vec<MediaItem>> {
        // TODO: Implement query
        unimplemented!("get_media_items_by_tag")
    }

    /// Search MediaItems by custom notes
    pub async fn search_by_notes(&self, query: &str, options: &SearchOptions) -> Result<Vec<MediaItem>> {
        let mut builder = QueryBuilder::new();

        // Build search condition for custom notes field
        let search_pattern = format!("%{}%", query);
        builder = builder.where_condition(&format!("m.custom_notes LIKE '{}'", search_pattern));

        // Add sorting by relevance
        builder = builder.order_by("m.updated_at", false);

        // Add pagination
        let limit = options.limit.unwrap_or(100);
        let offset = options.offset.unwrap_or(0);
        builder = builder.limit(limit, offset);

        let query_str = builder.build();
        self.database.query_media_items(&query_str).await
    }

    /// Get MediaItems by collection
    pub async fn get_media_items_by_collection(&self, _collection_id: CollectionId) -> Result<Vec<MediaItem>> {
        // TODO: Implement query
        unimplemented!("get_media_items_by_collection")
    }

    // ========================================================================
    // Duplicate and Variant Detection
    // ========================================================================

    /// Get duplicate groups by hash
    /// Delegates to database layer for duplicate detection
    pub async fn get_duplicates(&self) -> Result<Vec<DuplicateGroup>> {
        self.database.get_duplicates().await
    }

    /// Get variants for a MediaItem
    /// Returns all FileVersions for the same MediaItem (different versions of same media)
    pub async fn get_variants(&self, media_id: MediaItemId) -> Result<Vec<FileVersion>> {
        self.database.get_file_versions_for_media(media_id).await
    }

    // ========================================================================
    // Cleanup and Maintenance
    // ========================================================================

    /// Get cleanup candidates
    pub async fn get_cleanup_candidates(&self) -> Result<Vec<CleanupCandidate>> {
        // TODO: Implement cleanup candidate identification
        unimplemented!("get_cleanup_candidates")
    }

    /// Get trashed FileVersions
    pub async fn get_trashed_versions(&self) -> Result<Vec<FileVersion>> {
        // TODO: Implement query
        unimplemented!("get_trashed_versions")
    }

    /// Get FileVersions with pending hashing
    pub async fn get_pending_hashing(&self) -> Result<Vec<FileVersion>> {
        // TODO: Implement query
        unimplemented!("get_pending_hashing")
    }

    /// Get FileVersions with missing files
    pub async fn get_missing_versions(&self) -> Result<Vec<FileVersion>> {
        // TODO: Implement query
        unimplemented!("get_missing_versions")
    }

    // ========================================================================
    // Storage Analytics
    // ========================================================================

    /// Get storage statistics
    pub async fn get_storage_stats(&self) -> Result<StorageStats> {
        // TODO: Implement statistics calculation
        unimplemented!("get_storage_stats")
    }

    /// Get storage breakdown by resolution
    pub async fn get_storage_by_resolution(&self) -> Result<Vec<(String, u64)>> {
        // TODO: Implement breakdown query
        unimplemented!("get_storage_by_resolution")
    }

    /// Get storage breakdown by quality label
    pub async fn get_storage_by_quality(&self) -> Result<Vec<(String, u64)>> {
        // TODO: Implement breakdown query
        unimplemented!("get_storage_by_quality")
    }

    /// Get storage breakdown by status
    pub async fn get_storage_by_status(&self) -> Result<Vec<(String, u64)>> {
        // TODO: Implement breakdown query
        unimplemented!("get_storage_by_status")
    }

    /// Get duplicate space usage
    pub async fn get_duplicate_space(&self) -> Result<u64> {
        // TODO: Implement calculation
        unimplemented!("get_duplicate_space")
    }

    /// Get largest MediaItems by total size
    pub async fn get_largest_media(&self, _limit: u32) -> Result<Vec<(MediaItem, u64)>> {
        // TODO: Implement query
        unimplemented!("get_largest_media")
    }
}

/// Storage statistics
#[derive(Debug, Clone)]
pub struct StorageStats {
    pub total_size: u64,
    pub media_item_count: u32,
    pub file_version_count: u32,
    pub average_versions_per_media: f32,
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Normalize a title for searching
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

/// Calculate Levenshtein distance between two strings
/// Used for fuzzy matching in search
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();

    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if s1_chars[i - 1] == s2_chars[j - 1] { 0 } else { 1 };
            matrix[i][j] = std::cmp::min(
                std::cmp::min(
                    matrix[i - 1][j] + 1,      // deletion
                    matrix[i][j - 1] + 1,      // insertion
                ),
                matrix[i - 1][j - 1] + cost,   // substitution
            );
        }
    }

    matrix[len1][len2]
}
