//! Quick Fix - One-click solutions for common issues.
//!
//! v0.3.118: Automated fixes Anna can safely perform.

use serde::{Deserialize, Serialize};

/// A quick fix that Anna can perform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickFix {
    /// Unique identifier
    pub id: String,
    /// Short description
    pub title: String,
    /// What this fix does
    pub description: String,
    /// Command(s) to execute
    pub commands: Vec<String>,
    /// Whether this needs sudo
    pub needs_sudo: bool,
    /// Whether this is reversible
    pub reversible: bool,
    /// Category of fix
    pub category: FixCategory,
    /// Estimated impact (low/medium/high)
    pub impact: Impact,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FixCategory {
    Cleanup,
    Service,
    Package,
    Cache,
    Config,
    Security,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Impact {
    Low,
    Medium,
    High,
}

impl QuickFix {
    /// Create a new quick fix.
    pub fn new(id: &str, title: &str, description: &str) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            commands: Vec::new(),
            needs_sudo: false,
            reversible: true,
            category: FixCategory::Cleanup,
            impact: Impact::Low,
        }
    }

    pub fn command(mut self, cmd: &str) -> Self {
        self.commands.push(cmd.to_string());
        self
    }

    pub fn sudo(mut self) -> Self {
        self.needs_sudo = true;
        self
    }

    pub fn category(mut self, cat: FixCategory) -> Self {
        self.category = cat;
        self
    }

    pub fn impact(mut self, imp: Impact) -> Self {
        self.impact = imp;
        self
    }

    pub fn irreversible(mut self) -> Self {
        self.reversible = false;
        self
    }
}

/// Get available quick fixes based on current system state.
pub fn get_available_fixes() -> Vec<QuickFix> {
    let mut fixes = Vec::new();

    // Check for orphan packages
    if has_orphan_packages() {
        fixes.push(
            QuickFix::new(
                "remove-orphans",
                "Remove orphan packages",
                "Remove packages that were installed as dependencies but are no longer needed"
            )
            .command("pacman -Rns $(pacman -Qdtq)")
            .sudo()
            .category(FixCategory::Package)
            .impact(Impact::Low)
        );
    }

    // Check pacman cache
    let cache_size = get_pacman_cache_size_mb();
    if cache_size > 1000.0 {
        fixes.push(
            QuickFix::new(
                "clean-pacman-cache",
                "Clean package cache",
                &format!("Remove old package versions from cache ({:.1} GB)", cache_size / 1024.0)
            )
            .command("paccache -rk2")  // Keep last 2 versions
            .sudo()
            .category(FixCategory::Cache)
            .impact(Impact::Low)
        );
    }

    // Check journal size
    let journal_size = get_journal_size_mb();
    if journal_size > 500.0 {
        fixes.push(
            QuickFix::new(
                "clean-journal",
                "Clean old journal logs",
                &format!("Remove journal logs older than 7 days ({:.0} MB)", journal_size)
            )
            .command("journalctl --vacuum-time=7d")
            .sudo()
            .category(FixCategory::Cleanup)
            .impact(Impact::Low)
        );
    }

    // Check for failed services
    let failed = get_failed_services();
    for service in failed.iter().take(3) {
        fixes.push(
            QuickFix::new(
                &format!("restart-{}", service.replace(".service", "")),
                &format!("Restart {}", service),
                &format!("Attempt to restart the failed {} service", service)
            )
            .command(&format!("systemctl restart {}", service))
            .sudo()
            .category(FixCategory::Service)
            .impact(Impact::Medium)
        );
    }

    // Check tmp directory
    let tmp_size = get_tmp_size_mb();
    if tmp_size > 1000.0 {
        fixes.push(
            QuickFix::new(
                "clean-tmp",
                "Clean temporary files",
                &format!("Remove old files from /tmp ({:.1} GB)", tmp_size / 1024.0)
            )
            .command("find /tmp -type f -atime +7 -delete 2>/dev/null")
            .sudo()
            .category(FixCategory::Cleanup)
            .impact(Impact::Low)
        );
    }

    // Check for stale thumbnails
    let thumb_size = get_thumbnail_cache_size_mb();
    if thumb_size > 500.0 {
        fixes.push(
            QuickFix::new(
                "clean-thumbnails",
                "Clean thumbnail cache",
                &format!("Remove old thumbnail cache ({:.0} MB)", thumb_size)
            )
            .command("rm -rf ~/.cache/thumbnails/*")
            .category(FixCategory::Cache)
            .impact(Impact::Low)
        );
    }

    // Check for pending updates (informational)
    let updates = get_pending_updates_count();
    if updates > 10 {
        fixes.push(
            QuickFix::new(
                "system-update",
                "Update system packages",
                &format!("Install {} pending package updates", updates)
            )
            .command("pacman -Syu")
            .sudo()
            .category(FixCategory::Package)
            .impact(Impact::High)
            .irreversible()
        );
    }

    fixes
}

