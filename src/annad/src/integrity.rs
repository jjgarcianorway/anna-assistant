// Anna v0.10.1 - Integrity Watchdog
//
// Passive 10-minute sweeps: verify binaries, units, directories, permissions, and
// capability dependencies. Write alerts to journal and alerts.json. No auto-fix.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use tracing::{info, warn};

/// Alert severity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Integrity alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityAlert {
    pub id: String,
    pub timestamp: i64,
    pub severity: AlertSeverity,
    pub component: String,
    pub message: String,
    pub fix_command: Option<String>,
    pub impact: String,
}

/// Alerts collection
#[derive(Debug, Serialize, Deserialize)]
struct AlertsFile {
    version: String,
    generated: i64,
    alerts: Vec<IntegrityAlert>,
}

const ALERTS_PATH: &str = "/var/lib/anna/alerts.json";

pub struct IntegrityWatchdog {
    alerts_path: String,
}

impl IntegrityWatchdog {
    pub fn new() -> Self {
        Self {
            alerts_path: ALERTS_PATH.to_string(),
        }
    }

    /// Run complete integrity sweep
    pub fn sweep(&self) -> Result<Vec<IntegrityAlert>> {
        info!("Running integrity sweep...");

        let mut alerts = Vec::new();

        // 1. Check binaries exist and are executable
        alerts.extend(self.check_binaries()?);

        // 2. Check systemd units
        alerts.extend(self.check_systemd_units()?);

        // 3. Check directories and permissions
        alerts.extend(self.check_directories()?);

        // 4. Check capability dependencies
        alerts.extend(self.check_capabilities()?);

        // 5. Check group membership
        alerts.extend(self.check_groups()?);

        // 6. Check disk space
        alerts.extend(self.check_disk_space()?);

        // Write alerts to file
        self.write_alerts(&alerts)?;

        // Log summary
        let critical = alerts.iter().filter(|a| matches!(a.severity, AlertSeverity::Critical)).count();
        let errors = alerts.iter().filter(|a| matches!(a.severity, AlertSeverity::Error)).count();
        let warnings = alerts.iter().filter(|a| matches!(a.severity, AlertSeverity::Warning)).count();

        if critical > 0 || errors > 0 {
            warn!(
                "Integrity sweep complete: {} critical, {} errors, {} warnings",
                critical, errors, warnings
            );
        } else if warnings > 0 {
            info!("Integrity sweep complete: {} warnings", warnings);
        } else {
            info!("Integrity sweep complete: all checks passed");
        }

        Ok(alerts)
    }

    /// Check specific domain (for event-driven checks)
    pub fn check_domain(&self, domain: &str) -> Result<Vec<IntegrityAlert>> {
        info!("Running integrity check for domain: {}", domain);

        let alerts = match domain {
            "packages" => self.check_capabilities()?,
            "config" => self.check_config_drift()?,
            "devices" => self.check_devices()?,
            "network" => self.check_network()?,
            "storage" => self.check_disk_space()?,
            "kernel" => self.check_binaries()?, // Binaries may change with kernel updates
            _ => {
                warn!("Unknown domain: {}", domain);
                Vec::new()
            }
        };

        // Write alerts to file
        self.write_alerts(&alerts)?;

        Ok(alerts)
    }

    fn check_config_drift(&self) -> Result<Vec<IntegrityAlert>> {
        // Placeholder for config drift detection (will implement with inotify)
        Ok(Vec::new())
    }

    fn check_devices(&self) -> Result<Vec<IntegrityAlert>> {
        // Placeholder for device topology checks (will implement with udev)
        Ok(Vec::new())
    }

    fn check_network(&self) -> Result<Vec<IntegrityAlert>> {
        // Placeholder for network state checks (will implement with netlink)
        Ok(Vec::new())
    }

