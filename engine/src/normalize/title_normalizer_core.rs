use regex::Regex;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use once_cell::sync::Lazy;

// Static compiled regex patterns for performance
static RE_RELEASE_GROUPS: Lazy<Regex> = Lazy::new(|| Regex::new(r"[\[\(][^\]\)]+[\]\)]").unwrap());
static RE_MULTISPACE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());
static RE_SXX_EXX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)S\d{1,2}E\d{1,2}").unwrap());
static RE_SEASON_EP: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)Season\s+\d+|Episode\s+").unwrap());
static RE_YEAR: Lazy<Regex> = Lazy::new(|| Regex::new(r"(19|20)\d{2}").unwrap());
// Enhanced resolution patterns from Rust metadata parser
static RES_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(?:(?P<width>\d{3,4})(?:x|\*))?(?P<height>\d{3,4})(?P<scan>p|i)?(?:10bit)?")
        .unwrap()
});

// Enhanced source patterns with rip detection
static RE_SOURCE_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(HD-?CAM|HD-?TELESYNC|HD-?TELECINE|HD-?TV|TV-?HD|WEB-?DL|WEB-?RIP|BLU-?RAY|BDRIP|DVDRIP|HDTV|CAM|TELESYNC|TELECINE|DVD|VHS|WEB|DL|RIP|NETFLIX|DISNEY\+|HBO|AMAZON|PRIME|HULU|APPLE|TV\+|PARAMOUNT\+|PEACOCK)")
        .unwrap()
});

// Enhanced video codec patterns with profiles
static RE_VIDEO_CODEC: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)([HX]-?26[45]|HEVC|AVC|XVID|DIVX|VP[0-9]|MPEG-?2|VC-1)(?:10BIT)?")
        .unwrap()
});

// Enhanced audio codec patterns with channels
static RE_AUDIO_CODEC: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(AAC|AC3|EAC3|DDP|DTS|TRUEHD|ATMOS|FLAC)(?:[0-9]\.[0-9])?")
        .unwrap()
});

// Critical missing patterns from Rust metadata parser analysis
static RE_EPISODE_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(?:S(\d{1,2})E(\d{1,2})|(\d{1,2})x(\d{1,2})|Season\s+(\d{1,2})\s+Episode\s+(\d{1,2})|E(\d{1,2})|Ep(\d{1,2})|Episode\s+(\d{1,2})|Season|Episode)")
        .unwrap()
});

static RE_RELEASE_GROUP: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)-([A-Za-z0-9]+)$")
        .unwrap()
});

static RE_EDITION_TAGS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(PROPER|REPACK|EXTENDED|UNRATED|DIRECTORS\.?CUT|THEATRICAL|IMAX|3D|4K|HDR|DOLBY\.VISION|ATMOS|REMASTERED|CRITERION)")
        .unwrap()
});

static RE_STREAMING_SERVICE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(NETFLIX|DISNEY\+|HBO\s*MAX|AMAZON\s*PRIME|HULU|APPLE\s*TV\+|PARAMOUNT\+|PEACOCK)")
        .unwrap()
});

static RE_CONTAINER: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\.(mkv|mp4|avi|mov|wmv|flv|webm|m4v|mpg|mpeg|ts|m2ts|iso)$")
        .unwrap()
});

static RE_SIZE_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(\d+\.?\d*)\s*([TG])B")
        .unwrap()
});

static RE_CRC32: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)([a-f0-9]{8})")
        .unwrap()
});

// Russian episode patterns (critical for your use case)
static RE_RUSSIAN_EPISODES: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(Сезон|Серия|Сезон\s+(\d+)|(\d+)\s+Сезон|Серия\s+(\d+)|(\d+)\s+Серия)")
        .unwrap()
});

// Noise phrases for cleaning
const NOISE_PHRASES: &[&str] = &[
    r"\bIMAX\b",
    r"\bWEB[- ]?DL\b",
    r"\bWEB[- ]?Rip\b",
    r"\bBlu[- ]?Ray\b",
    r"\bHDRip\b",
    r"\bBRRip\b",
    r"\bBDRip\b",
    r"\bDVDRip\b",
    r"\bDL[- ]?RIP\b",
    r"\bREMASTER(?:ED)?\b",
    r"\bATMOS\b",
    r"\bTrueHD\b",
    r"\bDual\s+Audio\b",
    r"\bMulti\s+Audio\b",
    r"\bHEVC\b",
    r"\bx26[45]\b",
    r"\b10bit\b",
    r"\b8bit\b",
    r"\b4K\b",
    r"\bUHD\b",
    r"\b1080p\b",
    r"\b720p\b",
    r"\b480p\b",
    r"\b2160p\b",
    r"\bFHD\b",
    r"\bHD\b",
    r"\bHQ\b",
    r"\bPROPER\b",
    r"\bSUBS?\b",
    r"\bDUBBED\b",
    r"\bDUB\b",
    r"\bORIGINAL\b",
    r"\bLATINO\b",
    r"\bRUSSIAN\b",
    r"\bENG?\b",
    r"\bAAC(?:\d+(?:\s+\d+)*)?(?:\.\d+)?\b",
    r"\bAC3\b",
    r"\bDTS\b",
    r"\bMP3\b",
    r"\bFLAC\b",
    r"\bH\.?264\b",
    r"\bH\.?265\b",
    r"\bXVID\b",
    r"\bDIVX\b",
    r"\bAVC\b",
    r"\bMPEG[-]?[24]\b",
    r"\bIP\b",
    r"\bWEB\b",
    r"\bHDTV\b",
    r"\bPDTV\b",
    r"\bDSR\b",
    r"\bSAT\b",
    r"\bEZTV\b",
    r"\bBTW\b",
    r"\bDIMENSION\b",
    r"\bLOL\b",
    r"\bKILLERS\b",
    r"\bFQM\b",
    r"\b2HD\b",
    r"\bASAP\b",
    r"\bBATV\b",
    r"\b0TV\b",
    r"\bTLA\b",
    r"\bSYS\b",
    r"\bREPACK\b",
    r"\bINTERNAL\b",
    r"\bREADNFO\b",
    r"\bMONDAY\b",
    r"\bTUESDAY\b",
    r"\bWEDNESDAY\b",
    r"\bTHURSDAY\b",
    r"\bFRIDAY\b",
    r"\bSATURDAY\b",
    r"\bSUNDAY\b",
    r"\b\d{1,2}\s+\d{1,2}\b",
    r"\bH\s+264\b",
    r"\bH\s+265\b",
    r"\bX\s+VID\b",
    r"\bDIV\s+X\b",
    r"\bHDR10\b",
    r"\bDV\b",
    r"\bMA\b",
    r"\b\d{1,2}\s+S\d{1,2}\b",
    r"\bвыпуск\b",
    r"\bТВ-шоу\b",
    r"\bреалити-шоу\b",
    r"\bреалити-эксперимент\b",
    r"\bHDTV\s+1080P\b",
    r"\b-8\s+Of\s+X\b",
    r"\bMKV\b",
    r"\bDDP(?:\d+(?:\s+\d+)*)?(?:\.\d+)?\b",
    r"-Higgsboson\b",
    r"\bXXX\b",
    // Russian TV show prefixes
    r"\bСериал\b",
    r"\bМультсериал\b",
    r"\bАниме\b",
    r"\bДорама\b",
    r"\bТВ-шоу\b",
    r"\bMP4\b",
    r"\bXC\b",
    r"\b01V2\b",
];

