//! Recipe data model for Anna's learning system.
//! v0.0.418: Full recipe schema with matcher, plan steps, and metrics.
//!
//! Recipes are declarative JSON objects that describe:
//! - The INTENT they serve
//! - How to detect when they apply (matcher)
//! - Preconditions (probes or simple checks)
//! - A PLAN: steps the non-LLM engine can execute
//! - Success criteria and rollback behavior
//! - Origin/citations and metrics

mod plan_step;
mod precondition;
mod recipe_impl;
mod types;

#[cfg(test)]
mod tests;

// Re-export all public types to maintain the same API
pub use plan_step::PlanStep;
pub use precondition::Precondition;
pub use types::{
    ConfirmationPolicy, Recipe, RecipeMatcher, RecipeMetrics, RecipePattern, RecipeStatus,
    SuccessCriteria,
};
