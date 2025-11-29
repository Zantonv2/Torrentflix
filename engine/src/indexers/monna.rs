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
static ORIGINAL_TITLE_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"(?i)Оригинальное название:\s*(.+?)(?:\n|Год)").unwrap()
});
static YEAR_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"(?i)Год (?:выхода|выпуска):\s*(\d{4})").unwrap()
});
static GENRE_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"(?i)Жанр:\s*([^\n<]+?)(?:<br>|Режиссёр|Режиссер|Время|$)").unwrap()
});
static DIRECTOR_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"(?i)Режиссёр:\s*(.+?)(?:\n|В ролях|Время|$)").unwrap()
});
static CAST_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"(?i)(?:В ролях|Актеры):\s*(.+?)(?:\n\n|Описание|О фильме|Время|Формат|$)").unwrap()
});
// Support both "Продолжительность:" and "ВРЕМЯ:" formats
static RUNTIME_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"(?i)(?:Продолжительность|Время):\s*(\d+):(\d+):(\d+)").unwrap()
});
// More flexible description regex - looks for text after cast or other markers
static DESCRIPTION_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"(?i)(?:О фильме|Описание):?\s*(.+?)(?:Продолжительность|Время|Файл|Скачать|Формат|$)").unwrap()
});
// Alternative: extract description from paragraph text after cast section
static DESCRIPTION_FALLBACK_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"(?i)(?:В ролях|Актеры):\s*.+?\n\n(.+?)(?:Время|Продолжительность|Файл|Формат|Скачать|$)").unwrap()
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
}

#[derive(Debug, Clone)]
pub struct MonnaIndexer {
    client: Client,
    base_url: String,
    config: IndexerConfig,
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
        eprintln!("🔍 MonnaIndexer: parse_movie_list() called with {} chars", html.len());
        
