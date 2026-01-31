//! Proactive Issue Detection - Anna detects problems before you ask.
//!
//! Scans system for common issues and provides actionable alerts.
//! Runs automatically and surfaces issues in health reports and answers.

use crate::live_state::LiveState;
use std::process::Command;

/// Severity of detected issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IssueSeverity {
    Info,
    Warning,
    Critical,
}

/// A proactively detected issue.
#[derive(Debug, Clone)]
pub struct DetectedIssue {
    pub severity: IssueSeverity,
    pub category: String,
    pub title: String,
    pub description: String,
    pub suggestion: String,
    pub auto_fixable: bool,
}

impl DetectedIssue {
    fn new(
        severity: IssueSeverity,
        category: &str,
        title: &str,
        description: &str,
        suggestion: &str,
        auto_fixable: bool,
    ) -> Self {
        Self {
            severity,
            category: category.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            suggestion: suggestion.to_string(),
            auto_fixable,
        }
    }

    /// Format for display.
    pub fn format(&self) -> String {
        let icon = match self.severity {
            IssueSeverity::Critical => "!!",
            IssueSeverity::Warning => "!",
            IssueSeverity::Info => "i",
        };

        format!(
            "[{}] {}: {}\n    {}\n    Suggestion: {}{}",
            icon,
            self.category,
            self.title,
            self.description,
            self.suggestion,
            if self.auto_fixable { " [auto-fixable]" } else { "" }
        )
    }
}

/// Scan for all detectable issues.
pub fn scan_for_issues() -> Vec<DetectedIssue> {
    let mut issues = Vec::new();
    let state = LiveState::capture();

    // Resource issues
    issues.extend(check_resource_issues(&state));

    // Service issues
    issues.extend(check_service_issues(&state));

    // Security issues
    issues.extend(check_security_issues());

    // Maintenance issues
    issues.extend(check_maintenance_issues());

    // Package issues
    issues.extend(check_package_issues());

    // Sort by severity (critical first)
    issues.sort_by(|a, b| b.severity.cmp(&a.severity));

    issues
}

fn check_resource_issues(state: &LiveState) -> Vec<DetectedIssue> {
    let mut issues = Vec::new();

    // Disk space
    let disk_pct = state.disk.percent_used();
    if disk_pct > 95.0 {
        issues.push(DetectedIssue::new(
            IssueSeverity::Critical,
            "Disk",
            "Critically low disk space",
            &format!("Root partition is {:.0}% full", disk_pct),
            "Run 'paccache -rk1' and clean logs",
            true,
        ));
    } else if disk_pct > 85.0 {
        issues.push(DetectedIssue::new(
            IssueSeverity::Warning,
            "Disk",
            "Low disk space",
            &format!("Root partition is {:.0}% full", disk_pct),
            "Consider cleaning package cache and old logs",
            true,
        ));
    }

    // Memory
    let mem_pct = state.memory.percent_used();
    if mem_pct > 95.0 {
        issues.push(DetectedIssue::new(
            IssueSeverity::Critical,
            "Memory",
            "Critical memory pressure",
            &format!("Memory usage is {:.0}%", mem_pct),
            "Close applications or add swap space",
            false,
        ));
    } else if mem_pct > 90.0 {
        issues.push(DetectedIssue::new(
            IssueSeverity::Warning,
            "Memory",
            "High memory usage",
            &format!("Memory usage is {:.0}%", mem_pct),
            "Monitor for memory-hungry processes",
            false,
        ));
    }

    // Swap usage (indicates memory pressure)
    if state.memory.swap_percent_used() > 80.0 {
        issues.push(DetectedIssue::new(
            IssueSeverity::Warning,
            "Memory",
            "Heavy swap usage",
            &format!("Swap is {:.0}% used, indicating memory pressure", state.memory.swap_percent_used()),
            "Consider adding more RAM or closing applications",
            false,
        ));
    }

    // High load
    let cores = std::thread::available_parallelism().map(|p| p.get()).unwrap_or(1);
    if state.load_avg.0 > (cores * 3) as f32 {
        issues.push(DetectedIssue::new(
            IssueSeverity::Critical,
            "CPU",
            "Extremely high system load",
            &format!("Load average {:.1} is {}x the core count", state.load_avg.0, state.load_avg.0 as usize / cores),
            "Check for runaway processes with 'top' or 'htop'",
            false,
        ));
    } else if state.load_avg.0 > (cores * 2) as f32 {
        issues.push(DetectedIssue::new(
            IssueSeverity::Warning,
            "CPU",
            "High system load",
            &format!("Load average {:.1} is above normal", state.load_avg.0),
            "System may feel slow, check for CPU-intensive processes",
            false,
        ));
    }

    issues
}

