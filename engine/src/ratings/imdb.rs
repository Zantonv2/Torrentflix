// IMDb rating scraper
// Fetches rating from IMDb pages by scraping HTML

use anyhow::{Result, anyhow};
use reqwest::Client;
use scraper::{Html, Selector};
use tracing::debug;

/// IMDb client for scraping ratings
pub struct ImdbClient {
    client: Client,
}

impl ImdbClient {
    pub fn new() -> Self {
        // IMDb doesn't need proxy - it's publicly accessible
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
            .build()
            .expect("Failed to create HTTP client");
        
        Self { client }
    }

    /// Fetch IMDb rating by IMDb ID (e.g., "tt1234567")
    /// Returns the rating as a float (e.g., 7.8) or None if not found
    pub async fn get_rating(&self, imdb_id: &str) -> Result<Option<f32>> {
        // Ensure imdb_id starts with "tt" if it doesn't already
        let imdb_id = if imdb_id.starts_with("tt") {
            imdb_id.to_string()
        } else {
            format!("tt{}", imdb_id)
        };

        let url = format!("https://www.imdb.com/title/{}/", imdb_id);
        debug!("Fetching IMDb rating from: {}", url);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to fetch IMDb page: {}", response.status()));
        }

        let html = response.text().await?;
        self.parse_rating_from_html(&html)
    }

    /// Parse rating from IMDb HTML
    /// Optimized: First tries regex to extract just the rating container, then parses only that fragment
    /// Falls back to full document parsing if regex fails
    /// Looks for: <div data-testid="hero-rating-bar__aggregate-rating__score">...<span>7.8</span>...
    fn parse_rating_from_html(&self, html: &str) -> Result<Option<f32>> {
        use regex::Regex;
        
        // Helper function to extract rating from a document fragment
        let extract_rating_from_spans = |document: &Html| -> Option<f32> {
            let span_selector = match Selector::parse("span") {
                Ok(s) => s,
                Err(_) => return None,
            };
            
            for span in document.select(&span_selector) {
                let rating_text = span.text().collect::<String>().trim().to_string();
                
                // Skip spans that start with "/" (like "/10")
                if rating_text.starts_with('/') {
                    continue;
                }
                
                // Try to parse as float
                if let Ok(rating) = rating_text.parse::<f32>() {
                    // Validate that it's a reasonable rating (0-10)
                    if rating >= 0.0 && rating <= 10.0 {
                        return Some(rating);
                    }
                }
            }
            None
        };
        
        // First, try to extract just the rating container section using regex (much faster than parsing entire 1.3MB HTML)
        // Use (?s) for dotall mode to match across newlines
        let container_regex = Regex::new(
            r#"(?s)<div[^>]*data-testid="hero-rating-bar__aggregate-rating__score"[^>]*>(.*?)</div>"#
        ).map_err(|e| anyhow!("Failed to compile regex: {}", e))?;
        
        if let Some(captures) = container_regex.captures(html) {
            if let Some(container_html) = captures.get(1) {
                // Parse only the small container fragment instead of the entire HTML
                let container_fragment = format!("<div>{}</div>", container_html.as_str());
                let document = Html::parse_fragment(&container_fragment);
                
                if let Some(rating) = extract_rating_from_spans(&document) {
                    debug!("Parsed IMDb rating: {}", rating);
                    return Ok(Some(rating));
                }
            }
        }
        
        // Fallback: if regex doesn't find it, parse the entire document (slower but more reliable)
        debug!("Regex extraction failed, falling back to full document parsing");
        
        // Parse selector and document
        let selector_str = r#"div[data-testid="hero-rating-bar__aggregate-rating__score"]"#;
        let rating_container_selector = Selector::parse(selector_str)
            .map_err(|e| anyhow!("Failed to parse selector: {}", e))?;
        
        let document = Html::parse_document(html);

        if let Some(container) = document.select(&rating_container_selector).next() {
            let container_html = container.html();
            let container_doc = Html::parse_fragment(&container_html);
            
            if let Some(rating) = extract_rating_from_spans(&container_doc) {
                debug!("Parsed IMDb rating: {}", rating);
                return Ok(Some(rating));
            }
        }

        debug!("Rating not found in IMDb HTML");
        Ok(None)
    }
}

impl Default for ImdbClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_imdb_scraper_performance() {
        let client = ImdbClient::new();
        let imdb_id = "tt27704759";
        
        println!("\n🧪 ========== IMDb Scraper Performance Test ==========");
        println!("🧪 Testing IMDb ID: {}", imdb_id);
        
