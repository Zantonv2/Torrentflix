use async_trait::async_trait;
use anyhow::Result;
use reqwest::Client;
use scraper::{Html, Selector};
use tracing::{debug, info, warn};
use tokio::time::{sleep, Duration};
use regex;
use std::collections::HashMap;
use futures;
use once_cell::sync::Lazy;

use crate::models::{TorrentResult};
use crate::indexers::traits::{Indexer, IndexerConfig, SearchQuery};

// Pre-compiled regex patterns for Send safety (case-insensitive)
// All patterns use \s* for flexible whitespace matching (Requirement 6.3)
// Colons are made optional with :? for flexible punctuation (Requirement 6.4)
static ORIGINAL_TITLE_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"(?i)Оригинальное\s+название\s*:?\s*(.+?)(?:\n|Год)").unwrap()
});
static YEAR_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"(?i)(?:Год\s+(?:выхода|выпуска)|ГОД)\s*:?\s*(\d{4})").unwrap()
});
static GENRE_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"(?i)Жанр\s*:?\s*([^\n<]+?)(?:<br>|Режиссёр|Режиссер|Время|$)").unwrap()
});
static DIRECTOR_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"(?i)(?:Режисс[её]р|РЕЖИССЕРСКИЙ\s+СОСТАВ)\s*:?\s*(.+?)(?:\n|В ролях|Время|$)").unwrap()
});
static CAST_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
    // Stop before description keywords: "О фильме", "Описание", "О Сериале" (case-insensitive)
    regex::Regex::new(r"(?i)(?:В\s+ролях|Актеры)\s*:?\s*(.+?)(?:\n\n|\.?\s*(?:О\s+фильме|Описание|О\s+Сериале)|Время|Формат|$)").unwrap()
});
// Support both "Продолжительность:" and "ВРЕМЯ:" formats
static RUNTIME_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"(?i)(?:Продолжительность|Время)\s*:?\s*(\d+):(\d+):(\d+)").unwrap()
});
// More flexible description regex - looks for text after cast or other markers
// Supports: "О фильме", "Описание", "О Сериале" (for TV shows)
static DESCRIPTION_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"(?i)(?:О\s+фильме|Описание|О\s+Сериале)\s*:?\s*(.+?)(?:Продолжительность|Время|Файл|Скачать|Формат|$)").unwrap()
});
// Alternative: extract description from paragraph text after cast section
static DESCRIPTION_FALLBACK_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"(?i)(?:В\s+ролях|Актеры)\s*:?\s*.+?\n\n(.+?)(?:Время|Продолжительность|Файл|Формат|Скачать|$)").unwrap()
});
static WHITESPACE_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"\s+").unwrap()
});
static BR_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"<br>+").unwrap()
});
static TIME_PATTERN_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
    // Matches time patterns like "01:42:27" or "1:42:27"
    regex::Regex::new(r"^\s*\d{1,2}:\d{2}:\d{2}\s*").unwrap()
});
// New regex patterns for additional field label variations
// All patterns use \s* for flexible whitespace matching (Requirement 6.3)
// Colons are made optional with :? for flexible punctuation (Requirement 6.4)
static COUNTRY_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"(?i)Страна\s*:?\s*(.+?)(?:\n|<br>|Студия|Режиссёр|Режиссер|$)").unwrap()
});
static STUDIO_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"(?i)(?:Студия|Выпущено)\s*:?\s*(.+?)(?:\n|<br>|Режиссёр|Режиссер|В\s+ролях|$)").unwrap()
});
static TRANSLATION_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"(?i)Перевод\s*:?\s*(.+?)(?:\n|<br>|Качество|Видео|$)").unwrap()
});
static QUALITY_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"(?i)Качество\s*:?\s*(.+?)(?:\n|<br>|Видео|Аудио|$)").unwrap()
});
static VIDEO_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"(?i)Видео\s*:?\s*(.+?)(?:\n|<br>|Аудио|Звук|Субтитры|$)").unwrap()
});
static AUDIO_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"(?i)(?:Аудио|Звук)\s*:?\s*(.+?)(?:\n|<br>|Субтитры|Формат|$)").unwrap()
});
static SUBTITLES_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"(?i)Субтитры\s*:?\s*(.+?)(?:\n|<br>|Формат|Файл|$)").unwrap()
});

// Pre-compiled selectors for Send safety
static H1_SELECTOR: Lazy<Selector> = Lazy::new(|| {
    Selector::parse("h1").unwrap()
});
static FULLSTORY_SELECTOR: Lazy<Selector> = Lazy::new(|| {
    Selector::parse("div.fullstory").unwrap()
});
static INFO_TABLE_SELECTOR: Lazy<Selector> = Lazy::new(|| {
    Selector::parse("div.fullstory table tr").unwrap()
});
static INFO_CELL_SELECTOR: Lazy<Selector> = Lazy::new(|| {
    Selector::parse("td").unwrap()
});
static IMG_SELECTOR: Lazy<Selector> = Lazy::new(|| {
    Selector::parse("img").unwrap()
});
static OG_IMAGE_SELECTOR: Lazy<Selector> = Lazy::new(|| {
    Selector::parse("meta[property='og:image']").unwrap()
});
static DLE_CONTENT_SELECTOR: Lazy<Selector> = Lazy::new(|| {
    Selector::parse("#dle-content").unwrap()
});
static ARTICLE_LINK_SELECTOR: Lazy<Selector> = Lazy::new(|| {
    Selector::parse("a[href*='.html']").unwrap()
});
static PARAGRAPH_SELECTOR: Lazy<Selector> = Lazy::new(|| {
    Selector::parse("p").unwrap()
});

