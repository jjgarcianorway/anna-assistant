// v0.0.578: Settings Recommendations (Phase 154)
// Provide intelligent settings recommendations based on usage

mod checkers;
mod engine;
mod types;
mod utils;

// Re-export all public types and functions to preserve the original API
pub use engine::RecommendationEngine;
pub use types::{
    Recommendation, RecommendationPriority, RecommendationStatus, RecommendationType,
};
pub use utils::{
    format_recommendations, is_recommendations_query, settings_recommendations_fun_fact,
};
