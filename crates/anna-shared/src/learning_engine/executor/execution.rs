//! Recipe execution logic.

use super::probe::ProbeExecutor;
use super::types::ExecutionResult;
use super::variables::extract_variables_from_output;
use crate::learning_engine::{LearnedRecipe, RecipeMatch};
use std::collections::HashMap;

/// Execute a recipe
pub fn execute_recipe<E: ProbeExecutor>(
    recipe: &LearnedRecipe,
    match_result: &RecipeMatch,
    executor: &E,
) -> ExecutionResult {
    let start = std::time::Instant::now();

    // Merge params from match with any defaults
    let mut params = match_result.params.clone();

    // Execute required probes
    let mut probe_results = HashMap::new();
    let mut all_outputs = HashMap::new();

    for probe in &recipe.probes {
        let result = executor.execute(probe, &params);

        if !result.success && !probe.optional {
            // Required probe failed
            let elapsed = start.elapsed().as_millis() as u64;
            return ExecutionResult::failure(
                &recipe.id,
                &format!("Required probe {} failed: {:?}", probe.id, result.error),
            )
            .with_time(elapsed);
        }

        if result.success {
            // Extract variables from probe output
            let extracted = extract_variables_from_output(&probe.id, &result.output);
            all_outputs.extend(extracted);
        }

        probe_results.insert(probe.id.clone(), result);
    }

    // Merge extracted values with params
    params.extend(all_outputs.clone());

    // Fill answer templates
    let short = recipe.answer_template.render_short(&params);
    let detailed = recipe.answer_template.render_detailed(&params);

    let elapsed = start.elapsed().as_millis() as u64;

    let mut result = ExecutionResult::success(&recipe.id, &short, &detailed);
    result.probe_results = probe_results;
    result.variables = params;
    result.execution_ms = elapsed;

    result
}

/// Check if all recipe requirements are met
pub fn can_execute(recipe: &LearnedRecipe, match_result: &RecipeMatch) -> bool {
    // Check if strong match
    if !match_result.is_strong() {
        return false;
    }

    // Check if all required params are available
    for param_name in recipe.inputs.required_params() {
        if !match_result.params.contains_key(&param_name) {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::learning_engine::{AnswerTemplate, RecipePattern, RecipeProbe};
    use crate::learning_engine::executor::types::ProbeResult;
    use std::collections::HashMap;

    struct MockExecutor;

    impl ProbeExecutor for MockExecutor {
        fn execute(&self, probe: &RecipeProbe, _params: &HashMap<String, String>) -> ProbeResult {
            match probe.id.as_str() {
                "probe:free" => ProbeResult::ok(
                    "probe:free",
                    "              total        used        free      shared  buff/cache   available\nMem:           16Gi       8.0Gi       4.0Gi       1.0Gi       4.0Gi       7.0Gi",
                    50,
                ),
                _ => ProbeResult::ok(&probe.id, "mock output", 10),
            }
        }
    }

    fn make_recipe() -> LearnedRecipe {
        LearnedRecipe::new("test-ram", "performance.memory")
            .with_pattern(RecipePattern::new("check_free_ram"))
            .with_probe(RecipeProbe::new("probe:free", "probe.free"))
            .with_answer(
                "Available RAM: {{available_mem}}",
                "Memory Status:\n  Total: {{total_mem}}\n  Used: {{used_mem}}\n  Available: {{available_mem}}",
            )
    }

    fn make_match(recipe_id: &str, score: f32) -> RecipeMatch {
        RecipeMatch {
            recipe_id: recipe_id.to_string(),
            score,
            breakdown: Default::default(),
            params: HashMap::new(),
            missing_signals: vec![],
        }
    }

    #[test]
    fn test_execute_recipe() {
        let recipe = make_recipe();
        let match_result = make_match("test-ram", 0.9);
        let executor = MockExecutor;

        let result = execute_recipe(&recipe, &match_result, &executor);

        assert!(result.success);
        assert!(result.short_answer.contains("7.0Gi"));
        assert!(result.recipe_based);
    }

    #[test]
    fn test_can_execute() {
        let recipe = make_recipe();
        let strong_match = make_match("test-ram", 0.9);
        let weak_match = make_match("test-ram", 0.5);

        assert!(can_execute(&recipe, &strong_match));
        assert!(!can_execute(&recipe, &weak_match));
    }
}
