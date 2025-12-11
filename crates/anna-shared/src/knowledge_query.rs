//! Knowledge Query Interface (v0.0.414).
//!
//! The core API for querying Anna's knowledge sources.
//! Specialists use this to find relevant documentation before reasoning.
//!
//! Design principles:
//! - Doc-first: always check documentation before inventing answers
//! - No hallucination: answers must be grounded in retrieved docs
//! - Citations required: every claim needs a source reference

use crate::evidence_engine::{DocSnippet, DocSource};
use crate::knowledge::KnowledgeSource;
use serde::{Deserialize, Serialize};

/// Knowledge source kind - the type of documentation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeSourceKind {
    /// Man page (e.g., man systemctl)
    ManPage,
    /// CLI --help output
    CliHelp,
    /// Arch Wiki full page
    ArchWikiPage,
    /// Arch Wiki section
    ArchWikiSection,
    /// Local documentation file
    LocalDocFile,
    /// Probe output (system facts)
    ProbeOutput,
    /// Configuration file contents
    ConfigFile,
    /// Log file excerpt
    LogExcerpt,
    /// Built-in knowledge pack
    BuiltIn,
    /// Learned recipe
    LearnedRecipe,
}

impl KnowledgeSourceKind {
    /// Get display name for citations
    pub fn display(&self) -> &'static str {
        match self {
            Self::ManPage => "man",
            Self::CliHelp => "help",
            Self::ArchWikiPage => "Arch Wiki",
            Self::ArchWikiSection => "Arch Wiki",
            Self::LocalDocFile => "doc",
            Self::ProbeOutput => "probe",
            Self::ConfigFile => "config",
            Self::LogExcerpt => "log",
            Self::BuiltIn => "built-in",
            Self::LearnedRecipe => "recipe",
        }
    }

    /// Get priority for source ordering (lower = higher priority)
    pub fn priority(&self) -> u8 {
        match self {
            Self::ProbeOutput => 1,      // System facts first
            Self::ManPage => 2,          // Authoritative docs
            Self::ArchWikiPage => 3,     // Community docs
            Self::ArchWikiSection => 4,
            Self::CliHelp => 5,          // Quick reference
            Self::LearnedRecipe => 6,    // Learned patterns
            Self::BuiltIn => 7,          // Static knowledge
            Self::ConfigFile => 8,       // Context
            Self::LocalDocFile => 9,
            Self::LogExcerpt => 10,
        }
    }

    /// Convert from legacy KnowledgeSource
    pub fn from_legacy(source: &KnowledgeSource) -> Self {
        match source {
            KnowledgeSource::ManPage => Self::ManPage,
            KnowledgeSource::HelpOutput => Self::CliHelp,
            KnowledgeSource::ArchWiki => Self::ArchWikiPage,
            KnowledgeSource::Recipe => Self::LearnedRecipe,
            KnowledgeSource::SystemFact => Self::ProbeOutput,
            KnowledgeSource::PackageFact => Self::ProbeOutput,
            KnowledgeSource::Journal => Self::LogExcerpt,
            KnowledgeSource::BuiltIn => Self::BuiltIn,
            KnowledgeSource::AUR => Self::ArchWikiPage,
            KnowledgeSource::Usage => Self::ProbeOutput,
        }
    }
}

impl std::fmt::Display for KnowledgeSourceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}

/// A query for knowledge across all sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeQuery {
    /// Domain (network, storage, services, etc.)
    pub domain: String,
    /// Topic being searched (e.g., "boot time", "failed service")
    pub topic: String,
    /// Preferred source types (in priority order)
    pub preferred_sources: Vec<KnowledgeSourceKind>,
    /// Related commands to search for
    pub related_commands: Vec<String>,
    /// Maximum results to return
    pub max_results: usize,
    /// Minimum relevance score (0-100)
    pub min_relevance: u8,
}

impl KnowledgeQuery {
    /// Create a new knowledge query
    pub fn new(domain: &str, topic: &str) -> Self {
        Self {
            domain: domain.to_lowercase(),
            topic: topic.to_string(),
            preferred_sources: vec![
                KnowledgeSourceKind::ManPage,
                KnowledgeSourceKind::ArchWikiPage,
                KnowledgeSourceKind::CliHelp,
            ],
            related_commands: vec![],
            max_results: 5,
            min_relevance: 30,
        }
    }

    /// Builder: set preferred sources
    pub fn with_sources(mut self, sources: Vec<KnowledgeSourceKind>) -> Self {
        self.preferred_sources = sources;
        self
    }

    /// Builder: add related commands
    pub fn with_commands(mut self, commands: Vec<&str>) -> Self {
        self.related_commands = commands.into_iter().map(String::from).collect();
        self
    }

    /// Builder: set max results
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.max_results = limit;
        self
    }

    /// Builder: set min relevance
    pub fn with_min_relevance(mut self, min: u8) -> Self {
        self.min_relevance = min;
        self
    }

    /// Generate search tags from topic and commands
    pub fn search_tags(&self) -> Vec<String> {
        let mut tags = vec![self.topic.clone(), self.domain.clone()];
        tags.extend(self.related_commands.clone());
        // Add domain-specific terms
        tags.extend(domain_tags(&self.domain));
        tags
    }
}

/// A hit from knowledge search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeHit {
    /// Document ID for citation (e.g., "man:systemctl", "wiki:systemd-boot")
    pub doc_id: String,
    /// Source kind
    pub kind: KnowledgeSourceKind,
    /// Document title
    pub title: String,
    /// Origin description (e.g., "man systemctl", "Arch Wiki: systemd-boot")
    pub origin: String,
    /// Relevant excerpt
    pub excerpt: String,
    /// Relevance score (0-100)
    pub relevance: u8,
    /// Path to source file (for local docs)
    pub path: Option<String>,
}

