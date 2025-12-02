//! Boot Timeline v7.18.0 - Per-boot health view
//!
//! Constructs a compact per-boot timeline from journalctl and systemd data:
//! - Boot id and timestamp
//! - Kernel version and cmdline
//! - Key phases (early userspace, network up, graphical ready)
//! - Per-phase warnings/errors by service
//! - Failed or slow units
//!
//! Sources:
//! - journalctl --list-boots
//! - journalctl -b (per-boot logs)
//! - systemctl --failed

use std::collections::HashMap;
use std::process::Command;

use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use serde::{Deserialize, Serialize};

/// Directory for boot timeline storage
pub const BOOT_TIMELINE_DIR: &str = "/var/lib/anna/telemetry/boot";

/// Boot phases we track
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BootPhase {
    EarlyUserspace,
    NetworkUp,
    GraphicalReady,
}

impl BootPhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            BootPhase::EarlyUserspace => "early userspace",
            BootPhase::NetworkUp => "network up",
            BootPhase::GraphicalReady => "graphical ready",
        }
    }
}

/// Summary of a single boot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootSummary {
    /// Boot ID from systemd
    pub boot_id: String,
    /// Boot timestamp (Unix)
    pub timestamp: u64,
    /// Kernel version
    pub kernel: String,
    /// Duration to graphical.target in seconds (if available)
    pub duration_to_graphical: Option<f64>,
    /// Failed units count
    pub failed_units: u32,
    /// Services with warnings
    pub services_with_warnings: u32,
    /// Slow units (took > 5s to start)
    pub slow_units: Vec<SlowUnit>,
    /// Error counts by service
    pub error_counts: HashMap<String, u32>,
    /// Warning counts by service
    pub warning_counts: HashMap<String, u32>,
}

/// A unit that took unusually long to start
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlowUnit {
    pub name: String,
    pub duration_secs: f64,
}

impl BootSummary {
    /// Format timestamp for display
    pub fn format_time(&self) -> String {
        if let Some(dt) = DateTime::from_timestamp(self.timestamp as i64, 0) {
            let local: DateTime<Local> = dt.into();
            local.format("%Y-%m-%d %H:%M").to_string()
        } else {
            "unknown".to_string()
        }
    }

    /// Format duration for display
    pub fn format_duration(&self) -> String {
        if let Some(d) = self.duration_to_graphical {
            format!("{:.0}s", d)
        } else {
            "unknown".to_string()
        }
    }
}

/// Get summary of current boot
pub fn get_current_boot_summary() -> Option<BootSummary> {
    get_boot_summary("0")
}

/// Get summary of previous boot
pub fn get_previous_boot_summary() -> Option<BootSummary> {
    get_boot_summary("-1")
}

/// Get summary of a specific boot by index (0 = current, -1 = previous, etc.)
pub fn get_boot_summary(boot_idx: &str) -> Option<BootSummary> {
    // Get boot ID and timestamp
    let (boot_id, timestamp) = get_boot_info(boot_idx)?;

    // Get kernel version
    let kernel = get_kernel_version(boot_idx).unwrap_or_else(|| "unknown".to_string());

    // Get boot duration
    let duration = get_boot_duration(boot_idx);

    // Get failed units count
    let failed_units = get_failed_units_count(boot_idx);

    // Get error/warning counts by service
    let (error_counts, warning_counts) = get_log_counts_by_service(boot_idx);

    // Get slow units
    let slow_units = get_slow_units(boot_idx);

    let services_with_warnings = warning_counts.len() as u32;

    Some(BootSummary {
        boot_id,
        timestamp,
        kernel,
        duration_to_graphical: duration,
        failed_units,
        services_with_warnings,
        slow_units,
        error_counts,
        warning_counts,
    })
}

