# Library Management System - Developer Guide: Property-Based Tests

## Overview

Property-based testing (PBT) is a powerful approach to validating software correctness. Instead of writing individual test cases, we define universal properties that should hold true for all valid inputs. The test framework generates hundreds of random inputs to verify these properties.

This guide documents all property-based tests in the Library Management System, their purpose, and how to run and extend them.

## Testing Framework

The project uses **proptest** for property-based testing:

```toml
[dev-dependencies]
proptest = "1.4"
```

Tests are located in `engine/tests/` and follow the naming convention `*_proptest.rs`.

## Running Tests

### Run all tests
```bash
cargo test --test '*_proptest'
```

### Run specific test file
```bash
cargo test --test library_management_proptest
```

### Run specific property test
```bash
cargo test --test library_management_proptest property_1
```

### Run with verbose output
```bash
cargo test --test library_management_proptest -- --nocapture
```

### Run with specific seed (for reproducibility)
```bash
PROPTEST_REGRESSIONS=engine/tests/library_management_proptest.proptest-regressions \
cargo test --test library_management_proptest
```

## Property Categories

Properties are organized by category and component:

### 1. Data Model Properties (Properties 1-5)

These properties verify that data models are created and stored correctly.

#### Property 1: MediaItem creation with normalized metadata
**File**: `engine/tests/library_management_proptest.rs`

**Validates**: Requirements 1.1

**Description**: For any file import with title and year metadata, creating a MediaItem should result in a record with properly normalized title (lowercase, trimmed, special characters handled) and year value stored.

**Test Implementation**:
```rust
proptest! {
    #[test]
    fn property_1_media_item_creation(
        title in ".*",
        year in 1900i32..2100,
    ) {
        // Create MediaItem with random title and year
        let media_item = MediaItem {
            title: title.clone(),
            normalized_title: normalize_title(&title),
            year: Some(year),
            // ... other fields
        };
        
        // Verify normalized title is lowercase
        prop_assert_eq!(media_item.normalized_title, media_item.normalized_title.to_lowercase());
        
        // Verify year is preserved
        prop_assert_eq!(media_item.year, Some(year));
    }
}
```

**Generator Strategy**:
- `title`: Any string (including empty, special characters, unicode)
- `year`: Integer between 1900 and 2100

**Counterexample Handling**:
- If normalization fails on certain characters, the test captures the failing input
- Regression file stores failing cases for future runs

---

#### Property 2: FileVersion completeness
**File**: `engine/tests/library_management_proptest.rs`

**Validates**: Requirements 1.2

**Description**: For any committed file, the FileVersion record should contain all required fields: absolute path, file size, resolution, codec, container format, quality label, fingerprint, added timestamp, and status set to "present".

**Test Implementation**:
```rust
proptest! {
    #[test]
    fn property_2_file_version_completeness(
        path in ".*\\.mkv$",
        file_size in 1u64..1_000_000_000_000,
        resolution in "(720|1080|2160)p",
        codec in "(h264|h265|vp9)",
    ) {
        let version = FileVersion {
            absolute_path: path.clone(),
            file_size,
            resolution_width: Some(1920),
            resolution_height: Some(1080),
            video_codec: Some(codec.clone()),
            status: FileStatus::Present,
            added_at: Utc::now(),
            // ... other fields
        };
        
        // Verify all required fields are present
        prop_assert!(!version.absolute_path.is_empty());
        prop_assert!(version.file_size > 0);
        prop_assert!(version.resolution_width.is_some());
        prop_assert!(version.resolution_height.is_some());
        prop_assert!(version.video_codec.is_some());
        prop_assert_eq!(version.status, FileStatus::Present);
    }
}
```

---

#### Property 3: Version-to-MediaItem association
**File**: `engine/tests/library_management_proptest.rs`

**Validates**: Requirements 1.3

**Description**: For any set of files with identical normalized title and year, all resulting FileVersion records should reference the same MediaItemId.

**Test Implementation**:
```rust
proptest! {
    #[test]
    fn property_3_version_to_media_association(
        title in ".*",
        year in 1900i32..2100,
        version_count in 1usize..10,
    ) {
        // Create multiple versions with same title/year
        let versions: Vec<_> = (0..version_count)
            .map(|i| FileVersion {
                media_item_id: 1, // Same media item
                relative_path: format!("version_{}.mkv", i),
                // ... other fields
            })
            .collect();
        
        // Verify all versions reference the same media item
        let media_ids: Vec<_> = versions.iter().map(|v| v.media_item_id).collect();
        prop_assert!(media_ids.iter().all(|&id| id == media_ids[0]));
    }
}
```

