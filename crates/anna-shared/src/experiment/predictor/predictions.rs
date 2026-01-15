//! Individual command prediction functions.

use super::types::{SideEffect, SideEffectType};

pub fn predict_pacman_effects(cmd: &str, parts: &[&str]) -> Vec<SideEffect> {
    let mut effects = Vec::new();

    // Extract packages from command
    let packages: Vec<String> = parts
        .iter()
        .filter(|p| !p.starts_with('-') && **p != "pacman" && **p != "sudo")
        .map(|p| p.to_string())
        .collect();

    if cmd.contains("-S")
        && !cmd.contains("-Ss")
        && !cmd.contains("-Si")
        && !cmd.contains("-Sq")
    {
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

pub fn predict_aur_helper_effects(cmd: &str, parts: &[&str]) -> Vec<SideEffect> {
    // AUR helpers work similarly to pacman
    predict_pacman_effects(cmd, parts)
}

pub fn predict_systemctl_effects(cmd: &str, parts: &[&str]) -> Vec<SideEffect> {
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

pub fn predict_rm_effects(_cmd: &str, parts: &[&str]) -> Vec<SideEffect> {
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

pub fn predict_copy_move_effects(cmd: &str, parts: &[&str]) -> Vec<SideEffect> {
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

pub fn predict_mkdir_effects(parts: &[&str]) -> Vec<SideEffect> {
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

pub fn predict_touch_effects(parts: &[&str]) -> Vec<SideEffect> {
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

pub fn predict_permission_effects(_cmd: &str, parts: &[&str]) -> Vec<SideEffect> {
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

pub fn predict_mount_effects(cmd: &str, _parts: &[&str]) -> Vec<SideEffect> {
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

pub fn predict_network_effects(_cmd: &str, _parts: &[&str]) -> Vec<SideEffect> {
    vec![SideEffect {
        effect_type: SideEffectType::NetworkChange,
        targets: vec!["network".to_string()],
        confidence: 0.7,
        reversible: true,
        description: "Network configuration change".to_string(),
    }]
}

pub fn predict_firewall_effects(_cmd: &str) -> Vec<SideEffect> {
    vec![SideEffect {
        effect_type: SideEffectType::FirewallRule,
        targets: vec!["firewall".to_string()],
        confidence: 0.8,
        reversible: true,
        description: "Firewall rule change".to_string(),
    }]
}

pub fn predict_user_effects(_cmd: &str, parts: &[&str]) -> Vec<SideEffect> {
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

pub fn predict_kernel_effects(_cmd: &str, parts: &[&str]) -> Vec<SideEffect> {
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
