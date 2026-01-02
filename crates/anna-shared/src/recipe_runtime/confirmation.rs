//! User confirmation logic for recipe execution.

use crate::recipe_schema::{ConfirmationPolicy, Recipe};
use super::types::ExecutionContext;
use super::utils::{expand_path, describe_step};

/// Check if recipe needs user confirmation.
pub fn needs_confirmation(recipe: &Recipe, ctx: &ExecutionContext) -> bool {
    if ctx.user_confirmed {
        return false;
    }

    match recipe.confirmation_policy {
        ConfirmationPolicy::Never => false,
        ConfirmationPolicy::Require => true,
        ConfirmationPolicy::MutatingOnly => recipe.has_mutating_steps(),
    }
}

/// Generate a confirmation prompt for the user.
pub fn generate_confirmation_prompt(recipe: &Recipe, ctx: &ExecutionContext) -> String {
    let mut prompt = format!(
        "I can {} by performing the following steps:\n\n",
        recipe.pattern.user_goal
    );

    for (i, step) in recipe.plan.iter().enumerate() {
        let step_desc = describe_step(step, ctx);
        prompt.push_str(&format!("{}. {}\n", i + 1, step_desc));
    }

    if !recipe.touched_files().is_empty() {
        prompt.push_str("\nFiles that will be modified:\n");
        for file in recipe.touched_files() {
            prompt.push_str(&format!("  - {}\n", expand_path(&file, ctx)));
        }
    }

    prompt.push_str("\nApply this change?");
    prompt
}