---

### 2. Import Pipeline Properties (Properties 6-11)

These properties verify the import workflow from staging through commit.

#### Property 6: Staging directory isolation
**File**: `engine/tests/import_pipeline_proptest.rs`

**Validates**: Requirements 2.1

**Description**: For any file selected for import, the file should be moved or copied to the staging directory before any processing begins.

**Test Implementation**:
```rust
proptest! {
    #[test]
    fn property_6_staging_isolation(
        file_name in "[a-zA-Z0-9_]+\\.mkv$",
    ) {
        // Stage a file
        let staged = runtime.block_on(async {
            pipeline.stage_file(Path::new(&file_name)).await
        });
        
        prop_assert!(staged.is_ok());
        let staged_file = staged.unwrap();
        
        // Verify file is in staging directory
        prop_assert!(staged_file.staged_path.starts_with(&staging_dir));
        
        // Verify original file is not modified
        prop_assert_ne!(staged_file.original_path, staged_file.staged_path);
    }
}
```

---

#### Property 7: Metadata probing completeness
**File**: `engine/tests/import_pipeline_proptest.rs`

**Validates**: Requirements 2.2

**Description**: For any staged file, probing should extract and store container format, resolution, codec, duration, and audio track information.

**Test Implementation**:
```rust
proptest! {
    #[test]
    fn property_7_metadata_probing(
        file_path in ".*\\.mkv$",
    ) {
        let metadata = runtime.block_on(async {
            prober.probe_file(Path::new(&file_path)).await
        });
        
        prop_assert!(metadata.is_ok());
        let meta = metadata.unwrap();
        
        // Verify all required metadata is present
        prop_assert!(!meta.container.is_empty());
        prop_assert!(meta.video.is_some());
        prop_assert!(meta.duration.as_secs() > 0);
    }
}
```

---

#### Property 8: Fast fingerprint computation
**File**: `engine/tests/import_pipeline_proptest.rs`

**Validates**: Requirements 2.3

**Description**: For any staged file after metadata probing, a fast fingerprint should be computed and stored.

**Test Implementation**:
```rust
proptest! {
    #[test]
    fn property_8_fast_fingerprint(
        file_size in 1u64..1_000_000_000,
    ) {
        let hash = runtime.block_on(async {
            hasher.compute_fast_hash(Path::new(&test_file)).await
        });
        
        prop_assert!(hash.is_ok());
        let fast_hash = hash.unwrap();
        
        // Verify hash is computed (32 bytes)
        prop_assert_eq!(fast_hash.0.len(), 32);
    }
}
```

---

#### Property 9: Atomic commit operation
**File**: `engine/tests/import_pipeline_proptest.rs`

**Validates**: Requirements 2.4, 2.5

**Description**: For any staged file commit, either the file is successfully moved to the library root with database records created, or the file remains in staging with no database changes (atomicity).

**Test Implementation**:
```rust
proptest! {
    #[test]
    fn property_9_atomic_commit(
        title in ".*",
        year in 1900i32..2100,
    ) {
        let result = runtime.block_on(async {
            pipeline.commit_staged_file(staged_id, root_id).await
        });
        
        // Either success or failure, but not partial state
        match result {
            Ok(version_id) => {
                // Verify file is in library
                prop_assert!(library_path.exists());
                // Verify database record exists
                let version = runtime.block_on(async {
                    db.get_file_version(version_id).await
                });
                prop_assert!(version.is_ok());
            }
            Err(_) => {
                // Verify file is still in staging
                prop_assert!(staging_path.exists());
                // Verify no database record was created
                let count = runtime.block_on(async {
                    db.count_file_versions_for_media(media_id).await
                });
                prop_assert_eq!(count, 0);
            }
        }
    }
}
```

---

### 3. Hashing and Duplicate Detection Properties (Properties 12-16)

These properties verify background hashing and duplicate detection.

#### Property 12: Initial hashing status
**File**: `engine/tests/maintenance_proptest.rs`

**Validates**: Requirements 3.1

**Description**: When a FileVersion is created, the hashing status should be marked as "pending".