// Category sets for fast lookup
static RESOLUTIONS: Lazy<std::collections::HashSet<String>> = Lazy::new(|| {
    vec!["2160P", "1080P", "720P", "540P", "480P", "4K", "8K"]
    .iter().map(|s| s.to_string()).collect()
});

static SOURCES: Lazy<std::collections::HashSet<String>> = Lazy::new(|| {
    vec!["WEB-DL", "WEBDL", "WEB-RIP", "WEBRIP", "HDRIP", "HDTV", "HDTS", "HDTC", "CAM", "CAMRIP", 
         "TS", "TELESYNC", "DVDRIP", "DVD", "BDRIP", "BLURAY", "BRRIP", "REMUX", "UHD-BD", "UHD-4K"]
    .iter().map(|s| s.to_string()).collect()
});

static CODECS: Lazy<std::collections::HashSet<String>> = Lazy::new(|| {
    vec!["X264", "X265", "H.264", "H.265", "AVC", "HEVC", "XVID", "DIVX", "VP9", "HI10P"]
    .iter().map(|s| s.to_string()).collect()
});

static AUDIO: Lazy<std::collections::HashSet<String>> = Lazy::new(|| {
    vec!["AC3", "EAC3", "DDP", "DD5.1", "DTS", "DTS-HD", "TRUEHD", "ATMOS", "AAC", "FLAC"]
    .iter().map(|s| s.to_string()).collect()
});

static VERSIONS: Lazy<std::collections::HashSet<String>> = Lazy::new(|| {
    vec!["EXTENDED", "UNRATED", "DIRECTORS_CUT", "REMASTERED", "REMASTER", "LIMITED", "PROPER", 
         "REPACK", "REAL", "INTERNAL", "HDR10PLUS", "HDR10", "DV", "10BIT"]
    .iter().map(|s| s.to_string()).collect()
});

static LANGS: Lazy<std::collections::HashSet<String>> = Lazy::new(|| {
    vec!["RUS", "ENG", "FRENCH", "GERMAN", "MULTI", "SUBBED", "DUBBED"]
    .iter().map(|s| s.to_string()).collect()
});

static GROUPS: Lazy<std::collections::HashSet<String>> = Lazy::new(|| {
    vec!["YTS", "RARBG", "FGT", "EVO", "CM8", "QOQ", "CTRLHD", "TIGOLE"]
    .iter().map(|s| s.to_string()).collect()
});

const CATEGORY_ORDER: &[&str] = &["resolution", "source", "codec", "audio", "version", "language", "group"];

// Multi-word replacements
fn apply_multi_word_replacements(text: &str) -> String {
    let mut result = text.to_string();
    result = Regex::new(r"(?i)DIRECTORS\s+CUT").unwrap().replace_all(&result, "DIRECTORS_CUT").to_string();
    result = Regex::new(r"(?i)DOLBY\s+ATMOS").unwrap().replace_all(&result, "ATMOS").to_string();
    result = Regex::new(r"(?i)TRUE\s+HD").unwrap().replace_all(&result, "TRUEHD").to_string();
    result = Regex::new(r"(?i)DOLBY\s+VISION").unwrap().replace_all(&result, "DV").to_string();
    result
}

// NormalizedTitle struct is now defined in lib.rs with PyO3 bindings

#[derive(Clone, Debug)]
pub struct NormalizedTitle {
    pub original: String,
    pub cleaned: String,
    pub year: Option<i32>,
}

