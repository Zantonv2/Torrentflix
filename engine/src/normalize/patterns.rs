use anyhow::{Result, anyhow};
use regex::Regex;
use std::collections::HashMap;

use crate::models::{TorrentResult, ParsedMedia, MediaType};
use crate::normalize::MediaNormalizer;

/// Pattern-based normalizer using regex patterns
pub struct PatternNormalizer {
    patterns: HashMap<String, Regex>,
}

impl PatternNormalizer {
    pub fn new() -> Self {
        let mut patterns = HashMap::new();
        
        // Enhanced year extraction
        patterns.insert("year".to_string(), Regex::new(r"\b(19|20)\d{2}\b").unwrap());
        
        // Enhanced season/episode extraction (from title_normalizer_core)
        patterns.insert("season_episode".to_string(), Regex::new(r"(?i)S(\d{1,2})E(\d{1,2})").unwrap());
        patterns.insert("season_episode_lower".to_string(), Regex::new(r"(?i)s(\d{1,2})e(\d{1,2})").unwrap());
        patterns.insert("season_episode_x".to_string(), Regex::new(r"(?i)(\d{1,2})x(\d{1,2})").unwrap());
        patterns.insert("season_episode_word".to_string(), Regex::new(r"(?i)Season\s+(\d{1,2})\s+Episode\s+(\d{1,2})").unwrap());
        patterns.insert("episode_only".to_string(), Regex::new(r"(?i)E(\d{1,2})|Ep(\d{1,2})|Episode\s+(\d{1,2})").unwrap());
        
        // Enhanced resolution patterns (from title_normalizer_core)
        patterns.insert("resolution".to_string(), 
            Regex::new(r"(?i)(?:(?P<width>\d{3,4})(?:x|\*))?(?P<height>\d{3,4})(?P<scan>p|i)?(?:10bit)?").unwrap());
        patterns.insert("resolution_common".to_string(),
            Regex::new(r"\b(4320p|8k|2160p|4k|1440p|2k|1080p|720p|480p|360p|240p)\b").unwrap());
        
        // Enhanced source patterns (from title_normalizer_core)
        patterns.insert("source".to_string(), 
            Regex::new(r"(?i)(HD-?CAM|HD-?TELESYNC|HD-?TELECINE|HD-?TV|TV-?HD|WEB-?DL|WEB-?RIP|BLU-?RAY|BDRIP|DVDRIP|HDTV|CAM|TELESYNC|TELECINE|DVD|VHS|WEB|DL|RIP|NETFLIX|DISNEY\+|HBO|AMAZON|PRIME|HULU|APPLE|TV\+|PARAMOUNT\+|PEACOCK)").unwrap());
        
        // Enhanced video codec patterns (from title_normalizer_core)
        patterns.insert("codec".to_string(), 
            Regex::new(r"(?i)([HX]-?26[45]|HEVC|AVC|XVID|DIVX|VP[0-9]|MPEG-?2|VC-1)(?:10BIT)?").unwrap());
        
        // Enhanced audio codec patterns (from title_normalizer_core)
        patterns.insert("audio".to_string(), 
            Regex::new(r"(?i)(AAC|AC3|EAC3|DDP|DTS|TRUEHD|ATMOS|FLAC)(?:[0-9]\.[0-9])?").unwrap());
        
        // Enhanced release group patterns (from title_normalizer_core)
        patterns.insert("release_group_brackets".to_string(), 
            Regex::new(r"\[([^\]]+)\]$").unwrap());
        patterns.insert("release_group_hyphen".to_string(), 
            Regex::new(r"(?i)-([A-Za-z0-9]+)$").unwrap());
        patterns.insert("release_group_dot".to_string(), 
            Regex::new(r"\.([a-zA-Z0-9\-]+)$").unwrap());
        
        // Edition tags (from title_normalizer_core)
        patterns.insert("edition".to_string(),
            Regex::new(r"(?i)(PROPER|REPACK|EXTENDED|UNRATED|DIRECTORS\.?CUT|THEATRICAL|IMAX|3D|4K|HDR|DOLBY\.VISION|ATMOS|REMASTERED|CRITERION)").unwrap());
        
        // Container formats
        patterns.insert("container".to_string(),
            Regex::new(r"(?i)\.(mkv|mp4|avi|mov|wmv|flv|webm|m4v|mpg|mpeg|ts|m2ts|iso)$").unwrap());
        
        Self { patterns }
    }

