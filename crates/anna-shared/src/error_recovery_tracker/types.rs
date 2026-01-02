//! Error recovery types
//!
//! Defines error categories, recovery outcomes, and recovery records.

use serde::{Deserialize, Serialize};

/// Error category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ErrorCategory {
    #[default]
    System,
    Network,
    Permission,
    NotFound,
    Timeout,
    Configuration,
    Dependency,
    Other,
}

impl ErrorCategory {
    pub fn name(&self) -> &'static str {
        match self {
            ErrorCategory::System => "System",
            ErrorCategory::Network => "Network",
            ErrorCategory::Permission => "Permission",
            ErrorCategory::NotFound => "Not Found",
            ErrorCategory::Timeout => "Timeout",
            ErrorCategory::Configuration => "Configuration",
            ErrorCategory::Dependency => "Dependency",
            ErrorCategory::Other => "Other",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            ErrorCategory::System => "⚙",
            ErrorCategory::Network => "⚡",
            ErrorCategory::Permission => "🔒",
            ErrorCategory::NotFound => "?",
            ErrorCategory::Timeout => "⏱",
            ErrorCategory::Configuration => "⚙",
            ErrorCategory::Dependency => "→",
            ErrorCategory::Other => "·",
        }
    }
}

/// Recovery outcome
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum RecoveryOutcome {
    #[default]
    Success,
    PartialSuccess,
    Failed,
    Skipped,
    Manual,
}

impl RecoveryOutcome {
    pub fn name(&self) -> &'static str {
        match self {
            RecoveryOutcome::Success => "Success",
            RecoveryOutcome::PartialSuccess => "Partial",
            RecoveryOutcome::Failed => "Failed",
            RecoveryOutcome::Skipped => "Skipped",
            RecoveryOutcome::Manual => "Manual",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            RecoveryOutcome::Success => "✓",
            RecoveryOutcome::PartialSuccess => "~",
            RecoveryOutcome::Failed => "✗",
            RecoveryOutcome::Skipped => "-",
            RecoveryOutcome::Manual => "→",
        }
    }
}

/// An error recovery record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorRecoveryRecord {
    /// Error ID
    pub id: String,
    /// Error category
    pub category: ErrorCategory,
    /// Error message
    pub error_message: String,
    /// Recovery strategy used
    pub strategy: String,
    /// Recovery outcome
    pub outcome: RecoveryOutcome,
    /// Time taken (ms)
    pub duration_ms: u64,
    /// Number of retry attempts
    pub retry_count: u32,
    /// Timestamp
    pub timestamp: u64,
}
