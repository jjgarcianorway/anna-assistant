//! Knowledge Engine Types
//!
//! Core type definitions for the knowledge engine.

use serde::{Deserialize, Serialize};

/// Knowledge hit kind
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeKind {
    ManPage,
    CliHelp,
    LocalDoc,
    ArchWiki,
    BuiltIn,
}

impl std::fmt::Display for KnowledgeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ManPage => write!(f, "man"),
            Self::CliHelp => write!(f, "help"),
            Self::LocalDoc => write!(f, "doc"),
            Self::ArchWiki => write!(f, "wiki"),
            Self::BuiltIn => write!(f, "built-in"),
        }
    }
}

/// A knowledge hit from the engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEngineHit {
    /// Document ID (e.g., "man:systemctl", "help:pacman")
    pub doc_id: String,
    /// Kind of knowledge
    pub kind: KnowledgeKind,
    /// Title
    pub title: String,
    /// Command used to fetch (for reference)
    pub command: String,
    /// Relevant snippet (truncated)
    pub snippet: String,
    /// Source (local, cache)
    pub source: String,
    /// Relevance score (0-100)
    pub relevance: u8,
    /// Citation ID for provenance tracking (e.g., "man:systemctl:line42-50")
    #[serde(default)]
    pub citation_id: String,
    /// Line range in source document (if applicable)
    #[serde(default)]
    pub line_range: Option<(usize, usize)>,
}

impl KnowledgeEngineHit {
    /// Generate a citation reference for display
    pub fn citation_ref(&self) -> String {
        match self.kind {
            KnowledgeKind::ManPage => format!("[man {}]", self.title),
            KnowledgeKind::CliHelp => {
                format!("[{} --help]", self.title.trim_end_matches(" --help"))
            }
            KnowledgeKind::LocalDoc => format!("[doc:{}]", self.title),
            KnowledgeKind::ArchWiki => {
                format!("[wiki:{}]", self.title.trim_start_matches("Arch Wiki: "))
            }
            KnowledgeKind::BuiltIn => format!("[{}]", self.title),
        }
    }

    /// Format for display in citations footer
    pub fn citation_display(&self) -> String {
        let source_type = match self.kind {
            KnowledgeKind::ManPage => "man page",
            KnowledgeKind::CliHelp => "command help",
            KnowledgeKind::LocalDoc => "local doc",
            KnowledgeKind::ArchWiki => "Arch Wiki",
            KnowledgeKind::BuiltIn => "built-in",
        };
        format!(
            "{} ({}): \"{}\"",
            self.title,
            source_type,
            crate::knowledge_engine::utils::truncate(&self.snippet, 100)
        )
    }
}

/// Knowledge query request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeRequest {
    /// Topic to search (e.g., "failed services", "boot time")
    pub topic: String,
    /// Context for relevance
    pub context: KnowledgeContext,
    /// Which sources to use
    pub sources: Vec<KnowledgeKind>,
    /// Maximum hits to return
    pub limit: usize,
}

/// Context for knowledge request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeContext {
    /// Intent (check_status, diagnose, etc.)
    pub intent: String,
    /// Domain (services, storage, etc.)
    pub domain: String,
    /// Related commands (for focused search)
    pub commands: Vec<String>,
}

/// Knowledge query result
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KnowledgeResponse {
    /// Hits found
    pub hits: Vec<KnowledgeEngineHit>,
    /// Query time (ms)
    pub query_time_ms: u64,
    /// Sources searched
    pub sources_searched: Vec<KnowledgeKind>,
    /// Errors (non-fatal)
    pub errors: Vec<String>,
}

/// Cache entry for serialization
#[derive(Serialize, Deserialize)]
pub(super) struct CacheEntry {
    pub hit: KnowledgeEngineHit,
    pub cached_at: u64,
}
