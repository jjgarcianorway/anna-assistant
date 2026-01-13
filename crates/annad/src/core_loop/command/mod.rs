//! Command execution and retry logic.
//! v0.0.919: Added auto-tool installation when command not found

mod answer;
mod errors;
mod execute;
mod output;
mod package_map;
mod parallel;
mod retry;
mod types;

// Re-exports
pub use answer::{clean_answer, verify_answer_quality};
pub use errors::{classify_command_error, get_recovery_prompt};
pub use execute::execute_command;
pub use output::strip_ansi_codes;
pub use parallel::execute_commands_parallel;
pub use retry::execute_command_with_retry;
pub use types::CommandErrorType;
