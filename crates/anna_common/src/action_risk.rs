//! Action Risk Scoring v0.0.54
//!
//! Deterministic risk scoring for actions without LLM dependence.
//! Rules are policy-driven but with sensible defaults.

use std::path::Path;
use crate::action_engine::{MutationRiskLevel, ActionType, SystemdOperation, PacmanOperation};

// =============================================================================
// Path-Based Risk Scoring
// =============================================================================

/// High-risk paths (require high confirmation)
const HIGH_RISK_PATHS: &[&str] = &[
    "/etc/fstab",
    "/etc/crypttab",
    "/boot/loader",
    "/boot/grub",
    "/boot/EFI",
    "/etc/mkinitcpio.conf",
    "/etc/mkinitcpio.d",
    "/etc/modprobe.d",
    "/etc/modules-load.d",
    "/etc/default/grub",
];

/// Destructive paths (data loss risk)
const DESTRUCTIVE_PATH_PATTERNS: &[&str] = &[
    "/dev/sd",
    "/dev/nvme",
    "/dev/mapper",
];

/// Medium-risk paths (general config)
const MEDIUM_RISK_PATH_PREFIXES: &[&str] = &[
    "/etc/",
    "/var/lib/",
];

/// Blocked paths (never allowed)
const BLOCKED_PATHS: &[&str] = &[
    "/proc",
    "/sys",
    "/dev",
    "/run",
];

/// Score risk for a file path
pub fn score_path_risk(path: &Path) -> MutationRiskLevel {
    let path_str = path.to_string_lossy();

    // Check blocked paths first
    for blocked in BLOCKED_PATHS {
        if path_str.starts_with(blocked) {
            return MutationRiskLevel::Denied;
        }
    }

    // Check destructive patterns
    for pattern in DESTRUCTIVE_PATH_PATTERNS {
        if path_str.starts_with(pattern) {
            return MutationRiskLevel::Destructive;
        }
    }

    // Check high-risk paths
    for high_risk in HIGH_RISK_PATHS {
        if path_str.starts_with(high_risk) {
            return MutationRiskLevel::High;
        }
    }

    // Check medium-risk prefixes
    for prefix in MEDIUM_RISK_PATH_PREFIXES {
        if path_str.starts_with(prefix) {
            return MutationRiskLevel::Medium;
        }
    }

    // Home directory files are low risk
    if path_str.starts_with("/home/") || path_str.starts_with("~") {
        return MutationRiskLevel::Low;
    }

    // Default to medium for unknown paths
    MutationRiskLevel::Medium
}

// =============================================================================
// Systemd Service Risk Scoring
// =============================================================================

/// Core systemd services (never touch)
const CORE_SYSTEMD_SERVICES: &[&str] = &[
    "systemd-journald",
    "systemd-logind",
    "systemd-udevd",
    "dbus",
    "dbus-broker",
    "systemd-machined",
    "init.scope",
];

/// Critical services (high risk)
const CRITICAL_SERVICES: &[&str] = &[
    "sshd",
    "ssh",
    "display-manager",
    "gdm",
    "sddm",
    "lightdm",
    "ly",
    "xdm",
    "lxdm",
    "sddm-greeter",
];

/// Network services (medium risk)
const NETWORK_SERVICES: &[&str] = &[
    "NetworkManager",
    "iwd",
    "systemd-resolved",
    "wpa_supplicant",
    "dhcpcd",
    "systemd-networkd",
    "connman",
];

/// Score risk for a systemd operation
pub fn score_systemd_risk(unit: &str, operation: SystemdOperation) -> MutationRiskLevel {
    let unit_base = unit.trim_end_matches(".service")
        .trim_end_matches(".socket")
        .trim_end_matches(".timer")
        .trim_end_matches(".target");

    // Core services are never allowed
    for core in CORE_SYSTEMD_SERVICES {
        if unit_base == *core {
            return MutationRiskLevel::Denied;
        }
    }

    // Critical services are high risk
    for critical in CRITICAL_SERVICES {
        if unit_base == *critical {
            return MutationRiskLevel::High;
        }
    }

    // Network services are medium risk
    for network in NETWORK_SERVICES {
        if unit_base == *network {
            return MutationRiskLevel::Medium;
        }
    }

    // Operation-based risk
    match operation {
        SystemdOperation::Stop | SystemdOperation::Disable | SystemdOperation::DisableNow => {
            // Stopping is riskier than starting
            MutationRiskLevel::Medium
        }
        SystemdOperation::Start | SystemdOperation::Enable | SystemdOperation::EnableNow => {
            MutationRiskLevel::Low
        }
        SystemdOperation::Restart | SystemdOperation::Reload => {
            MutationRiskLevel::Low
        }
    }
}

// =============================================================================
// Package Risk Scoring
// =============================================================================

