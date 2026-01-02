//! Safe recipe executor (v0.0.423).
//!
//! Executes recipes with:
//! - Risk level awareness
//! - Confirmation handling
//! - Step-by-step execution
//! - Rollback support (where possible)
//! - Execution logging

mod executor_core;
mod executor_helpers;
mod executor_types;

#[cfg(test)]
mod tests;

// Re-export public types and functions
pub use executor_core::RecipeExecutor;
pub use executor_helpers::{create_execution_plan, execute_and_record};
pub use executor_types::{
    ConfirmFn, ExecutionPlan, ExecutionResult, PlannedStep, StepExecution,
};
