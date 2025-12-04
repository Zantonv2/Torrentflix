// Database operations for notifications

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use sqlx::SqlitePool;

use crate::notifications::models::{Notification, NotificationId, NotificationSeverity};

/// Database interface for notification operations
pub struct NotificationDatabase {
    pool: SqlitePool,
}

impl NotificationDatabase {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Create a new notification in the database
    pub async fn create_notification(&self, notification: &Notification) -> Result<NotificationId> {
        let severity = notification.severity.as_str();
        let read = if notification.read { 1 } else { 0 };
        let dismissed = if notification.dismissed { 1 } else { 0 };
        
        let id = sqlx::query(
            r#"
            INSERT INTO notifications (severity, title, message, read, dismissed, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(severity)
        .bind(&notification.title)
        .bind(&notification.message)
        .bind(read)
        .bind(dismissed)
        .bind(notification.timestamp)
        .execute(&self.pool)
        .await?
        .last_insert_rowid();

        Ok(id)
    }

    /// Get a notification by ID
    pub async fn get_notification(&self, id: NotificationId) -> Result<Option<Notification>> {
        let record = sqlx::query_as::<_, (i64, String, String, String, DateTime<Utc>, i64, i64, Option<DateTime<Utc>>, Option<DateTime<Utc>>)>(
            r#"
            SELECT id, severity, title, message, created_at, read, dismissed, read_at, dismissed_at
            FROM notifications
            WHERE id = ?
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(record.map(|(id, severity, title, message, created_at, read, dismissed, read_at, dismissed_at)| Notification {
            id,
            severity: NotificationSeverity::from_str(&severity).unwrap_or(NotificationSeverity::Info),
            title,
            message,
            timestamp: created_at,
            read: read != 0,
            dismissed: dismissed != 0,
            read_at,
            dismissed_at,
        }))
    }

    /// Get all unread notifications
    pub async fn get_unread_notifications(&self) -> Result<Vec<Notification>> {
        let records = sqlx::query_as::<_, (i64, String, String, String, DateTime<Utc>, i64, i64, Option<DateTime<Utc>>, Option<DateTime<Utc>>)>(
            r#"
            SELECT id, severity, title, message, created_at, read, dismissed, read_at, dismissed_at
            FROM notifications
            WHERE read = 0 AND dismissed = 0
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records
            .into_iter()
            .map(|(id, severity, title, message, created_at, read, dismissed, read_at, dismissed_at)| Notification {
                id,
                severity: NotificationSeverity::from_str(&severity).unwrap_or(NotificationSeverity::Info),
                title,
                message,
                timestamp: created_at,
                read: read != 0,
                dismissed: dismissed != 0,
                read_at,
                dismissed_at,
            })
            .collect())
    }

    /// Get all notifications (including read and dismissed)
    pub async fn get_all_notifications(&self, limit: Option<i64>) -> Result<Vec<Notification>> {
        let limit = limit.unwrap_or(100);
        
        let records = sqlx::query_as::<_, (i64, String, String, String, DateTime<Utc>, i64, i64, Option<DateTime<Utc>>, Option<DateTime<Utc>>)>(
            r#"
            SELECT id, severity, title, message, created_at, read, dismissed, read_at, dismissed_at
            FROM notifications
            ORDER BY created_at DESC
            LIMIT ?
            "#
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(records
            .into_iter()
            .map(|(id, severity, title, message, created_at, read, dismissed, read_at, dismissed_at)| Notification {
                id,
                severity: NotificationSeverity::from_str(&severity).unwrap_or(NotificationSeverity::Info),
                title,
                message,
                timestamp: created_at,
                read: read != 0,
                dismissed: dismissed != 0,
                read_at,
                dismissed_at,
            })
            .collect())
    }

    /// Mark a notification as read
    pub async fn mark_as_read(&self, id: NotificationId) -> Result<()> {
        let now = Utc::now();
        
        sqlx::query(
            r#"
            UPDATE notifications
            SET read = 1, read_at = ?
            WHERE id = ?
            "#
        )
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Dismiss a notification
    pub async fn dismiss_notification(&self, id: NotificationId) -> Result<()> {
        let now = Utc::now();
        
        sqlx::query(
            r#"
            UPDATE notifications
            SET dismissed = 1, dismissed_at = ?
            WHERE id = ?
            "#
        )
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Delete notifications older than the specified duration
    pub async fn cleanup_old_notifications(&self, older_than: Duration) -> Result<usize> {
        let cutoff = Utc::now() - older_than;
        
        let result = sqlx::query(
            r#"
            DELETE FROM notifications
            WHERE created_at < ? AND (dismissed = 1 OR read = 1)
            "#
        )
        .bind(cutoff)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as usize)
    }

    /// Get count of unread notifications
    pub async fn get_unread_count(&self) -> Result<i64> {
        let (count,): (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) as count
            FROM notifications
            WHERE read = 0 AND dismissed = 0
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }
}
