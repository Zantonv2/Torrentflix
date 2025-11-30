// Database module - single source of truth for all database operations

pub mod connection;
pub mod jobs;
pub mod library;
pub mod ratings;

pub use connection::Database;
pub use jobs::{JobDatabase, Job, JobStatus};
pub use library::LibraryDatabase;
pub use ratings::{RatingCacheDatabase, CachedRatingRecord};

