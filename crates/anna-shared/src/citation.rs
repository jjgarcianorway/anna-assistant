//! Local-only citation support (v0.0.75).
//!
//! Attaches citations to answers in teaching mode from local sources:
//! - man pages
//! - command --help output
//! - local Arch Wiki snapshots

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Citation source types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CitationSource {
    /// man page (e.g., man free)
    ManPage { command: String, section: Option<u8> },
    /// --help output
    HelpOutput { command: String },
    /// Local Arch Wiki snapshot
    ArchWiki { article: String },
    /// Internal Anna knowledge
    Internal { topic: String },
}

impl CitationSource {
    pub fn display(&self) -> String {
        match self {
            Self::ManPage { command, section } => {
                if let Some(s) = section {
                    format!("man {}({})", command, s)
                } else {
                    format!("man {}", command)
                }
            }
            Self::HelpOutput { command } => format!("{} --help", command),
            Self::ArchWiki { article } => format!("Arch Wiki: {}", article),
            Self::Internal { topic } => format!("Anna: {}", topic),
        }
    }

    pub fn reference(&self) -> String {
        match self {
            Self::ManPage { command, section } => {
                if let Some(s) = section {
                    format!("[man {}({})]", command, s)
                } else {
                    format!("[man {}]", command)
                }
            }
            Self::HelpOutput { command } => format!("[{} --help]", command),
            Self::ArchWiki { article } => format!("[Wiki:{}]", article),
            Self::Internal { topic } => format!("[{}]", topic),
        }
    }
}

/// A single citation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    /// Source of the citation
    pub source: CitationSource,
    /// Relevant excerpt (max 200 chars)
    pub excerpt: String,
    /// Relevance score (0-100)
    pub relevance: u8,
}

impl Citation {
    pub fn new(source: CitationSource, excerpt: &str) -> Self {
        Self {
            source,
            excerpt: truncate_excerpt(excerpt, 200),
            relevance: 50,
        }
    }

    pub fn with_relevance(mut self, relevance: u8) -> Self {
        self.relevance = relevance.min(100);
        self
    }

    /// Format for display in answer
    pub fn format_inline(&self) -> String {
        self.source.reference()
    }

    /// Format for citation list
    pub fn format_full(&self) -> String {
        format!("{}: \"{}\"", self.source.display(), self.excerpt)
    }
}

/// Truncate excerpt to max length, preserving word boundaries
fn truncate_excerpt(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        return text.to_string();
    }

    let truncated = &text[..max_len];
    // Find last space
    if let Some(last_space) = truncated.rfind(' ') {
        format!("{}...", &truncated[..last_space])
    } else {
        format!("{}...", truncated)
    }
}

/// Citation collection for an answer
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CitationSet {
    /// Citations by source reference
    pub citations: Vec<Citation>,
    /// Whether to include citations in output
    pub enabled: bool,
}

impl CitationSet {
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable citations (teaching mode)
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable citations (minimal mode)
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Add a citation
    pub fn add(&mut self, citation: Citation) {
        // Avoid duplicates
        let ref_str = citation.source.reference();
        if !self.citations.iter().any(|c| c.source.reference() == ref_str) {
            self.citations.push(citation);
        }
    }

    /// Add a man page citation
    pub fn cite_man(&mut self, command: &str, excerpt: &str) {
        self.add(Citation::new(
            CitationSource::ManPage {
                command: command.to_string(),
                section: None,
            },
            excerpt,
        ));
    }

    /// Add a --help citation
    pub fn cite_help(&mut self, command: &str, excerpt: &str) {
        self.add(Citation::new(
            CitationSource::HelpOutput {
                command: command.to_string(),
            },
            excerpt,
        ));
    }

    /// Add a wiki citation
    pub fn cite_wiki(&mut self, article: &str, excerpt: &str) {
        self.add(Citation::new(
            CitationSource::ArchWiki {
                article: article.to_string(),
            },
            excerpt,
        ));
    }

    /// Add internal citation
    pub fn cite_internal(&mut self, topic: &str, excerpt: &str) {
        self.add(Citation::new(
            CitationSource::Internal {
                topic: topic.to_string(),
            },
            excerpt,
        ));
    }

    /// Get inline references for answer text
    pub fn inline_refs(&self) -> String {
        if !self.enabled || self.citations.is_empty() {
            return String::new();
        }

        let refs: Vec<String> = self.citations.iter().map(|c| c.format_inline()).collect();
        refs.join(" ")
    }

