//! Side Effect Predictor - Predict what a command will do before running it.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A predicted side effect of a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SideEffect {
    /// What type of side effect
    pub effect_type: SideEffectType,
    /// Targets (files, services, packages, etc.)
    pub targets: Vec<String>,
    /// Confidence in this prediction (0.0-1.0)
    pub confidence: f32,
    /// Is this reversible?
    pub reversible: bool,
    /// Description
    pub description: String,
}

/// Types of side effects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SideEffectType {
    /// File creation
    FileCreate,
    /// File modification
    FileModify,
    /// File deletion
    FileDelete,
    /// Directory creation
    DirCreate,
    /// Directory deletion
    DirDelete,
    /// Permission change
    PermissionChange,
    /// Package installation
    PackageInstall,
    /// Package removal
    PackageRemove,
    /// Package upgrade
    PackageUpgrade,
    /// Service start
    ServiceStart,
    /// Service stop
    ServiceStop,
    /// Service restart
    ServiceRestart,
    /// Service enable
    ServiceEnable,
    /// Service disable
    ServiceDisable,
    /// Network change
    NetworkChange,
    /// Firewall rule
    FirewallRule,
    /// Mount operation
    MountOperation,
    /// User/group change
    UserChange,
    /// Kernel module
    KernelModule,
    /// System reboot
    SystemReboot,
    /// Unknown
    Unknown,
}

impl SideEffectType {
    /// Get risk level (0.0-1.0)
    pub fn risk_level(&self) -> f32 {
        match self {
            SideEffectType::FileCreate => 0.1,
            SideEffectType::FileModify => 0.3,
            SideEffectType::FileDelete => 0.5,
            SideEffectType::DirCreate => 0.1,
            SideEffectType::DirDelete => 0.6,
            SideEffectType::PermissionChange => 0.4,
            SideEffectType::PackageInstall => 0.3,
            SideEffectType::PackageRemove => 0.5,
            SideEffectType::PackageUpgrade => 0.4,
            SideEffectType::ServiceStart => 0.2,
            SideEffectType::ServiceStop => 0.4,
            SideEffectType::ServiceRestart => 0.3,
            SideEffectType::ServiceEnable => 0.2,
            SideEffectType::ServiceDisable => 0.4,
            SideEffectType::NetworkChange => 0.5,
            SideEffectType::FirewallRule => 0.5,
            SideEffectType::MountOperation => 0.6,
            SideEffectType::UserChange => 0.5,
            SideEffectType::KernelModule => 0.7,
            SideEffectType::SystemReboot => 0.8,
            SideEffectType::Unknown => 0.5,
        }
    }

    /// Is this type generally reversible?
    pub fn is_reversible(&self) -> bool {
        match self {
            SideEffectType::FileCreate => true,
            SideEffectType::FileModify => false, // Without backup
            SideEffectType::FileDelete => false, // Without backup
            SideEffectType::DirCreate => true,
            SideEffectType::DirDelete => false,
            SideEffectType::PermissionChange => true,
            SideEffectType::PackageInstall => true,
            SideEffectType::PackageRemove => true,
            SideEffectType::PackageUpgrade => false, // Downgrade is complex
            SideEffectType::ServiceStart => true,
            SideEffectType::ServiceStop => true,
            SideEffectType::ServiceRestart => true,
            SideEffectType::ServiceEnable => true,
            SideEffectType::ServiceDisable => true,
            SideEffectType::NetworkChange => true,
            SideEffectType::FirewallRule => true,
            SideEffectType::MountOperation => true,
            SideEffectType::UserChange => true,
            SideEffectType::KernelModule => true,
            SideEffectType::SystemReboot => false,
            SideEffectType::Unknown => false,
        }
    }
}

/// Predict side effects for a list of commands
pub fn predict_side_effects(commands: &[String]) -> Vec<SideEffect> {
    let mut effects = Vec::new();

    for cmd in commands {
        effects.extend(predict_command_effects(cmd));
    }

    // Deduplicate and merge
    deduplicate_effects(effects)
}

