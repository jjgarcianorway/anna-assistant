//! Recipe execution logic.

use super::types::{
    AnswerTemplate, LearnedRecipe, RecipeComputeStep, RecipeContext, RecipeResult, RenderedAnswer,
};

/// Execute a recipe
pub fn execute_recipe(recipe: &LearnedRecipe, ctx: &mut RecipeContext) -> RecipeResult {
    // Check required probes
    for probe in &recipe.required_probes {
        if !ctx.probe_outputs.contains_key(probe) {
            return RecipeResult::Failed {
                reason: format!("Missing required probe: {}", probe),
            };
        }
    }

    // Execute computation steps
    for step in &recipe.steps {
        if let Err(e) = execute_step(step, ctx) {
            return RecipeResult::Failed { reason: e };
        }
    }

    // Determine which answer template to use
    let (template, confidence) = select_answer_template(recipe, ctx);

    // Render answer
    let answer = template.render(&ctx.variables);

    RecipeResult::Success { answer, confidence }
}

/// Execute a single computation step
fn execute_step(step: &RecipeComputeStep, ctx: &mut RecipeContext) -> Result<(), String> {
    match step {
        RecipeComputeStep::Extract {
            probe,
            pattern,
            variable,
        } => {
            let output = ctx
                .probe_outputs
                .get(probe)
                .ok_or_else(|| format!("Probe {} not found", probe))?;

            let re = regex::Regex::new(pattern).map_err(|e| format!("Invalid pattern: {}", e))?;

            if let Some(caps) = re.captures(output) {
                let value = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                ctx.variables.insert(variable.clone(), value.to_string());
            }
            Ok(())
        }

        RecipeComputeStep::Compare {
            variable,
            operator,
            threshold,
            result_var,
        } => {
            let value: f64 = ctx
                .variables
                .get(variable)
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.0);

            let result = operator.eval(value, *threshold);
            ctx.variables.insert(result_var.clone(), result.to_string());
            Ok(())
        }

        RecipeComputeStep::Count {
            probe,
            pattern,
            variable,
        } => {
            let output = ctx
                .probe_outputs
                .get(probe)
                .ok_or_else(|| format!("Probe {} not found", probe))?;

            let re = regex::Regex::new(pattern).map_err(|e| format!("Invalid pattern: {}", e))?;

            let count = re.find_iter(output).count();
            ctx.variables.insert(variable.clone(), count.to_string());
            Ok(())
        }

        RecipeComputeStep::IsEmpty { probe, variable } => {
            let output = ctx
                .probe_outputs
                .get(probe)
                .map(|s| s.trim().is_empty())
                .unwrap_or(true);

            ctx.variables.insert(variable.clone(), output.to_string());
            Ok(())
        }

        RecipeComputeStep::ParseNumber {
            source_var,
            target_var,
        } => {
            let source = ctx
                .variables
                .get(source_var)
                .ok_or_else(|| format!("Variable {} not found", source_var))?;

            // Extract first numeric value
            let re = regex::Regex::new(r"[\d.]+").unwrap();
            if let Some(m) = re.find(source) {
                ctx.variables
                    .insert(target_var.clone(), m.as_str().to_string());
            }
            Ok(())
        }
    }
}

/// Select appropriate answer template based on computed variables
fn select_answer_template<'a>(
    recipe: &'a LearnedRecipe,
    ctx: &RecipeContext,
) -> (&'a AnswerTemplate, f32) {
    // Check for critical condition
    if let Some(critical) = &recipe.answer_critical {
        if let Some(is_critical) = ctx.variables.get("is_critical") {
            if is_critical == "true" {
                return (critical, 0.9);
            }
        }
    }

    // Default to ok template
    (&recipe.answer_ok, 0.95)
}