fn check_service_issues(state: &LiveState) -> Vec<DetectedIssue> {
    let mut issues = Vec::new();

    // Failed services
    for unit in &state.failed_units {
        let severity = if is_critical_service(unit) {
            IssueSeverity::Critical
        } else {
            IssueSeverity::Warning
        };

        issues.push(DetectedIssue::new(
            severity,
            "Services",
            &format!("Failed: {}", unit),
            &format!("Service {} has failed", unit),
            &format!("Check logs with 'journalctl -u {}'", unit),
            is_safe_to_restart(unit),
        ));
    }

    issues
}

fn check_security_issues() -> Vec<DetectedIssue> {
    let mut issues = Vec::new();

    // Check for SSH password auth (if sshd is running)
    if let Ok(output) = Command::new("grep")
        .args(["-E", "^PasswordAuthentication\\s+yes", "/etc/ssh/sshd_config"])
        .output()
    {
        if !output.stdout.is_empty() {
            issues.push(DetectedIssue::new(
                IssueSeverity::Warning,
                "Security",
                "SSH password authentication enabled",
                "Password authentication for SSH is less secure than key-based auth",
                "Consider switching to SSH key authentication",
                false,
            ));
        }
    }

    // Check for open world-writable directories
    if std::path::Path::new("/tmp").exists() {
        if let Ok(output) = Command::new("find")
            .args(["/tmp", "-maxdepth", "1", "-type", "d", "-perm", "-o+w", "-not", "-name", "tmp"])
            .output()
        {
            let dirs = String::from_utf8_lossy(&output.stdout);
            let count = dirs.lines().count();
            if count > 5 {
                issues.push(DetectedIssue::new(
                    IssueSeverity::Info,
                    "Security",
                    "Many world-writable directories",
                    &format!("{} world-writable directories in /tmp", count),
                    "Normal, but consider periodic cleanup",
                    true,
                ));
            }
        }
    }

    issues
}