/// Metadata extracted from Monna2 detail pages
#[derive(Debug, Clone, Default)]
pub struct MonnaMetadata {
    pub title: String,
    pub original_title: Option<String>,
    pub year: Option<u32>,
    pub genres: Vec<String>,
    pub director: Option<String>,
    pub cast: Vec<String>,
    pub runtime_minutes: Option<u32>,
    pub description: Option<String>,
    pub poster_url: Option<String>,
    pub kinopoisk_id: Option<String>,
    pub kinopoisk_url: Option<String>,
    pub is_series: bool, // Flag to indicate if this is a series (had "сериал" in title)
    pub country: Option<String>,
    pub studio: Option<String>,
    pub translation: Option<String>,
    pub quality: Option<String>,
    pub video_codec: Option<String>,
    pub audio_format: Option<String>,
    pub subtitles: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MonnaIndexer {
    client: Client,
    base_url: String,
    config: IndexerConfig,
}

/// Normalize title: remove "сериал" prefix/suffix, apply title case if all caps
/// Returns (normalized_title, is_series)
fn normalize_title(title: &str) -> (String, bool) {
    let mut normalized = title.trim().to_string();
    let mut is_series = false;
    
    // Check for "сериал" (case-insensitive) and remove it
    let title_lower = normalized.to_lowercase();
    if title_lower.contains("сериал") {
        is_series = true;
        // Remove "сериал" from various positions using regex
        normalized = regex::Regex::new(r"(?i)\s*сериал\s*")
            .unwrap()
            .replace_all(&normalized, " ")
            .to_string();
        // Clean up multiple spaces
        normalized = regex::Regex::new(r"\s+")
            .unwrap()
            .replace_all(&normalized, " ")
            .to_string();
        normalized = normalized.trim().to_string();
    }
    
    // Apply title case if the title is all caps
    if normalized == normalized.to_uppercase() && normalized.chars().any(|c| c.is_alphabetic()) {
        normalized = to_title_case(&normalized);
    }
    
    (normalized.trim().to_string(), is_series)
}

/// Convert text to title case (first letter uppercase, rest lowercase)
/// Handles both Latin and Cyrillic characters
fn to_title_case(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    
    let chars: Vec<char> = text.chars().collect();
    let mut result = String::with_capacity(text.len());
    let mut capitalize_next = true;
    
    for ch in chars.iter() {
        if capitalize_next {
            // Convert to uppercase (handles both Latin and Cyrillic)
            result.push(ch.to_uppercase().next().unwrap_or(*ch));
            capitalize_next = false;
        } else {
            // Convert to lowercase
            result.push(ch.to_lowercase().next().unwrap_or(*ch));
        }
        
        // Next char should be capitalized after spaces, hyphens, etc.
        if ch.is_whitespace() || *ch == '-' || *ch == '/' {
            capitalize_next = true;
        }
    }
    
    result
}

/// Filter out non-genre text from genre list
/// Removes: release info (Выпущено:), release groups, distributors, etc.
fn filter_valid_genres(genres: Vec<String>) -> Vec<String> {
    use regex::Regex;
    static NAME_PATTERN: Lazy<Regex> = Lazy::new(|| {
        // Pattern to detect names: capital letter followed by lowercase (like "John Smith")
        Regex::new(r"[А-ЯA-Z][а-яa-z]+\s+[А-ЯA-Z]").unwrap()
    });
    
    genres
        .into_iter()
        .filter(|g| {
            let g = g.trim();
            // Skip empty
            if g.is_empty() {
                return false;
            }
            // Skip if too long (likely not a genre)
            if g.len() > 30 {
                return false;
            }
            // Skip if contains colon (like "Выпущено: Дания")
            if g.contains(':') {
                return false;
            }
            // Skip if looks like a name (capital letters in middle)
            if NAME_PATTERN.is_match(g) {
                return false;
            }
            // Skip common release group/distributor patterns
            let g_lower = g.to_lowercase();
            if g_lower.contains("выпущено") || g_lower.contains("released") || 
               g_lower.contains("страна") || g_lower.contains("country") ||
               g_lower.contains("студия") || g_lower.contains("studio") ||
               g_lower.contains("дистрибьютор") || g_lower.contains("distributor") {
                return false;
            }
            // Allow all-caps if it's a known genre (Russian genres can be all caps)
            // Only filter out if it's a short acronym (likely release group like "WEB", "HD", etc.)
            if g == g.to_uppercase() && g.len() <= 4 && g.chars().all(|c| c.is_alphabetic()) {
                // Filter out common acronyms that aren't genres
                let g_upper = g.to_uppercase();
                if g_upper == "WEB" || g_upper == "HD" || g_upper == "SD" || g_upper == "DVDRIP" || 
                   g_upper == "BDRIP" || g_upper == "TS" || g_upper == "TC" || g_upper == "CAM" {
                    return false;
                }
            }
            true
        })
        .map(|g| to_title_case(&g)) // Convert to title case
        .take(5) // Max 5 genres
        .collect()
}

impl MonnaIndexer {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            client,
            base_url: "https://tv.monna2.top".to_string(),
            config: IndexerConfig::default(),
        }
    }

    async fn fetch_page(&self, url: &str) -> Result<String> {
        debug!("Fetching page: {}", url);
        
        // Add rate limiting to avoid connection resets
        sleep(Duration::from_millis(self.config.rate_limit_ms)).await;
        
        let response = self.client.get(url).send().await?;
        let html = response.text().await?;
        Ok(html)
    }

    fn parse_movie_list(&self, html: &str) -> Result<Vec<TorrentResult>> {
        
        // This function needs to be async for metadata fetching
        // We'll handle this in the calling function
        Ok(vec![])
    }

    /// Extract detail URLs from main page HTML (synchronous, no Send issues)
    fn extract_detail_urls(html: &str, base_url: &str) -> Vec<(String, &'static str)> {
        let document = Html::parse_document(html);

        // Use correct selectors from Python implementation
        // Find main content area first (like #dle-content)
        let main_content = document.select(&DLE_CONTENT_SELECTOR)
            .next()
            .unwrap_or_else(|| document.root_element());
        
        // Find all article links (correct approach from Python)
        let article_links: Vec<_> = main_content.select(&ARTICLE_LINK_SELECTOR).collect();
        
        
        // Normalize Russian category names to English equivalents
        fn normalize_category(url: &str) -> Option<&'static str> {
            if url.contains("/boevik/") { Some("action") }
            else if url.contains("/drama/") { Some("drama") }
            else if url.contains("/serial/") { Some("series") }
            else if url.contains("/triller/") { Some("thriller") }
            else if url.contains("/komediya/") { Some("comedy") }
            else if url.contains("/fantastika/") { Some("scifi") }
            else if url.contains("/uzhasy/") { Some("horror") }
            else if url.contains("/dokumentalnyy/") { Some("documentary") }
            else if url.contains("/melodrama/") { Some("romance") }
            else if url.contains("/priklucheniya/") { Some("adventure") }
            else if url.contains("/semeynyy/") { Some("family") }
            else if url.contains("/voennyy/") { Some("war") }
            else if url.contains("/istoriya/") { Some("history") }
            else if url.contains("/biografiya/") { Some("biography") }
            else if url.contains("/sport/") { Some("sport") }
            else if url.contains("/multfilm/") { Some("animation") }
            else if url.contains("/ujas/") { Some("horror") }
            else { None }
        }
        
        // Extract ALL data from scraper types and return plain strings
        let mut detail_urls = Vec::new();
        for link_elem in article_links {
            if detail_urls.len() >= 10 {
                break; // Limit to first 10 for feed
            }
            
            if let Some(href) = link_elem.value().attr("href") {
                // Build full URL
                let full_url = if href.starts_with("/") {
                    format!("{}{}", base_url, href)
                } else if href.starts_with("http") {
                    href.to_string()
                } else {
                    continue;
                };
                
                // Filter by valid categories and get normalized name
                let category = normalize_category(&full_url);
                if let Some(cat) = category {
                    // Skip non-content pages
                    if full_url.contains("xfsearch") || full_url.contains("page/") || full_url.contains("disklaimer") {
                        continue;
                    }
                    
                    detail_urls.push((full_url, cat));
                }
            }
        }
        
        detail_urls
    }

    /// Parse movie list and fetch metadata in parallel (async version)
    async fn parse_movie_list_with_metadata(&self, html: &str) -> Result<Vec<TorrentResult>> {
        
        // Extract URLs synchronously (no Send issues)
        let detail_urls = Self::extract_detail_urls(html, &self.base_url);
        
        // All HTML parsing is complete, now we can safely use await points
        
        // Create semaphore to limit concurrent requests (like Python implementation)
        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(10));
        let mut tasks = Vec::new();
        
        for (url, category) in detail_urls {
            let semaphore = semaphore.clone();
            let client = self.client.clone();
            let base_url = self.base_url.clone();
            
            let task = tokio::spawn(async move {
                // Gracefully handle semaphore acquisition failure
                let _permit = match semaphore.acquire().await {
                    Ok(permit) => permit,
                    Err(_) => return None,
                };
                
                // Only fetch HTML in parallel (no parsing, no Send issues)
                match Self::fetch_page_html(&client, &base_url, &url).await {
                    Ok(html) => {
                        Some((url, category, html))
                    }
                    Err(_) => {
                        None
                    }
                }
            });
            
            tasks.push(task);
        }
        
        // Wait for all parallel fetches to complete
        let results = futures::future::join_all(tasks).await;
        let mut movies = Vec::new();
        
        // Parse HTML sequentially (no Send issues)
        for result in results {
            if let Ok(Some((url, category, html))) = result {
                match Self::parse_metadata_from_html(&html, &self.base_url) {
                    Ok(metadata) => {
                        
                        // Create torrent result with metadata
                        let mut torrent_result = TorrentResult::new(
                            metadata.title.clone(),
                            String::new(), // magnet_link - will be populated later
                            0,             // size_bytes
                            0,             // seeders
                            0,             // leechers
                            "monna".to_string(),
                        );
                        
                        // Store all metadata from MonnaIndexer
                        // Use is_series flag from metadata (detected from "сериал" in title)
                        // or fall back to URL category
                        let is_series = metadata.is_series || category == "series";
                        torrent_result.category = if is_series {
                            Some("series".to_string())
                        } else {
                            Some(category.to_string())
                        };
                        torrent_result.poster_url = metadata.poster_url.clone();
                        torrent_result.description = metadata.description.clone();
                        torrent_result.cast = metadata.cast.clone();
                        torrent_result.runtime_minutes = metadata.runtime_minutes;
                        torrent_result.genres = metadata.genres.clone();
                        
                        if let Some(poster_url) = torrent_result.poster_url.as_ref() {
                        }
                        if let Some(description) = torrent_result.description.as_ref() {
                        }
                        if !torrent_result.cast.is_empty() {
                        }
                        if let Some(runtime) = torrent_result.runtime_minutes {
                        }
                        
                        movies.push(torrent_result);
                    }
                    Err(e) => {
                    }
                }
            }
        }
        
        info!("Successfully parsed {} movies with metadata", movies.len());
        Ok(movies)
    }
    async fn fetch_page_html(client: &Client, _base_url: &str, url: &str) -> Result<String> {
        let response = client.get(url).send().await?;
        let html = response.text().await?;
        Ok(html)
    }

    /// Helper function to parse metadata from HTML (no async, no Send issues)
    fn parse_metadata_from_html(html: &str, base_url: &str) -> Result<MonnaMetadata> {
        let document = Html::parse_document(html);
        let mut metadata = MonnaMetadata::default();
        
        // Extract title from h1
        if let Some(h1) = document.select(&H1_SELECTOR).next() {
            let title = h1.text().collect::<String>().trim().to_string();
            // Remove "скачать торрент" suffix if present
            let cleaned_title = title.replace("скачать торрент", "").trim().to_string();
            // Normalize title (remove "сериал", apply title case if all caps)
            let (normalized_title, is_series_from_title) = normalize_title(&cleaned_title);
            metadata.title = normalized_title;
            metadata.is_series = is_series_from_title;
            // The title normalization already removed "сериал" from the display title
            // but we keep the is_series flag to distinguish shows from movies
        }
        
        // Extract metadata from fullstory div
        if let Some(fullstory) = document.select(&FULLSTORY_SELECTOR).next() {
            let text = fullstory.text().collect::<String>();
            let mut genres_from_dom: Vec<String> = Vec::new();
            // Try DOM table extraction first
            for row in fullstory.select(&INFO_TABLE_SELECTOR) {
                let mut cells = row.select(&INFO_CELL_SELECTOR);
                if let (Some(label_cell), Some(value_cell)) = (cells.next(), cells.next()) {
                    let label = label_cell.text().collect::<String>().trim().to_lowercase();
                    if label.starts_with("жанр") {
                        let value_html = value_cell.html();
                        let cleaned = BR_REGEX.replace_all(&value_html, ",");
                        genres_from_dom = cleaned
                            .split(',')
                            .map(|g| g.trim())
                            .filter(|g| !g.is_empty())
                            .map(|g| g.to_string())
                            .collect();
                        break;
                    }
                }
            }
            
            // If DOM extraction failed, try regex on text (handles all-caps format)
            if genres_from_dom.is_empty() {
                if let Some(genre_match) = GENRE_REGEX.captures(&text) {
                    if let Some(genres_capture) = genre_match.get(1) {
                        let genres_str = genres_capture.as_str().trim();
                        genres_from_dom = genres_str
                            .split(',')
                            .map(|g| g.trim().to_string())
                            .filter(|g| !g.is_empty())
                            .collect();
                    }
                }
            }
            
            // Extract original title - gracefully handle missing capture group
            if let Some(original_match) = ORIGINAL_TITLE_REGEX.captures(&text) {
                if let Some(title_capture) = original_match.get(1) {
                    metadata.original_title = Some(title_capture.as_str().trim().to_string());
                }
            }
            
            // Extract year - gracefully handle missing capture group and parse errors
            if let Some(year_match) = YEAR_REGEX.captures(&text) {
                if let Some(year_capture) = year_match.get(1) {
                    metadata.year = year_capture.as_str().parse().ok();
                }
            }
            
            // Extract genres with filtering (genres_from_dom may have been populated by DOM or regex fallback)
            if !genres_from_dom.is_empty() {
                metadata.genres = filter_valid_genres(genres_from_dom);
            } else {
                // Last resort: try regex directly on text
                if let Some(genre_match) = GENRE_REGEX.captures(&text) {
                    if let Some(genres_capture) = genre_match.get(1) {
                        let genres_str = genres_capture.as_str().trim();
                        let raw_genres: Vec<String> = genres_str.split(',').map(|g| g.trim().to_string()).collect();
                        metadata.genres = filter_valid_genres(raw_genres);
                    }
                }
            }
            
            // Extract director - gracefully handle missing capture group
            if let Some(director_match) = DIRECTOR_REGEX.captures(&text) {
                if let Some(director_capture) = director_match.get(1) {
                    metadata.director = Some(director_capture.as_str().trim().to_string());
                }
            }
            
            // Extract cast - gracefully handle missing capture group
            if let Some(cast_match) = CAST_REGEX.captures(&text) {
                if let Some(cast_capture) = cast_match.get(1) {
                    let cast_str = cast_capture.as_str().trim();
                    
                    // Additional cleanup: remove description keywords that might have been captured
                    // Handle cases like "И Другие.о Сериале:" or "И Другие.О фильме:" (no space after period)
                    if let Ok(desc_keywords) = regex::Regex::new(r"(?i)\.?\s*(?:о фильме|описание|о сериале):.*$") {
                        let cast_str = desc_keywords.replace(cast_str, "");
                        
                        // Also remove trailing "И Другие" if it's followed by description keywords
                        if let Ok(others_pattern) = regex::Regex::new(r"(?i)\s*и другие\.?\s*(?:о фильме|описание|о сериале).*$") {
                            let cast_str = others_pattern.replace(&cast_str, "");
                            
                            metadata.cast = cast_str.trim().split(',')
                                .map(|c| c.trim())
                                .filter(|c| !c.is_empty())
                                .map(|c| to_title_case(c))
                                .take(10)
                                .collect();
                        }
                    }
                }
            }
            
            // Extract runtime (supports both "Продолжительность:" and "ВРЕМЯ:") - gracefully handle parse errors
            if let Some(runtime_match) = RUNTIME_REGEX.captures(&text) {
                if let (Some(hours_capture), Some(minutes_capture)) = (runtime_match.get(1), runtime_match.get(2)) {
                    if let (Ok(hours), Ok(minutes)) = (
                        hours_capture.as_str().parse::<u32>(),
                        minutes_capture.as_str().parse::<u32>()
                    ) {
                        metadata.runtime_minutes = Some(hours * 60 + minutes);
                    }
                }
            }
            
            // Extract country (supports "Страна:" and "СТРАНА:" case-insensitively) - gracefully handle missing capture group
            if let Some(country_match) = COUNTRY_REGEX.captures(&text) {
                if let Some(country_capture) = country_match.get(1) {
                    let country = country_capture.as_str().trim().to_string();
                    if !country.is_empty() {
                        metadata.country = Some(country);
                    }
                }
            }
            
            // Extract studio (supports "Студия:" and "Выпущено:" case-insensitively) - gracefully handle missing capture group
            if let Some(studio_match) = STUDIO_REGEX.captures(&text) {
                if let Some(studio_capture) = studio_match.get(1) {
                    let studio = studio_capture.as_str().trim().to_string();
                    if !studio.is_empty() {
                        metadata.studio = Some(studio);
                    }
                }
            }
            
            // Extract translation (supports "Перевод:" case-insensitively) - gracefully handle missing capture group
            if let Some(translation_match) = TRANSLATION_REGEX.captures(&text) {
                if let Some(translation_capture) = translation_match.get(1) {
                    let translation = translation_capture.as_str().trim().to_string();
                    if !translation.is_empty() {
                        metadata.translation = Some(translation);
                    }
                }
            }
            
            // Extract quality metadata - gracefully handle missing capture groups
            if let Some(quality_match) = QUALITY_REGEX.captures(&text) {
                if let Some(quality_capture) = quality_match.get(1) {
                    let quality = quality_capture.as_str().trim().to_string();
                    if !quality.is_empty() {
                        metadata.quality = Some(quality);
                    }
                }
            }
            
            if let Some(video_match) = VIDEO_REGEX.captures(&text) {
                if let Some(video_capture) = video_match.get(1) {
                    let video = video_capture.as_str().trim().to_string();
                    if !video.is_empty() {
                        metadata.video_codec = Some(video);
                    }
                }
            }
            
            if let Some(audio_match) = AUDIO_REGEX.captures(&text) {
                if let Some(audio_capture) = audio_match.get(1) {
                    let audio = audio_capture.as_str().trim().to_string();
                    if !audio.is_empty() {
                        metadata.audio_format = Some(audio);
                    }
                }
            }
            
            if let Some(subtitles_match) = SUBTITLES_REGEX.captures(&text) {
                if let Some(subtitles_capture) = subtitles_match.get(1) {
                    let subtitles = subtitles_capture.as_str().trim().to_string();
                    if !subtitles.is_empty() {
                        metadata.subtitles = Some(subtitles);
                    }
                }
            }
            
            // Extract description - try main regex first, then fallback
            if let Some(desc_match) = DESCRIPTION_REGEX.captures(&text) {
                let mut description = desc_match.get(1).map(|c| c.as_str()).unwrap_or("").trim().to_string();
                description = WHITESPACE_REGEX.replace_all(&description, " ").to_string();
                description = BR_REGEX.replace_all(&description, " ").to_string();
                if !description.is_empty() && description.len() > 20 {
                metadata.description = Some(description);
                }
            } else if let Some(desc_match) = DESCRIPTION_FALLBACK_REGEX.captures(&text) {
                // Fallback: extract text after cast section
                let mut description = desc_match.get(1).map(|c| c.as_str()).unwrap_or("").trim().to_string();
                description = WHITESPACE_REGEX.replace_all(&description, " ").to_string();
                description = BR_REGEX.replace_all(&description, " ").to_string();
                // Filter out very short or likely non-description text
                if !description.is_empty() && description.len() > 50 {
                    metadata.description = Some(description);
                }
            }
            
            // If still no description, try to extract from paragraph tags in fullstory
            if metadata.description.is_none() {
                let paragraphs: Vec<String> = fullstory.select(&PARAGRAPH_SELECTOR)
                    .map(|p| {
                        let text = p.text().collect::<String>();
                        let mut cleaned = WHITESPACE_REGEX.replace_all(&text, " ").to_string();
                        cleaned = cleaned.trim().to_string();
                        // Remove time pattern from start if present
                        cleaned = TIME_PATTERN_REGEX.replace(&cleaned, "").to_string();
                        cleaned.trim().to_string()
                    })
                    .filter(|p| {
                        // Filter out technical information and validate length (> 100 characters per requirement 4.5)
                        p.len() > 100 && 
                        !p.to_lowercase().contains("скачать") &&
                        !p.to_lowercase().contains("торрент") &&
                        !p.to_lowercase().starts_with("формат:") &&
                        !p.to_lowercase().starts_with("видео:") &&
                        !p.to_lowercase().starts_with("аудио:") &&
                        !p.to_lowercase().starts_with("качество:") &&
                        !p.to_lowercase().starts_with("субтитры:") &&
                        !TIME_PATTERN_REGEX.is_match(p) // Filter out paragraphs that are just time
                    })
                    .collect();
                
                if !paragraphs.is_empty() {
                    // Join paragraphs and use as description
                    let combined = paragraphs.join(" ");
                    // Clean up the combined text - remove any leading time patterns
                    let mut final_desc = TIME_PATTERN_REGEX.replace(&combined, "").to_string();
                    final_desc = final_desc.trim().to_string();
                    // Validate length (> 100 characters per requirement 4.5)
                    if final_desc.len() > 100 {
                        metadata.description = Some(final_desc);
                    }
                }
            }
            
            // Last resort: extract any long text block from fullstory that looks like description
            if metadata.description.is_none() {
                // Look for text that comes after cast/runtime but before technical info
                let full_text = fullstory.text().collect::<String>();
                // Try to find text between cast and "Формат:" or "Скачать"
                if let Some(cast_end) = full_text.to_lowercase().find("время:") {
                    let after_cast = &full_text[cast_end..];
                    if let Some(format_start) = after_cast.to_lowercase().find("формат:") {
                        let desc_candidate = &after_cast[..format_start];
                        let mut cleaned = WHITESPACE_REGEX.replace_all(desc_candidate, " ").to_string();
                        cleaned = cleaned.trim().to_string();
                        // Remove "ВРЕМЯ:" or "Время:" prefix (case-insensitive)
                        cleaned = regex::Regex::new(r"(?i)^\s*время:\s*")
                            .unwrap()
                            .replace(&cleaned, "")
                            .to_string();
                        // Remove time pattern (like "01:42:27") from the start
                        cleaned = TIME_PATTERN_REGEX.replace(&cleaned, "").to_string();
                        cleaned = cleaned.trim().to_string();
                        // Filter out technical information and validate length (> 100 characters per requirement 4.5)
                        let cleaned_lower = cleaned.to_lowercase();
                        if !cleaned_lower.starts_with("формат:") && 
                           !cleaned_lower.starts_with("скачать") &&
                           !cleaned_lower.starts_with("видео:") &&
                           !cleaned_lower.starts_with("аудио:") &&
                           !cleaned_lower.starts_with("качество:") &&
                           !cleaned_lower.starts_with("субтитры:") &&
                           cleaned.len() > 100 {
                            metadata.description = Some(cleaned);
                        }
                    }
                }
            }
            
            // Extract poster URL (priority: uploads/posts images)
            let all_imgs = fullstory.select(&IMG_SELECTOR);
            for img in all_imgs {
                if let Some(src) = img.value().attr("src") {
                    if src.contains("/uploads/posts/") {
                        let poster_url = if src.starts_with("/") {
                            format!("{}{}", base_url, src)
                        } else {
                            src.to_string()
                        };
                        metadata.poster_url = Some(poster_url);
                        break;
                    }
                }
            }
            
            // Fallback to og:image if no uploads/posts found
            if metadata.poster_url.is_none() {
                if let Some(og_image) = document.select(&OG_IMAGE_SELECTOR).next() {
                    if let Some(content) = og_image.value().attr("content") {
                        metadata.poster_url = Some(content.to_string());
                    }
                }
            }
        }
        
        Ok(metadata)
    }

    /// Extract detailed metadata from a Monna2 detail page
    async fn fetch_movie_metadata(&self, detail_url: &str) -> Result<MonnaMetadata> {
        
        let html = self.fetch_page(detail_url).await?;
        let document = Html::parse_document(&html);
        
        let mut metadata = MonnaMetadata::default();
        
        // Extract title from h1
        if let Some(h1) = document.select(&H1_SELECTOR).next() {
            let title = h1.text().collect::<String>().trim().to_string();
            // Remove "скачать торрент" suffix if present
            let cleaned_title = title.replace("скачать торрент", "").trim().to_string();
            // Normalize title (remove "сериал", apply title case if all caps)
            let (normalized_title, is_series_from_title) = normalize_title(&cleaned_title);
            metadata.title = normalized_title;
            metadata.is_series = is_series_from_title;
            // The title normalization already removed "сериал" from the display title
            // but we keep the is_series flag to distinguish shows from movies
        }
        
        // Extract metadata from fullstory div
        if let Some(fullstory) = document.select(&FULLSTORY_SELECTOR).next() {
            let text = fullstory.text().collect::<String>();
            
            // Extract original title - gracefully handle missing capture group
            if let Some(original_match) = ORIGINAL_TITLE_REGEX.captures(&text) {
                if let Some(title_capture) = original_match.get(1) {
                    metadata.original_title = Some(title_capture.as_str().trim().to_string());
                }
            }
            
            // Extract year - gracefully handle missing capture group and parse errors
            if let Some(year_match) = YEAR_REGEX.captures(&text) {
                if let Some(year_capture) = year_match.get(1) {
                    metadata.year = year_capture.as_str().parse().ok();
                }
            }
            
            // Extract genres with filtering - gracefully handle missing capture group
            if let Some(genre_match) = GENRE_REGEX.captures(&text) {
                if let Some(genres_capture) = genre_match.get(1) {
                    let genres_str = genres_capture.as_str().trim();
                    let raw_genres: Vec<String> = genres_str.split(',').map(|g| g.trim().to_string()).collect();
                    metadata.genres = filter_valid_genres(raw_genres);
                }
            }
            
            // Extract director - gracefully handle missing capture group
            if let Some(director_match) = DIRECTOR_REGEX.captures(&text) {
                if let Some(director_capture) = director_match.get(1) {
                    metadata.director = Some(director_capture.as_str().trim().to_string());
                }
            }
            
            // Extract cast - gracefully handle missing capture group
            if let Some(cast_match) = CAST_REGEX.captures(&text) {
                if let Some(cast_capture) = cast_match.get(1) {
                    let cast_str = cast_capture.as_str().trim();
                    
                    // Additional cleanup: remove description keywords that might have been captured
                    // Handle cases like "И Другие.о Сериале:" or "И Другие.О фильме:" (no space after period)
                    if let Ok(desc_keywords) = regex::Regex::new(r"(?i)\.?\s*(?:о фильме|описание|о сериале):.*$") {
                        let cast_str = desc_keywords.replace(cast_str, "");
                        
                        // Also remove trailing "И Другие" if it's followed by description keywords
                        if let Ok(others_pattern) = regex::Regex::new(r"(?i)\s*и другие\.?\s*(?:о фильме|описание|о сериале).*$") {
                            let cast_str = others_pattern.replace(&cast_str, "");
                            
                            metadata.cast = cast_str.trim().split(',')
                                .map(|c| c.trim())
                                .filter(|c| !c.is_empty())
                                .map(|c| to_title_case(c))
                                .take(10)
                                .collect();
                        }
                    }
                }
            }
            
            // Extract runtime (supports both "Продолжительность:" and "ВРЕМЯ:") - gracefully handle parse errors
            if let Some(runtime_match) = RUNTIME_REGEX.captures(&text) {
                if let (Some(hours_capture), Some(minutes_capture)) = (runtime_match.get(1), runtime_match.get(2)) {
                    if let (Ok(hours), Ok(minutes)) = (
                        hours_capture.as_str().parse::<u32>(),
                        minutes_capture.as_str().parse::<u32>()
                    ) {
                        metadata.runtime_minutes = Some(hours * 60 + minutes);
                    }
                }
            }
            
            // Extract description - try main regex first, then fallback
            if let Some(desc_match) = DESCRIPTION_REGEX.captures(&text) {
                let mut description = desc_match.get(1).map(|c| c.as_str()).unwrap_or("").trim().to_string();
                description = WHITESPACE_REGEX.replace_all(&description, " ").to_string();
                description = BR_REGEX.replace_all(&description, " ").to_string();
                if !description.is_empty() && description.len() > 20 {
                metadata.description = Some(description);
                }
            } else if let Some(desc_match) = DESCRIPTION_FALLBACK_REGEX.captures(&text) {
                // Fallback: extract text after cast section
                let mut description = desc_match.get(1).map(|c| c.as_str()).unwrap_or("").trim().to_string();
                description = WHITESPACE_REGEX.replace_all(&description, " ").to_string();
                description = BR_REGEX.replace_all(&description, " ").to_string();
                // Filter out very short or likely non-description text
                if !description.is_empty() && description.len() > 50 {
                    metadata.description = Some(description);
                }
            }
            
            // If still no description, try to extract from paragraph tags in fullstory
            if metadata.description.is_none() {
                let paragraphs: Vec<String> = fullstory.select(&PARAGRAPH_SELECTOR)
                    .map(|p| {
                        let text = p.text().collect::<String>();
                        let mut cleaned = WHITESPACE_REGEX.replace_all(&text, " ").to_string();
                        cleaned = cleaned.trim().to_string();
                        // Remove time pattern from start if present
                        cleaned = TIME_PATTERN_REGEX.replace(&cleaned, "").to_string();
                        cleaned.trim().to_string()
                    })
                    .filter(|p| {
                        // Filter out technical information and validate length (> 100 characters per requirement 4.5)
                        p.len() > 100 && 
                        !p.to_lowercase().contains("скачать") &&
                        !p.to_lowercase().contains("торрент") &&
                        !p.to_lowercase().starts_with("формат:") &&
                        !p.to_lowercase().starts_with("видео:") &&
                        !p.to_lowercase().starts_with("аудио:") &&
                        !p.to_lowercase().starts_with("качество:") &&
                        !p.to_lowercase().starts_with("субтитры:") &&
                        !TIME_PATTERN_REGEX.is_match(p) // Filter out paragraphs that are just time
                    })
                    .collect();
                
                if !paragraphs.is_empty() {
                    // Join paragraphs and use as description
                    let combined = paragraphs.join(" ");
                    // Clean up the combined text - remove any leading time patterns
                    let mut final_desc = TIME_PATTERN_REGEX.replace(&combined, "").to_string();
                    final_desc = final_desc.trim().to_string();
                    // Validate length (> 100 characters per requirement 4.5)
                    if final_desc.len() > 100 {
                        metadata.description = Some(final_desc);
                    }
                }
            }
            
            // Last resort: extract any long text block from fullstory that looks like description
            if metadata.description.is_none() {
                // Look for text that comes after cast/runtime but before technical info
                let full_text = fullstory.text().collect::<String>();
                // Try to find text between cast and "Формат:" or "Скачать"
                if let Some(cast_end) = full_text.to_lowercase().find("время:") {
                    let after_cast = &full_text[cast_end..];
                    if let Some(format_start) = after_cast.to_lowercase().find("формат:") {
                        let desc_candidate = &after_cast[..format_start];
                        let mut cleaned = WHITESPACE_REGEX.replace_all(desc_candidate, " ").to_string();
                        cleaned = cleaned.trim().to_string();
                        // Remove "ВРЕМЯ:" or "Время:" prefix (case-insensitive)
                        cleaned = regex::Regex::new(r"(?i)^\s*время:\s*")
                            .unwrap()
                            .replace(&cleaned, "")
                            .to_string();
                        // Remove time pattern (like "01:42:27") from the start
                        cleaned = TIME_PATTERN_REGEX.replace(&cleaned, "").to_string();
                        cleaned = cleaned.trim().to_string();
                        // Filter out technical information and validate length (> 100 characters per requirement 4.5)
                        let cleaned_lower = cleaned.to_lowercase();
                        if !cleaned_lower.starts_with("формат:") && 
                           !cleaned_lower.starts_with("скачать") &&
                           !cleaned_lower.starts_with("видео:") &&
                           !cleaned_lower.starts_with("аудио:") &&
                           !cleaned_lower.starts_with("качество:") &&
                           !cleaned_lower.starts_with("субтитры:") &&
                           cleaned.len() > 100 {
                            metadata.description = Some(cleaned);
                        }
                    }
                }
            }
            
            // Extract poster URL (priority: uploads/posts images)
            let all_imgs = fullstory.select(&IMG_SELECTOR);
            for img in all_imgs {
                if let Some(src) = img.value().attr("src") {
                    if src.contains("/uploads/posts/") {
                        let poster_url = if src.starts_with("/") {
                            format!("{}{}", self.base_url, src)
                        } else {
                            src.to_string()
                        };
                        metadata.poster_url = Some(poster_url);
                        break;
                    }
                }
            }
            
            // Fallback to og:image if no uploads/posts found
            if metadata.poster_url.is_none() {
                if let Some(og_image) = document.select(&OG_IMAGE_SELECTOR).next() {
                    if let Some(content) = og_image.value().attr("content") {
                        metadata.poster_url = Some(content.to_string());
                    }
                }
            }
        }
        
        Ok(metadata)
    }

    fn extract_movie_from_prewposter(&self, movie_elem: &scraper::ElementRef) -> Option<TorrentResult> {
        
        // The <a> tag is the parent of div.prewposter, need to go up one level
        let parent = movie_elem.parent()?;
        
        let link_node = parent.parent()?;
        
        let link_elem = scraper::ElementRef::wrap(link_node)?;
        
        let href = link_elem.value().attr("href");
        
        if href.is_none() {
            return None;
        }
        
        // Extract title from image alt text
        let img_elem = movie_elem.select(&Selector::parse("img").unwrap()).next();
        
        if img_elem.is_none() {
            return None;
        }
        
        let img_elem = img_elem.unwrap();
        let text: String = movie_elem.text().collect();
        
        let alt = img_elem.value().attr("alt");
        
        let title = alt
            .unwrap_or(text.trim())
            .to_string();
            
        
        // Create basic torrent result with required fields
        let torrent_result = TorrentResult::new(
            title,
            String::new(), // magnet_link - will be populated from details page
            0,             // size_bytes - will be populated from details page
            0,             // seeders - will be populated from details page
            0,             // leechers - will be populated from details page
            "monna".to_string(),
        );
        
        Some(torrent_result)
    }

    fn extract_year_from_title(&self, _title: &str) -> Option<u32> {
        // Extract year from title - currently unused
        None
    }

    fn parse_movie_details(&self, _url: &str, _document: &Html) -> Option<TorrentResult> {
        // Parse movie details - currently unused
        None
    }

    fn extract_torrent_info(&self, _document: &Html) -> Vec<TorrentResult> {
        // Extract torrent info - currently unused
        Vec::new()
    }
}

