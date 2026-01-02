//! Knowledge Index - Anna's compiled knowledge store (v0.0.410).
//!
//! Stores and retrieves learned knowledge:
//! - Facts: Simple key-value knowledge (e.g., "swap_enabled: true")
//! - Patterns: Query patterns with proven solutions
//! - Snippets: Cached doc snippets that worked well
//!
//! Knowledge is accumulated from successful ticket resolutions
//! and allows Anna to answer without re-hitting the LLM.

mod doc_cache;
mod fact;
mod index;
mod pattern;
mod utils;

// Re-export all public types to preserve API
pub use doc_cache::CachedDoc;
pub use fact::LearnedFact;
pub use index::{IndexStats, KnowledgeIndex};
pub use pattern::{EvidenceHint, LearnedPattern};
