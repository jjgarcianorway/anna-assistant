//! Recipe Engine v2 Integration (v0.0.412).
//!
//! Integrates the new learned recipe system with the existing fast path.
//! Checks learned recipes BEFORE hardcoded ones to maximize self-learning.

mod execution;
mod matching;
mod params;
mod result_builder;
mod stats;
mod store;

#[cfg(test)]
mod tests;

// Re-export public API to maintain backward compatibility
pub use execution::execute_learned_recipe;
pub use matching::{check_learned_recipes, LearnedRecipeResult};
pub use params::extract_param_value;
pub use stats::{get_recipe_stats, list_recipes, RecipeSummary};
pub use store::{get_store, run_gc};
