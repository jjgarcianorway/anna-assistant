//! Power Diagnostics - TLP Status Detection (6.13.0)
//!
//! Detects power management issues, starting with TLP not being enabled.
//! This is the first "Anna reads logs + checks services + consults wiki" module.

use anyhow::Result;
use std::process::Command;

/// TLP service status on this system
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TlpStatus {
    /// TLP package is installed
    pub installed: bool,
    /// tlp.service unit file exists
    pub service_exists: bool,
    /// tlp.service is enabled (will start on boot)
    pub enabled: bool,
    /// tlp.service is currently running
    pub active: bool,
    /// Recent logs contain TLP warning about not being enabled
    pub warned_in_logs: bool,
}

/// Detect TLP status from systemd and logs
///
/// This function:
/// 1. Checks if TLP is installed (command exists)
/// 2. Queries systemd for service status
/// 3. Checks logs for the classic TLP warning
///
/// Returns comprehensive TLP status for decision making.
pub async fn detect_tlp_status() -> Result<TlpStatus> {
    // Check if TLP command exists (indicates installation)
    let installed = Command::new("which")
        .arg("tlp")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    // If not installed, return early
    if !installed {
        return Ok(TlpStatus {
            installed: false,
            service_exists: false,
            enabled: false,
            active: false,
            warned_in_logs: false,
        });
    }

    // Check systemd service status
    let service_status = Command::new("systemctl")
        .args(&["status", "tlp.service"])
        .output();

    let service_exists = service_status.is_ok();

    // Check if enabled
    let enabled = Command::new("systemctl")
        .args(&["is-enabled", "tlp.service"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    // Check if active
    let active = Command::new("systemctl")
        .args(&["is-active", "tlp.service"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    // Check logs for TLP warning
    // Classic warning: "TLP's power saving will not apply on boot because tlp.service is not enabled"
    let warned_in_logs = check_logs_for_tlp_warning().await;

    Ok(TlpStatus {
        installed,
        service_exists,
        enabled,
        active,
        warned_in_logs,
    })
}

/// Check recent logs for TLP "not enabled" warning
///
/// Looks for the classic TLP warning message in recent system logs.
/// This is the warning users actually see that prompts them to ask Anna for help.
async fn check_logs_for_tlp_warning() -> bool {
    // Try journalctl for recent TLP messages
    let journal_check = Command::new("journalctl")
        .args(&[
            "-b",           // current boot only
            "-n", "500",    // last 500 lines
            "--no-pager",   // don't paginate
        ])
        .output();

    if let Ok(output) = journal_check {
        let logs = String::from_utf8_lossy(&output.stdout);

        // Look for the classic TLP warning patterns
        let warning_patterns = [
            "tlp.service is not enabled",
            "TLP's power saving will not apply on boot",
            "TLP systemd service is not enabled",
        ];

        for pattern in &warning_patterns {
            if logs.to_lowercase().contains(&pattern.to_lowercase()) {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tlp_status_default() {
        let status = TlpStatus {
            installed: false,
            service_exists: false,
            enabled: false,
            active: false,
            warned_in_logs: false,
        };

        assert!(!status.installed);
        assert!(!status.enabled);
    }

    #[test]
    fn test_tlp_status_needs_enabling() {
        let status = TlpStatus {
            installed: true,
            service_exists: true,
            enabled: false,
            active: false,
            warned_in_logs: true,
        };

        // This is the "TLP installed but not enabled" case we want to fix
        assert!(status.installed);
        assert!(status.service_exists);
        assert!(!status.enabled);
        assert!(status.warned_in_logs);
    }

    #[test]
    fn test_tlp_status_healthy() {
        let status = TlpStatus {
            installed: true,
            service_exists: true,
            enabled: true,
            active: true,
            warned_in_logs: false,
        };

        // This is the healthy case - no action needed
        assert!(status.enabled);
        assert!(status.active);
        assert!(!status.warned_in_logs);
    }

    // Note: Can't test actual detection without mocking systemctl
    // Integration tests would run on real system with known state
}
