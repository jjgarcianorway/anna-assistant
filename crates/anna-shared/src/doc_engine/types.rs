//! Core types for the documentation engine (v0.0.429).

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Documentation source kind, in priority order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocSourceKind {
    /// Arch Wiki (preferred for conceptual/Arch-specific guidance)
    ArchWiki,
    /// Man pages (command semantics and flags)
    ManPage,
    /// Tool help output (--help, -h)
    ToolHelp,
    /// Local documentation files (/usr/share/doc, etc.)
    LocalDoc,
}

impl DocSourceKind {
    /// Priority for sorting (lower = higher priority)
    pub fn priority(&self) -> u8 {
        match self {
            Self::ArchWiki => 1,
            Self::ManPage => 2,
            Self::ToolHelp => 3,
            Self::LocalDoc => 4,
        }
    }

    /// Human-readable display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::ArchWiki => "Arch Wiki",
            Self::ManPage => "Man page",
            Self::ToolHelp => "Help output",
            Self::LocalDoc => "Local doc",
        }
    }

    /// Citation format for evidence
    pub fn citation_format(&self, name: &str, section: Option<&str>) -> String {
        match self {
            Self::ArchWiki => {
                if let Some(sec) = section {
                    format!("Arch Wiki: {}#{}", name, sec)
                } else {
                    format!("Arch Wiki: {}", name)
                }
            }
            Self::ManPage => {
                if let Some(sec) = section {
                    format!("{}({})", name, sec)
                } else {
                    format!("{}", name)
                }
            }
            Self::ToolHelp => format!("{} --help", name),
            Self::LocalDoc => format!("/usr/share/doc/{}", name),
        }
    }
}

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

/// Query for documentation
#[derive(Debug, Clone)]
pub struct DocQuery {
    /// Search query text
    pub query: String,
    /// Preferred sources (in order)
    pub preferred_sources: Vec<DocSourceKind>,
    /// Maximum results to return
    pub limit: usize,
    /// Specific name to search (e.g., "systemctl")
    pub name_filter: Option<String>,
    /// Minimum relevance score (0-100)
    pub min_relevance: u8,
}

impl DocQuery {
    /// Create a new query
    pub fn new(query: &str) -> Self {
        Self {
            query: query.to_string(),
            preferred_sources: vec![
                DocSourceKind::ArchWiki,
                DocSourceKind::ManPage,
                DocSourceKind::ToolHelp,
            ],
            limit: 5,
            name_filter: None,
            min_relevance: 0,
        }
    }

    /// Set preferred sources
    pub fn with_sources(mut self, sources: Vec<DocSourceKind>) -> Self {
        self.preferred_sources = sources;
        self
    }

    /// Set result limit
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Filter by name
    pub fn with_name(mut self, name: &str) -> Self {
        self.name_filter = Some(name.to_string());
        self
    }

    /// Set minimum relevance
    pub fn with_min_relevance(mut self, min: u8) -> Self {
        self.min_relevance = min;
        self
    }

    /// Query for man pages only
    pub fn man_page(command: &str) -> Self {
        Self::new(command)
            .with_sources(vec![DocSourceKind::ManPage])
            .with_name(command)
    }

    /// Query for Arch Wiki only
    pub fn arch_wiki(topic: &str) -> Self {
        Self::new(topic).with_sources(vec![DocSourceKind::ArchWiki])
    }

    /// Query for help output only
    pub fn tool_help(command: &str) -> Self {
        Self::new(command)
            .with_sources(vec![DocSourceKind::ToolHelp])
            .with_name(command)
    }
}

/// Result of a documentation query
#[derive(Debug, Clone, Default)]
pub struct DocResult {
    /// Matching snippets (sorted by relevance)
    pub snippets: Vec<DocSnippet>,
    /// Whether results came from cache
    pub from_cache: bool,
    /// Query time in milliseconds
    pub query_time_ms: u64,
    /// Sources that were searched
    pub sources_searched: Vec<DocSourceKind>,
}

impl DocResult {
    /// Create empty result
    pub fn empty() -> Self {
        Self::default()
    }

    /// Check if result has any snippets
    pub fn is_empty(&self) -> bool {
        self.snippets.is_empty()
    }

    /// Get best snippet (highest relevance)
    pub fn best(&self) -> Option<&DocSnippet> {
        self.snippets.first()
    }

    /// Get all citations
    pub fn citations(&self) -> Vec<String> {
        self.snippets.iter().map(|s| s.citation()).collect()
    }

    /// Merge with another result
    pub fn merge(mut self, other: DocResult) -> Self {
        self.snippets.extend(other.snippets);
        self.from_cache = self.from_cache || other.from_cache;
        self.query_time_ms += other.query_time_ms;

        for source in other.sources_searched {
            if !self.sources_searched.contains(&source) {
                self.sources_searched.push(source);
            }
        }

        // Re-sort by relevance
        self.snippets.sort_by(|a, b| b.relevance.cmp(&a.relevance));

        self
    }
}

/// Reference to documentation (for recipes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocReference {
    /// Source type
    pub source: DocSourceKind,
    /// Document/topic name
    pub name: String,
    /// Optional section
    pub section: Option<String>,
    /// Why this doc is referenced
    pub reason: Option<String>,
}

impl DocReference {
    /// Create a new doc reference
    pub fn new(source: DocSourceKind, name: &str) -> Self {
        Self {
            source,
            name: name.to_string(),
            section: None,
            reason: None,
        }
    }

    /// Set section
    pub fn with_section(mut self, section: &str) -> Self {
        self.section = Some(section.to_string());
        self
    }

    /// Set reason for reference
    pub fn with_reason(mut self, reason: &str) -> Self {
        self.reason = Some(reason.to_string());
        self
    }

    /// Generate snippet ID for lookup
    pub fn snippet_id(&self) -> String {
        DocSnippet::generate_id(self.source, &self.name, self.section.as_deref())
    }

    /// Get citation string
    pub fn citation(&self) -> String {
        self.source
            .citation_format(&self.name, self.section.as_deref())
    }

    /// Arch Wiki reference
    pub fn arch_wiki(page: &str) -> Self {
        Self::new(DocSourceKind::ArchWiki, page)
    }

    /// Man page reference
    pub fn man_page(command: &str, section: &str) -> Self {
        Self::new(DocSourceKind::ManPage, command).with_section(section)
    }

    /// Help output reference
    pub fn tool_help(command: &str) -> Self {
        Self::new(DocSourceKind::ToolHelp, command)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doc_source_priority() {
        assert!(DocSourceKind::ArchWiki.priority() < DocSourceKind::ManPage.priority());
        assert!(DocSourceKind::ManPage.priority() < DocSourceKind::ToolHelp.priority());
    }

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

    #[test]
    fn test_doc_query_builders() {
        let q = DocQuery::man_page("systemctl");
        assert_eq!(q.preferred_sources, vec![DocSourceKind::ManPage]);
        assert_eq!(q.name_filter, Some("systemctl".to_string()));

        let q = DocQuery::arch_wiki("solid state drive");
        assert_eq!(q.preferred_sources, vec![DocSourceKind::ArchWiki]);
    }

    #[test]
    fn test_doc_reference() {
        let r = DocReference::arch_wiki("pacman").with_section("Tips and tricks");
        assert_eq!(r.citation(), "Arch Wiki: pacman#Tips and tricks");

        let r = DocReference::man_page("journalctl", "1");
        assert_eq!(r.citation(), "journalctl(1)");
    }
}