    fn extract_with_pattern(&self, pattern_name: &str, text: &str) -> Option<String> {
        if let Some(re) = self.patterns.get(pattern_name) {
            if let Some(caps) = re.captures(text) {
                // For year pattern, use full match (group 0), for others use group 1
                if pattern_name == "year" {
                    return Some(caps.get(0)?.as_str().to_string());
                } else {
                    return Some(caps.get(1)?.as_str().to_string());
                }
            }
        }
        None
    }

    fn parse_title_components(&self, title: &str) -> ParsedMediaParse {
        let mut parse = ParsedMediaParse::new();
        
        // Extract year
        if let Some(year_str) = self.extract_with_pattern("year", title) {
            if let Ok(year) = year_str.parse::<u32>() {
                parse.year = Some(year);
            }
        }
        
        // Extract season/episode
        for pattern_name in ["season_episode", "season_episode_lower", "season_episode_x"] {
            if let Some(re) = self.patterns.get(pattern_name) {
                if let Some(caps) = re.captures(title) {
                    if let (Some(season), Some(episode)) = (caps.get(1), caps.get(2)) {
                        if let (Ok(s), Ok(e)) = (
                            season.as_str().parse::<u32>(),
                            episode.as_str().parse::<u32>()
                        ) {
                            parse.season = Some(s);
                            parse.episode = Some(e);
                            parse.media_type = MediaType::Series;
                            break;
                        }
                    }
                }
            }
        }
        
        // Extract quality info
        parse.resolution = self.extract_with_pattern("resolution", title)
            .map(|s| s.to_uppercase());
        
        parse.source = self.extract_with_pattern("source", title)
            .map(|s| self.normalize_source(&s));
        
        parse.codec = self.extract_with_pattern("codec", title)
            .map(|s| self.normalize_codec(&s));
        
        parse.audio = self.extract_with_pattern("audio", title)
            .map(|s| self.normalize_audio(&s));
        
        // Extract release group
        for pattern_name in ["release_group_brackets", "release_group_hyphen", "release_group_dot"] {
            if let Some(group) = self.extract_with_pattern(pattern_name, title) {
                if !is_common_suffix(&group) {
                    parse.release_group = Some(group);
                    break;
                }
            }
        }
        
        // Determine media type if not already set
        if parse.media_type == MediaType::Movie {
            parse.media_type = self.guess_media_type_from_title(title);
        }
        
        // Clean title
        parse.title = self.clean_title_for_parsing(title);
        
        parse
    }

    fn normalize_source(&self, source: &str) -> String {
        match source.to_lowercase().as_str() {
            "uhd bluray" => "UHD.BluRay".to_string(),
            "bluray" | "bd" => "BluRay".to_string(),
            "bdrip" | "brrip" => "BDRip".to_string(),
            "web-dl" | "webdl" => "WEB-DL".to_string(),
            "webrip" | "web" => "WEBRip".to_string(),
            "hdtv" => "HDTV".to_string(),
            "dvd" | "dvdrip" => "DVD".to_string(),
            "cam" => "CAM".to_string(),
            "ts" => "TS".to_string(),
            _ => source.to_uppercase(),
        }
    }

    fn normalize_codec(&self, codec: &str) -> String {
        match codec.to_lowercase().as_str() {
            "h.265" | "hevc" => "H.265".to_string(),
            "h264" | "avc" => "H.264".to_string(),
            "mpeg-2" | "mpeg2" => "MPEG-2".to_string(),
            _ => codec.to_uppercase(),
        }
    }

