//! Evidence types (probes, docs, recipes)

use serde::{Deserialize, Serialize};

use super::utils::current_millis;

/// Evidence from a probe (system command)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeEvidence {
    /// Unique ID (e.g., "probe:df_root")
    pub id: String,
    /// Short human summary
    pub summary: String,
    /// Relevant excerpt (compact)
    pub excerpt: String,
    /// Reference to raw output if needed
    pub raw_ref: Option<String>,
    /// Command that was run
    pub command: String,
    /// Exit code
    pub exit_code: i32,
    /// Timestamp
    pub timestamp: u64,
}

impl ProbeEvidence {
    pub fn new(id: &str, command: &str, summary: &str, excerpt: &str) -> Self {
        Self {
            id: id.to_string(),
            summary: summary.to_string(),
            excerpt: excerpt.to_string(),
            raw_ref: None,
            command: command.to_string(),
            exit_code: 0,
            timestamp: current_millis(),
        }
    }

    pub fn with_exit_code(mut self, code: i32) -> Self {
        self.exit_code = code;
        self
    }
}

/// Documentation snippet from authoritative source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocSnippet {
    /// Unique ID (e.g., "doc:arch:fancontrol", "man:systemd.service")
    pub id: String,
    /// Human-readable title
    pub title: String,
    /// Source type
    pub source: DocSource,
    /// Relevant text snippet
    pub snippet: String,
    /// Location reference (URL, man section, file path)
    pub location: String,
    /// Relevance score (0-100)
    pub relevance: u8,
}

/// Documentation source type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocSource {
    ArchWiki,
    ManPage,
    HelpOutput,
    LocalDoc,
    ConfigFile,
}

impl std::fmt::Display for DocSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ArchWiki => write!(f, "arch_wiki"),
            Self::ManPage => write!(f, "man"),
            Self::HelpOutput => write!(f, "help"),
            Self::LocalDoc => write!(f, "doc"),
            Self::ConfigFile => write!(f, "config"),
        }
    }
}

impl DocSnippet {
    pub fn new(source: DocSource, title: &str, snippet: &str, location: &str) -> Self {
        let id = format!("{}:{}", source, title.to_lowercase().replace(' ', "_"));
        Self {
            id,
            title: title.to_string(),
            source,
            snippet: super::utils::truncate_snippet(snippet, 500),
            location: location.to_string(),
            relevance: 50,
        }
    }

    pub fn with_relevance(mut self, relevance: u8) -> Self {
        self.relevance = relevance.min(100);
        self
    }
}

/// A matching recipe candidate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeMatch {
    /// Recipe ID
    pub id: String,
    /// Recipe title
    pub title: String,
    /// Short summary
    pub summary: String,
    /// Confidence percentage
    pub confidence: u8,
    /// Required actions
    pub actions: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doc_snippet() {
        let doc = DocSnippet::new(
            DocSource::ManPage,
            "systemd.service",
            "A service unit file...",
            "man systemd.service(5)",
        );

        assert!(doc.id.starts_with("man:"));
        assert_eq!(doc.source, DocSource::ManPage);
    }
}
