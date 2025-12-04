// Database module - single source of truth for all database operations

pub mod connection;
pub mod jobs;
pub mod library;
pub mod notifications;
pub mod ratings;
pub mod settings;
pub mod transaction;

pub use connection::Database;
pub use jobs::{JobDatabase, Job, JobStatus};
pub use library::LibraryDatabase;
pub use notifications::NotificationDatabase;
pub use ratings::{RatingCacheDatabase, CachedRatingRecord};
pub use settings::{SettingsDatabase, SettingRecord};
pub use transaction::{RetryConfig, with_retry, with_transaction, is_retryable_error};

