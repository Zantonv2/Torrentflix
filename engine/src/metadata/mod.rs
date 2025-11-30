// Metadata enrichment module
// Provides TMDb client with caching layer for general metadata (posters, descriptions, etc.)

pub mod models;
pub mod traits;
pub mod tmdb;
pub mod cache;

pub use models::{MediaMetadata, MetadataSearchResult, MediaType as MetadataMediaType};
pub use traits::MetadataProvider;
pub use tmdb::TmdbClient;
pub use cache::{MetadataCache, MetadataManager};
