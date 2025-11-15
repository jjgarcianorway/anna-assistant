//! Systemd service management utilities
//!
//! Provides idempotent operations for managing the annad systemd service:
//! - Detection: Check if service exists, is enabled, is active
//! - Repair: Install/reinstall service files, enable, start
//! - Status: Get detailed service status and failure reasons

use anyhow::{Context, Result};
use std::process::Command;

const SERVICE_NAME: &str = "annad.service";
const SERVICE_FILE_SOURCE: &str = "annad.service"; // In repo root
const SERVICE_FILE_DEST: &str = "/etc/systemd/system/annad.service";

/// Status of the annad systemd service
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceStatus {
    /// Service unit file exists
    pub exists: bool,
    /// Service is enabled (will start on boot)
    pub enabled: bool,
    /// Service is currently active/running
    pub active: bool,
    /// Service has failed
    pub failed: bool,
    /// Human-readable status description
    pub status_text: String,
}

impl ServiceStatus {
    /// Check if service needs repair (missing, disabled, or failed)
    pub fn needs_repair(&self) -> bool {
        !self.exists || !self.enabled || self.failed || !self.active
    }

    /// Get a human-readable summary
    pub fn summary(&self) -> String {
        if !self.exists {
            "Service not installed".to_string()
        } else if self.failed {
            "Service failed".to_string()
        } else if !self.enabled {
            "Service disabled (won't start on boot)".to_string()
        } else if !self.active {
            "Service not running".to_string()
        } else {
            "Service healthy".to_string()
        }
    }
}

/// Get current status of the annad service
pub fn get_service_status() -> Result<ServiceStatus> {
    let exists = check_service_exists()?;

    if !exists {
        return Ok(ServiceStatus {
            exists: false,
            enabled: false,
            active: false,
            failed: false,
            status_text: "Service not installed".to_string(),
        });
    }

    let enabled = check_service_enabled()?;
    let (active, failed) = check_service_active()?;
    let status_text = get_service_status_text()?;

    Ok(ServiceStatus {
        exists,
        enabled,
        active,
        failed,
        status_text,
    })
}

/// Check if service unit file exists
fn check_service_exists() -> Result<bool> {
    let output = Command::new("systemctl")
        .args(["list-unit-files", SERVICE_NAME])
        .output()
        .context("Failed to run systemctl list-unit-files")?;

    // If the service is listed, it exists
    Ok(output.status.success()
        && String::from_utf8_lossy(&output.stdout).contains(SERVICE_NAME))
}

/// Check if service is enabled (will start on boot)
fn check_service_enabled() -> Result<bool> {
    let output = Command::new("systemctl")
        .args(["is-enabled", SERVICE_NAME])
        .output()
        .context("Failed to run systemctl is-enabled")?;

    // Exit code 0 = enabled, anything else = not enabled
    Ok(output.status.success())
}

/// Check if service is active and if it has failed
/// Returns (active, failed)
fn check_service_active() -> Result<(bool, bool)> {
    let output = Command::new("systemctl")
        .args(["is-active", SERVICE_NAME])
        .output()
        .context("Failed to run systemctl is-active")?;

    let status_str = String::from_utf8_lossy(&output.stdout).trim().to_string();

    let active = output.status.success() && status_str == "active";
    let failed = status_str == "failed";

    Ok((active, failed))
}

