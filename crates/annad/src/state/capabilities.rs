//! Command capabilities per system state
//!
//! Defines which commands are available in each state.
//! This is the authoritative mapping for state-aware command filtering.
//!
//! Citation: [archwiki:installation_guide], [archwiki:system_maintenance]

use super::types::SystemState;
use serde::{Deserialize, Serialize};

/// Command capability definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandCapability {
    /// Command name (e.g., "install", "update")
    pub name: String,

    /// Human-readable description
    pub description: String,

    /// Version when command was introduced
    pub since: String,

    /// Arch Wiki citation for this command
    pub citation: String,

    /// Whether command requires root privileges
    pub requires_root: bool,
}

/// Get all available commands for a given state
///
/// This is the single source of truth for command availability.
pub fn get_capabilities(state: SystemState) -> Vec<CommandCapability> {
    match state {
        SystemState::IsoLive => iso_live_commands(),
        SystemState::RecoveryCandidate => recovery_commands(),
        SystemState::PostInstallMinimal => post_install_commands(),
        SystemState::Configured => configured_commands(),
        SystemState::Degraded => degraded_commands(),
        SystemState::Unknown => unknown_commands(),
    }
}

/// Commands available in iso_live state
///
/// Citation: [archwiki:installation_guide]
fn iso_live_commands() -> Vec<CommandCapability> {
    vec![
        CommandCapability {
            name: "install".to_string(),
            description: "Interactive Arch Linux installation".to_string(),
            since: "1.0.0".to_string(),
            citation: "[archwiki:installation_guide]".to_string(),
            requires_root: true,
        },
        CommandCapability {
            name: "hardware-report".to_string(),
            description: "Show hardware detection results".to_string(),
            since: "1.0.0".to_string(),
            citation: "[archwiki:hwinfo]".to_string(),
            requires_root: false,
        },
        CommandCapability {
            name: "rescue-detect".to_string(),
            description: "Scan for installed Linux roots".to_string(),
            since: "1.0.0".to_string(),
            citation: "[archwiki:chroot]".to_string(),
            requires_root: false,
        },
        CommandCapability {
            name: "rescue-chroot".to_string(),
            description: "Mount and chroot into installed system".to_string(),
            since: "1.0.0".to_string(),
            citation: "[archwiki:chroot]".to_string(),
            requires_root: true,
        },
    ]
}

/// Commands available in recovery_candidate state
///
/// Citation: [archwiki:chroot], [archwiki:system_maintenance]
fn recovery_commands() -> Vec<CommandCapability> {
    vec![
        CommandCapability {
            name: "rescue-overview".to_string(),
            description: "Show detected issues and repair options".to_string(),
            since: "1.0.0".to_string(),
            citation: "[archwiki:system_maintenance]".to_string(),
            requires_root: false,
        },
        CommandCapability {
            name: "rescue-chroot".to_string(),
            description: "Mount and chroot into installed system".to_string(),
            since: "1.0.0".to_string(),
            citation: "[archwiki:chroot]".to_string(),
            requires_root: true,
        },
        CommandCapability {
            name: "boot-repair".to_string(),
            description: "Fix bootloader configuration".to_string(),
            since: "1.0.0".to_string(),
            citation: "[archwiki:systemd-boot]".to_string(),
            requires_root: true,
        },
        CommandCapability {
            name: "fs-check".to_string(),
            description: "Check and repair filesystem".to_string(),
            since: "1.0.0".to_string(),
            citation: "[archwiki:fsck]".to_string(),
            requires_root: true,
        },
        CommandCapability {
            name: "hardware-report".to_string(),
            description: "Show hardware detection results".to_string(),
            since: "1.0.0".to_string(),
            citation: "[archwiki:hwinfo]".to_string(),
            requires_root: false,
        },
    ]
}