        // This function needs to be async for metadata fetching
        // We'll handle this in the calling function
        Ok(vec![])
    }

    /// Extract detail URLs from main page HTML (synchronous, no Send issues)
    fn extract_detail_urls(html: &str, base_url: &str) -> Vec<(String, &'static str)> {
        eprintln!("🔍 MonnaIndexer: extract_detail_urls() called with {} chars", html.len());
        let document = Html::parse_document(html);

        // Use correct selectors from Python implementation
        // Find main content area first (like #dle-content)
        let main_content = document.select(&DLE_CONTENT_SELECTOR)
            .next()
            .unwrap_or_else(|| document.root_element());
        
        // Find all article links (correct approach from Python)
        let article_links: Vec<_> = main_content.select(&ARTICLE_LINK_SELECTOR).collect();
        
        eprintln!("🎯 MonnaIndexer: Found {} article links with selector 'a[href*=\".html\"]'", article_links.len());
        
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
                if category.is_none() {
                    continue;
                }
                
                // Skip non-content pages
                if full_url.contains("xfsearch") || full_url.contains("page/") || full_url.contains("disklaimer") {
                    continue;
                }
                
                eprintln!("🔄 MonnaIndexer: Found valid link: {} (category: {:?})", full_url, category);
                detail_urls.push((full_url, category.unwrap()));
            }
        }
        
        eprintln!("🎉 MonnaIndexer: Extracted {} detail URLs from main page", detail_urls.len());
        detail_urls
    }

    /// Parse movie list and fetch metadata in parallel (async version)
    async fn parse_movie_list_with_metadata(&self, html: &str) -> Result<Vec<TorrentResult>> {
        eprintln!("🔍 MonnaIndexer: parse_movie_list_with_metadata() called with {} chars", html.len());
        
        // Extract URLs synchronously (no Send issues)
        let detail_urls = Self::extract_detail_urls(html, &self.base_url);
        
        // All HTML parsing is complete, now we can safely use await points
        eprintln!("📊 MonnaIndexer: Fetching {} detail pages in parallel", detail_urls.len());
        
        // Create semaphore to limit concurrent requests (like Python implementation)
        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(10));
        let mut tasks = Vec::new();
        
        for (url, category) in detail_urls {
            let semaphore = semaphore.clone();
            let client = self.client.clone();
            let base_url = self.base_url.clone();
            
            let task = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                
                eprintln!("🔄 MonnaIndexer: Fetching HTML from: {}", url);
                
                // Only fetch HTML in parallel (no parsing, no Send issues)
                match Self::fetch_page_html(&client, &base_url, &url).await {
                    Ok(html) => {
                        eprintln!("✅ MonnaIndexer: Got HTML from: {} ({} chars)", url, html.len());
                        Some((url, category, html))
                    }
                    Err(e) => {
                        eprintln!("⚠️ MonnaIndexer: Failed to fetch HTML from {}: {}", url, e);
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
                        eprintln!("✅ MonnaIndexer: Parsed metadata for: {} (poster: {:?})", metadata.title, metadata.poster_url.is_some());
                        
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
                        torrent_result.category = Some(category.to_string());
                        torrent_result.poster_url = metadata.poster_url.clone();
                        torrent_result.description = metadata.description.clone();
                        torrent_result.cast = metadata.cast.clone();
                        torrent_result.runtime_minutes = metadata.runtime_minutes;
                        torrent_result.genres = metadata.genres.clone();
                        
                        if let Some(poster_url) = torrent_result.poster_url.as_ref() {
                            eprintln!("🖼️ MonnaIndexer: Added poster URL: {}", poster_url);
                        }
                        if let Some(description) = torrent_result.description.as_ref() {
                            eprintln!("📝 MonnaIndexer: Added description: {} chars", description.len());
                        }
                        if !torrent_result.cast.is_empty() {
                            eprintln!("👥 MonnaIndexer: Added {} cast members", torrent_result.cast.len());
                        }
                        if let Some(runtime) = torrent_result.runtime_minutes {
                            eprintln!("⏱️ MonnaIndexer: Added runtime: {} minutes", runtime);
                        }
                        
                        movies.push(torrent_result);
                    }
                    Err(e) => {
                        eprintln!("⚠️ MonnaIndexer: Failed to parse metadata from {}: {}", url, e);
                    }
                }
            }
        }
        
        eprintln!("🎉 MonnaIndexer: Successfully parsed {} movies with metadata", movies.len());
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
            metadata.title = title.replace("скачать торрент", "").trim().to_string();
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
                    let genres_str = genre_match.get(1).unwrap().as_str().trim();
                    genres_from_dom = genres_str
                        .split(',')
                        .map(|g| g.trim().to_string())
                        .filter(|g| !g.is_empty())
                        .collect();
                }
            }
            
            // Extract original title
            if let Some(original_match) = ORIGINAL_TITLE_REGEX.captures(&text) {
                metadata.original_title = Some(original_match.get(1).unwrap().as_str().trim().to_string());
            }
            
            // Extract year
            if let Some(year_match) = YEAR_REGEX.captures(&text) {
                metadata.year = Some(year_match.get(1).unwrap().as_str().parse().unwrap_or(2024));
            }
            
            // Extract genres with filtering (genres_from_dom may have been populated by DOM or regex fallback)
            if !genres_from_dom.is_empty() {
                metadata.genres = filter_valid_genres(genres_from_dom);
            } else {
                // Last resort: try regex directly on text
                if let Some(genre_match) = GENRE_REGEX.captures(&text) {
                    let genres_str = genre_match.get(1).unwrap().as_str().trim();
                    let raw_genres: Vec<String> = genres_str.split(',').map(|g| g.trim().to_string()).collect();
                    metadata.genres = filter_valid_genres(raw_genres);
                }
            }
            
            // Extract director
            if let Some(director_match) = DIRECTOR_REGEX.captures(&text) {
                metadata.director = Some(director_match.get(1).unwrap().as_str().trim().to_string());
            }
            
            // Extract cast
            if let Some(cast_match) = CAST_REGEX.captures(&text) {
                let cast_str = cast_match.get(1).unwrap().as_str().trim();
                metadata.cast = cast_str.split(',').map(|c| c.trim().to_string()).take(10).collect();
            }
            
            // Extract runtime (supports both "Продолжительность:" and "ВРЕМЯ:")
            if let Some(runtime_match) = RUNTIME_REGEX.captures(&text) {
                if let (Ok(hours), Ok(minutes)) = (
                    runtime_match.get(1).unwrap().as_str().parse::<u32>(),
                    runtime_match.get(2).unwrap().as_str().parse::<u32>()
                ) {
                    metadata.runtime_minutes = Some(hours * 60 + minutes);
                }
            }
            
            // Extract description - try main regex first, then fallback
            if let Some(desc_match) = DESCRIPTION_REGEX.captures(&text) {
                let mut description = desc_match.get(1).unwrap().as_str().trim().to_string();
                description = WHITESPACE_REGEX.replace_all(&description, " ").to_string();
                description = BR_REGEX.replace_all(&description, " ").to_string();
                if !description.is_empty() && description.len() > 20 {
                    metadata.description = Some(description);
                }
            } else if let Some(desc_match) = DESCRIPTION_FALLBACK_REGEX.captures(&text) {
                // Fallback: extract text after cast section
                let mut description = desc_match.get(1).unwrap().as_str().trim().to_string();
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
                        // Filter out very short paragraphs and common non-description text
                        p.len() > 50 && 
                        !p.to_lowercase().contains("скачать") &&
                        !p.to_lowercase().contains("торрент") &&
                        !p.to_lowercase().starts_with("формат:") &&
                        !p.to_lowercase().starts_with("видео:") &&
                        !p.to_lowercase().starts_with("аудио:") &&
                        !TIME_PATTERN_REGEX.is_match(p) // Filter out paragraphs that are just time
                    })
                    .collect();
                
                if !paragraphs.is_empty() {
                    // Join paragraphs and use as description
                    let combined = paragraphs.join(" ");
                    // Clean up the combined text - remove any leading time patterns
                    let mut final_desc = TIME_PATTERN_REGEX.replace(&combined, "").to_string();
                    final_desc = final_desc.trim().to_string();
                    if final_desc.len() > 50 {
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
                        // Skip if it starts with technical info or is too short
                        let cleaned_lower = cleaned.to_lowercase();
                        if !cleaned_lower.starts_with("формат:") && 
                           !cleaned_lower.starts_with("скачать") &&
                           !cleaned_lower.starts_with("видео:") &&
                           !cleaned_lower.starts_with("аудио:") &&
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
        eprintln!("🔍 MonnaIndexer: Fetching metadata from: {}", detail_url);
        
        let html = self.fetch_page(detail_url).await?;
        let document = Html::parse_document(&html);
        
        let mut metadata = MonnaMetadata::default();
        
        // Extract title from h1
        if let Some(h1) = document.select(&H1_SELECTOR).next() {
            let title = h1.text().collect::<String>().trim().to_string();
            // Remove "скачать торрент" suffix if present
            metadata.title = title.replace("скачать торрент", "").trim().to_string();
        }
        
        // Extract metadata from fullstory div
        if let Some(fullstory) = document.select(&FULLSTORY_SELECTOR).next() {
            let text = fullstory.text().collect::<String>();
            
            // Extract original title
            if let Some(original_match) = ORIGINAL_TITLE_REGEX.captures(&text) {
                metadata.original_title = Some(original_match.get(1).unwrap().as_str().trim().to_string());
            }
            
            // Extract year
            if let Some(year_match) = YEAR_REGEX.captures(&text) {
                metadata.year = Some(year_match.get(1).unwrap().as_str().parse().unwrap_or(2024));
            }
            
            // Extract genres with filtering
            if let Some(genre_match) = GENRE_REGEX.captures(&text) {
                let genres_str = genre_match.get(1).unwrap().as_str().trim();
                let raw_genres: Vec<String> = genres_str.split(',').map(|g| g.trim().to_string()).collect();
                metadata.genres = filter_valid_genres(raw_genres);
            }
            
            // Extract director
            if let Some(director_match) = DIRECTOR_REGEX.captures(&text) {
                metadata.director = Some(director_match.get(1).unwrap().as_str().trim().to_string());
            }
            
            // Extract cast
            if let Some(cast_match) = CAST_REGEX.captures(&text) {
                let cast_str = cast_match.get(1).unwrap().as_str().trim();
                metadata.cast = cast_str.split(',').map(|c| c.trim().to_string()).take(10).collect();
            }
            
            // Extract runtime (supports both "Продолжительность:" and "ВРЕМЯ:")
            if let Some(runtime_match) = RUNTIME_REGEX.captures(&text) {
                if let (Ok(hours), Ok(minutes)) = (
                    runtime_match.get(1).unwrap().as_str().parse::<u32>(),
                    runtime_match.get(2).unwrap().as_str().parse::<u32>()
                ) {
                    metadata.runtime_minutes = Some(hours * 60 + minutes);
                }
            }
            
            // Extract description - try main regex first, then fallback
            if let Some(desc_match) = DESCRIPTION_REGEX.captures(&text) {
                let mut description = desc_match.get(1).unwrap().as_str().trim().to_string();
                description = WHITESPACE_REGEX.replace_all(&description, " ").to_string();
                description = BR_REGEX.replace_all(&description, " ").to_string();
                if !description.is_empty() && description.len() > 20 {
                    metadata.description = Some(description);
                }
            } else if let Some(desc_match) = DESCRIPTION_FALLBACK_REGEX.captures(&text) {
                // Fallback: extract text after cast section
                let mut description = desc_match.get(1).unwrap().as_str().trim().to_string();
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
                        // Filter out very short paragraphs and common non-description text
                        p.len() > 50 && 
                        !p.to_lowercase().contains("скачать") &&
                        !p.to_lowercase().contains("торрент") &&
                        !p.to_lowercase().starts_with("формат:") &&
                        !p.to_lowercase().starts_with("видео:") &&
                        !p.to_lowercase().starts_with("аудио:") &&
                        !TIME_PATTERN_REGEX.is_match(p) // Filter out paragraphs that are just time
                    })
                    .collect();
                
                if !paragraphs.is_empty() {
                    // Join paragraphs and use as description
                    let combined = paragraphs.join(" ");
                    // Clean up the combined text - remove any leading time patterns
                    let mut final_desc = TIME_PATTERN_REGEX.replace(&combined, "").to_string();
                    final_desc = final_desc.trim().to_string();
                    if final_desc.len() > 50 {
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
                        // Skip if it starts with technical info or is too short
                        let cleaned_lower = cleaned.to_lowercase();
                        if !cleaned_lower.starts_with("формат:") && 
                           !cleaned_lower.starts_with("скачать") &&
                           !cleaned_lower.starts_with("видео:") &&
                           !cleaned_lower.starts_with("аудио:") &&
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
        
        eprintln!("✅ MonnaIndexer: Extracted metadata for: {} (poster: {:?})", metadata.title, metadata.poster_url.is_some());
        Ok(metadata)
    }

    fn extract_movie_from_prewposter(&self, movie_elem: &scraper::ElementRef) -> Option<TorrentResult> {
        eprintln!("🔍 Extraction: Starting extraction for movie element");
        
        // The <a> tag is the parent of div.prewposter, need to go up one level
        let parent = movie_elem.parent()?;
        eprintln!("🔍 Extraction: Found first parent");
        
        let link_node = parent.parent()?;
        eprintln!("🔍 Extraction: Found second parent (link node)");
        
        let link_elem = scraper::ElementRef::wrap(link_node)?;
        eprintln!("🔍 Extraction: Wrapped link element");
        
        let href = link_elem.value().attr("href");
        eprintln!("🔍 Extraction: Found href: {:?}", href);
        
        if href.is_none() {
            eprintln!("❌ Extraction: FAILED - No href found");
            return None;
        }
        
        // Extract title from image alt text
        let img_elem = movie_elem.select(&Selector::parse("img").unwrap()).next();
        eprintln!("🔍 Extraction: Found img element: {:?}", img_elem.is_some());
        
        if img_elem.is_none() {
            eprintln!("❌ Extraction: FAILED - No img element found");
            return None;
        }
        
        let img_elem = img_elem.unwrap();
        let text: String = movie_elem.text().collect();
        eprintln!("🔍 Extraction: Text content: '{}'", text.trim());
        
        let alt = img_elem.value().attr("alt");
        eprintln!("🔍 Extraction: Alt text: {:?}", alt);
        
        let title = alt
            .unwrap_or(text.trim())
            .to_string();
            
        eprintln!("✅ Extraction: Extracted title: '{}'", title);
        
        // Create basic torrent result with required fields
        let torrent_result = TorrentResult::new(
            title,
            String::new(), // magnet_link - will be populated from details page
            0,             // size_bytes - will be populated from details page
            0,             // seeders - will be populated from details page
            0,             // leechers - will be populated from details page
            "monna".to_string(),
        );
        
        eprintln!("✅ Extraction: Created TorrentResult successfully");
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
        eprintln!("🎬 MonnaIndexer: get_feed() called");
        info!("🎬 Fetching movie feed from Monna2 main page with metadata");
        let html = self.fetch_page(&self.base_url).await?;
        eprintln!("📄 MonnaIndexer: Fetched HTML length: {} chars", html.len());
        info!("📄 Fetched HTML length: {} chars", html.len());
        debug!("📄 HTML preview: {}", &html[..html.len().min(300)]);
        
        // Use the new parallel metadata fetching
        let movies = self.parse_movie_list_with_metadata(&html).await?;
        eprintln!("🎬 MonnaIndexer: Parsed {} raw movies with metadata from Monna2", movies.len());
        info!("🎬 Parsed {} raw movies with metadata from Monna2", movies.len());
        
        for (i, movie) in movies.iter().enumerate().take(3) {
            eprintln!("📽️ MonnaIndexer Movie {}: '{}' - {}", i, movie.title, movie.magnet_link);
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
                } else if line.starts_with("О фильме:") {
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
        let re = match Regex::new(r"(\d+(?:\.\d+)?)\s*(GB|MB|KB)") {
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