/// Critical packages (never remove)
const CRITICAL_PACKAGES: &[&str] = &[
    "linux",
    "linux-lts",
    "linux-zen",
    "linux-hardened",
    "base",
    "base-devel",
    "systemd",
    "glibc",
    "pacman",
    "bash",
    "coreutils",
    "filesystem",
    "grub",
    "efibootmgr",
    "mkinitcpio",
];

/// High-risk packages to remove
const HIGH_RISK_PACKAGES: &[&str] = &[
    "networkmanager",
    "iwd",
    "openssh",
    "sudo",
    "doas",
    "polkit",
];

/// Score risk for a package operation
pub fn score_package_risk(packages: &[String], operation: PacmanOperation) -> MutationRiskLevel {
    match operation {
        PacmanOperation::Install => {
            // Installing is generally low risk
            MutationRiskLevel::Low
        }
        PacmanOperation::Remove => {
            // Check each package
            for pkg in packages {
                let pkg_lower = pkg.to_lowercase();

                // Critical packages cannot be removed
                for critical in CRITICAL_PACKAGES {
                    if pkg_lower == *critical {
                        return MutationRiskLevel::Denied;
                    }
                }

                // High-risk packages need explicit confirmation
                for high_risk in HIGH_RISK_PACKAGES {
                    if pkg_lower == *high_risk {
                        return MutationRiskLevel::High;
                    }
                }
            }
            MutationRiskLevel::Medium
        }
    }
}

// =============================================================================
// File Delete Risk Scoring
// =============================================================================

/// Score risk for file deletion
pub fn score_delete_risk(path: &Path) -> MutationRiskLevel {
    let path_str = path.to_string_lossy();

    // Any delete is at least high risk
    let base_risk = score_path_risk(path);

    match base_risk {
        MutationRiskLevel::Denied => MutationRiskLevel::Denied,
        MutationRiskLevel::Destructive => MutationRiskLevel::Destructive,
        _ => {
            // Deleting system files is destructive
            if path_str.starts_with("/etc/") || path_str.starts_with("/var/lib/") {
                return MutationRiskLevel::Destructive;
            }
            // Deleting home files is high risk
            MutationRiskLevel::High
        }
    }
}

// =============================================================================
// Action Risk Scoring (Unified)
// =============================================================================

/// Score risk for any action type
pub fn score_action_risk(action: &ActionType) -> MutationRiskLevel {
    match action {
        ActionType::EditFile(edit) => score_path_risk(&edit.path),
        ActionType::WriteFile(write) => {
            // New file creation is same as edit risk
            score_path_risk(&write.path)
        }
        ActionType::DeleteFile(delete) => score_delete_risk(&delete.path),
        ActionType::Systemd(systemd) => score_systemd_risk(&systemd.unit, systemd.operation),
        ActionType::Pacman(pacman) => score_package_risk(&pacman.packages, pacman.operation),
    }
}

// =============================================================================
// Risk Description
// =============================================================================

/// Get human-readable risk description
pub fn describe_risk(risk: MutationRiskLevel) -> &'static str {
    match risk {
        MutationRiskLevel::Low => "This action is low risk and easily reversible.",
        MutationRiskLevel::Medium => "This action modifies system configuration. A backup will be created.",
        MutationRiskLevel::High => "This action affects critical system components. Review carefully.",
        MutationRiskLevel::Destructive => "This action may result in data loss. Cannot be fully undone.",
        MutationRiskLevel::Denied => "This action is not allowed for safety reasons.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_path_risk_scoring() {
        assert_eq!(score_path_risk(Path::new("/home/user/.bashrc")), MutationRiskLevel::Low);
        assert_eq!(score_path_risk(Path::new("/etc/hosts")), MutationRiskLevel::Medium);
        assert_eq!(score_path_risk(Path::new("/etc/fstab")), MutationRiskLevel::High);
        assert_eq!(score_path_risk(Path::new("/proc/1/status")), MutationRiskLevel::Denied);
    }

    #[test]
    fn test_systemd_risk_scoring() {
        assert_eq!(
            score_systemd_risk("nginx.service", SystemdOperation::Restart),
            MutationRiskLevel::Low
        );
        assert_eq!(
            score_systemd_risk("NetworkManager.service", SystemdOperation::Restart),
            MutationRiskLevel::Medium
        );
        assert_eq!(
            score_systemd_risk("sshd.service", SystemdOperation::Stop),
            MutationRiskLevel::High
        );
        assert_eq!(
            score_systemd_risk("systemd-journald.service", SystemdOperation::Stop),
            MutationRiskLevel::Denied
        );
    }

    #[test]
    fn test_package_risk_scoring() {
        assert_eq!(
            score_package_risk(&["htop".to_string()], PacmanOperation::Install),
            MutationRiskLevel::Low
        );
        assert_eq!(
            score_package_risk(&["htop".to_string()], PacmanOperation::Remove),
            MutationRiskLevel::Medium
        );
        assert_eq!(
            score_package_risk(&["linux".to_string()], PacmanOperation::Remove),
            MutationRiskLevel::Denied
        );
    }
}
