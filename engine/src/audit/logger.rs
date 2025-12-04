use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde_json::Value as JsonValue;
use sqlx::{Pool, Sqlite};
use std::sync::Arc;

use super::models::{AuditLogEntry, AuditLogFilter, Pagination};

/// Component responsible for audit logging operations
#[derive(Clone)]
pub struct AuditLogger {
    pool: Arc<Pool<Sqlite>>,
}

impl AuditLogger {
    /// Create a new AuditLogger instance
    pub fn new(pool: Arc<Pool<Sqlite>>) -> Self {
        Self { pool }
    }

    /// Log an operation to the audit log
    /// 
    /// # Arguments
    /// * `operation_type` - Type of operation (e.g., "create", "update", "delete", "move")
    /// * `entity_type` - Type of entity affected (e.g., "media_item", "file_version")
    /// * `entity_id` - ID of the affected entity (optional for batch operations)
    /// * `old_value` - Previous state as JSON (optional)
    /// * `new_value` - New state as JSON (optional)
    /// * `initiator` - Who initiated the operation (e.g., "user", "system:job_name")
    /// * `error_message` - Error message if operation failed (optional)
    pub async fn log_operation(
        &self,
        operation_type: &str,
        entity_type: &str,
        entity_id: Option<i64>,
        old_value: Option<JsonValue>,
        new_value: Option<JsonValue>,
        initiator: &str,
        error_message: Option<&str>,
    ) -> Result<i64> {
        let result = sqlx::query(
            r#"
            INSERT INTO audit_log (
                operation_type, entity_type, entity_id,
                old_value, new_value, initiator, error_message, timestamp
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
            "#,
        )
        .bind(operation_type)
        .bind(entity_type)
        .bind(entity_id)
        .bind(old_value.map(|v| v.to_string()))
        .bind(new_value.map(|v| v.to_string()))
        .bind(initiator)
        .bind(error_message)
        .execute(&*self.pool)
        .await
        .context("Failed to create audit log entry")?;

        Ok(result.last_insert_rowid())
    }

    /// Query audit logs with filters
    /// Returns entries in reverse chronological order (newest first)
    pub async fn query_logs(
        &self,
        filter: AuditLogFilter,
        pagination: Pagination,
    ) -> Result<Vec<AuditLogEntry>> {
        let mut query = String::from(
            r#"
            SELECT id, operation_type, entity_type, entity_id,
                   old_value, new_value, initiator, error_message, timestamp
            FROM audit_log
            WHERE 1=1
            "#,
        );

        // Build dynamic WHERE clause
        if filter.operation_type.is_some() {
            query.push_str(" AND operation_type = ?");
        }
        if filter.entity_type.is_some() {
            query.push_str(" AND entity_type = ?");
        }
        if filter.entity_id.is_some() {
            query.push_str(" AND entity_id = ?");
        }
        if filter.start_date.is_some() {
            query.push_str(" AND timestamp >= ?");
        }
        if filter.end_date.is_some() {
            query.push_str(" AND timestamp <= ?");
        }
        if filter.initiator.is_some() {
            query.push_str(" AND initiator = ?");
        }

        // Add ordering and pagination
        query.push_str(" ORDER BY timestamp DESC LIMIT ? OFFSET ?");

        // Build query with bindings
        let mut sql_query = sqlx::query_as::<_, (
            i64,
            String,
            String,
            Option<i64>,
            Option<String>,
            Option<String>,
            String,
            Option<String>,
            String,
        )>(&query);

        if let Some(op_type) = &filter.operation_type {
            sql_query = sql_query.bind(op_type);
        }
        if let Some(ent_type) = &filter.entity_type {
            sql_query = sql_query.bind(ent_type);
        }
        if let Some(ent_id) = filter.entity_id {
            sql_query = sql_query.bind(ent_id);
        }
        if let Some(start) = filter.start_date {
            sql_query = sql_query.bind(start.to_rfc3339());
        }
        if let Some(end) = filter.end_date {
            sql_query = sql_query.bind(end.to_rfc3339());
        }
        if let Some(init) = &filter.initiator {
            sql_query = sql_query.bind(init);
        }

        sql_query = sql_query.bind(pagination.limit).bind(pagination.offset);

        let rows = sql_query
            .fetch_all(&*self.pool)
            .await
            .context("Failed to query audit logs")?;

        let entries = rows
            .into_iter()
            .map(
                |(id, op_type, ent_type, ent_id, old_val, new_val, init, err_msg, ts)| {
                    AuditLogEntry {
                        id,
                        operation_type: op_type,
                        entity_type: ent_type,
                        entity_id: ent_id,
                        old_value: old_val.and_then(|s| serde_json::from_str(&s).ok()),
                        new_value: new_val.and_then(|s| serde_json::from_str(&s).ok()),
                        initiator: init,
                        error_message: err_msg,
                        timestamp: DateTime::parse_from_rfc3339(&ts)
                            .ok()
                            .map(|dt| dt.with_timezone(&Utc))
                            .unwrap_or_else(Utc::now),
                    }
                },
            )
            .collect();

        Ok(entries)
    }

    /// Archive old audit log entries
    /// Moves entries older than the retention period to an archive table
    pub async fn archive_old_entries(&self, retention_period: Duration) -> Result<usize> {
        let cutoff_date = Utc::now() - retention_period;

        // Count entries to be archived
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM audit_log
            WHERE timestamp < ?
            "#,
        )
        .bind(cutoff_date.to_rfc3339())
        .fetch_one(&*self.pool)
        .await
        .context("Failed to count old audit log entries")?;

        if count.0 == 0 {
            return Ok(0);
        }

        // Create archive table if it doesn't exist
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS audit_log_archive (
                id INTEGER PRIMARY KEY,
                operation_type TEXT NOT NULL,
                entity_type TEXT NOT NULL,
                entity_id INTEGER,
                old_value TEXT,
                new_value TEXT,
                initiator TEXT NOT NULL,
                error_message TEXT,
                timestamp DATETIME NOT NULL,
                archived_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&*self.pool)
        .await
        .context("Failed to create audit_log_archive table")?;

        // Move old entries to archive
        sqlx::query(
            r#"
            INSERT INTO audit_log_archive 
                (id, operation_type, entity_type, entity_id, old_value, new_value, initiator, error_message, timestamp)
            SELECT id, operation_type, entity_type, entity_id, old_value, new_value, initiator, error_message, timestamp
            FROM audit_log
            WHERE timestamp < ?
            "#,
        )
        .bind(cutoff_date.to_rfc3339())
        .execute(&*self.pool)
        .await
        .context("Failed to archive old audit log entries")?;

        // Delete archived entries from main table
        let result = sqlx::query(
            r#"
            DELETE FROM audit_log
            WHERE timestamp < ?
            "#,
        )
        .bind(cutoff_date.to_rfc3339())
        .execute(&*self.pool)
        .await
        .context("Failed to delete archived audit log entries")?;

        Ok(result.rows_affected() as usize)
    }

    /// Get the total size of the audit log table
    pub async fn get_log_size(&self) -> Result<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM audit_log")
            .fetch_one(&*self.pool)
            .await
            .context("Failed to get audit log size")?;

        Ok(count)
    }

    /// Check if archival is needed based on size threshold
    pub async fn should_archive(&self, size_threshold: i64) -> Result<bool> {
        let size = self.get_log_size().await?;
        Ok(size > size_threshold)
    }
}
