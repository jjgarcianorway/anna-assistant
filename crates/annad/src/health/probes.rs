//! Health probe definitions and execution
//!
//! Phase 0.5: Individual health probes with safe execution
//! Citation: [archwiki:System_maintenance]

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// Health probe status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProbeStatus {
    Ok,
    Warn,
    Fail,
}

/// Result from running a health probe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeResult {
    pub probe: String,
    pub status: ProbeStatus,
    pub details: serde_json::Value,
    pub citation: String,
    pub duration_ms: u64,
}

/// Health probe interface
pub trait HealthProbe {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn citation(&self) -> &str;
    fn run(&self) -> Result<ProbeResult>;
}

/// Disk space probe
pub struct DiskSpaceProbe {
    pub warn_threshold: u32,
    pub fail_threshold: u32,
}

impl HealthProbe for DiskSpaceProbe {
    fn name(&self) -> &str {
        "disk-space"
    }

    fn description(&self) -> &str {
        "Check filesystem disk space usage"
    }

    fn citation(&self) -> &str {
        "[archwiki:System_maintenance#Check_for_errors]"
    }

    fn run(&self) -> Result<ProbeResult> {
        let start = Instant::now();

        // Run df command (read-only)
        let output = Command::new("df")
            .args(&["-h", "/"])
            .output()
            .context("Failed to execute df command")?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();

        // Parse output
        let status = if lines.len() >= 2 {
            let data_line = lines[1];
            let fields: Vec<&str> = data_line.split_whitespace().collect();

            if fields.len() >= 5 {
                let usage_str = fields[4].trim_end_matches('%');
                if let Ok(usage) = usage_str.parse::<u32>() {
                    if usage >= self.fail_threshold {
                        ProbeStatus::Fail
                    } else if usage >= self.warn_threshold {
                        ProbeStatus::Warn
                    } else {
                        ProbeStatus::Ok
                    }
                } else {
                    ProbeStatus::Warn
                }
            } else {
                ProbeStatus::Warn
            }
        } else {
            ProbeStatus::Warn
        };

        Ok(ProbeResult {
            probe: self.name().to_string(),
            status,
            details: serde_json::json!({
                "output": stdout.trim(),
            }),
            citation: self.citation().to_string(),
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }
}

/// Systemd units probe
pub struct SystemdUnitsProbe;

impl HealthProbe for SystemdUnitsProbe {
    fn name(&self) -> &str {
        "systemd-units"
    }

    fn description(&self) -> &str {
        "Check for failed systemd units"
    }

    fn citation(&self) -> &str {
        "[archwiki:Systemd#Basic_systemctl_usage]"
    }

    fn run(&self) -> Result<ProbeResult> {
        let start = Instant::now();

        // Run systemctl --failed (read-only)
        let output = Command::new("systemctl")
            .args(&["--failed", "--no-pager", "--no-legend"])
            .output()
            .context("Failed to execute systemctl command")?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let failed_count = stdout.lines().count();

        let status = if failed_count == 0 {
            ProbeStatus::Ok
        } else if failed_count <= 2 {
            ProbeStatus::Warn
        } else {
            ProbeStatus::Fail
        };

        Ok(ProbeResult {
            probe: self.name().to_string(),
            status,
            details: serde_json::json!({
                "failed_count": failed_count,
                "failed_units": stdout.trim(),
            }),
            citation: self.citation().to_string(),
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }
}

/// Pacman database probe
pub struct PacmanDbProbe;

impl HealthProbe for PacmanDbProbe {
    fn name(&self) -> &str {
        "pacman-db"
    }

    fn description(&self) -> &str {
        "Verify pacman database integrity"
    }

    fn citation(&self) -> &str {
        "[archwiki:Pacman#Databases]"
    }

    fn run(&self) -> Result<ProbeResult> {
        let start = Instant::now();

        // Simple check: query pacman itself (read-only)
        let output = Command::new("pacman")
            .args(&["-Q", "pacman"])
            .output()
            .context("Failed to execute pacman command")?;

        let status = if output.status.success() {
            ProbeStatus::Ok
        } else {
            ProbeStatus::Fail
        };

        let stdout = String::from_utf8_lossy(&output.stdout);

        Ok(ProbeResult {
            probe: self.name().to_string(),
            status,
            details: serde_json::json!({
                "pacman_version": stdout.trim(),
                "database_accessible": output.status.success(),
            }),
            citation: self.citation().to_string(),
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }
}

/// TLP configuration probe - detects if TLP is installed but not enabled
/// Citation: [archwiki:TLP#Installation]
pub struct TlpConfigProbe;

impl HealthProbe for TlpConfigProbe {
    fn name(&self) -> &str {
        "tlp-config"
    }

    fn description(&self) -> &str {
        "Check if TLP service is properly configured"
    }

    fn citation(&self) -> &str {
        "[archwiki:TLP#Installation]"
    }

    fn run(&self) -> Result<ProbeResult> {
        let start = Instant::now();

        // Check if TLP package is installed
        let tlp_installed = Command::new("pacman")
            .args(&["-Q", "tlp"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        let status = if !tlp_installed {
            // TLP not installed = OK (no misconfiguration)
            ProbeStatus::Ok
        } else {
            // TLP installed - check if service is enabled
            let service_enabled = Command::new("systemctl")
                .args(&["is-enabled", "tlp.service"])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

            if service_enabled {
                ProbeStatus::Ok
            } else {
                ProbeStatus::Warn
            }
        };

        let message = match status {
            ProbeStatus::Warn => "TLP installed but tlp.service not enabled - power saving won't apply on boot",
            _ => "TLP configuration OK"
        };

        Ok(ProbeResult {
            probe: self.name().to_string(),
            status,
            details: serde_json::json!({
                "message": message,
                "installed": tlp_installed,
            }),
            citation: self.citation().to_string(),
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }
}

/// Missing firmware probe - checks dmesg for firmware load failures
/// Citation: [archwiki:mkinitcpio#Possibly_missing_firmware_for_module_XXXX]
pub struct MissingFirmwareProbe;

impl HealthProbe for MissingFirmwareProbe {
    fn name(&self) -> &str {
        "missing-firmware"
    }

    fn description(&self) -> &str {
        "Check for missing firmware in dmesg"
    }

    fn citation(&self) -> &str {
        "[archwiki:mkinitcpio#Possibly_missing_firmware_for_module_XXXX]"
    }

    fn run(&self) -> Result<ProbeResult> {
        let start = Instant::now();

        // Check dmesg for firmware load failures
        let output = Command::new("dmesg")
            .output()
            .context("Failed to execute dmesg")?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Look for common firmware error patterns
        let firmware_errors: Vec<&str> = stdout
            .lines()
            .filter(|line| {
                line.contains("firmware: failed to load") ||
                line.contains("Direct firmware load") ||
                line.contains("Possible missing firmware")
            })
            .take(5) // Limit to first 5
            .collect();

        let status = if firmware_errors.is_empty() {
            ProbeStatus::Ok
        } else if firmware_errors.len() <= 2 {
            ProbeStatus::Warn
        } else {
            ProbeStatus::Fail
        };

        Ok(ProbeResult {
            probe: self.name().to_string(),
            status,
            details: serde_json::json!({
                "message": if firmware_errors.is_empty() {
                    "No firmware errors detected"
                } else {
                    "Missing firmware detected - may affect hardware functionality"
                },
                "errors": firmware_errors,
                "count": firmware_errors.len(),
            }),
            citation: self.citation().to_string(),
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }
}

/// Bluetooth service probe - checks if bluetooth service is failing
/// Citation: [archwiki:Bluetooth#Installation]
pub struct BluetoothServiceProbe;

impl HealthProbe for BluetoothServiceProbe {
    fn name(&self) -> &str {
        "bluetooth-service"
    }

    fn description(&self) -> &str {
        "Check if Bluetooth service is working"
    }

    fn citation(&self) -> &str {
        "[archwiki:Bluetooth#Installation]"
    }

    fn run(&self) -> Result<ProbeResult> {
        let start = Instant::now();

        // Check if bluetooth package is installed
        let bluez_installed = Command::new("pacman")
            .args(&["-Q", "bluez"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        let status = if !bluez_installed {
            // Bluetooth not installed = OK (user doesn't want it)
            ProbeStatus::Ok
        } else {
            // Bluetooth installed - check service status
            let service_active = Command::new("systemctl")
                .args(&["is-active", "bluetooth.service"])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

            if service_active {
                ProbeStatus::Ok
            } else {
                // Check if it's enabled but failed
                let service_enabled = Command::new("systemctl")
                    .args(&["is-enabled", "bluetooth.service"])
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false);

                if service_enabled {
                    ProbeStatus::Fail // Enabled but not running = problem
                } else {
                    ProbeStatus::Warn // Installed but not enabled = potential issue
                }
            }
        };

        let message = match status {
            ProbeStatus::Fail => "Bluetooth service enabled but not running",
            ProbeStatus::Warn => "Bluetooth installed but service not enabled",
            _ => "Bluetooth configuration OK"
        };

        Ok(ProbeResult {
            probe: self.name().to_string(),
            status,
            details: serde_json::json!({
                "message": message,
                "installed": bluez_installed,
            }),
            citation: self.citation().to_string(),
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }
}

/// Mockable test probe for integration testing
/// Reads TEST_PROBE_STATUS environment variable to force specific outcomes
#[cfg(test)]
pub struct MockableProbe {
    pub name: String,
    pub forced_status: Option<ProbeStatus>,
}

#[cfg(test)]
impl MockableProbe {
    pub fn new(name: String, forced_status: Option<ProbeStatus>) -> Self {
        Self {
            name,
            forced_status,
        }
    }

    pub fn from_env(name: String) -> Self {
        let forced_status = std::env::var(&format!(
            "TEST_PROBE_STATUS_{}",
            name.to_uppercase().replace("-", "_")
        ))
        .ok()
        .and_then(|s| match s.as_str() {
            "ok" => Some(ProbeStatus::Ok),
            "warn" => Some(ProbeStatus::Warn),
            "fail" => Some(ProbeStatus::Fail),
            _ => None,
        });
        Self {
            name,
            forced_status,
        }
    }
}

#[cfg(test)]
impl HealthProbe for MockableProbe {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "Mockable test probe"
    }

    fn citation(&self) -> &str {
        "[archwiki:System_maintenance]"
    }

    fn run(&self) -> Result<ProbeResult> {
        let start = Instant::now();

        let status = self.forced_status.clone().unwrap_or(ProbeStatus::Ok);

        Ok(ProbeResult {
            probe: self.name.clone(),
            status,
            details: serde_json::json!({
                "test": true,
                "forced": self.forced_status.is_some(),
            }),
            citation: self.citation().to_string(),
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disk_space_probe() {
        let probe = DiskSpaceProbe {
            warn_threshold: 80,
            fail_threshold: 95,
        };

        assert_eq!(probe.name(), "disk-space");
        assert!(!probe.description().is_empty());
        assert!(!probe.citation().is_empty());

        // Test execution (may fail in CI without df)
        if let Ok(result) = probe.run() {
            assert_eq!(result.probe, "disk-space");
            assert!(result.duration_ms > 0);
        }
    }

    #[test]
    fn test_systemd_units_probe() {
        let probe = SystemdUnitsProbe;

        assert_eq!(probe.name(), "systemd-units");

        // Test execution (may fail in CI without systemctl)
        if let Ok(result) = probe.run() {
            assert_eq!(result.probe, "systemd-units");
            assert!(result.duration_ms >= 0);
        }
    }

    #[test]
    fn test_probe_status_serialization() {
        let status = ProbeStatus::Ok;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, r#""ok""#);

        let status = ProbeStatus::Warn;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, r#""warn""#);
    }
}