/// Get boot ID and timestamp for a boot index
fn get_boot_info(boot_idx: &str) -> Option<(String, u64)> {
    let output = Command::new("journalctl")
        .args(["--list-boots", "--no-pager"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        // Skip header line
        if line.starts_with("IDX") || line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        // Format: "IDX BOOT_ID FIRST_ENTRY..."
        // e.g.: "  0 bf2f68dc27c14897bc8a4ab1a0079ab7 Sun 2025-11-30 20:50:28 CET ..."
        // parts[0] = boot index (0, -1, etc.)
        // parts[1] = boot ID
        // parts[2..] = timestamp parts (weekday date time timezone)

        if parts[0] == boot_idx {
            let boot_id = parts.get(1)?.to_string();

            // Parse timestamp from remaining parts (skip boot id)
            // parts[2] = weekday (Sun), parts[3] = date (2025-11-30), parts[4] = time (20:50:28)
            let timestamp = parse_boot_line_timestamp(&parts[2..]);

            return Some((boot_id, timestamp));
        }
    }

    None
}

/// Parse timestamp from journalctl --list-boots line
fn parse_boot_line_timestamp(parts: &[&str]) -> u64 {
    // Format: "Sun 2025-11-30 20:50:28 CET" or similar
    // parts might be ["Sun", "2025-11-30", "20:50:28", "CET"]

    if parts.len() >= 3 {
        // Try with weekday: "Sun 2025-11-30 20:50:28"
        let date_time = format!("{} {}", parts[1], parts[2]);
        if let Ok(dt) = NaiveDateTime::parse_from_str(&date_time, "%Y-%m-%d %H:%M:%S") {
            if let Some(local) = Local.from_local_datetime(&dt).single() {
                return local.timestamp() as u64;
            }
        }

        // Try without weekday: "2025-11-30 20:50:28"
        let date_time = format!("{} {}", parts[0], parts[1]);
        if let Ok(dt) = NaiveDateTime::parse_from_str(&date_time, "%Y-%m-%d %H:%M:%S") {
            if let Some(local) = Local.from_local_datetime(&dt).single() {
                return local.timestamp() as u64;
            }
        }
    }

    if parts.len() >= 2 {
        let date_time = format!("{} {}", parts[0], parts[1]);
        if let Ok(dt) = NaiveDateTime::parse_from_str(&date_time, "%Y-%m-%d %H:%M:%S") {
            if let Some(local) = Local.from_local_datetime(&dt).single() {
                return local.timestamp() as u64;
            }
        }
    }

    0
}

/// Get kernel version for a boot
fn get_kernel_version(boot_idx: &str) -> Option<String> {
    // For current boot, use uname -r (most reliable)
    if boot_idx == "0" {
        let output = Command::new("uname")
            .args(["-r"])
            .output()
            .ok()?;

        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            return Some(version.trim().to_string());
        }
    }

    // For older boots, search journal for kernel version
    // Look for messages from kernel with version info
    let output = Command::new("journalctl")
        .args(["-b", boot_idx, "-k", "--no-pager", "--grep", "Linux version"])
        .output()
        .ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains("Linux version") {
                // Extract: "Linux version 6.17.9-arch1-1 (..."
                if let Some(idx) = line.find("Linux version") {
                    let rest = &line[idx + 14..];
                    if let Some(version) = rest.split_whitespace().next() {
                        return Some(version.to_string());
                    }
                }
            }
        }
    }

    None
}

/// Get boot duration to graphical.target (or multi-user.target)
fn get_boot_duration(boot_idx: &str) -> Option<f64> {
    // Try systemd-analyze for current boot
    if boot_idx == "0" {
        let output = Command::new("systemd-analyze")
            .output()
            .ok()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Format: "Startup finished in X.XXXs (firmware) + Y.YYYs (loader) + Z.ZZZs (kernel) + W.WWWs (userspace) = T.TTTs"
            // Or: "graphical.target reached after X.XXXs in userspace"

            // Look for total time
            if let Some(idx) = stdout.find(" = ") {
                let rest = &stdout[idx + 3..];
                if let Some(time_str) = rest.split('s').next() {
                    if let Ok(t) = time_str.trim().parse::<f64>() {
                        return Some(t);
                    }
                }
            }

            // Look for "reached after"
            if let Some(idx) = stdout.find("reached after ") {
                let rest = &stdout[idx + 14..];
                if let Some(time_str) = rest.split('s').next() {
                    if let Ok(t) = time_str.trim().parse::<f64>() {
                        return Some(t);
                    }
                }
            }
        }
    }

    // For non-current boots, estimate from journal timestamps
    // Look for graphical.target or multi-user.target reached
    let output = Command::new("journalctl")
        .args(["-b", boot_idx, "--no-pager", "-o", "short-monotonic"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut start_time: Option<f64> = None;
    let mut end_time: Option<f64> = None;

    for line in stdout.lines() {
        // Parse monotonic timestamp
        if let Some(ts) = parse_monotonic_timestamp(line) {
            if start_time.is_none() {
                start_time = Some(ts);
            }

            // Look for target reached messages
            if line.contains("graphical.target") && line.contains("Reached") {
                end_time = Some(ts);
                break;
            }
            if line.contains("multi-user.target") && line.contains("Reached") {
                end_time = Some(ts);
                // Don't break, continue looking for graphical
            }
        }
    }

    match (start_time, end_time) {
        (Some(s), Some(e)) => Some(e - s),
        _ => None,
    }
}

/// Parse monotonic timestamp from journalctl -o short-monotonic
fn parse_monotonic_timestamp(line: &str) -> Option<f64> {
    // Format: "[    1.234567] hostname kernel: message"
    if let Some(start) = line.find('[') {
        if let Some(end) = line.find(']') {
            let ts_str = line[start + 1..end].trim();
            if let Ok(ts) = ts_str.parse::<f64>() {
                return Some(ts);
            }
        }
    }
    None
}

/// Get count of failed units for a boot
fn get_failed_units_count(boot_idx: &str) -> u32 {
    if boot_idx == "0" {
        // For current boot, use systemctl --failed
        let output = Command::new("systemctl")
            .args(["--failed", "--no-legend", "--no-pager"])
            .output();

        if let Ok(out) = output {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                return stdout.lines().count() as u32;
            }
        }
    }

    // For other boots, scan journal for "Failed to start" or "failed"
    let output = Command::new("journalctl")
        .args(["-b", boot_idx, "-p", "err", "--no-pager"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let failed_count = stdout.lines()
                .filter(|l| l.contains("Failed to start") || l.contains("failed."))
                .count();
            return failed_count as u32;
        }
    }

    0
}

