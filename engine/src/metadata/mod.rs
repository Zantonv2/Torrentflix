// Metadata enrichment module
// Provides TMDb, Kinopoisk, IMDb clients with caching layer

pub mod models;
pub mod traits;
pub mod tmdb;
pub mod kinopoisk;
pub mod imdb;
pub mod cache;

pub use models::{MediaMetadata, MetadataSearchResult, MediaType as MetadataMediaType};
pub use traits::MetadataProvider;
pub use tmdb::TmdbClient;
pub use kinopoisk::KinopoiskClient;
pub use imdb::ImdbClient;
pub use cache::{MetadataCache, MetadataManager};
