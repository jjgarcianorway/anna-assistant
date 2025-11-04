//! Core data types for Anna Assistant

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Risk level for actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

/// Autonomy tier configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutonomyTier {
    /// Tier 0: Advise only, never execute
    AdviseOnly = 0,
    /// Tier 1: Auto-execute Low risk only
    SafeAutoApply = 1,
    /// Tier 2: Auto-execute Low + Medium risk
    SemiAutonomous = 2,
    /// Tier 3: Auto-execute all risk levels
    FullyAutonomous = 3,
}

impl AutonomyTier {
    /// Check if this tier allows auto-execution for a given risk level
    pub fn allows(&self, risk: RiskLevel) -> bool {
        match (self, risk) {
            (AutonomyTier::AdviseOnly, _) => false,
            (AutonomyTier::SafeAutoApply, RiskLevel::Low) => true,
            (AutonomyTier::SafeAutoApply, _) => false,
            (AutonomyTier::SemiAutonomous, RiskLevel::High) => false,
            (AutonomyTier::SemiAutonomous, _) => true,
            (AutonomyTier::FullyAutonomous, _) => true,
        }
    }
}

/// System facts collected by telemetry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemFacts {
    pub timestamp: DateTime<Utc>,
    pub hostname: String,
    pub kernel: String,
    pub cpu_model: String,
    pub cpu_cores: usize,
    pub total_memory_gb: f64,
    pub gpu_vendor: Option<String>,
    pub storage_devices: Vec<StorageDevice>,
    pub installed_packages: usize,
    pub orphan_packages: Vec<String>,
    pub network_interfaces: Vec<String>,
}

/// Storage device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageDevice {
    pub name: String,
    pub filesystem: String,
    pub size_gb: f64,
    pub used_gb: f64,
    pub mount_point: String,
}

/// A single piece of advice from the recommendation engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Advice {
    pub id: String,
    pub title: String,
    pub reason: String,
    pub action: String,
    pub command: Option<String>,
    pub risk: RiskLevel,
    pub wiki_refs: Vec<String>,
}

/// An action to be executed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub id: String,
    pub advice_id: String,
    pub command: String,
    pub executed_at: DateTime<Utc>,
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

/// Rollback token for reversing an action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackToken {
    pub action_id: String,
    pub advice_id: String,
    pub executed_at: DateTime<Utc>,
    pub command: String,
    pub rollback_command: Option<String>,
    pub snapshot_before: Option<String>,
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: DateTime<Utc>,
    pub actor: String,
    pub action_type: String,
    pub details: String,
    pub success: bool,
}

/// RPC Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    pub method: String,
    pub params: serde_json::Value,
}

/// RPC Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    pub success: bool,
    pub data: serde_json::Value,
    pub error: Option<String>,
}