/// Get error and warning counts by service for a boot
fn get_log_counts_by_service(boot_idx: &str) -> (HashMap<String, u32>, HashMap<String, u32>) {
    let mut error_counts: HashMap<String, u32> = HashMap::new();
    let mut warning_counts: HashMap<String, u32> = HashMap::new();

    // Get errors
    let output = Command::new("journalctl")
        .args(["-b", boot_idx, "-p", "err", "--no-pager", "-o", "json"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                if let Some(unit) = extract_unit_from_json(line) {
                    *error_counts.entry(unit).or_insert(0) += 1;
                }
            }
        }
    }

    // Get warnings
    let output = Command::new("journalctl")
        .args(["-b", boot_idx, "-p", "warning", "--no-pager", "-o", "json"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                if let Some(unit) = extract_unit_from_json(line) {
                    // Don't double-count errors as warnings
                    if !error_counts.contains_key(&unit) {
                        *warning_counts.entry(unit).or_insert(0) += 1;
                    }
                }
            }
        }
    }

    (error_counts, warning_counts)
}

/// Extract unit name from journalctl JSON output
fn extract_unit_from_json(line: &str) -> Option<String> {
    // Parse JSON and extract _SYSTEMD_UNIT or SYSLOG_IDENTIFIER
    let json: serde_json::Value = serde_json::from_str(line).ok()?;

    // Try _SYSTEMD_UNIT first
    if let Some(unit) = json.get("_SYSTEMD_UNIT") {
        if let Some(s) = unit.as_str() {
            return Some(s.to_string());
        }
    }

    // Fall back to SYSLOG_IDENTIFIER
    if let Some(ident) = json.get("SYSLOG_IDENTIFIER") {
        if let Some(s) = ident.as_str() {
            return Some(s.to_string());
        }
    }

    None
}

/// Get slow units (took > threshold seconds to start)
fn get_slow_units(boot_idx: &str) -> Vec<SlowUnit> {
    const SLOW_THRESHOLD: f64 = 5.0;
    let mut slow = Vec::new();

    if boot_idx != "0" {
        // Can only get blame for current boot
        return slow;
    }

    let output = Command::new("systemd-analyze")
        .args(["blame", "--no-pager"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);

            for line in stdout.lines().take(10) {
                // Format: "5.432s NetworkManager.service"
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let time_str = parts[0].trim_end_matches('s');
                    if let Ok(duration) = time_str.parse::<f64>() {
                        if duration >= SLOW_THRESHOLD {
                            slow.push(SlowUnit {
                                name: parts[1].to_string(),
                                duration_secs: duration,
                            });
                        }
                    }
                }
            }
        }
    }

    slow
}

