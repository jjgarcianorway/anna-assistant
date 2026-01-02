//! Plan step types for recipe execution.
//!
//! Plan steps define the individual actions that a recipe performs.

use serde::{Deserialize, Serialize};

/// A step in the recipe plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PlanStep {
    /// Explain something to the user (no action)
    Explain { message: String },
    /// Backup a file before modification
    BackupFile { path: String },
    /// Append a line to a file
    AppendLine { path: String, line: String },
    /// Prepend a line to a file
    PrependLine { path: String, line: String },
    /// Replace a line matching pattern
    ReplaceLine {
        path: String,
        pattern: String,
        replacement: String,
    },
    /// Ensure a line exists (add if missing)
    EnsureLine { path: String, line: String },
    /// Remove lines matching pattern
    RemoveLines { path: String, pattern: String },
    /// Run a command (read-only, for verification)
    VerifyCommand {
        command: String,
        expect_success: bool,
    },
    /// Run a command that changes system state
    RunCommand {
        command: String,
        description: String,
        rollback_command: Option<String>,
    },
    /// Enable a systemd service
    EnableService { service: String, start: bool },
    /// Disable a systemd service
    DisableService { service: String, stop: bool },
    /// Restart a systemd service
    RestartService { service: String },
    /// Create a directory
    CreateDir { path: String, mode: Option<String> },
    /// Create or overwrite a file
    WriteFile {
        path: String,
        content: String,
        mode: Option<String>,
    },
    /// Set environment variable (in shell config)
    SetEnvVar {
        name: String,
        value: String,
        shell_config: String,
    },
}

impl PlanStep {
    /// Get the step type name
    pub fn type_name(&self) -> &'static str {
        match self {
            PlanStep::Explain { .. } => "explain",
            PlanStep::BackupFile { .. } => "backup_file",
            PlanStep::AppendLine { .. } => "append_line",
            PlanStep::PrependLine { .. } => "prepend_line",
            PlanStep::ReplaceLine { .. } => "replace_line",
            PlanStep::EnsureLine { .. } => "ensure_line",
            PlanStep::RemoveLines { .. } => "remove_lines",
            PlanStep::VerifyCommand { .. } => "verify_command",
            PlanStep::RunCommand { .. } => "run_command",
            PlanStep::EnableService { .. } => "enable_service",
            PlanStep::DisableService { .. } => "disable_service",
            PlanStep::RestartService { .. } => "restart_service",
            PlanStep::CreateDir { .. } => "create_dir",
            PlanStep::WriteFile { .. } => "write_file",
            PlanStep::SetEnvVar { .. } => "set_env_var",
        }
    }

    /// Check if this step modifies the system
    pub fn is_mutating(&self) -> bool {
        !matches!(
            self,
            PlanStep::Explain { .. } | PlanStep::VerifyCommand { .. }
        )
    }
}
