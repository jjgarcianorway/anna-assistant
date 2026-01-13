//! System Models - Explicit graphs and state machines for system understanding.
//!
//! Anna maintains mental models of system state as explicit graphs:
//! - Service graph: units, dependencies, failure states, log signatures
//! - Network model: interfaces, routes, DNS, firewall, reachability
//! - Storage model: block devices, mounts, fstab, SMART status
//! - Package model: official vs AUR, file ownership, hooks, upgrade risks

pub mod network;
pub mod package;
pub mod service;
pub mod storage;

pub use network::*;
pub use package::*;
pub use service::*;
pub use storage::*;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A unified system model combining all sub-models
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemModel {
    /// Service dependency graph
    pub services: ServiceGraph,
    /// Network topology model
    pub network: NetworkModel,
    /// Storage hierarchy model
    pub storage: StorageModel,
    /// Package dependency model
    pub packages: PackageModel,
    /// When this model was last updated
    pub last_updated: Option<String>,
    /// Model confidence scores by component
    pub confidence: HashMap<String, f32>,
}

impl SystemModel {
    /// Create empty system model
    pub fn new() -> Self {
        Self::default()
    }

    /// Update the timestamp
    pub fn mark_updated(&mut self) {
        self.last_updated = Some(chrono::Utc::now().to_rfc3339());
    }

    /// Set confidence for a component
    pub fn set_confidence(&mut self, component: &str, confidence: f32) {
        self.confidence
            .insert(component.to_string(), confidence.clamp(0.0, 1.0));
    }

    /// Get overall model health
    pub fn health_summary(&self) -> ModelHealth {
        let service_issues = self.services.count_failed();
        let network_issues = self.network.count_unreachable();
        let storage_issues = self.storage.count_unhealthy();
        let package_issues = self.packages.count_risks();

        let total_issues = service_issues + network_issues + storage_issues + package_issues;

        ModelHealth {
            service_issues,
            network_issues,
            storage_issues,
            package_issues,
            total_issues,
            status: if total_issues == 0 {
                HealthStatus::Healthy
            } else if total_issues < 3 {
                HealthStatus::Warning
            } else {
                HealthStatus::Critical
            },
        }
    }
}

/// Health summary of the system model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelHealth {
    pub service_issues: usize,
    pub network_issues: usize,
    pub storage_issues: usize,
    pub package_issues: usize,
    pub total_issues: usize,
    pub status: HealthStatus,
}

/// Overall health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_model_creation() {
        let mut model = SystemModel::new();
        model.mark_updated();
        assert!(model.last_updated.is_some());
    }

    #[test]
    fn test_confidence_clamping() {
        let mut model = SystemModel::new();
        model.set_confidence("services", 1.5);
        assert_eq!(model.confidence.get("services"), Some(&1.0));

        model.set_confidence("network", -0.5);
        assert_eq!(model.confidence.get("network"), Some(&0.0));
    }
}
