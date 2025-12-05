// Torrent client integration module for qBittorrent communication

pub mod client;
pub mod status;
pub mod control;

pub use client::TorrentClient;
pub use status::DownloadTracker;
pub use control::TorrentController;