    fn check_binaries(&self) -> Result<Vec<IntegrityAlert>> {
        let mut alerts = Vec::new();

        let binaries = vec![
            ("/usr/local/bin/annad", "annad"),
            ("/usr/local/bin/annactl", "annactl"),
        ];

        for (path, name) in binaries {
            if !Path::new(path).exists() {
                alerts.push(IntegrityAlert {
                    id: format!("missing_binary_{}", name),
                    timestamp: chrono::Utc::now().timestamp(),
                    severity: AlertSeverity::Critical,
                    component: format!("binary:{}", name),
                    message: format!("Binary '{}' not found at {}", name, path),
                    fix_command: Some("sudo ./scripts/install.sh".to_string()),
                    impact: format!("{} unavailable", name),
                });
            } else {
                // Check executable bit
                if let Ok(metadata) = fs::metadata(path) {
                    let perms = metadata.permissions();
                    if perms.mode() & 0o111 == 0 {
                        alerts.push(IntegrityAlert {
                            id: format!("not_executable_{}", name),
                            timestamp: chrono::Utc::now().timestamp(),
                            severity: AlertSeverity::Error,
                            component: format!("binary:{}", name),
                            message: format!("Binary '{}' not executable", name),
                            fix_command: Some(format!("sudo chmod +x {}", path)),
                            impact: format!("{} cannot be executed", name),
                        });
                    }
                }
            }
        }

        Ok(alerts)
    }

    fn check_systemd_units(&self) -> Result<Vec<IntegrityAlert>> {
        let mut alerts = Vec::new();

        let unit_path = "/etc/systemd/system/annad.service";
        if !Path::new(unit_path).exists() {
            alerts.push(IntegrityAlert {
                id: "missing_unit_annad".to_string(),
                timestamp: chrono::Utc::now().timestamp(),
                severity: AlertSeverity::Critical,
                component: "systemd:annad.service".to_string(),
                message: "Systemd unit annad.service not found".to_string(),
                fix_command: Some("sudo ./scripts/install.sh".to_string()),
                impact: "Anna daemon will not start on boot".to_string(),
            });
        }

        Ok(alerts)
    }

    fn check_directories(&self) -> Result<Vec<IntegrityAlert>> {
        let mut alerts = Vec::new();

        let dirs = vec![
            ("/var/lib/anna", "anna", "anna", 0o700, "State directory"),
            ("/var/log/anna", "anna", "anna", 0o750, "Log directory"),
            ("/run/anna", "anna", "anna", 0o750, "Runtime directory"),
        ];

        for (path, expected_user, expected_group, expected_mode, description) in dirs {
            if !Path::new(path).exists() {
                alerts.push(IntegrityAlert {
                    id: format!("missing_dir_{}", path.replace('/', "_")),
                    timestamp: chrono::Utc::now().timestamp(),
                    severity: AlertSeverity::Error,
                    component: format!("directory:{}", path),
                    message: format!("{} '{}' does not exist", description, path),
                    fix_command: Some(format!(
                        "sudo mkdir -p {} && sudo chown {}:{} {} && sudo chmod {:o} {}",
                        path, expected_user, expected_group, path, expected_mode, path
                    )),
                    impact: "Anna may fail to start or store data".to_string(),
                });
            } else {
                // Check permissions
                if let Ok(metadata) = fs::metadata(path) {
                    let perms = metadata.permissions();
                    let mode = perms.mode() & 0o777;

                    if mode != expected_mode {
                        alerts.push(IntegrityAlert {
                            id: format!("wrong_perms_{}", path.replace('/', "_")),
                            timestamp: chrono::Utc::now().timestamp(),
                            severity: AlertSeverity::Warning,
                            component: format!("directory:{}", path),
                            message: format!(
                                "{} has permissions {:o}, expected {:o}",
                                path, mode, expected_mode
                            ),
                            fix_command: Some(format!("sudo chmod {:o} {}", expected_mode, path)),
                            impact: "Potential security risk or access denied".to_string(),
                        });
                    }
                }
            }
        }

        Ok(alerts)
    }

