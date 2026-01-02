//! Core types for command execution logging

use serde::{Deserialize, Serialize};

/// Execution result status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecStatus {
    Success,
    Failed,
    Timeout,
    Cancelled,
    Pending,
}

impl ExecStatus {
    pub fn symbol(&self) -> &'static str {
        match self {
            ExecStatus::Success => "+",
            ExecStatus::Failed => "x",
            ExecStatus::Timeout => "!",
            ExecStatus::Cancelled => "-",
            ExecStatus::Pending => "?",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ExecStatus::Success => "completed successfully",
            ExecStatus::Failed => "failed",
            ExecStatus::Timeout => "timed out",
            ExecStatus::Cancelled => "was cancelled",
            ExecStatus::Pending => "is pending",
        }
    }
}

/// Risk level for command execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandRisk {
    ReadOnly,
    LowRisk,
    MediumRisk,
    HighRisk,
    Critical,
}

impl CommandRisk {
    pub fn level(&self) -> u8 {
        match self {
            CommandRisk::ReadOnly => 0,
            CommandRisk::LowRisk => 1,
            CommandRisk::MediumRisk => 2,
            CommandRisk::HighRisk => 3,
            CommandRisk::Critical => 4,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            CommandRisk::ReadOnly => "read-only",
            CommandRisk::LowRisk => "low risk",
            CommandRisk::MediumRisk => "medium risk",
            CommandRisk::HighRisk => "high risk",
            CommandRisk::Critical => "critical",
        }
    }
}

/// A single command execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    /// Unique execution ID
    pub id: String,
    /// The command that was executed
    pub command: String,
    /// Working directory
    pub working_dir: Option<String>,
    /// User who ran the command
    pub user: String,
    /// Whether elevated (sudo) was used
    pub elevated: bool,
    /// Execution status
    pub status: ExecStatus,
    /// Exit code if available
    pub exit_code: Option<i32>,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Timestamp when started
    pub started_at: u64,
    /// Associated ticket ID if any
    pub ticket_id: Option<String>,
    /// Risk level
    pub risk: CommandRisk,
    /// Output excerpt (truncated if long)
    pub output_excerpt: Option<String>,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Whether user confirmed before execution
    pub user_confirmed: bool,
}
