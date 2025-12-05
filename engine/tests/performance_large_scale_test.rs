// Performance testing for large-scale library operations
// Task 55.1: Test with 10,000+ MediaItems
// Task 55.2: Profile and optimize slow queries
// Task 55.3: Optimize HDD I/O patterns
// Task 55.4: Test rescan performance on large directories

use engine::database::Database;
use engine::library::manager::LibraryManager;
use engine::library::models::*;
use engine::library::database::LibraryDatabase;
use std::sync::Arc;
use std::time::Instant;
use tempfile::TempDir;
use tokio::fs;

/// Helper to normalize titles for testing
fn normalize_title(title: &str) -> String {
    title
        .trim()
        .to_lowercase()
        .replace("  ", " ")
}

/// Helper to create a test database
async fn create_test_db() -> anyhow::Result<(Arc<LibraryDatabase>, TempDir)> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test.db");
    let db_url = format!("sqlite://{}", db_path.display());
    
    let db = Database::new(&db_url).await?;
    // Database initializes migrations automatically in the constructor
    
    let lib_db = Arc::new(LibraryDatabase::new(db.pool().clone()));
    
    Ok((lib_db, temp_dir))
}

/// Test 55.1: Load and query 10,000+ MediaItems
#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored test_large_scale_media_items_10k
async fn test_large_scale_media_items_10k() -> anyhow::Result<()> {
    let (lib_db, _temp_dir) = create_test_db().await?;
    let manager = LibraryManager::new(lib_db);
    
    let start = Instant::now();
    
    // Create 10,000 MediaItems
    println!("Creating 10,000 MediaItems...");
    let create_start = Instant::now();
    
    for i in 0..10_000 {
        if i % 1000 == 0 {
            println!("  Created {} items...", i);
        }
        
        let title = format!("Movie {}", i);
        let media_item = MediaItem {
            id: MediaItemId(0), // Will be assigned by database
            title: title.clone(),
            normalized_title: normalize_title(&title),
            original_title: None,
            year: Some(2000 + (i % 25) as i32),
            media_type: if i % 2 == 0 { MediaType::Movie } else { MediaType::Series },
            overview: Some(format!("Overview for movie {}", i)),
            genres: vec!["Action".to_string(), "Drama".to_string()],
            cast: vec![],
            director: Some("Director Name".to_string()),
            runtime: Some(7200),
            poster: None,
            backdrop: None,
            external_ids: ExternalIds {
                tmdb_id: Some((i % 100_000) as i32),
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
        };
        
        let _media_id = manager.create_media_item(&media_item).await?;
    }
    
    let create_duration = create_start.elapsed();
    println!("Created 10,000 MediaItems in {:?}", create_duration);
    println!("Average creation time per item: {:?}", create_duration / 10_000);
    
    let total_duration = start.elapsed();
    println!("\nTotal test duration: {:?}", total_duration);
    
    Ok(())
}

/// Test 55.2: Profile query performance with various filters
#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored test_query_performance_profiling
async fn test_query_performance_profiling() -> anyhow::Result<()> {
    let (lib_db, _temp_dir) = create_test_db().await?;
    let manager = LibraryManager::new(lib_db);
    
    // Create test data
    println!("Creating test data (5,000 items)...");
    for i in 0..5_000 {
        if i % 500 == 0 {
            println!("  Created {} items...", i);
        }
        
        let title = format!("Movie {}", i);
        let media_item = MediaItem {
            id: MediaItemId(0),
            title: title.clone(),
            normalized_title: normalize_title(&title),
            original_title: None,
            year: Some(2000 + (i % 25) as i32),
            media_type: if i % 2 == 0 { MediaType::Movie } else { MediaType::Series },
            overview: Some(format!("Overview for movie {}", i)),
            genres: vec!["Action".to_string(), "Drama".to_string()],
            cast: vec![],
            director: Some("Director Name".to_string()),
            runtime: Some(7200),
            poster: None,
            backdrop: None,
            external_ids: ExternalIds {
                tmdb_id: Some((i % 100_000) as i32),
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
        };
        
        let _media_id = manager.create_media_item(&media_item).await?;
    }
    
    println!("\nTest 1: Simple title query (100 iterations)...");
    let start = Instant::now();
    for _ in 0..100 {
        let filters = LibraryFilters {
            title: Some("Movie 100".to_string()),
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
        let _ = manager.query_library(&filters).await?;
    }
    let duration = start.elapsed();
    println!("100 title queries: {:?} (avg: {:?})", duration, duration / 100);
    
    println!("\nTest 2: Year filter query (100 iterations)...");
    let start = Instant::now();
    for _ in 0..100 {
        let filters = LibraryFilters {
            title: None,
            year: Some(2010),
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
        let _ = manager.query_library(&filters).await?;
    }
    let duration = start.elapsed();
    println!("100 year filter queries: {:?} (avg: {:?})", duration, duration / 100);
    
    Ok(())
}

/// Test 55.3: HDD I/O pattern optimization
#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored test_hdd_io_patterns
async fn test_hdd_io_patterns() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let library_root = temp_dir.path().join("library");
    fs::create_dir(&library_root).await?;
    
    // Create test files
    println!("Creating test files for I/O pattern testing...");
    let start = Instant::now();
    
    for i in 0..1000 {
        let file_path = library_root.join(format!("file_{}.mkv", i));
        fs::write(&file_path, vec![0u8; 1024 * 1024]).await?; // 1MB files
    }
    
    let create_duration = start.elapsed();
    println!("Created 1000 files (1GB total) in {:?}", create_duration);
    
    // Test sequential read
    println!("\nTest 1: Sequential read pattern...");
    let start = Instant::now();
    
    for i in 0..1000 {
        let file_path = library_root.join(format!("file_{}.mkv", i));
        let _ = fs::read(&file_path).await?;
    }
    
    let seq_read_duration = start.elapsed();
    println!("Sequential read of 1000 files: {:?}", seq_read_duration);
    
    // Test random read
    println!("\nTest 2: Random read pattern...");
    let start = Instant::now();
    
    for i in (0..1000).step_by(7) {
        let file_path = library_root.join(format!("file_{}.mkv", i));
        let _ = fs::read(&file_path).await?;
    }
    
    let random_read_duration = start.elapsed();
    println!("Random read of ~143 files: {:?}", random_read_duration);
    
    Ok(())
}

/// Test 55.4: Rescan performance on large directories
#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored test_rescan_performance_large_directories
async fn test_rescan_performance_large_directories() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let library_root = temp_dir.path().join("library");
    fs::create_dir(&library_root).await?;
    
    // Create directory structure with many files
    println!("Creating directory structure with 5,000 files...");
    let create_start = Instant::now();
    
    for i in 0..50 {
        let subdir = library_root.join(format!("dir_{}", i));
        fs::create_dir(&subdir).await?;
        
        for j in 0..100 {
            let file_path = subdir.join(format!("file_{}.mkv", j));
            fs::write(&file_path, vec![0u8; 100 * 1024]).await?; // 100KB files
        }
    }
    
    let create_duration = create_start.elapsed();
    println!("Created directory structure in {:?}", create_duration);
    
    // Test directory traversal
    println!("\nTest: Directory traversal performance...");
    let start = Instant::now();
    
    let mut file_count = 0;
    let mut dir_count = 0;
    
    // Use iterative approach instead of recursive to avoid boxing
    let mut dirs_to_process = vec![library_root.clone()];
    
    while let Some(current_dir) = dirs_to_process.pop() {
        let mut entries = fs::read_dir(&current_dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                dir_count += 1;
                dirs_to_process.push(path);
            } else {
                file_count += 1;
            }
        }
    }
    
    let traverse_duration = start.elapsed();
    println!("Traversed {} directories and {} files in {:?}", dir_count, file_count, traverse_duration);
    
    Ok(())
}
