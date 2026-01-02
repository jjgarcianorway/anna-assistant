//! Recipe V3 - Learning and Reuse Engine (v0.0.423).
//!
//! A robust recipe system where good solutions become reusable, parametric recipes.
//! Key improvements over v2:
//! - Safe execution with risk levels and confirmation policies
//! - Precondition evaluation via probes
//! - Parametric steps with variable substitution
//! - Full execution logging and stats tracking
//! - Ticket-to-recipe builder with strict learning rules

pub mod builder_core;
pub mod condition;
pub mod executor;
pub mod matcher;
pub mod seed;
pub mod step_execution;
pub mod step_helpers;
pub mod step_types;
pub mod store;
pub mod store_impl;
pub mod store_query;
pub mod store_types;
pub mod ticket_extraction;
pub mod types;
pub mod types_enums;
pub mod types_matcher;
pub mod types_stats;

pub use builder_core::*;
pub use condition::*;
pub use executor::*;
pub use matcher::{MatchBreakdown, MatchQuery, MatchResult, RecipeMatcher as QueryMatcher};
pub use seed::*;
pub use step_types::{RecipeStep, StepResult};
pub use store::*;
pub use ticket_extraction::*;
pub use types::{
    ConfirmationPolicy, RecipeAuthor, RecipeDomain, RecipeMatcher, RecipeOrigin, RecipeRiskLevel,
    RecipeStats, RecipeV3,
};

/// Maximum recipes to check during matching
pub const MAX_RECIPES_TO_CHECK: usize = 50;

/// Minimum match score to consider a recipe
pub const MIN_MATCH_SCORE: f32 = 0.50;

/// Threshold for auto-execution without confirmation
pub const AUTO_EXECUTE_THRESHOLD: f32 = 0.85;

/// Threshold for suggesting a recipe (but requiring confirmation)
pub const SUGGEST_THRESHOLD: f32 = 0.65;

/// Maximum steps in a single recipe
pub const MAX_RECIPE_STEPS: usize = 10;

/// Minimum successful uses before a recipe is considered mature
pub const MIN_MATURE_USES: u32 = 5;

/// Minimum success rate for mature recipes
pub const MIN_SUCCESS_RATE: f32 = 0.80;

/// Default risk level for unknown commands
pub const DEFAULT_RISK_LEVEL: RecipeRiskLevel = RecipeRiskLevel::Medium;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert!(MIN_MATCH_SCORE < AUTO_EXECUTE_THRESHOLD);
        assert!(SUGGEST_THRESHOLD < AUTO_EXECUTE_THRESHOLD);
        assert!(MIN_SUCCESS_RATE > 0.5);
    }
}