/// Commands available in post_install_minimal state
///
/// Citation: [archwiki:installation_guide:configure_the_system]
fn post_install_commands() -> Vec<CommandCapability> {
    vec![
        CommandCapability {
            name: "converge".to_string(),
            description: "Initialize Anna management and adopt system".to_string(),
            since: "1.0.0".to_string(),
            citation: "[archwiki:system_maintenance]".to_string(),
            requires_root: true,
        },
        CommandCapability {
            name: "adopt-state".to_string(),
            description: "Create Anna state file for this system".to_string(),
            since: "1.0.0".to_string(),
            citation: "[archwiki:system_maintenance]".to_string(),
            requires_root: true,
        },
        CommandCapability {
            name: "health".to_string(),
            description: "Check system health status".to_string(),
            since: "1.0.0".to_string(),
            citation: "[archwiki:system_maintenance#check_for_errors]".to_string(),
            requires_root: false,
        },
        CommandCapability {
            name: "status".to_string(),
            description: "Show current system state".to_string(),
            since: "1.0.0".to_string(),
            citation: "[archwiki:system_maintenance]".to_string(),
            requires_root: false,
        },
        CommandCapability {
            name: "hardware-report".to_string(),
            description: "Show hardware detection results".to_string(),
            since: "1.0.0".to_string(),
            citation: "[archwiki:hwinfo]".to_string(),
            requires_root: false,
        },
    ]
}

/// Commands available in configured state
///
/// Citation: [archwiki:system_maintenance]
fn configured_commands() -> Vec<CommandCapability> {
    vec![
        CommandCapability {
            name: "update".to_string(),
            description: "Update system packages with safety checks".to_string(),
            since: "1.0.0".to_string(),
            citation: "[archwiki:system_maintenance#upgrading_the_system]".to_string(),
            requires_root: true,
        },
        CommandCapability {
            name: "backup".to_string(),
            description: "Create system backup".to_string(),
            since: "1.0.0".to_string(),
            citation: "[archwiki:system_backup]".to_string(),
            requires_root: true,
        },
        CommandCapability {
            name: "health".to_string(),
            description: "Check system health status".to_string(),
            since: "1.0.0".to_string(),
            citation: "[archwiki:system_maintenance#check_for_errors]".to_string(),
            requires_root: false,
        },
        CommandCapability {
            name: "advise".to_string(),
            description: "Get system administration recommendations".to_string(),
            since: "1.0.0".to_string(),
            citation: "[archwiki:system_maintenance]".to_string(),
            requires_root: false,
        },
        CommandCapability {
            name: "apply".to_string(),
            description: "Apply recommendation by ID".to_string(),
            since: "1.0.0".to_string(),
            citation: "[archwiki:system_maintenance]".to_string(),
            requires_root: true,
        },
        CommandCapability {
            name: "suggest".to_string(),
            description: "Get recommendations by category".to_string(),
            since: "1.0.0".to_string(),
            citation: "[archwiki:system_maintenance]".to_string(),
            requires_root: false,
        },
        CommandCapability {
            name: "approve".to_string(),
            description: "Approve recommendation for later application".to_string(),
            since: "1.0.0".to_string(),
            citation: "[archwiki:system_maintenance]".to_string(),
            requires_root: false,
        },
        CommandCapability {
            name: "revert".to_string(),
            description: "Revert previously applied recommendation".to_string(),
            since: "1.0.0".to_string(),
            citation: "[archwiki:system_maintenance]".to_string(),
            requires_root: true,
        },
        CommandCapability {
            name: "status".to_string(),
            description: "Show current system state".to_string(),
            since: "1.0.0".to_string(),
            citation: "[archwiki:system_maintenance]".to_string(),
            requires_root: false,
        },
        CommandCapability {
            name: "history".to_string(),
            description: "Show command history".to_string(),
            since: "1.0.0".to_string(),
            citation: "[archwiki:system_maintenance]".to_string(),
            requires_root: false,
        },
        CommandCapability {
            name: "doctor".to_string(),
            description: "Diagnose and fix common issues".to_string(),
            since: "1.0.0".to_string(),
            citation: "[archwiki:system_maintenance]".to_string(),
            requires_root: true,
        },
        CommandCapability {
            name: "hardware-report".to_string(),
            description: "Show hardware detection results".to_string(),
            since: "1.0.0".to_string(),
            citation: "[archwiki:hwinfo]".to_string(),
            requires_root: false,
        },
    ]
}