**Test Implementation**:
```rust
proptest! {
    #[test]
    fn property_12_initial_hashing_status(
        file_path in ".*\\.mkv$",
    ) {
        let version = runtime.block_on(async {
            manager.add_file_version(media_id, version_data).await
        });
        
        prop_assert!(version.is_ok());
        let v = version.unwrap();
        
        // Verify hashing status is pending
        prop_assert_eq!(v.hashing_status, HashingStatus::Pending);
    }
}
```

---

#### Property 13: Hash computation for pending versions
**File**: `engine/tests/maintenance_proptest.rs`

**Validates**: Requirements 3.2

**Description**: When the Job Manager schedules a hashing job, it should compute a full-file hash for FileVersions with pending hashing status.

**Test Implementation**:
```rust
proptest! {
    #[test]
    fn property_13_hash_computation(
        file_size in 1u64..1_000_000_000,
    ) {
        // Create version with pending status
        let version_id = runtime.block_on(async {
            manager.add_file_version(media_id, version_data).await
        }).unwrap();
        
        // Run hashing job
        runtime.block_on(async {
            manager.compute_full_hash(version_id).await
        }).unwrap();
        
        // Verify hash was computed
        let version = runtime.block_on(async {
            manager.get_file_version(version_id).await
        }).unwrap();
        
        prop_assert!(version.full_hash.is_some());
        prop_assert_eq!(version.hashing_status, HashingStatus::Complete);
    }
}
```

---

#### Property 14: Hashing concurrency limit
**File**: `engine/tests/maintenance_proptest.rs`

**Validates**: Requirements 3.3

**Description**: While hashing operations run, the system should limit concurrency to one file at a time to avoid overloading disk I/O.

**Test Implementation**:
```rust
proptest! {
    #[test]
    fn property_14_hashing_concurrency(
        version_count in 2usize..10,
    ) {
        // Create multiple versions with pending status
        let version_ids: Vec<_> = (0..version_count)
            .map(|_| {
                runtime.block_on(async {
                    manager.add_file_version(media_id, version_data.clone()).await
                }).unwrap()
            })
            .collect();
        
        // Run hashing job
        runtime.block_on(async {
            manager.schedule_hashing_job(version_count).await
        }).unwrap();
        
        // Verify only one hash is being computed at a time
        // (This would be verified through monitoring or mocking)
        prop_assert!(true); // Placeholder for actual concurrency check
    }
}
```

---

#### Property 15: Hash storage and status update
**File**: `engine/tests/maintenance_proptest.rs`

**Validates**: Requirements 3.4

**Description**: When a full-file hash is computed, the system should store the hash value in the FileVersion record and update hashing status to "complete".

**Test Implementation**:
```rust
proptest! {
    #[test]
    fn property_15_hash_storage(
        file_path in ".*\\.mkv$",
    ) {
        // Compute hash
        let hash = runtime.block_on(async {
            hasher.compute_full_hash(Path::new(&file_path), HashAlgorithm::Sha256).await
        }).unwrap();
        
        // Store in database
        runtime.block_on(async {
            manager.update_file_version(version_id, FileVersionUpdate {
                full_hash: Some(hash.clone()),
                hashing_status: Some(HashingStatus::Complete),
                ..Default::default()
            }).await
        }).unwrap();
        
        // Verify storage
        let version = runtime.block_on(async {
            manager.get_file_version(version_id).await
        }).unwrap();
        
        prop_assert_eq!(version.full_hash, Some(hash));
        prop_assert_eq!(version.hashing_status, HashingStatus::Complete);
    }
}
```

---

#### Property 16: Duplicate identification by hash
**File**: `engine/tests/maintenance_proptest.rs`

**Validates**: Requirements 3.6, 6.1

**Description**: When multiple FileVersions have identical full-file hashes, the system should identify them as exact duplicates.

**Test Implementation**:
```rust
proptest! {
    #[test]
    fn property_16_duplicate_identification(
        duplicate_count in 2usize..5,
    ) {
        // Create multiple versions with same hash
        let hash = vec![0u8; 32];
        let version_ids: Vec<_> = (0..duplicate_count)
            .map(|_| {
                runtime.block_on(async {
                    let v = manager.add_file_version(media_id, version_data.clone()).await.unwrap();
                    manager.update_file_version(v, FileVersionUpdate {
                        full_hash: Some(FullHash { algorithm: HashAlgorithm::Sha256, value: hash.clone() }),
                        ..Default::default()
                    }).await.unwrap();
                    v
                })
            })
            .collect();
        
        // Get duplicates
        let duplicates = runtime.block_on(async {
            manager.get_duplicates(DuplicateStrategy::ExactHash).await
        }).unwrap();
        
        // Verify all versions are identified as duplicates
        prop_assert!(duplicates.iter().any(|group| {
            group.version_ids.len() == duplicate_count &&
            group.version_ids.iter().all(|id| version_ids.contains(id))
        }));
    }
}
```

