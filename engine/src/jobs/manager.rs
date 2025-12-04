use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use tracing::{info, warn};

use crate::database::jobs::{JobDatabase, Job};
use crate::ratings::RatingManager;

/// Trait for worker managers that can process jobs
#[async_trait]
pub trait WorkerManager: Send + Sync {
    /// Process a job with given parameters
    async fn process(&self, params: Value) -> Result<Value>;
    
    /// Get the job kind this manager handles
    fn job_kind(&self) -> &str;
    
    /// Check if this manager can handle a job kind
    fn can_handle(&self, kind: &str) -> bool {
        self.job_kind() == kind
    }
}

/// JobManager coordinates all worker managers and job queue
pub struct JobManager {
    database: Arc<JobDatabase>,
    pub(crate) workers: HashMap<String, Box<dyn WorkerManager>>,
}

impl JobManager {
    /// Create a new JobManager
    pub async fn new(database: Arc<crate::database::Database>) -> Result<Self> {
        let job_database = Arc::new(JobDatabase::new(database.clone()));
        job_database.initialize().await?;
        
        let workers = HashMap::new();

        let mut manager = Self {
            database: job_database,
            workers,
        };

        // Register default worker managers
        manager.register_default_workers().await?;

        Ok(manager)
    }

    /// Register default worker managers
    /// Note: RatingManager should be registered separately via register_rating_worker
    async fn register_default_workers(&mut self) -> Result<()> {
        info!("JobManager: Default worker managers registered (none yet)");
        Ok(())
    }

    /// Register RatingManager as a worker
    pub fn register_rating_worker(&mut self, rating_manager: Arc<RatingManager>) {
        use super::rating_worker::RatingWorkerManager;
        let worker = Box::new(RatingWorkerManager::new(rating_manager));
        self.register_worker(worker);
    }

    /// Register HashingWorkerManager as a worker
    pub fn register_hashing_worker(&mut self) {
        use super::hashing_worker::HashingWorkerManager;
        let worker = Box::new(HashingWorkerManager::new(self.database.database().clone()));
        self.register_worker(worker);
    }

    /// Register a worker manager
    pub fn register_worker(&mut self, worker: Box<dyn WorkerManager>) {
        let kind = worker.job_kind().to_string();
        info!("JobManager: Registering worker for job kind '{}'", kind);
        self.workers.insert(kind.clone(), worker);
        info!("JobManager: Worker '{}' registered. Total workers now: {}", kind, self.workers.len());
    }

    /// Enqueue a job
    pub async fn enqueue(&self, job: Job) -> Result<String> {
        info!("JobManager: Enqueueing job '{}' of kind '{}'", job.job_id, job.kind);
        job.enqueue(&self.database).await?;
        Ok(job.job_id)
    }

    /// Get next pending job
    pub async fn get_next_pending_job(&self) -> Result<Option<Job>> {
        use sqlx::Row;
        let now = chrono::Utc::now().timestamp();
        
        let row = sqlx::query(
            r#"
            SELECT job_id, kind, params, status, priority, attempts, max_attempts,
                   depends_on, scheduled_at, created_at, started_at, finished_at, last_error, result
            FROM jobs
            WHERE status = 'pending'
              AND scheduled_at <= ?
              AND (depends_on IS NULL OR depends_on IN (
                  SELECT job_id FROM jobs WHERE status = 'succeeded'
              ))
            ORDER BY priority DESC, scheduled_at ASC
            LIMIT 1
            "#,
        )
        .bind(now)
        .fetch_optional(self.database.pool())
        .await?;

        if let Some(row) = row {
            let job_id: String = row.try_get("job_id")?;
            Job::load(&self.database, &job_id).await
        } else {
            Ok(None)
        }
    }

    /// Claim a job (mark as working) atomically using transactions
    pub async fn claim_job(&self, job_id: &str) -> Result<Option<Job>> {
        use crate::database::transaction::{with_retry, RetryConfig};
        
        // Use retry logic for the entire claim operation
        with_retry(
            || async {
                self.claim_job_internal(job_id).await
            },
            &RetryConfig::default(),
        )
        .await
    }