#[async_trait]
impl Indexer for MonnaIndexer {
    fn name(&self) -> &str {
        "monna"
    }

    fn description(&self) -> &str {
        "Monna2.top - Russian torrent site for movies and series"
    }

    fn base_url(&self) -> &str {
        &self.base_url
    }

    async fn search(&self, query: &SearchQuery) -> Result<Vec<TorrentResult>> {
        info!("Searching Monna for: {}", query.query);
        
        // Build search URL based on provided pattern
        let search_url = format!("https://tv.monna2.top/index.php?do=search&q={}", 
                                query.query);
        
        debug!("Fetching search URL: {}", search_url);
        let html = self.fetch_page(&search_url).await?;
        info!("Fetched HTML length: {} chars", html.len());
        debug!("HTML preview: {}", &html[..html.len().min(500)]);
        
        // Parse search results - they should have the same structure as main page
        let results = self.parse_movie_list(&html)?;
        info!("Parsed {} results from Monna search", results.len());
        Ok(results)
    }

    async fn get_details(&self, info_hash: &str) -> Result<Option<TorrentResult>> {
        debug!("Getting details for hash: {}", info_hash);
        
        // For now, return None as we need to implement proper hash-based lookup
        // This would require storing mappings from URLs to hashes
        Ok(None)
    }

