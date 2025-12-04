// Import Pipeline Module
// Handles file import workflow from staging through commit

pub mod pipeline;
pub mod models;
pub mod metadata_prober;

// Re-export public types
pub use pipeline::ImportPipeline;
pub use models::*;
pub use metadata_prober::{MetadataProber, TechnicalMetadata, VideoInfo, AudioTrack, SubtitleTrack};

// Re-export import profile types
pub use models::{ImportProfile, ImportProfileId, QualityFilters, CollectionId, ImportActions};
