//! Recovery plan types and data structures
//!
//! Phase 0.6: Recovery Framework - Type definitions
//! Citation: [archwiki:System_maintenance#Backup]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Recovery plan definition loaded from YAML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryPlan {
    /// Plan identifier (e.g., "bootloader", "initramfs")
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Arch Wiki citation for this recovery plan
    pub citation: String,
    /// Ordered list of recovery steps
    pub steps: Vec<RecoveryStep>,
}

/// Individual recovery step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryStep {
    /// Step identifier
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Whether this step modifies system state
    #[serde(default)]
    pub destructive: bool,
    /// Optional command to execute
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// Optional command arguments
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    /// Optional pre-execution check command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check: Option<String>,
    /// Optional environment variables
    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
}

/// Recovery execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryResult {
    /// Plan identifier
    pub plan_id: String,
    /// Execution UUID
    pub run_id: String,
    /// ISO 8601 timestamp
    pub timestamp: String,
    /// Whether execution succeeded
    pub success: bool,
    /// Steps executed
    pub steps_executed: Vec<StepResult>,
    /// Path to rollback script
    pub rollback_script: Option<String>,
    /// Error message if failed
    pub error: Option<String>,
}

/// Individual step execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    /// Step name
    pub step: String,
    /// Whether step succeeded
    pub success: bool,
    /// Exit code (if command was run)
    pub exit_code: Option<i32>,
    /// Stdout output
    pub stdout: String,
    /// Stderr output
    pub stderr: String,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Pre-execution state snapshot
    pub pre_state: Option<StateSnapshot>,
    /// Post-execution state snapshot
    pub post_state: Option<StateSnapshot>,
}

/// State snapshot for rollback generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    /// Files modified or created
    pub files: Vec<FileState>,
    /// Packages installed/removed
    pub packages: Vec<PackageState>,
    /// Services enabled/disabled
    pub services: Vec<ServiceState>,
}

/// File state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileState {
    pub path: String,
    pub exists: bool,
    pub content_hash: Option<String>,
    pub permissions: Option<u32>,
}

/// Package state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageState {
    pub name: String,
    pub version: Option<String>,
    pub installed: bool,
}

/// Service state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceState {
    pub name: String,
    pub enabled: bool,
    pub active: bool,
}

/// Rollback script metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackMetadata {
    /// Rollback script identifier
    pub rollback_id: String,
    /// Original recovery plan ID
    pub plan_id: String,
    /// When recovery was executed
    pub executed_at: String,
    /// Path to rollback script
    pub script_path: String,
    /// Path to state diff
    pub state_diff_path: String,
    /// Whether rollback is available
    pub available: bool,
    /// Reason if unavailable
    pub unavailable_reason: Option<String>,
}
