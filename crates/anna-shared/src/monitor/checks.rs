//! System monitoring checks.

use std::path::PathBuf;
use std::process::Command;

use super::types::{Issue, IssueType, MonitorResults, MonitorThresholds, Severity};

/// Run all monitoring checks
/// v0.0.990: Added security, hardware, and behavioral checks
pub fn run_checks(thresholds: &MonitorThresholds) -> MonitorResults {
    let start = std::time::Instant::now();
    let mut issues = Vec::new();

    // Basic system checks
    issues.extend(check_disk_space(thresholds));
    issues.extend(check_memory(thresholds));
    issues.extend(check_failed_services());
    issues.extend(check_journal_errors());
    issues.extend(check_updates(thresholds));

    // v0.0.990: Security checks
    issues.extend(check_ssh_security());
    issues.extend(check_firewall());
    issues.extend(check_suspicious_logins());
    issues.extend(check_open_ports());

    // v0.0.990: Hardware checks
    issues.extend(check_thermal());
    issues.extend(check_smart_health());

    // v0.0.990: Hardware and config change detection
    issues.extend(check_hardware_changes());
    issues.extend(check_config_changes());

    // v0.0.990: Learning-based checks
    issues.extend(check_learned_changes());

    // v0.0.990: Performance checks
    issues.extend(check_boot_time());

    MonitorResults {
        issues,
        checked_at: chrono::Utc::now().to_rfc3339(),
        duration_ms: start.elapsed().as_millis() as u64,
    }
}

/// Check disk space on all mounted filesystems
fn check_disk_space(thresholds: &MonitorThresholds) -> Vec<Issue> {
    let mut issues = Vec::new();

    let output = Command::new("df")
        .args(["--output=target,pcent", "-x", "tmpfs", "-x", "devtmpfs", "-x", "squashfs"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let mount = parts[0];
                let percent_str = parts[1].trim_end_matches('%');
                if let Ok(percent) = percent_str.parse::<u8>() {
                    if percent >= thresholds.disk_critical_percent {
                        issues.push(Issue {
                            issue_type: IssueType::DiskSpaceLow,
                            severity: Severity::Critical,
                            summary: format!("{} is {}% full", mount, percent),
                            details: format!(
                                "Filesystem {} has only {}% free space. This can cause system instability.",
                                mount,
                                100 - percent
                            ),
                            suggested_fix: Some(format!(
                                "Run: du -sh {}/* | sort -rh | head -10 to find large directories",
                                mount
                            )),
                            detected_at: chrono::Utc::now().to_rfc3339(),
                            notified: false,
                            acknowledged: false,
                        });
                    } else if percent >= thresholds.disk_warning_percent {
                        issues.push(Issue {
                            issue_type: IssueType::DiskSpaceLow,
                            severity: Severity::Warning,
                            summary: format!("{} is {}% full", mount, percent),
                            details: format!(
                                "Filesystem {} is getting full. Consider cleaning up.",
                                mount
                            ),
                            suggested_fix: Some("Consider running: paccache -rk2 && pacman -Sc".to_string()),
                            detected_at: chrono::Utc::now().to_rfc3339(),
                            notified: false,
                            acknowledged: false,
                        });
                    }
                }
            }
        }
    }

    issues
}

/// Check memory usage
fn check_memory(thresholds: &MonitorThresholds) -> Vec<Issue> {
    let mut issues = Vec::new();

    let output = Command::new("free").args(["-m"]).output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.starts_with("Mem:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    if let (Ok(total), Ok(used)) = (
                        parts[1].parse::<u64>(),
                        parts[2].parse::<u64>(),
                    ) {
                        let percent = (used * 100 / total) as u8;
                        if percent >= thresholds.memory_warning_percent {
                            issues.push(Issue {
                                issue_type: IssueType::MemoryHigh,
                                severity: Severity::Warning,
                                summary: format!("Memory {}% used ({}/{}MB)", percent, used, total),
                                details: "High memory usage detected. System may become slow.".to_string(),
                                suggested_fix: Some("Check: ps aux --sort=-%mem | head -10".to_string()),
                                detected_at: chrono::Utc::now().to_rfc3339(),
                                notified: false,
                                acknowledged: false,
                            });
                        }
                    }
                }
            }
        }
    }

    issues
}

