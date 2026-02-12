//! Memory System - Anna remembers and learns from experience.
//!
//! Two types of memory:
//! - Episodic: Specific past interactions (conversations, fixes)
//! - Semantic: General knowledge and facts learned over time

mod types;
mod store;

pub use types::{
    Interaction, InteractionContext, InteractionOutcome,
    EpisodicMemory, SemanticMemory, LearnedFact, FactCategory,
};
pub use store::MemoryStore;
