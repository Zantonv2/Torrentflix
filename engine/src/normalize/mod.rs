use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, warn};

use crate::models::{TorrentResult, ParsedMedia, MediaType};

pub mod patterns;
pub mod guessit;

pub use patterns::PatternNormalizer;
pub use guessit::GuessitNormalizer;

/// Trait for media filename normalization
pub trait MediaNormalizer: Send + Sync {
    fn normalize(&self, torrent_result: &TorrentResult) -> Result<ParsedMedia>;
    fn name(&self) -> &str;
}

/// Main normalizer that combines multiple strategies
pub struct Normalizer {
    strategies: Vec<Box<dyn MediaNormalizer>>,
}

impl Normalizer {
    pub fn new() -> Self {
        Self {
            strategies: Vec::new(),
        }
    }

    pub fn add_strategy(mut self, strategy: Box<dyn MediaNormalizer>) -> Self {
        self.strategies.push(strategy);
        self
    }

    pub fn normalize(&self, torrent_result: &TorrentResult) -> Result<ParsedMedia> {
        for strategy in &self.strategies {
            match strategy.normalize(torrent_result) {
                Ok(parsed) => {
                    debug!("Successfully normalized '{}' using {}", torrent_result.title, strategy.name());
                    return Ok(parsed);
                }
                Err(e) => {
                    debug!("Failed to normalize '{}' with {}: {}", torrent_result.title, strategy.name(), e);
                    continue;
                }
            }
        }

        // Fallback to basic parsing
        warn!("All normalization strategies failed for '{}', using basic parsing", torrent_result.title);
        self.basic_fallback(torrent_result)
    }

    fn basic_fallback(&self, torrent_result: &TorrentResult) -> Result<ParsedMedia> {
        let title = torrent_result.title.clone();
        let media_type = self.guess_media_type(&title);
        
        Ok(ParsedMedia::new(title, media_type))
    }

    fn guess_media_type(&self, title: &str) -> MediaType {
        let title_lower = title.to_lowercase();
        
        if title_lower.contains("s0") && (title_lower.contains("e0") || title_lower.contains("ex")) {
            MediaType::Series
        } else if title_lower.contains("season") || title_lower.contains("episode") {
            MediaType::Series
        } else if title_lower.contains("documentary") || title_lower.contains("doc") {
            MediaType::Documentary
        } else if title_lower.contains("anime") || title_lower.contains("manga") {
            MediaType::Anime
        } else {
            MediaType::Movie
        }
    }
}

impl Default for Normalizer {
    fn default() -> Self {
        let mut normalizer = Self::new();
        
        // Add pattern-based normalizer as primary strategy
        normalizer = normalizer.add_strategy(Box::new(PatternNormalizer::new()));
        
        // TODO: Add guessit normalizer when available
        // #[cfg(feature = "guessit")]
        // {
        //     normalizer = normalizer.add_strategy(Box::new(GuessitNormalizer::new()));
        // }
        
        normalizer
    }
}

/// Configuration for normalization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizerConfig {
    pub enable_patterns: bool,
    pub fallback_to_basic: bool,
    pub custom_patterns: HashMap<String, String>,
}

impl Default for NormalizerConfig {
    fn default() -> Self {
        Self {
            enable_patterns: true,
            fallback_to_basic: true,
            custom_patterns: HashMap::new(),
        }
    }
}

/// Utility functions for common normalization tasks
pub mod utils {
    pub fn extract_year(title: &str) -> Option<u32> {
        let re = regex::Regex::new(r"\b(19|20)\d{2}\b").ok()?;
        
        if let Some(caps) = re.captures(title) {
            caps.get(0)?.as_str().parse().ok()
        } else {
            None
        }
    }

    pub fn extract_resolution(title: &str) -> Option<String> {
        let resolutions = vec![
            "4320p", "8k", "2160p", "4k", "1440p", "2k", 
            "1080p", "720p", "480p", "360p", "240p"
        ];

        for res in resolutions {
            if title.to_lowercase().contains(&res.to_lowercase()) {
                return Some(res.to_uppercase());
            }
        }

        None
    }

    pub fn extract_source(title: &str) -> Option<String> {
        let sources = vec![
            "uhd bluray", "uhd", "bluray", "bd", "bdrip", "brrip",
            "web-dl", "webdl", "webrip", "web", "hdtv", "pdtv", 
            "dsr", "dvd", "dvdrip", "cam", "ts", "tc"
        ];

        let title_lower = title.to_lowercase();
        
        for source in sources {
            if title_lower.contains(source) {
                return Some(match source {
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
                });
            }
        }

        None
    }

    pub fn extract_codec(title: &str) -> Option<String> {
        let codecs = vec![
            "h.265", "hevc", "x265", "h264", "x264", "avc", 
            "mpeg-2", "mpeg2", "xvid", "divx", "vp9", "av1"
        ];

        let title_lower = title.to_lowercase();
        
        for codec in codecs {
            if title_lower.contains(codec) {
                return Some(match codec {
                    "h.265" | "hevc" => "H.265".to_string(),
                    "h264" | "avc" => "H.264".to_string(),
                    "mpeg-2" | "mpeg2" => "MPEG-2".to_string(),
                    _ => codec.to_uppercase(),
                });
            }
        }

        None
    }