/// Commands available in degraded state
///
/// Citation: [archwiki:system_maintenance#check_for_errors]
fn degraded_commands() -> Vec<CommandCapability> {
    vec![
        CommandCapability {
            name: "triage".to_string(),
            description: "Analyze system health issues".to_string(),
            since: "1.0.0".to_string(),
            citation: "[archwiki:system_maintenance#check_for_errors]".to_string(),
            requires_root: false,
        },
        CommandCapability {
            name: "rollback-last".to_string(),
            description: "Rollback last system change".to_string(),
            since: "1.0.0".to_string(),
            citation: "[archwiki:system_maintenance]".to_string(),
            requires_root: true,
        },
        CommandCapability {
            name: "collect-logs".to_string(),
            description: "Collect diagnostic logs".to_string(),
            since: "1.0.0".to_string(),
            citation: "[archwiki:system_maintenance]".to_string(),
            requires_root: true,
        },
        CommandCapability {
            name: "revert".to_string(),
            description: "Revert previously applied recommendation".to_string(),
            since: "1.0.0".to_string(),
            citation: "[archwiki:system_maintenance]".to_string(),
            requires_root: true,
        },
        CommandCapability {
            name: "status".to_string(),
            description: "Show current system state".to_string(),
            since: "1.0.0".to_string(),
            citation: "[archwiki:system_maintenance]".to_string(),
            requires_root: false,
        },
        CommandCapability {
            name: "history".to_string(),
            description: "Show command history".to_string(),
            since: "1.0.0".to_string(),
            citation: "[archwiki:system_maintenance]".to_string(),
            requires_root: false,
        },
        CommandCapability {
            name: "health".to_string(),
            description: "Check system health status".to_string(),
            since: "1.0.0".to_string(),
            citation: "[archwiki:system_maintenance#check_for_errors]".to_string(),
            requires_root: false,
        },
        CommandCapability {
            name: "hardware-report".to_string(),
            description: "Show hardware detection results".to_string(),
            since: "1.0.0".to_string(),
            citation: "[archwiki:hwinfo]".to_string(),
            requires_root: false,
        },
    ]
}

/// Commands available in unknown state
///
/// Citation: [archwiki:installation_guide]
fn unknown_commands() -> Vec<CommandCapability> {
    vec![
        CommandCapability {
            name: "discover".to_string(),
            description: "Attempt to classify system state".to_string(),
            since: "1.0.0".to_string(),
            citation: "[archwiki:installation_guide]".to_string(),
            requires_root: false,
        },
        CommandCapability {
            name: "hardware-report".to_string(),
            description: "Show hardware detection results".to_string(),
            since: "1.0.0".to_string(),
            citation: "[archwiki:hwinfo]".to_string(),
            requires_root: false,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_states_have_capabilities() {
        let states = vec![
            SystemState::IsoLive,
            SystemState::RecoveryCandidate,
            SystemState::PostInstallMinimal,
            SystemState::Configured,
            SystemState::Degraded,
            SystemState::Unknown,
        ];

        for state in states {
            let caps = get_capabilities(state);
            assert!(!caps.is_empty(), "State {:?} has no capabilities", state);

            // All capabilities must have citations
            for cap in caps {
                assert!(
                    !cap.citation.is_empty(),
                    "Command {} has no citation",
                    cap.name
                );
                assert!(
                    cap.citation.starts_with("[archwiki:") || cap.citation.starts_with("[man:"),
                    "Command {} has invalid citation: {}",
                    cap.name,
                    cap.citation
                );
            }
        }
    }

    #[test]
    fn test_iso_live_has_install() {
        let caps = get_capabilities(SystemState::IsoLive);
        assert!(caps.iter().any(|c| c.name == "install"));
    }

    #[test]
    fn test_configured_has_update() {
        let caps = get_capabilities(SystemState::Configured);
        assert!(caps.iter().any(|c| c.name == "update"));
    }

    #[test]
    fn test_unknown_limited_commands() {
        let caps = get_capabilities(SystemState::Unknown);
        assert!(
            caps.len() <= 3,
            "Unknown state should have minimal commands"
        );
    }

    #[test]
    fn test_all_commands_have_since_version() {
        let states = vec![SystemState::IsoLive, SystemState::Configured];

        for state in states {
            let caps = get_capabilities(state);
            for cap in caps {
                assert!(!cap.since.is_empty(), "Command {} has no version", cap.name);
            }
        }
    }
}
