//! Recipe execution planning logic.

use crate::recipe_schema::{Recipe, RecipeStatus};
use super::types::{ExecutionContext, ExecutionPlan, ExecutionStep};
use super::preconditions::check_preconditions;
use super::confirmation::needs_confirmation;
use super::utils::expand_step_paths;

/// Execute a recipe (returns execution plan, actual execution done by caller).
pub fn prepare_execution(recipe: &Recipe, ctx: &ExecutionContext) -> Result<ExecutionPlan, String> {
    // Check if recipe is usable
    if recipe.status != RecipeStatus::Active {
        return Err(format!("Recipe '{}' is not active", recipe.id));
    }

    // Check preconditions
    let precond_result = check_preconditions(recipe, ctx);
    if !precond_result.all_met {
        return Err(format!(
            "Preconditions not met: {}",
            precond_result.failed.join(", ")
        ));
    }

    // Check confirmation
    if needs_confirmation(recipe, ctx) && !ctx.user_confirmed {
        return Err("User confirmation required".into());
    }

    // Build execution plan
    let steps: Vec<ExecutionStep> = recipe
        .plan
        .iter()
        .map(|s| ExecutionStep {
            step: s.clone(),
            expanded_paths: expand_step_paths(s, ctx),
        })
        .collect();

    Ok(ExecutionPlan {
        recipe_id: recipe.id.clone(),
        steps,
        rollback_on_failure: recipe.success_criteria.rollback_on_failure,
    })
}