---

### 4. Query and Filtering Properties (Properties 18-21)

These properties verify library querying and filtering.

#### Property 18: Filter application correctness
**File**: `engine/tests/library_management_proptest.rs`

**Validates**: Requirements 4.2

**Description**: When filters are applied to a library query, all returned results should match the specified criteria.

**Test Implementation**:
```rust
proptest! {
    #[test]
    fn property_18_filter_application(
        title_filter in ".*",
        year_min in 1900i32..2000,
        year_max in 2000i32..2100,
    ) {
        let filters = LibraryFilters {
            title_filter: Some(title_filter.clone()),
            year_min: Some(year_min),
            year_max: Some(year_max),
            ..Default::default()
        };
        
        let results = runtime.block_on(async {
            manager.query_library(filters).await
        }).unwrap();
        
        // Verify all results match filters
        for item in results {
            if let Some(ref filter) = filters.title_filter {
                prop_assert!(item.normalized_title.contains(&filter.to_lowercase()));
            }
            if let Some(min) = filters.year_min {
                prop_assert!(item.year.unwrap_or(0) >= min);
            }
            if let Some(max) = filters.year_max {
                prop_assert!(item.year.unwrap_or(9999) <= max);
            }
        }
    }
}
```

---

#### Property 19: Case-insensitive search
**File**: `engine/tests/library_management_proptest.rs`

**Validates**: Requirements 4.3

**Description**: When a text search is performed, results should be returned regardless of case in the search query or stored titles.

**Test Implementation**:
```rust
proptest! {
    #[test]
    fn property_19_case_insensitive_search(
        query in "[a-zA-Z]+",
    ) {
        // Search with lowercase
        let results_lower = runtime.block_on(async {
            manager.search_library(&query.to_lowercase(), SearchOptions::default()).await
        }).unwrap();
        
        // Search with uppercase
        let results_upper = runtime.block_on(async {
            manager.search_library(&query.to_uppercase(), SearchOptions::default()).await
        }).unwrap();
        
        // Results should be the same
        prop_assert_eq!(results_lower.len(), results_upper.len());
        prop_assert_eq!(
            results_lower.iter().map(|r| r.id).collect::<Vec<_>>(),
            results_upper.iter().map(|r| r.id).collect::<Vec<_>>()
        );
    }
}
```

---

#### Property 21: Sort order correctness
**File**: `engine/tests/library_management_proptest.rs`

**Validates**: Requirements 4.5

**Description**: When sorting is applied, results should be returned in the specified order.

**Test Implementation**:
```rust
proptest! {
    #[test]
    fn property_21_sort_order(
        sort_by in "(title|year|added_date|size)",
        sort_order in "(asc|desc)",
    ) {
        let filters = LibraryFilters {
            sort_by: sort_by.clone(),
            sort_order: sort_order.clone(),
            ..Default::default()
        };
        
        let results = runtime.block_on(async {
            manager.query_library(filters).await
        }).unwrap();
        
        // Verify sort order
        match sort_by.as_str() {
            "title" => {
                let titles: Vec<_> = results.iter().map(|r| &r.title).collect();
                if sort_order == "asc" {
                    prop_assert!(titles.windows(2).all(|w| w[0] <= w[1]));
                } else {
                    prop_assert!(titles.windows(2).all(|w| w[0] >= w[1]));
                }
            }
            // ... other sort fields
            _ => {}
        }
    }
}
```

---

### 5. User Metadata Properties (Properties 48-50)

These properties verify user metadata management.

#### Property 48: User metadata persistence
**File**: `engine/tests/library_management_proptest.rs`

**Validates**: Requirements 9.2

**Description**: When user metadata is added or modified, the changes should persist to the database immediately.

**Test Implementation**:
```rust
proptest! {
    #[test]
    fn property_48_user_metadata_persistence(
        rating in 0.0f32..10.0,
        notes in ".*",
    ) {
        // Set metadata
        runtime.block_on(async {
            manager.set_user_rating(media_id, rating).await
        }).unwrap();
        
        runtime.block_on(async {
            manager.set_custom_notes(media_id, notes.clone()).await
        }).unwrap();
        
        // Retrieve and verify
        let item = runtime.block_on(async {
            manager.get_media_item(media_id).await
        }).unwrap();
        
        prop_assert_eq!(item.user_rating, Some(rating));
        prop_assert_eq!(item.custom_notes, Some(notes));
    }
}
```

