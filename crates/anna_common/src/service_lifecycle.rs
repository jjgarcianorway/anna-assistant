//! Service Lifecycle v7.16.0 - Track systemd unit lifecycle and restarts
//!
//! Provides structured lifecycle information for systemd services:
//! - Current state (active, inactive, failed)
//! - Restart counts per boot
//! - Exit codes and status
//! - Activation failures over time windows (24h, 7d)
//!
//! Sources:
//! - systemctl show <unit>
//! - systemctl status <unit>
//! - journalctl -u <unit>

use serde::{Deserialize, Serialize};
use std::process::Command;

/// Lifecycle summary for a systemd unit
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceLifecycle {
    /// Unit name (e.g., "sshd.service")
    pub unit_name: String,
    /// Current active state
    pub state: String,
    /// Sub-state (e.g., "running", "exited", "dead")
    pub sub_state: String,
    /// Number of restarts this boot
    pub restarts_this_boot: u32,
    /// Last exit code
    pub last_exit_code: Option<i32>,
    /// Last exit status description
    pub last_exit_status: String,
    /// Activation failures in last 24h
    pub failures_24h: usize,
    /// Activation failures in last 7d
    pub failures_7d: usize,
    /// Whether unit is static (no restart semantics)
    pub is_static: bool,
    /// Whether unit exists
    pub exists: bool,
    /// Source description
    pub source: String,
}

impl ServiceLifecycle {
    /// Query lifecycle information for a unit
    pub fn query(unit_name: &str) -> Self {
        let mut lifecycle = ServiceLifecycle {
            unit_name: unit_name.to_string(),
            source: "systemctl show, journalctl".to_string(),
            ..Default::default()
        };

        // Query systemctl show for current state
        let output = Command::new("systemctl")
            .args([
                "show", unit_name,
                "--property=ActiveState,SubState,UnitFileState,NRestarts,ExecMainCode,ExecMainStatus,Type",
            ])
            .output();

        if let Ok(out) = output {
            if !out.status.success() {
                lifecycle.exists = false;
                lifecycle.state = "not found".to_string();
                return lifecycle;
            }

            lifecycle.exists = true;
            let stdout = String::from_utf8_lossy(&out.stdout);

            for line in stdout.lines() {
                if let Some((key, value)) = line.split_once('=') {
                    match key {
                        "ActiveState" => lifecycle.state = value.to_string(),
                        "SubState" => lifecycle.sub_state = value.to_string(),
                        "UnitFileState" => {
                            lifecycle.is_static = value == "static" || value == "indirect";
                        }
                        "NRestarts" => {
                            lifecycle.restarts_this_boot = value.parse().unwrap_or(0);
                        }
                        "ExecMainCode" => {
                            if value != "0" && !value.is_empty() {
                                lifecycle.last_exit_code = value.parse().ok();
                            }
                        }
                        "ExecMainStatus" => {
                            lifecycle.last_exit_code = value.parse().ok();
                        }
                        "Type" => {
                            if value == "oneshot" {
                                lifecycle.is_static = true;
                            }
                        }
                        _ => {}
                    }
                }
            }
        } else {
            lifecycle.exists = false;
            lifecycle.state = "not found".to_string();
            return lifecycle;
        }

        // Determine exit status description
        lifecycle.last_exit_status = match lifecycle.last_exit_code {
            Some(0) => "success".to_string(),
            Some(code) => format!("code={}", code),
            None if lifecycle.state == "active" => "running".to_string(),
            None => "unknown".to_string(),
        };

        // Query failures from journalctl
        lifecycle.failures_24h = count_activation_failures(unit_name, "24h");
        lifecycle.failures_7d = count_activation_failures(unit_name, "7d");

        lifecycle
    }

    /// Format state for display (e.g., "active (running)")
    pub fn format_state(&self) -> String {
        if self.sub_state.is_empty() || self.sub_state == "unknown" {
            self.state.clone()
        } else {
            format!("{} ({})", self.state, self.sub_state)
        }
    }

    /// Format restarts for display
    pub fn format_restarts(&self) -> String {
        if self.is_static {
            "not applicable (static unit)".to_string()
        } else {
            format!("{} this boot", self.restarts_this_boot)
        }
    }

    /// Format last exit for display
    pub fn format_last_exit(&self) -> String {
        if let Some(code) = self.last_exit_code {
            format!("code={} status={}", code, self.last_exit_status)
        } else if self.state == "active" {
            "n/a (currently running)".to_string()
        } else {
            format!("status={}", self.last_exit_status)
        }
    }
}

/// Count activation failures in a time window
fn count_activation_failures(unit: &str, window: &str) -> usize {
    let since = match window {
        "24h" => "24 hours ago",
        "7d" => "7 days ago",
        "30d" => "30 days ago",
        _ => return 0,
    };

    let output = Command::new("journalctl")
        .args(["--since", since, "-u", unit, "--no-pager", "-q"])
        .output();

    let logs = match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => return 0,
    };

    // Count failure indicators
    let failure_patterns = [
        "Failed to start",
        "failed with",
        "activation failure",
        "Start request repeated too quickly",
        "service failed",
        "Main process exited, code=exited, status=",
    ];

    let mut count = 0;
    for line in logs.lines() {
        let line_lower = line.to_lowercase();
        for pattern in &failure_patterns {
            if line_lower.contains(&pattern.to_lowercase()) {
                count += 1;
                break;
            }
        }
    }

    count
}