    fn normalize_audio(&self, audio: &str) -> String {
        match audio.to_lowercase().as_str() {
            "dts-hd.ma" => "DTS-HD.MA".to_string(),
            "dts-hd" => "DTS-HD".to_string(),
            "dd+" | "dolby.digital.plus" => "DD+".to_string(),
            _ => audio.to_uppercase(),
        }
    }

    fn clean_title_for_parsing(&self, title: &str) -> String {
        let mut cleaned = title.to_string();
        
        // Remove patterns we've already extracted
        let patterns_to_remove = vec![
            r"\[.*?\]",                // [anything]
            r"\(.*?\)",                // (anything)
            r"\..*?(?:avi|mkv|mp4|mov|wmv|flv|webm)$", // .extension
            r"(?i)\b(1080p|720p|480p|2160p|4k|bluray|web|hdtv|dvd|cam|ts|x264|x265|h264|h265|aac|ac3|dts|proper|repack|extended|unrated|theatrical|internal)\b",
            r"(?i)\b(dts-hd\.ma|dts-hd|truehd|atmos|dd\+)\b",
            r"(?i)S\d{1,2}E\d{1,2}",   // S01E02
            r"(?i)s\d{1,2}e\d{1,2}",   // s01e02
            r"(?i)\d{1,2}x\d{1,2}",    // 1x02
            r"(?i)\b(19|20)\d{2}\b",   // years
            r"(?i)\-[a-zA-Z0-9\-]+$",  // -GroupName
            r"(?i)\.[a-zA-Z0-9\-]+$",  // .GroupName
        ];

        for pattern in patterns_to_remove {
            if let Ok(re) = Regex::new(pattern) {
                cleaned = re.replace_all(&cleaned, "").to_string();
            }
        }

        // Clean up separators
        cleaned = cleaned.replace('.', " ");
        cleaned = cleaned.replace('_', " ");
        cleaned = cleaned.replace('-', " ");
        
        // Remove extra whitespace
        if let Ok(re) = Regex::new(r"\s+") {
            cleaned = re.replace_all(&cleaned, " ").to_string();
        }

        cleaned.trim().to_string()
    }

    fn guess_media_type_from_title(&self, title: &str) -> MediaType {
        let title_lower = title.to_lowercase();
        
        // Check for series indicators
        if title_lower.contains("season") || title_lower.contains("episode") ||
           title_lower.contains("complete") || title_lower.contains("series") {
            return MediaType::Series;
        }
        
        // Check for documentary indicators
        if title_lower.contains("documentary") || title_lower.contains("docu") {
            return MediaType::Documentary;
        }
        
        // Check for anime indicators
        if title_lower.contains("anime") || title_lower.contains("manga") ||
           title_lower.contains("subbed") && title_lower.contains("episode") {
            return MediaType::Anime;
        }
        
        MediaType::Movie
    }
}

// Check if this is a common suffix that should be ignored
fn is_common_suffix(text: &str) -> bool {
    matches!(text.to_lowercase().as_str(), 
        "proper" | "repack" | "extended" | "unrated" | "directors.cut" | "theatrical" | "imax" | "3d" | "4k" | "hdr" |
        "dolby.vision" | "atmos" | "remastered" | "criterion" | "uncut" | "screener" | "internal" |
        "readnfo" | "limited" | "docu" | "documentary"
    )
}

#[derive(Debug, Clone)]
struct ParsedMediaParse {
    title: String,
    year: Option<u32>,
    season: Option<u32>,
    episode: Option<u32>,
    resolution: Option<String>,
    source: Option<String>,
    codec: Option<String>,
    audio: Option<String>,
    release_group: Option<String>,
    media_type: MediaType,
}

impl ParsedMediaParse {
    fn new() -> Self {
        Self {
            title: String::new(),
            year: None,
            season: None,
            episode: None,
            resolution: None,
            source: None,
            codec: None,
            audio: None,
            release_group: None,
            media_type: MediaType::Movie,
        }
    }

    fn to_parsed_media(self) -> ParsedMedia {
        let mut media = ParsedMedia::new(self.title, self.media_type);
        
        media.year = self.year;
        media.season = self.season;
        media.episode = self.episode;
        media.resolution = self.resolution;
        media.source = self.source;
        media.codec = self.codec;
        media.audio = self.audio;
        media.release_group = self.release_group;
        
        media
    }
}

