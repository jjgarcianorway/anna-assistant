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
