//! Alert Detectors v0.0.58 - High-Signal Issue Detection
//!
//! Implements the 5 alert types for v0.0.58:
//! 1. BOOT_REGRESSION - boot time vs rolling baseline
//! 2. DISK_PRESSURE - / free < 10% or < 15 GiB
//! 3. JOURNAL_ERROR_BURST - unit >= 20 errors in 10 min
//! 4. SERVICE_FAILED - any systemd unit in failed state
//! 5. THERMAL_THROTTLING - CPU near throttle (best-effort)
//!
//! Each detector returns structured evidence that can be stored.

use serde::{Deserialize, Serialize};
use std::process::Command;

use crate::proactive_alerts::{AlertSeverity, AlertType, ProactiveAlert};

// ============================================================================
// Evidence Structures
// ============================================================================

/// Evidence for boot regression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootRegressionEvidence {
    pub last_boot_secs: f64,
    pub baseline_mean_secs: f64,
    pub baseline_stddev: f64,
    pub boot_count: usize,
    pub boot_times: Vec<f64>,
}

/// Evidence for disk pressure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskPressureEvidence {
    pub mount_point: String,
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub used_percent: f64,
    pub free_gib: f64,
}

/// Evidence for journal error burst
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalErrorBurstEvidence {
    pub unit: String,
    pub error_count: u32,
    pub window_minutes: u32,
    pub sample_messages: Vec<String>,
}

/// Evidence for service failed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceFailedEvidence {
    pub unit: String,
    pub load_state: String,
    pub active_state: String,
    pub sub_state: String,
    pub result: Option<String>,
}

/// Evidence for thermal throttling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalThrottlingEvidence {
    pub sensor_name: String,
    pub current_temp_c: f64,
    pub threshold_temp_c: Option<f64>,
    pub is_throttling: bool,
}

// ============================================================================
// Detection Thresholds
// ============================================================================

/// Boot regression thresholds
pub const BOOT_REGRESSION_MIN_SAMPLES: usize = 3;
pub const BOOT_REGRESSION_STDDEV_FACTOR: f64 = 2.0;
pub const BOOT_REGRESSION_MIN_DELTA_SECS: f64 = 5.0;

/// Disk pressure thresholds
pub const DISK_PRESSURE_WARNING_PERCENT: f64 = 10.0;
pub const DISK_PRESSURE_WARNING_GIB: f64 = 15.0;
pub const DISK_PRESSURE_CRITICAL_PERCENT: f64 = 5.0;
pub const DISK_PRESSURE_CRITICAL_GIB: f64 = 5.0;

/// Journal error burst thresholds
pub const JOURNAL_ERROR_BURST_WINDOW_MINS: u32 = 10;
pub const JOURNAL_ERROR_BURST_WARNING: u32 = 20;
pub const JOURNAL_ERROR_BURST_CRITICAL: u32 = 50;

/// Thermal thresholds (Celsius)
pub const THERMAL_WARNING_TEMP: f64 = 85.0;
pub const THERMAL_CRITICAL_TEMP: f64 = 95.0;

// ============================================================================
// Boot Regression Detection
// ============================================================================

/// Detect boot regression by comparing to baseline
pub fn detect_boot_regression(evidence_id: &str) -> Option<(ProactiveAlert, BootRegressionEvidence)> {
    // Get boot times using systemd-analyze
    let boot_times = get_boot_times(10)?;

    if boot_times.len() < BOOT_REGRESSION_MIN_SAMPLES {
        return None;
    }

    let last_boot = boot_times[0];
    let baseline = &boot_times[1..];

    // Calculate baseline statistics
    let mean: f64 = baseline.iter().sum::<f64>() / baseline.len() as f64;
    let variance: f64 = baseline.iter()
        .map(|t| (t - mean).powi(2))
        .sum::<f64>() / baseline.len() as f64;
    let stddev = variance.sqrt();

    // Check trigger: last > mean + 2*stddev AND > mean + 5s
    let threshold = mean + BOOT_REGRESSION_STDDEV_FACTOR * stddev;
    let min_threshold = mean + BOOT_REGRESSION_MIN_DELTA_SECS;

    if last_boot > threshold && last_boot > min_threshold {
        let evidence = BootRegressionEvidence {
            last_boot_secs: last_boot,
            baseline_mean_secs: mean,
            baseline_stddev: stddev,
            boot_count: boot_times.len(),
            boot_times: boot_times.clone(),
        };

        let delta = last_boot - mean;
        let summary = format!(
            "Boot took {:.1}s, {:.1}s slower than baseline ({:.1}s avg)",
            last_boot, delta, mean
        );

        let alert = ProactiveAlert::new(
            AlertType::BootRegression,
            AlertSeverity::Warning,
            "boot",
            "Boot time regression",
            &summary,
        )
        .with_evidence(evidence_id)
        .with_data(serde_json::to_value(&evidence).unwrap_or_default());

        return Some((alert, evidence));
    }

    None
}

