//! Anna Inventory Drift v7.23.0 - Change Detection
//!
//! Compares current system inventory with last stored snapshot
//! to detect package, command, and service changes.
//!
//! All data comes from pacman, PATH, and systemctl.

use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process::Command;

use serde::{Deserialize, Serialize};

/// Snapshot storage path
const SNAPSHOT_DIR: &str = "/var/lib/anna/snapshots";
const INVENTORY_SNAPSHOT: &str = "/var/lib/anna/snapshots/inventory.json";

/// Inventory snapshot for comparison
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InventorySnapshot {
    pub timestamp: u64,
    pub package_count: usize,
    pub command_count: usize,
    pub service_count: usize,
    pub packages: HashSet<String>,
    pub services: HashSet<String>,
}

impl InventorySnapshot {
    /// Take a snapshot of current inventory
    pub fn current() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let packages = get_installed_packages();
        let services = get_systemd_services();
        let command_count = get_command_count();

        InventorySnapshot {
            timestamp: now,
            package_count: packages.len(),
            command_count,
            service_count: services.len(),
            packages,
            services,
        }
    }

    /// Load last saved snapshot
    pub fn load_last() -> Option<Self> {
        let path = Path::new(INVENTORY_SNAPSHOT);
        if !path.exists() {
            return None;
        }

        let content = fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// Save current snapshot
    pub fn save(&self) -> Result<(), std::io::Error> {
        // Ensure directory exists
        fs::create_dir_all(SNAPSHOT_DIR)?;

        let content = serde_json::to_string_pretty(self)?;
        fs::write(INVENTORY_SNAPSHOT, content)?;
        Ok(())
    }
}

/// Drift summary between two snapshots
#[derive(Debug, Clone, Default)]
pub struct DriftSummary {
    pub has_changes: bool,
    pub packages_added: i32,
    pub packages_removed: i32,
    pub services_added: i32,
    pub services_removed: i32,
    pub commands_delta: i32,
    pub added_packages: Vec<String>,
    pub removed_packages: Vec<String>,
    pub added_services: Vec<String>,
    pub removed_services: Vec<String>,
}

impl DriftSummary {
    /// Compare current state with last snapshot
    pub fn compute() -> Self {
        let current = InventorySnapshot::current();
        let last = match InventorySnapshot::load_last() {
            Some(s) => s,
            None => {
                // No previous snapshot, save current and report no changes
                let _ = current.save();
                return DriftSummary::default();
            }
        };

        let added_packages: Vec<String> = current
            .packages
            .difference(&last.packages)
            .cloned()
            .collect();
        let removed_packages: Vec<String> = last
            .packages
            .difference(&current.packages)
            .cloned()
            .collect();

        let added_services: Vec<String> = current
            .services
            .difference(&last.services)
            .cloned()
            .collect();
        let removed_services: Vec<String> = last
            .services
            .difference(&current.services)
            .cloned()
            .collect();

        let commands_delta = current.command_count as i32 - last.command_count as i32;

        let has_changes = !added_packages.is_empty()
            || !removed_packages.is_empty()
            || !added_services.is_empty()
            || !removed_services.is_empty()
            || commands_delta != 0;

        DriftSummary {
            has_changes,
            packages_added: added_packages.len() as i32,
            packages_removed: removed_packages.len() as i32,
            services_added: added_services.len() as i32,
            services_removed: removed_services.len() as i32,
            commands_delta,
            added_packages,
            removed_packages,
            added_services,
            removed_services,
        }
    }

    /// Format as status line
    pub fn format_status_line(&self) -> String {
        if !self.has_changes {
            "ok (no changes since last scan)".to_string()
        } else {
            let mut parts = Vec::new();

            if self.packages_added > 0 {
                parts.push(format!("+{} packages", self.packages_added));
            }
            if self.packages_removed > 0 {
                parts.push(format!("-{} packages", self.packages_removed));
            }
            if self.services_added > 0 {
                parts.push(format!("+{} services", self.services_added));
            }
            if self.services_removed > 0 {
                parts.push(format!("-{} services", self.services_removed));
            }
            if self.commands_delta != 0 {
                if self.commands_delta > 0 {
                    parts.push(format!("+{} commands", self.commands_delta));
                } else {
                    parts.push(format!("{} commands", self.commands_delta));
                }
            }

            format!("changed ({} since last scan)", parts.join(", "))
        }
    }

    /// Update snapshot after reporting drift
    pub fn update_snapshot(&self) {
        let current = InventorySnapshot::current();
        let _ = current.save();
    }
}

/// Get installed packages from pacman
fn get_installed_packages() -> HashSet<String> {
    let output = Command::new("pacman")
        .args(["-Qq"])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .map(|s| s.to_string())
                .collect()
        }
        _ => HashSet::new(),
    }
}

/// Get systemd services
fn get_systemd_services() -> HashSet<String> {
    let output = Command::new("systemctl")
        .args(["list-unit-files", "--type=service", "--no-legend", "--no-pager"])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .filter_map(|line| {
                    line.split_whitespace().next().map(|s| s.to_string())
                })
                .collect()
        }
        _ => HashSet::new(),
    }
}

/// Get command count from PATH
fn get_command_count() -> usize {
    let path = std::env::var("PATH").unwrap_or_default();
    let mut commands = HashSet::new();

    for dir in path.split(':') {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        if let Some(name) = entry.file_name().to_str() {
                            commands.insert(name.to_string());
                        }
                    }
                }
            }
        }
    }

    commands.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drift_summary_format_no_changes() {
        let summary = DriftSummary::default();
        assert_eq!(summary.format_status_line(), "ok (no changes since last scan)");
    }

    #[test]
    fn test_drift_summary_format_with_changes() {
        let summary = DriftSummary {
            has_changes: true,
            packages_added: 3,
            packages_removed: 0,
            services_added: 0,
            services_removed: 1,
            commands_delta: 5,
            ..Default::default()
        };
        let line = summary.format_status_line();
        assert!(line.contains("+3 packages"));
        assert!(line.contains("-1 services"));
        assert!(line.contains("+5 commands"));
    }
}
