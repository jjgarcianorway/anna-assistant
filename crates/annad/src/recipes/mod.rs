//! Configuration recipes - Anna makes real changes to your system.
//! v0.0.998: Initial implementation
//!
//! Recipes handle:
//! - Vim/Neovim configuration
//! - Git configuration
//! - Shell aliases and config
//! - Service management
//! - Undo operations
//! - Conversation settings

mod vim;
mod git;
mod shell;
mod services;
mod settings;
mod undo;

use anna_shared::rpc::{AskResult, DialogueStep, StepType};
use tracing::info;

pub use undo::handle_undo_request;
pub use settings::{get_settings, ConversationSettings};

/// Result of applying a recipe
pub struct RecipeResult {
    pub success: bool,
    pub message: String,
    pub needs_confirmation: bool,
    pub confirmation_prompt: Option<String>,
}

/// Check if a question matches any recipe and execute it
pub fn try_recipe(question: &str) -> Option<RecipeResult> {
    let q = question.to_lowercase();

    // Try settings first (immediate effect, no confirmation needed)
    if let Some(result) = settings::try_recipe(&q) {
        return Some(result);
    }

    // Try undo next
    if let Some(result) = undo::try_undo(&q) {
        return Some(result);
    }

    // Try vim recipes
    if let Some(result) = vim::try_recipe(&q) {
        return Some(result);
    }

    // Try git recipes
    if let Some(result) = git::try_recipe(&q) {
        return Some(result);
    }

    // Try shell recipes
    if let Some(result) = shell::try_recipe(&q) {
        return Some(result);
    }

    // Try service recipes
    if let Some(result) = services::try_recipe(&q) {
        return Some(result);
    }

    None
}

/// Execute a confirmed recipe (after user says yes)
pub fn execute_confirmed_recipe(recipe_id: &str) -> RecipeResult {
    // Parse recipe ID to determine which module handles it
    if recipe_id.starts_with("vim-") {
        vim::execute_confirmed(recipe_id)
    } else if recipe_id.starts_with("git-") {
        git::execute_confirmed(recipe_id)
    } else if recipe_id.starts_with("shell-") {
        shell::execute_confirmed(recipe_id)
    } else if recipe_id.starts_with("service-") {
        services::execute_confirmed(recipe_id)
    } else {
        RecipeResult {
            success: false,
            message: "Unknown recipe".to_string(),
            needs_confirmation: false,
            confirmation_prompt: None,
        }
    }
}
