// Library Management System
// Provides comprehensive media library organization, version tracking, metadata management,
// and maintenance capabilities for TorrentFlix.

pub mod models;
pub mod database;
pub mod queries;
pub mod manager;
pub mod quality_scorer;

// Re-export public types and interfaces
pub use models::*;
pub use database::LibraryDatabase;
pub use queries::{LibraryQueries, StorageStats};
pub use manager::LibraryManager;
pub use quality_scorer::{QualityScorer, QualityWeights, RankedVersion};
