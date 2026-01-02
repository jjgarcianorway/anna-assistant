//! Learned Recipes (v0.0.416).
//!
//! Self-learning recipe system that:
//! - Creates recipes from successful tickets
//! - Matches by intent (not by specific phrasing)
//! - Executes deterministically without LLM
//! - Tracks effectiveness and deprecates bad recipes
//!
//! Design goals:
//! - No hardcoded answers or questions
//! - Generic, parameterized recipes
//! - Learns from specialist success cases

mod execution;
mod storage;
#[cfg(test)]
mod tests;
mod types;

// Re-export all public types and functions to preserve the original API
pub use execution::execute_recipe;
pub use storage::RecipeStore;
pub use types::{
    AnswerTemplate, CompareOp, LearnedRecipe, RecipeComputeStep, RecipeContext, RecipeResult,
    RecipeStats, RecipeStoreSummary, RenderedAnswer,
};