/// Format quick fixes for display.
pub fn format_quick_fixes(fixes: &[QuickFix]) -> String {
    if fixes.is_empty() {
        return "No quick fixes available - system looks good!".to_string();
    }

    let mut output = String::new();
    output.push_str("\nQUICK FIXES AVAILABLE\n");
    output.push_str("=====================\n\n");

    for (i, fix) in fixes.iter().enumerate() {
        let impact_str = match fix.impact {
            Impact::Low => "[Low]",
            Impact::Medium => "[Med]",
            Impact::High => "[High]",
        };

        let sudo_str = if fix.needs_sudo { "(sudo)" } else { "" };

        output.push_str(&format!(
            "{}. {} {} {}\n",
            i + 1,
            fix.title,
            impact_str,
            sudo_str
        ));
        output.push_str(&format!("   {}\n", fix.description));
        output.push_str(&format!("   Run: annactl fix {}\n\n", fix.id));
    }

    output.push_str("Use 'annactl fix <id>' to apply a fix, or 'annactl fix --all' for all low-impact fixes.\n");

    output
}

/// Get a summary line of available fixes.
pub fn fixes_summary() -> String {
    let fixes = get_available_fixes();
    if fixes.is_empty() {
        return "No fixes needed".to_string();
    }

    let low = fixes.iter().filter(|f| f.impact == Impact::Low).count();
    let med = fixes.iter().filter(|f| f.impact == Impact::Medium).count();
    let high = fixes.iter().filter(|f| f.impact == Impact::High).count();

    format!(
        "{} fixes available: {} low, {} medium, {} high impact",
        fixes.len(), low, med, high
    )
}

// === Helper functions ===

fn has_orphan_packages() -> bool {
    std::process::Command::new("pacman")
        .args(["-Qdtq"])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false)
}

fn get_pacman_cache_size_mb() -> f64 {
    std::process::Command::new("du")
        .args(["-sm", "/var/cache/pacman/pkg"])
        .output()
        .ok()
        .and_then(|o| {
            String::from_utf8_lossy(&o.stdout)
                .split_whitespace()
                .next()
                .and_then(|s| s.parse::<f64>().ok())
        })
        .unwrap_or(0.0)
}

fn get_journal_size_mb() -> f64 {
    std::process::Command::new("journalctl")
        .args(["--disk-usage"])
        .output()
        .ok()
        .and_then(|o| {
            let out = String::from_utf8_lossy(&o.stdout);
            // Parse "Archived and active journals take up 123.4M"
            out.split_whitespace()
                .find(|s| s.ends_with('M') || s.ends_with('G'))
                .and_then(|s| {
                    let multiplier = if s.ends_with('G') { 1024.0 } else { 1.0 };
                    s.trim_end_matches(|c| c == 'M' || c == 'G')
                        .parse::<f64>()
                        .ok()
                        .map(|v| v * multiplier)
                })
        })
        .unwrap_or(0.0)
}

fn get_failed_services() -> Vec<String> {
    std::process::Command::new("systemctl")
        .args(["--failed", "--no-legend", "--no-pager"])
        .output()
        .ok()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .filter_map(|l| l.split_whitespace().next())
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default()
}

fn get_tmp_size_mb() -> f64 {
    std::process::Command::new("du")
        .args(["-sm", "/tmp"])
        .output()
        .ok()
        .and_then(|o| {
            String::from_utf8_lossy(&o.stdout)
                .split_whitespace()
                .next()
                .and_then(|s| s.parse::<f64>().ok())
        })
        .unwrap_or(0.0)
}

fn get_thumbnail_cache_size_mb() -> f64 {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    let path = format!("{}/.cache/thumbnails", home);
    std::process::Command::new("du")
        .args(["-sm", &path])
        .output()
        .ok()
        .and_then(|o| {
            String::from_utf8_lossy(&o.stdout)
                .split_whitespace()
                .next()
                .and_then(|s| s.parse::<f64>().ok())
        })
        .unwrap_or(0.0)
}

fn get_pending_updates_count() -> usize {
    std::process::Command::new("checkupdates")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).lines().count())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quickfix_builder() {
        let fix = QuickFix::new("test", "Test Fix", "A test fix")
            .command("echo test")
            .sudo()
            .category(FixCategory::Cleanup)
            .impact(Impact::Low);

        assert_eq!(fix.id, "test");
        assert!(fix.needs_sudo);
        assert_eq!(fix.commands.len(), 1);
    }

    #[test]
    fn test_format_empty() {
        let output = format_quick_fixes(&[]);
        assert!(output.contains("looks good"));
    }
}
