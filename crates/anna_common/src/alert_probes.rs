//! Alert Probes v0.0.58 - Daemon-owned probes for alert detection
//!
//! Read-only probes that gather evidence for the proactive alert system.
//! These are called by annad on an interval; annactl reads the results from snapshot.
//!
//! Probes:
//! - boot_time_summary
//! - disk_pressure_summary
//! - journal_error_burst_summary
//! - failed_units_summary
//! - thermal_summary
//! - alerts_summary (for user queries)

use serde::{Deserialize, Serialize};

use crate::alert_detectors::*;
use crate::proactive_alerts::{AlertCounts, ProactiveAlert, ProactiveAlertsState};
use crate::tools::ToolResult;

// ============================================================================
// Probe Results
// ============================================================================

/// Boot time probe result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootTimeSummary {
    pub last_boot_secs: Option<f64>,
    pub baseline_mean_secs: Option<f64>,
    pub is_regression: bool,
    pub regression: Option<BootRegressionEvidence>,
}

/// Disk pressure probe result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskPressureSummary {
    pub mount_point: String,
    pub free_gib: f64,
    pub free_percent: f64,
    pub is_pressure: bool,
    pub evidence: Option<DiskPressureEvidence>,
}

/// Journal error burst probe result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalErrorBurstSummary {
    pub units_with_errors: Vec<JournalErrorBurstEvidence>,
    pub total_units_affected: usize,
}

/// Failed units probe result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedUnitsSummary {
    pub failed_units: Vec<ServiceFailedEvidence>,
    pub count: usize,
}

/// Thermal probe result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalSummary {
    pub cpu_temp_c: Option<f64>,
    pub is_throttling: bool,
    pub evidence: Option<ThermalThrottlingEvidence>,
}