/// Get detailed service status text
fn get_service_status_text() -> Result<String> {
    let output = Command::new("systemctl")
        .args(["status", SERVICE_NAME, "--no-pager", "--lines=0"])
        .output()
        .context("Failed to run systemctl status")?;

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Install or reinstall the annad service file
///
/// This is idempotent - safe to run multiple times
pub fn install_service_file() -> Result<()> {
    // Check if source file exists
    if !std::path::Path::new(SERVICE_FILE_SOURCE).exists() {
        anyhow::bail!(
            "Service file not found: {}. Are you running from the anna-assistant directory?",
            SERVICE_FILE_SOURCE
        );
    }

    // Copy service file to systemd directory
    let output = Command::new("sudo")
        .args(["cp", SERVICE_FILE_SOURCE, SERVICE_FILE_DEST])
        .output()
        .context("Failed to copy service file (need sudo)")?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to install service file: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Reload systemd to pick up changes
    reload_systemd()?;

    Ok(())
}

/// Reload systemd daemon configuration
///
/// This is idempotent - safe to run multiple times
pub fn reload_systemd() -> Result<()> {
    let output = Command::new("sudo")
        .args(["systemctl", "daemon-reload"])
        .output()
        .context("Failed to reload systemd daemon")?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to reload systemd: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
}

/// Enable the service (will start on boot)
///
/// This is idempotent - safe to run multiple times
pub fn enable_service() -> Result<()> {
    let output = Command::new("sudo")
        .args(["systemctl", "enable", SERVICE_NAME])
        .output()
        .context("Failed to enable service")?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to enable service: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
}

/// Start the service
///
/// This is idempotent - safe to run multiple times
pub fn start_service() -> Result<()> {
    let output = Command::new("sudo")
        .args(["systemctl", "start", SERVICE_NAME])
        .output()
        .context("Failed to start service")?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to start service: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
}

/// Restart the service (clean restart)
///
/// This is idempotent - safe to run multiple times
pub fn restart_service() -> Result<()> {
    let output = Command::new("sudo")
        .args(["systemctl", "restart", SERVICE_NAME])
        .output()
        .context("Failed to restart service")?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to restart service: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
}

/// Get failure reason from journalctl (last 50 lines)
pub fn get_failure_reason() -> Result<String> {
    let output = Command::new("sudo")
        .args(["journalctl", "-u", SERVICE_NAME, "-n", "50", "--no-pager"])
        .output()
        .context("Failed to get service logs")?;

    if !output.status.success() {
        return Ok("Unable to retrieve logs".to_string());
    }

    let logs = String::from_utf8_lossy(&output.stdout);

    // Try to find error messages in the logs
    let error_lines: Vec<&str> = logs
        .lines()
        .filter(|line| {
            line.contains("error")
                || line.contains("Error")
                || line.contains("ERROR")
                || line.contains("failed")
                || line.contains("Failed")
        })
        .take(5)
        .collect();

    if error_lines.is_empty() {
        Ok("No obvious errors in recent logs".to_string())
    } else {
        Ok(error_lines.join("\n"))
    }
}

/// Comprehensive repair: install, enable, and start service
///
/// This handles all common failure modes:
/// - Missing service file
/// - Service disabled
/// - Service failed
/// - Service not running
///
/// Returns a human-readable report of what was done
pub fn repair_service() -> Result<String> {
    let mut report = Vec::new();
    let status = get_service_status()?;

    // Step 1: Install/reinstall service file if missing
    if !status.exists {
        report.push("Installing annad.service to /etc/systemd/system/...".to_string());
        install_service_file()
            .context("Failed to install service file")?;
        report.push("✓ Service file installed".to_string());
    } else {
        report.push("✓ Service file already exists".to_string());
    }

    // Step 2: Enable service if not enabled
    if !status.enabled {
        report.push("Enabling annad.service (will start on boot)...".to_string());
        enable_service()
            .context("Failed to enable service")?;
        report.push("✓ Service enabled".to_string());
    } else {
        report.push("✓ Service already enabled".to_string());
    }

    // Step 3: Start or restart service
    if status.failed {
        report.push("Service has failed, attempting restart...".to_string());

        // Get failure reason before restart
        if let Ok(reason) = get_failure_reason() {
            if !reason.is_empty() && reason != "No obvious errors in recent logs" {
                report.push(format!("Failure reason:\n{}", reason));
            }
        }

        restart_service()
            .context("Failed to restart service")?;
        report.push("✓ Service restarted".to_string());
    } else if !status.active {
        report.push("Starting annad.service...".to_string());
        start_service()
            .context("Failed to start service")?;
        report.push("✓ Service started".to_string());
    } else {
        report.push("✓ Service already running".to_string());
    }

    // Step 4: Verify final status
    let final_status = get_service_status()?;
    if final_status.active && final_status.enabled {
        report.push("\n✓ Anna daemon is now healthy and will start on boot".to_string());
    } else {
        report.push("\n⚠ Service may need manual attention. Check: sudo systemctl status annad".to_string());
    }

    Ok(report.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_status_needs_repair() {
        let healthy = ServiceStatus {
            exists: true,
            enabled: true,
            active: true,
            failed: false,
            status_text: "active".to_string(),
        };
        assert!(!healthy.needs_repair());

        let missing = ServiceStatus {
            exists: false,
            enabled: false,
            active: false,
            failed: false,
            status_text: "not found".to_string(),
        };
        assert!(missing.needs_repair());

        let disabled = ServiceStatus {
            exists: true,
            enabled: false,
            active: true,
            failed: false,
            status_text: "disabled".to_string(),
        };
        assert!(disabled.needs_repair());
    }
}
