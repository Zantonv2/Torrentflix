// Property-based tests for JobManager atomic job claiming
// Feature: database-race-condition-fix

use engine::database::{Database, jobs::{JobDatabase, Job, JobStatus}};
use engine::jobs::JobManager;
use proptest::prelude::*;
use std::sync::Arc;
use serde_json::json;

// Generator for job priority
fn job_priority() -> impl Strategy<Value = i32> {
    -100i32..=100i32
}

// Generator for job kind
fn job_kind() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-zA-Z]{5,15}").unwrap()
}

// Generator for job params
fn job_params() -> impl Strategy<Value = serde_json::Value> {
    prop::collection::vec(any::<u32>(), 0..5)
        .prop_map(|vec| json!({"data": vec}))
}

// Property 7: Atomic Job Claiming
// For any job in 'pending' status, when multiple workers attempt to claim it 
// concurrently, exactly one worker should succeed in changing the status to 'working'.
// Validates: Requirements 5.1, 5.2, 5.3, 5.4
#[cfg(test)]
mod proptest_tests {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(50))]
        
        #[test]
        fn test_atomic_job_claiming_property(
            priority in job_priority(),
            kind in job_kind(),
            params in job_params(),
            num_workers in 2usize..=10usize,
        ) {
            // Use tokio runtime for async test
            let runtime = tokio::runtime::Runtime::new().unwrap();
            
            runtime.block_on(async {
                // Create a temporary database for this test
                let temp_dir = tempfile::TempDir::new().unwrap();
                let db_path = temp_dir.path().join("test_job_claiming.db");
                let db_url = format!("sqlite://{}", db_path.to_string_lossy());
                
                // Initialize database and job manager
                let db = Arc::new(Database::new(&db_url).await.unwrap());
                let job_manager = Arc::new(JobManager::new(db.clone()).await.unwrap());
                
                // Create a job
                let job = Job::new(kind.clone(), params.clone(), priority);
                let job_id = job.job_id.clone();
                
                // Enqueue the job
                job_manager.enqueue(job).await.unwrap();
                
                // Verify job is in pending state
                let job_db = JobDatabase::new(db.clone());
                let loaded_job = Job::load(&job_db, &job_id).await.unwrap().unwrap();
                assert_eq!(loaded_job.status, JobStatus::Pending, "Job should start in pending state");
                
                // Spawn multiple workers trying to claim the same job concurrently
                let mut handles = vec![];
                for _ in 0..num_workers {
                    let manager = job_manager.clone();
                    let job_id_clone = job_id.clone();
                    
                    let handle = tokio::spawn(async move {
                        manager.claim_job(&job_id_clone).await
                    });
                    
                    handles.push(handle);
                }
                
                // Wait for all workers to complete
                let mut results = vec![];
                for handle in handles {
                    let result = handle.await.unwrap();
                    results.push(result);
                }
                
                // Count how many workers successfully claimed the job
                let successful_claims: Vec<_> = results.iter()
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
                assert_eq!(
                    claimed_job.status,
                    JobStatus::Working,
                    "Claimed job should have status 'working'"
                );
                
                // Verify the job in the database has status 'working'
                let final_job = Job::load(&job_db, &job_id).await.unwrap().unwrap();
                assert_eq!(
                    final_job.status,
                    JobStatus::Working,
                    "Job in database should have status 'working' after claim"
                );
                
                // Verify started_at is set
                assert!(
                    final_job.started_at.is_some(),
                    "Job should have started_at timestamp after being claimed"
                );
            });
        }
    }
}

// Property 8: Job Status Verification
// For any job claim attempt, if the job status changes from 'pending' to another 
// state during the claim transaction, the claim should fail and return None.
// Validates: Requirements 5.4

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]
    
    #[test]
    fn test_job_status_verification_property(
        priority in job_priority(),
        kind in job_kind(),
        params in job_params(),
    ) {
        // Use tokio runtime for async test
        let runtime = tokio::runtime::Runtime::new().unwrap();
        
        runtime.block_on(async {
            // Create a temporary database for this test
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test_status_verification.db");
            let db_url = format!("sqlite://{}", db_path.to_string_lossy());
            
            // Initialize database and job manager
            let db = Arc::new(Database::new(&db_url).await.unwrap());
            let job_manager = JobManager::new(db.clone()).await.unwrap();
            let job_db = JobDatabase::new(db.clone());
            
            // Create and enqueue a job
            let job = Job::new(kind.clone(), params.clone(), priority);
            let job_id = job.job_id.clone();
            job_manager.enqueue(job).await.unwrap();
            
            // Verify job is in pending state
            let loaded_job = Job::load(&job_db, &job_id).await.unwrap().unwrap();
            assert_eq!(loaded_job.status, JobStatus::Pending);
            
            // Change the job status to 'working' directly in the database
            // This simulates another worker claiming the job
            sqlx::query(
                "UPDATE jobs SET status = 'working', started_at = ? WHERE job_id = ?"
            )
            .bind(chrono::Utc::now().timestamp())
            .bind(&job_id)
            .execute(job_db.pool())
            .await
            .unwrap();
            
            // Verify the status was changed
            let modified_job = Job::load(&job_db, &job_id).await.unwrap().unwrap();
            assert_eq!(modified_job.status, JobStatus::Working);
            
            // Now try to claim the job - it should fail because status is no longer 'pending'
            let claim_result = job_manager.claim_job(&job_id).await.unwrap();
            
            // The claim should return None because the job is no longer pending
            assert!(
                claim_result.is_none(),
                "Claim should fail (return None) when job status is not 'pending'"
            );
            
            // Verify the job status remains 'working' (unchanged by failed claim)
            let final_job = Job::load(&job_db, &job_id).await.unwrap().unwrap();
            assert_eq!(
                final_job.status,
                JobStatus::Working,
                "Job status should remain 'working' after failed claim attempt"
            );
        });
    }
}

// Additional test: Verify claim fails for non-existent job
proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]
    
    #[test]
    fn test_claim_nonexistent_job_property(
        job_id in prop::string::string_regex("[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}").unwrap(),
    ) {
        // Use tokio runtime for async test
        let runtime = tokio::runtime::Runtime::new().unwrap();
        
        runtime.block_on(async {
            // Create a temporary database for this test
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test_nonexistent.db");
            let db_url = format!("sqlite://{}", db_path.to_string_lossy());
            
            // Initialize database and job manager
            let db = Arc::new(Database::new(&db_url).await.unwrap());
            let job_manager = JobManager::new(db.clone()).await.unwrap();
            
            // Try to claim a job that doesn't exist
            let claim_result = job_manager.claim_job(&job_id).await.unwrap();
            
            // The claim should return None because the job doesn't exist
            assert!(
                claim_result.is_none(),
                "Claim should fail (return None) for non-existent job"
            );
        });
    }
}
