//! Helper functions for recipe execution.

use std::collections::HashMap;

use crate::recipe_v3::{MatchResult, RecipeStore, RecipeV3, StoreError};

use super::executor_core::RecipeExecutor;
use super::executor_types::{ExecutionPlan, ExecutionResult, PlannedStep};

/// Execute a recipe and update store stats
pub fn execute_and_record(
    executor: &RecipeExecutor,
    match_result: &MatchResult,
    store: &mut RecipeStore,
) -> Result<ExecutionResult, StoreError> {
    let result = executor.execute_match(match_result);

    // Update stats in store
    store.record_execution(&result.recipe_id, result.success, result.duration_ms)?;

    Ok(result)
}

/// Create an execution plan without executing
pub fn create_execution_plan(recipe: &RecipeV3, vars: &HashMap<String, String>) -> ExecutionPlan {
    let mut steps = vec![];

    for (index, step) in recipe.steps.iter().enumerate() {
        steps.push(PlannedStep {
            index,
            description: step.describe(),
            risk_level: step.risk_level(),
            needs_confirmation: step.risk_level().requires_confirmation(),
        });
    }

    ExecutionPlan {
        recipe_id: recipe.id.clone(),
        recipe_title: recipe.title.clone(),
        total_steps: steps.len(),
        steps,
        variables: vars.clone(),
        overall_risk: recipe.risk_level,
        confirmation_policy: recipe.confirmation,
    }
}
