//! ArchWiki Cache - Offline Wiki Search (v0.0.435).
//!
//! Local cache of Arch Wiki pages for offline evidence retrieval.
//! Updated via `annactl wiki update`.

use super::citations::{Citation, CitationStore, EvidenceId};
use super::sources::KnowledgeSource;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// A cached wiki page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiPage {
    /// Page title.
    pub title: String,
    /// URL (for reference).
    pub url: String,
    /// Plain text content.
    pub content: String,
    /// Sections extracted.
    pub sections: Vec<WikiSection>,
    /// When cached.
    pub cached_at: u64,
    /// Tags/categories.
    pub categories: Vec<String>,
}

impl WikiPage {
    /// Create a new wiki page.
    pub fn new(title: &str, url: &str, content: &str) -> Self {
        let mut page = Self {
            title: title.to_string(),
            url: url.to_string(),
            content: content.to_string(),
            sections: Vec::new(),
            cached_at: timestamp_now(),
            categories: Vec::new(),
        };
        page.extract_sections();
        page
    }

    /// Extract sections from content.
    fn extract_sections(&mut self) {
        let mut current_section = String::new();
        let mut current_content = String::new();
        let mut current_level = 0u8;

        for line in self.content.lines() {
            // Detect markdown-style headers
            if line.starts_with('#') {
                // Save previous section
                if !current_section.is_empty() && !current_content.is_empty() {
                    self.sections.push(WikiSection {
                        title: current_section.clone(),
                        level: current_level,
                        content: current_content.trim().to_string(),
                    });
                }

                // Parse new section
                let level = line.chars().take_while(|c| *c == '#').count() as u8;
                current_section = line.trim_start_matches('#').trim().to_string();
                current_level = level;
                current_content.clear();
            } else {
                current_content.push_str(line);
                current_content.push('\n');
            }
        }

        // Add last section
        if !current_section.is_empty() && !current_content.is_empty() {
            self.sections.push(WikiSection {
                title: current_section,
                level: current_level,
                content: current_content.trim().to_string(),
            });
        }
    }

    /// Search page for query.
    pub fn search(&self, query: &str) -> Vec<WikiSearchHit> {
        let mut hits = Vec::new();
        let query_lower = query.to_lowercase();

        for section in &self.sections {
            let content_lower = section.content.to_lowercase();
            if content_lower.contains(&query_lower) {
                // Find matching lines
                for line in section.content.lines() {
                    if line.to_lowercase().contains(&query_lower) {
                        let excerpt = if line.len() > super::MAX_CITATION_EXCERPT_LEN {
                            format!("{}...", &line[..super::MAX_CITATION_EXCERPT_LEN])
                        } else {
                            line.to_string()
                        };

                        hits.push(WikiSearchHit {
                            page_title: self.title.clone(),
                            section_title: section.title.clone(),
                            excerpt,
                        });

                        if hits.len() >= 5 {
                            return hits;
                        }
                    }
                }
            }
        }

        hits
    }
}

/// A section within a wiki page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiSection {
    /// Section title.
    pub title: String,
    /// Heading level (1-6).
    pub level: u8,
    /// Section content.
    pub content: String,
}

/// A search hit from wiki.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiSearchHit {
    /// Page title.
    pub page_title: String,
    /// Section title.
    pub section_title: String,
    /// Matching excerpt.
    pub excerpt: String,
}

impl WikiSearchHit {
    /// Format as citation.
    pub fn format(&self) -> String {
        format!(
            "Arch Wiki: {} ({}) → \"{}\"",
            self.page_title, self.section_title, self.excerpt
        )
    }
}

/// Search result from wiki cache.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiSearchResult {
    /// Query used.
    pub query: String,
    /// Hits found.
    pub hits: Vec<WikiSearchHit>,
    /// Pages searched.
    pub pages_searched: usize,
}

impl WikiSearchResult {
    /// Check if any results found.
    pub fn has_results(&self) -> bool {
        !self.hits.is_empty()
    }

    /// Get first hit.
    pub fn first(&self) -> Option<&WikiSearchHit> {
        self.hits.first()
    }
}

/// The wiki cache.
#[derive(Debug, Clone, Default)]
pub struct WikiCache {
    /// Cache directory.
    cache_dir: PathBuf,
    /// Index of pages by title.
    index: HashMap<String, PathBuf>,
    /// When index was last updated.
    index_updated: u64,
}

