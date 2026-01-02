//! Recipe summary generation.

use crate::recipe_schema::Recipe;
use super::types::ExecutionContext;
use super::utils::describe_step;

/// Generate a summary of what a recipe would do (for display).
pub fn generate_recipe_summary(recipe: &Recipe, ctx: &ExecutionContext) -> String {
    let mut summary = String::new();

    summary.push_str(&format!("Recipe: {} (v{})\n", recipe.id, recipe.version));
    summary.push_str(&format!("Goal: {}\n", recipe.pattern.user_goal));

    if !recipe.citations.is_empty() {
        summary.push_str(&format!("Based on: {}\n", recipe.citations.join(", ")));
    }

    summary.push_str("\nSteps:\n");
    for (i, step) in recipe.plan.iter().enumerate() {
        summary.push_str(&format!("  {}. {}\n", i + 1, describe_step(step, ctx)));
    }

    if recipe.metrics.times_used > 0 {
        let success_rate = if recipe.metrics.times_used > 0 {
            let successes = recipe.metrics.times_used - recipe.metrics.times_failed;
            successes as f32 / recipe.metrics.times_used as f32 * 100.0
        } else {
            100.0
        };
        summary.push_str(&format!(
            "\nUsed {} times ({:.0}% success rate)\n",
            recipe.metrics.times_used, success_rate
        ));
    }

    summary
}