    async fn get_feed(&self) -> Result<Vec<TorrentResult>> {
        info!("🎬 Fetching movie feed from Monna2 main page with metadata");
        let html = self.fetch_page(&self.base_url).await?;
        info!("📄 Fetched HTML length: {} chars", html.len());
        debug!("📄 HTML preview: {}", &html[..html.len().min(300)]);
        
        // Use the new parallel metadata fetching
        let movies = self.parse_movie_list_with_metadata(&html).await?;
        info!("🎬 Parsed {} raw movies with metadata from Monna2", movies.len());
        
        for (i, movie) in movies.iter().enumerate().take(3) {
            debug!("📽️ Movie {}: '{}' - {}", i, movie.title, movie.magnet_link);
        }
        
        Ok(movies)
    }

    async fn test_connection(&self) -> Result<bool> {
        // Test if the site is accessible
        match self.fetch_page(&self.base_url).await {
            Ok(_) => Ok(true),
            Err(e) => {
                warn!("Monna indexer not accessible: {}", e);
                Ok(false)
            }
        }
    }

    fn config(&self) -> &IndexerConfig {
        &self.config
    }

    async fn update_config(&mut self, config: IndexerConfig) -> Result<()> {
        self.config = config;
        Ok(())
    }

    async fn is_enabled(&self) -> Result<bool> {
        Ok(self.config.enabled)
    }
}

