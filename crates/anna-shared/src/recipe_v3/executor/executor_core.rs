//! Safe recipe executor core implementation (v0.0.423).
//!
//! Executes recipes with:
//! - Risk level awareness
//! - Confirmation handling
//! - Step-by-step execution
//! - Rollback support (where possible)
//! - Execution logging

use std::collections::HashMap;

use crate::recipe_v3::{
    ConfirmationPolicy, MatchResult, RecipeRiskLevel, RecipeStep, RecipeV3, StepResult,
};

use super::executor_types::{ConfirmFn, ExecutionResult, StepExecution};

/// Recipe executor
pub struct RecipeExecutor {
    /// Whether to actually execute commands (false = dry run)
    dry_run: bool,
    /// Confirmation callback (None = auto-confirm safe, reject risky)
    confirm_fn: Option<ConfirmFn>,
    /// Maximum execution time per step in ms
    step_timeout_ms: u64,
    /// Variables to pass to recipes
    variables: HashMap<String, String>,
}

impl Default for RecipeExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl RecipeExecutor {
    /// Create new executor
    pub fn new() -> Self {
        Self {
            dry_run: false,
            confirm_fn: None,
            step_timeout_ms: 30_000,
            variables: HashMap::new(),
        }
    }

    /// Enable dry-run mode (no actual execution)
    pub fn dry_run(mut self) -> Self {
        self.dry_run = true;
        self
    }

    /// Set confirmation callback
    pub fn with_confirm(mut self, f: ConfirmFn) -> Self {
        self.confirm_fn = Some(f);
        self
    }

    /// Set initial variables
    pub fn with_variables(mut self, vars: HashMap<String, String>) -> Self {
        self.variables = vars;
        self
    }

    /// Add a variable
    pub fn with_var(mut self, key: &str, value: &str) -> Self {
        self.variables.insert(key.to_string(), value.to_string());
        self
    }

    /// Execute a recipe from a match result
    pub fn execute_match(&self, match_result: &MatchResult) -> ExecutionResult {
        // Merge extracted variables with executor variables
        let mut vars = self.variables.clone();
        vars.extend(match_result.extracted_vars.clone());

        self.execute_recipe(&match_result.recipe, vars)
    }

    /// Execute a recipe
    pub fn execute_recipe(
        &self,
        recipe: &RecipeV3,
        initial_vars: HashMap<String, String>,
    ) -> ExecutionResult {
        let start = std::time::Instant::now();
        let mut vars = initial_vars;
        let mut steps_executed = vec![];
        let mut errors = vec![];
        let mut all_success = true;

        // Check preconditions
        for precond in &recipe.preconditions {
            let result = precond.evaluate(&vars);
            if !result.success {
                return ExecutionResult {
                    recipe_id: recipe.id.clone(),
                    success: false,
                    steps_executed: vec![],
                    duration_ms: start.elapsed().as_millis() as u64,
                    message: format!("Precondition failed: {}", result.message),
                    variables: vars,
                    errors: vec![result.message],
                };
            }
        }

        // Execute each step
        for (index, step) in recipe.steps.iter().enumerate() {
            let step_risk = step.risk_level();

            // Check if confirmation is needed
            let needs_confirm = self.needs_confirmation(recipe.confirmation, step_risk);
            let confirmed = if needs_confirm {
                self.request_confirmation(&recipe.title, step)
            } else {
                true
            };

            if !confirmed {
                steps_executed.push(StepExecution {
                    index,
                    description: step.describe(),
                    success: false,
                    result: StepResult::fail("User declined"),
                    confirmed: Some(false),
                });
                errors.push(format!("Step {} declined by user", index));
                all_success = false;
                break;
            }

            // Execute the step
            let result = if self.dry_run {
                StepResult::ok(&format!("[DRY RUN] Would execute: {}", step.describe()))
            } else {
                step.execute(&mut vars)
            };

            let step_success = result.success;
            steps_executed.push(StepExecution {
                index,
                description: step.describe(),
                success: step_success,
                result: result.clone(),
                confirmed: if needs_confirm { Some(true) } else { None },
            });

            if !step_success {
                errors.push(result.message.clone());
                all_success = false;
                break;
            }
        }

        // Check postconditions if all steps succeeded
        if all_success && !recipe.postconditions.is_empty() && !self.dry_run {
            for postcond in &recipe.postconditions {
                let result = postcond.evaluate(&vars);
                if !result.success {
                    errors.push(format!("Postcondition failed: {}", result.message));
                    all_success = false;
                    break;
                }
            }
        }

        // Build final message
        let message = if all_success {
            if self.dry_run {
                format!(
                    "[DRY RUN] Recipe '{}' would complete successfully",
                    recipe.title
                )
            } else {
                format!("Recipe '{}' completed successfully", recipe.title)
            }
        } else {
            format!(
                "Recipe '{}' failed: {}",
                recipe.title,
                errors.last().unwrap_or(&"Unknown error".to_string())
            )
        };

        ExecutionResult {
            recipe_id: recipe.id.clone(),
            success: all_success,
            steps_executed,
            duration_ms: start.elapsed().as_millis() as u64,
            message,
            variables: vars,
            errors,
        }
    }

    /// Check if confirmation is needed
    fn needs_confirmation(&self, policy: ConfirmationPolicy, risk: RecipeRiskLevel) -> bool {
        match policy {
            ConfirmationPolicy::Never => false,
            ConfirmationPolicy::Always => true,
            ConfirmationPolicy::Once => false, // Handled at recipe level, not step level
            ConfirmationPolicy::PerStep => risk.requires_confirmation(),
        }
    }

    /// Request confirmation from user
    fn request_confirmation(&self, recipe_title: &str, step: &RecipeStep) -> bool {
        if let Some(ref confirm_fn) = self.confirm_fn {
            confirm_fn(recipe_title, step)
        } else {
            // Default: auto-confirm safe operations, reject risky ones
            !step.risk_level().requires_confirmation()
        }
    }
}
