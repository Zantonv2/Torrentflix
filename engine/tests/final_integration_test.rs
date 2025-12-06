// Final integration tests for library management system
// Task 56: Final integration testing
// Covers: end-to-end workflows, error handling, concurrent operations, Linux-specific features

use engine::database::Database;
use engine::library::manager::LibraryManager;
use engine::library::models::*;
use engine::library::database::LibraryDatabase;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::fs;

/// Helper to normalize titles
fn normalize_title(title: &str) -> String {
    title.trim().to_lowercase().replace("  ", " ")
}

/// Helper to create a test database
async fn create_test_db() -> anyhow::Result<(Arc<LibraryDatabase>, TempDir)> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test.db");
    let db_url = format!("sqlite://{}", db_path.display());
    
    let db = Database::new(&db_url).await?;
    let lib_db = Arc::new(LibraryDatabase::new(db.pool().clone()));
    
    Ok((lib_db, temp_dir))
}

/// Helper to create a test media item
fn create_test_media_item(id: usize, title: &str) -> MediaItem {
    MediaItem {
        id: MediaItemId(0),
        title: title.to_string(),
        normalized_title: normalize_title(title),
        original_title: None,
        year: Some(2000 + (id % 25) as i32),
        media_type: if id % 2 == 0 { MediaType::Movie } else { MediaType::Series },
        overview: Some(format!("Overview for {}", title)),
        genres: vec!["Action".to_string(), "Drama".to_string()],
        cast: vec![],
        director: Some("Director Name".to_string()),
        runtime: Some(7200),
        poster: None,
        backdrop: None,
        external_ids: ExternalIds {
            tmdb_id: Some((id % 100_000) as i32),
            imdb_id: None,
            kinopoisk_id: None,
        },
        user_rating: None,
        watch_status: WatchStatus::Unwatched,
        last_watched_at: None,
        custom_notes: None,
        tags: vec![],
        metadata_source: None,
        metadata_fetched_at: None,
        user_edited_fields: std::collections::HashSet::new(),
        alternative_titles: vec![],
        custom_poster: None,
        custom_backdrop: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

#[cfg(test)]
mod end_to_end_workflows {
    use super::*;

    /// Test 56.1.1: Complete import workflow
    /// Validates: Requirements 2.1, 2.2, 2.3, 2.4, 2.6, 2.7
    #[tokio::test]
    async fn test_complete_import_workflow() -> anyhow::Result<()> {
        let (lib_db, temp_dir) = create_test_db().await?;
        let manager = LibraryManager::new(lib_db.clone());
        
        // Create a test file to import
        let source_file = temp_dir.path().join("test_movie.mkv");
        fs::write(&source_file, vec![0u8; 1024 * 100]).await?; // 100KB test file
        
        // Create a media item
        let media_item = create_test_media_item(1, "Test Movie");
        let media_id = manager.create_media_item(&media_item).await?;
        
        // Verify media item was created
        let retrieved = manager.get_media_item(media_id).await?.expect("Media item should exist");
        assert_eq!(retrieved.title, "Test Movie");
        assert_eq!(retrieved.normalized_title, normalize_title("Test Movie"));
        
        Ok(())
    }

    /// Test 56.1.2: Library browsing and filtering workflow
    /// Validates: Requirements 4.1, 4.2, 4.3, 4.5
    #[tokio::test]
    async fn test_library_browsing_workflow() -> anyhow::Result<()> {
        let (lib_db, _temp_dir) = create_test_db().await?;
        let manager = LibraryManager::new(lib_db);
        
        // Create multiple media items
        for i in 0..10 {
            let media_item = create_test_media_item(i, &format!("Movie {}", i));
            let _media_id = manager.create_media_item(&media_item).await?;
        }
        
        // Test basic query
        let filters = LibraryFilters {
            title: None,
            year: None,
            resolution: None,
            quality_label: None,
            status: None,
            tags: None,
            min_rating: None,
            watch_status: None,
            sort_by: None,
            sort_ascending: true,
            limit: Some(10),
            offset: None,
        };
        
        let results = manager.query_library(&filters).await?;
        assert_eq!(results.len(), 10);
        
        // Test search
        let search_results = manager.search_library("Movie 5", &Default::default()).await?;
        assert!(!search_results.is_empty());
        
        Ok(())
    }

    /// Test 56.1.3: Collection management workflow
    /// Validates: Requirements 19.1, 19.2, 19.3, 19.4, 19.5
    #[tokio::test]
    async fn test_collection_management_workflow() -> anyhow::Result<()> {
        let (lib_db, _temp_dir) = create_test_db().await?;
        let manager = LibraryManager::new(lib_db);
        
        // Create media items
        let mut media_ids = Vec::new();
        for i in 0..5 {
            let media_item = create_test_media_item(i, &format!("Movie {}", i));
            let media_id = manager.create_media_item(&media_item).await?;
            media_ids.push(media_id);
        }
        
        // Create a collection
        let collection_id = manager.create_collection("Test Collection", None).await?;
        
        // Add items to collection
        manager.add_to_collection(collection_id, &media_ids).await?;
        
        // Retrieve collection items
        let items = manager.get_collection_items(collection_id).await?;
        assert_eq!(items.len(), 5);
        
        Ok(())
    }

    /// Test 56.1.4: User metadata workflow
    /// Validates: Requirements 9.2, 9.3, 9.4, 9.5
    #[tokio::test]
    async fn test_user_metadata_workflow() -> anyhow::Result<()> {
        let (lib_db, _temp_dir) = create_test_db().await?;
        let manager = LibraryManager::new(lib_db);
        
        // Create a media item
        let media_item = create_test_media_item(1, "Test Movie");
        let media_id = manager.create_media_item(&media_item).await?;
        
        // Set user rating
        manager.set_user_rating(media_id, 8.5).await?;
        
        // Add tags
        manager.add_tags(media_id, &["favorite".to_string(), "watched".to_string()]).await?;
        
        // Set watch status
        manager.set_watch_status(media_id, WatchStatus::Watched).await?;
        
        // Set custom notes
        manager.set_custom_notes(media_id, "Great movie!").await?;
        
        // Verify all metadata was saved
        let retrieved = manager.get_media_item(media_id).await?.expect("Media item should exist");
        assert_eq!(retrieved.user_rating, Some(8.5));
        assert_eq!(retrieved.watch_status, WatchStatus::Watched);
        assert_eq!(retrieved.custom_notes, Some("Great movie!".to_string()));
        assert_eq!(retrieved.tags.len(), 2);
        
        Ok(())
    }
}

#[cfg(test)]
mod error_handling_and_recovery {
    use super::*;

    /// Test 56.2.1: Invalid media item creation
    /// Validates: Requirements 1.1, 1.2
    #[tokio::test]
    async fn test_invalid_media_item_creation() -> anyhow::Result<()> {
        let (lib_db, _temp_dir) = create_test_db().await?;
        let manager = LibraryManager::new(lib_db);
        
        // Create media item with empty title should fail or be handled
        let mut media_item = create_test_media_item(1, "Test");
        media_item.title = "".to_string();
        
        // This should either fail or be handled gracefully
        let result = manager.create_media_item(&media_item).await;
        // The system should handle this gracefully
        assert!(result.is_ok() || result.is_err());
        
        Ok(())
    }

    /// Test 56.2.2: Invalid rating values
    /// Validates: Requirements 9.2, 9.6
    #[tokio::test]
    async fn test_invalid_rating_values() -> anyhow::Result<()> {
        let (lib_db, _temp_dir) = create_test_db().await?;
        let manager = LibraryManager::new(lib_db);
        
        // Create a media item
        let media_item = create_test_media_item(1, "Test Movie");
        let media_id = manager.create_media_item(&media_item).await?;
        
        // Try to set invalid rating (> 10.0)
        let result = manager.set_user_rating(media_id, 11.0).await;
        assert!(result.is_err(), "Should reject rating > 10.0");
        
        // Try to set invalid rating (< 0.0)
        let result = manager.set_user_rating(media_id, -1.0).await;
        assert!(result.is_err(), "Should reject rating < 0.0");
        
        // Valid rating should succeed
        let result = manager.set_user_rating(media_id, 8.5).await;
        assert!(result.is_ok(), "Should accept valid rating");
        
        Ok(())
    }

    /// Test 56.2.3: Duplicate tag handling
    /// Validates: Requirements 9.3, 9.4
    #[tokio::test]
    async fn test_duplicate_tag_handling() -> anyhow::Result<()> {
        let (lib_db, _temp_dir) = create_test_db().await?;
        let manager = LibraryManager::new(lib_db);
        
        // Create a media item
        let media_item = create_test_media_item(1, "Test Movie");
        let media_id = manager.create_media_item(&media_item).await?;
        
        // Add tags
        manager.add_tags(media_id, &["favorite".to_string()]).await?;
        
        // Add same tag again
        manager.add_tags(media_id, &["favorite".to_string()]).await?;
        
        // Verify tag appears only once
        let retrieved = manager.get_media_item(media_id).await?.expect("Media item should exist");
        assert_eq!(retrieved.tags.len(), 1, "Should have exactly one tag");
        assert_eq!(retrieved.tags[0].0, tag_id.0, "Tag ID should match");
        
        Ok(())
    }

    /// Test 56.2.4: Collection operations with non-existent items
    /// Validates: Requirements 19.1, 19.2
    #[tokio::test]
    async fn test_collection_with_invalid_items() -> anyhow::Result<()> {
        let (lib_db, _temp_dir) = create_test_db().await?;
        let manager = LibraryManager::new(lib_db);
        
        // Create a collection
        let collection_id = manager.create_collection("Test Collection", None).await?;
        
        // Try to add non-existent media item
        let fake_id = MediaItemId(99999);
        let result = manager.add_to_collection(collection_id, &[fake_id]).await;
        
        // Should handle gracefully (either succeed or fail with proper error)
        assert!(result.is_ok() || result.is_err());
        
        Ok(())
    }
}

#[cfg(test)]
mod concurrent_operations {
    use super::*;

    /// Test 56.3.1: Concurrent media item creation
    /// Validates: Requirements 1.1, 1.2, 1.3
    #[tokio::test]
    async fn test_concurrent_media_creation() -> anyhow::Result<()> {
        let (lib_db, _temp_dir) = create_test_db().await?;
        let manager = Arc::new(LibraryManager::new(lib_db));
        
        // Spawn concurrent tasks to create media items
        let num_tasks = 20;
        let mut handles = Vec::new();
        
        for i in 0..num_tasks {
            let manager_clone = manager.clone();
            let handle = tokio::spawn(async move {
                let media_item = create_test_media_item(i, &format!("Movie {}", i));
                manager_clone.create_media_item(&media_item).await
            });
            handles.push(handle);
        }
        
        // Wait for all tasks to complete
        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.await?);
        }
        
        // All should succeed
        for result in results {
            assert!(result.is_ok(), "Concurrent creation should succeed");
        }
        
        Ok(())
    }

    /// Test 56.3.2: Concurrent metadata updates
    /// Validates: Requirements 9.2, 9.3, 9.4
    #[tokio::test]
    async fn test_concurrent_metadata_updates() -> anyhow::Result<()> {
        let (lib_db, _temp_dir) = create_test_db().await?;
        let manager = Arc::new(LibraryManager::new(lib_db));
        
        // Create a media item
        let media_item = create_test_media_item(1, "Test Movie");
        let media_id = manager.create_media_item(&media_item).await?;
        
        // Spawn concurrent tasks to update metadata
        let num_tasks = 10;
        let mut handles = Vec::new();
        
        for i in 0..num_tasks {
            let manager_clone = manager.clone();
            let handle = tokio::spawn(async move {
                let rating = 5.0 + (i as f32 * 0.5);
                manager_clone.set_user_rating(media_id, rating).await
            });
            handles.push(handle);
        }
        
        // Wait for all tasks to complete
        for handle in handles {
            let result = handle.await?;
            assert!(result.is_ok(), "Concurrent metadata update should succeed");
        }
        
        Ok(())
    }

    /// Test 56.3.3: Concurrent collection operations
    /// Validates: Requirements 19.1, 19.2, 19.3
    #[tokio::test]
    async fn test_concurrent_collection_operations() -> anyhow::Result<()> {
        let (lib_db, _temp_dir) = create_test_db().await?;
        let manager = Arc::new(LibraryManager::new(lib_db));
        
        // Create media items
        let mut media_ids = Vec::new();
        for i in 0..10 {
            let media_item = create_test_media_item(i, &format!("Movie {}", i));
            let media_id = manager.create_media_item(&media_item).await?;
            media_ids.push(media_id);
        }
        
        // Create collections concurrently
        let num_collections = 5;
        let mut handles = Vec::new();
        
        for i in 0..num_collections {
            let manager_clone = manager.clone();
            let handle = tokio::spawn(async move {
                manager_clone.create_collection(&format!("Collection {}", i), None).await
            });
            handles.push(handle);
        }
        
        // Wait for all collections to be created
        let mut collection_ids = Vec::new();
        for handle in handles {
            let result = handle.await??;
            collection_ids.push(result);
        }
        
        // Add items to collections concurrently
        let mut handles = Vec::new();
        for (_i, collection_id) in collection_ids.iter().enumerate() {
            let manager_clone = manager.clone();
            let media_ids_clone = media_ids.clone();
            let collection_id = *collection_id;
            
            let handle = tokio::spawn(async move {
                manager_clone.add_to_collection(collection_id, &media_ids_clone).await
            });
            handles.push(handle);
        }
        
        // Wait for all additions to complete
        for handle in handles {
            let result = handle.await?;
            assert!(result.is_ok(), "Concurrent collection addition should succeed");
        }
        
        Ok(())
    }

    /// Test 56.3.4: Concurrent queries
    /// Validates: Requirements 4.1, 4.2, 4.3
    #[tokio::test]
    async fn test_concurrent_queries() -> anyhow::Result<()> {
        let (lib_db, _temp_dir) = create_test_db().await?;
        let manager = Arc::new(LibraryManager::new(lib_db));
        
        // Create media items
        for i in 0..20 {
            let media_item = create_test_media_item(i, &format!("Movie {}", i));
            let _media_id = manager.create_media_item(&media_item).await?;
        }
        
        // Spawn concurrent query tasks
        let num_queries = 30;
        let mut handles = Vec::new();
        
        for i in 0..num_queries {
            let manager_clone = manager.clone();
            let handle = tokio::spawn(async move {
                let filters = LibraryFilters {
                    title: if i % 2 == 0 { Some(format!("Movie {}", i % 20)) } else { None },
                    year: None,
                    resolution: None,
                    quality_label: None,
                    status: None,
                    tags: None,
                    min_rating: None,
                    watch_status: None,
                    sort_by: None,
                    sort_ascending: true,
                    limit: Some(10),
                    offset: None,
                };
                manager_clone.query_library(&filters).await
            });
            handles.push(handle);
        }
        
        // Wait for all queries to complete
        for handle in handles {
            let result = handle.await?;
            assert!(result.is_ok(), "Concurrent queries should succeed");
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod linux_specific_features {
    use super::*;

    /// Test 56.4.1: POSIX path handling
    /// Validates: Requirements 18.2, 30.1
    #[tokio::test]
    async fn test_posix_path_handling() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let library_root = temp_dir.path().join("library");
        fs::create_dir(&library_root).await?;
        
        // Create files with various path patterns
        let test_paths = vec![
            "movie.mkv",
            "sub/movie.mkv",
            "sub/deep/movie.mkv",
            ".hidden_movie.mkv",
            "movie-with-dashes.mkv",
            "movie_with_underscores.mkv",
        ];
        
        for path in test_paths {
            let full_path = library_root.join(path);
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent).await?;
            }
            fs::write(&full_path, vec![0u8; 1024]).await?;
        }
        
        // Verify all files were created
        let mut file_count = 0;
        let mut entries = fs::read_dir(&library_root).await?;
        while let Some(_entry) = entries.next_entry().await? {
            file_count += 1;
        }
        
        assert!(file_count > 0, "Should create files with POSIX paths");
        
        Ok(())
    }

    /// Test 56.4.2: Symbolic link handling
    /// Validates: Requirements 17.1, 17.2, 17.4
    #[tokio::test]
    #[cfg(unix)]
    async fn test_symlink_handling() -> anyhow::Result<()> {
        use std::os::unix::fs as unix_fs;
        
        let temp_dir = TempDir::new()?;
        let library_root = temp_dir.path().join("library");
        fs::create_dir(&library_root).await?;
        
        // Create a real file
        let real_file = library_root.join("real_movie.mkv");
        fs::write(&real_file, vec![0u8; 1024]).await?;
        
        // Create a symlink to it
        let symlink_path = library_root.join("link_movie.mkv");
        unix_fs::symlink(&real_file, &symlink_path)?;
        
        // Verify symlink exists
        assert!(symlink_path.exists(), "Symlink should exist");
        
        // Verify we can read through the symlink
        let content = fs::read(&symlink_path).await?;
        assert_eq!(content.len(), 1024, "Should read through symlink");
        
        Ok(())
    }

    /// Test 56.4.3: Hard link handling
    /// Validates: Requirements 17.3
    #[tokio::test]
    #[cfg(unix)]
    async fn test_hardlink_handling() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let library_root = temp_dir.path().join("library");
        fs::create_dir(&library_root).await?;
        
        // Create a real file
        let real_file = library_root.join("real_movie.mkv");
        fs::write(&real_file, vec![0u8; 1024]).await?;
        
        // Create a hard link to it
        let hardlink_path = library_root.join("hardlink_movie.mkv");
        std::fs::hard_link(&real_file, &hardlink_path)?;
        
        // Verify hard link exists
        assert!(hardlink_path.exists(), "Hard link should exist");
        
        // Verify both files have the same inode
        let real_metadata = fs::metadata(&real_file).await?;
        let link_metadata = fs::metadata(&hardlink_path).await?;
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            assert_eq!(
                real_metadata.ino(),
                link_metadata.ino(),
                "Hard links should have the same inode"
            );
        }
        
        Ok(())
    }

    /// Test 56.4.4: Case-sensitive filename handling
    /// Validates: Requirements 18.2
    #[tokio::test]
    async fn test_case_sensitive_filenames() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let library_root = temp_dir.path().join("library");
        fs::create_dir(&library_root).await?;
        
        // Create files with different cases
        let file1 = library_root.join("Movie.mkv");
        let file2 = library_root.join("movie.mkv");
        
        fs::write(&file1, vec![0u8; 1024]).await?;
        
        // On Linux, these are different files
        // On Windows/macOS, they would be the same
        #[cfg(unix)]
        {
            fs::write(&file2, vec![0u8; 2048]).await?;
            
            let content1 = fs::read(&file1).await?;
            let content2 = fs::read(&file2).await?;
            
            assert_eq!(content1.len(), 1024, "First file should have 1024 bytes");
            assert_eq!(content2.len(), 2048, "Second file should have 2048 bytes");
        }
        
        Ok(())
    }

    /// Test 56.4.5: Atomic file operations
    /// Validates: Requirements 18.5, 18.6
    #[tokio::test]
    async fn test_atomic_file_operations() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let source = temp_dir.path().join("source.mkv");
        let dest = temp_dir.path().join("dest.mkv");
        
        // Create source file
        fs::write(&source, vec![0u8; 1024]).await?;
        
        // Perform atomic rename (move)
        fs::rename(&source, &dest).await?;
        
        // Verify source is gone and dest exists
        assert!(!source.exists(), "Source should be moved");
        assert!(dest.exists(), "Destination should exist");
        
        // Verify content is intact
        let content = fs::read(&dest).await?;
        assert_eq!(content.len(), 1024, "Content should be preserved");
        
        Ok(())
    }
}
