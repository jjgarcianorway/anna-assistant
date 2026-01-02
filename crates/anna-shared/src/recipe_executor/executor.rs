//! Main recipe execution logic - orchestrates step execution.

use crate::doc_snippet::DocSnippet;
use crate::recipe_engine::{Recipe, RecipeStepType};
use tracing::{debug, info};

use super::step_handlers::RecipeExecutor;
use super::types::{ExecutionContext, ExecutionResult};
use super::utils::topological_sort;

impl RecipeExecutor {
    /// Execute a recipe
    pub fn execute(&self, recipe: &Recipe, ctx: &mut ExecutionContext) -> ExecutionResult {
        let mut result = ExecutionResult::new();

        ctx.recipe_id = Some(recipe.id.clone());
        info!("Executing recipe: {} ({})", recipe.name, recipe.id);

        // Build dependency graph and execution order
        let order = match topological_sort(&recipe.steps) {
            Ok(o) => o,
            Err(e) => {
                result.success = false;
                result.errors.push(format!("Dependency error: {}", e));
                return result;
            }
        };

        // Execute steps in order
        for step_id in order {
            let step = match recipe.steps.iter().find(|s| s.id == step_id) {
                Some(s) => s,
                None => continue,
            };

            // Check dependencies succeeded
            if !step
                .depends_on
                .iter()
                .all(|dep| ctx.outputs.get(dep).map(|o| o.success).unwrap_or(false))
            {
                debug!("Skipping step {} - dependency not met", step_id);
                continue;
            }

            // Execute step
            match self.execute_step(step, recipe, ctx) {
                Ok(output) => {
                    let success = output.success;
                    ctx.outputs.insert(step_id.clone(), output.clone());
                    result.step_outputs.insert(step_id.clone(), output);

                    if !success && step.kind != RecipeStepType::CheckCondition {
                        result.success = false;
                        result.errors.push(format!("Step {} failed", step_id));
                        break;
                    }
                }
                Err(e) => {
                    result.success = false;
                    result.errors.push(format!("Step {} error: {}", step_id, e));
                    break;
                }
            }
        }

        // Find render_answer step output for final answer
        for step in &recipe.steps {
            if step.kind == RecipeStepType::RenderAnswer {
                if let Some(output) = result.step_outputs.get(&step.id) {
                    result.answer = output.stdout.clone();
                }
            }
        }

        // Add doc sources
        for doc_ref in &recipe.doc_sources {
            result.sources.push(DocSnippet::new(
                crate::doc_snippet::DocSourceKind::Builtin,
                doc_ref,
                "",
            ));
        }

        result
    }
}