impl WikiCache {
    /// Create a new wiki cache.
    pub fn new(cache_dir: &Path) -> Self {
        let mut cache = Self {
            cache_dir: cache_dir.to_path_buf(),
            index: HashMap::new(),
            index_updated: 0,
        };
        cache.load_index();
        cache
    }

    /// Create with default directory.
    pub fn default_location() -> Self {
        Self::new(Path::new(super::WIKI_CACHE_DIR))
    }

    /// Load index from cache directory.
    fn load_index(&mut self) {
        let index_path = self.cache_dir.join("index.json");
        if let Ok(content) = fs::read_to_string(&index_path) {
            if let Ok(index) = serde_json::from_str::<WikiIndex>(&content) {
                self.index = index
                    .pages
                    .into_iter()
                    .map(|(k, v)| (k, PathBuf::from(v)))
                    .collect();
                self.index_updated = index.updated_at;
            }
        }
    }

    /// Save index to cache directory.
    fn save_index(&self) -> Result<(), CacheError> {
        fs::create_dir_all(&self.cache_dir).map_err(|e| CacheError::IoError(e.to_string()))?;

        let index = WikiIndex {
            pages: self
                .index
                .iter()
                .map(|(k, v)| (k.clone(), v.to_string_lossy().to_string()))
                .collect(),
            updated_at: self.index_updated,
        };

        let content = serde_json::to_string_pretty(&index)
            .map_err(|e| CacheError::SerializeError(e.to_string()))?;

        fs::write(self.cache_dir.join("index.json"), content)
            .map_err(|e| CacheError::IoError(e.to_string()))?;

        Ok(())
    }

    /// Get a cached page.
    pub fn get_page(&self, title: &str) -> Option<WikiPage> {
        let path = self.index.get(title)?;
        let content = fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// Add a page to cache.
    pub fn add_page(&mut self, page: &WikiPage) -> Result<(), CacheError> {
        fs::create_dir_all(&self.cache_dir).map_err(|e| CacheError::IoError(e.to_string()))?;

        let filename = sanitize_filename(&page.title);
        let path = self.cache_dir.join(format!("{}.json", filename));

        let content = serde_json::to_string_pretty(page)
            .map_err(|e| CacheError::SerializeError(e.to_string()))?;

        fs::write(&path, content).map_err(|e| CacheError::IoError(e.to_string()))?;

        self.index.insert(page.title.clone(), path);
        self.index_updated = timestamp_now();
        self.save_index()?;

        Ok(())
    }

    /// Search all cached pages.
    pub fn search(&self, query: &str) -> WikiSearchResult {
        let mut all_hits = Vec::new();
        let mut pages_searched = 0;

        for title in self.index.keys() {
            if let Some(page) = self.get_page(title) {
                pages_searched += 1;
                let hits = page.search(query);
                all_hits.extend(hits);

                if all_hits.len() >= 10 {
                    break;
                }
            }
        }

        // Sort by relevance (title match first)
        all_hits.sort_by(|a, b| {
            let a_title_match = a.page_title.to_lowercase().contains(&query.to_lowercase());
            let b_title_match = b.page_title.to_lowercase().contains(&query.to_lowercase());
            b_title_match.cmp(&a_title_match)
        });

        all_hits.truncate(10);

        WikiSearchResult {
            query: query.to_string(),
            hits: all_hits,
            pages_searched,
        }
    }

    /// Search and add citations to store.
    pub fn search_with_citations(
        &self,
        query: &str,
        store: &mut CitationStore,
    ) -> WikiSearchResult {
        let result = self.search(query);

        for hit in &result.hits {
            let evidence_id = EvidenceId::wiki(&hit.page_title);

            // Add raw evidence if not already present
            if !store.has_evidence(&evidence_id) {
                if let Some(page) = self.get_page(&hit.page_title) {
                    store.add_evidence(
                        evidence_id.clone(),
                        KnowledgeSource::ArchWiki(page.title.clone()),
                        &page.content,
                    );
                }
            }

            // Add citation
            store.add_citation(
                Citation::new(
                    evidence_id,
                    &format!("Arch Wiki: {}", hit.page_title),
                    &hit.excerpt,
                )
                .with_context(&hit.section_title),
            );
        }

        result
    }

    /// List all cached pages.
    pub fn list_pages(&self) -> Vec<String> {
        self.index.keys().cloned().collect()
    }

    /// Get cache stats.
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            page_count: self.index.len(),
            last_updated: self.index_updated,
            cache_dir: self.cache_dir.to_string_lossy().to_string(),
        }
    }

    /// Check if cache is stale (older than 30 days).
    pub fn is_stale(&self) -> bool {
        let now = timestamp_now();
        let thirty_days = 30 * 24 * 60 * 60;
        now - self.index_updated > thirty_days
    }

    /// Clear the cache.
    pub fn clear(&mut self) -> Result<(), CacheError> {
        for path in self.index.values() {
            let _ = fs::remove_file(path);
        }
        self.index.clear();
        self.index_updated = 0;
        self.save_index()
    }
}

