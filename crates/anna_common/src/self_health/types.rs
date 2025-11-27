//! Self-Health Types v0.7.0
//!
//! Type definitions for self-health monitoring and auto-repair.

use serde::{Deserialize, Serialize};

/// Overall health status of Anna
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OverallHealth {
    /// All components functioning normally
    Healthy,
    /// Some components degraded but core functionality works
    Degraded,
    /// Critical failure - Anna cannot function
    Critical,
    /// Status unknown (not yet checked)
    Unknown,
}

/// Health status of a single component
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComponentStatus {
    /// Component is functioning normally
    Healthy,
    /// Component has issues but is still working
    Degraded,
    /// Component has failed
    Critical,
    /// Unable to determine status
    Unknown,
}

/// Health report for a single component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component name (daemon, llm, model, tools, permissions, config)
    pub name: String,
    /// Current status
    pub status: ComponentStatus,
    /// Human-readable message
    pub message: String,
    /// Additional details (JSON)
    pub details: Option<serde_json::Value>,
}

/// Complete self-health report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfHealthReport {
    /// Overall health status
    pub overall: OverallHealth,
    /// Individual component reports
    pub components: Vec<ComponentHealth>,
    /// Repairs that could be attempted
    pub repairs_available: Vec<RepairPlan>,
    /// Repairs that were executed
    pub repairs_executed: Vec<RepairResult>,
}

/// Safety level for a repair action
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepairSafety {
    /// Safe to execute automatically
    Auto,
    /// Requires user confirmation or manual action
    WarnOnly,
}

/// Type of repair action
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepairAction {
    /// Restart the annad daemon
    RestartDaemon,
    /// Restart Ollama service
    RestartOllama,
    /// Pull a missing model
    PullModel(String),
    /// Fix directory permissions
    FixPermissions(String),
    /// Regenerate config file
    RegenerateConfig,
    /// Clear probe cache
    ClearProbeCache,
    /// Custom command
    Custom(String),
}

/// A planned repair action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairPlan {
    /// What action to take
    pub action: RepairAction,
    /// Safety level
    pub safety: RepairSafety,
    /// Human-readable description
    pub description: String,
    /// Command to execute (for display)
    pub command: String,
    /// Target component
    pub target_component: String,
}

/// Result of executing a repair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairResult {
    /// The action that was attempted
    pub action: RepairAction,
    /// Whether the repair succeeded
    pub success: bool,
    /// Result message
    pub message: String,
}

impl Default for SelfHealthReport {
    fn default() -> Self {
        Self {
            overall: OverallHealth::Unknown,
            components: vec![],
            repairs_available: vec![],
            repairs_executed: vec![],
        }
    }
}

impl OverallHealth {
    /// Check if healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self, OverallHealth::Healthy)
    }

    /// Check if critical
    pub fn is_critical(&self) -> bool {
        matches!(self, OverallHealth::Critical)
    }
}

impl ComponentStatus {
    /// Check if healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self, ComponentStatus::Healthy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overall_health_is_healthy() {
        assert!(OverallHealth::Healthy.is_healthy());
        assert!(!OverallHealth::Degraded.is_healthy());
        assert!(!OverallHealth::Critical.is_healthy());
    }

    #[test]
    fn test_overall_health_is_critical() {
        assert!(!OverallHealth::Healthy.is_critical());
        assert!(!OverallHealth::Degraded.is_critical());
        assert!(OverallHealth::Critical.is_critical());
    }

    #[test]
    fn test_component_status_is_healthy() {
        assert!(ComponentStatus::Healthy.is_healthy());
        assert!(!ComponentStatus::Degraded.is_healthy());
        assert!(!ComponentStatus::Critical.is_healthy());
        assert!(!ComponentStatus::Unknown.is_healthy());
    }

    #[test]
    fn test_repair_action_debug() {
        let action = RepairAction::PullModel("llama3.2:3b".to_string());
        let debug_str = format!("{:?}", action);
        assert!(debug_str.contains("PullModel"));
        assert!(debug_str.contains("llama3.2:3b"));
    }

    #[test]
    fn test_default_report() {
        let report = SelfHealthReport::default();
        assert!(report.components.is_empty());
        assert!(report.repairs_available.is_empty());
    }
}
