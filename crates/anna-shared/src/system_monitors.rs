//! System Monitoring (v0.0.469).
//!
//! Proactive system monitoring capabilities per VISION.md:
//! "Impressive system monitoring capabilities"
//!
//! Checks system metrics and triggers alarms when conditions are met.

use crate::user_alarms::{AlarmCondition, AlarmStore, NotifyChannel, UserAlarm};
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

/// Check disk usage
pub fn check_disk_usage(path: Option<&str>) -> MonitorResult {
    let path = path.unwrap_or("/");

    // Try to get actual disk usage
    let (total, available) = get_disk_space(path);
    let used_percent = if total > 0 {
        ((total - available) * 100 / total) as u64
    } else {
        0
    };

    MonitorResult {
        check_type: CheckType::DiskUsage,
        current_value: used_percent,
        threshold: None,
        should_alert: false,
        message: format!("Disk usage on {}: {}%", path, used_percent),
    }
}

/// Check memory usage
pub fn check_memory_usage() -> MonitorResult {
    let (total, available) = get_memory_info();
    let used_percent = if total > 0 {
        ((total - available) * 100 / total) as u64
    } else {
        0
    };

    MonitorResult {
        check_type: CheckType::MemoryUsage,
        current_value: used_percent,
        threshold: None,
        should_alert: false,
        message: format!("Memory usage: {}%", used_percent),
    }
}

/// Check for failed systemd services
pub fn check_failed_services() -> MonitorResult {
    let count = count_failed_services();

    MonitorResult {
        check_type: CheckType::FailedServices,
        current_value: count,
        threshold: None,
        should_alert: count > 0,
        message: if count == 0 {
            "All services running".to_string()
        } else {
            format!("{} service(s) in failed state", count)
        },
    }
}

/// Check system load average
pub fn check_load_average() -> MonitorResult {
    let load = get_load_average();
    let cpu_count = get_cpu_count() as u64;
    let load_percent = if cpu_count > 0 {
        (load * 100 / cpu_count) as u64
    } else {
        load as u64
    };

    MonitorResult {
        check_type: CheckType::LoadAverage,
        current_value: load_percent,
        threshold: None,
        should_alert: false,
        message: format!("Load average: {} ({}% of {} cores)", load, load_percent, cpu_count),
    }
}

/// Check swap usage
pub fn check_swap_usage() -> MonitorResult {
    let (total, used) = get_swap_info();
    let used_percent = if total > 0 {
        (used * 100 / total) as u64
    } else {
        0
    };

    MonitorResult {
        check_type: CheckType::SwapUsage,
        current_value: used_percent,
        threshold: None,
        should_alert: false,
        message: format!("Swap usage: {}%", used_percent),
    }
}

/// Evaluate an alarm condition and return result
pub fn evaluate_condition(condition: &AlarmCondition) -> MonitorResult {
    match condition {
        AlarmCondition::DiskAbove { threshold_percent, path } => {
            let mut result = check_disk_usage(path.as_deref());
            result.threshold = Some(*threshold_percent as u64);
            result.should_alert = result.current_value >= *threshold_percent as u64;
            if result.should_alert {
                result.message = format!(
                    "Disk usage ({}%) exceeds threshold ({}%)",
                    result.current_value, threshold_percent
                );
            }
            result
        }
        AlarmCondition::MemoryAbove { threshold_percent } => {
            let mut result = check_memory_usage();
            result.threshold = Some(*threshold_percent as u64);
            result.should_alert = result.current_value >= *threshold_percent as u64;
            if result.should_alert {
                result.message = format!(
                    "Memory usage ({}%) exceeds threshold ({}%)",
                    result.current_value, threshold_percent
                );
            }
            result
        }
        AlarmCondition::ServiceFailed { service } => {
            let is_failed = is_service_failed(service);
            MonitorResult {
                check_type: CheckType::FailedServices,
                current_value: if is_failed { 1 } else { 0 },
                threshold: Some(1),
                should_alert: is_failed,
                message: if is_failed {
                    format!("Service '{}' is in failed state", service)
                } else {
                    format!("Service '{}' is running", service)
                },
            }
        }
        AlarmCondition::AnyServiceFailed => check_failed_services(),
        AlarmCondition::ProbeMatches { probe, pattern } => {
            // This would run a probe and match output - simplified here
            MonitorResult {
                check_type: CheckType::FailedServices,
                current_value: 0,
                threshold: None,
                should_alert: false,
                message: format!("Probe '{}' with pattern '{}'", probe, pattern),
            }
        }
    }
}

