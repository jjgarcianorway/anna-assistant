// v0.0.537: Query History Tracker Module (Phase 113)
// Tracks user queries for "repeated questions" and "topic most asked about" per VISION.md

mod types;
mod tracker;
mod utils;
mod formatting;

#[cfg(test)]
mod tests;

// Re-export types
pub use types::{QueryCategory, QueryOutcome, QueryRecord};

// Re-export tracker
pub use tracker::QueryHistoryTracker;

// Re-export utility functions
pub use utils::{
    classify_query,
    is_history_query,
    normalize_query,
    query_history_fun_fact,
    query_similarity,
};

// Re-export formatting functions
pub use formatting::{format_query, format_tracker_summary};
