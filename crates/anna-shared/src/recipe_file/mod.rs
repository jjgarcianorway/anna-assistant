//! File-based recipe system (v0.0.406).
//!
//! Allows manually authored recipes in TOML format that can be used
//! to handle common patterns without LLM involvement.
//!
//! Recipe locations:
//! - /etc/anna/recipes/*.toml (system-wide)
//! - ~/.anna/recipes/authored/*.toml (user-authored)
//!
//! This complements the learned recipe system (JSON-based) with
//! curated, version-controlled recipes.

pub mod engine;
mod format;
mod loader;

pub use engine::{
    execute_recipe, find_matching_recipe, render_answer, ExecutionResult, RecipeContext,
    RecipeEngine, RecipeMatchResult, StepResult,
};
pub use format::{
    ConfirmLevel, FileRecipe, RecipeAnswer, RecipeMatch as FileRecipeMatch, RecipePlan, RecipeStep,
};
pub use loader::{load_all_recipes, load_recipe_from_file, recipe_dirs};