    pub fn extract_audio(title: &str) -> Option<String> {
        let audio_formats = vec![
            "dts-hd.ma", "dts-hd", "dts", "truehd", "atmos", 
            "dd+", "dolby.digital.plus", "ac3", "aac", "mp3", "flac"
        ];

        let title_lower = title.to_lowercase();
        
        for audio in audio_formats {
            if title_lower.contains(audio) {
                return Some(match audio {
                    "dts-hd.ma" => "DTS-HD.MA".to_string(),
                    "dts-hd" => "DTS-HD".to_string(),
                    "dd+" | "dolby.digital.plus" => "DD+".to_string(),
                    _ => audio.to_uppercase(),
                });
            }
        }

        None
    }

    pub fn extract_release_group(title: &str) -> Option<String> {
        // Look for patterns like [GroupName], -GroupName, or GroupName at the end
        let patterns = vec![
            r"\[([^\]]+)\]$",      // [GroupName]
            r"\-([a-zA-Z0-9\-]+)$", // -GroupName
            r"\.([a-zA-Z0-9\-]+)$", // .GroupName
            r"([a-zA-Z0-9\-]+)$",   // GroupName at end
        ];

        for pattern in patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if let Some(caps) = re.captures(title) {
                    if let Some(group) = caps.get(1) {
                        let group_name = group.as_str();
                        // Filter out common non-group patterns
                        if !is_common_suffix(group_name) {
                            return Some(group_name.to_string());
                        }
                    }
                }
            }
        }

        None
    }

    fn is_common_suffix(text: &str) -> bool {
        let common_suffixes = vec![
            "1080p", "720p", "480p", "2160p", "4k", "bluray", "web", 
            "x264", "x265", "h264", "h265", "aac", "ac3", "dts",
            "proper", "repack", "extended", "unrated", "theatrical",
            "internal", "french", "german", "spanish", "multisubs",
            "subbed", "subforced", "nfo", "rarbg", "yts", "yify"
        ];

        common_suffixes.contains(&text.to_lowercase().as_str())
    }

    pub fn extract_season_episode(title: &str) -> Option<(u32, u32)> {
        // Patterns like S01E02, S1E2, 1x02, Season 1 Episode 2, etc.
        let patterns = vec![
            r"S(\d{1,2})E(\d{1,2})",           // S01E02
            r"s(\d{1,2})e(\d{1,2})",           // s01e02
            r"(\d{1,2})x(\d{1,2})",            // 1x02
            r"Season\s*(\d{1,2}).*Episode\s*(\d{1,2})", // Season 1 Episode 2
        ];

        let title_lower = title.to_lowercase();
        
        for pattern in patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if let Some(caps) = re.captures(&title_lower) {
                    if let (Some(season), Some(episode)) = (caps.get(1), caps.get(2)) {
                        if let (Ok(s), Ok(e)) = (
                            season.as_str().parse::<u32>(),
                            episode.as_str().parse::<u32>()
                        ) {
                            return Some((s, e));
                        }
                    }
                }
            }
        }

        None
    }

    pub fn clean_title(mut title: String) -> String {
        // Remove common patterns and clean up
        let patterns_to_remove = vec![
            r"\[.*?\]",                // [anything]
            r"\(.*?\)",                // (anything)
            r"\..*?(?:avi|mkv|mp4|mov|wmv|flv|webm)$", // .extension
            r"(?i)\b(1080p|720p|480p|2160p|4k|bluray|web|hdtv|dvd|cam|ts|x264|x265|h264|h265|aac|ac3|dts|proper|repack|extended|unrated|theatrical|internal)\b",
            r"(?i)\b(dts-hd\.ma|dts-hd|truehd|atmos|dd\+)\b",
        ];

        for pattern in patterns_to_remove {
            if let Ok(re) = regex::Regex::new(pattern) {
                title = re.replace_all(&title, "").to_string();
            }
        }

        // Clean up extra spaces, dots, and hyphens
        title = title.replace('.', " ");
        title = title.replace('_', " ");
        title = regex::Regex::new(r"\s+").unwrap().replace_all(&title, " ").to_string();
        title.trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::utils::*;

    #[test]
    fn test_extract_year() {
        assert_eq!(extract_year("Movie 2023 1080p"), Some(2023));
        assert_eq!(extract_year("Film (2022)"), Some(2022));
        assert_eq!(extract_year("No Year Here"), None);
    }

    #[test]
    fn test_extract_resolution() {
        assert_eq!(extract_resolution("Movie 1080p"), Some("1080P".to_string()));
        assert_eq!(extract_resolution("Film 4K"), Some("4K".to_string()));
        assert_eq!(extract_resolution("Show 720p"), Some("720P".to_string()));
        assert_eq!(extract_resolution("No Res"), None);
    }

    #[test]
    fn test_extract_season_episode() {
        assert_eq!(extract_season_episode("Show S01E02"), Some((1, 2)));
        assert_eq!(extract_season_episode("Series s2e5"), Some((2, 5)));
        assert_eq!(extract_season_episode("Program 3x04"), Some((3, 4)));
        assert_eq!(extract_season_episode("Movie No Season"), None);
    }

    #[test]
    fn test_clean_title() {
        assert_eq!(clean_title("Movie.2023.1080p.BluRay.x264-Group".to_string()), "Movie 2023");
        assert_eq!(clean_title("[YTS] Film (2022) 720p WEB".to_string()), "Film 2022");
        assert_eq!(clean_title("Show_S01E01_HDTV_x264".to_string()), "Show S01E01");
    }
}
