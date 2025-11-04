//! Recommendation engine
//!
//! Analyzes system facts and generates actionable advice with Arch Wiki citations.

use anna_common::{Advice, Priority, RiskLevel, SystemFacts};
use std::process::Command;

/// Generate advice based on system facts
pub fn generate_advice(facts: &SystemFacts) -> Vec<Advice> {
    let mut advice = Vec::new();

    advice.extend(check_microcode(facts));
    advice.extend(check_gpu_drivers(facts));
    advice.extend(check_orphan_packages(facts));
    advice.extend(check_btrfs_maintenance(facts));
    advice.extend(check_system_updates());

    advice
}

/// Rule 1: Check for microcode installation
fn check_microcode(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    // Detect CPU vendor
    let is_amd = facts.cpu_model.to_lowercase().contains("amd");
    let is_intel = facts.cpu_model.to_lowercase().contains("intel");

    if is_amd {
        // Check if amd-ucode is installed
        let installed = Command::new("pacman")
            .args(&["-Q", "amd-ucode"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !installed {
            result.push(Advice {
                id: "microcode-amd".to_string(),
                title: "Install AMD microcode".to_string(),
                reason: "AMD CPU detected without microcode updates installed".to_string(),
                action: "Install amd-ucode package for CPU security and stability fixes".to_string(),
                command: Some("pacman -S --noconfirm amd-ucode".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "maintenance".to_string(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Microcode".to_string()],
            });
        }
    } else if is_intel {
        let installed = Command::new("pacman")
            .args(&["-Q", "intel-ucode"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !installed {
            result.push(Advice {
                id: "microcode-intel".to_string(),
                title: "Install Intel microcode".to_string(),
                reason: "Intel CPU detected without microcode updates installed".to_string(),
                action: "Install intel-ucode package for CPU security and stability fixes".to_string(),
                command: Some("pacman -S --noconfirm intel-ucode".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "maintenance".to_string(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Microcode".to_string()],
            });
        }
    }

    result
}

/// Rule 2: Check GPU drivers
fn check_gpu_drivers(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    if let Some(ref gpu) = facts.gpu_vendor {
        match gpu.as_str() {
            "NVIDIA" => {
                // Check for nvidia driver
                let has_driver = Command::new("pacman")
                    .args(&["-Q", "nvidia"])
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false);

                if !has_driver {
                    result.push(Advice {
                        id: "nvidia-driver".to_string(),
                        title: "Install NVIDIA proprietary driver".to_string(),
                        reason: "NVIDIA GPU detected without proprietary driver".to_string(),
                        action: "Install nvidia package for optimal graphics performance".to_string(),
                        command: Some("pacman -S --noconfirm nvidia nvidia-utils".to_string()),
                        risk: RiskLevel::Medium,
                priority: Priority::Recommended,
                category: "maintenance".to_string(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/NVIDIA".to_string()],
                    });
                }
            }
            "AMD" => {
                let has_vulkan = Command::new("pacman")
                    .args(&["-Q", "vulkan-radeon"])
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false);

                if !has_vulkan {
                    result.push(Advice {
                        id: "amd-vulkan".to_string(),
                        title: "Install AMD Vulkan driver".to_string(),
                        reason: "AMD GPU detected without Vulkan support".to_string(),
                        action: "Install vulkan-radeon for modern graphics API support".to_string(),
                        command: Some("pacman -S --noconfirm vulkan-radeon".to_string()),
                        risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "maintenance".to_string(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/AMDGPU".to_string()],
                    });
                }
            }
            _ => {}
        }
    }

    result
}

/// Rule 3: Check for orphaned packages
fn check_orphan_packages(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    if !facts.orphan_packages.is_empty() {
        let count = facts.orphan_packages.len();
        result.push(Advice {
            id: "orphan-packages".to_string(),
            title: format!("Remove {} orphaned packages", count),
            reason: format!("{} packages were installed as dependencies but are no longer needed", count),
            action: "Remove orphaned packages to free disk space and reduce clutter".to_string(),
            command: Some("pacman -Rns --noconfirm $(pacman -Qdtq)".to_string()),
            risk: RiskLevel::Medium,
                priority: Priority::Recommended,
                category: "maintenance".to_string(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Pacman/Tips_and_tricks".to_string()],
        });
    }

    result
}

/// Rule 4: Check for Btrfs maintenance
fn check_btrfs_maintenance(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if any filesystem is btrfs
    let has_btrfs = facts.storage_devices.iter().any(|d| d.filesystem == "btrfs");

    if has_btrfs {
        // Check if btrfs-progs is installed
        let has_progs = Command::new("pacman")
            .args(&["-Q", "btrfs-progs"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_progs {
            result.push(Advice {
                id: "btrfs-progs".to_string(),
                title: "Install Btrfs utilities".to_string(),
                reason: "Btrfs filesystem detected without maintenance tools".to_string(),
                action: "Install btrfs-progs for filesystem maintenance and health checks".to_string(),
                command: Some("pacman -S --noconfirm btrfs-progs".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "maintenance".to_string(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Btrfs".to_string()],
            });
        } else {
            // Suggest regular scrub
            result.push(Advice {
                id: "btrfs-scrub".to_string(),
                title: "Run Btrfs scrub".to_string(),
                reason: "Regular scrubbing maintains filesystem integrity".to_string(),
                action: "Run a Btrfs scrub to detect and fix data corruption".to_string(),
                command: Some("btrfs scrub start /".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "maintenance".to_string(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Btrfs#Scrub".to_string()],
            });
        }
    }

    result
}

/// Rule 5: Check for system updates
fn check_system_updates() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if updates are available
    let output = Command::new("pacman")
        .args(&["-Qu"])
        .output();

    if let Ok(output) = output {
        let updates = String::from_utf8_lossy(&output.stdout);
        let update_count = updates.lines().count();

        if update_count > 0 {
            result.push(Advice {
                id: "system-updates".to_string(),
                title: format!("{} system updates available", update_count),
                reason: format!("{} packages have updates available", update_count),
                action: "Update system to get latest features and security fixes".to_string(),
                command: Some("pacman -Syu --noconfirm".to_string()),
                risk: RiskLevel::Medium,
                priority: Priority::Recommended,
                category: "maintenance".to_string(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/System_maintenance".to_string()],
            });
        }
    }

    result
}
