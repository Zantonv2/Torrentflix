pub mod models;
pub mod indexers;
pub mod normalize;
pub mod metadata;
pub mod dedupe;
pub mod scoring;
pub mod torrent;
pub mod filesystem;
pub mod library;
pub mod import;
pub mod maintenance;
pub mod database;
pub mod jobs;
pub mod config;
pub mod settings;
pub mod ui;
pub mod engine;
pub mod ratings;
pub mod watch;
pub mod audit;
pub mod notifications;
pub mod desktop;

pub use models::*;
pub use engine::{Engine, RatingUpdate};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_torrent_result_creation() {
        let torrent = TorrentResult::new(
            "Test Movie 2023 1080p BluRay".to_string(),
            "magnet:?xt=urn:btih:test".to_string(),
            1_500_000_000,
            100,
            50,
            "TestIndexer".to_string(),
        );

        assert_eq!(torrent.title, "Test Movie 2023 1080p BluRay");
        assert_eq!(torrent.size_bytes, 1_500_000_000);
        assert_eq!(torrent.seeders, 100);
        assert_eq!(torrent.leechers, 50);
        assert_eq!(torrent.source, "TestIndexer");
    }

    #[test]
    fn test_parsed_media_identity_key() {
        let movie = ParsedMedia::new("Inception".to_string(), MediaType::Movie)
            .with_year(2010);
        assert_eq!(movie.identity_key(), "Inception_2010");

        let series = ParsedMedia::new("Breaking Bad".to_string(), MediaType::Series)
            .with_season(1)
            .with_episode(1);
        assert_eq!(series.identity_key(), "Breaking Bad_S01E01");
    }

    #[test]
    fn test_library_item_creation() {
        let parsed = ParsedMedia::new("Test Movie".to_string(), MediaType::Movie);
        let item = LibraryItem::new(
            "/path/to/movie.mkv".to_string(),
            parsed,
            2_000_000_000,
        );

        assert_eq!(item.file_path, "/path/to/movie.mkv");
        assert_eq!(item.file_size_bytes, 2_000_000_000);
        assert_eq!(item.watch_count, 0);
        assert!(!item.is_favorite);
    }
}