impl MonnaIndexer {
    /// Get feed of recent movies from the main page
    pub async fn get_feed(&self) -> Result<Vec<TorrentResult>> {
        info!("Fetching movie feed from Monna2 main page");
        
        let html = self.fetch_page(&self.base_url).await?;
        let movies = self.parse_movie_list(&html)?;
        
        info!("Successfully fetched {} movies for feed", movies.len());
        Ok(movies)
    }

    async fn fetch_movie_details(&self, movie_url: &str) -> Result<TorrentResult> {
        debug!("Fetching movie details from: {}", movie_url);
        
        let html = self.fetch_page(movie_url).await?;
        let document = Html::parse_document(&html);
        
        // Extract title from h1
        let title_selector = Selector::parse("h1").unwrap();
        let title = document.select(&title_selector)
            .next()
            .map(|elem| {
                let text: String = elem.text().collect();
                text.trim().to_string()
            })
            .unwrap_or_default();
        
        // Extract metadata from main content div
        let content_selector = Selector::parse("#news-id-262").unwrap();
        let mut metadata = std::collections::HashMap::new();
        let mut poster_url = String::new();
        
        if let Some(content_elem) = document.select(&content_selector).next() {
            let content_text: String = content_elem.text().collect();
            
            // Extract poster URL from images in content
            let img_selector = Selector::parse("img").unwrap();
            if let Some(img_elem) = content_elem.select(&img_selector).next() {
                if let Some(src) = img_elem.value().attr("src") {
                    // Check if it's a poster URL (usually from uploads/posts directory)
                    if src.contains("/uploads/posts/") {
                        poster_url = if src.starts_with("http") {
                            src.to_string()
                        } else {
                            format!("https://tv.monna2.top{}", src)
                        };
                        debug!("Found poster URL: {}", poster_url);
                    }
                }
            }
            
            // Parse metadata lines
            for line in content_text.lines() {
                let line = line.trim();
                if line.starts_with("Жанр:") {
                    metadata.insert("genre".to_string(), line.split(':').nth(1).unwrap_or("").trim().to_string());
                } else if line.starts_with("Режиссер:") {
                    metadata.insert("director".to_string(), line.split(':').nth(1).unwrap_or("").trim().to_string());
                } else if line.starts_with("В ролях:") {
                    metadata.insert("cast".to_string(), line.split(':').nth(1).unwrap_or("").trim().to_string());
                } else if line.starts_with("О фильме:") || line.starts_with("О Сериале:") {
                    metadata.insert("description".to_string(), line.split(':').nth(1).unwrap_or("").trim().to_string());
                } else if line.starts_with("Продолжительность:") {
                    metadata.insert("duration".to_string(), line.split(':').nth(1).unwrap_or("").trim().to_string());
                } else if line.starts_with("Год выхода:") {
                    metadata.insert("year".to_string(), line.split(':').nth(1).unwrap_or("").trim().to_string());
                } else if line.starts_with("Страна:") {
                    metadata.insert("country".to_string(), line.split(':').nth(1).unwrap_or("").trim().to_string());
                } else if line.starts_with("Студия:") {
                    metadata.insert("studio".to_string(), line.split(':').nth(1).unwrap_or("").trim().to_string());
                }
            }
        }
        
        // Look for external torrent script first
        let mut magnet_link = String::new();
        let mut size_bytes = 0u64;
        
        // Find the external script that loads torrent data
        let script_selector = Selector::parse("script[src*='torrents.ll33.top']").unwrap();
        if let Some(script_elem) = document.select(&script_selector).next() {
            if let Some(script_url) = script_elem.value().attr("src") {
                debug!("Found external torrent script: {}", script_url);
                
                // Fetch the external script to get torrent data
                if let Ok(script_html) = self.fetch_page(script_url).await {
                    let script_doc = Html::parse_document(&script_html);
                    
                    // Parse the torrent table from the script response
                    let torrent_row_selector = Selector::parse("table#tor-tbl tbody tr").unwrap();
                    for torrent_row in script_doc.select(&torrent_row_selector) {
                        // Extract magnet link (2nd column)
                        let magnet_selector = Selector::parse("td:nth-child(2) a[href*='magnet:']").unwrap();
                        if let Some(magnet_elem) = torrent_row.select(&magnet_selector).next() {
                            if let Some(href) = magnet_elem.value().attr("href") {
                                magnet_link = href.to_string();
                                debug!("Found magnet link from external script");
                                
                                // Extract size (3rd column) - format: "2203801600 2.06 GB"
                                let size_selector = Selector::parse("td:nth-child(3)").unwrap();
                                if let Some(size_elem) = torrent_row.select(&size_selector).next() {
                                    let size_text: String = size_elem.text().collect();
                                    // Extract the GB part from "2203801600 2.06 GB"
                                    if let Some(size_part) = size_text.split_whitespace().nth(1) {
                                        size_bytes = self.extract_size_bytes_from_text(size_part);
                                    }
                                }
                                
                                // Extract seeders (5th column)
                                let seeders_selector = Selector::parse("td:nth-child(5) b").unwrap();
                                let seeders = torrent_row.select(&seeders_selector)
                                    .next()
                                    .and_then(|elem| elem.text().collect::<String>().trim().parse::<u32>().ok())
                                    .unwrap_or(0);
                                
                                // Extract leechers (6th column)
                                let leechers_selector = Selector::parse("td:nth-child(6) b").unwrap();
                                let leechers = torrent_row.select(&leechers_selector)
                                    .next()
                                    .and_then(|elem| elem.text().collect::<String>().trim().parse::<u32>().ok())
                                    .unwrap_or(0);
                                
                                // Extract quality from title (4th column) - currently unused
                                let _title_selector = Selector::parse("td:nth-child(4) b").unwrap();
                                let _quality = torrent_row.select(&_title_selector)
                                    .next()
                                    .and_then(|elem| {
                                        let title_text: String = elem.text().collect();
                                        self.extract_quality_from_text(&title_text)
                                    });
                                
                                // Create TorrentResult with all available data
                                let mut torrent_result = TorrentResult::new(
                                    title,
                                    magnet_link,
                                    size_bytes,
                                    seeders,
                                    leechers,
                                    "monna".to_string(),
                                );
                                
                                // Add metadata to torrent result if available
                                if let Some(genre) = metadata.get("genre") {
                                    torrent_result.category = Some(genre.clone());
                                    // Split the genre string and filter valid genres
                                    let raw_genres: Vec<String> = genre.split(',') 
                                        .map(|g| g.trim().to_string())
                                        .filter(|g| !g.is_empty())
                                        .collect();
                                    torrent_result.genres = filter_valid_genres(raw_genres);
                                }
                                
                                // Add poster URL if found
                                if !poster_url.is_empty() {
                                    torrent_result.poster_url = Some(poster_url);
                                }
                                
                                return Ok(torrent_result);
                            }
                        }
                    }
                }
            }
        }
        
        // Fallback: look for direct magnet/torrent links on the movie page
        if magnet_link.is_empty() {
            let torrent_selectors = [
                "a[href*='magnet']",
                "a[href*='.torrent']", 
                "a[href*='download']",
                ".torrent-link a",
                ".download a"
            ];
            
            for selector in torrent_selectors {
                if let Ok(sel) = Selector::parse(selector) {
                    for link_elem in document.select(&sel) {
                        if let Some(href) = link_elem.value().attr("href") {
                            let text: String = link_elem.text().collect();
                            let link_text = text.trim();
                            
                            // Check for magnet links
                            if href.starts_with("magnet:") && magnet_link.is_empty() {
                                magnet_link = href.to_string();
                            }
                            
                            // Extract size from link text
                            if size_bytes == 0 {
                                size_bytes = self.extract_size_bytes_from_text(link_text);
                            }
                        }
                    }
                }
            }
        }
        
        // Look for download containers as final fallback
        if magnet_link.is_empty() {
            let download_selectors = [
                ".download", ".torrent", ".player", ".watch", ".links"
            ];
            
            for selector in download_selectors {
                if let Ok(sel) = Selector::parse(selector) {
                    if let Some(container) = document.select(&sel).next() {
                        for link_elem in container.select(&Selector::parse("a").unwrap()) {
                            if let Some(href) = link_elem.value().attr("href") {
                                let text: String = link_elem.text().collect();
                                let link_text = text.trim();
                                
                                if href.starts_with("magnet:") && magnet_link.is_empty() {
                                    magnet_link = href.to_string();
                                }
                                
                                if size_bytes == 0 {
                                    size_bytes = self.extract_size_bytes_from_text(link_text);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(TorrentResult::new(
            title,
            magnet_link,
            size_bytes,
            0, // seeders - not available on this site
            0, // leechers - not available on this site  
            "unknown".to_string(),
        ))
    }

    fn extract_size_bytes_from_text(&self, text: &str) -> u64 {
        use regex::Regex;
        let re = match Regex::new(r"(?i)(\d+(?:\.\d+)?)\s*(GB|MB|KB)") {
            Ok(re) => re,
            Err(_) => return 0,
        };
        
        if let Some(m) = re.find(text) {
            let parts: Vec<&str> = m.as_str().split_whitespace().collect();
            if parts.len() == 2 {
                if let Ok(size) = parts[0].parse::<f64>() {
                    return match parts[1].to_uppercase().as_str() {
                        "GB" => (size * 1024.0 * 1024.0 * 1024.0) as u64,
                        "MB" => (size * 1024.0 * 1024.0) as u64,
                        "KB" => (size * 1024.0) as u64,
                        _ => 0,
                    };
                }
            }
        }
        0
    }

    fn extract_quality_from_text(&self, text: &str) -> Option<String> {
        let quality_regex = regex::Regex::new(r"(?i)(4K|2160p|Ultra\s+HD|1080p|Full\s+HD|720p|HD|480p|SD|CAM|TS|TC|WEB-DLRip|WEB-DL|BDRip|DVDRip)").ok()?;
        
        if let Some(caps) = quality_regex.captures(text) {
            caps.get(1).map(|m| m.as_str().to_string())
        } else {
            None
        }
    }
}

impl Default for MonnaIndexer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_metadata_with_missing_fields() {
        // Test that parsing HTML with missing fields returns None for those fields
        // without panicking or causing errors
        let html = r#"
            <h1>Test Movie</h1>
            <div class="fullstory">
                <p>Some content without any metadata fields</p>
            </div>
        "#;
        
        let result = MonnaIndexer::parse_metadata_from_html(html, "https://test.com");
        assert!(result.is_ok());
        
        let metadata = result.unwrap();
        assert_eq!(metadata.title, "Test Movie");
        assert_eq!(metadata.year, None);
        assert_eq!(metadata.director, None);
        assert_eq!(metadata.country, None);
        assert_eq!(metadata.studio, None);
        assert_eq!(metadata.translation, None);
        assert_eq!(metadata.quality, None);
        assert_eq!(metadata.video_codec, None);
        assert_eq!(metadata.audio_format, None);
        assert_eq!(metadata.subtitles, None);
    }

    #[test]
    fn test_parse_metadata_with_malformed_html() {
        // Test that parsing malformed HTML doesn't panic
        let html = r#"
            <h1>Test Movie
            <div class="fullstory">
                <p>Год выпуска: not_a_year</p>
                <p>Продолжительность: invalid:time:format</p>
            </div>
        "#;
        
        let result = MonnaIndexer::parse_metadata_from_html(html, "https://test.com");
        assert!(result.is_ok());
        
        let metadata = result.unwrap();
        // Year parsing should fail gracefully
        assert_eq!(metadata.year, None);
        // Runtime parsing should fail gracefully
        assert_eq!(metadata.runtime_minutes, None);
    }

    #[test]
    fn test_parse_metadata_continues_after_field_failure() {
        // Test that if one field fails to parse, other fields are still extracted
        let html = r#"
            <h1>Test Movie</h1>
            <div class="fullstory">
                <p>Год выпуска: invalid_year</p>
                <p>Страна: Russia</p>
                <p>Студия: Test Studio</p>
            </div>
        "#;
        
        let result = MonnaIndexer::parse_metadata_from_html(html, "https://test.com");
        assert!(result.is_ok());
        
        let metadata = result.unwrap();
        // Year should fail to parse
        assert_eq!(metadata.year, None);
        // But country and studio should still be extracted
        assert_eq!(metadata.country, Some("Russia".to_string()));
        assert_eq!(metadata.studio, Some("Test Studio".to_string()));
    }

    #[test]
    fn test_parse_metadata_with_empty_html() {
        // Test that parsing empty HTML returns a valid but empty metadata structure
        let html = "";
        
        let result = MonnaIndexer::parse_metadata_from_html(html, "https://test.com");
        assert!(result.is_ok());
        
        let metadata = result.unwrap();
        assert_eq!(metadata.title, "");
        assert_eq!(metadata.year, None);
        assert_eq!(metadata.genres.len(), 0);
    }

    #[test]
    fn test_whitespace_normalization() {
        // Test that patterns handle extra whitespace (Requirement 6.3)
        let html = r#"
            <h1>Test Movie</h1>
            <div class="fullstory">
                <p>Год   выпуска  :   2023</p>
                <p>Страна  :  Russia</p>
                <p>Режиссёр   :   John Doe</p>
            </div>
        "#;
        
        let result = MonnaIndexer::parse_metadata_from_html(html, "https://test.com");
        assert!(result.is_ok());
        
        let metadata = result.unwrap();
        assert_eq!(metadata.year, Some(2023));
        assert_eq!(metadata.country, Some("Russia".to_string()));
        assert_eq!(metadata.director, Some("John Doe".to_string()));
    }

    #[test]
    fn test_flexible_punctuation() {
        // Test that patterns handle missing colons (Requirement 6.4)
        let html = r#"
            <h1>Test Movie</h1>
            <div class="fullstory">
                <p>Год выпуска 2023</p>
                <p>Страна Russia</p>
                <p>Перевод Профессиональный</p>
            </div>
        "#;
        
        let result = MonnaIndexer::parse_metadata_from_html(html, "https://test.com");
        assert!(result.is_ok());
        
        let metadata = result.unwrap();
        assert_eq!(metadata.year, Some(2023));
        assert_eq!(metadata.country, Some("Russia".to_string()));
        assert_eq!(metadata.translation, Some("Профессиональный".to_string()));
    }

    // Integration test fixtures from Selectors.md
    // These fixtures contain real HTML examples extracted from the documentation
    
    /// Test fixture for Example 1 from Selectors.md
    /// Tests: "Год выпуска:", "Выпущено:", "Перевод: оригинал"
    fn get_example_1_html() -> &'static str {
        r#"
            <h1>Полярный-17</h1>
            <div class="fullstory">
                <p>Год выпуска: 2019</p>
                <p>Жанр: Комедия</p>
                <p>Выпущено: Россия, ТНТ</p>
                <p>Режиссер: Дмитрий Малина, Павел Кривец, Эльдар Джафаргулиев</p>
                <p>В ролях: Михаил Пореченков, Иван Охлобыстин, Владимир Епифанцев, Катерина Шпица, Игорь Жижикин, Ян Цапник, Филипп Дьячков (II), Светлана Листова, Карина Зверева, Виктор Бычков, Александр Баширов, Сергей Чирков, Вера Строкова, Игорь Филиппов, Юрий Тузов, Александр Потапов (III), Юрий Назаров, Антон Богданов, Сергей Черданцев, Александр Мосолов</p>
                <p>Комедийный детективный сериал «Полярный-17» поведает о жизни бывшего криминального элемента Вити-Мясника, который решил оставить темное бандитское прошлое и заняться легальным бизнесом. Мужчина обладает невероятным талантом по ведению предпринимательской деятельности, в которой быстро преуспевает и чувствует себя в бизнесе как рыба в воде. Ему нравится спокойная размеренная жизнь, новая деятельность и честный заработок.</p>
                <p>Качество: SATRip</p>
                <p>Видео: XviD, ~ 1850 Кбит/с, 720x352</p>
                <p>Аудио: MP3, 2 ch, 192 Кбит/с</p>
                <p>Продолжительность: 2 x ~ 00:25:00</p>
                <p>Перевод: оригинал</p>
            </div>
        "#
    }

    /// Expected metadata for Example 1
    fn get_example_1_expected() -> MonnaMetadata {
        MonnaMetadata {
            title: "Полярный-17".to_string(),
            original_title: None,
            year: Some(2019),
            genres: vec!["Комедия".to_string()],
            director: Some("Дмитрий Малина, Павел Кривец, Эльдар Джафаргулиев".to_string()),
            cast: vec![
                "Михаил Пореченков".to_string(),
                "Иван Охлобыстин".to_string(),
                "Владимир Епифанцев".to_string(),
                "Катерина Шпица".to_string(),
                "Игорь Жижикин".to_string(),
                "Ян Цапник".to_string(),
                "Филипп Дьячков (Ii)".to_string(),
                "Светлана Листова".to_string(),
                "Карина Зверева".to_string(),
                "Виктор Бычков".to_string(),
            ],
            runtime_minutes: Some(50), // 2 x 25 minutes
            description: Some("Комедийный детективный сериал «Полярный-17» поведает о жизни бывшего криминального элемента Вити-Мясника, который решил оставить темное бандитское прошлое и заняться легальным бизнесом. Мужчина обладает невероятным талантом по ведению предпринимательской деятельности, в которой быстро преуспевает и чувствует себя в бизнесе как рыба в воде. Ему нравится спокойная размеренная жизнь, новая деятельность и честный заработок.".to_string()),
            poster_url: None,
            kinopoisk_id: None,
            kinopoisk_url: None,
            is_series: false,
            country: Some("Россия, ТНТ".to_string()),
            studio: Some("Россия, ТНТ".to_string()),
            translation: Some("оригинал".to_string()),
            quality: Some("SATRip".to_string()),
            video_codec: Some("XviD, ~ 1850 Кбит/с, 720x352".to_string()),
            audio_format: Some("MP3, 2 ch, 192 Кбит/с".to_string()),
            subtitles: None,
        }
    }

    /// Test fixture for Example 2 from Selectors.md
    /// Tests: "Год выхода:", "Страна:", "Студия:", "Перевод: Не требуется"
    fn get_example_2_html() -> &'static str {
        r#"
            <h1>Test Movie 2</h1>
            <div class="fullstory">
                <p>Год выхода: 2025</p>
                <p>Жанр: Детектив, драма, полицейский, остросюжетный</p>
                <p>Режиссер: Андрей Голубев</p>
                <p>В ролях: Сергей Чачин, Сергей Ионкин, Евгений Рыжик, Денис Старков, Илья Тиунов, Анастасия Нечаева (III), Оксана Сырцова, Борис Бедросов, Артур Харитоненко, Владислав Бургард</p>
                <p>О фильме: В семье скромного преподавателя строительного колледжа происходит невероятный и абсурдно-комический поворот: тихую, размеренную жизнь разрывает напополам появление модной светской красавицы — избалованной, блестящей, роскошной, привыкшей к вспышкам камер и восхищённым взглядам.</p>
                <p>Страна: Россия</p>
                <p>Студия: Пятый канал, Кинокомпания "Триикс Медиа"</p>
                <p>Продолжительность: 10 х 00:42:00</p>
                <p>Перевод: Не требуется</p>
                <p>Файл</p>
                <p>Кодек: h.264</p>
                <p>Качество: HDTV 1080p</p>
                <p>Видео: MPEG-4 AVC, ~ 6576 Кбит/с, 1920x1080, 25 кадр/с</p>
                <p>Звук: AC3, 2 ch, 192 Кбит/с</p>
            </div>
        "#
    }

    /// Expected metadata for Example 2
    fn get_example_2_expected() -> MonnaMetadata {
        MonnaMetadata {
            title: "Test Movie 2".to_string(),
            original_title: None,
            year: Some(2025),
            genres: vec!["Детектив".to_string(), "Драма".to_string(), "Полицейский".to_string(), "Остросюжетный".to_string()],
            director: Some("Андрей Голубев".to_string()),
            cast: vec![
                "Сергей Чачин".to_string(),
                "Сергей Ионкин".to_string(),
                "Евгений Рыжик".to_string(),
                "Денис Старков".to_string(),
                "Илья Тиунов".to_string(),
                "Анастасия Нечаева (Iii)".to_string(),
                "Оксана Сырцова".to_string(),
                "Борис Бедросов".to_string(),
                "Артур Харитоненко".to_string(),
                "Владислав Бургард".to_string(),
            ],
            runtime_minutes: Some(420), // 10 x 42 minutes
            description: Some("В семье скромного преподавателя строительного колледжа происходит невероятный и абсурдно-комический поворот: тихую, размеренную жизнь разрывает напополам появление модной светской красавицы — избалованной, блестящей, роскошной, привыкшей к вспышкам камер и восхищённым взглядам.".to_string()),
            poster_url: None,
            kinopoisk_id: None,
            kinopoisk_url: None,
            is_series: false,
            country: Some("Россия".to_string()),
            studio: Some("Пятый канал, Кинокомпания \"Триикс Медиа\"".to_string()),
            translation: Some("Не требуется".to_string()),
            quality: Some("HDTV 1080p".to_string()),
            video_codec: Some("MPEG-4 AVC, ~ 6576 Кбит/с, 1920x1080, 25 кадр/с".to_string()),
            audio_format: Some("AC3, 2 ch, 192 Кбит/с".to_string()),
            subtitles: None,
        }
    }

    /// Test fixture for Example 3 from Selectors.md
    /// Tests: "Год выпуска:", "Страна:", "Перевод: Профессиональный"
    fn get_example_3_html() -> &'static str {
        r#"
            <h1>Test Movie 3</h1>
            <div class="fullstory">
                <p>Год выпуска: 2025</p>
                <p>Страна: Корея Южная, Франция</p>
                <p>Жанр: триллер, комедия, драма, криминал</p>
                <p>Перевод: Профессиональный (многоголосый, закадровый) - ВПОДПОЛЬЕ - по заказу SCARFILM</p>
                <p>Продолжительность: 02:16:34</p>
                <p>Субтитры: русские (полные) + оригинал (полные)</p>
                <p>Режиссер: Пак Чхан-ук</p>
                <p>В ролях: Ли Бён-хон, Сон Е-джин, Пак Хи-сун, Ли Сон-мин, Ём Хе-ран, Чха Сын-вон, Ён Сок Ю, Юн Га-и, Даль Су О, Ли Сок-хён</p>
                <p>Описание: Четверть века проработав в бумажной промышленности, Ю Ман-су по праву считался мастером своего дела. Он прошёл путь от рядового сотрудника до специалиста, которому доверяли самые сложные процессы производства. Его уважали коллеги, к нему прислушивались начальники, и сам Ман-су чувствовал себя человеком, который твёрдо стоит на ногах.</p>
                <p>Качество: WEB-DL 1080p</p>
                <p>Формат: MKV</p>
                <p>Видео: 1920x1080, 23.976 fps, AVC, ~7000 kbps avg</p>
                <p>Аудио №1: AC3, 2.0 ch, 320 Kbps</p>
                <p>Аудио №2: AAС, 2.0 ch, 192 Kbps - оригинал</p>
            </div>
        "#
    }

    /// Expected metadata for Example 3
    fn get_example_3_expected() -> MonnaMetadata {
        MonnaMetadata {
            title: "Test Movie 3".to_string(),
            original_title: None,
            year: Some(2025),
            genres: vec!["Триллер".to_string(), "Комедия".to_string(), "Драма".to_string(), "Криминал".to_string()],
            director: Some("Пак Чхан-ук".to_string()),
            cast: vec![
                "Ли Бён-Хон".to_string(),
                "Сон Е-Джин".to_string(),
                "Пак Хи-Сун".to_string(),
                "Ли Сон-Мин".to_string(),
                "Ём Хе-Ран".to_string(),
                "Чха Сын-Вон".to_string(),
                "Ён Сок Ю".to_string(),
                "Юн Га-И".to_string(),
                "Даль Су О".to_string(),
                "Ли Сок-Хён".to_string(),
            ],
            runtime_minutes: Some(136), // 2:16:34 = 136 minutes
            description: Some("Четверть века проработав в бумажной промышленности, Ю Ман-су по праву считался мастером своего дела. Он прошёл путь от рядового сотрудника до специалиста, которому доверяли самые сложные процессы производства. Его уважали коллеги, к нему прислушивались начальники, и сам Ман-су чувствовал себя человеком, который твёрдо стоит на ногах.".to_string()),
            poster_url: None,
            kinopoisk_id: None,
            kinopoisk_url: None,
            is_series: false,
            country: Some("Корея Южная, Франция".to_string()),
            studio: None,
            translation: Some("Профессиональный (многоголосый, закадровый) - ВПОДПОЛЬЕ - по заказу SCARFILM".to_string()),
            quality: Some("WEB-DL 1080p".to_string()),
            video_codec: Some("1920x1080, 23.976 fps, AVC, ~7000 kbps avg".to_string()),
            audio_format: Some("AC3, 2.0 ch, 320 Kbps".to_string()),
            subtitles: Some("русские (полные) + оригинал (полные)".to_string()),
        }
    }

    /// Test fixture for Example 7 from Selectors.md
    /// Tests: "ГОД:", "СТРАНА:", "ПЕРЕВОД:", "РЕЖИССЕРСКИЙ СОСТАВ:" (all caps)
    fn get_example_7_html() -> &'static str {
        r#"
            <h1>TEST MOVIE 7</h1>
            <div class="fullstory">
                <p>ГОД: 2025</p>
                <p>СТРАНА: РОССИЯ</p>
                <p>ПЕРЕВОД: ОРИГИНАЛ</p>
                <p>ЖАНР: ФЭНТЕЗИ, КОМЕДИЯ, СЕМЕЙНЫЙ</p>
                <p>КАЧЕСТВО: TELECINE</p>
                <p>РЕЖИССЕРСКИЙ СОСТАВ: АЛЕКСАНДР ВОЙТИНСКИЙ</p>
                <p>В РОЛЯХ: СВЕТЛАНА ХОДЧЕНКОВА, ЮРИЙ КОЛОКОЛЬНИКОВ, АЛЕКСАНДР САМУСЕВ, ДЕНИС ПРЫТКОВ, ВЕРОНИКА ТИМОФЕЕВА, МАКСИМ ЛАГАШКИН, ФЕОДОР КИРСАНОВ, ВИОЛЕТТА АНТОНОВА, ИРИНА РОМАНОВА, ДМИТРИЙ КОНДРАТКОВ</p>
                <p>ВРЕМЯ: 1 Ч 24 МИН</p>
                <p>Когда у десятилетнего Пети появляется новая воспитательница, он сразу чувствует: в ней есть что-то необычное. Она кажется странной — немного ворчливая, с колючими замечаниями и привычкой смотреть исподлобья. Но в то же время — невероятно обаятельная. В её голосе слышится та самая теплая, чуть насмешливая интонация, которая сразу располагает к себе, а в глазах — искорки хитрости и древней мудрости.</p>
                <p>Формат: MKV</p>
                <p>Видео: 7000 Кбит/с, 1920x800</p>
                <p>Аудио: AC3, 384 kb/s (2 ch)</p>
            </div>
        "#
    }

    /// Expected metadata for Example 7
    fn get_example_7_expected() -> MonnaMetadata {
        MonnaMetadata {
            title: "Test Movie 7".to_string(), // Should be converted to title case
            original_title: None,
            year: Some(2025),
            genres: vec!["Фэнтези".to_string(), "Комедия".to_string(), "Семейный".to_string()],
            director: Some("Александр Войтинский".to_string()),
            cast: vec![
                "Светлана Ходченкова".to_string(),
                "Юрий Колокольников".to_string(),
                "Александр Самусев".to_string(),
                "Денис Прытков".to_string(),
                "Вероника Тимофеева".to_string(),
                "Максим Лагашкин".to_string(),
                "Феодор Кирсанов".to_string(),
                "Виолетта Антонова".to_string(),
                "Ирина Романова".to_string(),
                "Дмитрий Кондратков".to_string(),
            ],
            runtime_minutes: Some(84), // 1 hour 24 minutes
            description: Some("Когда у десятилетнего Пети появляется новая воспитательница, он сразу чувствует: в ней есть что-то необычное. Она кажется странной — немного ворчливая, с колючими замечаниями и привычкой смотреть исподлобья. Но в то же время — невероятно обаятельная. В её голосе слышится та самая теплая, чуть насмешливая интонация, которая сразу располагает к себе, а в глазах — искорки хитрости и древней мудрости.".to_string()),
            poster_url: None,
            kinopoisk_id: None,
            kinopoisk_url: None,
            is_series: false,
            country: Some("Россия".to_string()),
            studio: None,
            translation: Some("Оригинал".to_string()),
            quality: Some("Telecine".to_string()),
            video_codec: Some("7000 Кбит/с, 1920x800".to_string()),
            audio_format: Some("AC3, 384 kb/s (2 ch)".to_string()),
            subtitles: None,
        }
    }
}