impl NormalizedTitle {
    pub fn new(original: String, cleaned: String, year: Option<i32>) -> Self {
        Self { original, cleaned, year }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParsedTorrentMetadata {
    pub original: String,
    pub title: String,
    pub title_local: String,
    pub year: Option<i32>,
    pub tags: HashMap<String, Vec<String>>,
    pub title_for_api: String,
    pub title_for_query: String,
    pub normalized_name: String,
}

impl ParsedTorrentMetadata {
    pub fn new(
        original: String,
        title: String,
        year: Option<i32>,
        tags: HashMap<String, Vec<String>>,
        normalized_name: String,
        title_local: String,
        title_for_api: String,
        title_for_query: String,
    ) -> Self {
        Self {
            original,
            title,
            year,
            tags,
            normalized_name,
            title_local,
            title_for_api,
            title_for_query,
        }
    }
}

fn get_current_year() -> i32 {
    use std::process::Command;
    let output = Command::new("date")
        .arg("+%Y")
        .output()
        .expect("Failed to execute date command");
    let year_str = String::from_utf8_lossy(&output.stdout);
    year_str.trim().parse().unwrap_or(2024)
}

pub fn parse_torrent_metadata(name: String) -> ParsedTorrentMetadata {
    // Handle empty input
    if name.trim().is_empty() {
        return ParsedTorrentMetadata::new(
            "".to_string(),
            "".to_string(),
            None,
            HashMap::new(), // Completely empty tags for empty input
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
        );
    }

    let mut working = apply_multi_word_replacements(&name);
    // Split on dots, underscores, and spaces, but preserve hyphens for patterns like WEB-DL, x264-GROUP
    let tokens: Vec<&str> = working.split(&['.', '_', ' '][..]).collect();
    
    let mut tags: HashMap<String, Vec<String>> = CATEGORY_ORDER.iter()
        .map(|cat| (cat.to_string(), vec![]))
        .collect();
    tags.insert("other".to_string(), vec![]);
    
    let mut title_tokens = Vec::new();
    let mut year: Option<i32> = None;
    let mut tag_section_started = false;
    let mut previous_number_token: Option<String> = None;
    let mut previous_edition_token: Option<String> = None;
    let current_year = get_current_year();

    for (i, token) in tokens.iter().enumerate() {
        let tok = token.trim_matches(&['(', ')', '[', ']', ',', ':'][..]);
        if tok.is_empty() {
            continue;
        }
        
        let tok_display = tok.replace('_', " ");
        let tok_upper = tok_display.to_uppercase();
        

        // Check for year
        if let Ok(candidate) = tok.parse::<i32>() {
            if candidate >= 1900 && candidate <= current_year + 1 && year.is_none() {
                year = Some(candidate);
                continue;
            }
        }

        // Check for CRC32 hashes FIRST (before edition detection to prevent consumption)
        if let Some(crc_match) = RE_CRC32.captures(&tok_upper) {
            let crc_value = crc_match.get(1).unwrap().as_str();
            let crc_formatted = format!("CRC:{}", crc_value);
            tags.get_mut("other").unwrap().push(crc_formatted);
            tag_section_started = true;
            continue;
        }

        // Check for text-based resolutions (moved BEFORE episode detection to prevent misclassification)
        if tok_upper == "4K" || tok_upper == "8K" || tok_upper == "UHD" {
            // For "4K", check if it should be treated as an edition (when followed by another resolution)
            if tok_upper == "4K" {
                if let Some(next_token) = tokens.get(i + 1) {
                    let next_upper = next_token.trim_matches(&['(', ')', '[', ']', ',', ':'][..]).to_uppercase();
                    // Simple check for common resolution patterns
                    if next_upper.contains("1080") || next_upper.contains("720") || next_upper.contains("480") || 
                       next_upper.contains("2160") || next_upper.contains("1440") ||
                       next_upper.ends_with("P") {
                        // "4K" followed by another resolution - let edition detection handle it
                        // Don't process here, continue to edition detection
                    } else {
                        // Standalone "4K" - process as resolution
                        tags.get_mut("resolution").unwrap().push(tok_upper.to_string());
                        tag_section_started = true;
                        continue;
                    }
                } else {
                    // "4K" with no next token - process as resolution
                    tags.get_mut("resolution").unwrap().push(tok_upper.to_string());
                    tag_section_started = true;
                    continue;
                }
            } else {
                // "8K" and "UHD" always process as resolution
                tags.get_mut("resolution").unwrap().push(tok_upper.to_string());
                tag_section_started = true;
                continue;
            }
        }
        
        if let Some(res_match) = RES_PATTERN.captures(&tok_upper) {
            let mut resolution = String::new();
            
            // Try to build standard resolution from height
            if let Some(height) = res_match.name("height") {
                let height_str = height.as_str();
                if let Ok(height_num) = height_str.parse::<i32>() {
                    // Filter out false positives like "264" from codecs (minimum 480p)
                    if height_num >= 480 {
                        match height_str {
                            "2160" => resolution = "2160".to_string(),
                            "1080" => resolution = "1080".to_string(),
                            "720" => resolution = "720".to_string(),
                            "480" => resolution = "480".to_string(),
                            _ => resolution = height_str.to_string(),
                        }
                    }
                }
            } else if let Some(width) = res_match.name("width") {
                // Fallback to width if no height
                if let Ok(width_num) = width.as_str().parse::<i32>() {
                    if width_num >= 480 {
                        resolution = width.as_str().to_string();
                    }
                }
            }
            
            // Only add scan type and bit depth if we have a valid resolution
            if !resolution.is_empty() {
                if let Some(scan) = res_match.name("scan") {
                    resolution.push_str(scan.as_str());
                } else {
                    // Default to 'p' if no scan type specified
                    resolution.push('p');
                }
                
                if tok_upper.contains("10BIT") {
                    resolution.push_str(" 10BIT");
                }
                
                tags.get_mut("resolution").unwrap().push(resolution.to_lowercase());
                tag_section_started = true;
                continue;
            }
        }

        // Check for episode/season patterns (critical for TV shows) - moved BEFORE edition detection to prevent consumption
        if RE_EPISODE_PATTERN.is_match(&tok_upper) {
            let mut season: Option<i32> = None;
            let mut episode: Option<i32> = None;
            
            if let Some(ep_match) = RE_EPISODE_PATTERN.captures(&tok_upper) {
                // Parse different episode formats
                if let Some(s) = ep_match.get(1) { season = s.as_str().parse().ok(); }
                if let Some(e) = ep_match.get(2) { episode = e.as_str().parse().ok(); }
                if let Some(s) = ep_match.get(3) { season = s.as_str().parse().ok(); }
                if let Some(e) = ep_match.get(4) { episode = e.as_str().parse().ok(); }
                if let Some(s) = ep_match.get(5) { season = s.as_str().parse().ok(); }
                if let Some(e) = ep_match.get(6) { episode = e.as_str().parse().ok(); }
                if let Some(e) = ep_match.get(7) { episode = e.as_str().parse().ok(); }
                if let Some(e) = ep_match.get(8) { episode = e.as_str().parse().ok(); }
                if let Some(e) = ep_match.get(9) { episode = e.as_str().parse().ok(); }
            }
            
            // Always tag as episode when regex matches (standalone words or with numbers)
            tags.get_mut("other").unwrap().push("episode".to_string());
            tag_section_started = true;
            continue;
        }

        // Check for Russian season/episode patterns - moved BEFORE edition detection to prevent consumption
        if RE_RUSSIAN_EPISODES.is_match(&tok_upper) {
            let mut season: Option<i32> = None;
            let mut episode: Option<i32> = None;
            
            if let Some(captures) = RE_RUSSIAN_EPISODES.captures(&tok_upper) {
                if let Some(s) = captures.get(2) { season = s.as_str().parse().ok(); }
                if let Some(s) = captures.get(3) { season = s.as_str().parse().ok(); }
                if let Some(e) = captures.get(5) { episode = e.as_str().parse().ok(); }
                if let Some(e) = captures.get(6) { episode = e.as_str().parse().ok(); }
            }
            
            // Always tag as russian_episode when regex matches (standalone words or with numbers)
            tags.get_mut("other").unwrap().push("russian_episode".to_string());
            tag_section_started = true;
            continue;
        }

        // Check for edition tags first (before resolution to catch patterns like "4K" as editions)
        let mut edition_processed = false;
        
        // Check if current token can form a multi-word edition with previous token
        if let Some(ref prev_edition) = previous_edition_token {
            if prev_edition == "DOLBY" && tok_upper == "VISION" {
                tags.get_mut("version").unwrap().push("Dolby Vision".to_string());
                tag_section_started = true;
                edition_processed = true;
                previous_edition_token = None;
            } else if prev_edition == "DIRECTORS" && tok_upper == "CUT" {
                tags.get_mut("version").unwrap().push("Directors Cut".to_string());
                tag_section_started = true;
                edition_processed = true;
                previous_edition_token = None;
            }
        }
        
        // Check for single-token edition patterns
        if !edition_processed {
            // Handle "4K" - edition if followed by another resolution, resolution if standalone
            if tok_upper == "4K" {
                // Look ahead to see if next token is a resolution
                if let Some(next_token) = tokens.get(i + 1) {
                    let next_upper = next_token.trim_matches(&['(', ')', '[', ']', ',', ':'][..]).to_uppercase();
                    if RESOLUTIONS.contains(next_upper.as_str()) {
                        // "4K" followed by actual resolution = edition
                        tags.get_mut("version").unwrap().push("4K".to_string());
                        tag_section_started = true;
                        edition_processed = true;
                        continue;
                    }
                }
                // Standalone "4K" = resolution
                tags.get_mut("resolution").unwrap().push("4K".to_string());
                tag_section_started = true;
                continue;
            }
            
            // Handle "ATMOS" - edition if followed by resolution, audio if standalone or followed by audio codec
            if tok_upper == "ATMOS" {
                // Look ahead to see if next token suggests edition context
                if let Some(next_token) = tokens.get(i + 1) {
                    let next_upper = next_token.trim_matches(&['(', ')', '[', ']', ',', ':'][..]).to_uppercase();
                    // "ATMOS" followed by resolution = edition
                    if RESOLUTIONS.contains(next_upper.as_str()) {
                        tags.get_mut("version").unwrap().push("Atmos".to_string());
                        tag_section_started = true;
                        edition_processed = true;
                        continue;
                    }
                    // "ATMOS" followed by audio codec = edition
                    if next_upper.contains("AAC") || next_upper.contains("DTS") || next_upper.contains("AC3") || next_upper.contains("TRUEHD") {
                        tags.get_mut("version").unwrap().push("Atmos".to_string());
                        tag_section_started = true;
                        edition_processed = true;
                        continue;
                    }
                }
                // Standalone "ATMOS" = audio
                tags.get_mut("audio").unwrap().push("Atmos".to_string());
                tag_section_started = true;
                continue;
            }
            
            let edition = match tok_upper.as_str() {
                s if s.contains("PROPER") => "Proper",
                s if s.contains("REPACK") => "Repack",
                s if s.contains("EXTENDED") => "Extended",
                s if s.contains("UNRATED") => "Unrated",
                s if s.contains("THEATRICAL") => "Theatrical",
                s if s.contains("DIRECTORS") => "Directors Cut", // Will be combined with next token
                s if s.contains("REMASTERED") => "Remastered",
                s if s.contains("CRITERION") => "Criterion Collection",
                s if s.contains("IMAX") => "IMAX",
                s if s.contains("3D") => "3D",
                s if s.contains("HDR") => "HDR",
                s if s.contains("COLLECTORS") => "Collectors",
                s if s.contains("SPECIAL") => "Special",
                s if s.contains("ULTIMATE") => "Ultimate",
                s if s.contains("DELUXE") => "Deluxe",
                _ => "",
            };
            
            if !edition.is_empty() {
                if edition == "Directors Cut" {
                    // Store "DIRECTORS" token for multi-token combination
                    previous_edition_token = Some("DIRECTORS".to_string());
                    continue;
                } else {
                    tags.get_mut("version").unwrap().push(edition.to_string());
                    tag_section_started = true;
                    edition_processed = true;
                }
            }
        }
        
        // Check if current token is "DOLBY" for potential multi-token combination
        if !edition_processed && tok_upper == "DOLBY" {
            previous_edition_token = Some("DOLBY".to_string());
            continue;
        }
        
        // Clear previous edition token only if current token is not a valid second part of multi-word edition
        if !edition_processed {
            // Only clear if current token is not "VISION" or "CUT" (valid multi-word completion tokens)
            if tok_upper != "VISION" && tok_upper != "CUT" {
                previous_edition_token = None;
            }
        }
        
        if edition_processed {
            continue;
        }

        // Check for source with enhanced patterns
        if RE_SOURCE_PATTERN.is_match(&tok_upper) {
            let source = match tok_upper.as_str() {
                // Streaming services (check first as they're more specific)
                s if s.contains("NETFLIX") => "Netflix",
                s if s.contains("DISNEY+") => "Disney+",
                s if s.contains("HBO") => "HBO Max", // Simplified - will match both HBO and HBO.MAX tokens
                s if s.contains("AMAZON") || s.contains("PRIME") => "Amazon Prime",
                s if s.contains("HULU") => "Hulu",
                s if s.contains("APPLE") || s.contains("TV+") => "Apple TV+",
                s if s.contains("PARAMOUNT+") => "Paramount+",
                s if s.contains("PEACOCK") => "Peacock",
                // Traditional sources
                s if s.contains("HD-CAM") || s.contains("HDCAM") => "HD Camera",
                s if s.contains("HD-TELESYNC") || s.contains("HDTELESYNC") => "HD Telesync",
                s if s.contains("HD-TELECINE") || s.contains("HDTELECINE") => "HD Telecine",
                s if s.contains("HD-TV") || s.contains("TV-HD") || s.contains("HDTV") => "HDTV",
                s if s.contains("WEB-DL") => "WEB",
                s if s.contains("WEB-RIP") => "WEBRIP",
                s if s.contains("BLU-RAY") => "BluRay",
                s if s.contains("BDRIP") => "BDRIP",
                s if s.contains("DVDRIP") => "DVDRIP",
                s if s.contains("CAM") => "Camera",
                s if s.contains("TELESYNC") => "Telesync",
                s if s.contains("TELECINE") => "Telecine",
                s if s.contains("DVD") => "DVD",
                s if s.contains("VHS") => "VHS",
                s if s.contains("TV") => "TV",
                _ => tok_display.as_str(),
            };
            tags.get_mut("source").unwrap().push(source.to_string());
            tag_section_started = true;
            continue;
        }

        // Check for video codec with enhanced patterns
        if RE_VIDEO_CODEC.is_match(&tok_upper) {
            let codec = match tok_upper.as_str() {
                s if s.contains("H-264") || s.contains("H264") || s.contains("X264") || s.contains("AVC") => "H.264",
                s if s.contains("H-265") || s.contains("H265") || s.contains("X265") || s.contains("HEVC") => "H.265",
                s if s.contains("MPEG-2") => "MPEG-2",
                s if s.contains("VC-1") => "VC-1",
                s if s.contains("XVID") => "XviD",
                s if s.contains("DIVX") => "DivX",
                s if s.starts_with("VP") => tok_upper.split('-').next().unwrap_or(tok_upper.as_str()),
                _ => tok_display.as_str(),
            };
            tags.get_mut("codec").unwrap().push(codec.to_string());
            
            // Extract group from codec tokens like "X264-GROUP"
            if let Some(group_match) = RE_RELEASE_GROUP.captures(&tok_upper) {
                if let Some(group) = group_match.get(1) {
                    tags.get_mut("group").unwrap().push(group.as_str().to_uppercase());
                }
            }
            
            tag_section_started = true;
            continue;
        }

        // Check for audio codec with enhanced patterns
        if RE_AUDIO_CODEC.is_match(&tok_upper) {
            let audio = match tok_upper.as_str() {
                s if s.contains("E-AC3") || s.contains("EAC3") => "EAC3",
                s if s.contains("DOLBY") && s.contains("DIGITAL") => "DDP",
                s if s.contains("DDP") => "DDP",
                s if s.contains("AC-3") || s.contains("AC3") => "AC3",
                s if s.contains("DTS") => "DTS",
                s if s.contains("TRUEHD") || s.contains("TRUE-HD") => "TrueHD",
                s if s.contains("ATMOS") => "Atmos",
                s if s.contains("FLAC") => "FLAC",
                s if s.contains("AAC") => "AAC",
                _ => tok_display.as_str(),
            };
            tags.get_mut("audio").unwrap().push(audio.to_string());
            
            // Extract group from audio tokens like "AAC-GROUP"
            if let Some(group_match) = RE_RELEASE_GROUP.captures(&tok_upper) {
                if let Some(group) = group_match.get(1) {
                    tags.get_mut("group").unwrap().push(group.as_str().to_uppercase());
                }
            }
            
            tag_section_started = true;
            continue;
        }

        // Check for file containers
        // Handle both with and without dot since tokenization splits on dots
        if RE_CONTAINER.is_match(&tok_upper) || tok_upper == "MKV" || tok_upper == "MP4" || tok_upper == "AVI" || 
           tok_upper == "MOV" || tok_upper == "WMV" || tok_upper == "FLV" || tok_upper == "WEBM" {
            let container = match tok_upper.as_str() {
                s if s.contains("MKV") => "MKV",
                s if s.contains("MP4") => "MP4",
                s if s.contains("AVI") => "AVI",
                s if s.contains("MOV") => "MOV",
                s if s.contains("WMV") => "WMV",
                s if s.contains("FLV") => "FLV",
                s if s.contains("WEBM") => "WEBM",
                _ => tok_upper.as_str(),
            };
            tags.get_mut("other").unwrap().push(container.to_uppercase());
            tag_section_started = true;
            continue;
        }

        // Check for file sizes (handle both single tokens and decimal patterns like "1.5GB")
        let mut size_processed = false;
        
        // Check if current token can form a decimal size with previous number token
        if let Some(ref prev_num) = previous_number_token {
            if (tok_upper.ends_with("GB") || tok_upper.ends_with("TB")) && 
               RE_SIZE_PATTERN.is_match(&format!("{}.{}", prev_num, tok_upper)) {
                let combined_decimal = format!("{}.{}", prev_num, tok_upper);
                if let Some(size_match) = RE_SIZE_PATTERN.captures(&combined_decimal) {
                    let size_value = size_match.get(1).unwrap().as_str();
                    let size_unit = size_match.get(2).unwrap().as_str();
                    let size_formatted = format!("{}{}B", size_value, size_unit);
                    tags.get_mut("other").unwrap().push(size_formatted);
                    tag_section_started = true;
                    size_processed = true;
                    previous_number_token = None; // Clear the stored number
                }
            }
        }
        
        // Check for regular single-token size patterns
        if !size_processed {
            if let Some(size_match) = RE_SIZE_PATTERN.captures(&tok_upper) {
                let size_value = size_match.get(1).unwrap().as_str();
                let size_unit = size_match.get(2).unwrap().as_str();
                let size_formatted = format!("{}{}B", size_value, size_unit);
                tags.get_mut("other").unwrap().push(size_formatted);
                tag_section_started = true;
                size_processed = true;
                previous_number_token = None; // Clear any stored number
            }
        }
        
        // If no size found and current token is a pure number, store it for potential decimal combination
        if !size_processed && tok_upper.chars().all(|c| c.is_ascii_digit()) {
            previous_number_token = Some(tok_upper.clone());
            continue;
        }
        
        // Clear previous number if this token wasn't used for size detection
        if !size_processed && !tok_upper.chars().all(|c| c.is_ascii_digit()) {
            previous_number_token = None;
        }
        
        if size_processed {
            continue;
        }

        // Check for CRC32 hashes
        if let Some(crc_match) = RE_CRC32.captures(&tok_upper) {
            let crc_value = crc_match.get(1).unwrap().as_str();
            let crc_formatted = format!("CRC:{}", crc_value);
            tags.get_mut("other").unwrap().push(crc_formatted);
            tag_section_started = true;
            continue;
        }

        // Check for streaming services (Netflix, Disney+, etc.)
        if RE_STREAMING_SERVICE.is_match(&tok_upper) {
            let service = match tok_upper.as_str() {
                s if s.contains("NETFLIX") => "Netflix",
                s if s.contains("DISNEY") => "Disney+",
                s if s.contains("HBO") => "HBO Max",
                s if s.contains("AMAZON") => "Amazon Prime",
                s if s.contains("HULU") => "Hulu",
                s if s.contains("APPLE") => "Apple TV+",
                s if s.contains("PARAMOUNT") => "Paramount+",
                s if s.contains("PEACOCK") => "Peacock",
                _ => tok_display.as_str(),
            };
            tags.get_mut("source").unwrap().push(service.to_string());
            tag_section_started = true;
            continue;
        }

        // Check for file containers (mkv, mp4, avi, etc.)
        if RE_CONTAINER.is_match(&tok_upper) {
            let container = match tok_upper.to_uppercase().as_str() {
                ".MKV" => "MKV",
                ".MP4" => "MP4", 
                ".AVI" => "AVI",
                ".MOV" => "MOV",
                ".WMV" => "WMV",
                ".FLV" => "FLV",
                ".WEBM" => "WebM",
                ".M4V" => "M4V",
                ".MPG" | ".MPEG" => "MPEG",
                ".TS" | ".M2TS" => "TS",
                ".ISO" => "ISO",
                _ => tok_display.as_str(),
            };
            tags.get_mut("other").unwrap().push(container.to_string());
            tag_section_started = true;
            continue;
        }

        // Check for file sizes (1.5GB, 2GB, etc.)
        if let Some(size_match) = RE_SIZE_PATTERN.captures(&tok_upper) {
            if let Some(size_num) = size_match.get(1) {
                if let Some(size_unit) = size_match.get(2) {
                    let size = format!("{}{}B", size_num.as_str(), size_unit.as_str());
                    tags.get_mut("other").unwrap().push(size);
                    tag_section_started = true;
                    continue;
                }
            }
        }

        // Check for CRC32 hashes
        if RE_CRC32.is_match(&tok_upper) && tok_upper.len() == 8 {
            tags.get_mut("other").unwrap().push(format!("CRC:{}", tok_upper));
            tag_section_started = true;
            continue;
        }

        // Check for release groups (dash-separated at end)
        if let Some(group_match) = RE_RELEASE_GROUP.captures(&tok_upper) {
            if let Some(group) = group_match.get(1) {
                tags.get_mut("group").unwrap().push(group.as_str().to_uppercase());
                tag_section_started = true;
                continue;
            }
        }

        // Check categories
        let mut matched_category = None;
        
        if SOURCES.contains(tok_upper.as_str()) {
            matched_category = Some("source");
        } else if CODECS.contains(tok_upper.as_str()) {
            matched_category = Some("codec");
        } else if AUDIO.contains(tok_upper.as_str()) {
            matched_category = Some("audio");
        } else if VERSIONS.contains(tok_upper.as_str()) {
            matched_category = Some("version");
        } else if LANGS.contains(tok_upper.as_str()) {
            matched_category = Some("language");
        } else if GROUPS.contains(tok_upper.as_str()) {
            matched_category = Some("group");
        } else {
            // Try to detect group suffix like x265-DRONES
            let fragments: Vec<&str> = tok_upper.split(&['-', '+'][..]).collect();
            if fragments.len() > 1 && GROUPS.contains(fragments[fragments.len() - 1]) {
                matched_category = Some("group");
            }
        }

        if let Some(category) = matched_category {
            tags.get_mut(category).unwrap()
                .push(tok_display.clone());
            tag_section_started = true;
        } else {
            if !tag_section_started {
                title_tokens.push(tok_display.clone());
            } else {
                tags.get_mut("other").unwrap()
                    .push(tok_display);
            }
        }
    }

    let title_local = title_tokens.join(" ").trim().to_string();
    let title_local = if title_local.is_empty() {
        name.trim().to_string()
    } else {
        title_local
    };

    // Separate English and non-English tokens
    let english_tokens: Vec<String> = title_tokens.iter()
        .filter(|tok| tok.chars().all(|c| c.is_ascii()))
        .cloned()
        .collect();
    
    let non_english_tokens: Vec<String> = title_tokens.iter()
        .filter(|tok| tok.chars().any(|c| !c.is_ascii()))
        .cloned()
        .collect();

    let title_for_api = english_tokens.join(" ").trim().to_string();
    let non_english_title = non_english_tokens.join(" ").trim().to_string();

    // Determine title_for_query
    let title_for_query = if !non_english_title.is_empty() && !english_tokens.is_empty() {
        title_local.clone()
    } else {
        if !title_for_api.is_empty() {
            title_for_api.clone()
        } else if !non_english_title.is_empty() {
            non_english_title
        } else {
            title_local.clone()
        }
    };

    let title = title_for_query.clone();
    let title_with_year = if let Some(y) = year {
        format!("{} ({})", title_local, y)
    } else {
        title_local.clone()
    };

    // Build normalized name
    let mut meta_parts = Vec::new();
    for category in CATEGORY_ORDER {
        if let Some(category_tags) = tags.get(*category) {
            let mut unique_tags: Vec<String> = category_tags.iter()
                .cloned()
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();
            unique_tags.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
            meta_parts.extend(unique_tags);
        }
    }

    let normalized_name = if meta_parts.is_empty() {
        title_with_year.clone()
    } else {
        format!("{} [{}]", title_with_year, meta_parts.join(" "))
    };

    ParsedTorrentMetadata::new(
        name,
        title,
        year,
        tags,
        normalized_name,
        title_local,
        title_for_api,
        title_for_query,
    )
}

pub fn clean_title(title: String) -> NormalizedTitle {
    if title.is_empty() {
        return NormalizedTitle::new("".to_string(), "".to_string(), None);
    }

    let parsed = parse_torrent_metadata(title.clone());
    let mut working = title.clone();

    // Remove release groups, season/episode info
    working = RE_RELEASE_GROUPS.replace_all(&working, " ").to_string();
    working = RE_SXX_EXX.replace_all(&working, " ").to_string();
    working = RE_SEASON_EP.replace_all(&working, " ").to_string();

    let year = parsed.year.or_else(|| extract_year(&working));

    // Remove noise phrases
    for phrase in NOISE_PHRASES {
        let re = Regex::new(&format!("(?i){}", phrase)).unwrap();
        working = re.replace_all(&working, " ").to_string();
    }

    // Clean up non-alphanumeric characters
    working = Regex::new(r"[^A-Za-zА-Яа-яЁё0-9'&:,.-]").unwrap()
        .replace_all(&working, " ")
        .to_string();
    
    working = RE_MULTISPACE.replace_all(&working, " ").to_string();
    working = working.trim().to_string();
    
    // Clean up repeated punctuation
    working = Regex::new(r"([\-_.:,]){2,}").unwrap()
        .replace_all(&working, "$1")
        .to_string();

    let mut display_source = if !parsed.title.is_empty() {
        parsed.title.clone()
    } else if !parsed.title_local.is_empty() {
        parsed.title_local.clone()
    } else {
        working.clone()
    };

    // Append year to display if present
    if let Some(year) = year {
        display_source = format!("{} {}", display_source, year);
    }

    let display = RE_MULTISPACE.replace_all(&display_source, " ")
        .to_string()
        .trim()
        .chars()
        .enumerate()
        .map(|(i, c)| if i == 0 || display_source.chars().nth(i-1).unwrap_or(' ').is_whitespace() {
            c.to_uppercase().collect::<String>()
        } else {
            c.to_string()
        })
        .collect::<String>();

    NormalizedTitle::new(title, display, year)
}

pub fn extract_year(title: &str) -> Option<i32> {
    if let Some(caps) = RE_YEAR.captures(title) {
        caps.get(0).unwrap().as_str().parse().ok()
    } else {
        None
    }
}

pub fn normalize_for_metadata(title: String) -> (String, Option<i32>) {
    let result = clean_title(title.clone());
    let mut cleaned = result.cleaned;
    
    // Remove Russian torrent noise phrases
    cleaned = Regex::new(r"(?i)скачать\s+торрент").unwrap()
        .replace_all(&cleaned, "")
        .to_string();
    cleaned = Regex::new(r"(?i)торрент").unwrap()
        .replace_all(&cleaned, "")
        .to_string();
    cleaned = Regex::new(r"(?i)скачать").unwrap()
        .replace_all(&cleaned, "")
        .to_string();
    
    // Remove standalone year numbers (clean_title extracts year but leaves it in string)
    cleaned = Regex::new(r"\b\d{4}\b").unwrap()
        .replace_all(&cleaned, "")
        .to_string();

    // Remove Russian TV show markers
    cleaned = Regex::new(r"(?i)\bСериал\b").unwrap()
        .replace_all(&cleaned, "")
        .to_string();

    // Remove season/episode info (Russian)
    cleaned = Regex::new(r"(?i)\b\d+\s*Сезон\b").unwrap()
        .replace_all(&cleaned, "")
        .to_string();
    cleaned = Regex::new(r"(?i)\bСезон\s*\d+\b").unwrap()
        .replace_all(&cleaned, "")
        .to_string();
    cleaned = Regex::new(r"(?i)\b\d+\s*Серия\b").unwrap()
        .replace_all(&cleaned, "")
        .to_string();

    // Remove season/episode info (English)
    cleaned = Regex::new(r"(?i)\bSeason\s*\d+\b").unwrap()
        .replace_all(&cleaned, "")
        .to_string();
    cleaned = Regex::new(r"(?i)\bS\d+\b").unwrap()
        .replace_all(&cleaned, "")
        .to_string();

    // Clean up extra spaces
    cleaned = RE_MULTISPACE.replace_all(&cleaned, " ").to_string();
    cleaned = cleaned.trim().to_string();
    
    // Convert to lowercase for TMDB API compatibility (TMDB expects normal case)
    cleaned = cleaned.to_lowercase();

    (cleaned, result.year)
}

pub fn prettify_for_display(title: String) -> String {
    if title.is_empty() {
        title
    } else {
        let (cleaned, _) = normalize_for_metadata(title.clone());
        
        // Additional cleanup for Russian torrent noise and years
        let mut final_cleaned = cleaned;
        
        // Remove Russian torrent phrases
        final_cleaned = Regex::new(r"(?i)скачать\s+торрент").unwrap()
            .replace_all(&final_cleaned, "")
            .to_string();
        final_cleaned = Regex::new(r"(?i)торрент").unwrap()
            .replace_all(&final_cleaned, "")
            .to_string();
        final_cleaned = Regex::new(r"(?i)скачать").unwrap()
            .replace_all(&final_cleaned, "")
            .to_string();
        
        // Remove years from display title
        final_cleaned = Regex::new(r"\(\d{4}\)").unwrap()
            .replace_all(&final_cleaned, "")
            .to_string();
        final_cleaned = Regex::new(r"\b\d{4}\b").unwrap()
            .replace_all(&final_cleaned, "")
            .to_string();
        
        // Clean up extra spaces and trim
        final_cleaned = Regex::new(r"\s+").unwrap()
            .replace_all(&final_cleaned.trim(), " ")
            .to_string();
        
        // Capitalize each word for proper display
        final_cleaned = final_cleaned
            .split_whitespace()
            .map(|word| {
                if word.is_empty() {
                    word.to_string()
                } else {
                    let mut chars = word.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                }
            })
            .collect::<Vec<String>>()
            .join(" ");
        
        if final_cleaned.trim().is_empty() {
            title
        } else {
            final_cleaned
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_basic_resolution_patterns() {
        let result = parse_torrent_metadata("Movie.2023.1080p.WEB-DL.x264-GROUP.mkv".to_string());
        assert_eq!(result.title, "Movie");
        assert_eq!(result.year, Some(2023));
        assert!(result.tags["resolution"].contains(&"1080p".to_string()));
        assert!(result.tags["source"].contains(&"WEB".to_string()));
        assert!(result.tags["codec"].contains(&"H.264".to_string()));
        assert!(result.tags["group"].contains(&"GROUP".to_string()));
    }

    #[test]
    fn test_episode_patterns() {
        // S01E02 format
        let result = parse_torrent_metadata("Show.S01E02.1080p.WEB-DL.x264-GROUP.mkv".to_string());
        assert_eq!(result.title, "Show");
        assert!(result.tags["other"].contains(&"episode".to_string()));
    }

    #[test]
    fn test_episode_1x02_format() {
        // 1x02 format
        let result = parse_torrent_metadata("Show.1x02.Episode.Title.1080p.WEB-DL.x264-GROUP.mkv".to_string());
        assert_eq!(result.title, "Show");
        assert!(result.tags["other"].contains(&"episode".to_string()));
    }

    #[test]
    fn test_russian_episodes() {
        // Russian format: Сезон 1 Серия 2
        let result = parse_torrent_metadata("Сериал.Сезон 1.Серия 2.1080p.WEB-DL.x264-GROUP.mkv".to_string());
        assert_eq!(result.title, "Сериал");
        assert!(result.tags["other"].contains(&"russian_episode".to_string()));
    }

    #[test]
    fn test_edition_tags() {
        // PROPER tag
        let result = parse_torrent_metadata("Movie.2023.PROPER.1080p.BluRay.x264-GROUP.mkv".to_string());
        assert!(result.tags["version"].contains(&"Proper".to_string()));
        
        // EXTENDED tag
        let result = parse_torrent_metadata("Movie.2023.EXTENDED.1080p.BluRay.x264-GROUP.mkv".to_string());
        assert!(result.tags["version"].contains(&"Extended".to_string()));
        
        // DIRECTORS.CUT tag
        let result = parse_torrent_metadata("Movie.2023.DIRECTORS.CUT.1080p.BluRay.x264-GROUP.mkv".to_string());
        assert!(result.tags["version"].contains(&"Directors Cut".to_string()));
    }

    #[test]
    fn test_streaming_services() {
        // Netflix
        let result = parse_torrent_metadata("Show.S01E01.NETFLIX.1080p.WEB-DL.x264-GROUP.mkv".to_string());
        assert!(result.tags["source"].contains(&"Netflix".to_string()));
        
        // Disney+
        let result = parse_torrent_metadata("Show.S01E01.DISNEY+.1080p.WEB-DL.x264-GROUP.mkv".to_string());
        assert!(result.tags["source"].contains(&"Disney+".to_string()));
        
        // HBO Max
        let result = parse_torrent_metadata("Show.S01E01.HBO.MAX.1080p.WEB-DL.x264-GROUP.mkv".to_string());
        assert!(result.tags["source"].contains(&"HBO Max".to_string()));
    }

    #[test]
    fn test_file_containers() {
        // MKV container
        let result = parse_torrent_metadata("Movie.2023.1080p.WEB-DL.x264-GROUP.mkv".to_string());
        assert!(result.tags["other"].contains(&"MKV".to_string()));
        
        // MP4 container
        let result = parse_torrent_metadata("Movie.2023.1080p.WEB-DL.x264-GROUP.mp4".to_string());
        assert!(result.tags["other"].contains(&"MP4".to_string()));
        
        // AVI container
        let result = parse_torrent_metadata("Movie.2023.1080p.WEB-DL.x264-GROUP.avi".to_string());
        assert!(result.tags["other"].contains(&"AVI".to_string()));
    }

    #[test]
    fn test_file_sizes() {
        // 1.5GB size
        let result = parse_torrent_metadata("Movie.2023.1080p.WEB-DL.x264-GROUP.1.5GB.mkv".to_string());
        assert!(result.tags["other"].iter().any(|s| s.contains("1.5GB")));
        
        // 2GB size
        let result = parse_torrent_metadata("Movie.2023.1080p.WEB-DL.x264-GROUP.2GB.mkv".to_string());
        assert!(result.tags["other"].iter().any(|s| s.contains("2GB")));
        
        // 1.2TB size
        let result = parse_torrent_metadata("Movie.2023.1080p.WEB-DL.x264-GROUP.1.2TB.mkv".to_string());
        assert!(result.tags["other"].iter().any(|s| s.contains("1.2TB")));
    }

    #[test]
    fn test_crc32_hashes() {
        // CRC32 hash
        let result = parse_torrent_metadata("Movie.2023.1080p.WEB-DL.x264-GROUP.A1B2C3D4.mkv".to_string());
        assert!(result.tags["other"].iter().any(|s| s.contains("CRC:A1B2C3D4")));
    }

    #[test]
    fn test_release_groups() {
        // Release group at end
        let result = parse_torrent_metadata("Movie.2023.1080p.WEB-DL.x264-GROUP".to_string());
        assert!(result.tags["group"].contains(&"GROUP".to_string()));
    }

    #[test]
    fn test_clean_title() {
        let result = clean_title("Movie.2023.1080p.WEB-DL.x264-GROUP.mkv".to_string());
        assert_eq!(result.cleaned, "Movie 2023");
        assert_eq!(result.year, Some(2023));
    }

    #[test]
    fn test_normalize_for_metadata() {
        let (cleaned, year) = normalize_for_metadata("Movie.2023.1080p.WEB-DL.x264-GROUP.mkv".to_string());
        assert_eq!(cleaned, "Movie 2023");
        assert_eq!(year, Some(2023));
    }

    #[test]
    fn test_prettify_for_display() {
        let result = prettify_for_display("Movie.2023.1080p.WEB-DL.x264-GROUP.mkv".to_string());
        assert_eq!(result, "Movie 2023");
    }

    #[test]
    fn test_extract_year() {
        assert_eq!(extract_year("Movie.2023.1080p.WEB-DL.x264-GROUP.mkv"), Some(2023));
        assert_eq!(extract_year("Movie.1999.720p.BluRay.x264-GROUP.mkv"), Some(1999));
        assert_eq!(extract_year("Movie.2024.1080p.WEB-DL.x264-GROUP.mkv"), Some(2024));
    }

    // Additional comprehensive tests based on Rust metadata parser test data
    #[test]
    fn test_comprehensive_episode_patterns() {
        // Test various episode formats from Rust metadata parser
        let test_cases = vec![
            ("Show.S01E02.Episode.Title.1080p.WEB-DL.x264-GROUP.mkv", "Show"),
            ("Show.1x02.Episode.Title.1080p.WEB-DL.x264-GROUP.mkv", "Show"),
            ("Show.Season 1 Episode 2.1080p.WEB-DL.x264-GROUP.mkv", "Show"),
            ("Show.E02.1080p.WEB-DL.x264-GROUP.mkv", "Show"),
            ("Show.Ep02.1080p.WEB-DL.x264-GROUP.mkv", "Show"),
            ("Show.Episode 2.1080p.WEB-DL.x264-GROUP.mkv", "Show"),
        ];

        for (input, expected_title) in test_cases {
            let result = parse_torrent_metadata(input.to_string());
            assert_eq!(result.title, expected_title);
            assert!(result.tags["other"].contains(&"episode".to_string()));
        }
    }

    #[test]
    fn test_comprehensive_russian_patterns() {
        // Test Russian patterns
        let test_cases = vec![
            ("Сериал.Сезон 1.Серия 2.1080p.WEB-DL.x264-GROUP.mkv", "Сериал"),
            ("Сериал.1 Сезон.2 Серия.1080p.WEB-DL.x264-GROUP.mkv", "Сериал"),
            ("Сериал.Сезон 1.1080p.WEB-DL.x264-GROUP.mkv", "Сериал"),
            ("Сериал.Серия 2.1080p.WEB-DL.x264-GROUP.mkv", "Сериал"),
        ];

        for (input, expected_title) in test_cases {
            let result = parse_torrent_metadata(input.to_string());
            assert_eq!(result.title, expected_title);
            assert!(result.tags["other"].contains(&"russian_episode".to_string()));
        }
    }

    #[test]
    fn test_comprehensive_edition_tags() {
        // Test all edition tags
        let test_cases = vec![
            ("Movie.2023.PROPER.1080p.BluRay.x264-GROUP.mkv", "Proper"),
            ("Movie.2023.REPACK.1080p.BluRay.x264-GROUP.mkv", "Repack"),
            ("Movie.2023.EXTENDED.1080p.BluRay.x264-GROUP.mkv", "Extended"),
            ("Movie.2023.UNRATED.1080p.BluRay.x264-GROUP.mkv", "Unrated"),
            ("Movie.2023.DIRECTORS.CUT.1080p.BluRay.x264-GROUP.mkv", "Directors Cut"),
            ("Movie.2023.THEATRICAL.1080p.BluRay.x264-GROUP.mkv", "Theatrical"),
            ("Movie.2023.IMAX.1080p.BluRay.x264-GROUP.mkv", "IMAX"),
            ("Movie.2023.3D.1080p.BluRay.x264-GROUP.mkv", "3D"),
            ("Movie.2023.4K.1080p.BluRay.x264-GROUP.mkv", "4K"),
            ("Movie.2023.HDR.1080p.BluRay.x264-GROUP.mkv", "HDR"),
            ("Movie.2023.DOLBY.VISION.1080p.BluRay.x264-GROUP.mkv", "Dolby Vision"),
            ("Movie.2023.ATMOS.1080p.BluRay.x264-GROUP.mkv", "Atmos"),
            ("Movie.2023.REMASTERED.1080p.BluRay.x264-GROUP.mkv", "Remastered"),
            ("Movie.2023.CRITERION.1080p.BluRay.x264-GROUP.mkv", "Criterion Collection"),
        ];

        for (input, expected_edition) in test_cases {
            let result = crate::title_normalizer_core::parse_torrent_metadata(input.to_string());
            assert!(result.tags["version"].contains(&expected_edition.to_string()));
        }
    }

    #[test]
    fn test_comprehensive_streaming_services() {
        // Test all streaming services
        let test_cases = vec![
            ("Show.S01E01.NETFLIX.1080p.WEB-DL.x264-GROUP.mkv", "Netflix"),
            ("Show.S01E01.DISNEY+.1080p.WEB-DL.x264-GROUP.mkv", "Disney+"),
            ("Show.S01E01.HBO.MAX.1080p.WEB-DL.x264-GROUP.mkv", "HBO Max"),
            ("Show.S01E01.AMAZON.PRIME.1080p.WEB-DL.x264-GROUP.mkv", "Amazon Prime"),
            ("Show.S01E01.HULU.1080p.WEB-DL.x264-GROUP.mkv", "Hulu"),
            ("Show.S01E01.APPLE.TV+.1080p.WEB-DL.x264-GROUP.mkv", "Apple TV+"),
            ("Show.S01E01.PARAMOUNT+.1080p.WEB-DL.x264-GROUP.mkv", "Paramount+"),
            ("Show.S01E01.PEACOCK.1080p.WEB-DL.x264-GROUP.mkv", "Peacock"),
        ];

        for (input, expected_service) in test_cases {
            let result = parse_torrent_metadata(input.to_string());
            assert!(result.tags["source"].contains(&expected_service.to_string()));
        }
    }

    #[test]
    fn test_comprehensive_video_codecs() {
        // Test all video codecs
        let test_cases = vec![
            ("Movie.2023.1080p.WEB-DL.x264-GROUP.mkv", "H.264"),
            ("Movie.2023.1080p.WEB-DL.x265-GROUP.mkv", "H.265"),
            ("Movie.2023.1080p.WEB-DL.H264-GROUP.mkv", "H.264"),
            ("Movie.2023.1080p.WEB-DL.H265-GROUP.mkv", "H.265"),
            ("Movie.2023.1080p.WEB-DL.HEVC-GROUP.mkv", "H.265"),
            ("Movie.2023.1080p.WEB-DL.AVC-GROUP.mkv", "H.264"),
            ("Movie.2023.1080p.WEB-DL.XVID-GROUP.mkv", "XviD"),
            ("Movie.2023.1080p.WEB-DL.DIVX-GROUP.mkv", "DivX"),
            ("Movie.2023.1080p.WEB-DL.VP9-GROUP.mkv", "VP9"),
            ("Movie.2023.1080p.WEB-DL.MPEG-2-GROUP.mkv", "MPEG-2"),
            ("Movie.2023.1080p.WEB-DL.VC-1-GROUP.mkv", "VC-1"),
        ];

        for (input, expected_codec) in test_cases {
            let result = parse_torrent_metadata(input.to_string());
            assert!(result.tags["codec"].contains(&expected_codec.to_string()));
        }
    }

    #[test]
    fn test_comprehensive_audio_codecs() {
        // Test all audio codecs
        let test_cases = vec![
            ("Movie.2023.1080p.WEB-DL.x264.AAC-GROUP.mkv", "AAC"),
            ("Movie.2023.1080p.WEB-DL.x264.AC3-GROUP.mkv", "AC3"),
            ("Movie.2023.1080p.WEB-DL.x264.EAC3-GROUP.mkv", "EAC3"),
            ("Movie.2023.1080p.WEB-DL.x264.DDP-GROUP.mkv", "DDP"),
            ("Movie.2023.1080p.WEB-DL.x264.DTS-GROUP.mkv", "DTS"),
            ("Movie.2023.1080p.WEB-DL.x264.TRUEHD-GROUP.mkv", "TrueHD"),
            ("Movie.2023.1080p.WEB-DL.x264.ATMOS-GROUP.mkv", "Atmos"),
            ("Movie.2023.1080p.WEB-DL.x264.FLAC-GROUP.mkv", "FLAC"),
        ];

        for (input, expected_audio) in test_cases {
            let result = parse_torrent_metadata(input.to_string());
            assert!(result.tags["audio"].contains(&expected_audio.to_string()));
        }
    }

    #[test]
    fn test_comprehensive_sources() {
        // Test all sources
        let test_cases = vec![
            ("Movie.2023.1080p.WEB-DL.x264-GROUP.mkv", "WEB"),
            ("Movie.2023.1080p.WEBRIP.x264-GROUP.mkv", "WEBRIP"),
            ("Movie.2023.1080p.BluRay.x264-GROUP.mkv", "BluRay"),
            ("Movie.2023.1080p.BDRIP.x264-GROUP.mkv", "BDRIP"),
            ("Movie.2023.1080p.DVDRIP.x264-GROUP.mkv", "DVDRIP"),
            ("Movie.2023.1080p.HDTV.x264-GROUP.mkv", "HDTV"),
            ("Movie.2023.1080p.HDCAM.x264-GROUP.mkv", "HD Camera"),
            ("Movie.2023.1080p.HDTELESYNC.x264-GROUP.mkv", "HD Telesync"),
            ("Movie.2023.1080p.CAM.x264-GROUP.mkv", "Camera"),
            ("Movie.2023.1080p.TELESYNC.x264-GROUP.mkv", "Telesync"),
            ("Movie.2023.1080p.DVD.x264-GROUP.mkv", "DVD"),
            ("Movie.2023.1080p.VHS.x264-GROUP.mkv", "VHS"),
        ];

        for (input, expected_source) in test_cases {
            let result = parse_torrent_metadata(input.to_string());
            assert!(result.tags["source"].contains(&expected_source.to_string()));
        }
    }

    #[test]
    fn test_comprehensive_resolutions() {
        // Test all resolutions
        let test_cases = vec![
            ("Movie.2023.2160p.WEB-DL.x264-GROUP.mkv", "2160p"),
            ("Movie.2023.1080p.WEB-DL.x264-GROUP.mkv", "1080p"),
            ("Movie.2023.720p.WEB-DL.x264-GROUP.mkv", "720p"),
            ("Movie.2023.480p.WEB-DL.x264-GROUP.mkv", "480p"),
            ("Movie.2023.4K.WEB-DL.x264-GROUP.mkv", "4K"),
            ("Movie.2023.1920x1080.WEB-DL.x264-GROUP.mkv", "1080p"),
            ("Movie.2023.3840x2160.WEB-DL.x264-GROUP.mkv", "2160p"),
        ];

        for (input, expected_resolution) in test_cases {
            let result = parse_torrent_metadata(input.to_string());
            assert!(result.tags["resolution"].contains(&expected_resolution.to_string()));
        }
    }

    #[test]
    fn test_empty_input() {
        let result = parse_torrent_metadata("".to_string());
        assert_eq!(result.title, "");
        assert_eq!(result.year, None);
        assert!(result.tags.is_empty());
    }

    #[test]
    fn test_no_patterns_found() {
        let result = parse_torrent_metadata("JustSomeRandomString".to_string());
        assert_eq!(result.title, "JustSomeRandomString");
        assert_eq!(result.year, None);
        // Should have empty tags for all categories
        assert!(result.tags["resolution"].is_empty());
        assert!(result.tags["source"].is_empty());
        assert!(result.tags["codec"].is_empty());
        assert!(result.tags["audio"].is_empty());
        assert!(result.tags["version"].is_empty());
        assert!(result.tags["language"].is_empty());
        assert!(result.tags["group"].is_empty());
    }
}
