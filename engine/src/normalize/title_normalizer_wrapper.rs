use anyhow::Result;

use crate::models::{MediaType, ParsedMedia, TorrentResult};
use crate::normalize::MediaNormalizer;

/// Wrapper for the advanced title normalizer core
pub struct TitleNormalizerWrapper;

impl TitleNormalizerWrapper {
    pub fn new() -> Self {
        Self
    }

    fn convert_media_type(core_type: &str) -> MediaType {
        match core_type.to_lowercase().as_str() {
            "movie" => MediaType::Movie,
            "series" | "episode" => MediaType::Series,
            "documentary" => MediaType::Documentary,
            "anime" => MediaType::Anime,
            _ => MediaType::Other,
        }
    }

    fn core_to_parsed_media(
        &self,
        core_result: crate::normalize::title_normalizer_core::ParsedTorrentMetadata,
    ) -> Result<ParsedMedia> {
        let title = core_result.title;
        let media_type = if core_result.tags.contains_key("episode")
            && !core_result.tags["episode"].is_empty()
        {
            MediaType::Series
        } else {
            MediaType::Movie
        };

        let mut parsed = ParsedMedia::new(title, media_type);

        // Extract year
        if let Some(year) = core_result.year {
            parsed.year = Some(year as u32);
        }

        // Extract season and episode from tags
        if let Some(episodes) = core_result.tags.get("episode") {
            if !episodes.is_empty() {
                // Parse season/episode from the first episode tag
                if let Some(episode_str) = episodes.first() {
                    let (season, episode) = self.parse_season_episode(episode_str);
                    parsed.season = season;
                    parsed.episode = episode;
                }
            }
        }

        // Extract quality information from tags
        if let Some(resolutions) = core_result.tags.get("resolution") {
            if !resolutions.is_empty() {
                parsed.resolution = Some(resolutions[0].clone());
            }
        }

        if let Some(sources) = core_result.tags.get("source") {
            if !sources.is_empty() {
                parsed.source = Some(sources[0].clone());
            }
        }

        if let Some(codecs) = core_result.tags.get("codec") {
            if !codecs.is_empty() {
                parsed.codec = Some(codecs[0].clone());
            }
        }

        if let Some(audio_codecs) = core_result.tags.get("audio") {
            if !audio_codecs.is_empty() {
                parsed.audio = Some(audio_codecs[0].clone());
            }
        }

        if let Some(groups) = core_result.tags.get("group") {
            if !groups.is_empty() {
                parsed.release_group = Some(groups[0].clone());
            }
        }

        if let Some(languages) = core_result.tags.get("language") {
            if !languages.is_empty() {
                parsed.language = Some(languages[0].clone());
            }
        }

        Ok(parsed)
    }

    fn parse_season_episode(&self, episode_str: &str) -> (Option<u32>, Option<u32>) {
        // Parse patterns like "S01E02", "1x02", etc.
        if let Ok(re) = regex::Regex::new(r"(?i)S(\d{1,2})E(\d{1,2})") {
            if let Some(caps) = re.captures(episode_str) {
                if let (Some(season), Some(episode)) = (caps.get(1), caps.get(2)) {
                    if let (Ok(s), Ok(e)) = (
                        season.as_str().parse::<u32>(),
                        episode.as_str().parse::<u32>(),
                    ) {
                        return (Some(s), Some(e));
                    }
                }
            }
        }

        if let Ok(re) = regex::Regex::new(r"(?i)(\d{1,2})x(\d{1,2})") {
            if let Some(caps) = re.captures(episode_str) {
                if let (Some(season), Some(episode)) = (caps.get(1), caps.get(2)) {
                    if let (Ok(s), Ok(e)) = (
                        season.as_str().parse::<u32>(),
                        episode.as_str().parse::<u32>(),
                    ) {
                        return (Some(s), Some(e));
                    }
                }
            }
        }

        (None, None)
    }
}

impl MediaNormalizer for TitleNormalizerWrapper {
    fn normalize(&self, torrent_result: &TorrentResult) -> Result<ParsedMedia> {
        let core_result = crate::normalize::title_normalizer_core::parse_torrent_metadata(
            torrent_result.title.clone(),
        );
        self.core_to_parsed_media(core_result)
    }

    fn name(&self) -> &str {
        "TitleNormalizerCore"
    }
}

impl Default for TitleNormalizerWrapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TorrentResult;

    #[test]
    fn test_title_normalizer_wrapper() {
        let wrapper = TitleNormalizerWrapper::new();
        let torrent = TorrentResult::new(
            "Inception.2010.1080p.BluRay.x264-SPARKS".to_string(),
            "magnet:?xt=urn:btih:test".to_string(),
            1_500_000_000,
            100,
            50,
            "TestIndexer".to_string(),
        );

        let result = wrapper.normalize(&torrent);

        // Should successfully parse
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.title, "Inception");
        assert_eq!(parsed.year, Some(2010));
        assert_eq!(parsed.resolution, Some("1080p".to_string()));
        assert_eq!(parsed.source, Some("BluRay".to_string()));
        assert_eq!(parsed.codec, Some("x264".to_string()));
        assert_eq!(parsed.release_group, Some("SPARKS".to_string()));
    }

    #[test]
    fn test_series_normalization() {
        let wrapper = TitleNormalizerWrapper::new();
        let torrent = TorrentResult::new(
            "Breaking.Bad.S01E01.2008.720p.BluRay.x264-REWARDERS".to_string(),
            "magnet:?xt=urn:btih:test".to_string(),
            1_200_000_000,
            150,
            75,
            "TestIndexer".to_string(),
        );

        let result = wrapper.normalize(&torrent).unwrap();

        assert_eq!(result.title, "Breaking Bad");
        assert_eq!(result.year, Some(2008));
        assert_eq!(result.season, Some(1));
        assert_eq!(result.episode, Some(1));
        assert_eq!(result.media_type, MediaType::Series);
        assert_eq!(result.resolution, Some("720p".to_string()));
    }
}