/// Check all conditional alarms and return those that should trigger
pub fn check_conditional_alarms(store: &AlarmStore) -> Vec<(&UserAlarm, MonitorResult)> {
    let mut triggered = Vec::new();

    for alarm in store.list() {
        if !alarm.enabled {
            continue;
        }

        if let crate::user_alarms::AlarmSchedule::Conditional { condition } = &alarm.schedule {
            let result = evaluate_condition(condition);
            if result.should_alert {
                triggered.push((alarm, result));
            }
        }
    }

    triggered
}

/// Run all standard system checks
pub fn run_all_checks() -> Vec<MonitorResult> {
    vec![
        check_disk_usage(None),
        check_memory_usage(),
        check_failed_services(),
        check_load_average(),
        check_swap_usage(),
    ]
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

// Platform-specific implementations

fn get_disk_space(path: &str) -> (u64, u64) {
    // Read from /proc or use statvfs
    if let Ok(output) = std::process::Command::new("df")
        .args(["--output=size,avail", "-B1", path])
        .output()
    {
        if let Ok(text) = String::from_utf8(output.stdout) {
            let lines: Vec<&str> = text.lines().collect();
            if lines.len() >= 2 {
                let parts: Vec<&str> = lines[1].split_whitespace().collect();
                if parts.len() >= 2 {
                    let total = parts[0].parse().unwrap_or(0);
                    let avail = parts[1].parse().unwrap_or(0);
                    return (total, avail);
                }
            }
        }
    }
    (0, 0)
}

fn get_memory_info() -> (u64, u64) {
    // Read from /proc/meminfo
    if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
        let mut total: u64 = 0;
        let mut available: u64 = 0;

        for line in content.lines() {
            if line.starts_with("MemTotal:") {
                total = parse_meminfo_value(line);
            } else if line.starts_with("MemAvailable:") {
                available = parse_meminfo_value(line);
            }
        }
        return (total, available);
    }
    (0, 0)
}

fn parse_meminfo_value(line: &str) -> u64 {
    line.split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

fn get_swap_info() -> (u64, u64) {
    if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
        let mut total: u64 = 0;
        let mut free: u64 = 0;

        for line in content.lines() {
            if line.starts_with("SwapTotal:") {
                total = parse_meminfo_value(line);
            } else if line.starts_with("SwapFree:") {
                free = parse_meminfo_value(line);
            }
        }
        return (total, total - free);
    }
    (0, 0)
}

fn get_load_average() -> u64 {
    if let Ok(content) = std::fs::read_to_string("/proc/loadavg") {
        if let Some(first) = content.split_whitespace().next() {
            if let Ok(load) = first.parse::<f64>() {
                return (load * 100.0) as u64;
            }
        }
    }
    0
}

fn get_cpu_count() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}

fn count_failed_services() -> u64 {
    if let Ok(output) = std::process::Command::new("systemctl")
        .args(["--failed", "--no-legend", "--plain"])
        .output()
    {
        if let Ok(text) = String::from_utf8(output.stdout) {
            return text.lines().count() as u64;
        }
    }
    0
}

fn is_service_failed(service: &str) -> bool {
    if let Ok(output) = std::process::Command::new("systemctl")
        .args(["is-failed", "--quiet", service])
        .output()
    {
        return output.status.code() == Some(0);
    }
    false
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
    fn test_run_all_checks() {
        let results = run_all_checks();
        assert_eq!(results.len(), 5);
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

    #[test]
    fn test_evaluate_disk_condition() {
        let condition = AlarmCondition::DiskAbove {
            threshold_percent: 90,
            path: None,
        };
        let result = evaluate_condition(&condition);
        assert_eq!(result.check_type, CheckType::DiskUsage);
        assert_eq!(result.threshold, Some(90));
    }

    #[test]
    fn test_evaluate_memory_condition() {
        let condition = AlarmCondition::MemoryAbove {
            threshold_percent: 80,
        };
        let result = evaluate_condition(&condition);
        assert_eq!(result.check_type, CheckType::MemoryUsage);
        assert_eq!(result.threshold, Some(80));
    }
}
