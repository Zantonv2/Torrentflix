/// Rating Manager and rating fetching workers
/// 
/// This module handles fetching ratings from Kinopoisk and IMDB,
/// coordinating workers, and managing the rating cache.

pub mod kinopoisk;
pub mod imdb;
pub mod models;
pub mod cache;
pub mod manager;

// Re-export RatingTitleInfo from normalize module for convenience
pub use crate::normalize::RatingTitleInfo;
pub use kinopoisk::KinopoiskClient;
pub use imdb::ImdbClient;
pub use manager::RatingManager;
pub use models::{RatingWorkerResult, RatingSource, TMDBMetadataResult};
pub use cache::RatingCache;

