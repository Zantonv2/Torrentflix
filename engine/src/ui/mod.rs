// UI module for Tauri frontend integration
// Provides minimal Netflix-style DTOs and commands

pub mod dto;
pub mod commands;

pub use dto::{UiSearchResult, UiSearchRequest, UiSearchResponse, UiTorrentInfo};
pub use commands::*;
