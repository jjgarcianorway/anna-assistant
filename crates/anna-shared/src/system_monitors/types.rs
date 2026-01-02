//! Core types for system monitoring.

use serde::{Deserialize, Serialize};

/// Result of a monitoring check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorResult {
    /// What was checked
    pub check_type: CheckType,
    /// Current value (percentage, count, etc.)
    pub current_value: u64,
    /// Threshold that triggered (if any)
    pub threshold: Option<u64>,
    /// Whether alert should be triggered
    pub should_alert: bool,
    /// Human-readable message
    pub message: String,
}

/// Types of monitoring checks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckType {
    /// Disk usage percentage
    DiskUsage,
    /// Memory usage percentage
    MemoryUsage,
    /// Failed services count
    FailedServices,
    /// System load
    LoadAverage,
    /// Swap usage
    SwapUsage,
}

impl CheckType {
    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            CheckType::DiskUsage => "Disk Usage",
            CheckType::MemoryUsage => "Memory Usage",
            CheckType::FailedServices => "Failed Services",
            CheckType::LoadAverage => "Load Average",
            CheckType::SwapUsage => "Swap Usage",
        }
    }
}

/// Format monitor results for display
pub fn format_monitor_results(results: &[MonitorResult]) -> String {
    let mut output = String::new();
    output.push_str("System Monitor Status\n");
    output.push_str("---------------------\n");

    for result in results {
        let status = if result.should_alert { "[!]" } else { "[ok]" };
        output.push_str(&format!("{} {}\n", status, result.message));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_type_name() {
        assert_eq!(CheckType::DiskUsage.name(), "Disk Usage");
        assert_eq!(CheckType::MemoryUsage.name(), "Memory Usage");
    }

    #[test]
    fn test_format_results() {
        let results = vec![
            MonitorResult {
                check_type: CheckType::DiskUsage,
                current_value: 50,
                threshold: None,
                should_alert: false,
                message: "Disk usage: 50%".into(),
            },
            MonitorResult {
                check_type: CheckType::MemoryUsage,
                current_value: 95,
                threshold: Some(90),
                should_alert: true,
                message: "Memory high!".into(),
            },
        ];

        let formatted = format_monitor_results(&results);
        assert!(formatted.contains("[ok]"));
        assert!(formatted.contains("[!]"));
    }
}
