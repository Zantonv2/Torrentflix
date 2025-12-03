// Integration tests for concurrent database scenarios
// Feature: database-race-condition-fix
// Task 8: Add integration tests for concurrent scenarios
// Requirements: 1.3, 4.3, 5.2

use anyhow::Result;
use engine::database::{Database, jobs::{JobDatabase, Job, JobStatus}, SettingsDatabase};
use engine::jobs::JobManager;
use engine::settings::{SettingsManager, Settings};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;

// Helper function to create a test database and settings manager
async fn create_test_settings_manager() -> Result<(SettingsManager, Arc<SettingsDatabase>, TempDir)> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test_concurrent.db");
    let db_url = format!("sqlite://{}", db_path.to_string_lossy());

    let db = Database::new(&db_url).await?;
    let settings_db = Arc::new(SettingsDatabase::new(Arc::new(db)));
    settings_db.initialize().await?;

    let manager = SettingsManager::new(settings_db.clone());
    Ok((manager, settings_db, temp_dir))
}

// Helper function to create a test database and job manager
async fn create_test_job_manager() -> Result<(Arc<JobManager>, Arc<Database>, TempDir)> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test_jobs.db");
    let db_url = format!("sqlite://{}", db_path.to_string_lossy());

    let db = Arc::new(Database::new(&db_url).await?);
    let job_manager = Arc::new(JobManager::new(db.clone()).await?);

    Ok((job_manager, db, temp_dir))
}