/// Get recent boot times from systemd-analyze
fn get_boot_times(count: usize) -> Option<Vec<f64>> {
    // Try systemd-analyze time first for current boot
    let output = Command::new("systemd-analyze")
        .arg("time")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse: "Startup finished in X.XXXs (kernel) + Y.YYYs (userspace) = Z.ZZZs"
    let total = parse_systemd_analyze_time(&stdout)?;

    // For now, return just current boot. Full implementation would read from
    // /var/lib/anna/internal/boot_times.json maintained by daemon
    Some(vec![total])
}

/// Parse systemd-analyze time output
fn parse_systemd_analyze_time(output: &str) -> Option<f64> {
    // Look for pattern: "= X.XXXs" or "= Xmin Y.YYYs"
    let lines: Vec<&str> = output.lines().collect();
    for line in lines {
        if line.contains("Startup finished") {
            // Find the total time after "="
            if let Some(pos) = line.rfind('=') {
                let time_str = line[pos+1..].trim();
                return parse_time_duration(time_str);
            }
        }
    }
    None
}

/// Parse time duration string like "45.123s" or "1min 23.456s"
fn parse_time_duration(s: &str) -> Option<f64> {
    let s = s.trim();
    let mut total = 0.0;

    // Handle "Xmin Y.YYYs" format
    if s.contains("min") {
        let parts: Vec<&str> = s.split("min").collect();
        if let Ok(mins) = parts[0].trim().parse::<f64>() {
            total += mins * 60.0;
        }
        if parts.len() > 1 {
            if let Some(secs) = parse_seconds(parts[1].trim()) {
                total += secs;
            }
        }
        return Some(total);
    }

    // Handle "X.XXXs" format
    parse_seconds(s)
}

fn parse_seconds(s: &str) -> Option<f64> {
    let s = s.trim_end_matches('s').trim();
    s.parse::<f64>().ok()
}

// ============================================================================
// Disk Pressure Detection
// ============================================================================

/// Detect disk pressure on root filesystem
pub fn detect_disk_pressure(evidence_id: &str) -> Option<(ProactiveAlert, DiskPressureEvidence)> {
    let evidence = get_disk_usage("/")?;

    let free_percent = 100.0 - evidence.used_percent;
    let free_gib = evidence.free_gib;

    // Determine severity
    let severity = if free_percent < DISK_PRESSURE_CRITICAL_PERCENT || free_gib < DISK_PRESSURE_CRITICAL_GIB {
        AlertSeverity::Critical
    } else if free_percent < DISK_PRESSURE_WARNING_PERCENT || free_gib < DISK_PRESSURE_WARNING_GIB {
        AlertSeverity::Warning
    } else {
        return None; // No pressure
    };

    let summary = format!(
        "Root filesystem {} free ({:.1} GiB), {:.1}% used",
        format_percent(free_percent), free_gib, evidence.used_percent
    );

    let alert = ProactiveAlert::new(
        AlertType::DiskPressure,
        severity,
        &evidence.mount_point,
        "Low disk space",
        &summary,
    )
    .with_evidence(evidence_id)
    .with_data(serde_json::to_value(&evidence).unwrap_or_default());

    Some((alert, evidence))
}

