//! Recipe executor for learning engine (v0.0.427).
//!
//! Executes recipes without calling LLM:
//! - Runs probes
//! - Fills answer templates
//! - Tracks success/failure

mod execution;
mod probe;
mod types;
mod variables;

pub use execution::{can_execute, execute_recipe};
pub use probe::{ProbeExecutor, ShellProbeExecutor};
pub use types::{ExecutionResult, ProbeResult};
pub use variables::extract_variables_from_output;