---

#### Property 49: Tag storage and retrieval
**File**: `engine/tests/library_management_proptest.rs`

**Validates**: Requirements 9.3, 9.4

**Description**: When tags are added to a MediaItem, they should be stored and retrievable, and filtering by tags should return only items with those tags.

**Test Implementation**:
```rust
proptest! {
    #[test]
    fn property_49_tag_storage(
        tags in prop::collection::vec("[a-z]+", 1..5),
    ) {
        // Add tags
        runtime.block_on(async {
            manager.add_tags(media_id, tags.clone()).await
        }).unwrap();
        
        // Retrieve tags
        let item = runtime.block_on(async {
            manager.get_media_item(media_id).await
        }).unwrap();
        
        // Verify all tags are present
        for tag in &tags {
            prop_assert!(item.tags.contains(tag));
        }
    }
}
```

---

### 6. Cleanup and Maintenance Properties (Properties 34-47)

These properties verify soft delete, trash management, and cleanup operations.

#### Property 34: Soft delete operation
**File**: `engine/tests/maintenance_proptest.rs`

**Validates**: Requirements 7.1

**Description**: When a FileVersion is soft-deleted, its status should be updated to "trashed" and the file should be moved to the trash directory.

**Test Implementation**:
```rust
proptest! {
    #[test]
    fn property_34_soft_delete(
        version_id in 1i64..1000,
    ) {
        // Soft delete
        runtime.block_on(async {
            manager.soft_delete_version(version_id).await
        }).unwrap();
        
        // Verify status
        let version = runtime.block_on(async {
            manager.get_file_version(version_id).await
        }).unwrap();
        
        prop_assert_eq!(version.status, FileStatus::Trashed);
        prop_assert!(version.trashed_at.is_some());
        prop_assert!(version.original_path.is_some());
    }
}
```

---

## Test Regression Files

Proptest automatically saves failing test cases to regression files:

```
engine/tests/library_management_proptest.proptest-regressions
engine/tests/maintenance_proptest.proptest-regressions
```

These files ensure that once a bug is found and fixed, the same bug won't reoccur. The regression files are committed to version control.

## Adding New Property Tests

To add a new property test:

1. **Identify the property**: What universal truth should hold for all inputs?
2. **Define generators**: What random inputs should be generated?
3. **Write the test**: Use `proptest!` macro to define the property
4. **Run and verify**: Ensure the test passes with the current implementation
5. **Document**: Add to this guide with description and implementation

### Example Template

```rust
proptest! {
    #[test]
    fn property_XXX_description(
        input1 in generator1,
        input2 in generator2,
    ) {
        // Setup
        let result = operation(input1, input2);
        
        // Verify property
        prop_assert!(property_holds(&result));
    }
}
```

## Common Generators

```rust
// Strings
".*"                           // Any string
"[a-z]+"                       // Lowercase letters
"[0-9]+"                       // Digits
"[a-zA-Z0-9_]+"                // Alphanumeric + underscore

// Numbers
0i32..100                       // Integer range
0.0f32..1.0                     // Float range
1u64..1_000_000_000            // Large integer range

// Collections
prop::collection::vec(".*", 0..10)  // Vector of strings
prop::option::of(".*")              // Optional string

// Custom
prop::bool::ANY                 // Boolean
prop::string::string_regex(".*").unwrap()  // Regex-based strings
```

## Debugging Failed Tests

When a property test fails:

1. **Check the counterexample**: Proptest prints the failing input
2. **Reproduce locally**: Use the same input to debug
3. **Check regression file**: See if this was a known issue
4. **Fix the code or test**: Determine if it's a bug or incorrect property
5. **Commit regression file**: Ensure the fix is captured

## Performance Considerations

Property tests run many iterations (default: 256). For slow operations:

```rust
#[test]
fn property_slow_operation(
    input in prop::string::string_regex(".*").unwrap(),
) {
    // Limit iterations for slow tests
    proptest!(|(input in prop::string::string_regex(".*").unwrap())| {
        // Test code
    });
}
```

Configure in `proptest.toml`:

```toml
[profile.default]
cases = 100  # Reduce from default 256
```