/// Predict side effects for a single command
fn predict_command_effects(cmd: &str) -> Vec<SideEffect> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let base_cmd = parts.first().map(|s| *s).unwrap_or("");

    match base_cmd {
        "pacman" => predict_pacman_effects(cmd, &parts),
        "yay" | "paru" | "pikaur" => predict_aur_helper_effects(cmd, &parts),
        "systemctl" => predict_systemctl_effects(cmd, &parts),
        "rm" => predict_rm_effects(cmd, &parts),
        "cp" | "mv" => predict_copy_move_effects(cmd, &parts),
        "mkdir" => predict_mkdir_effects(&parts),
        "touch" => predict_touch_effects(&parts),
        "chmod" | "chown" | "chgrp" => predict_permission_effects(cmd, &parts),
        "mount" | "umount" => predict_mount_effects(cmd, &parts),
        "ip" | "nmcli" => predict_network_effects(cmd, &parts),
        "iptables" | "nft" | "firewall-cmd" => predict_firewall_effects(cmd),
        "useradd" | "usermod" | "userdel" | "groupadd" | "groupmod" | "groupdel" => {
            predict_user_effects(cmd, &parts)
        }
        "reboot" | "shutdown" | "poweroff" => {
            vec![SideEffect {
                effect_type: SideEffectType::SystemReboot,
                targets: vec!["system".to_string()],
                confidence: 1.0,
                reversible: false,
                description: "System will reboot/shutdown".to_string(),
            }]
        }
        "modprobe" | "insmod" | "rmmod" => predict_kernel_effects(cmd, &parts),
        _ => Vec::new(),
    }
}

fn predict_pacman_effects(cmd: &str, parts: &[&str]) -> Vec<SideEffect> {
    let mut effects = Vec::new();

    // Extract packages from command
    let packages: Vec<String> = parts
        .iter()
        .filter(|p| !p.starts_with('-') && **p != "pacman" && **p != "sudo")
        .map(|p| p.to_string())
        .collect();

    if cmd.contains("-S") && !cmd.contains("-Ss") && !cmd.contains("-Si") && !cmd.contains("-Sq") {
        for pkg in &packages {
            effects.push(SideEffect {
                effect_type: SideEffectType::PackageInstall,
                targets: vec![pkg.clone()],
                confidence: 0.9,
                reversible: true,
                description: format!("Install package {}", pkg),
            });
        }
    }

    if cmd.contains("-R") {
        for pkg in &packages {
            effects.push(SideEffect {
                effect_type: SideEffectType::PackageRemove,
                targets: vec![pkg.clone()],
                confidence: 0.9,
                reversible: true,
                description: format!("Remove package {}", pkg),
            });
        }
    }

    if cmd.contains("-Syu") || cmd.contains("-Syyu") {
        effects.push(SideEffect {
            effect_type: SideEffectType::PackageUpgrade,
            targets: vec!["system".to_string()],
            confidence: 0.9,
            reversible: false,
            description: "Full system upgrade".to_string(),
        });
    }

    effects
}

fn predict_aur_helper_effects(cmd: &str, parts: &[&str]) -> Vec<SideEffect> {
    // AUR helpers work similarly to pacman
    predict_pacman_effects(cmd, parts)
}

fn predict_systemctl_effects(cmd: &str, parts: &[&str]) -> Vec<SideEffect> {
    let mut effects = Vec::new();

    let services: Vec<String> = parts
        .iter()
        .filter(|p| p.ends_with(".service") || p.ends_with(".socket") || p.ends_with(".timer"))
        .map(|p| p.to_string())
        .collect();

    let service = services.first().cloned().unwrap_or_else(|| {
        // Try to find service without extension
        parts
            .iter()
            .skip(2) // skip systemctl and action
            .filter(|p| !p.starts_with('-'))
            .next()
            .map(|s| s.to_string())
            .unwrap_or_default()
    });

    if cmd.contains(" start ") || cmd.contains(" start\n") || cmd.ends_with(" start") {
        effects.push(SideEffect {
            effect_type: SideEffectType::ServiceStart,
            targets: vec![service.clone()],
            confidence: 0.9,
            reversible: true,
            description: format!("Start service {}", service),
        });
    }

    if cmd.contains(" stop ") || cmd.contains(" stop\n") || cmd.ends_with(" stop") {
        effects.push(SideEffect {
            effect_type: SideEffectType::ServiceStop,
            targets: vec![service.clone()],
            confidence: 0.9,
            reversible: true,
            description: format!("Stop service {}", service),
        });
    }

    if cmd.contains(" restart ") || cmd.contains(" restart\n") || cmd.ends_with(" restart") {
        effects.push(SideEffect {
            effect_type: SideEffectType::ServiceRestart,
            targets: vec![service.clone()],
            confidence: 0.9,
            reversible: true,
            description: format!("Restart service {}", service),
        });
    }

    if cmd.contains(" enable ") || cmd.contains(" enable\n") || cmd.ends_with(" enable") {
        effects.push(SideEffect {
            effect_type: SideEffectType::ServiceEnable,
            targets: vec![service.clone()],
            confidence: 0.9,
            reversible: true,
            description: format!("Enable service {}", service),
        });
    }

    if cmd.contains(" disable ") || cmd.contains(" disable\n") || cmd.ends_with(" disable") {
        effects.push(SideEffect {
            effect_type: SideEffectType::ServiceDisable,
            targets: vec![service.clone()],
            confidence: 0.9,
            reversible: true,
            description: format!("Disable service {}", service),
        });
    }

    effects
}

