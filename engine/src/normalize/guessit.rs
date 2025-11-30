use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::models::{TorrentResult, ParsedMedia, MediaType};
use crate::normalize::MediaNormalizer;

/// Guessit-based normalizer (placeholder implementation)
/// TODO: Implement actual guessit-rs integration when available
pub struct GuessitNormalizer {
    enabled: bool,
}

impl GuessitNormalizer {
    pub fn new() -> Self {
        Self {
            enabled: false, // Disabled until guessit-rs is implemented
        }
    }

    #[allow(dead_code)]
    fn call_guessit(&self, _title: &str) -> Result<GuessitResult> {
        if !self.enabled {
            return Err(anyhow!("Guessit normalizer not enabled"));
        }

        // Placeholder for actual guessit-rs integration
        Err(anyhow!("Guessit integration not yet implemented"))
    }

    fn guessit_to_parsed_media(&self, guessit_result: GuessitResult) -> ParsedMedia {
        let mut media = ParsedMedia::new(
            guessit_result.title.unwrap_or_else(|| "Unknown".to_string()),
            guessit_result.media_type.unwrap_or(MediaType::Movie),
        );

        media.year = guessit_result.year;
        media.season = guessit_result.season;
        media.episode = guessit_result.episode;
        media.resolution = guessit_result.resolution;
        media.source = guessit_result.source;
        media.codec = guessit_result.video_codec;
        media.audio = guessit_result.audio_codec;
        media.release_group = guessit_result.release_group;
        media.language = guessit_result.language;
        media.subtitles = guessit_result.subtitle;

        media
    }
}

impl MediaNormalizer for GuessitNormalizer {
    fn normalize(&self, torrent_result: &TorrentResult) -> Result<ParsedMedia> {
        if !self.enabled {
            return Err(anyhow!("Guessit normalizer not enabled"));
        }

        match self.call_guessit(&torrent_result.title) {
            Ok(guessit_result) => Ok(self.guessit_to_parsed_media(guessit_result)),
            Err(e) => Err(anyhow!("Guessit normalization failed: {}", e)),
        }
    }

    fn name(&self) -> &str {
        "GuessitNormalizer"
    }
}

impl Default for GuessitNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Result structure from guessit parsing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuessitResult {
    pub title: Option<String>,
    pub year: Option<u32>,
    pub season: Option<u32>,
    pub episode: Option<u32>,
    pub resolution: Option<String>,
    pub source: Option<String>,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub release_group: Option<String>,
    pub media_type: Option<MediaType>,
    pub language: Option<String>,
    pub subtitle: Option<Vec<String>>,
    pub edition: Option<String>,
    pub part: Option<u32>,
    pub cd: Option<u32>,
    pub cd_count: Option<u32>,
    pub container: Option<String>,
    pub format: Option<String>,
    pub archive: Option<String>,
    pub website: Option<String>,
    pub source_media: Option<String>,
    pub video_api: Option<String>,
    pub audio_channels: Option<String>,
    pub screen_size: Option<String>,
    pub other: Option<HashMap<String, serde_json::Value>>,
}

impl GuessitResult {
    pub fn new() -> Self {
        Self {
            title: None,
            year: None,
            season: None,
            episode: None,
            resolution: None,
            source: None,
            video_codec: None,
            audio_codec: None,
            release_group: None,
            media_type: None,
            language: None,
            subtitle: None,
            edition: None,
            part: None,
            cd: None,
            cd_count: None,
            container: None,
            format: None,
            archive: None,
            website: None,
            source_media: None,
            video_api: None,
            audio_channels: None,
            screen_size: None,
            other: None,
        }
    }
}

impl Default for GuessitResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock guessit implementation for testing
#[cfg(test)]
pub mod mock {
    use super::*;
    use crate::normalize::utils::*;

    pub fn mock_guessit_parse(title: &str) -> GuessitResult {
        let mut result = GuessitResult::new();

        // Extract basic components using utility functions
        result.title = Some(clean_title(title.to_string()));
        result.year = extract_year(title);
        result.resolution = extract_resolution(title);
        result.source = extract_source(title);
        result.video_codec = extract_codec(title);
        result.audio_codec = extract_audio(title);
        result.release_group = extract_release_group(title);

        // Extract season/episode if present
        if let Some((season, episode)) = extract_season_episode(title) {
            result.season = Some(season);
            result.episode = Some(episode);
            result.media_type = Some(MediaType::Series);
        } else {
            result.media_type = Some(MediaType::Movie);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::mock::mock_guessit_parse;
    use crate::models::TorrentResult;

    #[test]
    fn test_guessit_normalizer_disabled() {
        let normalizer = GuessitNormalizer::new();
        let torrent = TorrentResult::new(
            "Test Movie 2023".to_string(),
            "magnet:?xt=urn:btih:test".to_string(),
            1_000_000_000,
            100,
            50,
            "TestIndexer".to_string(),
        );

        let result = normalizer.normalize(&torrent);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not enabled"));
    }

    #[test]
    fn test_mock_guessit_parse() {
        let result = mock_guessit_parse("Inception.2010.1080p.BluRay.x264-SPARKS");
        
        assert_eq!(result.title, Some("Inception 2010".to_string()));
        assert_eq!(result.year, Some(2010));
        assert_eq!(result.resolution, Some("1080P".to_string()));
        assert_eq!(result.source, Some("BluRay".to_string()));
        assert_eq!(result.video_codec, Some("H.264".to_string()));
        assert_eq!(result.release_group, Some("SPARKS".to_string()));
        assert_eq!(result.media_type, Some(MediaType::Movie));
    }

    #[test]
    fn test_mock_guessit_parse_series() {
        let result = mock_guessit_parse("Breaking.Bad.S01E01.2008.720p.BluRay.x264-REWARDERS");
        
        assert_eq!(result.title, Some("Breaking Bad 2008".to_string()));
        assert_eq!(result.year, Some(2008));
        assert_eq!(result.season, Some(1));
        assert_eq!(result.episode, Some(1));
        assert_eq!(result.resolution, Some("720P".to_string()));
        assert_eq!(result.media_type, Some(MediaType::Series));
    }
}
