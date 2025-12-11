//! Knowledge snippets and results (v0.0.424).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::query::KnowledgeSource;

/// A snippet of knowledge from a source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeSnippet {
    /// Source type (ManPage, CommandHelp, etc.)
    pub source: KnowledgeSource,

    /// Human-readable title (e.g., "man vim", "pacman(8)")
    pub title: String,

    /// Extracted text excerpt
    pub excerpt: String,

    /// Stable citation ID (e.g., "man:vim", "help:pacman")
    pub citation_id: String,

    /// Underlying file path (if applicable)
    pub path: Option<PathBuf>,

    /// Command used to retrieve this (e.g., "man vim")
    pub command: Option<String>,

    /// Section name within the source (e.g., "DESCRIPTION")
    pub section: Option<String>,

    /// Relevance score (0.0 to 1.0)
    pub relevance: f32,
}

impl KnowledgeSnippet {
    /// Create a new snippet
    pub fn new(source: KnowledgeSource, title: &str, excerpt: &str) -> Self {
        let citation_id = format!("{}:{}", source.citation_prefix(), sanitize_for_citation(title));

        Self {
            source,
            title: title.to_string(),
            excerpt: excerpt.to_string(),
            citation_id,
            path: None,
            command: None,
            section: None,
            relevance: 0.5,
        }
    }

    /// Create from man page
    pub fn from_man(command: &str, excerpt: &str) -> Self {
        Self {
            source: KnowledgeSource::ManPage,
            title: format!("man {}", command),
            excerpt: excerpt.to_string(),
            citation_id: format!("man:{}", command),
            path: None,
            command: Some(format!("man {}", command)),
            section: None,
            relevance: 0.8,
        }
    }

    /// Create from help output
    pub fn from_help(command: &str, excerpt: &str) -> Self {
        Self {
            source: KnowledgeSource::CommandHelp,
            title: format!("{} --help", command),
            excerpt: excerpt.to_string(),
            citation_id: format!("help:{}", command),
            path: None,
            command: Some(format!("{} --help", command)),
            section: None,
            relevance: 0.7,
        }
    }

    /// Create from local doc
    pub fn from_doc(name: &str, path: &PathBuf, excerpt: &str) -> Self {
        Self {
            source: KnowledgeSource::LocalDocs,
            title: format!("doc:{}", name),
            excerpt: excerpt.to_string(),
            citation_id: format!("doc:{}", name),
            path: Some(path.clone()),
            command: None,
            section: None,
            relevance: 0.6,
        }
    }

    /// Create from wiki
    pub fn from_wiki(topic: &str, excerpt: &str) -> Self {
        Self {
            source: KnowledgeSource::ArchWiki,
            title: format!("Arch Wiki: {}", topic),
            excerpt: excerpt.to_string(),
            citation_id: format!("wiki:{}", sanitize_for_citation(topic)),
            path: None,
            command: None,
            section: None,
            relevance: 0.75,
        }
    }

    /// Set section
    pub fn with_section(mut self, section: &str) -> Self {
        self.section = Some(section.to_string());
        self
    }

    /// Set relevance
    pub fn with_relevance(mut self, relevance: f32) -> Self {
        self.relevance = relevance.clamp(0.0, 1.0);
        self
    }

    /// Set path
    pub fn with_path(mut self, path: PathBuf) -> Self {
        self.path = Some(path);
        self
    }

    /// Get a short reference for display
    pub fn short_ref(&self) -> String {
        match self.source {
            KnowledgeSource::ManPage => format!("man:{}", self.title.strip_prefix("man ").unwrap_or(&self.title)),
            KnowledgeSource::CommandHelp => format!("help:{}", self.title.split_whitespace().next().unwrap_or(&self.title)),
            KnowledgeSource::LocalDocs => self.citation_id.clone(),
            KnowledgeSource::ArchWiki => self.citation_id.clone(),
        }
    }

    /// Truncate excerpt to max chars
    pub fn truncate(&mut self, max_chars: usize) {
        if self.excerpt.len() > max_chars {
            // Try to truncate at a sentence boundary
            let truncated = &self.excerpt[..max_chars];
            if let Some(pos) = truncated.rfind(|c| c == '.' || c == '\n') {
                self.excerpt = format!("{}", &self.excerpt[..=pos]);
            } else {
                self.excerpt = format!("{}...", truncated.trim_end());
            }
        }
    }
}