fn check_maintenance_issues() -> Vec<DetectedIssue> {
    let mut issues = Vec::new();

    // Pacman lock file
    if std::path::Path::new("/var/lib/pacman/db.lck").exists() {
        // Check if pacman is actually running
        let pacman_running = Command::new("pgrep")
            .arg("pacman")
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        if !pacman_running {
            issues.push(DetectedIssue::new(
                IssueSeverity::Warning,
                "Packages",
                "Stale pacman lock file",
                "Pacman database is locked but pacman isn't running",
                "Remove with 'sudo rm /var/lib/pacman/db.lck'",
                true,
            ));
        }
    }

    // Journal size
    if let Ok(output) = Command::new("journalctl")
        .args(["--disk-usage"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Parse "Archived and active journals take up X on disk."
        if let Some(size) = extract_size_from_journal_output(&stdout) {
            if size > 1024 * 1024 * 1024 { // > 1GB
                issues.push(DetectedIssue::new(
                    IssueSeverity::Info,
                    "Maintenance",
                    "Large journal logs",
                    &format!("System journals using {:.1}GB", size as f64 / 1024.0 / 1024.0 / 1024.0),
                    "Run 'sudo journalctl --vacuum-size=500M' to clean",
                    true,
                ));
            }
        }
    }

    // Orphan packages
    if let Ok(output) = Command::new("pacman").args(["-Qdtq"]).output() {
        let orphans = String::from_utf8_lossy(&output.stdout);
        let count = orphans.lines().count();
        if count > 10 {
            issues.push(DetectedIssue::new(
                IssueSeverity::Info,
                "Packages",
                "Many orphan packages",
                &format!("{} orphan packages installed", count),
                "Review with 'pacman -Qdtq' and remove if unneeded",
                false,
            ));
        }
    }

    issues
}

fn check_package_issues() -> Vec<DetectedIssue> {
    let mut issues = Vec::new();

    // Check for partial upgrades (packages from different epochs)
    // This is a simplified check - real partial upgrade detection is complex
    if let Ok(output) = Command::new("pacman")
        .args(["-Qu", "--dbpath", "/var/lib/pacman"])
        .output()
    {
        let updates = String::from_utf8_lossy(&output.stdout);
        let update_count = updates.lines().count();

        if update_count > 100 {
            issues.push(DetectedIssue::new(
                IssueSeverity::Warning,
                "Packages",
                "Many pending updates",
                &format!("{} packages pending update", update_count),
                "Run 'sudo pacman -Syu' to update system",
                false,
            ));
        } else if update_count > 50 {
            issues.push(DetectedIssue::new(
                IssueSeverity::Info,
                "Packages",
                "Updates available",
                &format!("{} packages have updates", update_count),
                "Consider updating with 'sudo pacman -Syu'",
                false,
            ));
        }
    }

    issues
}

fn is_critical_service(unit: &str) -> bool {
    let critical = ["NetworkManager", "sshd", "gdm", "sddm", "docker", "dbus"];
    critical.iter().any(|c| unit.contains(c))
}

fn is_safe_to_restart(unit: &str) -> bool {
    let safe = ["pipewire", "wireplumber", "xdg-desktop-portal", "gvfs", "tracker"];
    safe.iter().any(|s| unit.contains(s))
}

fn extract_size_from_journal_output(output: &str) -> Option<u64> {
    // Parse strings like "Archived and active journals take up 512.0M on disk."
    for word in output.split_whitespace() {
        if word.ends_with('G') || word.ends_with('M') || word.ends_with('K') {
            let (num_str, suffix) = word.split_at(word.len() - 1);
            if let Ok(num) = num_str.parse::<f64>() {
                let multiplier = match suffix {
                    "G" => 1024 * 1024 * 1024,
                    "M" => 1024 * 1024,
                    "K" => 1024,
                    _ => 1,
                };
                return Some((num * multiplier as f64) as u64);
            }
        }
    }
    None
}

/// Format all issues for display.
pub fn format_issues(issues: &[DetectedIssue]) -> String {
    if issues.is_empty() {
        return "No issues detected. System looks healthy.".to_string();
    }

    let critical = issues.iter().filter(|i| i.severity == IssueSeverity::Critical).count();
    let warnings = issues.iter().filter(|i| i.severity == IssueSeverity::Warning).count();
    let info = issues.iter().filter(|i| i.severity == IssueSeverity::Info).count();

    let mut lines = vec![
        format!("Detected {} issue(s): {} critical, {} warnings, {} info",
            issues.len(), critical, warnings, info),
        "═".repeat(60),
    ];

    for issue in issues {
        lines.push(issue.format());
        lines.push(String::new());
    }

    lines.join("\n")
}

/// Get a one-line summary of issues.
pub fn issues_summary() -> String {
    let issues = scan_for_issues();

    if issues.is_empty() {
        return "No issues detected".to_string();
    }

    let critical = issues.iter().filter(|i| i.severity == IssueSeverity::Critical).count();
    let warnings = issues.iter().filter(|i| i.severity == IssueSeverity::Warning).count();

    if critical > 0 {
        format!("!! {} critical issue(s), {} warning(s)", critical, warnings)
    } else if warnings > 0 {
        format!("! {} warning(s) detected", warnings)
    } else {
        format!("{} info items", issues.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_for_issues() {
        let issues = scan_for_issues();
        // Just verify it runs without panic
        assert!(issues.len() >= 0);
    }

    #[test]
    fn test_format_issues() {
        let issues = vec![
            DetectedIssue::new(
                IssueSeverity::Warning,
                "Test",
                "Test issue",
                "This is a test",
                "Do nothing",
                false,
            ),
        ];
        let formatted = format_issues(&issues);
        assert!(formatted.contains("Test issue"));
    }
}
