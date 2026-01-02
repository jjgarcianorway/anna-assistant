//! Interaction Counter (v0.0.488).
//!
//! Tracks interactions between Anna and specialists.
//! Provides detailed statistics on communication patterns.

mod types;
mod stats;
mod counter;
mod formatting;

// Re-export public types
pub use types::{InteractionRecord, InteractionType};
pub use stats::SpecialistStats;
pub use counter::{InteractionCounter, InteractionSummary};
pub use formatting::{
    format_interactions,
    format_interactions_compact,
    interaction_fun_fact,
    is_interaction_query,
};