    fn check_capabilities(&self) -> Result<Vec<IntegrityAlert>> {
        let mut alerts = Vec::new();

        // Load capability manager and check for degraded modules
        if let Ok(cap_mgr) = crate::capabilities::CapabilityManager::new() {
            for check in cap_mgr.degraded_modules() {
                if check.required {
                    // Required module degraded - critical
                    alerts.push(IntegrityAlert {
                        id: format!("degraded_required_{}", check.module_name),
                        timestamp: chrono::Utc::now().timestamp(),
                        severity: AlertSeverity::Critical,
                        component: format!("module:{}", check.module_name),
                        message: format!(
                            "Required module '{}' degraded: {}",
                            check.module_name,
                            check.reason.as_ref().unwrap_or(&"Unknown".to_string())
                        ),
                        fix_command: check.action.clone(),
                        impact: check.impact.unwrap_or_else(|| "Core functionality impaired".to_string()),
                    });
                } else {
                    // Optional module degraded - warning
                    alerts.push(IntegrityAlert {
                        id: format!("degraded_optional_{}", check.module_name),
                        timestamp: chrono::Utc::now().timestamp(),
                        severity: AlertSeverity::Warning,
                        component: format!("module:{}", check.module_name),
                        message: format!(
                            "Module '{}' degraded: {}",
                            check.module_name,
                            check.reason.as_ref().unwrap_or(&"Unknown".to_string())
                        ),
                        fix_command: check.action.clone(),
                        impact: check.impact.unwrap_or_else(|| "Feature unavailable".to_string()),
                    });
                }
            }
        }

        Ok(alerts)
    }

    fn check_groups(&self) -> Result<Vec<IntegrityAlert>> {
        let mut alerts = Vec::new();

        // Check if anna user exists
        let output = std::process::Command::new("id")
            .arg("anna")
            .output();

        if output.is_err() || !output.unwrap().status.success() {
            alerts.push(IntegrityAlert {
                id: "missing_user_anna".to_string(),
                timestamp: chrono::Utc::now().timestamp(),
                severity: AlertSeverity::Critical,
                component: "system:user".to_string(),
                message: "User 'anna' does not exist".to_string(),
                fix_command: Some("sudo useradd -r -s /usr/bin/nologin anna".to_string()),
                impact: "Anna daemon cannot run with correct privileges".to_string(),
            });
        }

        Ok(alerts)
    }

    fn check_disk_space(&self) -> Result<Vec<IntegrityAlert>> {
        let mut alerts = Vec::new();

        // Check free space on /var
        if let Ok(output) = std::process::Command::new("df")
            .args(&["--output=avail", "/var"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(line) = stdout.lines().nth(1) {
                if let Ok(available_kb) = line.trim().parse::<u64>() {
                    let available_mb = available_kb / 1024;

                    if available_mb < 200 {
                        alerts.push(IntegrityAlert {
                            id: "low_disk_var".to_string(),
                            timestamp: chrono::Utc::now().timestamp(),
                            severity: AlertSeverity::Warning,
                            component: "system:disk".to_string(),
                            message: format!(
                                "Low disk space on /var: {}MB available (threshold: 200MB)",
                                available_mb
                            ),
                            fix_command: Some("sudo journalctl --vacuum-time=7d".to_string()),
                            impact: "Anna may fail to write logs or telemetry data".to_string(),
                        });
                    }
                }
            }
        }

        Ok(alerts)
    }

    fn write_alerts(&self, alerts: &[IntegrityAlert]) -> Result<()> {
        let alerts_file = AlertsFile {
            version: "0.10.1".to_string(),
            generated: chrono::Utc::now().timestamp(),
            alerts: alerts.to_vec(),
        };

        let json = serde_json::to_string_pretty(&alerts_file)
            .context("Failed to serialize alerts")?;

        std::fs::write(&self.alerts_path, json)
            .context("Failed to write alerts.json")?;

        Ok(())
    }

    /// Read current alerts from file
    pub fn read_alerts(&self) -> Result<Vec<IntegrityAlert>> {
        if !Path::new(&self.alerts_path).exists() {
            return Ok(Vec::new());
        }

        let content = std::fs::read_to_string(&self.alerts_path)
            .context("Failed to read alerts.json")?;

        let alerts_file: AlertsFile = serde_json::from_str(&content)
            .context("Failed to parse alerts.json")?;

        Ok(alerts_file.alerts)
    }
}
