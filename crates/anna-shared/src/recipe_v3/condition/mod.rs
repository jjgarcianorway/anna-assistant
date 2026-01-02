//! Recipe preconditions and postconditions (v0.0.423).
//!
//! Conditions that must be true before/after recipe execution.

mod evaluators;
mod types;

pub use types::{ConditionResult, RecipeCondition};
