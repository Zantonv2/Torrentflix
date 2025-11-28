use async_trait::async_trait;
use anyhow::Result;
use reqwest::Client;
use scraper::{Html, Selector};
use tracing::{debug, info, warn};
use tokio::time::{sleep, Duration};

use crate::models::{TorrentResult};
use crate::indexers::traits::{Indexer, IndexerConfig, SearchQuery};

#[derive(Debug, Clone)]
pub struct MonnaIndexer {
    client: Client,
    base_url: String,
    config: IndexerConfig,
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
        let document = Html::parse_document(html);
        let mut results = Vec::new();

        // Based on HTML analysis, movies are in <div class="prewposter"> within <center> tags
        let movie_selector = Selector::parse("div.prewposter").unwrap();
        let movie_elements: Vec<_> = document.select(&movie_selector).collect();
        
        info!("Found {} movie elements with selector div.prewposter", movie_elements.len());
        
        for movie_elem in movie_elements {
            if let Some(torrent_result) = self.extract_movie_from_prewposter(&movie_elem) {
                results.push(torrent_result);
            }
        }
        
        info!("Successfully parsed {} movies from main page", results.len());
        Ok(results)
    }

    fn extract_movie_from_prewposter(&self, movie_elem: &scraper::ElementRef) -> Option<TorrentResult> {
        // The <a> tag is the parent of div.prewposter, need to go up one level
        let link_node = movie_elem.parent()?.parent()?;
        let link_elem = scraper::ElementRef::wrap(link_node)?;
        let _href = link_elem.value().attr("href")?; // Prefix with underscore to indicate unused
        
        // Extract title from image alt text
        let img_elem = movie_elem.select(&Selector::parse("img").unwrap()).next()?;
        let text: String = movie_elem.text().collect();
        let title = img_elem.value().attr("alt")
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
        
        // Parse search results - they should have the same structure as main page
        self.parse_movie_list(&html)
    }

    async fn get_details(&self, info_hash: &str) -> Result<Option<TorrentResult>> {
        debug!("Getting details for hash: {}", info_hash);
        
        // For now, return None as we need to implement proper hash-based lookup
        // This would require storing mappings from URLs to hashes
        Ok(None)
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
