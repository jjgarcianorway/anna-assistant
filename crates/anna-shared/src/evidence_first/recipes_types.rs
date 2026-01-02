//! Recipe template types and core structures (v0.0.435).

use super::probe_plan::ProbeOutput;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A recipe template - parameterized solution pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeTemplate {
    /// Unique recipe ID.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// What problem this solves.
    pub problem_pattern: String,
    /// Required probes to run.
    pub required_probes: Vec<String>,
    /// Conditions that must be met (probe_id -> expected pattern).
    pub preconditions: HashMap<String, String>,
    /// Solution steps (parameterized).
    pub steps: Vec<RecipeStep>,
    /// Expected outcome.
    pub expected_outcome: String,
    /// Tags for matching.
    pub tags: Vec<String>,
}

impl RecipeTemplate {
    /// Create a new recipe template.
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            problem_pattern: String::new(),
            required_probes: Vec::new(),
            preconditions: HashMap::new(),
            steps: Vec::new(),
            expected_outcome: String::new(),
            tags: Vec::new(),
        }
    }

    /// Set problem pattern.
    pub fn with_problem(mut self, pattern: &str) -> Self {
        self.problem_pattern = pattern.to_string();
        self
    }

    /// Add required probe.
    pub fn with_probe(mut self, probe_id: &str) -> Self {
        self.required_probes.push(probe_id.to_string());
        self
    }

    /// Add precondition.
    pub fn with_precondition(mut self, probe_id: &str, pattern: &str) -> Self {
        self.preconditions
            .insert(probe_id.to_string(), pattern.to_string());
        self
    }

    /// Add solution step.
    pub fn with_step(mut self, step: RecipeStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Set expected outcome.
    pub fn with_outcome(mut self, outcome: &str) -> Self {
        self.expected_outcome = outcome.to_string();
        self
    }

    /// Add tag.
    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }

    /// Check if preconditions are met based on probe outputs.
    pub fn check_preconditions(&self, outputs: &[ProbeOutput]) -> bool {
        for (probe_id, pattern) in &self.preconditions {
            let found = outputs.iter().any(|o| {
                o.primitive_id == *probe_id
                    && o.success()
                    && o.raw_output
                        .to_lowercase()
                        .contains(&pattern.to_lowercase())
            });
            if !found {
                return false;
            }
        }
        true
    }

    /// Instantiate recipe with parameters.
    pub fn instantiate(&self, params: &HashMap<String, String>) -> RecipeInstance {
        let steps: Vec<String> = self
            .steps
            .iter()
            .map(|s| super::recipes_helpers::substitute_params(&s.instruction, params))
            .collect();

        RecipeInstance {
            recipe_id: self.id.clone(),
            parameters: params.clone(),
            steps,
            current_step: 0,
            outcome: None,
        }
    }
}

/// A step in a recipe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeStep {
    /// Step number (1-indexed).
    pub number: u8,
    /// Instruction (may contain {param} placeholders).
    pub instruction: String,
    /// Command to run (if any).
    pub command: Option<String>,
    /// Whether this step requires confirmation.
    pub requires_confirmation: bool,
    /// Expected result pattern.
    pub expected_result: Option<String>,
}

impl RecipeStep {
    /// Create a new recipe step.
    pub fn new(number: u8, instruction: &str) -> Self {
        Self {
            number,
            instruction: instruction.to_string(),
            command: None,
            requires_confirmation: false,
            expected_result: None,
        }
    }

    /// With command to execute.
    pub fn with_command(mut self, cmd: &str) -> Self {
        self.command = Some(cmd.to_string());
        self
    }

    /// Require user confirmation.
    pub fn with_confirmation(mut self) -> Self {
        self.requires_confirmation = true;
        self
    }

    /// With expected result.
    pub fn with_expected(mut self, pattern: &str) -> Self {
        self.expected_result = Some(pattern.to_string());
        self
    }
}

/// An instantiated recipe ready for execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeInstance {
    /// Recipe ID.
    pub recipe_id: String,
    /// Parameters used.
    pub parameters: HashMap<String, String>,
    /// Instantiated steps.
    pub steps: Vec<String>,
    /// Current step index.
    pub current_step: usize,
    /// Final outcome.
    pub outcome: Option<RecipeOutcome>,
}

impl RecipeInstance {
    /// Get next step.
    pub fn next_step(&self) -> Option<&str> {
        self.steps.get(self.current_step).map(|s| s.as_str())
    }

    /// Advance to next step.
    pub fn advance(&mut self) {
        if self.current_step < self.steps.len() {
            self.current_step += 1;
        }
    }

    /// Mark as complete.
    pub fn complete(&mut self, success: bool) {
        self.outcome = Some(if success {
            RecipeOutcome::Success
        } else {
            RecipeOutcome::Failed
        });
    }

    /// Check if complete.
    pub fn is_complete(&self) -> bool {
        self.outcome.is_some()
    }
}

/// Outcome of recipe execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecipeOutcome {
    /// Recipe succeeded.
    Success,
    /// Recipe failed.
    Failed,
    /// User cancelled.
    Cancelled,
}
