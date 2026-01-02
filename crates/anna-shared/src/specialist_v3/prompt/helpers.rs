//! Helper functions for severity and risk assessment.

use super::super::{RiskLevel, Severity};

/// Map severity to risk assessment.
pub fn severity_for_finding(key: &str, value: &str) -> Severity {
    // Memory thresholds
    if key.contains("mem_available") || key.contains("mem_free") {
        if let Ok(mb) = value.parse::<u64>() {
            return if mb < 500 {
                Severity::Critical
            } else if mb < 2000 {
                Severity::Warning
            } else {
                Severity::Info
            };
        }
    }

    // Disk thresholds
    if key.contains("disk_free") || key.contains("disk_available") {
        if let Ok(gb) = value.parse::<f64>() {
            return if gb < 1.0 {
                Severity::Critical
            } else if gb < 5.0 {
                Severity::Warning
            } else {
                Severity::Info
            };
        }
    }

    // Default
    Severity::Info
}

/// Map action risk based on command patterns.
pub fn risk_for_command(command: &str) -> RiskLevel {
    let cmd_lower = command.to_lowercase();

    // High risk patterns
    if cmd_lower.contains("rm -rf")
        || cmd_lower.contains("dd if=")
        || cmd_lower.contains("mkfs")
        || cmd_lower.contains("> /dev/")
    {
        return RiskLevel::High;
    }

    // Medium risk patterns
    if cmd_lower.contains("sudo")
        || cmd_lower.contains("systemctl")
        || cmd_lower.contains("pacman -R")
        || cmd_lower.contains("kill")
    {
        return RiskLevel::Medium;
    }

    // Low risk (default)
    RiskLevel::Low
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_for_finding() {
        assert_eq!(
            severity_for_finding("mem_available_mb", "100"),
            Severity::Critical
        );
        assert_eq!(
            severity_for_finding("mem_available_mb", "1000"),
            Severity::Warning
        );
        assert_eq!(
            severity_for_finding("mem_available_mb", "8000"),
            Severity::Info
        );
    }

    #[test]
    fn test_risk_for_command() {
        assert_eq!(risk_for_command("rm -rf /tmp/*"), RiskLevel::High);
        assert_eq!(
            risk_for_command("sudo systemctl restart foo"),
            RiskLevel::Medium
        );
        assert_eq!(risk_for_command("ls -la"), RiskLevel::Low);
    }
}
