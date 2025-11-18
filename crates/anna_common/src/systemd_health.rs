//! Systemd health detection
//!
//! Detects systemd health and configuration:
//! - Failed systemd units
//! - Essential timer status
//! - Journal disk usage and rotation

use serde::{Deserialize, Serialize};
use std::process::Command;

/// Systemd health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemdHealth {
    /// Failed units (services, timers, mounts, etc.)
    pub failed_units: Vec<FailedUnit>,
    /// Essential timers status
    pub essential_timers: Vec<TimerStatus>,
    /// Journal disk usage in MB
    pub journal_disk_usage_mb: Option<u64>,
    /// Journal rotation is configured
    pub journal_rotation_configured: bool,
}

/// A failed systemd unit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedUnit {
    /// Unit name
    pub name: String,
    /// Unit type (service, timer, mount, etc.)
    pub unit_type: String,
    /// Load state
    pub load_state: String,
    /// Active state
    pub active_state: String,
    /// Sub state
    pub sub_state: String,
}

/// Status of an essential timer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerStatus {
    /// Timer name
    pub name: String,
    /// Is enabled
    pub enabled: bool,
    /// Is active
    pub active: bool,
    /// Next run time (if available)
    pub next_run: Option<String>,
}

impl SystemdHealth {
    /// Detect systemd health
    pub fn detect() -> Self {
        let failed_units = detect_failed_units();
        let essential_timers = detect_essential_timers();
        let journal_disk_usage_mb = detect_journal_disk_usage();
        let journal_rotation_configured = check_journal_rotation_config();

        Self {
            failed_units,
            essential_timers,
            journal_disk_usage_mb,
            journal_rotation_configured,
        }
    }
}

/// Detect failed systemd units
fn detect_failed_units() -> Vec<FailedUnit> {
    let mut failed_units = Vec::new();

    // Get list of failed units
    if let Ok(output) = Command::new("systemctl")
        .arg("list-units")
        .arg("--state=failed")
        .arg("--no-pager")
        .arg("--no-legend")
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let name = parts[0].to_string();
                    let unit_type = extract_unit_type(&name);
                    let load_state = parts[1].to_string();
                    let active_state = parts[2].to_string();
                    let sub_state = parts[3].to_string();

                    failed_units.push(FailedUnit {
                        name,
                        unit_type,
                        load_state,
                        active_state,
                        sub_state,
                    });
                }
            }
        }
    }

    failed_units
}

/// Extract unit type from unit name (e.g., "foo.service" -> "service")
fn extract_unit_type(name: &str) -> String {
    if let Some(dot_pos) = name.rfind('.') {
        name[dot_pos + 1..].to_string()
    } else {
        "unknown".to_string()
    }
}

/// Detect status of essential timers
fn detect_essential_timers() -> Vec<TimerStatus> {
    let essential_timer_names = vec![
        "fstrim.timer",
        "reflector.timer",
        "paccache.timer",
        "systemd-tmpfiles-clean.timer",
    ];

    let mut timer_statuses = Vec::new();

    for timer_name in essential_timer_names {
        if let Some(status) = get_timer_status(timer_name) {
            timer_statuses.push(status);
        }
    }

    timer_statuses
}

/// Get status of a specific timer
fn get_timer_status(timer_name: &str) -> Option<TimerStatus> {
    let enabled = is_timer_enabled(timer_name);
    let active = is_timer_active(timer_name);
    let next_run = get_timer_next_run(timer_name);

    // Only include timer if it exists (either enabled or active)
    if enabled || active || next_run.is_some() {
        Some(TimerStatus {
            name: timer_name.to_string(),
            enabled,
            active,
            next_run,
        })
    } else {
        None
    }
}

/// Check if timer is enabled
fn is_timer_enabled(timer_name: &str) -> bool {
    if let Ok(output) = Command::new("systemctl")
        .arg("is-enabled")
        .arg(timer_name)
        .output()
    {
        if output.status.success() {
            let status = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return status == "enabled";
        }
    }
    false
}

/// Check if timer is active
fn is_timer_active(timer_name: &str) -> bool {
    if let Ok(output) = Command::new("systemctl")
        .arg("is-active")
        .arg(timer_name)
        .output()
    {
        if output.status.success() {
            let status = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return status == "active";
        }
    }
    false
}

/// Get next run time for timer
fn get_timer_next_run(timer_name: &str) -> Option<String> {
    if let Ok(output) = Command::new("systemctl")
        .arg("status")
        .arg(timer_name)
        .arg("--no-pager")
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains("Trigger:") {
                    // Extract trigger time
                    if let Some(trigger_pos) = line.find("Trigger:") {
                        let trigger_text = line[trigger_pos + 8..].trim();
                        return Some(trigger_text.to_string());
                    }
                }
            }
        }
    }
    None
}

/// Detect journal disk usage
fn detect_journal_disk_usage() -> Option<u64> {
    if let Ok(output) = Command::new("journalctl")
        .arg("--disk-usage")
        .arg("--no-pager")
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Expected format: "Archived and active journals take up 123.4M in the file system."
            for line in stdout.lines() {
                if line.contains("take up") {
                    // Extract size
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    for (i, part) in parts.iter().enumerate() {
                        if *part == "up" && i + 1 < parts.len() {
                            let size_str = parts[i + 1];
                            return parse_size_to_mb(size_str);
                        }
                    }
                }
            }
        }
    }
    None
}

/// Parse size string to MB (e.g., "123.4M" -> 123, "1.2G" -> 1200)
fn parse_size_to_mb(size_str: &str) -> Option<u64> {
    let size_str = size_str.trim();
    if size_str.is_empty() {
        return None;
    }

    // Extract numeric part and unit
    let numeric_part: String = size_str
        .chars()
        .take_while(|c| c.is_numeric() || *c == '.')
        .collect();
    let unit = size_str
        .chars()
        .skip(numeric_part.len())
        .collect::<String>();

    if let Ok(value) = numeric_part.parse::<f64>() {
        let mb = match unit.to_uppercase().as_str() {
            "B" => value / 1024.0 / 1024.0,
            "K" | "KB" => value / 1024.0,
            "M" | "MB" => value,
            "G" | "GB" => value * 1024.0,
            "T" | "TB" => value * 1024.0 * 1024.0,
            _ => return None,
        };
        return Some(mb as u64);
    }

    None
}

/// Check if journal rotation is configured
fn check_journal_rotation_config() -> bool {
    // Check journald.conf for SystemMaxUse or RuntimeMaxUse settings
    if let Ok(content) = std::fs::read_to_string("/etc/systemd/journald.conf") {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("SystemMaxUse=") || line.starts_with("RuntimeMaxUse=") {
                // Check if it's not commented out
                if !line.starts_with('#') {
                    return true;
                }
            }
        }
    }

    // Check drop-in configs
    let dropins_dir = "/etc/systemd/journald.conf.d";
    if let Ok(entries) = std::fs::read_dir(dropins_dir) {
        for entry in entries.flatten() {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                for line in content.lines() {
                    let line = line.trim();
                    if (line.starts_with("SystemMaxUse=") || line.starts_with("RuntimeMaxUse="))
                        && !line.starts_with('#')
                    {
                        return true;
                    }
                }
            }
        }
    }

    false
}
