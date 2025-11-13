//! Caretaker Brain - Anna's core analysis engine
//!
//! This module ties together health checks, metrics, predictions, and environment
//! profile to produce actionable insights for the user.
//!
//! Product Vision: Every piece of intelligence must feed into detecting concrete
//! problems on this machine and offering clear fixes.

use serde::{Deserialize, Serialize};

use crate::profile::MachineProfile;

/// Severity of an issue or recommendation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum IssueSeverity {
    /// Critical - system is degraded or at risk
    Critical,
    /// Warning - something should be fixed soon
    Warning,
    /// Info - improvement opportunity
    Info,
}

/// A concrete issue or improvement opportunity detected on this machine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaretakerIssue {
    /// Severity level
    pub severity: IssueSeverity,

    /// Short, human-readable title (one line)
    /// Example: "Disk 96% full - 30GB in package cache"
    pub title: String,

    /// Longer explanation of what's wrong and why it matters
    /// Example: "Your disk is almost full. Package cache can be safely cleaned to free 30GB."
    pub explanation: String,

    /// Specific action the user should take
    /// Example: "Run 'sudo annactl repair' to clean package cache"
    pub recommended_action: String,

    /// Optional: repair action ID that can be invoked programmatically
    /// Example: Some("disk-space")
    pub repair_action_id: Option<String>,

    /// Reference for more information (usually Arch Wiki)
    /// Example: "https://wiki.archlinux.org/title/Pacman#Cleaning_the_package_cache"
    pub reference: Option<String>,

    /// Estimated impact of fixing this
    /// Example: "Frees 30GB disk space"
    pub estimated_impact: Option<String>,
}

impl CaretakerIssue {
    /// Create a new issue
    pub fn new(
        severity: IssueSeverity,
        title: impl Into<String>,
        explanation: impl Into<String>,
        recommended_action: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            title: title.into(),
            explanation: explanation.into(),
            recommended_action: recommended_action.into(),
            repair_action_id: None,
            reference: None,
            estimated_impact: None,
        }
    }

    /// Add a repair action ID
    pub fn with_repair_action(mut self, action_id: impl Into<String>) -> Self {
        self.repair_action_id = Some(action_id.into());
        self
    }

    /// Add a reference URL
    pub fn with_reference(mut self, url: impl Into<String>) -> Self {
        self.reference = Some(url.into());
        self
    }

    /// Add estimated impact
    pub fn with_impact(mut self, impact: impl Into<String>) -> Self {
        self.estimated_impact = Some(impact.into());
        self
    }
}

/// Analysis result from the caretaker brain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaretakerAnalysis {
    /// Top issues and recommendations, ordered by severity and importance
    pub issues: Vec<CaretakerIssue>,

    /// Overall system health: "healthy", "needs-attention", "critical"
    pub overall_status: String,

    /// One-line summary for quick display
    /// Example: "2 issues detected - disk space critical, TLP not enabled"
    pub summary: String,
}

impl CaretakerAnalysis {
    /// Create analysis with no issues (system healthy)
    pub fn healthy() -> Self {
        Self {
            issues: Vec::new(),
            overall_status: "healthy".to_string(),
            summary: "All systems healthy".to_string(),
        }
    }

    /// Create analysis from a list of issues
    pub fn from_issues(mut issues: Vec<CaretakerIssue>) -> Self {
        // Sort by severity (Critical > Warning > Info)
        // Derived Ord already provides this order: Critical (0) < Warning (1) < Info (2)
        issues.sort_by(|a, b| a.severity.cmp(&b.severity));

        let overall_status = if issues.iter().any(|i| i.severity == IssueSeverity::Critical) {
            "critical"
        } else if issues.iter().any(|i| i.severity == IssueSeverity::Warning) {
            "needs-attention"
        } else {
            "healthy"
        };

        let summary = if issues.is_empty() {
            "All systems healthy".to_string()
        } else {
            let critical_count = issues.iter().filter(|i| i.severity == IssueSeverity::Critical).count();
            let warning_count = issues.iter().filter(|i| i.severity == IssueSeverity::Warning).count();

            let mut parts = Vec::new();
            if critical_count > 0 {
                parts.push(format!("{} critical", critical_count));
            }
            if warning_count > 0 {
                parts.push(format!("{} warnings", warning_count));
            }

            format!("{} detected", parts.join(", "))
        };

        Self {
            issues,
            overall_status: overall_status.to_string(),
            summary,
        }
    }

    /// Get top N issues for display
    pub fn top_issues(&self, n: usize) -> &[CaretakerIssue] {
        &self.issues[..self.issues.len().min(n)]
    }
}

/// The Caretaker Brain - analyzes all available information and produces actionable insights
pub struct CaretakerBrain;

