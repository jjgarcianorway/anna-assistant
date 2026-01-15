//! Side Effect Predictor - Predict what a command will do before running it.

mod predictions;
mod types;

pub use types::{SideEffect, SideEffectType};

use predictions::*;
use std::collections::HashSet;

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
    let base_cmd = parts.first().copied().unwrap_or("");

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
        assert!(effects
            .iter()
            .any(|e| e.effect_type == SideEffectType::PackageInstall));
    }

    #[test]
    fn test_systemctl_prediction() {
        let effects = predict_side_effects(&["systemctl restart nginx".to_string()]);
        assert!(!effects.is_empty());
        assert!(effects
            .iter()
            .any(|e| e.effect_type == SideEffectType::ServiceRestart));
    }

    #[test]
    fn test_rm_prediction() {
        let effects = predict_side_effects(&["rm -rf /tmp/test".to_string()]);
        assert!(!effects.is_empty());
        assert!(effects
            .iter()
            .any(|e| e.effect_type == SideEffectType::DirDelete));
    }

    #[test]
    fn test_reboot_prediction() {
        let effects = predict_side_effects(&["reboot".to_string()]);
        assert!(!effects.is_empty());
        assert!(effects
            .iter()
            .any(|e| e.effect_type == SideEffectType::SystemReboot));
    }
}
