//! RPC parameter types (v0.0.220).

use serde::{Deserialize, Serialize};

/// Parameters for the request method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestParams {
    pub prompt: String,
}

/// Parameters for the probe method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeParams {
    pub probe_type: ProbeType,
}

/// Types of probes that can be run
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProbeType {
    TopMemory,
    TopCpu,
    DiskUsage,
    NetworkInterfaces,
}

/// v0.0.95: Parameters for PlanChange RPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanChangeParams {
    /// Path to the config file
    pub config_path: String,
    /// Line to ensure exists
    pub line: String,
}

/// v0.0.95: Parameters for ApplyChange/RollbackChange RPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeParams {
    /// The change plan to apply/rollback
    pub plan: crate::change::ChangePlan,
}

/// v0.0.312: Parameters for ExecuteCommand RPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteCommandParams {
    /// The command to execute
    pub command: String,
    /// Request ID for tracking (from original ServiceDeskResult)
    pub request_id: String,
}

/// v0.0.312: Result of command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandExecutionResult {
    /// Whether command succeeded (exit code 0)
    pub success: bool,
    /// Exit code
    pub exit_code: i32,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Execution time in milliseconds
    pub duration_ms: u64,
}

/// v0.0.401: Parameters for SubmitFeedback RPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackParams {
    /// The request_id from the ServiceDeskResult
    pub request_id: String,
    /// The original query
    pub query: String,
    /// Was the answer helpful?
    pub helpful: bool,
    /// Optional comment from user
    pub comment: Option<String>,
}

/// v0.0.401: Result of feedback submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackResult {
    /// Whether feedback was recorded
    pub recorded: bool,
    /// Optional message about learning
    pub learning_message: Option<String>,
}