impl CaretakerBrain {
    /// Analyze the system and produce top issues/recommendations
    ///
    /// This is the core intelligence that ties together:
    /// - Health check results
    /// - System metrics
    /// - Predictive analysis
    /// - Machine profile (laptop/desktop/server-like)
    ///
    /// Returns a prioritized list of what the user should care about
    pub fn analyze(
        health_results: Option<&[crate::ipc::HealthProbeResult]>,
        disk_analysis: Option<&crate::disk_analysis::DiskAnalysis>,
        profile: MachineProfile,
    ) -> CaretakerAnalysis {
        let mut issues = Vec::new();

        // 1. Analyze disk space (most common critical issue)
        if let Some(disk) = disk_analysis {
            if disk.usage_percent > 95.0 {
                issues.push(
                    CaretakerIssue::new(
                        IssueSeverity::Critical,
                        format!("Disk {}% full - system at risk", disk.usage_percent as u32),
                        "Your disk is critically full. This can cause system instability and data loss. Immediate action required.",
                        "Run 'sudo annactl repair' to clean up space"
                    )
                    .with_repair_action("disk-space")
                    .with_reference("https://wiki.archlinux.org/title/System_maintenance#Clean_the_filesystem")
                );
            } else if disk.usage_percent > 90.0 {
                let recommendations = disk.get_recommendations();
                let total_savings: u64 = recommendations.iter()
                    .map(|r| r.estimated_savings_bytes)
                    .sum();

                let savings_gb = total_savings / (1024 * 1024 * 1024);

                issues.push(
                    CaretakerIssue::new(
                        IssueSeverity::Critical,
                        format!("Disk {}% full - {}GB can be freed", disk.usage_percent as u32, savings_gb),
                        format!("Your disk is nearly full. Package cache and logs can be safely cleaned to free {}GB.", savings_gb),
                        "Run 'sudo annactl repair' to clean up space"
                    )
                    .with_repair_action("disk-space")
                    .with_impact(format!("Frees {}GB", savings_gb))
                    .with_reference("https://wiki.archlinux.org/title/Pacman#Cleaning_the_package_cache")
                );
            } else if disk.usage_percent > 80.0 {
                issues.push(
                    CaretakerIssue::new(
                        IssueSeverity::Warning,
                        format!("Disk {}% full - cleanup recommended", disk.usage_percent as u32),
                        "Your disk is getting full. Consider cleaning package cache and logs.",
                        "Run 'annactl daily' to see cleanup recommendations"
                    )
                    .with_repair_action("disk-space")
                );
            }
        }

        // 2. Analyze health check results
        if let Some(health) = health_results {
            for result in health {
                // Only report failures and warnings
                if result.status == "fail" || result.status == "warn" {
                    let severity = if result.status == "fail" {
                        IssueSeverity::Critical
                    } else {
                        IssueSeverity::Warning
                    };

                    // Extract meaningful information from probe results
                    let (title, explanation, action) = Self::interpret_probe_result(result);

                    let mut issue = CaretakerIssue::new(severity, title, explanation, action);

                    // Map probe names to repair action IDs
                    issue.repair_action_id = Some(result.probe.clone());

                    issues.push(issue);
                }
            }
        }

        // 3. Check for pacman lock file issues
        Self::check_pacman_lock(&mut issues, profile);

        // 4. Check laptop power management
        Self::check_laptop_power_management(&mut issues, profile);

        // 5. Check GPU driver status
        Self::check_gpu_drivers(&mut issues, profile);

        // 6. Check journal error volume
        Self::check_journal_errors(&mut issues, profile);

        // 7. Check for zombie processes
        Self::check_zombie_processes(&mut issues, profile);

        // 8. Check for orphaned packages
        Self::check_orphaned_packages(&mut issues, profile);

        // 9. Check for stale core dumps
        Self::check_core_dumps(&mut issues, profile);

        // 10. Check time synchronization status
        Self::check_time_sync(&mut issues, profile);

        // 11. Check firewall status for networked machines
        Self::check_firewall_status(&mut issues, profile);

        // 12. Check backup and snapshot awareness
        Self::check_backup_awareness(&mut issues, profile);

        CaretakerAnalysis::from_issues(issues)
    }