        // Test 1: Single request timing
        let start = std::time::Instant::now();
        match client.get_rating(imdb_id).await {
            Ok(rating) => {
                let elapsed = start.elapsed();
                println!("✅ Test 1: Single request completed in {:?}", elapsed);
                println!("   Rating: {:?}", rating);
                
                if elapsed.as_secs_f64() > 5.0 {
                    println!("⚠️  WARNING: Request took > 5 seconds! This is very slow.");
                } else if elapsed.as_secs_f64() > 2.0 {
                    println!("⚠️  WARNING: Request took > 2 seconds. This is slow.");
                } else {
                    println!("✅ Request speed is acceptable");
                }
            }
            Err(e) => {
                let elapsed = start.elapsed();
                println!("❌ Test 1 FAILED after {:?}: {}", elapsed, e);
            }
        }
        
        // Test 2: Multiple requests to see if there's caching or connection reuse issues
        println!("\n🧪 Test 2: Multiple sequential requests (5x)");
        let mut total_time = std::time::Duration::ZERO;
        for i in 1..=5 {
            let start = std::time::Instant::now();
            match client.get_rating(imdb_id).await {
                Ok(rating) => {
                    let elapsed = start.elapsed();
                    total_time += elapsed;
                    println!("   Request {}: {:?} - Rating: {:?}", i, elapsed, rating);
                }
                Err(e) => {
                    let elapsed = start.elapsed();
                    total_time += elapsed;
                    println!("   Request {}: {:?} - ERROR: {}", i, elapsed, e);
                }
            }
        }
        let avg_time = total_time / 5;
        println!("   Average time: {:?}", avg_time);
        
        // Test 3: Check if DNS resolution is the bottleneck
        println!("\n🧪 Test 3: Testing connection speed");
        let test_start = std::time::Instant::now();
        let test_url = format!("https://www.imdb.com/title/{}/", imdb_id);
        match client.client.get(&test_url).send().await {
            Ok(response) => {
                let connect_time = test_start.elapsed();
                println!("   Connection established in: {:?}", connect_time);
                
                let read_start = std::time::Instant::now();
                match response.text().await {
                    Ok(html) => {
                        let read_time = read_start.elapsed();
                        println!("   Response read in: {:?}", read_time);
                        println!("   HTML length: {} bytes", html.len());
                        println!("   Total time: {:?}", test_start.elapsed());
                    }
                    Err(e) => {
                        println!("   Failed to read response: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("   Connection failed: {}", e);
            }
        }
        
        println!("\n🧪 ========== Test Complete ==========\n");
    }
    
    #[tokio::test]
    async fn test_imdb_rating_parsing() {
        let client = ImdbClient::new();
        let imdb_id = "tt27704759";
        
        println!("\n🧪 ========== IMDb Rating Parsing Test ==========");
        
        let start = std::time::Instant::now();
        let url = format!("https://www.imdb.com/title/{}/", imdb_id);
        
        println!("🧪 Step 1: Fetching HTML...");
        let fetch_start = std::time::Instant::now();
        let response = match client.client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                println!("❌ Failed to fetch: {}", e);
                return;
            }
        };
        let fetch_time = fetch_start.elapsed();
        println!("   Fetch time: {:?}", fetch_time);
        
        println!("🧪 Step 2: Reading response body...");
        let read_start = std::time::Instant::now();
        let html = match response.text().await {
            Ok(h) => h,
            Err(e) => {
                println!("❌ Failed to read: {}", e);
                return;
            }
        };
        let read_time = read_start.elapsed();
        println!("   Read time: {:?}", read_time);
        println!("   HTML size: {} bytes", html.len());
        
        println!("🧪 Step 3: Parsing HTML...");
        let parse_start = std::time::Instant::now();
        let rating = match client.parse_rating_from_html(&html) {
            Ok(r) => r,
            Err(e) => {
                println!("❌ Failed to parse: {}", e);
                return;
            }
        };
        let parse_time = parse_start.elapsed();
        println!("   Parse time: {:?}", parse_time);
        println!("   Rating: {:?}", rating);
        
        let total_time = start.elapsed();
        println!("\n📊 Performance Summary:");
        println!("   Fetch: {:?} ({:.1}%)", fetch_time, fetch_time.as_secs_f64() / total_time.as_secs_f64() * 100.0);
        println!("   Read:  {:?} ({:.1}%)", read_time, read_time.as_secs_f64() / total_time.as_secs_f64() * 100.0);
        println!("   Parse: {:?} ({:.1}%)", parse_time, parse_time.as_secs_f64() / total_time.as_secs_f64() * 100.0);
        println!("   Total: {:?}", total_time);
        
        println!("\n🧪 ========== Test Complete ==========\n");
    }
}
