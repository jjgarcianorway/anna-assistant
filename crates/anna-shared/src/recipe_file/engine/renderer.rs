//! Recipe answer rendering and high-level engine (v0.0.406).

use super::executor::execute_recipe;
use super::matcher::find_matching_recipe;
use super::types::{ExecutionResult, RecipeContext};
use crate::recipe_file::format::FileRecipe;
use crate::rpc::SpecialistDomain;
use std::collections::HashMap;

/// Render the answer template with execution results
pub fn render_answer(recipe: &FileRecipe, result: &ExecutionResult) -> String {
    let mut answer = recipe.answer.template.clone();

    // Apply defaults first
    for (key, default) in &recipe.answer.defaults {
        let placeholder = format!("{{{}}}", key);
        if answer.contains(&placeholder) && !result.variables.contains_key(key) {
            answer = answer.replace(&placeholder, default);
        }
    }

    // Apply extracted variables
    for (key, value) in &result.variables {
        let placeholder = format!("{{{}}}", key);
        answer = answer.replace(&placeholder, value);
    }

    // Add raw output if requested
    if recipe.answer.include_raw_output {
        let mut raw = String::new();
        for step in &result.steps {
            if !step.skipped && !step.stdout.is_empty() {
                raw.push_str(&format!("\n[{}]\n{}", step.id, step.stdout));
            }
        }
        if !raw.is_empty() {
            answer.push_str(&format!("\n\nRaw output:{}", raw));
        }
    }

    answer.trim().to_string()
}

/// Recipe engine combining all operations
#[derive(Debug, Default)]
pub struct RecipeEngine {
    /// Probe lookup function registry
    probe_commands: HashMap<String, String>,
}

impl RecipeEngine {
    /// Create a new engine
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a probe command
    pub fn register_probe(&mut self, id: &str, command: &str) {
        self.probe_commands
            .insert(id.to_string(), command.to_string());
    }

    /// Find and execute matching recipe
    pub fn run(
        &self,
        domain: SpecialistDomain,
        intent: &str,
        params: &HashMap<String, String>,
        query: &str,
        context: &RecipeContext,
    ) -> Option<(ExecutionResult, String)> {
        let match_result = find_matching_recipe(domain, intent, params, query)?;

        let probe_lookup = |id: &str| self.probe_commands.get(id).cloned();
        let exec_result = execute_recipe(&match_result.recipe, context, probe_lookup);

        if exec_result.success {
            let answer = render_answer(&match_result.recipe, &exec_result);
            Some((exec_result, answer))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe_file::format::*;

    #[test]
    fn test_render_answer() {
        let recipe = FileRecipe {
            id: RecipeId {
                name: "test".to_string(),
                domain: "system".to_string(),
                version: "1".to_string(),
            },
            match_criteria: RecipeMatch {
                intent: "diagnose".to_string(),
                keywords: vec![],
                required_keywords: vec![],
                key: None,
                target: None,
                params: HashMap::new(),
                min_confidence: 60,
            },
            plan: RecipePlan {
                steps: vec![],
                stop_on_error: true,
                backup_paths: vec![],
            },
            answer: RecipeAnswer {
                template: "Found {count} items (default: {missing})".to_string(),
                defaults: [("missing".to_string(), "none".to_string())]
                    .into_iter()
                    .collect(),
                include_raw_output: false,
            },
            meta: Default::default(),
        };

        let mut result = ExecutionResult::empty("test".to_string());
        result
            .variables
            .insert("count".to_string(), "5".to_string());

        let answer = render_answer(&recipe, &result);
        assert_eq!(answer, "Found 5 items (default: none)");
    }
}
