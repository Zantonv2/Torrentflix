// Notification data models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Unique identifier for a notification
pub type NotificationId = i64;

/// Severity level for notifications
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl NotificationSeverity {
    /// Convert to string for database storage
    pub fn as_str(&self) -> &'static str {
        match self {
            NotificationSeverity::Info => "info",
            NotificationSeverity::Warning => "warning",
            NotificationSeverity::Error => "error",
            NotificationSeverity::Critical => "critical",
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "info" => Some(NotificationSeverity::Info),
            "warning" => Some(NotificationSeverity::Warning),
            "error" => Some(NotificationSeverity::Error),
            "critical" => Some(NotificationSeverity::Critical),
            _ => None,
        }
    }
}

/// A notification to be displayed to the user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: NotificationId,
    pub severity: NotificationSeverity,
    pub title: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub read: bool,
    pub dismissed: bool,
    pub read_at: Option<DateTime<Utc>>,
    pub dismissed_at: Option<DateTime<Utc>>,
}

impl Notification {
    /// Create a new notification (without ID, for insertion)
    pub fn new(severity: NotificationSeverity, title: String, message: String) -> Self {
        Self {
            id: 0, // Will be set by database
            severity,
            title,
            message,
            timestamp: Utc::now(),
            read: false,
            dismissed: false,
            read_at: None,
            dismissed_at: None,
        }
    }

    /// Check if notification is unread
    pub fn is_unread(&self) -> bool {
        !self.read
    }

    /// Check if notification is dismissed
    pub fn is_dismissed(&self) -> bool {
        self.dismissed
    }
}

/// Builder for creating notifications
pub struct NotificationBuilder {
    severity: NotificationSeverity,
    title: String,
    message: String,
}

impl NotificationBuilder {
    pub fn new(severity: NotificationSeverity) -> Self {
        Self {
            severity,
            title: String::new(),
            message: String::new(),
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    pub fn build(self) -> Notification {
        Notification::new(self.severity, self.title, self.message)
    }
}