/// Alerts summary for user queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertsSummary {
    pub counts: AlertCountsData,
    pub active_alerts: Vec<AlertSummaryEntry>,
    pub recently_resolved: Vec<AlertSummaryEntry>,
    pub snapshot_age: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertCountsData {
    pub critical: usize,
    pub warning: usize,
    pub info: usize,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertSummaryEntry {
    pub id: String,
    pub alert_type: String,
    pub severity: String,
    pub title: String,
    pub summary: String,
    pub age: String,
    pub evidence_ids: Vec<String>,
}

// ============================================================================
// Probe Execution
// ============================================================================

/// Execute boot_time_summary probe
pub fn probe_boot_time_summary(evidence_id: &str) -> ToolResult {
    let result = detect_boot_regression(evidence_id);

    let summary = match &result {
        Some((_, evidence)) => BootTimeSummary {
            last_boot_secs: Some(evidence.last_boot_secs),
            baseline_mean_secs: Some(evidence.baseline_mean_secs),
            is_regression: true,
            regression: Some(evidence.clone()),
        },
        None => {
            // Get current boot time even if no regression
            let last_boot = get_current_boot_time();
            BootTimeSummary {
                last_boot_secs: last_boot,
                baseline_mean_secs: None,
                is_regression: false,
                regression: None,
            }
        }
    };

    let human = if summary.is_regression {
        format!(
            "Boot regression detected: {:.1}s (baseline: {:.1}s)",
            summary.last_boot_secs.unwrap_or(0.0),
            summary.baseline_mean_secs.unwrap_or(0.0)
        )
    } else {
        format!(
            "Boot time: {:.1}s (no regression)",
            summary.last_boot_secs.unwrap_or(0.0)
        )
    };

    ToolResult {
        tool_name: "boot_time_summary".to_string(),
        evidence_id: evidence_id.to_string(),
        data: serde_json::to_value(&summary).unwrap_or_default(),
        human_summary: human,
        success: true,
        error: None,
        timestamp: current_timestamp(),
    }
}

/// Execute disk_pressure_summary probe
pub fn probe_disk_pressure_summary(evidence_id: &str) -> ToolResult {
    let result = detect_disk_pressure(evidence_id);

    let summary = match &result {
        Some((_, evidence)) => DiskPressureSummary {
            mount_point: evidence.mount_point.clone(),
            free_gib: evidence.free_gib,
            free_percent: 100.0 - evidence.used_percent,
            is_pressure: true,
            evidence: Some(evidence.clone()),
        },
        None => {
            // Get disk usage even if no pressure
            let (free_gib, free_percent) = get_root_disk_usage();
            DiskPressureSummary {
                mount_point: "/".to_string(),
                free_gib,
                free_percent,
                is_pressure: false,
                evidence: None,
            }
        }
    };

    let human = if summary.is_pressure {
        format!(
            "Disk pressure on {}: {:.1} GiB free ({:.1}%)",
            summary.mount_point, summary.free_gib, summary.free_percent
        )
    } else {
        format!(
            "Disk /: {:.1} GiB free ({:.1}%) - OK",
            summary.free_gib, summary.free_percent
        )
    };

    ToolResult {
        tool_name: "disk_pressure_summary".to_string(),
        evidence_id: evidence_id.to_string(),
        data: serde_json::to_value(&summary).unwrap_or_default(),
        human_summary: human,
        success: true,
        error: None,
        timestamp: current_timestamp(),
    }
}

/// Execute journal_error_burst_summary probe
pub fn probe_journal_error_burst_summary(evidence_id: &str) -> ToolResult {
    let results = detect_journal_error_burst(evidence_id);

    let units: Vec<JournalErrorBurstEvidence> = results.into_iter().map(|(_, e)| e).collect();

    let summary = JournalErrorBurstSummary {
        units_with_errors: units.clone(),
        total_units_affected: units.len(),
    };

    let human = if units.is_empty() {
        "No journal error bursts detected".to_string()
    } else {
        let total_errors: u32 = units.iter().map(|u| u.error_count).sum();
        format!(
            "{} units with error bursts ({} total errors in last {} min)",
            units.len(),
            total_errors,
            JOURNAL_ERROR_BURST_WINDOW_MINS
        )
    };

    ToolResult {
        tool_name: "journal_error_burst_summary".to_string(),
        evidence_id: evidence_id.to_string(),
        data: serde_json::to_value(&summary).unwrap_or_default(),
        human_summary: human,
        success: true,
        error: None,
        timestamp: current_timestamp(),
    }
}

/// Execute failed_units_summary probe
pub fn probe_failed_units_summary(evidence_id: &str) -> ToolResult {
    let results = detect_service_failed(evidence_id);

    let units: Vec<ServiceFailedEvidence> = results.into_iter().map(|(_, e)| e).collect();

    let summary = FailedUnitsSummary {
        failed_units: units.clone(),
        count: units.len(),
    };

    let human = if units.is_empty() {
        "No failed systemd units".to_string()
    } else {
        let names: Vec<&str> = units.iter().map(|u| u.unit.as_str()).collect();
        format!("{} failed units: {}", units.len(), names.join(", "))
    };

    ToolResult {
        tool_name: "failed_units_summary".to_string(),
        evidence_id: evidence_id.to_string(),
        data: serde_json::to_value(&summary).unwrap_or_default(),
        human_summary: human,
        success: true,
        error: None,
        timestamp: current_timestamp(),
    }
}

/// Execute thermal_summary probe
pub fn probe_thermal_summary(evidence_id: &str) -> ToolResult {
    let result = detect_thermal_throttling(evidence_id);

    let summary = match &result {
        Some((_, evidence)) => ThermalSummary {
            cpu_temp_c: Some(evidence.current_temp_c),
            is_throttling: evidence.is_throttling,
            evidence: Some(evidence.clone()),
        },
        None => {
            // Try to get temp even if no throttling
            let temp = get_cpu_temp();
            ThermalSummary {
                cpu_temp_c: temp,
                is_throttling: false,
                evidence: None,
            }
        }
    };

    let human = match (summary.cpu_temp_c, summary.is_throttling) {
        (Some(temp), true) => format!("CPU thermal throttling at {:.0}C", temp),
        (Some(temp), false) => format!("CPU temperature: {:.0}C - OK", temp),
        (None, _) => "Thermal sensors not available".to_string(),
    };

    ToolResult {
        tool_name: "thermal_summary".to_string(),
        evidence_id: evidence_id.to_string(),
        data: serde_json::to_value(&summary).unwrap_or_default(),
        human_summary: human,
        success: true,
        error: None,
        timestamp: current_timestamp(),
    }
}

/// Execute alerts_summary probe (for user queries)
pub fn probe_alerts_summary(evidence_id: &str) -> ToolResult {
    let state = ProactiveAlertsState::load();

    let counts = state.count_by_severity();
    let active_alerts: Vec<AlertSummaryEntry> = state
        .get_active()
        .into_iter()
        .map(|a| AlertSummaryEntry {
            id: a.id.clone(),
            alert_type: a.alert_type.to_string(),
            severity: a.severity.to_string(),
            title: a.title.clone(),
            summary: a.summary.clone(),
            age: a.age_str(),
            evidence_ids: a.evidence_ids.clone(),
        })
        .collect();

    let recently_resolved: Vec<AlertSummaryEntry> = state
        .recently_resolved
        .iter()
        .map(|a| AlertSummaryEntry {
            id: a.id.clone(),
            alert_type: a.alert_type.to_string(),
            severity: a.severity.to_string(),
            title: a.title.clone(),
            summary: a.summary.clone(),
            age: a.age_str(),
            evidence_ids: a.evidence_ids.clone(),
        })
        .collect();

    let summary = AlertsSummary {
        counts: AlertCountsData {
            critical: counts.critical,
            warning: counts.warning,
            info: counts.info,
            total: counts.total(),
        },
        active_alerts,
        recently_resolved,
        snapshot_age: state.snapshot_age_str(),
    };

    let human = if counts.total() == 0 {
        "No active alerts".to_string()
    } else {
        format!(
            "{} active alerts ({} critical, {} warnings, {} info)",
            counts.total(),
            counts.critical,
            counts.warning,
            counts.info
        )
    };

    ToolResult {
        tool_name: "alerts_summary".to_string(),
        evidence_id: evidence_id.to_string(),
        data: serde_json::to_value(&summary).unwrap_or_default(),
        human_summary: human,
        success: true,
        error: None,
        timestamp: current_timestamp(),
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn get_current_boot_time() -> Option<f64> {
    let output = std::process::Command::new("systemd-analyze")
        .arg("time")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse: look for total time
    for line in stdout.lines() {
        if line.contains("Startup finished") {
            if let Some(pos) = line.rfind('=') {
                let time_str = line[pos + 1..].trim();
                return parse_boot_time(time_str);
            }
        }
    }
    None
}

fn parse_boot_time(s: &str) -> Option<f64> {
    let s = s.trim();
    let mut total = 0.0;

    if s.contains("min") {
        let parts: Vec<&str> = s.split("min").collect();
        if let Ok(mins) = parts[0].trim().parse::<f64>() {
            total += mins * 60.0;
        }
        if parts.len() > 1 {
            if let Some(secs) = parts[1].trim().trim_end_matches('s').parse::<f64>().ok() {
                total += secs;
            }
        }
        return Some(total);
    }

    s.trim_end_matches('s').parse::<f64>().ok()
}

fn get_root_disk_usage() -> (f64, f64) {
    let output = std::process::Command::new("df").args(["-B1", "/"]).output();

    let output = match output {
        Ok(o) if o.status.success() => o,
        _ => return (0.0, 0.0),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();

    if lines.len() < 2 {
        return (0.0, 0.0);
    }

    let parts: Vec<&str> = lines[1].split_whitespace().collect();
    if parts.len() < 4 {
        return (0.0, 0.0);
    }

    let total: u64 = parts[1].parse().unwrap_or(1);
    let free: u64 = parts[3].parse().unwrap_or(0);

    let free_gib = free as f64 / (1024.0 * 1024.0 * 1024.0);
    let free_percent = (free as f64 / total as f64) * 100.0;

    (free_gib, free_percent)
}

fn get_cpu_temp() -> Option<f64> {
    // Try coretemp or k10temp
    for hwmon in std::fs::read_dir("/sys/class/hwmon").ok()? {
        let path = hwmon.ok()?.path();
        let name = std::fs::read_to_string(path.join("name")).ok()?;
        if name.trim() == "coretemp" || name.trim() == "k10temp" {
            let temp_str = std::fs::read_to_string(path.join("temp1_input")).ok()?;
            let temp_millic: i64 = temp_str.trim().parse().ok()?;
            return Some(temp_millic as f64 / 1000.0);
        }
    }
    None
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_boot_time() {
        assert_eq!(parse_boot_time("45.123s"), Some(45.123));
        assert_eq!(parse_boot_time("1min 23s"), Some(83.0));
    }

    #[test]
    fn test_probe_alerts_summary_empty() {
        // Should work even with no alerts file
        let result = probe_alerts_summary("E1");
        assert!(result.success);
    }
}
