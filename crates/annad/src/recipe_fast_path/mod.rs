//! Recipe-based fast path for queries (v0.0.101, v0.0.102: direct answers).
//! Checks recipe index BEFORE LLM translator. High-confidence matches skip LLM entirely.
//! v0.0.163: Built-in recipe matchers extracted to separate module.
//! v0.0.264: Added config hint support for specialist context.
//! v0.0.406: TOML-based FileRecipe check moved to file_recipe_path.rs.
//! v0.0.412: Check learned recipes FIRST (RecipeStoreV2) before hardcoded.

// Module declarations
mod checker;
mod converter;
mod learned;
mod types;

#[cfg(test)]
mod tests;

// Re-export built-in recipe matchers from recipe_builtins
pub use crate::recipe_builtins::{
    check_cron_recipes, check_docker_recipes, check_git_recipes, check_shell_recipes,
    check_ssh_recipes, check_systemd_recipes,
};

// Re-export main types
pub use types::{RecipeFastPathResult, RECIPE_SKIP_LLM_THRESHOLD};

// Re-export main checker function
pub use checker::check_recipe_fast_path;

// Re-export converter functions
pub use converter::{
    build_recipe_result, get_config_hint_for_specialist, team_to_domain, ticket_from_recipe,
};

// Re-export learned recipe functions
pub use learned::{can_answer_directly, execute_learned_recipe, is_learned_recipe};