/// Index file structure.
#[derive(Debug, Serialize, Deserialize)]
struct WikiIndex {
    pages: HashMap<String, String>,
    updated_at: u64,
}

/// Cache statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// Number of cached pages.
    pub page_count: usize,
    /// When last updated.
    pub last_updated: u64,
    /// Cache directory path.
    pub cache_dir: String,
}

/// Cache error.
#[derive(Debug, Clone)]
pub enum CacheError {
    /// IO error.
    IoError(String),
    /// Serialization error.
    SerializeError(String),
    /// Network error.
    NetworkError(String),
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(msg) => write!(f, "IO error: {}", msg),
            Self::SerializeError(msg) => write!(f, "Serialize error: {}", msg),
            Self::NetworkError(msg) => write!(f, "Network error: {}", msg),
        }
    }
}

impl std::error::Error for CacheError {}

/// Sanitize a title for use as filename.
fn sanitize_filename(title: &str) -> String {
    title
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Get current timestamp.
fn timestamp_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Common Arch Wiki pages to pre-cache.
pub const ESSENTIAL_PAGES: &[&str] = &[
    "Systemd",
    "Pacman",
    "Network_configuration",
    "Wireless_network_configuration",
    "PulseAudio",
    "PipeWire",
    "Xorg",
    "Wayland",
    "NVIDIA",
    "AMD_GPU",
    "Intel_graphics",
    "Boot_debugging",
    "General_troubleshooting",
    "Improving_performance",
    "SSD",
    "Swap",
    "Kernel_parameters",
    "Grub",
    "Systemd-boot",
    "Bluetooth",
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn temp_cache_dir() -> PathBuf {
        env::temp_dir().join(format!("anna_wiki_test_{}", timestamp_now()))
    }

    #[test]
    fn test_wiki_page_creation() {
        let content = "# Introduction\nThis is the intro.\n# Configuration\nConfig details here.";
        let page = WikiPage::new("Test Page", "https://wiki.archlinux.org/Test", content);

        assert_eq!(page.title, "Test Page");
        assert_eq!(page.sections.len(), 2);
    }

    #[test]
    fn test_wiki_page_search() {
        let content = "# Introduction\nSystemd is the init system.\n# Services\nUse systemctl to manage services.";
        let page = WikiPage::new("Systemd", "https://wiki.archlinux.org/Systemd", content);

        let hits = page.search("systemctl");
        assert!(!hits.is_empty());
        assert!(hits[0].excerpt.contains("systemctl"));
    }

    #[test]
    fn test_wiki_cache_new() {
        let cache_dir = temp_cache_dir();
        let cache = WikiCache::new(&cache_dir);

        // New cache should be empty
        assert!(cache.list_pages().is_empty());
        assert_eq!(cache.stats().page_count, 0);
    }

    #[test]
    fn test_wiki_cache_stale_check() {
        let cache_dir = temp_cache_dir();
        let cache = WikiCache::new(&cache_dir);

        // New cache with no updates is stale (index_updated is 0)
        assert!(cache.is_stale());
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("Simple"), "Simple");
        assert_eq!(sanitize_filename("With Spaces"), "With_Spaces");
        assert_eq!(sanitize_filename("Special/Chars!"), "Special_Chars_");
    }

    #[test]
    fn test_search_hit_format() {
        let hit = WikiSearchHit {
            page_title: "Systemd".to_string(),
            section_title: "Services".to_string(),
            excerpt: "Use systemctl".to_string(),
        };

        let formatted = hit.format();
        assert!(formatted.contains("Systemd"));
        assert!(formatted.contains("Services"));
        assert!(formatted.contains("systemctl"));
    }
}
