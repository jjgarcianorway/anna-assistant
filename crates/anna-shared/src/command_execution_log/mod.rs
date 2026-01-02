//! Command Execution Logging - Phase 77
//!
//! Tracks commands executed by Anna for auditing, statistics, and learning.
//! VISION.md mentions Anna running commands and keeping track of actions.

mod format;
mod log;
mod queries;
mod types;
mod utils;

// Re-export public API
pub use format::{format_execution_log, format_execution_log_compact, format_execution_log_oneline};
pub use log::ExecutionLog;
pub use queries::{execution_fun_fact, is_execution_log_query};
pub use types::{CommandRisk, ExecStatus, ExecutionRecord};
pub use utils::{classify_risk, extract_command_pattern};

#[cfg(test)]
mod tests;