impl MediaNormalizer for PatternNormalizer {
    fn normalize(&self, torrent_result: &TorrentResult) -> Result<ParsedMedia> {
        let parse = self.parse_title_components(&torrent_result.title);
        
        if parse.title.is_empty() {
            return Err(anyhow!("Failed to extract title from: {}", torrent_result.title));
        }

        Ok(parse.to_parsed_media())
    }

    fn name(&self) -> &str {
        "PatternNormalizer"
    }
}

impl Default for PatternNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TorrentResult;

    #[test]
    fn test_movie_normalization() {
        let normalizer = PatternNormalizer::new();
        let torrent = TorrentResult::new(
            "Inception.2010.1080p.BluRay.x264-SPARKS".to_string(),
            "magnet:?xt=urn:btih:test".to_string(),
            1_500_000_000,
            100,
            50,
            "TestIndexer".to_string(),
        );

        let result = normalizer.normalize(&torrent).unwrap();
        
        assert_eq!(result.title, "Inception");
        assert_eq!(result.year, Some(2010));
        assert_eq!(result.resolution, Some("1080P".to_string()));
        assert_eq!(result.source, Some("BluRay".to_string()));
        assert_eq!(result.codec, Some("H.264".to_string()));
        assert_eq!(result.release_group, Some("SPARKS".to_string()));
        assert_eq!(result.media_type, MediaType::Movie);
    }

    #[test]
    fn test_series_normalization() {
        let normalizer = PatternNormalizer::new();
        let torrent = TorrentResult::new(
            "Breaking.Bad.S01E01.2008.720p.BluRay.x264-REWARDERS".to_string(),
            "magnet:?xt=urn:btih:test".to_string(),
            1_200_000_000,
            150,
            75,
            "TestIndexer".to_string(),
        );

        let result = normalizer.normalize(&torrent).unwrap();
        
        assert_eq!(result.title, "Breaking Bad");
        assert_eq!(result.year, Some(2008));
        assert_eq!(result.season, Some(1));
        assert_eq!(result.episode, Some(1));
        assert_eq!(result.resolution, Some("720P".to_string()));
        assert_eq!(result.source, Some("BluRay".to_string()));
        assert_eq!(result.media_type, MediaType::Series);
    }

    #[test]
    fn test_4k_movie_normalization() {
        let normalizer = PatternNormalizer::new();
        let torrent = TorrentResult::new(
            "Dune.2021.2160p.UHD.BluRay.x265-FRAMESTOR".to_string(),
            "magnet:?xt=urn:btih:test".to_string(),
            25_000_000_000,
            200,
            100,
            "TestIndexer".to_string(),
        );

        let result = normalizer.normalize(&torrent).unwrap();
        
        assert_eq!(result.title, "Dune");
        assert_eq!(result.year, Some(2021));
        assert_eq!(result.resolution, Some("2160P".to_string()));
        assert_eq!(result.source, Some("UHD.BluRay".to_string()));
        assert_eq!(result.codec, Some("H.265".to_string()));
        assert_eq!(result.release_group, Some("FRAMESTOR".to_string()));
    }

    #[test]
    fn test_web_dl_normalization() {
        let normalizer = PatternNormalizer::new();
        let torrent = TorrentResult::new(
            "[YTS.MX] The.Matrix.1999.1080p.WEB-DL.x264".to_string(),
            "magnet:?xt=urn:btih:test".to_string(),
            1_800_000_000,
            300,
            150,
            "YTS".to_string(),
        );

        let result = normalizer.normalize(&torrent).unwrap();
        
        assert_eq!(result.title, "The Matrix");
        assert_eq!(result.year, Some(1999));
        assert_eq!(result.resolution, Some("1080P".to_string()));
        assert_eq!(result.source, Some("WEB-DL".to_string()));
        assert_eq!(result.codec, Some("H.264".to_string()));
    }
}
