//! RPC method enum and daemon info (v0.0.811).

use serde::{Deserialize, Serialize};

use crate::version::VersionInfo;

/// RPC methods supported by annad
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RpcMethod {
    Status,
    Request,
    Reset,
    Uninstall,
    Autofix,
    Probe,
    /// Get progress events for current/last request
    Progress,
    /// Get per-team statistics (v0.0.27)
    Stats,
    /// Get comprehensive status snapshot (v0.0.29)
    StatusSnapshot,
    /// v0.0.73: Get daemon version info (for client/daemon version comparison)
    GetDaemonInfo,
    /// v0.0.95: Plan a config change (returns ChangePlan for user confirmation)
    PlanChange,
    /// v0.0.95: Apply a confirmed change plan
    ApplyChange,
    /// v0.0.95: Rollback a change using backup
    RollbackChange,
    /// v0.0.275: Generate personalized greeting via translator LLM
    GenerateGreeting,
    /// v0.0.312: Execute a user-approved command (runs as daemon with elevated privileges)
    ExecuteCommand,
    /// v0.0.401: Submit feedback for a completed request (helpful/not helpful)
    SubmitFeedback,
    /// Submit feedback for a claim in the TruthLedger
    SubmitClaimFeedback,
    /// Get status of the TruthLedger
    GetTruthLedgerStatus,
    /// Get filtered claims from the TruthLedger (v0.0.449)
    GetTruthLedgerClaims,
    /// Perform a web search
    WebSearch,
}

/// v0.0.73: Response from GetDaemonInfo RPC call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonInfo {
    /// Daemon version info
    pub version_info: VersionInfo,
    /// Daemon process ID
    pub pid: u32,
    /// Daemon uptime in seconds
    pub uptime_secs: u64,
}
