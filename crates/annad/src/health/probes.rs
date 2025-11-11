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
