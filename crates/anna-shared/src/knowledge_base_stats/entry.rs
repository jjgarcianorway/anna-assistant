//! Individual knowledge entry

use serde::{Deserialize, Serialize};
use super::types::{KnowledgeType, KnowledgeSource};

/// Individual knowledge entry stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEntry {
    /// Entry ID
    pub id: String,
    /// Type of knowledge
    pub knowledge_type: KnowledgeType,
    /// Source of knowledge
    pub source: KnowledgeSource,
    /// When first acquired (Unix timestamp)
    pub acquired_at: u64,
    /// When last used (Unix timestamp)
    pub last_used: u64,
    /// Number of times used
    pub use_count: u64,
    /// Topic/category
    pub topic: Option<String>,
    /// Reliability score (0-100)
    pub reliability: u8,
}

impl KnowledgeEntry {
    /// Create a new entry
    pub fn new(
        id: impl Into<String>,
        knowledge_type: KnowledgeType,
        source: KnowledgeSource,
        acquired_at: u64,
    ) -> Self {
        Self {
            id: id.into(),
            knowledge_type,
            source,
            acquired_at,
            last_used: acquired_at,
            use_count: 0,
            topic: None,
            reliability: 80,
        }
    }

    /// Record usage
    pub fn record_use(&mut self, timestamp: u64) {
        self.use_count += 1;
        self.last_used = timestamp;
    }

    /// Is this entry stale (not used in 30 days)?
    pub fn is_stale(&self, now: u64) -> bool {
        now.saturating_sub(self.last_used) > 30 * 86400
    }
}
