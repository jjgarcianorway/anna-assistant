//! Systemd Action Engine v0.0.51
//!
//! Safe systemd service operations with evidence-driven execution:
//! - Probe service state before any action
//! - Preview impact with risk assessment
//! - Require confirmation phrases by risk level
//! - Execute via systemctl with state capture
//! - Verify outcome with evidence
//! - Provide rollback capability
//!
//! Supported operations:
//! - start_service
//! - stop_service
//! - restart_service
//! - enable_service
//! - disable_service
//!
//! NOT supported in v0.0.51:
//! - mask/unmask
//! - edit unit files
//! - daemon-reload
//! - target changes

use serde::{Deserialize, Serialize};

// Reuse the medium risk confirmation from mutation_tools
pub use crate::mutation_tools::MEDIUM_RISK_CONFIRMATION;

// =============================================================================
// Constants
// =============================================================================

/// Confirmation phrases by risk level
pub const LOW_RISK_CONFIRMATION: &str = "I CONFIRM (low risk)";
// MEDIUM_RISK_CONFIRMATION is re-exported from mutation_tools
pub const HIGH_RISK_CONFIRMATION: &str = "I ASSUME THE RISK";

/// Network-critical services (medium risk)
const NETWORK_SERVICES: &[&str] = &[
    "NetworkManager",
    "iwd",
    "systemd-resolved",
    "wpa_supplicant",
    "dhcpcd",
    "systemd-networkd",
];

/// Critical services (high risk)
const CRITICAL_SERVICES: &[&str] = &[
    "sshd",
    "ssh",
    "display-manager",
    "gdm",
    "sddm",
    "lightdm",
    "ly",
    "xdm",
    "lxdm",
];

/// Core systemd services (deny by default)
const CORE_SYSTEMD_SERVICES: &[&str] = &[
    "systemd-journald",
    "systemd-logind",
    "systemd-udevd",
    "dbus",
    "dbus-broker",
    "systemd-machined",
    "systemd-resolved",
];

// =============================================================================
// Operation Types
// =============================================================================

/// Systemd service operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceOperation {
    Start,
    Stop,
    Restart,
    Enable,
    Disable,
}

impl ServiceOperation {
    pub fn as_str(&self) -> &'static str {
        match self {
            ServiceOperation::Start => "start",
            ServiceOperation::Stop => "stop",
            ServiceOperation::Restart => "restart",
            ServiceOperation::Enable => "enable",
            ServiceOperation::Disable => "disable",
        }
    }

    pub fn past_tense(&self) -> &'static str {
        match self {
            ServiceOperation::Start => "started",
            ServiceOperation::Stop => "stopped",
            ServiceOperation::Restart => "restarted",
            ServiceOperation::Enable => "enabled",
            ServiceOperation::Disable => "disabled",
        }
    }

    pub fn inverse(&self) -> Self {
        match self {
            ServiceOperation::Start => ServiceOperation::Stop,
            ServiceOperation::Stop => ServiceOperation::Start,
            ServiceOperation::Restart => ServiceOperation::Restart, // No inverse
            ServiceOperation::Enable => ServiceOperation::Disable,
            ServiceOperation::Disable => ServiceOperation::Enable,
        }
    }
}

impl std::fmt::Display for ServiceOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// =============================================================================
// Risk Level
// =============================================================================

/// Risk level for systemd operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Denied,
}

impl RiskLevel {
    pub fn confirmation_phrase(&self) -> Option<&'static str> {
        match self {
            RiskLevel::Low => Some(LOW_RISK_CONFIRMATION),
            RiskLevel::Medium => Some(MEDIUM_RISK_CONFIRMATION),
            RiskLevel::High => Some(HIGH_RISK_CONFIRMATION),
            RiskLevel::Denied => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            RiskLevel::Low => "low",
            RiskLevel::Medium => "medium",
            RiskLevel::High => "high",
            RiskLevel::Denied => "denied",
        }
    }
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// =============================================================================
// Service Action Request
// =============================================================================

/// Systemd service action request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceAction {
    /// Service unit name (e.g., "nginx.service" or "nginx")
    pub service: String,
    /// Operation to perform
    pub operation: ServiceOperation,
    /// Reason for the action (user intent)
    pub reason: Option<String>,
}

