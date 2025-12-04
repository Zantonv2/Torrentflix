// Notification system module

pub mod models;
pub mod manager;

pub use models::{Notification, NotificationId, NotificationSeverity};
pub use manager::NotificationManager;
