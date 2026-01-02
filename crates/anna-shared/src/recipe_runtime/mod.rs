//! Recipe executor for Anna's learning system.
//! v0.0.418: Executes recipe plans using existing transaction mechanisms.
//!
//! The executor:
//! - Verifies preconditions using probes
//! - Builds a plan for the transaction engine
//! - Prompts user for confirmation if required
//! - Executes steps with rollback on failure
//! - Updates recipe metrics

mod types;
mod preconditions;
mod confirmation;
mod execution;
mod utils;
mod summary;

#[cfg(test)]
mod tests;

// Re-export public types
pub use types::{
    ExecutionResult,
    StepResult,
    PreconditionResult,
    ExecutionContext,
    ExecutionPlan,
    ExecutionStep,
};

// Re-export public functions
pub use preconditions::check_preconditions;
pub use confirmation::{needs_confirmation, generate_confirmation_prompt};
pub use execution::prepare_execution;
pub use summary::generate_recipe_summary;
