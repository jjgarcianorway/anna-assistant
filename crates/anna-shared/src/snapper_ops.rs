//! Snapper/Btrfs Operations - Snapshot management and system rollback.
//!
//! v0.3.127: Support for btrfs snapshots, snapper configs, automatic snapshots.

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

/// Snapper configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapperConfig {
    pub name: String,
    pub subvolume: String,
    pub fstype: String,
}

/// Snapshot information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub number: u32,
    pub snapshot_type: String,
    pub date: String,
    pub description: String,
}

/// Check if snapper is installed.
pub fn is_snapper_installed() -> bool {
    Command::new("which")
        .arg("snapper")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if filesystem is btrfs.
pub fn is_btrfs(path: &str) -> bool {
    let output = Command::new("findmnt")
        .args(&["-n", "-o", "FSTYPE", path])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let fstype = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return fstype == "btrfs";
        }
    }
    false
}

/// Get list of snapper configs.
pub fn list_configs() -> Vec<SnapperConfig> {
    let output = Command::new("snapper")
        .args(&["list-configs"])
        .output();

    let mut configs = Vec::new();

    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines().skip(2) { // Skip header
                let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
                if parts.len() >= 3 {
                    configs.push(SnapperConfig {
                        name: parts[0].to_string(),
                        subvolume: parts[1].to_string(),
                        fstype: parts[2].to_string(),
                    });
                }
            }
        }
    }

    configs
}

/// List snapshots for a config.
pub fn list_snapshots(config: &str) -> Vec<Snapshot> {
    let output = Command::new("snapper")
        .args(&["-c", config, "list"])
        .output();

    let mut snapshots = Vec::new();

    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines().skip(2) { // Skip header
                let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
                if parts.len() >= 4 {
                    if let Ok(num) = parts[0].parse::<u32>() {
                        snapshots.push(Snapshot {
                            number: num,
                            snapshot_type: parts[1].to_string(),
                            date: parts[2].to_string(),
                            description: parts[3].to_string(),
                        });
                    }
                }
            }
        }
    }

    snapshots
}

/// Generate installation steps for snapper.
pub fn snapper_install_steps() -> Vec<String> {
    vec![
        "# Install snapper and snap-pac".to_string(),
        "pacman -S --noconfirm snapper snap-pac".to_string(),
        "".to_string(),
        "# Create root config".to_string(),
        "snapper -c root create-config /".to_string(),
        "".to_string(),
        "# Enable automatic snapshots".to_string(),
        "systemctl enable --now snapper-timeline.timer".to_string(),
        "systemctl enable --now snapper-cleanup.timer".to_string(),
        "".to_string(),
        "# Optional: Enable snapper-boot for bootable snapshots".to_string(),
        "# pacman -S --noconfirm snapper-rollback".to_string(),
    ]
}

/// Generate steps to configure snapper for automatic snapshots.
pub fn configure_auto_snapshots_steps(config: &str) -> Vec<String> {
    vec![
        format!("# Configure automatic snapshots for {}", config),
        "".to_string(),
        "# Set timeline limits".to_string(),
        format!("snapper -c {} set-config TIMELINE_MIN_AGE=1800", config),
        format!("snapper -c {} set-config TIMELINE_LIMIT_HOURLY=5", config),
        format!("snapper -c {} set-config TIMELINE_LIMIT_DAILY=7", config),
        format!("snapper -c {} set-config TIMELINE_LIMIT_WEEKLY=0", config),
        format!("snapper -c {} set-config TIMELINE_LIMIT_MONTHLY=0", config),
        format!("snapper -c {} set-config TIMELINE_LIMIT_YEARLY=0", config),
        "".to_string(),
        "# Enable timeline snapshots".to_string(),
        "systemctl enable --now snapper-timeline.timer".to_string(),
        "systemctl enable --now snapper-cleanup.timer".to_string(),
    ]
}

/// Generate rollback steps.
pub fn generate_rollback_steps(config: &str, snapshot_number: u32) -> Vec<String> {
    vec![
        format!("# Rollback {} to snapshot {}", config, snapshot_number),
        "".to_string(),
        "# Create pre-rollback snapshot".to_string(),
        format!("snapper -c {} create --description 'Before rollback to #{}'", config, snapshot_number),
        "".to_string(),
        "# Perform rollback".to_string(),
        format!("snapper -c {} undochange {}..0", config, snapshot_number),
        "".to_string(),
        "# Note: For full system rollback, consider using snapper-rollback or booting into snapshot".to_string(),
    ]
}

/// Format snapper setup plan.
pub fn format_snapper_setup_plan(root_is_btrfs: bool) -> String {
    let steps = if root_is_btrfs {
        snapper_install_steps()
    } else {
        vec![
            "ERROR: Root filesystem is not btrfs.".to_string(),
            "".to_string(),
            "Snapper requires btrfs. Your root filesystem is not btrfs.".to_string(),
            "".to_string(),
            "To use snapper, you would need to:".to_string(),
            "1. Backup all data".to_string(),
            "2. Reinstall system with btrfs root".to_string(),
            "3. Then install snapper".to_string(),
        ]
    };

    format!(
        "Snapper Setup Plan\n\
        ==================\n\
        \n\
        Root filesystem: {}\n\
        \n\
        Risk Level: {} - {}\n\
        \n\
        Steps:\n\
        {}\n\
        \n\
        IMPORTANT:\n\
        - Snapper requires btrfs filesystem\n\
        - Automatic snapshots will take disk space\n\
        - Configure snapshot retention to manage space\n\
        - snap-pac creates snapshots before/after pacman operations\n\
        \n\
        Do you want to proceed with snapper setup?",
        if root_is_btrfs { "btrfs (compatible)" } else { "not btrfs (incompatible)" },
        if root_is_btrfs { "MEDIUM" } else { "N/A" },
        if root_is_btrfs { "System config change, reversible" } else { "Cannot proceed" },
        steps.join("\n")
    )
}

/// Format rollback plan.
pub fn format_rollback_plan(config: &str, snapshot: &Snapshot) -> String {
    let steps = generate_rollback_steps(config, snapshot.number);

    format!(
        "Snapshot Rollback Plan\n\
        ======================\n\
        \n\
        Config: {}\n\
        Snapshot: #{}\n\
        Type: {}\n\
        Date: {}\n\
        Description: {}\n\
        \n\
        Risk Level: HIGH - System state will be reverted\n\
        \n\
        Steps:\n\
        {}\n\
        \n\
        IMPORTANT:\n\
        - A pre-rollback snapshot will be created\n\
        - Changes after snapshot {} will be lost\n\
        - For full system rollback, you may need to reboot\n\
        - Test thoroughly after rollback\n\
        \n\
        Do you want to proceed with this rollback?",
        config,
        snapshot.number,
        snapshot.snapshot_type,
        snapshot.date,
        snapshot.description,
        steps.join("\n"),
        snapshot.number
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_snapper_installed() {
        let _installed = is_snapper_installed();
        // Just ensure it doesn't panic
    }

    #[test]
    fn test_is_btrfs() {
        let _is_btrfs_root = is_btrfs("/");
        // Just ensure it doesn't panic
    }

    #[test]
    fn test_snapper_install_steps() {
        let steps = snapper_install_steps();
        assert!(!steps.is_empty());
        assert!(steps.iter().any(|s| s.contains("snapper")));
    }
}
