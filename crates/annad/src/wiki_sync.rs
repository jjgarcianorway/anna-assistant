//! Wiki Sync - Cache Arch Wiki solutions for offline access
//!
//! Anna learns from the Arch Wiki by caching relevant pages based on:
//! - Errors she sees in logs
//! - User questions and issues
//! - Common system patterns

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, info};

const WIKI_CACHE_DIR: &str = "/var/lib/anna/wiki";
const WIKI_INDEX_FILE: &str = "/var/lib/anna/wiki/index.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiPage {
    pub title: String,
    pub url: String,
    pub content: String,
    pub keywords: Vec<String>,
    pub cached_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiIndex {
    pub pages: HashMap<String, WikiPage>,
    pub last_sync: String,
}

impl Default for WikiIndex {
    fn default() -> Self {
        Self {
            pages: HashMap::new(),
            last_sync: chrono::Utc::now().to_rfc3339(),
        }
    }
}

impl WikiIndex {
    /// Load wiki index from disk
    pub fn load() -> Self {
        let path = PathBuf::from(WIKI_INDEX_FILE);
        if !path.exists() {
            return Self::default();
        }

        match std::fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Save wiki index to disk
    pub fn save(&self) -> Result<()> {
        let path = PathBuf::from(WIKI_INDEX_FILE);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Add or update a wiki page
    pub fn add_page(&mut self, page: WikiPage) {
        info!("Caching wiki page: {}", page.title);
        self.pages.insert(page.title.clone(), page);
        self.last_sync = chrono::Utc::now().to_rfc3339();
    }

    /// Search for pages by keyword
    pub fn search(&self, query: &str) -> Vec<&WikiPage> {
        let query_lower = query.to_lowercase();
        self.pages
            .values()
            .filter(|page| {
                page.title.to_lowercase().contains(&query_lower)
                    || page.keywords.iter().any(|k| k.to_lowercase().contains(&query_lower))
            })
            .collect()
    }
}

/// Priority wiki topics to cache based on common Arch issues
const PRIORITY_TOPICS: &[&str] = &[
    "Installation_guide",
    "General_recommendations",
    "Pacman",
    "System_maintenance",
    "Systemd",
    "Network_configuration",
    "Graphics_drivers",
    "Sound_system",
    "Bluetooth",
    "Power_management",
    "Kernel",
    "Boot_loaders",
    "File_systems",
    "Users_and_groups",
    "Security",
];

/// Sync wiki pages based on recent system errors
pub async fn sync_wiki_pages() -> Result<Vec<String>> {
    debug!("Starting wiki sync");

    let mut index = WikiIndex::load();
    let mut synced = Vec::new();

    // Get topics from recent errors
    let error_topics = extract_topics_from_errors().await?;

    // Combine priority topics with error-based topics
    let mut topics_to_fetch: Vec<String> = error_topics;

    // Add a few priority topics if index is small
    if index.pages.len() < 10 {
        topics_to_fetch.extend(
            PRIORITY_TOPICS[..5]
                .iter()
                .map(|s| s.to_string())
        );
    }

    // Limit to 3 pages per sync (don't overwhelm)
    topics_to_fetch.truncate(3);

    for topic in topics_to_fetch {
        // Skip if already cached recently (within 7 days)
        if let Some(existing) = index.pages.get(&topic) {
            if let Ok(cached_time) = chrono::DateTime::parse_from_rfc3339(&existing.cached_at) {
                let age = chrono::Utc::now().signed_duration_since(cached_time.with_timezone(&chrono::Utc));
                if age.num_days() < 7 {
                    debug!("Skipping {} (cached {} days ago)", topic, age.num_days());
                    continue;
                }
            }
        }

        match fetch_wiki_page(&topic).await {
            Ok(page) => {
                synced.push(topic.clone());
                index.add_page(page);
            }
            Err(e) => {
                debug!("Failed to fetch wiki page {}: {}", topic, e);
            }
        }

        // Be nice to the wiki - small delay between requests
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    index.save()?;

    if !synced.is_empty() {
        info!("Synced {} wiki pages: {:?}", synced.len(), synced);
    }

    Ok(synced)
}

/// Extract topics from recent system errors
async fn extract_topics_from_errors() -> Result<Vec<String>> {
    let output = std::process::Command::new("journalctl")
        .args(["-p", "err", "-n", "50", "--no-pager"])
        .output()?;

    let logs = String::from_utf8_lossy(&output.stdout);
    let mut topics = Vec::new();

    // Look for common error patterns and map to wiki topics
    if logs.contains("bluetooth") || logs.contains("Bluetooth") {
        topics.push("Bluetooth".to_string());
    }
    if logs.contains("network") || logs.contains("NetworkManager") {
        topics.push("Network_configuration".to_string());
    }
    if logs.contains("systemd") || logs.contains(".service") {
        topics.push("Systemd".to_string());
    }
    if logs.contains("pacman") || logs.contains("package") {
        topics.push("Pacman".to_string());
    }
    if logs.contains("gpu") || logs.contains("nvidia") || logs.contains("amdgpu") {
        topics.push("Graphics_drivers".to_string());
    }
    if logs.contains("audio") || logs.contains("alsa") || logs.contains("pulseaudio") {
        topics.push("Sound_system".to_string());
    }
    if logs.contains("grub") || logs.contains("boot") {
        topics.push("Boot_loaders".to_string());
    }

    // Remove duplicates
    topics.sort();
    topics.dedup();

    Ok(topics)
}

/// Fetch a wiki page from Arch Wiki
async fn fetch_wiki_page(topic: &str) -> Result<WikiPage> {
    let url = format!("https://wiki.archlinux.org/title/{}", topic);

    debug!("Fetching wiki page: {}", url);

    // Use curl to fetch (simple, reliable, already installed)
    let output = std::process::Command::new("curl")
        .args(["-s", "-L", &url])
        .output()?;

    if !output.status.success() {
        anyhow::bail!("Failed to fetch wiki page");
    }

    let html = String::from_utf8_lossy(&output.stdout);

    // Extract main content (very basic HTML parsing)
    let content = extract_wiki_content(&html);

    // Extract keywords from content
    let keywords = extract_keywords(&content);

    Ok(WikiPage {
        title: topic.to_string(),
        url,
        content,
        keywords,
        cached_at: chrono::Utc::now().to_rfc3339(),
    })
}

/// Extract main content from Arch Wiki HTML (basic parsing)
fn extract_wiki_content(html: &str) -> String {
    // Find the main content div
    let content_start = html.find("<div id=\"mw-content-text\"");
    let content_end = html.find("<div id=\"catlinks\"");

    if let (Some(start), Some(end)) = (content_start, content_end) {
        let content_html = &html[start..end];

        // Strip HTML tags (very basic)
        let mut text = String::new();
        let mut in_tag = false;

        for c in content_html.chars() {
            match c {
                '<' => in_tag = true,
                '>' => in_tag = false,
                _ if !in_tag => text.push(c),
                _ => {}
            }
        }

        // Clean up whitespace
        text.split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .chars()
            .take(5000) // Limit to 5000 chars
            .collect()
    } else {
        String::new()
    }
}

/// Extract keywords from content
fn extract_keywords(content: &str) -> Vec<String> {
    let mut keywords = Vec::new();
    let words: Vec<&str> = content.split_whitespace().collect();

    // Common technical terms that appear in Arch Wiki
    let technical_terms = [
        "pacman", "systemd", "bluetooth", "network", "kernel",
        "grub", "nvidia", "amdgpu", "pulseaudio", "alsa",
        "wifi", "ethernet", "ssh", "firewall", "driver",
    ];

    for term in &technical_terms {
        if content.to_lowercase().contains(term) {
            keywords.push(term.to_string());
        }
    }

    keywords.truncate(10); // Max 10 keywords
    keywords
}

/// Search cached wiki pages for a query
pub fn search_wiki(query: &str) -> Vec<WikiPage> {
    let index = WikiIndex::load();
    index.search(query).into_iter().cloned().collect()
}

/// Get wiki page summary for a topic
pub fn get_wiki_summary(topic: &str) -> Option<String> {
    let index = WikiIndex::load();
    index.pages.get(topic).map(|page| {
        // Return first 500 chars as summary
        page.content.chars().take(500).collect::<String>()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_keywords() {
        let content = "This is about pacman and systemd networking with bluetooth.";
        let keywords = extract_keywords(content);

        assert!(keywords.contains(&"pacman".to_string()));
        assert!(keywords.contains(&"systemd".to_string()));
        assert!(keywords.contains(&"bluetooth".to_string()));
        assert!(keywords.contains(&"network".to_string()));
    }

    #[test]
    fn test_wiki_index() {
        let mut index = WikiIndex::default();

        let page = WikiPage {
            title: "Test".to_string(),
            url: "https://test".to_string(),
            content: "Test content".to_string(),
            keywords: vec!["test".to_string()],
            cached_at: chrono::Utc::now().to_rfc3339(),
        };

        index.add_page(page);
        assert_eq!(index.pages.len(), 1);

        let results = index.search("test");
        assert_eq!(results.len(), 1);
    }
}