    /// Internal implementation of atomic job claiming
    async fn claim_job_internal(&self, job_id: &str) -> Result<Option<Job>> {
        use sqlx::Row;
        
        // Begin an immediate transaction to acquire write lock
        let mut tx = self.database.pool().begin().await?;
        
        // Lock the row and verify status is still 'pending'
        // SQLite doesn't support SELECT FOR UPDATE, but BEGIN IMMEDIATE gives us a write lock
        let row = sqlx::query(
            r#"
            SELECT job_id, kind, params, status, priority, attempts, max_attempts,
                   depends_on, scheduled_at, created_at, started_at, finished_at, 
                   last_error, result
            FROM jobs
            WHERE job_id = ? AND status = 'pending'
            "#
        )
        .bind(job_id)
        .fetch_optional(&mut *tx)
        .await?;
        
        if row.is_none() {
            // Job already claimed, doesn't exist, or is not pending
            tx.rollback().await?;
            return Ok(None);
        }
        
        // Update status to 'working'
        let now = chrono::Utc::now().timestamp();
        let result = sqlx::query(
            r#"
            UPDATE jobs
            SET status = 'working', started_at = ?
            WHERE job_id = ? AND status = 'pending'
            "#
        )
        .bind(now)
        .bind(job_id)
        .execute(&mut *tx)
        .await?;
        
        // Verify the update succeeded (status verification)
        if result.rows_affected() == 0 {
            // Status changed between SELECT and UPDATE
            tx.rollback().await?;
            return Ok(None);
        }
        
        // Commit the transaction
        tx.commit().await?;
        
        // Load and return the claimed job
        Job::load(&self.database, job_id).await
    }

    /// Process a job using the appropriate worker
    pub async fn process_job(&self, job: Job) -> Result<Value> {
        let worker = self.workers.get(&job.kind)
            .ok_or_else(|| anyhow::anyhow!("No worker registered for job kind '{}'", job.kind))?;

        info!("JobManager: Processing job '{}' of kind '{}'", job.job_id, job.kind);
        
        match worker.process(job.params.clone()).await {
            Ok(result) => {
                info!("JobManager: Job '{}' completed successfully", job.job_id);
                Ok(result)
            }
            Err(e) => {
                warn!("JobManager: Job '{}' failed: {}", job.job_id, e);
                Err(e)
            }
        }
    }

    /// Directly process a rating fetch job without queuing (immediate execution)
    /// This is for Phase 1: immediate execution pattern
    pub async fn fetch_ratings_immediate(&self, movies: Vec<crate::models::EnrichedMedia>) -> Result<Vec<crate::models::EnrichedMedia>> {
        info!("JobManager: fetch_ratings_immediate called with {} movies", movies.len());
        info!("JobManager: Registered workers: {:?}", self.workers.keys().collect::<Vec<_>>());
        
        let worker = self.workers.get("FetchRatings")
            .ok_or_else(|| {
                let error_msg = format!("RatingManager worker not registered. Available workers: {:?}", self.workers.keys().collect::<Vec<_>>());
                warn!("JobManager: {}", error_msg);
                anyhow::anyhow!(error_msg)
            })?;

        info!("JobManager: Found FetchRatings worker, starting immediate execution for {} movies", movies.len());
        
        let params = match serde_json::to_value(&movies) {
            Ok(p) => {
                info!("JobManager: Successfully serialized {} movies to JSON", movies.len());
                p
            }
            Err(e) => {
                warn!("JobManager: Failed to serialize movies to JSON: {}", e);
                return Err(anyhow::anyhow!("Failed to serialize movies: {}", e));
            }
        };
        
        info!("JobManager: Calling worker.process()...");
        let result = match worker.process(params).await {
            Ok(r) => {
                info!("JobManager: Worker.process() completed successfully");
                r
            }
            Err(e) => {
                warn!("JobManager: Worker.process() failed: {}", e);
                return Err(e);
            }
        };
        
        info!("JobManager: Deserializing results...");
        let updated_movies: Vec<crate::models::EnrichedMedia> = match serde_json::from_value::<Vec<crate::models::EnrichedMedia>>(result) {
            Ok(movies) => {
                info!("JobManager: Successfully deserialized {} movies with ratings", movies.len());
                movies
            }
            Err(e) => {
                warn!("JobManager: Failed to deserialize results: {}", e);
                return Err(anyhow::anyhow!("Failed to deserialize results: {}", e));
            }
        };
        
        info!("JobManager: fetch_ratings_immediate completed successfully, returning {} movies", updated_movies.len());
        Ok(updated_movies)
    }