/// Result of a knowledge query
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KnowledgeResult {
    /// Retrieved snippets, ordered by relevance
    pub snippets: Vec<KnowledgeSnippet>,

    /// Sources that were queried
    pub sources_queried: Vec<KnowledgeSource>,

    /// Sources that returned results
    pub sources_with_results: Vec<KnowledgeSource>,

    /// Query duration in milliseconds
    pub duration_ms: u64,

    /// Any warnings or notes
    pub notes: Vec<String>,
}

impl KnowledgeResult {
    /// Create empty result
    pub fn empty() -> Self {
        Self::default()
    }

    /// Check if any knowledge was found
    pub fn has_knowledge(&self) -> bool {
        !self.snippets.is_empty()
    }

    /// Get primary snippet (highest relevance)
    pub fn primary(&self) -> Option<&KnowledgeSnippet> {
        self.snippets.first()
    }

    /// Get all citation IDs
    pub fn citations(&self) -> Vec<&str> {
        self.snippets.iter().map(|s| s.citation_id.as_str()).collect()
    }

    /// Get snippets from a specific source
    pub fn from_source(&self, source: KnowledgeSource) -> Vec<&KnowledgeSnippet> {
        self.snippets.iter().filter(|s| s.source == source).collect()
    }

    /// Add a snippet
    pub fn add_snippet(&mut self, snippet: KnowledgeSnippet) {
        if !self.sources_with_results.contains(&snippet.source) {
            self.sources_with_results.push(snippet.source);
        }
        self.snippets.push(snippet);
    }

    /// Sort snippets by relevance
    pub fn sort_by_relevance(&mut self) {
        self.snippets.sort_by(|a, b| {
            b.relevance.partial_cmp(&a.relevance).unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /// Truncate to max results
    pub fn truncate(&mut self, max: usize) {
        self.snippets.truncate(max);
    }

    /// Add a note
    pub fn add_note(&mut self, note: &str) {
        self.notes.push(note.to_string());
    }

    /// Format for specialist context
    pub fn format_for_specialist(&self) -> String {
        if self.snippets.is_empty() {
            return String::new();
        }

        let mut output = String::from("=== Knowledge Context ===\n\n");

        for (i, snippet) in self.snippets.iter().enumerate() {
            output.push_str(&format!(
                "[{}] {} ({})\n{}\n\n",
                i + 1,
                snippet.title,
                snippet.citation_id,
                snippet.excerpt
            ));
        }

        output.push_str("=== End Knowledge Context ===\n");
        output
    }

    /// Format citations for user display
    pub fn format_citations(&self) -> String {
        if self.snippets.is_empty() {
            return String::new();
        }

        self.snippets
            .iter()
            .map(|s| format!("- {}", s.citation_id))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Sanitize a string for use in citation IDs
fn sanitize_for_citation(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c.to_ascii_lowercase()
            } else if c.is_whitespace() {
                '-'
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches(|c| c == '-' || c == '_')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snippet_from_man() {
        let snippet = KnowledgeSnippet::from_man("vim", "Vim is a text editor.");
        assert_eq!(snippet.source, KnowledgeSource::ManPage);
        assert_eq!(snippet.citation_id, "man:vim");
        assert_eq!(snippet.title, "man vim");
    }

    #[test]
    fn test_snippet_from_help() {
        let snippet = KnowledgeSnippet::from_help("pacman", "Usage: pacman <operation> [...]");
        assert_eq!(snippet.source, KnowledgeSource::CommandHelp);
        assert_eq!(snippet.citation_id, "help:pacman");
    }

    #[test]
    fn test_snippet_from_wiki() {
        let snippet = KnowledgeSnippet::from_wiki("Systemd", "systemd is a system and service manager.");
        assert_eq!(snippet.citation_id, "wiki:systemd");
    }

    #[test]
    fn test_result_citations() {
        let mut result = KnowledgeResult::empty();
        result.add_snippet(KnowledgeSnippet::from_man("vim", "text"));
        result.add_snippet(KnowledgeSnippet::from_help("pacman", "text"));

        let citations = result.citations();
        assert!(citations.contains(&"man:vim"));
        assert!(citations.contains(&"help:pacman"));
    }

    #[test]
    fn test_truncate_snippet() {
        let mut snippet = KnowledgeSnippet::from_man("test", "This is a long text. It has multiple sentences. And more content here.");
        snippet.truncate(30);
        assert!(snippet.excerpt.len() <= 35); // Allow for "..."
    }

    #[test]
    fn test_sanitize_for_citation() {
        assert_eq!(sanitize_for_citation("Arch Wiki"), "arch-wiki");
        assert_eq!(sanitize_for_citation("systemd/User"), "systemd_user");
        assert_eq!(sanitize_for_citation("vim-8.2"), "vim-8_2");
    }
}
