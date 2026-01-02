//! Data types for recipe execution.

use crate::recipe_schema::PlanStep;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result of recipe execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Whether execution succeeded
    pub success: bool,
    /// Human-readable summary
    pub summary: String,
    /// Steps that were executed
    pub steps_executed: Vec<StepResult>,
    /// Whether rollback was performed
    pub rolled_back: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Result of executing a single step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    /// Step type name
    pub step_type: String,
    /// Whether step succeeded
    pub success: bool,
    /// Output or error message
    pub message: String,
    /// Whether this step can be rolled back
    pub rollback_available: bool,
}

/// Precondition check result.
#[derive(Debug, Clone)]
pub struct PreconditionResult {
    pub all_met: bool,
    pub failed: Vec<String>,
    pub passed: Vec<String>,
}

/// Recipe execution context.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Probe results available
    pub probes: HashMap<String, String>,
    /// User-provided parameters (for parameterized recipes)
    pub params: HashMap<String, String>,
    /// Whether user confirmed execution
    pub user_confirmed: bool,
    /// Dry run mode (don't actually execute)
    pub dry_run: bool,
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self {
            probes: HashMap::new(),
            params: HashMap::new(),
            user_confirmed: false,
            dry_run: false,
        }
    }
}

/// Execution plan ready for the transaction engine.
#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    pub recipe_id: String,
    pub steps: Vec<ExecutionStep>,
    pub rollback_on_failure: bool,
}

/// A single execution step with expanded paths.
#[derive(Debug, Clone)]
pub struct ExecutionStep {
    pub step: PlanStep,
    pub expanded_paths: HashMap<String, String>,
}
