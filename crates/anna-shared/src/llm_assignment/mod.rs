//! LLM Assignment Tracker - Phase 88
//!
//! Tracks which LLM model each specialist uses.
//! VISION.md: "Which specialist uses which LLM"
//! "Models adjusted by Anna based on hardware available"

mod types;
mod tracker;
mod models;
mod formatting;
mod queries;

#[cfg(test)]
mod tests;

// Re-export all public types to preserve the original API
pub use types::{AssignmentReason, LlmAssignment, ModelTier};
pub use tracker::LlmAssignmentTracker;
pub use models::{get_model_tier, COMMON_MODELS};
pub use formatting::{format_llm_tracker, format_llm_tracker_compact, format_llm_tracker_oneline};
pub use queries::{is_llm_query, llm_fun_fact};
