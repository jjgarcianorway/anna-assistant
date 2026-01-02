//! Recipe Executor - Safe execution of learned recipes (v0.0.412).
//!
//! Executes recipe steps with:
//! - Safety checks and user confirmation
//! - Dependency resolution
//! - Output collection for rendering
//! - Audit trail logging
//!
//! This module is split into smaller files for maintainability:
//! - `types.rs` - Core data types (ExecutionResult, StepOutput, ExecutionContext, etc.)
//! - `utils.rs` - Utility functions (parameter substitution, condition evaluation, etc.)
//! - `file_ops.rs` - File operations (backup, append, prepend)
//! - `step_handlers.rs` - Step execution handlers (run_probe, run_command, etc.)
//! - `executor.rs` - Main execution orchestration logic

mod executor;
mod file_ops;
mod step_handlers;
mod types;
mod utils;

// Re-export public types and functions
pub use step_handlers::RecipeExecutor;
pub use types::{AuditEntry, ExecutionContext, ExecutionResult, StepOutput};