    /// Format citations section for answer footer
    pub fn format_footer(&self) -> Option<String> {
        if !self.enabled || self.citations.is_empty() {
            return None;
        }

        let mut lines = vec!["Sources:".to_string()];
        for citation in &self.citations {
            lines.push(format!("  • {}", citation.format_full()));
        }
        Some(lines.join("\n"))
    }

    /// Check if has any citations
    pub fn has_citations(&self) -> bool {
        !self.citations.is_empty()
    }

    /// Count citations
    pub fn len(&self) -> usize {
        self.citations.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.citations.is_empty()
    }
}

/// Local citation store with cached man pages and wiki snapshots
pub struct CitationStore {
    /// Path to wiki snapshots directory
    wiki_dir: PathBuf,
    /// Cached man page excerpts
    man_cache: HashMap<String, String>,
    /// Cached help output excerpts
    help_cache: HashMap<String, String>,
}

impl CitationStore {
    pub fn new() -> Self {
        Self {
            wiki_dir: PathBuf::from("/var/lib/anna/wiki"),
            man_cache: HashMap::new(),
            help_cache: HashMap::new(),
        }
    }

    /// Set wiki directory
    pub fn with_wiki_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.wiki_dir = dir.into();
        self
    }

    /// Get wiki snapshot path for an article
    pub fn wiki_path(&self, article: &str) -> PathBuf {
        let safe_name = article.replace(['/', '\\', ':'], "_");
        self.wiki_dir.join(format!("{}.txt", safe_name))
    }

    /// Load wiki snapshot if it exists locally
    pub fn load_wiki(&self, article: &str) -> Option<String> {
        let path = self.wiki_path(article);
        std::fs::read_to_string(path).ok()
    }

    /// Get or cache man page excerpt
    pub fn get_man_excerpt(&mut self, command: &str) -> Option<String> {
        if let Some(cached) = self.man_cache.get(command) {
            return Some(cached.clone());
        }

        // Try to get from system (but don't run commands - use cached only)
        // In a real implementation, we'd cache man page output during idle time
        None
    }

    /// Cache a man page excerpt
    pub fn cache_man(&mut self, command: &str, excerpt: &str) {
        self.man_cache
            .insert(command.to_string(), excerpt.to_string());
    }

    /// Cache a help output excerpt
    pub fn cache_help(&mut self, command: &str, excerpt: &str) {
        self.help_cache
            .insert(command.to_string(), excerpt.to_string());
    }

    /// Get cached help excerpt
    pub fn get_help_excerpt(&self, command: &str) -> Option<&String> {
        self.help_cache.get(command)
    }
}

impl Default for CitationStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_citation_source_display() {
        let man = CitationSource::ManPage {
            command: "free".to_string(),
            section: Some(1),
        };
        assert_eq!(man.display(), "man free(1)");
        assert_eq!(man.reference(), "[man free(1)]");

        let help = CitationSource::HelpOutput {
            command: "df".to_string(),
        };
        assert_eq!(help.display(), "df --help");
        assert_eq!(help.reference(), "[df --help]");
    }

    #[test]
    fn test_citation_set_enabled() {
        let mut set = CitationSet::new();
        set.cite_man("free", "Display free and used memory");
        set.enable();

        assert!(set.enabled);
        assert!(!set.inline_refs().is_empty());
        assert!(set.format_footer().is_some());
    }

    #[test]
    fn test_citation_set_disabled() {
        let mut set = CitationSet::new();
        set.cite_man("free", "Display free and used memory");
        // Not enabled by default

        assert!(!set.enabled);
        assert!(set.inline_refs().is_empty());
        assert!(set.format_footer().is_none());
    }

    #[test]
    fn test_truncate_excerpt() {
        let short = "Short text";
        assert_eq!(truncate_excerpt(short, 100), "Short text");

        let long = "This is a very long text that should be truncated at word boundaries";
        let truncated = truncate_excerpt(long, 30);
        assert!(truncated.ends_with("..."));
        assert!(truncated.len() <= 33); // 30 + "..."
    }

    #[test]
    fn test_citation_set_no_duplicates() {
        let mut set = CitationSet::new();
        set.cite_man("free", "First excerpt");
        set.cite_man("free", "Second excerpt"); // Same command

        assert_eq!(set.len(), 1); // Should not duplicate
    }
}
