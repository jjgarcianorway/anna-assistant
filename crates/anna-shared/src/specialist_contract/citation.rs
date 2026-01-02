//! Citation types for knowledge source provenance.

use serde::{Deserialize, Serialize};

/// Citation from a knowledge source (v0.0.419)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeCitation {
    /// Citation ID for provenance (e.g., "man:systemctl:line42-50")
    pub citation_id: String,
    /// Kind of source (man, help, wiki, doc)
    pub kind: CitationKind,
    /// Human-readable title
    pub title: String,
    /// Relevant excerpt that was used
    pub excerpt: String,
    /// Relevance score (0-100)
    #[serde(default)]
    pub relevance: u8,
}

impl KnowledgeCitation {
    /// Create a new citation
    pub fn new(citation_id: &str, kind: CitationKind, title: &str, excerpt: &str) -> Self {
        Self {
            citation_id: citation_id.to_string(),
            kind,
            title: title.to_string(),
            excerpt: excerpt.to_string(),
            relevance: 80,
        }
    }

    /// Format as inline reference (e.g., "[man systemctl(1)]")
    pub fn inline_ref(&self) -> String {
        match self.kind {
            CitationKind::ManPage => format!("[man {}]", self.title),
            CitationKind::CliHelp => format!("[{} --help]", self.title),
            CitationKind::ArchWiki => format!("[wiki:{}]", self.title),
            CitationKind::LocalDoc => format!("[doc:{}]", self.title),
            CitationKind::Internal => format!("[{}]", self.title),
        }
    }

    /// Format for citation footer
    pub fn footer_display(&self) -> String {
        let kind_str = match self.kind {
            CitationKind::ManPage => "man page",
            CitationKind::CliHelp => "command help",
            CitationKind::ArchWiki => "Arch Wiki",
            CitationKind::LocalDoc => "local doc",
            CitationKind::Internal => "internal",
        };
        format!(
            "{} ({}): \"{}\"",
            self.title,
            kind_str,
            truncate_str(&self.excerpt, 100)
        )
    }
}

/// Kind of citation source
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CitationKind {
    ManPage,
    CliHelp,
    ArchWiki,
    LocalDoc,
    Internal,
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}
