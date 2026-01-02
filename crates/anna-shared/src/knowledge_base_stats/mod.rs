//! Knowledge Base Stats (Phase 75)
//!
//! Tracks and displays statistics about Anna's knowledge base,
//! including recipes, facts, and documentation.

mod types;
mod entry;
mod stats;
mod formatting;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public items
pub use types::{KnowledgeType, KnowledgeSource};
pub use entry::KnowledgeEntry;
pub use stats::KnowledgeBaseStats;
pub use formatting::{
    format_knowledge_stats,
    format_knowledge_stats_compact,
    format_knowledge_stats_oneline,
};
pub use utils::{
    knowledge_insight,
    is_knowledge_stats_query,
    knowledge_health,
};