impl ServiceAction {
    pub fn new(service: &str, operation: ServiceOperation) -> Self {
        Self {
            service: normalize_service_name(service),
            operation,
            reason: None,
        }
    }

    pub fn with_reason(mut self, reason: &str) -> Self {
        self.reason = Some(reason.to_string());
        self
    }
}

/// Normalize service name to include .service suffix
pub fn normalize_service_name(name: &str) -> String {
    if name.contains('.') {
        name.to_string()
    } else {
        format!("{}.service", name)
    }
}

// =============================================================================
// Risk Assessment
// =============================================================================

/// Assess risk level for a service action
pub fn assess_risk(action: &ServiceAction) -> RiskLevel {
    let service_lower = action.service.to_lowercase();
    let base_name = service_lower.trim_end_matches(".service");

    // Check for core systemd services (denied)
    for core in CORE_SYSTEMD_SERVICES {
        if base_name == core.to_lowercase() {
            return RiskLevel::Denied;
        }
    }

    // Check for critical services (high risk)
    for critical in CRITICAL_SERVICES {
        if base_name == critical.to_lowercase()
            || base_name.contains(critical.to_lowercase().as_str())
        {
            return RiskLevel::High;
        }
    }

    // Check for network services (medium risk for stop/disable)
    for network in NETWORK_SERVICES {
        if base_name == network.to_lowercase() {
            return match action.operation {
                ServiceOperation::Stop | ServiceOperation::Disable => RiskLevel::Medium,
                ServiceOperation::Restart => RiskLevel::Medium,
                ServiceOperation::Start | ServiceOperation::Enable => RiskLevel::Low,
            };
        }
    }

    // Default: low risk for most operations
    RiskLevel::Low
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_service_name() {
        assert_eq!(normalize_service_name("nginx"), "nginx.service");
        assert_eq!(normalize_service_name("nginx.service"), "nginx.service");
        assert_eq!(normalize_service_name("foo.socket"), "foo.socket");
    }

    #[test]
    fn test_risk_assessment_network() {
        let action = ServiceAction::new("NetworkManager", ServiceOperation::Stop);
        assert_eq!(assess_risk(&action), RiskLevel::Medium);

        let action = ServiceAction::new("NetworkManager", ServiceOperation::Start);
        assert_eq!(assess_risk(&action), RiskLevel::Low);
    }

    #[test]
    fn test_risk_assessment_critical() {
        let action = ServiceAction::new("sshd", ServiceOperation::Stop);
        assert_eq!(assess_risk(&action), RiskLevel::High);

        let action = ServiceAction::new("gdm", ServiceOperation::Restart);
        assert_eq!(assess_risk(&action), RiskLevel::High);
    }

    #[test]
    fn test_risk_assessment_core() {
        let action = ServiceAction::new("systemd-journald", ServiceOperation::Stop);
        assert_eq!(assess_risk(&action), RiskLevel::Denied);

        let action = ServiceAction::new("dbus", ServiceOperation::Restart);
        assert_eq!(assess_risk(&action), RiskLevel::Denied);
    }

    #[test]
    fn test_risk_assessment_normal() {
        let action = ServiceAction::new("nginx", ServiceOperation::Restart);
        assert_eq!(assess_risk(&action), RiskLevel::Low);

        let action = ServiceAction::new("docker", ServiceOperation::Stop);
        assert_eq!(assess_risk(&action), RiskLevel::Low);
    }

    #[test]
    fn test_confirmation_phrases() {
        assert_eq!(
            RiskLevel::Low.confirmation_phrase(),
            Some(LOW_RISK_CONFIRMATION)
        );
        assert_eq!(
            RiskLevel::Medium.confirmation_phrase(),
            Some(MEDIUM_RISK_CONFIRMATION)
        );
        assert_eq!(
            RiskLevel::High.confirmation_phrase(),
            Some(HIGH_RISK_CONFIRMATION)
        );
        assert_eq!(RiskLevel::Denied.confirmation_phrase(), None);
    }

    #[test]
    fn test_operation_inverse() {
        assert_eq!(ServiceOperation::Start.inverse(), ServiceOperation::Stop);
        assert_eq!(ServiceOperation::Stop.inverse(), ServiceOperation::Start);
        assert_eq!(
            ServiceOperation::Enable.inverse(),
            ServiceOperation::Disable
        );
        assert_eq!(
            ServiceOperation::Disable.inverse(),
            ServiceOperation::Enable
        );
    }
}