fn format_percent(p: f64) -> String {
    if p < 1.0 {
        format!("{:.1}%", p)
    } else {
        format!("{:.0}%", p)
    }
}

/// Get disk usage for a mount point
fn get_disk_usage(mount_point: &str) -> Option<DiskPressureEvidence> {
    // Use df -B1 for bytes
    let output = Command::new("df")
        .args(["-B1", mount_point])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();

    // Skip header, parse data line
    if lines.len() < 2 {
        return None;
    }

    let parts: Vec<&str> = lines[1].split_whitespace().collect();
    if parts.len() < 6 {
        return None;
    }

    // Format: Filesystem Size Used Avail Use% Mounted
    let total_bytes: u64 = parts[1].parse().ok()?;
    let used_bytes: u64 = parts[2].parse().ok()?;
    let free_bytes: u64 = parts[3].parse().ok()?;

    let used_percent = if total_bytes > 0 {
        (used_bytes as f64 / total_bytes as f64) * 100.0
    } else {
        0.0
    };

    let free_gib = free_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

    Some(DiskPressureEvidence {
        mount_point: mount_point.to_string(),
        total_bytes,
        free_bytes,
        used_percent,
        free_gib,
    })
}

// ============================================================================
// Journal Error Burst Detection
// ============================================================================

/// Detect burst of journal errors from any unit
pub fn detect_journal_error_burst(evidence_id: &str) -> Vec<(ProactiveAlert, JournalErrorBurstEvidence)> {
    let mut alerts = Vec::new();

    // Get error counts by unit in last N minutes
    let errors = get_journal_error_counts(JOURNAL_ERROR_BURST_WINDOW_MINS);

    for (unit, count, samples) in errors {
        let severity = if count >= JOURNAL_ERROR_BURST_CRITICAL {
            AlertSeverity::Critical
        } else if count >= JOURNAL_ERROR_BURST_WARNING {
            AlertSeverity::Warning
        } else {
            continue; // Below threshold
        };

        let evidence = JournalErrorBurstEvidence {
            unit: unit.clone(),
            error_count: count,
            window_minutes: JOURNAL_ERROR_BURST_WINDOW_MINS,
            sample_messages: samples,
        };

        let summary = format!(
            "{} errors from {} in last {} minutes",
            count, unit, JOURNAL_ERROR_BURST_WINDOW_MINS
        );

        let alert = ProactiveAlert::new(
            AlertType::JournalErrorBurst,
            severity,
            &unit,
            "Journal error burst",
            &summary,
        )
        .with_evidence(evidence_id)
        .with_data(serde_json::to_value(&evidence).unwrap_or_default());

        alerts.push((alert, evidence));
    }

    alerts
}