    /// Mark job as succeeded
    pub async fn mark_succeeded(&self, job_id: &str, result: Option<Value>) -> Result<()> {
        let now = chrono::Utc::now().timestamp();
        
        sqlx::query(
            r#"
            UPDATE jobs
            SET status = 'succeeded', finished_at = ?, result = ?
            WHERE job_id = ?
            "#,
        )
        .bind(now)
        .bind(result.as_ref().map(|r| serde_json::to_string(r).unwrap_or_default()))
        .bind(job_id)
        .execute(self.database.pool())
        .await?;

        Ok(())
    }

    /// Mark job as failed
    pub async fn mark_failed(&self, job_id: &str, error: &str, retry: bool) -> Result<()> {
        let job = Job::load(&self.database, job_id).await?
            .ok_or_else(|| anyhow::anyhow!("Job not found: {}", job_id))?;

        if retry && job.attempts < job.max_attempts {
            // Schedule retry with exponential backoff
            let backoff_seconds = 60 * (2_i64.pow(job.attempts as u32));
            let scheduled_at = chrono::Utc::now().timestamp() + backoff_seconds;
            
            sqlx::query(
                r#"
                UPDATE jobs
                SET status = 'pending', attempts = attempts + 1, scheduled_at = ?, last_error = ?
                WHERE job_id = ?
                "#,
            )
            .bind(scheduled_at)
            .bind(error)
            .bind(job_id)
            .execute(self.database.pool())
            .await?;

            info!("JobManager: Job '{}' scheduled for retry (attempt {}/{})", 
                  job_id, job.attempts + 1, job.max_attempts);
        } else {
            // Mark as permanently failed
            let now = chrono::Utc::now().timestamp();
            
            sqlx::query(
                r#"
                UPDATE jobs
                SET status = 'failed', finished_at = ?, last_error = ?
                WHERE job_id = ?
                "#,
            )
            .bind(now)
            .bind(error)
            .bind(job_id)
            .execute(self.database.pool())
            .await?;

            warn!("JobManager: Job '{}' marked as failed permanently", job_id);
        }

        Ok(())
    }

    /// Get database reference
    pub fn database(&self) -> &Arc<JobDatabase> {
        &self.database
    }

    /// Get current settings from database
    pub async fn get_settings(&self) -> Result<crate::settings::models::Settings> {
        info!("JobManager: Loading settings from database");
        
        let settings_db = Arc::new(crate::database::SettingsDatabase::new(self.database.database().clone()));
        let settings_manager = crate::settings::manager::SettingsManager::new(settings_db);
        settings_manager.load_settings().await
    }

    /// Save settings to database with validation
    pub async fn save_settings(&self, settings: &crate::settings::models::Settings) -> Result<()> {
        info!("JobManager: Saving settings to database");
        
        let settings_db = Arc::new(crate::database::SettingsDatabase::new(self.database.database().clone()));
        let settings_manager = crate::settings::manager::SettingsManager::new(settings_db);
        settings_manager.save_settings(settings).await
    }

    /// Test qBittorrent connection with provided credentials
    pub async fn test_qbittorrent_connection(
        &self,
        url: &str,
        username: &str,
        password: &str,
    ) -> Result<()> {
        info!("JobManager: Testing qBittorrent connection to {}", url);
        
        let settings_db = Arc::new(crate::database::SettingsDatabase::new(self.database.database().clone()));
        let settings_manager = crate::settings::manager::SettingsManager::new(settings_db);
        settings_manager.test_qbittorrent_connection(url, username, password).await
    }
}

