//! Specialist Learning System (v0.0.401)
//!
//! Captures knowledge when Anna receives help from specialists (Senior escalation,
//! LLM self-healing, or user feedback). This knowledge is used to create generic
//! recipes and improve future responses.
//!
//! Learning Threshold (Adaptive):
//! - High-confidence (80+): Learn immediately from first success
//! - Lower confidence: Require 2+ successes before creating generic recipe

mod patterns;
mod store;
mod tests;
mod types;
mod utils;

// Re-export public types to maintain the original API
pub use patterns::{detect_pattern_category, extract_target};
pub use store::SpecialistLearningStore;
pub use types::{
    GenericPattern, PatternCategory, PatternVariable, PendingPattern, SolutionType,
    SpecialistLesson,
};
