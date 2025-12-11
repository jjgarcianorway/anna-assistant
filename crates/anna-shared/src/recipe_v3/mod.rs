//! Recipe V3 - Learning and Reuse Engine (v0.0.423).
//!
//! A robust recipe system where good solutions become reusable, parametric recipes.
//! Key improvements over v2:
//! - Safe execution with risk levels and confirmation policies
//! - Precondition evaluation via probes
//! - Parametric steps with variable substitution
//! - Full execution logging and stats tracking
//! - Ticket-to-recipe builder with strict learning rules

pub mod types;
pub mod condition;
pub mod step;
pub mod matcher;
pub mod store;
pub mod executor;
pub mod builder;

pub use types::{
    RecipeV3, RecipeOrigin, RecipeAuthor, RecipeDomain, RecipeRiskLevel,
    ConfirmationPolicy, RecipeMatcher, RecipeStats,
};
pub use condition::*;
pub use step::*;
pub use matcher::{MatchResult, MatchBreakdown, MatchQuery, RecipeMatcher as QueryMatcher};
pub use store::*;
pub use executor::*;
pub use builder::*;

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
