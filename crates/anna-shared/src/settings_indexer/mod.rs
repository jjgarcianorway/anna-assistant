// v0.0.669: Settings Indexer Module (Phase 245)
// Index settings for fast lookup and search

mod types;
mod stats;
mod indexer;
mod registry;
mod helpers;

#[cfg(test)]
mod tests;

// Re-export all public types and functions
pub use types::{
    IndexType,
    IndexStatus,
    IndexerConfig,
    IndexEntry,
    IndexLookupResult,
};

pub use stats::IndexerStats;
pub use indexer::SettingsIndexer;
pub use registry::IndexerRegistry;

pub use helpers::{
    format_indexer_registry,
    is_indexer_query,
    indexer_fun_fact,
};