fn predict_rm_effects(_cmd: &str, parts: &[&str]) -> Vec<SideEffect> {
    let mut effects = Vec::new();
    let is_recursive = parts.iter().any(|p| p.contains('r'));
    let is_force = parts.iter().any(|p| p.contains('f'));

    let targets: Vec<String> = parts
        .iter()
        .filter(|p| !p.starts_with('-') && **p != "rm" && **p != "sudo")
        .map(|p| p.to_string())
        .collect();

    for target in &targets {
        let effect_type = if is_recursive && (target.ends_with('/') || !target.contains('.')) {
            SideEffectType::DirDelete
        } else {
            SideEffectType::FileDelete
        };

        effects.push(SideEffect {
            effect_type,
            targets: vec![target.clone()],
            confidence: if is_force { 0.95 } else { 0.85 },
            reversible: false,
            description: format!("Delete {}", target),
        });
    }

    effects
}

fn predict_copy_move_effects(cmd: &str, parts: &[&str]) -> Vec<SideEffect> {
    let mut effects = Vec::new();
    let is_move = cmd.starts_with("mv") || parts.first() == Some(&"mv");

    let targets: Vec<String> = parts
        .iter()
        .filter(|p| !p.starts_with('-') && **p != "cp" && **p != "mv" && **p != "sudo")
        .map(|p| p.to_string())
        .collect();

    if targets.len() >= 2 {
        let dest = targets.last().unwrap();

        effects.push(SideEffect {
            effect_type: SideEffectType::FileCreate,
            targets: vec![dest.clone()],
            confidence: 0.9,
            reversible: true,
            description: format!("Create/overwrite {}", dest),
        });

        if is_move {
            for src in targets.iter().take(targets.len() - 1) {
                effects.push(SideEffect {
                    effect_type: SideEffectType::FileDelete,
                    targets: vec![src.clone()],
                    confidence: 0.9,
                    reversible: false,
                    description: format!("Remove source {}", src),
                });
            }
        }
    }

    effects
}

fn predict_mkdir_effects(parts: &[&str]) -> Vec<SideEffect> {
    let targets: Vec<String> = parts
        .iter()
        .filter(|p| !p.starts_with('-') && **p != "mkdir" && **p != "sudo")
        .map(|p| p.to_string())
        .collect();

    targets
        .into_iter()
        .map(|t| SideEffect {
            effect_type: SideEffectType::DirCreate,
            targets: vec![t.clone()],
            confidence: 0.95,
            reversible: true,
            description: format!("Create directory {}", t),
        })
        .collect()
}

fn predict_touch_effects(parts: &[&str]) -> Vec<SideEffect> {
    let targets: Vec<String> = parts
        .iter()
        .filter(|p| !p.starts_with('-') && **p != "touch" && **p != "sudo")
        .map(|p| p.to_string())
        .collect();

    targets
        .into_iter()
        .map(|t| SideEffect {
            effect_type: SideEffectType::FileCreate,
            targets: vec![t.clone()],
            confidence: 0.95,
            reversible: true,
            description: format!("Create/update timestamp {}", t),
        })
        .collect()
}

