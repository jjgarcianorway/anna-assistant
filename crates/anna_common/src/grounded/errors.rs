//! Error Module v6.0 - Grounded in journalctl
//!
//! Source of truth: journalctl commands only
//! No invented data. No hallucinations.

use std::collections::HashMap;
use std::process::Command;

/// Error/warning severity levels (systemd priorities)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Priority {
    Emergency = 0,  // System unusable
    Alert = 1,      // Action must be taken immediately
    Critical = 2,   // Critical conditions
    Error = 3,      // Error conditions
    Warning = 4,    // Warning conditions
    Notice = 5,     // Normal but significant
    Info = 6,       // Informational
    Debug = 7,      // Debug messages
}

impl Priority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Priority::Emergency => "emergency",
            Priority::Alert => "alert",
            Priority::Critical => "critical",
            Priority::Error => "error",
            Priority::Warning => "warning",
            Priority::Notice => "notice",
            Priority::Info => "info",
            Priority::Debug => "debug",
        }
    }

    pub fn from_u8(n: u8) -> Self {
        match n {
            0 => Priority::Emergency,
            1 => Priority::Alert,
            2 => Priority::Critical,
            3 => Priority::Error,
            4 => Priority::Warning,
            5 => Priority::Notice,
            6 => Priority::Info,
            _ => Priority::Debug,
        }
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Priority::Emergency | Priority::Alert | Priority::Critical | Priority::Error)
    }
}

/// A log entry from journalctl
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: u64,
    pub unit: String,
    pub priority: Priority,
    pub message: String,
}

/// Error counts by time period
#[derive(Debug, Clone, Default)]
pub struct ErrorCounts {
    pub errors: usize,
    pub warnings: usize,
    pub critical: usize,
}

impl ErrorCounts {
    /// Get error counts for last N hours
    /// Source: journalctl --since "N hours ago" -p err/warning
    pub fn query_hours(hours: u64) -> Self {
        let since = format!("{} hours ago", hours);
        Self {
            errors: count_priority(&since, "err"),
            warnings: count_priority(&since, "warning"),
            critical: count_priority(&since, "crit"),
        }
    }

    /// Get error counts for last 24 hours
    pub fn query_24h() -> Self {
        Self::query_hours(24)
    }
}

/// Count log entries at a priority level
/// Source: journalctl --since "X" -p PRIORITY
fn count_priority(since: &str, priority: &str) -> usize {
    let output = Command::new("journalctl")
        .args(["--since", since, "-p", priority, "--no-pager", "-q"])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .filter(|line| !line.trim().is_empty())
                .count()
        }
        _ => 0,
    }
}

/// Unit error summary
#[derive(Debug, Clone)]
pub struct UnitErrorSummary {
    pub unit: String,
    pub error_count: usize,
    pub last_error_time: String,
    pub sample_message: String,
}

/// Get top error-producing units in last N hours
/// Source: journalctl --since "N hours ago" -p err
pub fn get_top_error_units(hours: u64, limit: usize) -> Vec<UnitErrorSummary> {
    let since = format!("{} hours ago", hours);

    let output = Command::new("journalctl")
        .args([
            "--since", &since,
            "-p", "err",
            "--no-pager",
            "-o", "short",
            "-q",
        ])
        .output();

    let entries = match output {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout).to_string()
        }
        _ => return Vec::new(),
    };

    // Count errors per unit
    let mut unit_counts: HashMap<String, (usize, String, String)> = HashMap::new();

    for line in entries.lines() {
        if line.trim().is_empty() {
            continue;
        }

        // Parse journalctl short format
        // "Dec 01 14:30:45 hostname unit[pid]: message"
        let parts: Vec<&str> = line.splitn(5, ' ').collect();
        if parts.len() < 5 {
            continue;
        }

        // Extract unit name (before [pid]:)
        let unit_part = parts[3];
        let unit = unit_part.split('[').next().unwrap_or(unit_part).to_string();

        let message = parts.get(4).unwrap_or(&"").to_string();
        let time_str = format!("{} {} {}", parts[0], parts[1], parts[2]);

        let entry = unit_counts.entry(unit).or_insert((0, String::new(), String::new()));
        entry.0 += 1;
        entry.1 = time_str; // Update to latest
        if entry.2.is_empty() || entry.2.len() < message.len() {
            entry.2 = message; // Keep most informative message
        }
    }

    // Sort by count descending
    let mut results: Vec<_> = unit_counts.into_iter()
        .map(|(unit, (count, time, msg))| UnitErrorSummary {
            unit,
            error_count: count,
            last_error_time: time,
            sample_message: if msg.len() > 80 {
                format!("{}...", &msg[..77])
            } else {
                msg
            },
        })
        .collect();

    results.sort_by(|a, b| b.error_count.cmp(&a.error_count));
    results.truncate(limit);
    results
}

/// Get recent errors for a specific unit
/// Source: journalctl -u <unit> -p err --since "N hours ago"
pub fn get_unit_errors(unit: &str, hours: u64, limit: usize) -> Vec<LogEntry> {
    let since = format!("{} hours ago", hours);
    let unit_pattern = if unit.ends_with(".service") {
        unit.to_string()
    } else {
        format!("{}.service", unit)
    };

    let output = Command::new("journalctl")
        .args([
            "-u", &unit_pattern,
            "-p", "err",
            "--since", &since,
            "--no-pager",
            "-o", "short",
            "-q",
            "-n", &limit.to_string(),
        ])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(|line| LogEntry {
                    timestamp: 0, // Would need JSON output for proper timestamp
                    unit: unit_pattern.clone(),
                    priority: Priority::Error,
                    message: line.to_string(),
                })
                .collect()
        }
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Emergency < Priority::Error);
        assert!(Priority::Error < Priority::Warning);
        assert!(Priority::Warning < Priority::Info);
    }

    #[test]
    fn test_error_counts() {
        // This should work on any systemd system
        let counts = ErrorCounts::query_24h();
        // Can't assert specific counts, just that it doesn't panic
        println!("24h errors: {}, warnings: {}", counts.errors, counts.warnings);
    }
}
