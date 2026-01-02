//! Index types and structures (v0.0.429).

use crate::doc_engine::{DocSnippet, DocSourceKind};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Documentation index (in-memory + on-disk)
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DocIndex {
    /// All snippets by ID
    pub(super) snippets: HashMap<String, DocSnippet>,
    /// Keyword index (keyword -> snippet IDs)
    pub(super) keyword_index: HashMap<String, Vec<String>>,
    /// Source index (source kind -> snippet IDs)
    pub(super) source_index: HashMap<String, Vec<String>>,
    /// Name index (name -> snippet IDs)
    pub(super) name_index: HashMap<String, Vec<String>>,
    /// Index version for migrations
    pub(super) version: u32,
    /// Last rebuild timestamp
    pub(super) last_rebuild: u64,
}

impl DocIndex {
    /// Current index version
    pub const VERSION: u32 = 1;

    /// Create new empty index
    pub fn new() -> Self {
        Self {
            version: Self::VERSION,
            ..Default::default()
        }
    }

    /// Total snippet count
    pub fn len(&self) -> usize {
        self.snippets.len()
    }

    /// Check if index is empty
    pub fn is_empty(&self) -> bool {
        self.snippets.is_empty()
    }

    /// Count snippets by source
    pub fn count_by_source(&self) -> HashMap<DocSourceKind, usize> {
        let mut counts = HashMap::new();
        for snippet in self.snippets.values() {
            *counts.entry(snippet.source).or_insert(0) += 1;
        }
        counts
    }

    /// Clear all snippets
    pub fn clear(&mut self) {
        self.snippets.clear();
        self.keyword_index.clear();
        self.source_index.clear();
        self.name_index.clear();
    }

    /// Mark index as rebuilt
    pub fn mark_rebuilt(&mut self) {
        self.last_rebuild = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
    }
}

/// Index errors
#[derive(Debug, Clone)]
pub enum IndexError {
    IoError(String),
    ParseError(String),
    VersionMismatch { expected: u32, found: u32 },
}

impl std::fmt::Display for IndexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "IO error: {}", e),
            Self::ParseError(e) => write!(f, "Parse error: {}", e),
            Self::VersionMismatch { expected, found } => {
                write!(
                    f,
                    "Index version mismatch: expected {}, found {}",
                    expected, found
                )
            }
        }
    }
}

impl std::error::Error for IndexError {}
