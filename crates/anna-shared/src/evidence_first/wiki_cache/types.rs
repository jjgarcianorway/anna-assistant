//! Wiki cache types - page, section, and search result structures.

use serde::{Deserialize, Serialize};

/// Maximum length for citation excerpts.
const MAX_CITATION_EXCERPT_LEN: usize = 200;

/// Get current timestamp.
pub(super) fn timestamp_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Sanitize a title for use as filename.
pub(super) fn sanitize_filename(title: &str) -> String {
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
                        let excerpt = if line.len() > MAX_CITATION_EXCERPT_LEN {
                            format!("{}...", &line[..MAX_CITATION_EXCERPT_LEN])
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

/// Index file structure.
#[derive(Debug, Serialize, Deserialize)]
pub(super) struct WikiIndex {
    pub(super) pages: std::collections::HashMap<String, String>,
    pub(super) updated_at: u64,
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