/// Get list of boots (most recent first)
pub fn get_boot_list(count: usize) -> Vec<BootSummary> {
    let mut boots = Vec::new();

    // Get list of boots
    let output = Command::new("journalctl")
        .args(["--list-boots", "--no-pager"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let lines: Vec<&str> = stdout.lines().collect();

            // Take last N boots (most recent)
            for line in lines.iter().rev().take(count) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if !parts.is_empty() {
                    let boot_idx = parts[0];
                    if let Some(summary) = get_boot_summary(boot_idx) {
                        boots.push(summary);
                    }
                }
            }
        }
    }

    boots
}

/// Get log patterns for a specific service across boots
pub fn get_service_log_patterns_by_boot(
    service: &str,
    boot_count: usize,
) -> Vec<(String, Vec<LogPatternEntry>)> {
    let mut result = Vec::new();

    for i in 0..boot_count {
        let boot_idx = if i == 0 {
            "0".to_string()
        } else {
            format!("-{}", i)
        };

        let boot_label = if i == 0 {
            "Boot 0 (current)".to_string()
        } else {
            format!("Boot -{}", i)
        };

        let patterns = get_patterns_for_boot_service(&boot_idx, service);
        if !patterns.is_empty() {
            result.push((boot_label, patterns));
        }
    }

    result
}

/// Log pattern entry with count
#[derive(Debug, Clone)]
pub struct LogPatternEntry {
    pub pattern: String,
    pub priority: String,
    pub count: u32,
}

/// Get log patterns for a service in a specific boot
fn get_patterns_for_boot_service(boot_idx: &str, service: &str) -> Vec<LogPatternEntry> {
    let mut pattern_counts: HashMap<String, (String, u32)> = HashMap::new();

    let unit_name = if service.ends_with(".service") {
        service.to_string()
    } else {
        format!("{}.service", service)
    };

    let output = Command::new("journalctl")
        .args(["-b", boot_idx, "-u", &unit_name, "-p", "warning..alert", "--no-pager"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);

            for line in stdout.lines() {
                // Normalize the message
                let (priority, normalized) = normalize_log_message(line);
                if !normalized.is_empty() {
                    let entry = pattern_counts.entry(normalized).or_insert((priority, 0));
                    entry.1 += 1;
                }
            }
        }
    }

    pattern_counts
        .into_iter()
        .map(|(pattern, (priority, count))| LogPatternEntry {
            pattern,
            priority,
            count,
        })
        .collect()
}

/// Normalize a log message by stripping volatile parts
fn normalize_log_message(line: &str) -> (String, String) {
    // Extract priority from line if present
    let priority = if line.contains("error") || line.contains("Error") || line.contains("ERROR") {
        "error".to_string()
    } else if line.contains("warning") || line.contains("Warning") || line.contains("WARN") {
        "warning".to_string()
    } else {
        "info".to_string()
    };

    // Skip timestamp prefix (usually first 2-3 fields)
    let parts: Vec<&str> = line.split_whitespace().collect();
    let message_start = parts.iter()
        .position(|p| !p.contains(':') || p.len() > 20)
        .unwrap_or(3)
        .min(4);

    let message = parts[message_start..].join(" ");

    // Normalize: replace IPs, ports, PIDs, timestamps with placeholders
    let normalized = normalize_volatile_parts(&message);

    (priority, normalized)
}

/// Replace volatile parts with placeholders
fn normalize_volatile_parts(message: &str) -> String {
    let mut result = message.to_string();

    // Replace IP addresses with %IP%
    let ip_pattern = regex::Regex::new(r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}").unwrap();
    result = ip_pattern.replace_all(&result, "%IP%").to_string();

    // Replace port numbers (after : or "port")
    let port_pattern = regex::Regex::new(r":\d{2,5}\b").unwrap();
    result = port_pattern.replace_all(&result, ":%PORT%").to_string();

    // Replace PIDs
    let pid_pattern = regex::Regex::new(r"\bpid[=: ]\d+").unwrap();
    result = pid_pattern.replace_all(&result, "pid=%PID%").to_string();

    // Replace hex addresses
    let hex_pattern = regex::Regex::new(r"0x[0-9a-fA-F]+").unwrap();
    result = hex_pattern.replace_all(&result, "%ADDR%").to_string();

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_volatile_parts() {
        let msg = "Connection from 192.168.1.1:8080 failed";
        let normalized = normalize_volatile_parts(msg);
        assert!(normalized.contains("%IP%"));
        assert!(normalized.contains("%PORT%"));
    }

    #[test]
    fn test_get_current_boot_summary() {
        // Should not crash even if journalctl isn't available
        let _summary = get_current_boot_summary();
    }
}
