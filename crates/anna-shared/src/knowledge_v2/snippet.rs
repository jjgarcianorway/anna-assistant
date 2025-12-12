//! Knowledge snippet data model (v0.0.422).
//!
//! Canonical struct for knowledge attached to tickets.
//! Specialists never see raw text blobs - only structured snippets.

use serde::{Deserialize, Serialize};

/// A normalized knowledge snippet attached to a ticket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeSnippet {
    /// Unique ID within the ticket (e.g., "k1", "k2")
    pub id: String,

    /// Knowledge source type
    pub source: KnowledgeSource,

    /// Title (e.g., "systemd-analyze(1)", "Arch Wiki: Systemd/Bootchart")
    pub title: String,

    /// Section within the doc (for wiki/man)
    #[serde(default)]
    pub section: Option<String>,

    /// Relevance to current ticket (0.0 - 1.0)
    #[serde(default = "default_relevance")]
    pub relevance: f32,

    /// Short summary (2-4 sentences, LLM or heuristic generated)
    pub summary: String,

    /// Key points (3-7 bullets max)
    #[serde(default)]
    pub key_points: Vec<String>,

    /// Truncated raw text excerpt (max 2k chars)
    pub raw_excerpt: String,

    /// Citations (e.g., ["man:systemd-analyze(1)", "archwiki:Systemd/Journal"])
    #[serde(default)]
    pub citations: Vec<String>,
}

fn default_relevance() -> f32 {
    0.5
}

impl KnowledgeSnippet {
    /// Create a new snippet with required fields
    pub fn new(id: &str, source: KnowledgeSource, title: &str) -> Self {
        Self {
            id: id.to_string(),
            source,
            title: title.to_string(),
            section: None,
            relevance: 0.5,
            summary: String::new(),
            key_points: vec![],
            raw_excerpt: String::new(),
            citations: vec![],
        }
    }

    /// Builder: set section
    pub fn with_section(mut self, section: &str) -> Self {
        self.section = Some(section.to_string());
        self
    }

    /// Builder: set relevance
    pub fn with_relevance(mut self, relevance: f32) -> Self {
        self.relevance = relevance.clamp(0.0, 1.0);
        self
    }

    /// Builder: set summary
    pub fn with_summary(mut self, summary: &str) -> Self {
        self.summary = truncate(summary, super::MAX_SUMMARY_LENGTH);
        self
    }

    /// Builder: add key point
    pub fn with_key_point(mut self, point: &str) -> Self {
        if self.key_points.len() < super::MAX_KEY_POINTS {
            self.key_points.push(point.to_string());
        }
        self
    }

    /// Builder: set key points
    pub fn with_key_points(mut self, points: Vec<String>) -> Self {
        self.key_points = points.into_iter().take(super::MAX_KEY_POINTS).collect();
        self
    }

    /// Builder: set raw excerpt (auto-truncated)
    pub fn with_excerpt(mut self, excerpt: &str) -> Self {
        self.raw_excerpt = truncate(excerpt, super::MAX_EXCERPT_LENGTH);
        self
    }

    /// Builder: add citation
    pub fn with_citation(mut self, citation: &str) -> Self {
        self.citations.push(citation.to_string());
        self
    }

    /// Builder: set citations
    pub fn with_citations(mut self, citations: Vec<String>) -> Self {
        self.citations = citations;
        self
    }

    /// Create from man page output
    pub fn from_man(id: &str, command: &str, content: &str) -> Self {
        let title = format!("{}(1)", command);
        let citation = format!("man:{}(1)", command);

        Self::new(id, KnowledgeSource::ManPage, &title)
            .with_excerpt(content)
            .with_citation(&citation)
            .with_relevance(0.8)
    }

    /// Create from help output
    pub fn from_help(id: &str, command: &str, content: &str) -> Self {
        let title = format!("{} --help", command);
        let citation = format!("help:{}", command);

        Self::new(id, KnowledgeSource::Help, &title)
            .with_excerpt(content)
            .with_citation(&citation)
            .with_relevance(0.7)
    }

    /// Create from Arch Wiki
    pub fn from_wiki(id: &str, page: &str, section: Option<&str>, content: &str) -> Self {
        let title = if let Some(sec) = section {
            format!("Arch Wiki: {}#{}", page, sec)
        } else {
            format!("Arch Wiki: {}", page)
        };

        let citation = if let Some(sec) = section {
            format!("archwiki:{}#{}", page, sec)
        } else {
            format!("archwiki:{}", page)
        };

        let mut snippet = Self::new(id, KnowledgeSource::ArchWiki, &title)
            .with_excerpt(content)
            .with_citation(&citation)
            .with_relevance(0.75);

        if let Some(sec) = section {
            snippet = snippet.with_section(sec);
        }

        snippet
    }

