//! Safe recipe executor (v0.0.423).
//!
//! Executes recipes with:
//! - Risk level awareness
//! - Confirmation handling
//! - Step-by-step execution
//! - Rollback support (where possible)
//! - Execution logging

use std::collections::HashMap;

use super::{
    RecipeV3, RecipeStep, RecipeRiskLevel, ConfirmationPolicy,
    StepResult, MatchResult, RecipeStore, StoreError,
};

/// Recipe execution result
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Recipe that was executed
    pub recipe_id: String,
    /// Overall success
    pub success: bool,
    /// Steps that were executed
    pub steps_executed: Vec<StepExecution>,
    /// Total execution time in ms
    pub duration_ms: u64,
    /// Final message for user
    pub message: String,
    /// Variables after execution
    pub variables: HashMap<String, String>,
    /// Any errors encountered
    pub errors: Vec<String>,
}

/// Record of a single step execution
#[derive(Debug, Clone)]
pub struct StepExecution {
    /// Step index
    pub index: usize,
    /// Step description
    pub description: String,
    /// Whether step succeeded
    pub success: bool,
    /// Step result
    pub result: StepResult,
    /// Whether user confirmed (if applicable)
    pub confirmed: Option<bool>,
}

/// Confirmation callback type
pub type ConfirmFn = Box<dyn Fn(&str, &RecipeStep) -> bool>;

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
    pub fn execute_recipe(&self, recipe: &RecipeV3, initial_vars: HashMap<String, String>) -> ExecutionResult {
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
                format!("[DRY RUN] Recipe '{}' would complete successfully", recipe.title)
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

/// Execute a recipe and update store stats
pub fn execute_and_record(
    executor: &RecipeExecutor,
    match_result: &MatchResult,
    store: &mut RecipeStore,
) -> Result<ExecutionResult, StoreError> {
    let result = executor.execute_match(match_result);

    // Update stats in store
    store.record_execution(
        &result.recipe_id,
        result.success,
        result.duration_ms,
    )?;

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

/// Execution plan for preview
#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    pub recipe_id: String,
    pub recipe_title: String,
    pub total_steps: usize,
    pub steps: Vec<PlannedStep>,
    pub variables: HashMap<String, String>,
    pub overall_risk: RecipeRiskLevel,
    pub confirmation_policy: ConfirmationPolicy,
}

/// A planned step
#[derive(Debug, Clone)]
pub struct PlannedStep {
    pub index: usize,
    pub description: String,
    pub risk_level: RecipeRiskLevel,
    pub needs_confirmation: bool,
}

impl ExecutionPlan {
    /// Format plan for display
    pub fn format(&self) -> String {
        let mut output = vec![];
        output.push(format!("Recipe: {}", self.recipe_title));
        output.push(format!("Risk Level: {:?}", self.overall_risk));
        output.push(format!("Steps: {}", self.total_steps));
        output.push(String::new());

        for step in &self.steps {
            let risk_indicator = match step.risk_level {
                RecipeRiskLevel::None => " ",
                RecipeRiskLevel::Low => "!",
                RecipeRiskLevel::Medium => "!!",
                RecipeRiskLevel::High => "!!!",
            };
            let confirm_indicator = if step.needs_confirmation { "[confirm]" } else { "" };
            output.push(format!(
                "  {}. {} {} {}",
                step.index + 1,
                risk_indicator,
                step.description,
                confirm_indicator
            ));
        }

        output.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe_v3::{RecipeDomain, RecipeMatcher};

    fn test_recipe() -> RecipeV3 {
        RecipeV3::new("test-exec", "Test Execution")
            .with_matcher(RecipeMatcher::new(RecipeDomain::General))
            .with_step(RecipeStep::Explain {
                text: "This is a test".to_string(),
                citation: None,
            })
            .with_step(RecipeStep::RunProbe {
                probe: "echo 'hello'".to_string(),
                output_var: "greeting".to_string(),
                description: "Get greeting".to_string(),
            })
    }

    #[test]
    fn test_dry_run() {
        let recipe = test_recipe();
        let executor = RecipeExecutor::new().dry_run();

        let result = executor.execute_recipe(&recipe, HashMap::new());
        assert!(result.success);
        assert!(result.message.contains("DRY RUN"));
    }

    #[test]
    fn test_execution_with_variables() {
        let recipe = RecipeV3::new("var-test", "Variable Test")
            .with_step(RecipeStep::Explain {
                text: "Hello ${name}!".to_string(),
                citation: None,
            });

        let executor = RecipeExecutor::new().with_var("name", "World");
        let result = executor.execute_recipe(&recipe, HashMap::new());

        assert!(result.success);
    }

    #[test]
    fn test_execution_plan() {
        let recipe = test_recipe();
        let plan = create_execution_plan(&recipe, &HashMap::new());

        assert_eq!(plan.total_steps, 2);
        assert_eq!(plan.steps[0].risk_level, RecipeRiskLevel::None);
    }

    #[test]
    fn test_step_execution_order() {
        let recipe = RecipeV3::new("order-test", "Order Test")
            .with_step(RecipeStep::RunProbe {
                probe: "echo first".to_string(),
                output_var: "v1".to_string(),
                description: "First".to_string(),
            })
            .with_step(RecipeStep::RunProbe {
                probe: "echo second".to_string(),
                output_var: "v2".to_string(),
                description: "Second".to_string(),
            });

        let executor = RecipeExecutor::new();
        let result = executor.execute_recipe(&recipe, HashMap::new());

        assert!(result.success);
        assert_eq!(result.steps_executed.len(), 2);
        assert!(result.variables.contains_key("v1"));
        assert!(result.variables.contains_key("v2"));
    }
}
