//! Recipe execution engine (v0.0.406).
//!
//! Provides the core recipe functions:
//! - find_matching_recipe(domain, intent, params) -> Option<Recipe>
//! - execute_recipe(recipe, context) -> ExecutionResult
//! - render_answer(recipe, execution_result) -> String

mod executor;
mod matcher;
mod renderer;
mod types;

pub use executor::execute_recipe;
pub use matcher::find_matching_recipe;
pub use renderer::{render_answer, RecipeEngine};
pub use types::{ExecutionResult, RecipeContext, RecipeMatchResult, StepResult};
