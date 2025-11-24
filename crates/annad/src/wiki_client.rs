//! Wiki Client - Fetches and caches Arch Wiki content
//!
//! Handles HTTP requests to wiki.archlinux.org with local caching to avoid
//! hammering the wiki servers. Extracts clean text from HTML for reasoning.

use anyhow::{Context, Result};
use anna_common::wiki_reasoner::WikiTopic;
use anna_common::wiki_topics;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use tracing::{debug, warn};

/// Cache TTL in seconds (7 days)
const CACHE_TTL_SECS: u64 = 7 * 24 * 60 * 60;

/// Wiki client with caching
pub struct WikiClient {
    cache_dir: PathBuf,
    http: reqwest::Client,
}

/// Complete wiki page content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiPage {
    pub url: String,
    pub title: String,
    pub content: String,
    #[serde(skip)]
    pub cached_at: Option<SystemTime>,
}

/// Wiki page section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiPageSection {
    pub url: String,
    pub section: Option<String>,
    pub content: String,
}

/// Wiki client errors
#[derive(Debug, thiserror::Error)]
pub enum WikiClientError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Wiki unavailable (offline mode)")]
    Unavailable,
}

impl WikiClient {
    /// Create a new wiki client with cache directory
    pub fn new(cache_dir: PathBuf) -> Result<Self> {
        // Ensure cache directory exists
        std::fs::create_dir_all(&cache_dir)
            .with_context(|| format!("Failed to create wiki cache dir: {:?}", cache_dir))?;

        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("Anna Assistant/6.23.0 (Arch Linux sysadmin; +https://github.com/jjgarcianorway/anna-assistant)")
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self { cache_dir, http })
    }

    /// Fetch a wiki topic page (with caching)
    pub async fn fetch_topic(&self, topic: WikiTopic) -> Result<WikiPage, WikiClientError> {
        let metadata = wiki_topics::get_topic_metadata(topic);
        let url = metadata.page_url.to_string();

        // Try cache first
        if let Ok(Some(cached)) = self.load_from_cache(&url) {
            debug!("Wiki cache hit for {}", url);
            return Ok(cached);
        }

        // Fetch from network
        debug!("Fetching wiki page: {}", url);
        let page = self.fetch_page(&url).await?;

        // Save to cache
        let _ = self.save_to_cache(&url, &page);

        Ok(page)
    }

    /// Fetch a specific section of a wiki page
    pub async fn fetch_section(
        &self,
        topic: WikiTopic,
        section_hint: &str,
    ) -> Result<WikiPageSection, WikiClientError> {
        let page = self.fetch_topic(topic).await?;

        // Extract section content
        let section_content = self.extract_section(&page.content, section_hint);

        Ok(WikiPageSection {
            url: format!("{}#{}", page.url, section_hint.replace(' ', "_")),
            section: Some(section_hint.to_string()),
            content: section_content.unwrap_or_else(|| {
                // If section not found, return full page content
                page.content
            }),
        })
    }

    /// Fetch raw page from network
    async fn fetch_page(&self, url: &str) -> Result<WikiPage, WikiClientError> {
        let response = self
            .http
            .get(url)
            .send()
            .await
            .map_err(|e| WikiClientError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(WikiClientError::Network(format!(
                "HTTP {}",
                response.status()
            )));
        }

        let html = response
            .text()
            .await
            .map_err(|e| WikiClientError::Network(e.to_string()))?;

        // Extract title and content from HTML
        let (title, content) = self.parse_wiki_html(&html)?;

        Ok(WikiPage {
            url: url.to_string(),
            title,
            content,
            cached_at: Some(SystemTime::now()),
        })
    }

    /// Parse Arch Wiki HTML to extract title and clean content
    fn parse_wiki_html(&self, html: &str) -> Result<(String, String), WikiClientError> {
        use scraper::{Html, Selector};

        let document = Html::parse_document(html);

        // Extract title
        let title_selector =
            Selector::parse("h1.firstHeading").map_err(|e| WikiClientError::Parse(e.to_string()))?;
        let title = document
            .select(&title_selector)
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_else(|| "Unknown".to_string());

        // Extract main content
        let content_selector = Selector::parse("#mw-content-text .mw-parser-output")
            .map_err(|e| WikiClientError::Parse(e.to_string()))?;
        let content_element = document
            .select(&content_selector)
            .next()
            .ok_or_else(|| WikiClientError::Parse("No content found".to_string()))?;

        // Convert to clean text (remove scripts, styles, etc.)
        let content = self.html_to_text(content_element.html().as_str());

        Ok((title, content))
    }

    /// Convert HTML to clean text
    fn html_to_text(&self, html: &str) -> String {
        use html2text::from_read;

        // Use html2text for basic conversion
        let text = from_read(html.as_bytes(), 100);

        // Clean up excessive whitespace
        text.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Extract a section from wiki content
    fn extract_section(&self, content: &str, section_hint: &str) -> Option<String> {
        let section_lower = section_hint.to_lowercase();
        let lines: Vec<&str> = content.lines().collect();

        // Find section start
        let start_idx = lines.iter().position(|line| {
            let line_lower = line.to_lowercase();
            line_lower.contains(&section_lower) && (line.starts_with('#') || line.starts_with("=="))
        })?;

        // Find next section (same or higher level heading)
        let end_idx = lines[start_idx + 1..]
            .iter()
            .position(|line| line.starts_with('#') || line.starts_with("=="))
            .map(|i| start_idx + 1 + i)
            .unwrap_or(lines.len());

        Some(lines[start_idx..end_idx].join("\n"))
    }

    /// Load page from cache if valid
    fn load_from_cache(&self, url: &str) -> Result<Option<WikiPage>> {
        let cache_path = self.cache_path(url);

        if !cache_path.exists() {
            return Ok(None);
        }

        // Check cache age
        let metadata = std::fs::metadata(&cache_path)?;
        let modified = metadata.modified()?;
        let age = SystemTime::now().duration_since(modified)?.as_secs();

        if age > CACHE_TTL_SECS {
            debug!("Wiki cache expired for {}", url);
            return Ok(None);
        }

        // Load from cache
        let content = std::fs::read_to_string(&cache_path)?;
        let mut page: WikiPage = serde_json::from_str(&content)?;
        page.cached_at = Some(modified);

        Ok(Some(page))
    }

    /// Save page to cache
    fn save_to_cache(&self, url: &str, page: &WikiPage) -> Result<()> {
        let cache_path = self.cache_path(url);
        let content = serde_json::to_string_pretty(page)?;
        std::fs::write(&cache_path, content)?;
        debug!("Saved wiki page to cache: {:?}", cache_path);
        Ok(())
    }

    /// Get cache file path for URL
    fn cache_path(&self, url: &str) -> PathBuf {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        url.hash(&mut hasher);
        let hash = hasher.finish();

        self.cache_dir.join(format!("wiki_{:x}.json", hash))
    }
}

impl Default for WikiClient {
    fn default() -> Self {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("anna")
            .join("wiki_cache");

        Self::new(cache_dir).expect("Failed to create wiki client")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_to_text_basic() {
        let client = WikiClient::default();
        let html = "<p>This is a <b>test</b> paragraph.</p>";
        let text = client.html_to_text(html);
        assert!(text.contains("test"));
    }

    #[test]
    fn test_extract_section() {
        let client = WikiClient::default();
        let content = r#"
# Main Title
Some content
## Troubleshooting
Troubleshooting content here
More troubleshooting
## Configuration
Config content
"#;
        let section = client.extract_section(content, "Troubleshooting");
        assert!(section.is_some());
        let section_text = section.unwrap();
        assert!(section_text.contains("Troubleshooting"));
        assert!(!section_text.contains("Configuration"));
    }

    #[test]
    fn test_cache_path_deterministic() {
        let client = WikiClient::default();
        let url = "https://wiki.archlinux.org/title/Power_management";
        let path1 = client.cache_path(url);
        let path2 = client.cache_path(url);
        assert_eq!(path1, path2);
    }
}