    /// Create from local doc
    pub fn from_local_doc(id: &str, path: &str, content: &str) -> Self {
        let title = path.split('/').last().unwrap_or(path).to_string();
        let citation = format!("doc:{}", path);

        Self::new(id, KnowledgeSource::LocalDoc, &title)
            .with_excerpt(content)
            .with_citation(&citation)
            .with_relevance(0.6)
    }

    /// Create from pacman metadata
    pub fn from_pacman(id: &str, package: &str, content: &str) -> Self {
        let title = format!("pacman: {}", package);
        let citation = format!("pacman:{}", package);

        Self::new(id, KnowledgeSource::PacmanDoc, &title)
            .with_excerpt(content)
            .with_citation(&citation)
            .with_relevance(0.7)
    }

    /// Get primary citation (first one or generated)
    pub fn primary_citation(&self) -> String {
        self.citations
            .first()
            .cloned()
            .unwrap_or_else(|| format!("{}:{}", self.source.prefix(), self.title))
    }

    /// Check if snippet has useful content
    pub fn has_content(&self) -> bool {
        !self.raw_excerpt.is_empty() || !self.summary.is_empty()
    }
}

/// Knowledge source type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeSource {
    /// Man pages (highest priority for command docs)
    ManPage,
    /// Command help output (--help, -h)
    Help,
    /// Arch Wiki (official community docs)
    ArchWiki,
    /// Pacman package metadata
    PacmanDoc,
    /// Local documentation (/usr/share/doc, etc.)
    LocalDoc,
    /// Other official docs
    #[default]
    OtherOfficial,
}

impl KnowledgeSource {
    /// Get priority (lower = higher priority)
    pub fn priority(&self) -> u8 {
        match self {
            Self::ManPage => 1,
            Self::Help => 2,
            Self::PacmanDoc => 3,
            Self::ArchWiki => 4,
            Self::LocalDoc => 5,
            Self::OtherOfficial => 6,
        }
    }

    /// Get citation prefix
    pub fn prefix(&self) -> &'static str {
        match self {
            Self::ManPage => "man",
            Self::Help => "help",
            Self::ArchWiki => "archwiki",
            Self::PacmanDoc => "pacman",
            Self::LocalDoc => "doc",
            Self::OtherOfficial => "doc",
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::ManPage => "man page",
            Self::Help => "command help",
            Self::ArchWiki => "Arch Wiki",
            Self::PacmanDoc => "pacman",
            Self::LocalDoc => "local doc",
            Self::OtherOfficial => "documentation",
        }
    }
}

/// Truncate string to max length
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snippet_builder() {
        let snippet = KnowledgeSnippet::new("k1", KnowledgeSource::ManPage, "systemctl(1)")
            .with_summary("Control the systemd system and service manager")
            .with_key_point("Start services: systemctl start <service>")
            .with_key_point("Stop services: systemctl stop <service>")
            .with_excerpt("NAME\n    systemctl - Control the systemd system...")
            .with_citation("man:systemctl(1)")
            .with_relevance(0.9);

        assert_eq!(snippet.id, "k1");
        assert_eq!(snippet.source, KnowledgeSource::ManPage);
        assert_eq!(snippet.key_points.len(), 2);
        assert!(snippet.has_content());
    }

    #[test]
    fn test_from_man() {
        let snippet = KnowledgeSnippet::from_man("k1", "systemctl", "NAME\n    systemctl...");
        assert_eq!(snippet.source, KnowledgeSource::ManPage);
        assert_eq!(snippet.title, "systemctl(1)");
        assert_eq!(snippet.citations, vec!["man:systemctl(1)"]);
    }

    #[test]
    fn test_from_wiki() {
        let snippet =
            KnowledgeSnippet::from_wiki("k1", "Systemd", Some("Services"), "Services section...");
        assert_eq!(snippet.source, KnowledgeSource::ArchWiki);
        assert!(snippet.title.contains("Systemd#Services"));
        assert!(snippet.citations[0].contains("archwiki:Systemd#Services"));
    }

    #[test]
    fn test_source_priority() {
        assert!(KnowledgeSource::ManPage.priority() < KnowledgeSource::ArchWiki.priority());
        assert!(KnowledgeSource::Help.priority() < KnowledgeSource::LocalDoc.priority());
    }

    #[test]
    fn test_truncation() {
        let long_text = "a".repeat(3000);
        let snippet =
            KnowledgeSnippet::new("k1", KnowledgeSource::ManPage, "test").with_excerpt(&long_text);
        assert!(snippet.raw_excerpt.len() <= super::super::MAX_EXCERPT_LENGTH);
    }
}