/// Summary of service lifecycle for multiple units
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceLifecycleSummary {
    /// Units with their lifecycle data
    pub units: Vec<ServiceLifecycle>,
    /// Source description
    pub source: String,
}

impl ServiceLifecycleSummary {
    /// Query lifecycle for multiple units
    pub fn query_units(unit_names: &[&str]) -> Self {
        let units = unit_names
            .iter()
            .map(|name| ServiceLifecycle::query(name))
            .filter(|l| l.exists)
            .collect();

        ServiceLifecycleSummary {
            units,
            source: "systemctl show, journalctl, Anna lifecycle history".to_string(),
        }
    }

    /// Check if any unit has failures
    pub fn has_failures(&self) -> bool {
        self.units
            .iter()
            .any(|u| u.failures_24h > 0 || u.failures_7d > 0)
    }

    /// Get units with restarts this boot
    pub fn units_with_restarts(&self) -> Vec<&ServiceLifecycle> {
        self.units
            .iter()
            .filter(|u| u.restarts_this_boot > 0)
            .collect()
    }
}

/// Find related units for a package or component
pub fn find_related_units(name: &str) -> Vec<String> {
    let mut units = Vec::new();

    // Try direct service name
    let service_name = if name.ends_with(".service") {
        name.to_string()
    } else {
        format!("{}.service", name)
    };

    // Check if unit exists
    let output = Command::new("systemctl")
        .args(["cat", &service_name])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            units.push(service_name.clone());
        }
    }

    // Try socket and timer
    let base_name = name.trim_end_matches(".service");
    for suffix in &[".socket", ".timer"] {
        let unit = format!("{}{}", base_name, suffix);
        let output = Command::new("systemctl").args(["cat", &unit]).output();

        if let Ok(out) = output {
            if out.status.success() {
                units.push(unit);
            }
        }
    }

    // Try common variations
    let variations = [
        format!("{}d.service", base_name), // sshd, httpd, etc.
        format!("{}-daemon.service", base_name),
    ];

    for variant in &variations {
        if !units.contains(variant) {
            let output = Command::new("systemctl").args(["cat", variant]).output();

            if let Ok(out) = output {
                if out.status.success() {
                    units.push(variant.clone());
                }
            }
        }
    }

    units
}

/// Get related units for a hardware component
pub fn find_hardware_related_units(component: &str) -> Vec<String> {
    let component_lower = component.to_lowercase();
    let mut units = Vec::new();

    // Known mappings
    let mappings: &[(&str, &[&str])] = &[
        (
            "wifi",
            &[
                "NetworkManager.service",
                "wpa_supplicant.service",
                "iwd.service",
            ],
        ),
        (
            "network",
            &[
                "NetworkManager.service",
                "systemd-networkd.service",
                "systemd-resolved.service",
            ],
        ),
        (
            "ethernet",
            &["NetworkManager.service", "systemd-networkd.service"],
        ),
        ("bluetooth", &["bluetooth.service"]),
        (
            "audio",
            &[
                "pipewire.service",
                "pipewire-pulse.service",
                "pulseaudio.service",
            ],
        ),
        (
            "power",
            &["upower.service", "tlp.service", "thermald.service"],
        ),
        ("battery", &["upower.service"]),
        ("storage", &["udisks2.service"]),
        ("gpu", &["nvidia-persistenced.service"]),
    ];

    for (key, services) in mappings {
        if component_lower.contains(key) {
            for service in *services {
                // Check if service exists
                let output = Command::new("systemctl").args(["cat", service]).output();

                if let Ok(out) = output {
                    if out.status.success() && !units.contains(&service.to_string()) {
                        units.push(service.to_string());
                    }
                }
            }
        }
    }

    units
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_lifecycle_query() {
        // Test with a common service that should exist
        let lifecycle = ServiceLifecycle::query("dbus.service");
        assert!(lifecycle.exists || lifecycle.state == "not found");
        assert!(!lifecycle.unit_name.is_empty());
    }

    #[test]
    fn test_format_state() {
        let mut lifecycle = ServiceLifecycle::default();
        lifecycle.state = "active".to_string();
        lifecycle.sub_state = "running".to_string();
        assert_eq!(lifecycle.format_state(), "active (running)");
    }

    #[test]
    fn test_format_restarts_static() {
        let mut lifecycle = ServiceLifecycle::default();
        lifecycle.is_static = true;
        assert!(lifecycle.format_restarts().contains("not applicable"));
    }

    #[test]
    fn test_format_restarts_normal() {
        let mut lifecycle = ServiceLifecycle::default();
        lifecycle.restarts_this_boot = 3;
        assert!(lifecycle.format_restarts().contains("3 this boot"));
    }

    #[test]
    fn test_find_related_units() {
        let units = find_related_units("dbus");
        // dbus.service should exist on most systems
        // Just check it doesn't crash
        assert!(units.is_empty() || units.iter().any(|u| u.contains("dbus")));
    }
}