/// Get error counts by unit from journal
fn get_journal_error_counts(minutes: u32) -> Vec<(String, u32, Vec<String>)> {
    let output = Command::new("journalctl")
        .args([
            "--priority=err",
            &format!("--since={} minutes ago", minutes),
            "--output=short",
            "--no-pager",
        ])
        .output();

    let output = match output {
        Ok(o) if o.status.success() => o,
        _ => return Vec::new(),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut unit_errors: std::collections::HashMap<String, (u32, Vec<String>)> = std::collections::HashMap::new();

    for line in stdout.lines() {
        // Parse unit from journal line
        if let Some(unit) = extract_unit_from_journal_line(line) {
            let entry = unit_errors.entry(unit).or_insert((0, Vec::new()));
            entry.0 += 1;
            if entry.1.len() < 3 {
                // Keep up to 3 sample messages
                entry.1.push(truncate_message(line, 100));
            }
        }
    }

    unit_errors.into_iter()
        .map(|(unit, (count, samples))| (unit, count, samples))
        .collect()
}

fn extract_unit_from_journal_line(line: &str) -> Option<String> {
    // Journal format: "Mon DD HH:MM:SS hostname unit[pid]: message"
    let parts: Vec<&str> = line.splitn(6, ' ').collect();
    if parts.len() >= 5 {
        let unit_part = parts[4];
        // Extract unit name before [pid] or :
        let unit = unit_part.split(&['[', ':'][..]).next()?;
        if !unit.is_empty() && unit != "kernel" {
            return Some(unit.to_string());
        }
    }
    None
}

fn truncate_message(msg: &str, max_len: usize) -> String {
    if msg.len() <= max_len {
        msg.to_string()
    } else {
        format!("{}...", &msg[..max_len])
    }
}

// ============================================================================
// Service Failed Detection
// ============================================================================

/// Detect any systemd units in failed state
pub fn detect_service_failed(evidence_id: &str) -> Vec<(ProactiveAlert, ServiceFailedEvidence)> {
    let mut alerts = Vec::new();

    let failed_units = get_failed_units();

    for evidence in failed_units {
        let summary = format!(
            "{} is in failed state ({})",
            evidence.unit,
            evidence.result.as_deref().unwrap_or("unknown")
        );

        let alert = ProactiveAlert::new(
            AlertType::ServiceFailed,
            AlertSeverity::Critical,
            &evidence.unit,
            "Service failed",
            &summary,
        )
        .with_evidence(evidence_id)
        .with_data(serde_json::to_value(&evidence).unwrap_or_default());

        alerts.push((alert, evidence));
    }

    alerts
}

/// Get list of failed systemd units
fn get_failed_units() -> Vec<ServiceFailedEvidence> {
    let output = Command::new("systemctl")
        .args(["list-units", "--state=failed", "--no-legend", "--plain"])
        .output();

    let output = match output {
        Ok(o) if o.status.success() => o,
        _ => return Vec::new(),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut units = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let unit = parts[0].to_string();
        let load_state = parts.get(1).unwrap_or(&"").to_string();
        let active_state = parts.get(2).unwrap_or(&"").to_string();
        let sub_state = parts.get(3).unwrap_or(&"").to_string();

        // Get failure reason
        let result = get_unit_result(&unit);

        units.push(ServiceFailedEvidence {
            unit,
            load_state,
            active_state,
            sub_state,
            result,
        });
    }

    units
}

fn get_unit_result(unit: &str) -> Option<String> {
    let output = Command::new("systemctl")
        .args(["show", unit, "--property=Result"])
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if line.starts_with("Result=") {
            return Some(line[7..].to_string());
        }
    }
    None
}

// ============================================================================
// Thermal Throttling Detection
// ============================================================================

/// Detect thermal throttling (best-effort)
pub fn detect_thermal_throttling(evidence_id: &str) -> Option<(ProactiveAlert, ThermalThrottlingEvidence)> {
    // Try to read CPU package temperature from hwmon
    let evidence = get_cpu_temperature()?;

    let severity = if evidence.current_temp_c >= THERMAL_CRITICAL_TEMP || evidence.is_throttling {
        AlertSeverity::Critical
    } else if evidence.current_temp_c >= THERMAL_WARNING_TEMP {
        AlertSeverity::Warning
    } else {
        return None; // Temperature is fine
    };

    let summary = if evidence.is_throttling {
        format!(
            "CPU thermal throttling active at {:.0}C",
            evidence.current_temp_c
        )
    } else {
        format!(
            "CPU temperature high: {:.0}C (threshold: {:.0}C)",
            evidence.current_temp_c,
            evidence.threshold_temp_c.unwrap_or(THERMAL_WARNING_TEMP)
        )
    };

    let alert = ProactiveAlert::new(
        AlertType::ThermalThrottling,
        severity,
        "cpu",
        "Thermal throttling",
        &summary,
    )
    .with_evidence(evidence_id)
    .with_data(serde_json::to_value(&evidence).unwrap_or_default());

    Some((alert, evidence))
}

