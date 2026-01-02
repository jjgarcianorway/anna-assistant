//! Service management types - Phase 81

use serde::{Deserialize, Serialize};

/// Service operation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ServiceOperation {
    Start,
    Stop,
    Restart,
    Reload,
    Enable,
    Disable,
    Status,
}

impl ServiceOperation {
    pub fn symbol(&self) -> &'static str {
        match self {
            ServiceOperation::Start => ">",
            ServiceOperation::Stop => "x",
            ServiceOperation::Restart => "~",
            ServiceOperation::Reload => "r",
            ServiceOperation::Enable => "+",
            ServiceOperation::Disable => "-",
            ServiceOperation::Status => "?",
        }
    }

    pub fn verb(&self) -> &'static str {
        match self {
            ServiceOperation::Start => "started",
            ServiceOperation::Stop => "stopped",
            ServiceOperation::Restart => "restarted",
            ServiceOperation::Reload => "reloaded",
            ServiceOperation::Enable => "enabled",
            ServiceOperation::Disable => "disabled",
            ServiceOperation::Status => "checked",
        }
    }
}

/// Result of service operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationResult {
    Success,
    Failed,
    Skipped,
    Pending,
}

impl OperationResult {
    pub fn symbol(&self) -> &'static str {
        match self {
            OperationResult::Success => "+",
            OperationResult::Failed => "x",
            OperationResult::Skipped => "-",
            OperationResult::Pending => "?",
        }
    }
}

/// A single service operation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRecord {
    /// Service name
    pub service_name: String,
    /// Operation performed
    pub operation: ServiceOperation,
    /// Result
    pub result: OperationResult,
    /// Timestamp
    pub timestamp: u64,
    /// Associated ticket ID
    pub ticket_id: Option<String>,
    /// Reason for operation
    pub reason: Option<String>,
    /// Error message if failed
    pub error: Option<String>,
    /// Whether user confirmed
    pub user_confirmed: bool,
}
