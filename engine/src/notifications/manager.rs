// Notification manager with Linux desktop integration

use anyhow::Result;
use chrono::Duration;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::database::NotificationDatabase;
use crate::notifications::models::{Notification, NotificationId, NotificationSeverity};

/// Backend for sending desktop notifications
#[derive(Debug, Clone)]
enum NotificationBackend {
    /// Use libnotify/D-Bus for desktop notifications
    Desktop,
    /// Fallback for non-desktop environments (database only)
    Fallback,
}

/// Manager for creating and managing notifications
pub struct NotificationManager {
    db: NotificationDatabase,
    backend: NotificationBackend,
    notifications: Arc<Mutex<Vec<Notification>>>,
}

impl NotificationManager {
    /// Create a new notification manager
    pub fn new(db: NotificationDatabase) -> Self {
        let backend = Self::detect_backend();
        
        Self {
            db,
            backend,
            notifications: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Detect the best notification backend for the current environment
    fn detect_backend() -> NotificationBackend {
        // Check if we're in a desktop environment
        // For now, we'll use a simple check for DISPLAY or WAYLAND_DISPLAY
        if std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok() {
            NotificationBackend::Desktop
        } else {
            NotificationBackend::Fallback
        }
    }

    /// Send a notification
    /// Creates a notification record in the database and optionally sends to desktop
    pub async fn send_notification(&self, notification: Notification) -> Result<NotificationId> {
        // Store in database first
        let id = self.db.create_notification(&notification).await?;
        
        // Update the notification with the ID
        let mut notification = notification;
        notification.id = id;
        
        // Send to desktop notification system if available
        self.send_to_desktop(&notification).await?;
        
        // Cache in memory
        let mut notifications = self.notifications.lock().await;
        notifications.push(notification);
        
        Ok(id)
    }

    /// Send notification to desktop notification system
    async fn send_to_desktop(&self, notification: &Notification) -> Result<()> {
        match self.backend {
            NotificationBackend::Desktop => {
                // Try to send via notify-rust or similar
                // For now, we'll implement a basic version
                #[cfg(target_os = "linux")]
                {
                    self.send_via_notify_send(notification).await?;
                }
            }
            NotificationBackend::Fallback => {
                // No desktop notification, just store in database
            }
        }
        
        Ok(())
    }

    /// Send notification using notify-send command (fallback method)
    #[cfg(target_os = "linux")]
    async fn send_via_notify_send(&self, notification: &Notification) -> Result<()> {
        use tokio::process::Command;
        
        let urgency = match notification.severity {
            NotificationSeverity::Info => "low",
            NotificationSeverity::Warning => "normal",
            NotificationSeverity::Error => "normal",
            NotificationSeverity::Critical => "critical",
        };
        
        // Try to send via notify-send if available
        let _ = Command::new("notify-send")
            .arg("-u")
            .arg(urgency)
            .arg(&notification.title)
            .arg(&notification.message)
            .spawn();
        
        Ok(())
    }

    /// Get all unread notifications
    pub async fn get_unread_notifications(&self) -> Result<Vec<Notification>> {
        self.db.get_unread_notifications().await
    }

    /// Get all notifications with optional limit
    pub async fn get_all_notifications(&self, limit: Option<i64>) -> Result<Vec<Notification>> {
        self.db.get_all_notifications(limit).await
    }

    /// Mark a notification as read
    pub async fn mark_as_read(&self, id: NotificationId) -> Result<()> {
        self.db.mark_as_read(id).await?;
        
        // Update in-memory cache
        let mut notifications = self.notifications.lock().await;
        if let Some(notification) = notifications.iter_mut().find(|n| n.id == id) {
            notification.read = true;
            notification.read_at = Some(chrono::Utc::now());
        }
        
        Ok(())
    }

    /// Dismiss a notification
    pub async fn dismiss_notification(&self, id: NotificationId) -> Result<()> {
        self.db.dismiss_notification(id).await?;
        
        // Update in-memory cache
        let mut notifications = self.notifications.lock().await;
        if let Some(notification) = notifications.iter_mut().find(|n| n.id == id) {
            notification.dismissed = true;
            notification.dismissed_at = Some(chrono::Utc::now());
        }
        
        Ok(())
    }

    /// Clean up old notifications
    /// Removes notifications older than the specified duration that are read or dismissed
    pub async fn cleanup_old_notifications(&self, older_than: Duration) -> Result<usize> {
        let count = self.db.cleanup_old_notifications(older_than).await?;
        
        // Clean up in-memory cache
        let mut notifications = self.notifications.lock().await;
        let cutoff = chrono::Utc::now() - older_than;
        notifications.retain(|n| {
            n.timestamp > cutoff || (!n.read && !n.dismissed)
        });
        
        Ok(count)
    }

    /// Get count of unread notifications
    pub async fn get_unread_count(&self) -> Result<i64> {
        self.db.get_unread_count().await
    }

    /// Create a job completion notification
    pub async fn notify_job_completion(
        &self,
        job_type: &str,
        status: &str,
        summary: &str,
    ) -> Result<NotificationId> {
        let severity = if status == "completed" {
            NotificationSeverity::Info
        } else if status == "failed" {
            NotificationSeverity::Error
        } else {
            NotificationSeverity::Warning
        };
        
        let notification = Notification::new(
            severity,
            format!("{} {}", job_type, status),
            summary.to_string(),
        );
        
        self.send_notification(notification).await
    }

    /// Create a corruption alert notification
    pub async fn notify_corruption_detected(
        &self,
        file_path: &str,
        details: &str,
    ) -> Result<NotificationId> {
        let notification = Notification::new(
            NotificationSeverity::Critical,
            "File Corruption Detected".to_string(),
            format!("File: {}\n{}", file_path, details),
        );
        
        self.send_notification(notification).await
    }

    /// Create a disk space warning notification
    pub async fn notify_disk_space_warning(
        &self,
        current_usage: u64,
        threshold: u64,
        suggestions: &str,
    ) -> Result<NotificationId> {
        let notification = Notification::new(
            NotificationSeverity::Warning,
            "Disk Space Warning".to_string(),
            format!(
                "Disk usage: {} / {} ({}%)\n{}",
                Self::format_bytes(current_usage),
                Self::format_bytes(threshold),
                (current_usage * 100) / threshold,
                suggestions
            ),
        );
        
        self.send_notification(notification).await
    }

    /// Create a missing files notification
    pub async fn notify_missing_files(
        &self,
        count: usize,
        affected_items: &str,
    ) -> Result<NotificationId> {
        let notification = Notification::new(
            NotificationSeverity::Warning,
            format!("{} Missing Files Detected", count),
            format!("Affected items:\n{}", affected_items),
        );
        
        self.send_notification(notification).await
    }

    /// Create a watch folder failure notification
    pub async fn notify_watch_folder_failure(
        &self,
        file_path: &str,
        error_reason: &str,
    ) -> Result<NotificationId> {
        let notification = Notification::new(
            NotificationSeverity::Error,
            "Watch Folder Import Failed".to_string(),
            format!("File: {}\nReason: {}", file_path, error_reason),
        );
        
        self.send_notification(notification).await
    }

    /// Format bytes for human-readable display
    fn format_bytes(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;
        
        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }
        
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(NotificationManager::format_bytes(0), "0.00 B");
        assert_eq!(NotificationManager::format_bytes(1024), "1.00 KB");
        assert_eq!(NotificationManager::format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(NotificationManager::format_bytes(1024 * 1024 * 1024), "1.00 GB");
    }
}