// Helper function to create valid test settings
fn create_test_settings(temp_path: &str, id: usize) -> Settings {
    let mut settings = Settings::default();
    settings.library_path = temp_path.to_string();
    settings.download_path = temp_path.to_string();
    settings.qbittorrent_host = format!("host-{}", id);
    settings.qbittorrent_port = 8080 + id as u16;
    settings.search_limit = 100 + id as u32;
    settings.max_concurrent_downloads = 5 + id as u32;
    settings.debug = id % 2 == 0;
    settings
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    // Integration Test 1: Concurrent Settings Saves
    // Validates: Requirements 1.3, 4.3
    // Test that multiple concurrent settings save operations complete successfully
    // and the final state matches one of the saved settings completely (no interleaving)
    #[tokio::test]
    async fn test_concurrent_settings_saves() {
        // Create test manager
        let (manager, _, temp_dir) = create_test_settings_manager().await.unwrap();
        let temp_path = temp_dir.path().to_string_lossy().to_string();
        let manager = Arc::new(manager);

        // Create multiple different settings
        let num_workers = 10;
        let mut settings_list = Vec::new();
        for i in 0..num_workers {
            let settings = create_test_settings(&temp_path, i);
            settings_list.push(settings);
        }

        // Spawn concurrent save operations
        let mut handles = Vec::new();
        for settings in settings_list.clone() {
            let manager_clone = manager.clone();
            let handle = tokio::spawn(async move {
                manager_clone.save_settings(&settings).await
            });
            handles.push(handle);
        }

        // Wait for all saves to complete
        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.await.unwrap());
        }

        // Verify that all saves succeeded
        for (i, result) in results.iter().enumerate() {
            assert!(
                result.is_ok(),
                "Concurrent save {} should succeed: {:?}",
                i,
                result
            );
        }

        // Load final settings
        let loaded = manager.load_settings().await.unwrap();

        // Verify that the loaded settings match one of the saved settings completely
        let matches_any = settings_list.iter().any(|s| {
            s.qbittorrent_host == loaded.qbittorrent_host
                && s.qbittorrent_port == loaded.qbittorrent_port
                && s.search_limit == loaded.search_limit
                && s.max_concurrent_downloads == loaded.max_concurrent_downloads
                && s.debug == loaded.debug
        });

        assert!(
            matches_any,
            "Loaded settings should match one of the concurrently saved settings"
        );

        // Verify internal consistency - all fields should come from the same settings object
        let matching_settings = settings_list
            .iter()
            .find(|s| s.qbittorrent_host == loaded.qbittorrent_host)
            .expect("Should find matching settings");

        assert_eq!(
            loaded.qbittorrent_port, matching_settings.qbittorrent_port,
            "All fields should come from the same settings object (no interleaving)"
        );
        assert_eq!(
            loaded.search_limit, matching_settings.search_limit,
            "All fields should come from the same settings object (no interleaving)"
        );
        assert_eq!(
            loaded.max_concurrent_downloads, matching_settings.max_concurrent_downloads,
            "All fields should come from the same settings object (no interleaving)"
        );
        assert_eq!(
            loaded.debug, matching_settings.debug,
            "All fields should come from the same settings object (no interleaving)"
        );
    }

    // Integration Test 2: Concurrent Job Claims
    // Validates: Requirements 5.2
    // Test that when multiple workers attempt to claim the same job concurrently,
    // exactly one worker succeeds
    #[tokio::test]
    async fn test_concurrent_job_claims() {
        // Create test job manager
        let (job_manager, db, _temp_dir) = create_test_job_manager().await.unwrap();

        // Create and enqueue a job
        let job = Job::new("test_job".to_string(), json!({"data": "test"}), 0);
        let job_id = job.job_id.clone();
        job_manager.enqueue(job).await.unwrap();

        // Verify job is in pending state
        let job_db = JobDatabase::new(db.clone());
        let loaded_job = Job::load(&job_db, &job_id).await.unwrap().unwrap();
        assert_eq!(loaded_job.status, JobStatus::Pending);

        // Spawn multiple workers trying to claim the same job concurrently
        let num_workers = 20;
        let mut handles = Vec::new();
        for _ in 0..num_workers {
            let manager = job_manager.clone();
            let job_id_clone = job_id.clone();

            let handle = tokio::spawn(async move {
                manager.claim_job(&job_id_clone).await
            });

            handles.push(handle);
        }

        // Wait for all workers to complete
        let mut results = Vec::new();
        for handle in handles {
            let result = handle.await.unwrap();
            results.push(result);
        }

        // Count how many workers successfully claimed the job
        let successful_claims: Vec<_> = results
            .iter()
            .filter_map(|r| r.as_ref().ok())
            .filter_map(|opt| opt.as_ref())
            .collect();

        // Exactly one worker should have successfully claimed the job
        assert_eq!(
            successful_claims.len(),
            1,
            "Exactly one worker should successfully claim the job. Got {} successful claims out of {} workers",
            successful_claims.len(),
            num_workers
        );

        // Verify the claimed job has status 'working'
        let claimed_job = successful_claims[0];
        assert_eq!(claimed_job.status, JobStatus::Working);

        // Verify the job in the database has status 'working'
        let final_job = Job::load(&job_db, &job_id).await.unwrap().unwrap();
        assert_eq!(final_job.status, JobStatus::Working);
        assert!(final_job.started_at.is_some());
    }

    // Integration Test 3: Lock Contention Handling
    // Validates: Requirements 1.3, 4.3
    // Test that the system handles high lock contention gracefully with retry logic
    #[tokio::test]
    async fn test_lock_contention_handling() {
        // Create test manager
        let (manager, _, temp_dir) = create_test_settings_manager().await.unwrap();
        let temp_path = temp_dir.path().to_string_lossy().to_string();
        let manager = Arc::new(manager);

        // Create a high contention scenario with many concurrent operations
        let num_operations = 50;
        let mut handles = Vec::new();

        for i in 0..num_operations {
            let manager_clone = manager.clone();
            let temp_path_clone = temp_path.clone();

            let handle = tokio::spawn(async move {
                let settings = create_test_settings(&temp_path_clone, i);
                // Add a small random delay to increase contention
                tokio::time::sleep(Duration::from_micros(i as u64 % 10)).await;
                manager_clone.save_settings(&settings).await
            });

            handles.push(handle);
        }

        // Wait for all operations to complete
        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.await.unwrap());
        }

        // Count successes and failures
        let successes = results.iter().filter(|r| r.is_ok()).count();
        let failures = results.iter().filter(|r| r.is_err()).count();

        // Most operations should succeed despite high contention
        // We allow some failures due to retry exhaustion, but most should succeed
        assert!(
            successes > num_operations * 8 / 10,
            "At least 80% of operations should succeed under high contention. Got {} successes, {} failures out of {} operations",
            successes,
            failures,
            num_operations
        );

        // Verify that the final state is consistent
        let loaded = manager.load_settings().await.unwrap();
        assert!(!loaded.qbittorrent_host.is_empty());
        assert!(loaded.qbittorrent_port > 0);
        assert!(loaded.search_limit > 0);
    }

    // Integration Test 4: Transaction Rollback
    // Validates: Requirements 4.2, 4.4
    // Test that when a transaction fails midway, all changes are rolled back
    #[tokio::test]
    async fn test_transaction_rollback() {
        // Create test manager
        let (manager, settings_db, temp_dir) = create_test_settings_manager().await.unwrap();
        let temp_path = temp_dir.path().to_string_lossy().to_string();

        // Save initial valid settings
        let initial_settings = create_test_settings(&temp_path, 1);
        manager.save_settings(&initial_settings).await.unwrap();

        // Verify initial settings were saved
        let loaded_initial = manager.load_settings().await.unwrap();
        assert_eq!(loaded_initial.qbittorrent_host, initial_settings.qbittorrent_host);
        assert_eq!(loaded_initial.qbittorrent_port, initial_settings.qbittorrent_port);

        // Get a snapshot of the database state
        let records_before = settings_db.get_all().await.unwrap();
        let count_before = records_before.len();

        // Create invalid settings (port = 0 is invalid)
        let mut invalid_settings = create_test_settings(&temp_path, 2);
        invalid_settings.qbittorrent_port = 0; // Invalid port

        // Attempt to save invalid settings - should fail
        let result = manager.save_settings(&invalid_settings).await;
        assert!(result.is_err(), "Save should fail for invalid settings");

        // Verify that the database state hasn't changed
        let records_after = settings_db.get_all().await.unwrap();
        let count_after = records_after.len();

        assert_eq!(
            count_before, count_after,
            "Database record count should remain unchanged after failed save"
        );

        // Verify that the original settings are still intact
        let loaded_after_failure = manager.load_settings().await.unwrap();
        assert_eq!(
            loaded_after_failure.qbittorrent_host, initial_settings.qbittorrent_host,
            "Original qbittorrent_host should be preserved after rollback"
        );
        assert_eq!(
            loaded_after_failure.qbittorrent_port, initial_settings.qbittorrent_port,
            "Original qbittorrent_port should be preserved after rollback"
        );
        assert_eq!(
            loaded_after_failure.search_limit, initial_settings.search_limit,
            "Original search_limit should be preserved after rollback"
        );

        // Verify that none of the invalid settings were persisted
        assert_ne!(
            loaded_after_failure.qbittorrent_host, invalid_settings.qbittorrent_host,
            "Invalid qbittorrent_host should not be persisted"
        );
        assert_ne!(
            loaded_after_failure.qbittorrent_port, invalid_settings.qbittorrent_port,
            "Invalid qbittorrent_port should not be persisted"
        );
    }

    // Integration Test 5: Concurrent Job Claims with Multiple Jobs
    // Validates: Requirements 5.1, 5.2, 5.3
    // Test that multiple workers can claim different jobs concurrently without conflicts
    // The key property: each job should be claimed at most once (no double-claiming)
    #[tokio::test]
    async fn test_concurrent_claims_multiple_jobs() {
        // Create test job manager
        let (job_manager, db, _temp_dir) = create_test_job_manager().await.unwrap();

        // Create and enqueue multiple jobs
        let num_jobs = 10;
        let mut job_ids = Vec::new();
        for i in 0..num_jobs {
            let job = Job::new(
                format!("test_job_{}", i),
                json!({"data": format!("test_{}", i)}),
                i as i32,
            );
            let job_id = job.job_id.clone();
            job_manager.enqueue(job).await.unwrap();
            job_ids.push(job_id);
        }

        // Verify all jobs are in pending state
        let job_db = JobDatabase::new(db.clone());
        for job_id in &job_ids {
            let job = Job::load(&job_db, job_id).await.unwrap().unwrap();
            assert_eq!(job.status, JobStatus::Pending);
        }

        // Spawn multiple workers trying to claim jobs concurrently
        // Each worker tries to claim all jobs (high contention scenario)
        let num_workers = 20;
        let mut handles = Vec::new();
        for _ in 0..num_workers {
            let manager = job_manager.clone();
            let job_ids_clone = job_ids.clone();

            let handle = tokio::spawn(async move {
                let mut claimed = Vec::new();
                for job_id in job_ids_clone {
                    if let Ok(Some(job)) = manager.claim_job(&job_id).await {
                        claimed.push(job);
                    }
                }
                claimed
            });

            handles.push(handle);
        }

        // Wait for all workers to complete
        let mut all_claimed_jobs = Vec::new();
        for handle in handles {
            let claimed = handle.await.unwrap();
            all_claimed_jobs.extend(claimed);
        }

        // The critical property: each job should be claimed exactly once
        // Count how many times each job_id appears in the claimed jobs
        let mut job_claim_counts = std::collections::HashMap::new();
        for job in &all_claimed_jobs {
            *job_claim_counts.entry(job.job_id.clone()).or_insert(0) += 1;
        }

        // Verify no job was claimed more than once
        for (job_id, count) in &job_claim_counts {
            assert_eq!(
                *count, 1,
                "Job {} should be claimed exactly once, but was claimed {} times",
                job_id, count
            );
        }

        // Verify that all claimed jobs are in working state
        for job_id in job_claim_counts.keys() {
            let job = Job::load(&job_db, job_id).await.unwrap().unwrap();
            assert_eq!(
                job.status,
                JobStatus::Working,
                "Claimed job {} should be in working state",
                job_id
            );
            assert!(job.started_at.is_some());
        }

        // Most or all jobs should have been claimed
        assert!(
            job_claim_counts.len() >= num_jobs - 2,
            "At least {} jobs should be claimed. Got {}",
            num_jobs - 2,
            job_claim_counts.len()
        );
    }

    // Integration Test 6: Settings Save with Validation Failure Under Concurrency
    // Validates: Requirements 4.2, 4.4
    // Test that validation failures don't corrupt the database under concurrent load
    #[tokio::test]
    async fn test_concurrent_saves_with_validation_failures() {
        // Create test manager
        let (manager, _, temp_dir) = create_test_settings_manager().await.unwrap();
        let temp_path = temp_dir.path().to_string_lossy().to_string();
        let manager = Arc::new(manager);

        // Save initial valid settings
        let initial_settings = create_test_settings(&temp_path, 0);
        manager.save_settings(&initial_settings).await.unwrap();

        // Spawn concurrent operations with mix of valid and invalid settings
        let num_operations = 20;
        let mut handles = Vec::new();

        for i in 0..num_operations {
            let manager_clone = manager.clone();
            let temp_path_clone = temp_path.clone();

            let handle = tokio::spawn(async move {
                let mut settings = create_test_settings(&temp_path_clone, i);
                
                // Make every other settings invalid
                if i % 2 == 0 {
                    settings.qbittorrent_port = 0; // Invalid port
                }
                
                manager_clone.save_settings(&settings).await
            });

            handles.push(handle);
        }

        // Wait for all operations to complete
        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.await.unwrap());
        }

        // Count successes and failures
        let successes = results.iter().filter(|r| r.is_ok()).count();
        let failures = results.iter().filter(|r| r.is_err()).count();

        // We expect roughly half to succeed (the valid ones) and half to fail (the invalid ones)
        assert!(
            failures >= num_operations / 3,
            "Expected at least {} failures, got {}",
            num_operations / 3,
            failures
        );
        assert!(
            successes >= num_operations / 3,
            "Expected at least {} successes, got {}",
            num_operations / 3,
            successes
        );

        // Verify that the final state is valid and consistent
        let loaded = manager.load_settings().await.unwrap();
        assert!(!loaded.qbittorrent_host.is_empty());
        assert!(loaded.qbittorrent_port > 0, "Final port should be valid");
        assert!(loaded.search_limit > 0);

        // Verify that the loaded settings are internally consistent
        // (all fields should come from the same settings object)
        let validation_result = manager.validate_settings(&loaded);
        assert!(
            validation_result.is_ok(),
            "Final settings should be valid: {:?}",
            validation_result
        );
    }

    // Integration Test 7: Stress Test - High Concurrency
    // Validates: Requirements 1.3, 4.3, 5.2
    // Test system behavior under very high concurrent load
    #[tokio::test]
    async fn test_high_concurrency_stress() {
        // Create test manager
        let (manager, _, temp_dir) = create_test_settings_manager().await.unwrap();
        let temp_path = temp_dir.path().to_string_lossy().to_string();
        let manager = Arc::new(manager);

        // Create a very high concurrency scenario
        let num_operations = 100;
        let mut handles = Vec::new();

        for i in 0..num_operations {
            let manager_clone = manager.clone();
            let temp_path_clone = temp_path.clone();

            let handle = tokio::spawn(async move {
                let settings = create_test_settings(&temp_path_clone, i);
                manager_clone.save_settings(&settings).await
            });

            handles.push(handle);
        }

        // Wait for all operations to complete
        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.await.unwrap());
        }

        // Count successes
        let successes = results.iter().filter(|r| r.is_ok()).count();

        // Under high load, we expect most operations to succeed
        // Allow for some failures due to retry exhaustion
        assert!(
            successes > num_operations * 7 / 10,
            "At least 70% of operations should succeed under high load. Got {} successes out of {} operations",
            successes,
            num_operations
        );

        // Verify that the final state is consistent
        let loaded = manager.load_settings().await.unwrap();
        let validation_result = manager.validate_settings(&loaded);
        assert!(
            validation_result.is_ok(),
            "Final settings should be valid after stress test: {:?}",
            validation_result
        );
    }
}