impl KnowledgeHit {
    /// Create from a DocSnippet
    pub fn from_doc_snippet(snippet: &DocSnippet) -> Self {
        let kind = match snippet.source {
            DocSource::ManPage => KnowledgeSourceKind::ManPage,
            DocSource::HelpOutput => KnowledgeSourceKind::CliHelp,
            DocSource::ArchWiki => KnowledgeSourceKind::ArchWikiPage,
            DocSource::LocalDoc => KnowledgeSourceKind::LocalDocFile,
            DocSource::ConfigFile => KnowledgeSourceKind::ConfigFile,
        };
        Self {
            doc_id: snippet.id.clone(),
            kind,
            title: snippet.title.clone(),
            origin: format_origin(&snippet.source, &snippet.title),
            excerpt: snippet.snippet.clone(),
            relevance: snippet.relevance,
            path: Some(snippet.location.clone()),
        }
    }

    /// Format for citation display
    pub fn citation_display(&self) -> String {
        self.origin.clone()
    }

    /// Format for short reference
    pub fn citation_ref(&self) -> String {
        format!("[{}]", self.doc_id)
    }
}

/// Format origin string for display
fn format_origin(source: &DocSource, title: &str) -> String {
    match source {
        DocSource::ManPage => format!("man {}", title.trim_start_matches("man ")),
        DocSource::HelpOutput => format!("{} --help", title.trim_end_matches(" --help")),
        DocSource::ArchWiki => format!("Arch Wiki: {}", title.trim_start_matches("Arch Wiki: ")),
        DocSource::LocalDoc => format!("doc: {}", title.trim_start_matches("doc: ")),
        DocSource::ConfigFile => format!("config: {}", title),
    }
}

/// Get domain-specific tags for search expansion
fn domain_tags(domain: &str) -> Vec<String> {
    match domain {
        "services" | "systemd" => vec![
            "systemctl".into(), "systemd".into(), "service".into(), "unit".into()
        ],
        "network" => vec![
            "ip".into(), "networkctl".into(), "nmcli".into(), "interface".into()
        ],
        "storage" => vec![
            "df".into(), "mount".into(), "lsblk".into(), "disk".into(), "filesystem".into()
        ],
        "packages" => vec![
            "pacman".into(), "yay".into(), "paru".into(), "package".into()
        ],
        "boot" => vec![
            "systemd-analyze".into(), "bootloader".into(), "grub".into(), "systemd-boot".into()
        ],
        "audio" => vec![
            "pipewire".into(), "pulseaudio".into(), "alsa".into(), "pactl".into()
        ],
        "desktop" => vec![
            "hyprland".into(), "sway".into(), "kde".into(), "gnome".into()
        ],
        "security" => vec![
            "ufw".into(), "iptables".into(), "ssh".into(), "firewall".into()
        ],
        _ => vec![],
    }
}

/// Result of a knowledge query
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KnowledgeResult {
    /// Hits found
    pub hits: Vec<KnowledgeHit>,
    /// Sources that were searched
    pub sources_searched: Vec<KnowledgeSourceKind>,
    /// Query execution time (ms)
    pub query_time_ms: u64,
    /// Whether wiki was available
    pub wiki_available: bool,
}

impl KnowledgeResult {
    /// Create empty result
    pub fn empty() -> Self {
        Self::default()
    }

    /// Check if any results found
    pub fn has_results(&self) -> bool {
        !self.hits.is_empty()
    }

    /// Get top hit
    pub fn top_hit(&self) -> Option<&KnowledgeHit> {
        self.hits.first()
    }

    /// Get all doc IDs for citation
    pub fn all_doc_ids(&self) -> Vec<String> {
        self.hits.iter().map(|h| h.doc_id.clone()).collect()
    }

    /// Format citations line
    pub fn format_citations(&self) -> String {
        if self.hits.is_empty() {
            return String::new();
        }
        let citations: Vec<String> = self.hits.iter()
            .map(|h| h.citation_display())
            .collect();
        format!("Evidence: {}", citations.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_query_creation() {
        let query = KnowledgeQuery::new("services", "failed systemd service")
            .with_commands(vec!["systemctl", "journalctl"])
            .with_limit(10);

        assert_eq!(query.domain, "services");
        assert_eq!(query.related_commands.len(), 2);
        assert_eq!(query.max_results, 10);
    }

    #[test]
    fn test_search_tags() {
        let query = KnowledgeQuery::new("services", "failed service")
            .with_commands(vec!["systemctl"]);

        let tags = query.search_tags();
        assert!(tags.contains(&"failed service".to_string()));
        assert!(tags.contains(&"services".to_string()));
        assert!(tags.contains(&"systemctl".to_string()));
    }

    #[test]
    fn test_knowledge_source_kind_priority() {
        assert!(KnowledgeSourceKind::ProbeOutput.priority() < KnowledgeSourceKind::ManPage.priority());
        assert!(KnowledgeSourceKind::ManPage.priority() < KnowledgeSourceKind::ArchWikiPage.priority());
    }

    #[test]
    fn test_knowledge_hit_citation() {
        let hit = KnowledgeHit {
            doc_id: "man:systemctl".to_string(),
            kind: KnowledgeSourceKind::ManPage,
            title: "systemctl".to_string(),
            origin: "man systemctl".to_string(),
            excerpt: "Control the systemd system and service manager".to_string(),
            relevance: 90,
            path: None,
        };

        assert_eq!(hit.citation_display(), "man systemctl");
        assert_eq!(hit.citation_ref(), "[man:systemctl]");
    }
}