fn predict_permission_effects(cmd: &str, parts: &[&str]) -> Vec<SideEffect> {
    let targets: Vec<String> = parts
        .iter()
        .filter(|p| {
            !p.starts_with('-')
                && **p != "chmod"
                && **p != "chown"
                && **p != "chgrp"
                && **p != "sudo"
                && !p.chars().all(|c| c.is_ascii_digit() || "+-rwx".contains(c))
        })
        .map(|p| p.to_string())
        .collect();

    targets
        .into_iter()
        .map(|t| SideEffect {
            effect_type: SideEffectType::PermissionChange,
            targets: vec![t.clone()],
            confidence: 0.9,
            reversible: true,
            description: format!("Change permissions on {}", t),
        })
        .collect()
}

fn predict_mount_effects(cmd: &str, _parts: &[&str]) -> Vec<SideEffect> {
    vec![SideEffect {
        effect_type: SideEffectType::MountOperation,
        targets: vec!["filesystem".to_string()],
        confidence: 0.8,
        reversible: true,
        description: if cmd.contains("umount") {
            "Unmount filesystem".to_string()
        } else {
            "Mount filesystem".to_string()
        },
    }]
}

fn predict_network_effects(_cmd: &str, _parts: &[&str]) -> Vec<SideEffect> {
    vec![SideEffect {
        effect_type: SideEffectType::NetworkChange,
        targets: vec!["network".to_string()],
        confidence: 0.7,
        reversible: true,
        description: "Network configuration change".to_string(),
    }]
}

fn predict_firewall_effects(_cmd: &str) -> Vec<SideEffect> {
    vec![SideEffect {
        effect_type: SideEffectType::FirewallRule,
        targets: vec!["firewall".to_string()],
        confidence: 0.8,
        reversible: true,
        description: "Firewall rule change".to_string(),
    }]
}

fn predict_user_effects(_cmd: &str, parts: &[&str]) -> Vec<SideEffect> {
    let users: Vec<String> = parts
        .iter()
        .filter(|p| !p.starts_with('-') && !p.contains('='))
        .skip(1)
        .map(|p| p.to_string())
        .collect();

    users
        .into_iter()
        .map(|u| SideEffect {
            effect_type: SideEffectType::UserChange,
            targets: vec![u.clone()],
            confidence: 0.9,
            reversible: true,
            description: format!("User/group change: {}", u),
        })
        .collect()
}

fn predict_kernel_effects(_cmd: &str, parts: &[&str]) -> Vec<SideEffect> {
    let modules: Vec<String> = parts
        .iter()
        .filter(|p| !p.starts_with('-'))
        .skip(1)
        .map(|p| p.to_string())
        .collect();

    modules
        .into_iter()
        .map(|m| SideEffect {
            effect_type: SideEffectType::KernelModule,
            targets: vec![m.clone()],
            confidence: 0.9,
            reversible: true,
            description: format!("Kernel module operation: {}", m),
        })
        .collect()
}

/// Deduplicate and merge similar effects
fn deduplicate_effects(effects: Vec<SideEffect>) -> Vec<SideEffect> {
    let mut seen: HashSet<(String, String)> = HashSet::new();
    let mut result = Vec::new();

    for effect in effects {
        let key = (
            format!("{:?}", effect.effect_type),
            effect.targets.join(","),
        );
        if !seen.contains(&key) {
            seen.insert(key);
            result.push(effect);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pacman_prediction() {
        let effects = predict_side_effects(&["pacman -S vim".to_string()]);
        assert!(!effects.is_empty());
        assert!(effects.iter().any(|e| e.effect_type == SideEffectType::PackageInstall));
    }

    #[test]
    fn test_systemctl_prediction() {
        let effects = predict_side_effects(&["systemctl restart nginx".to_string()]);
        assert!(!effects.is_empty());
        assert!(effects.iter().any(|e| e.effect_type == SideEffectType::ServiceRestart));
    }

    #[test]
    fn test_rm_prediction() {
        let effects = predict_side_effects(&["rm -rf /tmp/test".to_string()]);
        assert!(!effects.is_empty());
        assert!(effects.iter().any(|e| e.effect_type == SideEffectType::DirDelete));
    }

    #[test]
    fn test_reboot_prediction() {
        let effects = predict_side_effects(&["reboot".to_string()]);
        assert!(!effects.is_empty());
        assert!(effects.iter().any(|e| e.effect_type == SideEffectType::SystemReboot));
    }
}