/// Get CPU temperature from hwmon
fn get_cpu_temperature() -> Option<ThermalThrottlingEvidence> {
    // Try coretemp (Intel) or k10temp (AMD)
    let hwmon_dirs = std::fs::read_dir("/sys/class/hwmon").ok()?;

    for entry in hwmon_dirs.filter_map(|e| e.ok()) {
        let path = entry.path();

        // Check name
        let name_path = path.join("name");
        let name = std::fs::read_to_string(&name_path).ok()?.trim().to_string();

        if name == "coretemp" || name == "k10temp" {
            // Read Package temp (usually temp1)
            let temp_path = path.join("temp1_input");
            let temp_str = std::fs::read_to_string(&temp_path).ok()?;
            let temp_millic: i64 = temp_str.trim().parse().ok()?;
            let temp_c = temp_millic as f64 / 1000.0;

            // Try to read critical threshold
            let crit_path = path.join("temp1_crit");
            let crit_temp = std::fs::read_to_string(&crit_path).ok()
                .and_then(|s| s.trim().parse::<i64>().ok())
                .map(|t| t as f64 / 1000.0);

            // Check for throttling via /proc/cpuinfo or msr
            let is_throttling = check_thermal_throttle();

            return Some(ThermalThrottlingEvidence {
                sensor_name: name,
                current_temp_c: temp_c,
                threshold_temp_c: crit_temp,
                is_throttling,
            });
        }
    }

    None
}

/// Check if thermal throttling is active
fn check_thermal_throttle() -> bool {
    // Try reading from /sys/devices/system/cpu/cpu0/thermal_throttle/package_throttle_count
    let path = "/sys/devices/system/cpu/cpu0/thermal_throttle/package_throttle_count";
    if let Ok(content) = std::fs::read_to_string(path) {
        if let Ok(count) = content.trim().parse::<u64>() {
            return count > 0;
        }
    }
    false
}

// ============================================================================
// Main Detection Runner
// ============================================================================

/// Run all detectors and return detected alerts
pub fn run_all_detectors() -> Vec<ProactiveAlert> {
    let mut alerts = Vec::new();
    let mut evidence_counter = 1;

    // BOOT_REGRESSION
    let eid = format!("E{}", evidence_counter);
    if let Some((alert, _)) = detect_boot_regression(&eid) {
        alerts.push(alert);
        evidence_counter += 1;
    }

    // DISK_PRESSURE
    let eid = format!("E{}", evidence_counter);
    if let Some((alert, _)) = detect_disk_pressure(&eid) {
        alerts.push(alert);
        evidence_counter += 1;
    }

    // JOURNAL_ERROR_BURST
    for (alert, _) in detect_journal_error_burst(&format!("E{}", evidence_counter)) {
        alerts.push(alert);
        evidence_counter += 1;
    }

    // SERVICE_FAILED
    for (alert, _) in detect_service_failed(&format!("E{}", evidence_counter)) {
        alerts.push(alert);
        evidence_counter += 1;
    }

    // THERMAL_THROTTLING
    let eid = format!("E{}", evidence_counter);
    if let Some((alert, _)) = detect_thermal_throttling(&eid) {
        alerts.push(alert);
    }

    alerts
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_time_duration() {
        assert_eq!(parse_time_duration("45.123s"), Some(45.123));
        assert_eq!(parse_time_duration("1min 23.456s"), Some(83.456));
        assert_eq!(parse_time_duration("2min 0.0s"), Some(120.0));
    }

    #[test]
    fn test_format_percent() {
        assert_eq!(format_percent(0.5), "0.5%");
        assert_eq!(format_percent(5.0), "5%");
        assert_eq!(format_percent(50.0), "50%");
    }

    #[test]
    fn test_truncate_message() {
        let msg = "This is a long message that should be truncated";
        assert_eq!(truncate_message(msg, 20), "This is a long messa...");
        assert_eq!(truncate_message("short", 20), "short");
    }

    #[test]
    fn test_disk_pressure_thresholds() {
        // Verify thresholds are sensible
        assert!(DISK_PRESSURE_CRITICAL_PERCENT < DISK_PRESSURE_WARNING_PERCENT);
        assert!(DISK_PRESSURE_CRITICAL_GIB < DISK_PRESSURE_WARNING_GIB);
    }

    #[test]
    fn test_journal_error_thresholds() {
        assert!(JOURNAL_ERROR_BURST_WARNING < JOURNAL_ERROR_BURST_CRITICAL);
    }
}