/// Check for failed systemd services
fn check_failed_services() -> Vec<Issue> {
    let mut issues = Vec::new();

    let output = Command::new("systemctl")
        .args(["--failed", "--no-legend", "--plain"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if !parts.is_empty() {
                let service = parts[0];
                // Skip user services when running as root
                if !service.contains("user@") {
                    issues.push(Issue {
                        issue_type: IssueType::ServiceFailed,
                        severity: Severity::Warning,
                        summary: format!("Service {} failed", service),
                        details: format!("Systemd service {} is in failed state.", service),
                        suggested_fix: Some(format!(
                            "Check: journalctl -u {} -n 50 --no-pager",
                            service
                        )),
                        detected_at: chrono::Utc::now().to_rfc3339(),
                        notified: false,
                        acknowledged: false,
                    });
                }
            }
        }
    }

    issues
}

/// Check journal for recent errors
fn check_journal_errors() -> Vec<Issue> {
    let mut issues = Vec::new();

    // Check for critical errors in last hour
    let output = Command::new("journalctl")
        .args(["-p", "err", "--since", "1 hour ago", "-q", "--no-pager", "-n", "5"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let error_count = stdout.lines().count();

        if error_count > 3 {
            issues.push(Issue {
                issue_type: IssueType::JournalErrors,
                severity: Severity::Info,
                summary: format!("{} errors in journal (last hour)", error_count),
                details: "Multiple errors logged in the last hour.".to_string(),
                suggested_fix: Some("Check: journalctl -p err --since '1 hour ago'".to_string()),
                detected_at: chrono::Utc::now().to_rfc3339(),
                notified: false,
                acknowledged: false,
            });
        }
    }

    issues
}

/// Check for needed updates
fn check_updates(thresholds: &MonitorThresholds) -> Vec<Issue> {
    let mut issues = Vec::new();

    // Check when pacman was last synced
    let sync_db = PathBuf::from("/var/lib/pacman/sync");
    if let Ok(metadata) = std::fs::metadata(&sync_db) {
        if let Ok(modified) = metadata.modified() {
            let age = std::time::SystemTime::now()
                .duration_since(modified)
                .unwrap_or_default();

            let days = age.as_secs() / 86400;
            if days >= thresholds.update_warning_days as u64 {
                issues.push(Issue {
                    issue_type: IssueType::SecurityUpdates,
                    severity: if days > 14 { Severity::Warning } else { Severity::Info },
                    summary: format!("System not updated in {} days", days),
                    details: "Regular updates are important for security.".to_string(),
                    suggested_fix: Some("Run: pacman -Syu".to_string()),
                    detected_at: chrono::Utc::now().to_rfc3339(),
                    notified: false,
                    acknowledged: false,
                });
            }
        }
    }

    issues
}

// ========== v0.0.990: Security Checks ==========

/// Check SSH configuration for security issues
fn check_ssh_security() -> Vec<Issue> {
    let mut issues = Vec::new();
    let sshd_config = "/etc/ssh/sshd_config";

    if let Ok(content) = std::fs::read_to_string(sshd_config) {
        let content_lower = content.to_lowercase();

        // Check for root login enabled
        if content_lower.contains("permitrootlogin yes") {
            issues.push(Issue {
                issue_type: IssueType::RootLoginEnabled,
                severity: Severity::Warning,
                summary: "SSH root login is enabled".to_string(),
                details: "Allowing root login via SSH is a security risk.".to_string(),
                suggested_fix: Some("Set 'PermitRootLogin no' in /etc/ssh/sshd_config".to_string()),
                detected_at: chrono::Utc::now().to_rfc3339(),
                notified: false,
                acknowledged: false,
            });
        }

        // Check for password authentication (prefer keys)
        if content_lower.contains("passwordauthentication yes") {
            issues.push(Issue {
                issue_type: IssueType::SshSecurity,
                severity: Severity::Info,
                summary: "SSH password auth enabled".to_string(),
                details: "Key-based authentication is more secure than passwords.".to_string(),
                suggested_fix: Some("Consider: PasswordAuthentication no".to_string()),
                detected_at: chrono::Utc::now().to_rfc3339(),
                notified: false,
                acknowledged: false,
            });
        }

        // Check for weak ciphers (if explicitly set)
        if content_lower.contains("3des") || content_lower.contains("arcfour") {
            issues.push(Issue {
                issue_type: IssueType::SshSecurity,
                severity: Severity::Warning,
                summary: "Weak SSH ciphers configured".to_string(),
                details: "3DES and RC4 ciphers are considered weak.".to_string(),
                suggested_fix: Some("Remove weak ciphers from sshd_config".to_string()),
                detected_at: chrono::Utc::now().to_rfc3339(),
                notified: false,
                acknowledged: false,
            });
        }
    }

    issues
}

/// Check if firewall is active
fn check_firewall() -> Vec<Issue> {
    let mut issues = Vec::new();

    // Check for iptables rules
    let iptables = Command::new("iptables").args(["-L", "-n"]).output();
    let nftables = Command::new("nft").args(["list", "ruleset"]).output();

    let has_iptables = iptables
        .map(|o| {
            let out = String::from_utf8_lossy(&o.stdout);
            // More than just the default empty chains
            out.lines().count() > 8
        })
        .unwrap_or(false);

    let has_nftables = nftables
        .map(|o| {
            let out = String::from_utf8_lossy(&o.stdout);
            !out.trim().is_empty() && out.contains("chain")
        })
        .unwrap_or(false);

    // Check for firewalld or ufw
    let firewalld_active = Command::new("systemctl")
        .args(["is-active", "firewalld"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "active")
        .unwrap_or(false);

    let ufw_active = Command::new("ufw")
        .arg("status")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("active"))
        .unwrap_or(false);

    if !has_iptables && !has_nftables && !firewalld_active && !ufw_active {
        issues.push(Issue {
            issue_type: IssueType::FirewallInactive,
            severity: Severity::Warning,
            summary: "No firewall detected".to_string(),
            details: "No active firewall rules found. Your system may be exposed.".to_string(),
            suggested_fix: Some("Consider: ufw enable or systemctl start firewalld".to_string()),
            detected_at: chrono::Utc::now().to_rfc3339(),
            notified: false,
            acknowledged: false,
        });
    }

    issues
}

/// Check for suspicious login attempts
fn check_suspicious_logins() -> Vec<Issue> {
    let mut issues = Vec::new();

    // Check auth log for failed attempts
    let output = Command::new("journalctl")
        .args(["-u", "sshd", "--since", "24 hours ago", "-q", "--no-pager"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let failed_count = stdout.matches("Failed password").count()
            + stdout.matches("authentication failure").count()
            + stdout.matches("Invalid user").count();

        if failed_count > 20 {
            issues.push(Issue {
                issue_type: IssueType::SuspiciousLogin,
                severity: Severity::Warning,
                summary: format!("{} failed SSH logins (24h)", failed_count),
                details: "Many failed login attempts detected. Possible brute force attack.".to_string(),
                suggested_fix: Some("Consider: fail2ban or changing SSH port".to_string()),
                detected_at: chrono::Utc::now().to_rfc3339(),
                notified: false,
                acknowledged: false,
            });
        } else if failed_count > 5 {
            issues.push(Issue {
                issue_type: IssueType::SuspiciousLogin,
                severity: Severity::Info,
                summary: format!("{} failed SSH logins (24h)", failed_count),
                details: "Some failed login attempts detected.".to_string(),
                suggested_fix: Some("Check: journalctl -u sshd | grep Failed".to_string()),
                detected_at: chrono::Utc::now().to_rfc3339(),
                notified: false,
                acknowledged: false,
            });
        }
    }

    issues
}

/// Check for unexpected open ports
fn check_open_ports() -> Vec<Issue> {
    let mut issues = Vec::new();

    let output = Command::new("ss")
        .args(["-tlnp"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Check for common dangerous ports listening on all interfaces
        let dangerous_ports = [
            ("0.0.0.0:23", "Telnet (unencrypted)"),
            ("0.0.0.0:21", "FTP (unencrypted)"),
            ("0.0.0.0:3306", "MySQL (public)"),
            ("0.0.0.0:5432", "PostgreSQL (public)"),
            ("0.0.0.0:27017", "MongoDB (public)"),
            ("0.0.0.0:6379", "Redis (public)"),
            (":23", "Telnet (unencrypted)"),
            (":21", "FTP (unencrypted)"),
            ("*:3306", "MySQL (public)"),
            ("*:5432", "PostgreSQL (public)"),
        ];

        for (port_pattern, desc) in dangerous_ports {
            if stdout.contains(port_pattern) {
                issues.push(Issue {
                    issue_type: IssueType::OpenPort,
                    severity: Severity::Warning,
                    summary: format!("{} exposed", desc),
                    details: format!("Port {} is listening on all interfaces.", port_pattern),
                    suggested_fix: Some("Bind to 127.0.0.1 or use firewall".to_string()),
                    detected_at: chrono::Utc::now().to_rfc3339(),
                    notified: false,
                    acknowledged: false,
                });
            }
        }
    }

    issues
}

// ========== v0.0.990: Hardware Checks ==========

/// Check thermal/temperature status
fn check_thermal() -> Vec<Issue> {
    let mut issues = Vec::new();

    // Check for thermal throttling
    let output = Command::new("journalctl")
        .args(["-k", "--since", "1 hour ago", "-q", "--no-pager"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("thermal") && (stdout.contains("throttl") || stdout.contains("critical")) {
            issues.push(Issue {
                issue_type: IssueType::ThermalThrottling,
                severity: Severity::Warning,
                summary: "Thermal throttling detected".to_string(),
                details: "CPU/GPU is throttling due to high temperature.".to_string(),
                suggested_fix: Some("Check cooling: sensors, clean fans, improve airflow".to_string()),
                detected_at: chrono::Utc::now().to_rfc3339(),
                notified: false,
                acknowledged: false,
            });
        }
    }

    // Check current temps if sensors available
    if let Ok(output) = Command::new("sensors").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Look for high temperatures (crude check)
        for line in stdout.lines() {
            if line.contains("°C") || line.contains("°F") {
                // Extract temperature value
                if let Some(temp_str) = line.split_whitespace()
                    .find(|s| s.starts_with('+') && s.contains('°'))
                {
                    let temp_val: Option<f32> = temp_str
                        .trim_start_matches('+')
                        .split('°')
                        .next()
                        .and_then(|s| s.parse().ok());

                    if let Some(temp) = temp_val {
                        if temp > 90.0 {
                            issues.push(Issue {
                                issue_type: IssueType::ThermalThrottling,
                                severity: Severity::Critical,
                                summary: format!("High temperature: {}°C", temp),
                                details: "Component running very hot.".to_string(),
                                suggested_fix: Some("Improve cooling immediately".to_string()),
                                detected_at: chrono::Utc::now().to_rfc3339(),
                                notified: false,
                                acknowledged: false,
                            });
                            break;
                        }
                    }
                }
            }
        }
    }

    issues
}

/// Check SMART health for drives
fn check_smart_health() -> Vec<Issue> {
    let mut issues = Vec::new();

    // Get list of block devices
    let lsblk = Command::new("lsblk")
        .args(["-d", "-n", "-o", "NAME,TYPE"])
        .output();

    if let Ok(output) = lsblk {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 && parts[1] == "disk" {
                let device = format!("/dev/{}", parts[0]);

                // Try smartctl (may require root)
                if let Ok(smart) = Command::new("smartctl")
                    .args(["-H", &device])
                    .output()
                {
                    let smart_out = String::from_utf8_lossy(&smart.stdout);
                    if smart_out.contains("FAILED") || smart_out.contains("FAILING") {
                        issues.push(Issue {
                            issue_type: IssueType::HardwareError,
                            severity: Severity::Critical,
                            summary: format!("SMART failure: {}", parts[0]),
                            details: format!("Drive {} is reporting SMART failures. Backup immediately!", device),
                            suggested_fix: Some(format!("Run: smartctl -a {} and backup data", device)),
                            detected_at: chrono::Utc::now().to_rfc3339(),
                            notified: false,
                            acknowledged: false,
                        });
                    }
                }
            }
        }
    }

    issues
}

// ========== v0.0.990: Performance Checks ==========

/// Check boot time for anomalies
fn check_boot_time() -> Vec<Issue> {
    let mut issues = Vec::new();

    if let Ok(output) = Command::new("systemd-analyze").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Extract total boot time
        // Format: "Startup finished in 1.5s (firmware) + 2.3s (loader) + ... = 15.234s"
        if let Some(total) = stdout.split('=').last() {
            let total = total.trim();
            // Extract seconds
            if let Some(secs_str) = total.strip_suffix('s') {
                if let Ok(secs) = secs_str.trim().parse::<f32>() {
                    if secs > 120.0 {
                        issues.push(Issue {
                            issue_type: IssueType::SlowBoot,
                            severity: Severity::Warning,
                            summary: format!("Slow boot: {:.1}s", secs),
                            details: "Boot time exceeds 2 minutes.".to_string(),
                            suggested_fix: Some("Run: systemd-analyze blame | head -20".to_string()),
                            detected_at: chrono::Utc::now().to_rfc3339(),
                            notified: false,
                            acknowledged: false,
                        });
                    } else if secs > 60.0 {
                        issues.push(Issue {
                            issue_type: IssueType::SlowBoot,
                            severity: Severity::Info,
                            summary: format!("Boot time: {:.1}s", secs),
                            details: "Boot time is over 1 minute.".to_string(),
                            suggested_fix: Some("Check: systemd-analyze blame".to_string()),
                            detected_at: chrono::Utc::now().to_rfc3339(),
                            notified: false,
                            acknowledged: false,
                        });
                    }
                }
            }
        }
    }

    issues
}

// ========== v0.0.990: Change Detection ==========

/// Check for hardware changes since baseline
fn check_hardware_changes() -> Vec<Issue> {
    use super::baseline::SystemBaseline;

    let mut issues = Vec::new();

    // Load or create baseline
    let baseline = match SystemBaseline::load() {
        Some(b) => b,
        None => {
            // First run - capture baseline
            let baseline = SystemBaseline::capture();
            let _ = baseline.save();
            return issues; // No changes to report on first run
        }
    };

    let changes = baseline.compare();

    // Report new USB devices
    for device in &changes.usb_added {
        // Skip hubs and common internal devices
        if device.description.to_lowercase().contains("hub") {
            continue;
        }
        issues.push(Issue {
            issue_type: IssueType::HardwareAdded,
            severity: Severity::Info,
            summary: format!("New USB: {}", device.description),
            details: format!(
                "USB device added: {} ({}:{})",
                device.description, device.vendor_id, device.product_id
            ),
            suggested_fix: Some("Run: lsusb to see all USB devices".to_string()),
            detected_at: chrono::Utc::now().to_rfc3339(),
            notified: false,
            acknowledged: false,
        });
    }

    // Report removed USB devices (could indicate hardware failure or theft)
    for device in &changes.usb_removed {
        if device.description.to_lowercase().contains("hub") {
            continue;
        }
        issues.push(Issue {
            issue_type: IssueType::HardwareRemoved,
            severity: Severity::Warning,
            summary: format!("USB removed: {}", device.description),
            details: format!(
                "USB device removed: {} ({}:{})",
                device.description, device.vendor_id, device.product_id
            ),
            suggested_fix: Some("Check if device was intentionally removed".to_string()),
            detected_at: chrono::Utc::now().to_rfc3339(),
            notified: false,
            acknowledged: false,
        });
    }

    // Report new PCI devices (unusual - might indicate hardware change or driver issue)
    for device in &changes.pci_added {
        issues.push(Issue {
            issue_type: IssueType::HardwareAdded,
            severity: Severity::Info,
            summary: format!("New PCI: {} {}", device.vendor, device.class),
            details: format!(
                "PCI device appeared: {} {} (slot {})",
                device.vendor, device.device, device.slot
            ),
            suggested_fix: Some("Run: lspci to see all PCI devices".to_string()),
            detected_at: chrono::Utc::now().to_rfc3339(),
            notified: false,
            acknowledged: false,
        });
    }

    // Report removed PCI devices (concerning - hardware failure?)
    for device in &changes.pci_removed {
        issues.push(Issue {
            issue_type: IssueType::HardwareRemoved,
            severity: Severity::Warning,
            summary: format!("PCI missing: {} {}", device.vendor, device.class),
            details: format!(
                "PCI device disappeared: {} {} (slot {}). Possible hardware failure.",
                device.vendor, device.device, device.slot
            ),
            suggested_fix: Some("Check: dmesg | grep -i error".to_string()),
            detected_at: chrono::Utc::now().to_rfc3339(),
            notified: false,
            acknowledged: false,
        });
    }

    issues
}

/// Check for config file changes since baseline
fn check_config_changes() -> Vec<Issue> {
    use super::baseline::SystemBaseline;

    let mut issues = Vec::new();

    let baseline = match SystemBaseline::load() {
        Some(b) => b,
        None => return issues, // No baseline yet
    };

    let changes = baseline.compare();

    // Security-critical files that warrant warnings
    let critical_files = [
        "/etc/ssh/sshd_config",
        "/etc/sudoers",
        "/etc/passwd",
        "/etc/shadow",
        "/etc/group",
        "/etc/pam.d/system-auth",
    ];

    // Report changed config files
    for path in &changes.config_changed {
        let is_critical = critical_files.iter().any(|c| path.contains(c));
        let severity = if is_critical { Severity::Warning } else { Severity::Info };

        let summary = if path.contains("cron") {
            format!("Cron modified: {}", path.split('/').last().unwrap_or(path))
        } else {
            format!("Config changed: {}", path.split('/').last().unwrap_or(path))
        };

        issues.push(Issue {
            issue_type: IssueType::ConfigChanged,
            severity,
            summary,
            details: format!("File {} has been modified since baseline.", path),
            suggested_fix: Some(format!("Review changes: diff {} {}.bak", path, path)),
            detected_at: chrono::Utc::now().to_rfc3339(),
            notified: false,
            acknowledged: false,
        });
    }

    // Report new cron jobs (potential persistence mechanism)
    for path in &changes.config_added {
        if path.contains("cron") || path.contains("spool") {
            issues.push(Issue {
                issue_type: IssueType::CronAdded,
                severity: Severity::Warning,
                summary: format!("New cron: {}", path.split('/').last().unwrap_or(path)),
                details: format!("New cron job detected: {}", path),
                suggested_fix: Some(format!("Review: cat {}", path)),
                detected_at: chrono::Utc::now().to_rfc3339(),
                notified: false,
                acknowledged: false,
            });
        }
    }

    issues
}

/// Update the baseline with current system state
/// Call this after user acknowledges changes
pub fn update_baseline() -> anyhow::Result<()> {
    use super::baseline::SystemBaseline;
    let baseline = SystemBaseline::capture();
    baseline.save()
}

// ========== v0.0.990: Learning-Based Checks ==========

/// Check for changes detected by the learning system
fn check_learned_changes() -> Vec<Issue> {
    use super::learning::SystemLearning;

    let mut issues = Vec::new();

    // Load and update learning
    let mut learning = SystemLearning::load();
    let changes = learning.update();

    // Report package installations
    if !changes.packages_installed.is_empty() {
        let pkg_list = changes.packages_installed.join(", ");
        let count = changes.packages_installed.len();
        issues.push(Issue {
            issue_type: IssueType::PackagesInstalled,
            severity: Severity::Info,
            summary: format!("{} package(s) installed", count),
            details: format!("New packages: {}", pkg_list),
            suggested_fix: Some("Run: pacman -Qe to list explicitly installed packages".to_string()),
            detected_at: chrono::Utc::now().to_rfc3339(),
            notified: false,
            acknowledged: false,
        });
    }

    // Report boot time changes
    if let Some(diff) = changes.boot_time_change {
        let (summary, severity) = if diff > 0.0 {
            (format!("Boot {:.1}s slower than usual", diff), Severity::Warning)
        } else {
            (format!("Boot {:.1}s faster than usual", diff.abs()), Severity::Info)
        };

        issues.push(Issue {
            issue_type: IssueType::BootTimeChanged,
            severity,
            summary,
            details: "Boot time differs significantly from your system's average.".to_string(),
            suggested_fix: Some("Run: systemd-analyze blame | head -10".to_string()),
            detected_at: chrono::Utc::now().to_rfc3339(),
            notified: false,
            acknowledged: false,
        });
    }

    // Report unusual commands
    for cmd in &changes.unusual_commands {
        issues.push(Issue {
            issue_type: IssueType::UnusualCommand,
            severity: Severity::Warning,
            summary: "Unusual command detected".to_string(),
            details: format!("Command '{}' matches suspicious patterns.", cmd),
            suggested_fix: Some("Review shell history for unauthorized access".to_string()),
            detected_at: chrono::Utc::now().to_rfc3339(),
            notified: false,
            acknowledged: false,
        });
    }

    // Report performance anomalies
    for anomaly in &changes.performance_anomalies {
        issues.push(Issue {
            issue_type: IssueType::PerformanceAnomaly,
            severity: Severity::Warning,
            summary: anomaly.clone(),
            details: "Performance differs significantly from learned baseline.".to_string(),
            suggested_fix: Some("Check: htop or ps aux --sort=-%cpu | head -10".to_string()),
            detected_at: chrono::Utc::now().to_rfc3339(),
            notified: false,
            acknowledged: false,
        });
    }

    // Report performance trend
    let trend = learning.performance_trend();
    if trend == "degrading" {
        issues.push(Issue {
            issue_type: IssueType::PerformanceDegraded,
            severity: Severity::Info,
            summary: "Performance trend: degrading".to_string(),
            details: "System performance has been gradually decreasing.".to_string(),
            suggested_fix: Some("Consider: checking logs, clearing caches, reviewing recent changes".to_string()),
            detected_at: chrono::Utc::now().to_rfc3339(),
            notified: false,
            acknowledged: false,
        });
    }

    issues
}
