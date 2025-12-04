// Hashing job worker for background file hashing operations
// Processes FileVersions with pending hashing status one at a time to avoid I/O overload

use std::sync::Arc;
use anyhow::{Result, Context};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{info, warn, debug};

use super::manager::WorkerManager;
use crate::filesystem::hasher::{FileHasher, HashAlgorithm};
use crate::database::Database;

/// Parameters for hashing job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashingJobParams {
    /// Maximum number of files to hash in this job run
    pub batch_size: usize,
}

/// Result of hashing job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashingJobResult {
    /// Number of files successfully hashed
    pub files_hashed: usize,
    /// Number of files that failed to hash
    pub files_failed: usize,
    /// Errors encountered during hashing
    pub errors: Vec<String>,
}

/// Worker manager for background hashing operations
///
/// This worker:
/// - Queries FileVersions with hashing_status "pending"
/// - Limits concurrency to 1 file at a time (sequential processing)
/// - Computes full-file hash using FileHasher
/// - Updates FileVersion with hash and status "complete"
///
/// # Requirements
/// - Requirements: 3.1, 3.2, 3.3, 3.4
pub struct HashingWorkerManager {
    database: Arc<Database>,
    hasher: FileHasher,
}

impl HashingWorkerManager {
    /// Create a new HashingWorkerManager
    pub fn new(database: Arc<Database>) -> Self {
        info!("HashingWorkerManager: Creating new HashingWorkerManager");
        Self {
            database,
            hasher: FileHasher::new(),
        }
    }

    /// Query FileVersions with pending hashing status
    ///
    /// # Requirements
    /// - Requirements: 3.1
    async fn get_pending_versions(&self, limit: usize) -> Result<Vec<PendingVersion>> {
        use sqlx::Row;

        let rows = sqlx::query(
            r#"
            SELECT id, absolute_path, hash_algorithm
            FROM file_versions
            WHERE hashing_status = 'pending'
            ORDER BY added_at ASC
            LIMIT ?
            "#
        )
        .bind(limit as i64)
        .fetch_all(self.database.pool())
        .await
        .context("Failed to query pending file versions")?;

        let mut versions = Vec::new();
        for row in rows {
            let id: i64 = row.try_get("id")?;
            let absolute_path: String = row.try_get("absolute_path")?;
            let hash_algorithm: Option<String> = row.try_get("hash_algorithm")?;

            versions.push(PendingVersion {
                id,
                absolute_path: std::path::PathBuf::from(absolute_path),
                hash_algorithm: hash_algorithm.unwrap_or_else(|| "blake3".to_string()),
            });
        }

        debug!("HashingWorkerManager: Found {} pending file versions", versions.len());
        Ok(versions)
    }

    /// Update FileVersion with computed hash
    ///
    /// # Requirements
    /// - Requirements: 3.4
    async fn update_version_hash(
        &self,
        version_id: i64,
        hash_value: &[u8],
        algorithm: &str,
    ) -> Result<()> {
        let now = chrono::Utc::now();

        sqlx::query(
            r#"
            UPDATE file_versions
            SET full_hash = ?,
                hash_algorithm = ?,
                hashing_status = 'complete',
                hashed_at = ?
            WHERE id = ?
            "#
        )
        .bind(hash_value)
        .bind(algorithm)
        .bind(now)
        .bind(version_id)
        .execute(self.database.pool())
        .await
        .context("Failed to update file version hash")?;

        Ok(())
    }

    /// Mark FileVersion hashing as failed
    async fn mark_hashing_failed(&self, version_id: i64, error: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE file_versions
            SET hashing_status = 'failed'
            WHERE id = ?
            "#
        )
        .bind(version_id)
        .execute(self.database.pool())
        .await
        .context("Failed to mark hashing as failed")?;

        warn!("HashingWorkerManager: Marked version {} as failed: {}", version_id, error);
        Ok(())
    }

    /// Process a single file version (compute hash)
    ///
    /// # Requirements
    /// - Requirements: 3.2, 3.3 (concurrency limit enforced by sequential processing)
    async fn process_version(&self, version: &PendingVersion) -> Result<()> {
        info!("HashingWorkerManager: Processing version {} at {:?}", version.id, version.absolute_path);

        // Parse hash algorithm
        let algorithm = match version.hash_algorithm.to_lowercase().as_str() {
            "sha256" | "sha-256" => HashAlgorithm::Sha256,
            "blake3" => HashAlgorithm::Blake3,
            _ => {
                warn!("HashingWorkerManager: Unknown algorithm '{}', defaulting to BLAKE3", version.hash_algorithm);
                HashAlgorithm::Blake3
            }
        };

        // Compute full hash
        let full_hash = self.hasher.compute_full_hash(&version.absolute_path, algorithm)
            .await
            .context("Failed to compute full hash")?;

        // Update database
        self.update_version_hash(version.id, &full_hash.value, full_hash.algorithm.as_str())
            .await?;

        info!("HashingWorkerManager: Successfully hashed version {}", version.id);
        Ok(())
    }
}

#[async_trait]
impl WorkerManager for HashingWorkerManager {
    async fn process(&self, params: Value) -> Result<Value> {
        info!("HashingWorkerManager: Starting hashing job");

        // Deserialize parameters
        let job_params: HashingJobParams = serde_json::from_value(params)
            .context("Failed to deserialize hashing job parameters")?;

        debug!("HashingWorkerManager: Batch size: {}", job_params.batch_size);

        // Get pending versions
        let pending_versions = self.get_pending_versions(job_params.batch_size).await?;

        if pending_versions.is_empty() {
            info!("HashingWorkerManager: No pending file versions to hash");
            return Ok(serde_json::to_value(HashingJobResult {
                files_hashed: 0,
                files_failed: 0,
                errors: vec![],
            })?);
        }

        info!("HashingWorkerManager: Processing {} file versions", pending_versions.len());

        let mut files_hashed = 0;
        let mut files_failed = 0;
        let mut errors = Vec::new();

        // Process versions one at a time (sequential to limit concurrency to 1)
        // This satisfies Requirement 3.3: limit concurrency to 1 file at a time
        for version in pending_versions {
            match self.process_version(&version).await {
                Ok(()) => {
                    files_hashed += 1;
                }
                Err(e) => {
                    files_failed += 1;
                    let error_msg = format!("Failed to hash version {}: {}", version.id, e);
                    errors.push(error_msg.clone());
                    warn!("HashingWorkerManager: {}", error_msg);

                    // Mark as failed in database
                    if let Err(mark_err) = self.mark_hashing_failed(version.id, &e.to_string()).await {
                        warn!("HashingWorkerManager: Failed to mark version as failed: {}", mark_err);
                    }
                }
            }
        }

        let result = HashingJobResult {
            files_hashed,
            files_failed,
            errors,
        };

        info!(
            "HashingWorkerManager: Hashing job completed - {} hashed, {} failed",
            result.files_hashed, result.files_failed
        );

        Ok(serde_json::to_value(result)?)
    }

    fn job_kind(&self) -> &str {
        "HashFiles"
    }
}

/// Internal structure for pending file versions
#[derive(Debug, Clone)]
struct PendingVersion {
    id: i64,
    absolute_path: std::path::PathBuf,
    hash_algorithm: String,
}
