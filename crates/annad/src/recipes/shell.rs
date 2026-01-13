//! Shell configuration recipes (bash, zsh, fish).
//! NOTE: Anna does not modify user home directories (invariant 2).
//! This module is disabled.

use super::RecipeResult;

/// Try to match a shell-related recipe.
/// Always returns None - Anna does not modify user home files.
pub fn try_recipe(_q: &str) -> Option<RecipeResult> {
    // Invariant 2: Anna must never write to user home directories.
    // Shell config files are in ~/.bashrc, ~/.zshrc, ~/.config/fish, which are user home paths.
    // Therefore, shell recipes are not supported.
    None
}

/// Execute a confirmed shell recipe.
/// Always returns error - Anna does not modify user home files.
pub fn execute_confirmed(_recipe_id: &str) -> RecipeResult {
    RecipeResult {
        success: false,
        message: "Anna does not modify user home directories. Shell configuration must be done manually.".to_string(),
        needs_confirmation: false,
        confirmation_prompt: None,
    }
}
