//! Risky Operations - Templates for critical system operations.
//!
//! v0.3.126: Enable Anna to handle boot configuration, GRUB, kernel parameters,
//! and other operations that require elevated approval.

use serde::{Deserialize, Serialize};

/// Risk level for operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Low risk - easily reversible, safe
    Low,
    /// Medium risk - may require reboot, but recoverable
    Medium,
    /// High risk - boot issues possible, needs backup
    High,
    /// Critical risk - potential data loss or system unbootable
    Critical,
}

/// A template for a risky operation.
#[derive(Debug, Clone)]
pub struct RiskyOpTemplate {
    pub name: String,
    pub description: String,
    pub risk_level: RiskLevel,
    pub requires_backup: bool,
    pub requires_reboot: bool,
}

/// Get risk level for an operation type.
pub fn get_risk_level(operation: &str) -> RiskLevel {
    let op_lower = operation.to_lowercase();
    if op_lower.contains("grub") || op_lower.contains("boot") || op_lower.contains("fstab") {
        RiskLevel::High
    } else if op_lower.contains("kernel") || op_lower.contains("mask") {
        RiskLevel::Medium
    } else if op_lower.contains("network") {
        RiskLevel::Low
    } else {
        RiskLevel::Medium
    }
}

/// Format risk warning for user approval.
pub fn format_risk_warning(risk: RiskLevel) -> String {
    match risk {
        RiskLevel::Low => "Risk: LOW - Easily reversible, safe to proceed.".to_string(),
        RiskLevel::Medium => "Risk: MEDIUM - May require reboot or service restart. Backup created.".to_string(),
        RiskLevel::High => "Risk: HIGH - Boot issues possible. Full backup recommended before proceeding.".to_string(),
        RiskLevel::Critical => "Risk: CRITICAL - Potential data loss or system unbootable. Manual recovery may be needed.".to_string(),
    }
}

/// Check if an operation is risky and needs special approval.
pub fn is_risky_operation(question: &str) -> bool {
    let q = question.to_lowercase();
    q.contains("grub") ||
    q.contains("boot") ||
    q.contains("fstab") ||
    q.contains("kernel") ||
    q.contains("initramfs") ||
    (q.contains("edit") && (q.contains("/etc/") || q.contains("config")))
}

/// Get confirmation message for a risky operation.
pub fn get_confirmation_message(operation: &str, risk: RiskLevel) -> String {
    format!(
        "{}\n\nOperation: {}\n\nThis is a critical system operation. Are you sure you want to proceed?",
        format_risk_warning(risk),
        operation
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_levels() {
        assert_eq!(get_risk_level("grub_config"), RiskLevel::High);
        assert_eq!(get_risk_level("network_up"), RiskLevel::Low);
        assert_eq!(get_risk_level("kernel_param"), RiskLevel::Medium);
    }

    #[test]
    fn test_risky_detection() {
        assert!(is_risky_operation("modify grub config"));
        assert!(is_risky_operation("edit /etc/fstab"));
        assert!(!is_risky_operation("show disk space"));
    }
}
