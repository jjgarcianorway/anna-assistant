//! Documentation snippet types

use super::source::DocSourceKind;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// A documentation snippet (indexed and retrievable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocSnippet {
    /// Stable identifier (e.g., "man:systemctl:1:description")
    pub id: String,
    /// Source type
    pub source: DocSourceKind,
    /// Name (e.g., "systemd", "pacman", "fsck")
    pub name: String,
    /// Section (e.g., "systemd.mount", "pacman#Options", "1")
    pub section: Option<String>,
    /// Short description/summary
    pub summary: String,
    /// Raw or lightly cleaned content
    pub content: String,
    /// Keywords for indexing
    pub keywords: Vec<String>,
    /// When this was indexed
    pub indexed_at: u64,
    /// Relevance score (0-100, set during query)
    #[serde(default)]
    pub relevance: u8,
}

impl DocSnippet {
    /// Create a new snippet
    pub fn new(
        source: DocSourceKind,
        name: &str,
        section: Option<&str>,
        summary: &str,
        content: &str,
    ) -> Self {
        let id = Self::generate_id(source, name, section);
        let keywords = Self::extract_keywords(name, section, summary);

        Self {
            id,
            source,
            name: name.to_string(),
            section: section.map(|s| s.to_string()),
            summary: summary.to_string(),
            content: content.to_string(),
            keywords,
            indexed_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            relevance: 0,
        }
    }

    /// Generate stable ID from components
    pub fn generate_id(source: DocSourceKind, name: &str, section: Option<&str>) -> String {
        let source_prefix = match source {
            DocSourceKind::ArchWiki => "wiki",
            DocSourceKind::ManPage => "man",
            DocSourceKind::ToolHelp => "help",
            DocSourceKind::LocalDoc => "doc",
        };

        let name_clean = name.to_lowercase().replace(' ', "_");

        if let Some(sec) = section {
            let sec_clean = sec.to_lowercase().replace(' ', "_").replace('#', "_");
            format!("{}:{}:{}", source_prefix, name_clean, sec_clean)
        } else {
            format!("{}:{}", source_prefix, name_clean)
        }
    }

    /// Extract keywords from snippet metadata
    fn extract_keywords(name: &str, section: Option<&str>, summary: &str) -> Vec<String> {
        let mut keywords = vec![name.to_lowercase()];

        if let Some(sec) = section {
            keywords.push(sec.to_lowercase());
        }

        // Extract words from summary (skip common words)
        let stop_words = [
            "the", "a", "an", "is", "are", "to", "for", "and", "or", "of", "in",
        ];
        for word in summary.split_whitespace() {
            let word_lower = word
                .to_lowercase()
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_string();
            if word_lower.len() > 2 && !stop_words.contains(&word_lower.as_str()) {
                if !keywords.contains(&word_lower) {
                    keywords.push(word_lower);
                }
            }
        }

        keywords
    }

    /// Get citation string for this snippet
    pub fn citation(&self) -> String {
        self.source
            .citation_format(&self.name, self.section.as_deref())
    }

    /// Check if this snippet is stale (needs refresh)
    pub fn is_stale(&self, max_age_days: u64) -> bool {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let age_secs = now.saturating_sub(self.indexed_at);
        let age_days = age_secs / (24 * 60 * 60);

        age_days > max_age_days
    }

    /// Truncate content to max size
    pub fn truncate_content(&mut self, max_size: usize) {
        if self.content.len() > max_size {
            // Find a good break point
            let truncate_at = self.content[..max_size]
                .rfind(|c: char| c == '\n' || c == '.' || c == ' ')
                .unwrap_or(max_size);
            self.content.truncate(truncate_at);
            self.content.push_str("...");
        }
    }

    /// Set relevance score
    pub fn with_relevance(mut self, score: u8) -> Self {
        self.relevance = score.min(100);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snippet_id_generation() {
        let id = DocSnippet::generate_id(DocSourceKind::ManPage, "systemctl", Some("1"));
        assert_eq!(id, "man:systemctl:1");

        let id =
            DocSnippet::generate_id(DocSourceKind::ArchWiki, "Systemd", Some("Troubleshooting"));
        assert_eq!(id, "wiki:systemd:troubleshooting");
    }

    #[test]
    fn test_snippet_citation() {
        let snippet = DocSnippet::new(
            DocSourceKind::ManPage,
            "systemctl",
            Some("1"),
            "Control the systemd system and service manager",
            "...",
        );
        assert_eq!(snippet.citation(), "systemctl(1)");

        let snippet = DocSnippet::new(
            DocSourceKind::ArchWiki,
            "systemd",
            Some("Troubleshooting"),
            "Troubleshooting systemd",
            "...",
        );
        assert_eq!(snippet.citation(), "Arch Wiki: systemd#Troubleshooting");
    }
}