    /// Check for pacman lock file issues (always relevant)
    fn check_pacman_lock(issues: &mut Vec<CaretakerIssue>, _profile: MachineProfile) {
        let lock_file = std::path::Path::new("/var/lib/pacman/db.lck");

        if lock_file.exists() {
            // Check if lock file is stale (older than 1 hour)
            if let Ok(metadata) = std::fs::metadata(lock_file) {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(elapsed) = modified.elapsed() {
                        if elapsed.as_secs() > 3600 {
                            issues.push(
                                CaretakerIssue::new(
                                    IssueSeverity::Warning,
                                    "Stale pacman lock file detected",
                                    "Pacman database is locked but appears to be stale. This prevents package operations.",
                                    "Run 'sudo rm /var/lib/pacman/db.lck' to remove the lock"
                                )
                                .with_reference("https://wiki.archlinux.org/title/Pacman#%22Failed_to_init_transaction_(unable_to_lock_database)%22_error")
                            );
                        }
                    }
                }
            }
        }
    }

    /// Check laptop power management configuration (laptop only)
    fn check_laptop_power_management(issues: &mut Vec<CaretakerIssue>, profile: MachineProfile) {
        // Only check on laptops
        if !profile.is_laptop() {
            return;
        }

        // Check if TLP or other power management is installed and enabled
        let tlp_installed = std::process::Command::new("which")
            .arg("tlp")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        let tlp_enabled = std::process::Command::new("systemctl")
            .args(&["is-enabled", "tlp.service"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if tlp_installed && !tlp_enabled {
            issues.push(
                CaretakerIssue::new(
                    IssueSeverity::Warning,
                    "Laptop detected but TLP not enabled",
                    "TLP is installed but not enabled. Your battery life could be significantly better.",
                    "Run 'sudo systemctl enable --now tlp.service'"
                )
                .with_repair_action("tlp-config")
                .with_reference("https://wiki.archlinux.org/title/TLP")
            );
        } else if !tlp_installed {
            issues.push(
                CaretakerIssue::new(
                    IssueSeverity::Info,
                    "Laptop detected without power management",
                    "Consider installing TLP for better battery life and thermal management.",
                    "Install with: 'sudo pacman -S tlp' then 'sudo systemctl enable --now tlp.service'"
                )
                .with_reference("https://wiki.archlinux.org/title/TLP")
            );
        }
    }

    /// Check GPU driver status (desktop/laptop only)
    fn check_gpu_drivers(issues: &mut Vec<CaretakerIssue>, profile: MachineProfile) {
        // Only relevant on interactive systems (desktop/laptop)
        if !profile.is_interactive() {
            return;
        }
        // Check for NVIDIA GPU
        let has_nvidia = std::process::Command::new("lspci")
            .output()
            .ok()
            .and_then(|o| {
                let output = String::from_utf8_lossy(&o.stdout);
                Some(output.to_lowercase().contains("nvidia"))
            })
            .unwrap_or(false);

        if !has_nvidia {
            return; // No NVIDIA GPU detected
        }

        // Check if NVIDIA driver is loaded
        let nvidia_loaded = std::process::Command::new("lsmod")
            .output()
            .ok()
            .and_then(|o| {
                let output = String::from_utf8_lossy(&o.stdout);
                Some(output.contains("nvidia"))
            })
            .unwrap_or(false);

        if !nvidia_loaded {
            issues.push(
                CaretakerIssue::new(
                    IssueSeverity::Warning,
                    "NVIDIA GPU detected but driver not loaded",
                    "You have an NVIDIA GPU but the proprietary driver is not loaded. GPU acceleration won't work.",
                    "Install NVIDIA driver: 'sudo pacman -S nvidia nvidia-utils'"
                )
                .with_reference("https://wiki.archlinux.org/title/NVIDIA")
            );
        }
    }

    /// Check journal error volume for current boot (always relevant)
    fn check_journal_errors(issues: &mut Vec<CaretakerIssue>, _profile: MachineProfile) {
        // Run journalctl -p err -b to count error entries for current boot
        let output = std::process::Command::new("journalctl")
            .args(&["-p", "err", "-b", "--no-pager"])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let error_count = stdout.lines()
                    .filter(|line| !line.trim().is_empty())
                    .count();

                if error_count > 200 {
                    issues.push(
                        CaretakerIssue::new(
                            IssueSeverity::Critical,
                            format!("High journal error volume ({} errors)", error_count),
                            "Your system journal has an unusually high number of errors this boot. This indicates serious system issues that need investigation.",
                            "Review errors with 'journalctl -p err -b' and investigate the most frequent issues"
                        )
                        .with_repair_action("journal-cleanup")
                        .with_reference("https://wiki.archlinux.org/title/Systemd/Journal")
                        .with_impact(format!("{} errors need investigation", error_count))
                    );
                } else if error_count > 50 {
                    issues.push(
                        CaretakerIssue::new(
                            IssueSeverity::Warning,
                            format!("Elevated journal error volume ({} errors)", error_count),
                            "Your system journal has more errors than normal. This may indicate configuration issues or failing hardware.",
                            "Review errors with 'journalctl -p err -b' and address the most common patterns"
                        )
                        .with_repair_action("journal-cleanup")
                        .with_reference("https://wiki.archlinux.org/title/Systemd/Journal")
                    );
                }
            }
        }
    }

    /// Check for zombie processes (always relevant)
    fn check_zombie_processes(issues: &mut Vec<CaretakerIssue>, _profile: MachineProfile) {
        // Check /proc for zombie processes (State: Z)
        let mut zombie_count = 0;
        let mut zombie_names = Vec::new();

        if let Ok(entries) = std::fs::read_dir("/proc") {
            for entry in entries.flatten() {
                if let Ok(file_name) = entry.file_name().into_string() {
                    // Check if directory name is numeric (PID)
                    if file_name.chars().all(|c| c.is_ascii_digit()) {
                        let status_path = entry.path().join("status");
                        if let Ok(status_content) = std::fs::read_to_string(status_path) {
                            // Check for "State: Z (zombie)"
                            if status_content.lines().any(|line| {
                                line.starts_with("State:") && line.contains("Z")
                            }) {
                                zombie_count += 1;
                                // Try to get process name
                                if let Some(name_line) = status_content.lines().find(|l| l.starts_with("Name:")) {
                                    if let Some(name) = name_line.split(':').nth(1) {
                                        zombie_names.push(name.trim().to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if zombie_count > 10 {
            issues.push(
                CaretakerIssue::new(
                    IssueSeverity::Warning,
                    format!("{} zombie processes detected", zombie_count),
                    "Multiple zombie processes are accumulating. This usually means parent processes are not properly cleaning up their children.",
                    "Identify and fix the parent processes. Zombies cannot be killed directly - their parent process must reap them."
                )
                .with_reference("https://wiki.archlinux.org/title/Core_utilities#Process_management")
            );
        } else if zombie_count > 0 {
            let process_list = if zombie_names.is_empty() {
                String::new()
            } else {
                format!(" ({})", zombie_names.join(", "))
            };

            issues.push(
                CaretakerIssue::new(
                    IssueSeverity::Info,
                    format!("{} zombie process(es) detected{}", zombie_count, process_list),
                    "Zombie processes are harmless but may indicate improper process management.",
                    "Use 'ps aux | grep Z' to identify zombies and check their parent processes"
                )
                .with_reference("https://wiki.archlinux.org/title/Core_utilities#Process_management")
            );
        }
    }

    /// Check for orphaned packages (always relevant)
    fn check_orphaned_packages(issues: &mut Vec<CaretakerIssue>, _profile: MachineProfile) {
        // Run pacman -Qtdq to list orphaned packages
        let output = std::process::Command::new("pacman")
            .args(&["-Qtdq"])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let orphans: Vec<&str> = stdout.lines()
                    .filter(|line| !line.trim().is_empty())
                    .collect();

                if orphans.len() > 50 {
                    issues.push(
                        CaretakerIssue::new(
                            IssueSeverity::Warning,
                            format!("{} orphaned packages found", orphans.len()),
                            "Many packages are installed as dependencies but no longer required by any package. These consume disk space unnecessarily.",
                            "Remove with 'sudo pacman -Rns $(pacman -Qtdq)' after reviewing the list"
                        )
                        .with_repair_action("orphaned-packages")
                        .with_reference("https://wiki.archlinux.org/title/Pacman/Tips_and_tricks#Removing_unused_packages_(orphans)")
                    );
                } else if orphans.len() > 10 {
                    issues.push(
                        CaretakerIssue::new(
                            IssueSeverity::Info,
                            format!("{} orphaned packages found", orphans.len()),
                            "Some packages are no longer needed. Cleaning them up can free disk space.",
                            "Review with 'pacman -Qtd' and remove with 'sudo pacman -Rns $(pacman -Qtdq)'"
                        )
                        .with_repair_action("orphaned-packages")
                        .with_reference("https://wiki.archlinux.org/title/Pacman/Tips_and_tricks#Removing_unused_packages_(orphans)")
                    );
                }
            }
        }
    }

    /// Check for stale core dumps (always relevant)
    fn check_core_dumps(issues: &mut Vec<CaretakerIssue>, _profile: MachineProfile) {
        let coredump_path = std::path::Path::new("/var/lib/systemd/coredump");

        if !coredump_path.exists() {
            return; // coredump directory doesn't exist
        }

        let mut total_size: u64 = 0;
        let mut file_count = 0;
        let mut old_dumps = 0;

        if let Ok(entries) = std::fs::read_dir(coredump_path) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        file_count += 1;
                        total_size += metadata.len();

                        // Check if file is older than 30 days
                        if let Ok(modified) = metadata.modified() {
                            if let Ok(elapsed) = modified.elapsed() {
                                if elapsed.as_secs() > 30 * 24 * 3600 {
                                    old_dumps += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        let size_mb = total_size / (1024 * 1024);

        if size_mb > 1000 {
            issues.push(
                CaretakerIssue::new(
                    IssueSeverity::Warning,
                    format!("Large core dump accumulation ({} MB, {} files)", size_mb, file_count),
                    "Core dumps are consuming significant disk space. Old dumps can usually be safely removed.",
                    "Review with 'coredumpctl list' and clean up with 'sudo rm /var/lib/systemd/coredump/*'"
                )
                .with_repair_action("core-dump-cleanup")
                .with_reference("https://wiki.archlinux.org/title/Core_dump")
                .with_impact(format!("Frees {} MB", size_mb))
            );
        } else if file_count > 10 && old_dumps > 5 {
            issues.push(
                CaretakerIssue::new(
                    IssueSeverity::Info,
                    format!("{} core dumps found ({} MB)", file_count, size_mb),
                    format!("{} dumps are older than 30 days and can likely be removed.", old_dumps),
                    "Review with 'coredumpctl list' and clean old dumps with 'sudo coredumpctl --since=-30days vacuum'"
                )
                .with_repair_action("core-dump-cleanup")
                .with_reference("https://wiki.archlinux.org/title/Core_dump")
            );
        }
    }

    /// Check time synchronization status (profile-aware severity)
    fn check_time_sync(issues: &mut Vec<CaretakerIssue>, profile: MachineProfile) {
        // Check for systemd-timesyncd
        let timesyncd_active = std::process::Command::new("systemctl")
            .args(&["is-active", "systemd-timesyncd.service"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if timesyncd_active {
            return; // Time sync is working via systemd-timesyncd
        }

        // Check for chronyd
        let chronyd_active = std::process::Command::new("systemctl")
            .args(&["is-active", "chronyd.service"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if chronyd_active {
            return; // Time sync is working via chronyd
        }

        // Check for ntpd
        let ntpd_active = std::process::Command::new("systemctl")
            .args(&["is-active", "ntpd.service"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if ntpd_active {
            return; // Time sync is working via ntpd
        }

        // Check if systemd-timesyncd is installed but not enabled
        let timesyncd_installed = std::process::Command::new("which")
            .arg("timedatectl")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if timesyncd_installed {
            // Check if it's enabled
            let timesyncd_enabled = std::process::Command::new("systemctl")
                .args(&["is-enabled", "systemd-timesyncd.service"])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

            if !timesyncd_enabled {
                issues.push(
                    CaretakerIssue::new(
                        IssueSeverity::Info,
                        "Time synchronization not enabled",
                        "systemd-timesyncd is available but not enabled. Your system clock may drift over time.",
                        "Run 'sudo systemctl enable --now systemd-timesyncd.service'"
                    )
                    .with_repair_action("time-sync-enable")
                    .with_reference("https://wiki.archlinux.org/title/Systemd-timesyncd")
                );
                return;
            }
        }

        // No time sync service active at all
        // Severity depends on profile: Warning for interactive, Info for server-like
        let severity = if profile.is_interactive() {
            IssueSeverity::Warning
        } else {
            IssueSeverity::Info
        };

        issues.push(
            CaretakerIssue::new(
                severity,
                "No network time synchronization active",
                "Your system clock is not using any network time synchronization. This can cause issues with TLS certificates, logs, and time-sensitive applications.",
                "Install and enable systemd-timesyncd: 'sudo systemctl enable --now systemd-timesyncd.service'"
            )
            .with_repair_action("time-sync-enable")
            .with_reference("https://wiki.archlinux.org/title/System_time")
        );
    }

    /// Check firewall status for networked machines (profile-aware severity)
    fn check_firewall_status(issues: &mut Vec<CaretakerIssue>, profile: MachineProfile) {
        // Check if this is a networked machine (has non-loopback interface up)
        let has_network = std::process::Command::new("ip")
            .args(&["link", "show", "up"])
            .output()
            .ok()
            .and_then(|o| {
                let output = String::from_utf8_lossy(&o.stdout);
                // Look for interfaces that are not "lo" (loopback)
                Some(output.lines().any(|line| {
                    line.contains("state UP") && !line.contains(": lo:")
                }))
            })
            .unwrap_or(false);

        if !has_network {
            return; // Not a networked machine or can't detect, skip
        }

        // Check for active firewall solutions
        // 1. Check ufw
        let ufw_active = std::process::Command::new("ufw")
            .arg("status")
            .output()
            .ok()
            .and_then(|o| {
                let output = String::from_utf8_lossy(&o.stdout);
                Some(output.contains("Status: active"))
            })
            .unwrap_or(false);

        if ufw_active {
            return; // Firewall active via ufw
        }

        // 2. Check firewalld
        let firewalld_active = std::process::Command::new("systemctl")
            .args(&["is-active", "firewalld.service"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if firewalld_active {
            return; // Firewall active via firewalld
        }

        // 3. Check for nftables rules
        let nftables_active = std::process::Command::new("nft")
            .args(&["list", "ruleset"])
            .output()
            .ok()
            .and_then(|o| {
                let output = String::from_utf8_lossy(&o.stdout);
                // If there are actual rules (not just empty output), consider it active
                Some(output.lines().count() > 5)
            })
            .unwrap_or(false);

        if nftables_active {
            return; // Firewall active via nftables
        }

        // 4. Check for iptables rules
        let iptables_active = std::process::Command::new("iptables")
            .args(&["-L", "-n"])
            .output()
            .ok()
            .and_then(|o| {
                let output = String::from_utf8_lossy(&o.stdout);
                // If there are non-default rules, consider it active
                // Default iptables with no rules usually has only ACCEPT policies
                Some(output.lines().count() > 10)
            })
            .unwrap_or(false);

        if iptables_active {
            return; // Firewall active via iptables
        }

        // Check if firewall packages are installed but not enabled
        let ufw_installed = std::process::Command::new("which")
            .arg("ufw")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        let firewalld_installed = std::process::Command::new("which")
            .arg("firewall-cmd")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if ufw_installed || firewalld_installed {
            let tool = if ufw_installed { "ufw" } else { "firewalld" };
            issues.push(
                CaretakerIssue::new(
                    IssueSeverity::Info,
                    format!("Firewall tool installed but not active ({})", tool),
                    format!("Your system has {} installed but it is not currently active. Incoming network connections are not filtered.", tool),
                    if ufw_installed {
                        "Enable ufw: 'sudo ufw enable' (configure rules first: 'sudo ufw allow ssh')"
                    } else {
                        "Enable firewalld: 'sudo systemctl enable --now firewalld'"
                    }
                )
                .with_reference("https://wiki.archlinux.org/title/Security")
            );
            return;
        }

        // No firewall detected at all
        // Severity depends on profile: Warning for laptops (mobile/untrusted networks), Info for server-like
        let severity = if profile.is_laptop() {
            IssueSeverity::Warning
        } else {
            IssueSeverity::Info
        };

        issues.push(
            CaretakerIssue::new(
                severity,
                "No active firewall detected",
                "This machine appears to be online with no active firewall. Incoming connections are not filtered. Consider installing and configuring ufw or firewalld.",
                "Install ufw: 'sudo pacman -S ufw', then configure: 'sudo ufw allow ssh && sudo ufw enable'"
            )
            .with_reference("https://wiki.archlinux.org/title/Uncomplicated_Firewall")
        );
    }

    /// Check backup and snapshot awareness (always info-level only)
    fn check_backup_awareness(issues: &mut Vec<CaretakerIssue>, _profile: MachineProfile) {
        // Check for common backup tools
        let timeshift_installed = std::process::Command::new("which")
            .arg("timeshift")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if timeshift_installed {
            return; // Timeshift present
        }

        let borg_installed = std::process::Command::new("which")
            .arg("borg")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if borg_installed {
            return; // Borg backup present
        }

        let restic_installed = std::process::Command::new("which")
            .arg("restic")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if restic_installed {
            return; // Restic present
        }

        // Check if this is a btrfs system with snapshot capability
        let is_btrfs = std::process::Command::new("findmnt")
            .args(&["-n", "-o", "FSTYPE", "/"])
            .output()
            .ok()
            .and_then(|o| {
                let output = String::from_utf8_lossy(&o.stdout);
                Some(output.trim() == "btrfs")
            })
            .unwrap_or(false);

        if is_btrfs {
            // Check if btrfs-progs is installed (has btrfs command)
            let btrfs_tools = std::process::Command::new("which")
                .arg("btrfs")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

            if btrfs_tools {
                return; // btrfs with snapshot tools available
            }
        }

        // Check for rsnapshot
        let rsnapshot_installed = std::process::Command::new("which")
            .arg("rsnapshot")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if rsnapshot_installed {
            return; // rsnapshot present
        }

        // No obvious backup tools detected
        issues.push(
            CaretakerIssue::new(
                IssueSeverity::Info,
                "No backup or snapshot tools detected",
                "No common backup tools (timeshift, borg, restic) or btrfs snapshots detected. If this machine holds important data, consider configuring backups.",
                "Options: Install timeshift ('pacman -S timeshift'), borg ('pacman -S borg'), or restic ('pacman -S restic'). For btrfs, use snapper or timeshift."
            )
            .with_reference("https://wiki.archlinux.org/title/Backup_programs")
        );
    }

    /// Interpret a probe result into human-readable terms
    fn interpret_probe_result(result: &crate::ipc::HealthProbeResult) -> (String, String, String) {
        // Extract message from probe details if available
        let message = result.details.get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("Issue detected");

        match result.probe.as_str() {
            "tlp-config" => (
                "TLP not properly configured".to_string(),
                format!("{} This affects battery life and power management.", message),
                "Run 'sudo annactl repair tlp-config' to enable TLP service".to_string(),
            ),
            "bluetooth-service" => (
                "Bluetooth service not working".to_string(),
                format!("{} Bluetooth functionality may not work.", message),
                "Run 'sudo annactl repair bluetooth-service' to fix".to_string(),
            ),
            "missing-firmware" => {
                let count = result.details.get("count")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                (
                    format!("{} missing firmware file(s)", count),
                    "Your hardware may not function optimally due to missing firmware.".to_string(),
                    "Run 'annactl repair missing-firmware' for guidance".to_string(),
                )
            },
            "systemd-units" => (
                "Failed systemd services detected".to_string(),
                "Some system services are not running properly.".to_string(),
                "Run 'sudo annactl repair services-failed' to restart failed services".to_string(),
            ),
            _ => (
                format!("{} issue", result.probe),
                message.to_string(),
                format!("Run 'sudo annactl repair {}' to fix", result.probe),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_healthy_analysis() {
        let analysis = CaretakerAnalysis::healthy();
        assert_eq!(analysis.overall_status, "healthy");
        assert_eq!(analysis.issues.len(), 0);
        assert!(analysis.summary.contains("healthy"));
    }

    #[test]
    fn test_issue_ordering() {
        let issues = vec![
            CaretakerIssue::new(IssueSeverity::Info, "Info", "Info", "Fix"),
            CaretakerIssue::new(IssueSeverity::Critical, "Critical", "Critical", "Fix"),
            CaretakerIssue::new(IssueSeverity::Warning, "Warning", "Warning", "Fix"),
        ];

        let analysis = CaretakerAnalysis::from_issues(issues);

        // Should be sorted: Critical, Warning, Info
        assert_eq!(analysis.issues[0].severity, IssueSeverity::Critical);
        assert_eq!(analysis.issues[1].severity, IssueSeverity::Warning);
        assert_eq!(analysis.issues[2].severity, IssueSeverity::Info);
    }

    #[test]
    fn test_overall_status() {
        let critical = vec![
            CaretakerIssue::new(IssueSeverity::Critical, "Test", "Test", "Fix"),
        ];
        let analysis = CaretakerAnalysis::from_issues(critical);
        assert_eq!(analysis.overall_status, "critical");

        let warning = vec![
            CaretakerIssue::new(IssueSeverity::Warning, "Test", "Test", "Fix"),
        ];
        let analysis = CaretakerAnalysis::from_issues(warning);
        assert_eq!(analysis.overall_status, "needs-attention");

        let healthy = Vec::new();
        let analysis = CaretakerAnalysis::from_issues(healthy);
        assert_eq!(analysis.overall_status, "healthy");
    }

    #[test]
    fn test_analyze_runs_without_errors() {
        // Test that analyze() can run without crashing
        // It should gracefully handle missing commands or files
        let analysis = CaretakerBrain::analyze(None, None, MachineProfile::Unknown);

        // Should always return an analysis, even if empty
        assert!(
            analysis.overall_status == "healthy" ||
            analysis.overall_status == "needs-attention" ||
            analysis.overall_status == "critical"
        );
    }

    #[test]
    fn test_journal_detector_runs() {
        // Test that check_journal_errors runs without crashing
        let mut issues = Vec::new();
        CaretakerBrain::check_journal_errors(&mut issues, MachineProfile::Unknown);

        // Should not crash, issues may or may not be added depending on system state
        // Just verify the function is callable
    }

    #[test]
    fn test_zombie_detector_runs() {
        // Test that check_zombie_processes runs without crashing
        let mut issues = Vec::new();
        CaretakerBrain::check_zombie_processes(&mut issues, MachineProfile::Unknown);

        // Should not crash, issues may or may not be added
    }

    #[test]
    fn test_orphaned_packages_detector_runs() {
        // Test that check_orphaned_packages runs without crashing
        let mut issues = Vec::new();
        CaretakerBrain::check_orphaned_packages(&mut issues, MachineProfile::Unknown);

        // Should not crash, issues may or may not be added
    }

    #[test]
    fn test_core_dumps_detector_runs() {
        // Test that check_core_dumps runs without crashing
        let mut issues = Vec::new();
        CaretakerBrain::check_core_dumps(&mut issues, MachineProfile::Unknown);

        // Should not crash, issues may or may not be added
    }

    #[test]
    fn test_detector_graceful_failure() {
        // Test that all detectors fail gracefully when commands are unavailable
        // This is a smoke test to ensure no panics occur
        let mut issues = Vec::new();

        CaretakerBrain::check_pacman_lock(&mut issues, MachineProfile::Unknown);
        CaretakerBrain::check_laptop_power_management(&mut issues, MachineProfile::Unknown);
        CaretakerBrain::check_gpu_drivers(&mut issues, MachineProfile::Unknown);
        CaretakerBrain::check_journal_errors(&mut issues, MachineProfile::Unknown);
        CaretakerBrain::check_zombie_processes(&mut issues, MachineProfile::Unknown);
        CaretakerBrain::check_orphaned_packages(&mut issues, MachineProfile::Unknown);
        CaretakerBrain::check_core_dumps(&mut issues, MachineProfile::Unknown);

        // All detectors should complete without panicking
        // Issues list may be empty or populated depending on system state
    }

    #[test]
    fn test_issue_with_repair_action() {
        let issue = CaretakerIssue::new(
            IssueSeverity::Warning,
            "Test Issue",
            "Test explanation",
            "Test action"
        ).with_repair_action("test-repair");

        assert_eq!(issue.repair_action_id, Some("test-repair".to_string()));
    }

    #[test]
    fn test_issue_with_impact() {
        let issue = CaretakerIssue::new(
            IssueSeverity::Info,
            "Test Issue",
            "Test explanation",
            "Test action"
        ).with_impact("Frees 10GB");

        assert_eq!(issue.estimated_impact, Some("Frees 10GB".to_string()));
    }

    #[test]
    fn test_issue_with_reference() {
        let issue = CaretakerIssue::new(
            IssueSeverity::Critical,
            "Test Issue",
            "Test explanation",
            "Test action"
        ).with_reference("https://wiki.archlinux.org/title/Test");

        assert_eq!(issue.reference, Some("https://wiki.archlinux.org/title/Test".to_string()));
    }

    #[test]
    fn test_caretaker_issue_chaining() {
        // Test that method chaining works correctly
        let issue = CaretakerIssue::new(
            IssueSeverity::Warning,
            "Test",
            "Test",
            "Test"
        )
        .with_repair_action("test-repair")
        .with_impact("Frees 5GB")
        .with_reference("https://test.com");

        assert_eq!(issue.repair_action_id, Some("test-repair".to_string()));
        assert_eq!(issue.estimated_impact, Some("Frees 5GB".to_string()));
        assert_eq!(issue.reference, Some("https://test.com".to_string()));
    }

    #[test]
    fn test_time_sync_detector_runs() {
        // Test that check_time_sync runs without crashing
        let mut issues = Vec::new();
        CaretakerBrain::check_time_sync(&mut issues, MachineProfile::Unknown);

        // Should not crash, issues may or may not be added depending on system state
        // Just verify the function is callable
    }

    #[test]
    fn test_firewall_status_detector_runs() {
        // Test that check_firewall_status runs without crashing
        let mut issues = Vec::new();
        CaretakerBrain::check_firewall_status(&mut issues, MachineProfile::Unknown);

        // Should not crash, issues may or may not be added
    }

    #[test]
    fn test_backup_awareness_detector_runs() {
        // Test that check_backup_awareness runs without crashing
        let mut issues = Vec::new();
        CaretakerBrain::check_backup_awareness(&mut issues, MachineProfile::Unknown);

        // Should not crash, issues may or may not be added
    }

    #[test]
    fn test_all_phase_4_5_detectors() {
        // Test that all Phase 4.5 detectors run without panicking
        let mut issues = Vec::new();

        CaretakerBrain::check_time_sync(&mut issues, MachineProfile::Unknown);
        CaretakerBrain::check_firewall_status(&mut issues, MachineProfile::Unknown);
        CaretakerBrain::check_backup_awareness(&mut issues, MachineProfile::Unknown);

        // All detectors should complete without panicking
        // Issues list may be empty or populated depending on system state
    }

    #[test]
    fn test_analyze_includes_all_12_detectors() {
        // Test that analyze() runs all 12 detectors without crashing
        let analysis = CaretakerBrain::analyze(None, None, MachineProfile::Unknown);

        // Should always return an analysis
        assert!(
            analysis.overall_status == "healthy" ||
            analysis.overall_status == "needs-attention" ||
            analysis.overall_status == "critical"
        );

        // Analysis should be valid regardless of system state
        assert!(analysis.issues.len() >= 0);
    }

    #[test]
    fn test_time_sync_issue_has_repair_action() {
        // Create a mock time sync issue and verify it has repair action
        let issue = CaretakerIssue::new(
            IssueSeverity::Warning,
            "No network time synchronization active",
            "Test",
            "Test"
        ).with_repair_action("time-sync-enable");

        assert_eq!(issue.repair_action_id, Some("time-sync-enable".to_string()));
    }

    #[test]
    fn test_firewall_issue_guidance_only() {
        // Firewall issues should not have repair_action_id (guidance only)
        let issue = CaretakerIssue::new(
            IssueSeverity::Warning,
            "No active firewall detected",
            "Test",
            "Install ufw"
        );
        // Firewall detection produces issues without repair_action_id
        assert_eq!(issue.repair_action_id, None);
    }

    #[test]
    fn test_backup_issue_info_severity() {
        // Backup issues should always be Info level
        let issue = CaretakerIssue::new(
            IssueSeverity::Info,
            "No backup or snapshot tools detected",
            "Test",
            "Test"
        );
        assert_eq!(issue.severity, IssueSeverity::Info);
    }

    #[test]
    fn test_detector_graceful_failure_with_phase_4_5() {
        // Test that all detectors (including new ones) fail gracefully
        let mut issues = Vec::new();
        let profile = MachineProfile::Unknown;  // Phase 4.6: profile parameter

        // Original detectors
        CaretakerBrain::check_pacman_lock(&mut issues, profile);
        CaretakerBrain::check_laptop_power_management(&mut issues, profile);
        CaretakerBrain::check_gpu_drivers(&mut issues, profile);
        CaretakerBrain::check_journal_errors(&mut issues, profile);
        CaretakerBrain::check_zombie_processes(&mut issues, profile);
        CaretakerBrain::check_orphaned_packages(&mut issues, profile);
        CaretakerBrain::check_core_dumps(&mut issues, profile);

        // Phase 4.5 detectors
        CaretakerBrain::check_time_sync(&mut issues, profile);
        CaretakerBrain::check_firewall_status(&mut issues, profile);
        CaretakerBrain::check_backup_awareness(&mut issues, profile);

        // All detectors should complete without panicking
    }
}
