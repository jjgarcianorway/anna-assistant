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

mod format;
mod engine;
mod loader;

pub use format::{
    FileRecipe, RecipeMatch as FileRecipeMatch, RecipePlan, RecipeStep,
    RecipeAnswer, ConfirmLevel,
};
pub use engine::{
    RecipeEngine, RecipeContext, ExecutionResult, StepResult,
    find_matching_recipe, execute_recipe, render_answer,
};
pub use loader::{load_all_recipes, load_recipe_from_file, recipe_dirs};
