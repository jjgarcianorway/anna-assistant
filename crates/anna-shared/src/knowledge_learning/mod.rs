//! Knowledge Learning System (v0.0.414).
//!
//! Self-learning from docs and successful tickets.
//!
//! Learning workflow:
//! 1. On every high-confidence solved ticket, record:
//!    - Intent classification
//!    - Probes used and their effectiveness
//!    - Knowledge docs consulted
//!    - Final answer structure (citations, grounding)
//!
//! 2. Periodically (idle-time learning job):
//!    - Cluster tickets by intent
//!    - Extract patterns (probes that work, docs that help)
//!    - Propose new recipes from patterns
//!    - Senior LLM review for safety
//!
//! No hardcoded natural language - all learning is from evidence.

mod analysis;
mod store;
mod types;
mod utils;

// Re-export public API
pub use store::KnowledgeLearningStore;
pub use types::{
    DocReference, LearningStats, ProbeStats, ProposedRecipe, RecipeStatus, SolvedTicketRecord,
    UserFeedback,
};
pub use utils::create_ticket_record;

#[cfg(test)]
mod tests;
