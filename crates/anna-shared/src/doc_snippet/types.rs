//! Documentation snippet types and core structures.

use serde::{Deserialize, Serialize};

use super::utils::compute_snippet_id;
use super::utils::current_secs;

/// Documentation source kind
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocSourceKind {
    /// Arch Wiki page
    ArchWiki,
    /// Man page
    ManPage,
    /// Command --help output
    HelpFlag,
    /// Info page
    Info,
    /// Local file (config, etc.)
    LocalFile,
    /// Built-in knowledge
    Builtin,
}

impl std::fmt::Display for DocSourceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ArchWiki => write!(f, "arch_wiki"),
            Self::ManPage => write!(f, "man"),
            Self::HelpFlag => write!(f, "help"),
            Self::Info => write!(f, "info"),
            Self::LocalFile => write!(f, "file"),
            Self::Builtin => write!(f, "builtin"),
        }
    }
}

/// A documentation snippet used to support a recipe/answer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocSnippet {
    /// Unique ID
    pub id: String,
    /// Source kind
    pub kind: DocSourceKind,
    /// Reference (man page name, wiki URL, file path)
    pub reference: String,
    /// Section/heading within the doc
    pub section: Option<String>,
    /// Short excerpt used
    pub excerpt: String,
    /// When this was retrieved
    pub retrieved_at: u64,
    /// Relevance score (0.0-1.0)
    pub relevance: f32,
}

impl DocSnippet {
    /// Create a new doc snippet
    pub fn new(kind: DocSourceKind, reference: &str, excerpt: &str) -> Self {
        Self {
            id: compute_snippet_id(kind, reference),
            kind,
            reference: reference.to_string(),
            section: None,
            excerpt: excerpt.to_string(),
            retrieved_at: current_secs(),
            relevance: 0.8,
        }
    }

    /// Create from man page
    pub fn from_man(page: &str, excerpt: &str) -> Self {
        Self::new(DocSourceKind::ManPage, &format!("man:{}", page), excerpt)
    }

    /// Create from Arch Wiki
    pub fn from_wiki(title: &str, excerpt: &str) -> Self {
        Self::new(
            DocSourceKind::ArchWiki,
            &format!(
                "https://wiki.archlinux.org/title/{}",
                title.replace(' ', "_")
            ),
            excerpt,
        )
    }

    /// Create from help flag
    pub fn from_help(command: &str, excerpt: &str) -> Self {
        Self::new(
            DocSourceKind::HelpFlag,
            &format!("{} --help", command),
            excerpt,
        )
    }

    /// Set section
    pub fn with_section(mut self, section: &str) -> Self {
        self.section = Some(section.to_string());
        self
    }

    /// Set relevance
    pub fn with_relevance(mut self, relevance: f32) -> Self {
        self.relevance = relevance;
        self
    }

    /// Format as citation string
    pub fn citation(&self) -> String {
        match self.kind {
            DocSourceKind::ManPage => format!("man:{}", self.reference.trim_start_matches("man:")),
            DocSourceKind::ArchWiki => {
                if let Some(title) = self.reference.split("/title/").nth(1) {
                    format!("Arch Wiki: {}", title.replace('_', " "))
                } else {
                    format!("Arch Wiki: {}", self.reference)
                }
            }
            DocSourceKind::HelpFlag => self.reference.clone(),
            DocSourceKind::Info => format!("info:{}", self.reference),
            DocSourceKind::LocalFile => format!("file:{}", self.reference),
            DocSourceKind::Builtin => "builtin".to_string(),
        }
    }
}

/// Format multiple doc snippets as a sources section
pub fn format_sources(snippets: &[DocSnippet]) -> String {
    if snippets.is_empty() {
        return String::new();
    }

    let mut sources = String::from("\n**Sources:**\n");
    for snippet in snippets.iter().take(5) {
        sources.push_str(&format!("- {}\n", snippet.citation()));
    }
    sources
}
