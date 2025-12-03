use std::sync::Arc;
use anyhow::{Result, Context};
use sqlx::SqlitePool;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use tracing::info;

use super::connection::Database;

/// Database connection for job queue
pub struct JobDatabase {
    database: Arc<Database>,
}

impl JobDatabase {
    /// Create a new job database connection
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    /// Initialize job queue schema
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing job queue schema");

        // Create jobs table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS jobs (
                job_id TEXT PRIMARY KEY,
                kind TEXT NOT NULL,
                params TEXT NOT NULL,
                status TEXT NOT NULL,
                priority INTEGER NOT NULL DEFAULT 0,
                attempts INTEGER NOT NULL DEFAULT 0,
                max_attempts INTEGER NOT NULL DEFAULT 3,
                depends_on TEXT NULL,
                scheduled_at INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                started_at INTEGER NULL,
                finished_at INTEGER NULL,
                last_error TEXT NULL,
                result TEXT NULL
            )
            "#,
        )
        .execute(self.database.pool())
        .await
        .context("Failed to create jobs table")?;

        // Create indexes
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_jobs_status_priority_scheduled 
            ON jobs(status, priority DESC, scheduled_at)
            "#,
        )
        .execute(self.database.pool())
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_jobs_depends_on 
            ON jobs(depends_on)
            "#,
        )
        .execute(self.database.pool())
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_jobs_status_kind 
            ON jobs(status, kind)
            "#,
        )
        .execute(self.database.pool())
        .await?;

        info!("Job queue schema initialized");
        Ok(())
    }

    /// Get the connection pool
    pub fn pool(&self) -> &SqlitePool {
        self.database.pool()
    }

    /// Get the underlying database
    pub fn database(&self) -> Arc<Database> {
        Arc::clone(&self.database)
    }
}

/// Job status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Waiting,
    Working,
    Succeeded,
    Failed,
    Cancelled,
}

impl JobStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            JobStatus::Pending => "pending",
            JobStatus::Waiting => "waiting",
            JobStatus::Working => "working",
            JobStatus::Succeeded => "succeeded",
            JobStatus::Failed => "failed",
            JobStatus::Cancelled => "cancelled",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "pending" => JobStatus::Pending,
            "waiting" => JobStatus::Waiting,
            "working" => JobStatus::Working,
            "succeeded" => JobStatus::Succeeded,
            "failed" => JobStatus::Failed,
            "cancelled" => JobStatus::Cancelled,
            _ => JobStatus::Pending,
        }
    }
}

/// Job record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub job_id: String,
    pub kind: String,
    pub params: serde_json::Value,
    pub status: JobStatus,
    pub priority: i32,
    pub attempts: i32,
    pub max_attempts: i32,
    pub depends_on: Option<String>,
    pub scheduled_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub result: Option<serde_json::Value>,
}

impl Job {
    /// Create a new job
    pub fn new(kind: String, params: serde_json::Value, priority: i32) -> Self {
        let now = Utc::now();
        Self {
            job_id: Uuid::new_v4().to_string(),
            kind,
            params,
            status: JobStatus::Pending,
            priority,
            attempts: 0,
            max_attempts: 3,
            depends_on: None,
            scheduled_at: now,
            created_at: now,
            started_at: None,
            finished_at: None,
            last_error: None,
            result: None,
        }
    }

    /// Enqueue a job to the database
    pub async fn enqueue(&self, db: &JobDatabase) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO jobs (
                job_id, kind, params, status, priority, attempts, max_attempts,
                depends_on, scheduled_at, created_at, started_at, finished_at, last_error, result
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&self.job_id)
        .bind(&self.kind)
        .bind(serde_json::to_string(&self.params)?)
        .bind(self.status.as_str())
        .bind(self.priority)
        .bind(self.attempts)
        .bind(self.max_attempts)
        .bind(self.depends_on.as_ref())
        .bind(self.scheduled_at.timestamp())
        .bind(self.created_at.timestamp())
        .bind(self.started_at.map(|t| t.timestamp()))
        .bind(self.finished_at.map(|t| t.timestamp()))
        .bind(self.last_error.as_ref())
        .bind(self.result.as_ref().map(|r| serde_json::to_string(r).unwrap_or_default()))
        .execute(db.pool())
        .await?;

        Ok(())
    }

    /// Load a job from the database
    pub async fn load(db: &JobDatabase, job_id: &str) -> Result<Option<Self>> {
        use sqlx::Row;
        
        let row = sqlx::query(
            r#"
            SELECT job_id, kind, params, status, priority, attempts, max_attempts,
                   depends_on, scheduled_at, created_at, started_at, finished_at, last_error, result
            FROM jobs
            WHERE job_id = ?
            "#,
        )
        .bind(job_id)
        .fetch_optional(db.pool())
        .await?;

        if let Some(row) = row {
            Ok(Some(Self {
                job_id: row.try_get("job_id")?,
                kind: row.try_get("kind")?,
                params: {
                    let params_str: String = row.try_get("params")?;
                    serde_json::from_str(&params_str)?
                },
                status: {
                    let status_str: String = row.try_get("status")?;
                    JobStatus::from_str(&status_str)
                },
                priority: row.try_get("priority")?,
                attempts: row.try_get("attempts")?,
                max_attempts: row.try_get("max_attempts")?,
                depends_on: row.try_get("depends_on")?,
                scheduled_at: {
                    let ts: i64 = row.try_get("scheduled_at")?;
                    DateTime::from_timestamp(ts, 0).unwrap_or_else(Utc::now)
                },
                created_at: {
                    let ts: i64 = row.try_get("created_at")?;
                    DateTime::from_timestamp(ts, 0).unwrap_or_else(Utc::now)
                },
                started_at: {
                    let ts: Option<i64> = row.try_get("started_at")?;
                    ts.and_then(|t| DateTime::from_timestamp(t, 0))
                },
                finished_at: {
                    let ts: Option<i64> = row.try_get("finished_at")?;
                    ts.and_then(|t| DateTime::from_timestamp(t, 0))
                },
                last_error: row.try_get("last_error")?,
                result: {
                    let result_str: Option<String> = row.try_get("result")?;
                    result_str.and_then(|s| serde_json::from_str(&s).ok())
                },
            }))
        } else {
            Ok(None)
        }
    }

    /// Update job status
    pub async fn update_status(&self, db: &JobDatabase, status: JobStatus) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE jobs
            SET status = ?, attempts = ?, started_at = ?, finished_at = ?, last_error = ?, result = ?
            WHERE job_id = ?
            "#,
        )
        .bind(status.as_str())
        .bind(self.attempts)
        .bind(self.started_at.map(|t| t.timestamp()))
        .bind(self.finished_at.map(|t| t.timestamp()))
        .bind(self.last_error.as_ref())
        .bind(self.result.as_ref().map(|r| serde_json::to_string(r).unwrap_or_default()))
        .bind(&self.job_id)
        .execute(db.pool())
        .await?;

        Ok(())
    }
}

