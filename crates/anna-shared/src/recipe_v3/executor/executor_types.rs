//! Recipe executor type definitions.

use std::collections::HashMap;

use crate::recipe_v3::{
    ConfirmationPolicy, RecipeRiskLevel, RecipeStep, StepResult,
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
            let confirm_indicator = if step.needs_confirmation {
                "[confirm]"
            } else {
                ""
            };
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
