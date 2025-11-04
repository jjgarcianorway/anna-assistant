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
    advice.extend(check_trim_timer(facts));
    advice.extend(check_pacman_config());
    advice.extend(check_systemd_health());

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
                reason: "AMD CPU detected without microcode updates - critical for security patches and CPU bug fixes".to_string(),
                action: "Install amd-ucode package for CPU security and stability fixes".to_string(),
                command: Some("pacman -S --noconfirm amd-ucode".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Mandatory,
                category: "security".to_string(),
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
                reason: "Intel CPU detected without microcode updates - critical for security patches (Spectre/Meltdown) and CPU bug fixes".to_string(),
                action: "Install intel-ucode package for CPU security and stability fixes".to_string(),
                command: Some("pacman -S --noconfirm intel-ucode".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Mandatory,
                category: "security".to_string(),
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
            // Check mount options for compression
            let mount_output = Command::new("findmnt")
                .args(&["-rno", "OPTIONS", "/"])
                .output();

            if let Ok(output) = mount_output {
                let options = String::from_utf8_lossy(&output.stdout);

                // Check for compression
                if !options.contains("compress") {
                    result.push(Advice {
                        id: "btrfs-compression".to_string(),
                        title: "Enable Btrfs compression".to_string(),
                        reason: "Btrfs compression saves disk space (typically 20-30%) with minimal CPU overhead".to_string(),
                        action: "Add compress=zstd mount option to /etc/fstab for root filesystem".to_string(),
                        command: Some("# Add 'compress=zstd:3' to root mount options in /etc/fstab, then remount".to_string()),
                        risk: RiskLevel::Medium,
                        priority: Priority::Recommended,
                        category: "performance".to_string(),
                        wiki_refs: vec!["https://wiki.archlinux.org/title/Btrfs#Compression".to_string()],
                    });
                }

                // Check for noatime (performance optimization)
                if !options.contains("noatime") && !options.contains("relatime") {
                    result.push(Advice {
                        id: "btrfs-noatime".to_string(),
                        title: "Enable noatime for Btrfs".to_string(),
                        reason: "noatime improves performance by not updating access time on file reads".to_string(),
                        action: "Add noatime mount option to /etc/fstab for better I/O performance".to_string(),
                        command: Some("# Add 'noatime' to root mount options in /etc/fstab, then remount".to_string()),
                        risk: RiskLevel::Low,
                        priority: Priority::Optional,
                        category: "performance".to_string(),
                        wiki_refs: vec!["https://wiki.archlinux.org/title/Btrfs#Mount_options".to_string()],
                    });
                }
            }

            // Suggest regular scrub
            result.push(Advice {
                id: "btrfs-scrub".to_string(),
                title: "Run Btrfs scrub".to_string(),
                reason: "Regular scrubbing maintains filesystem integrity and detects silent corruption".to_string(),
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

/// Rule 6: Check TRIM timer for SSDs
fn check_trim_timer(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if any SSD is present
    let has_ssd = facts.storage_devices.iter().any(|d| {
        d.name.starts_with("/dev/sd") || d.name.starts_with("/dev/nvme")
    });

    if has_ssd {
        // Check if fstrim.timer is enabled
        let timer_status = Command::new("systemctl")
            .args(&["is-enabled", "fstrim.timer"])
            .output();

        let is_enabled = timer_status
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !is_enabled {
            result.push(Advice {
                id: "fstrim-timer".to_string(),
                title: "Enable TRIM timer for SSD".to_string(),
                reason: "SSD detected without TRIM timer - TRIM maintains SSD performance and longevity".to_string(),
                action: "Enable weekly TRIM operation for optimal SSD health".to_string(),
                command: Some("systemctl enable --now fstrim.timer".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "maintenance".to_string(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Solid_state_drive#TRIM".to_string()],
            });
        }
    }

    result
}

/// Rule 7: Check pacman configuration
fn check_pacman_config() -> Vec<Advice> {
    let mut result = Vec::new();

    // Read pacman.conf
    if let Ok(config) = std::fs::read_to_string("/etc/pacman.conf") {
        // Check for Color
        if !config.lines().any(|l| l.trim() == "Color") {
            result.push(Advice {
                id: "pacman-color".to_string(),
                title: "Enable colored output in pacman".to_string(),
                reason: "Colored pacman output makes updates easier to read and understand".to_string(),
                action: "Uncomment 'Color' in /etc/pacman.conf".to_string(),
                command: Some("sed -i 's/^#Color/Color/' /etc/pacman.conf".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "beautification".to_string(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Pacman#Enabling_color_output".to_string()],
            });
        }

        // Check for ParallelDownloads
        if !config.lines().any(|l| l.trim().starts_with("ParallelDownloads")) {
            result.push(Advice {
                id: "pacman-parallel".to_string(),
                title: "Enable parallel downloads in pacman".to_string(),
                reason: "Parallel downloads significantly speed up package installation (5x+ faster)".to_string(),
                action: "Add 'ParallelDownloads = 5' to /etc/pacman.conf".to_string(),
                command: Some("echo 'ParallelDownloads = 5' >> /etc/pacman.conf".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "performance".to_string(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Pacman#Enabling_parallel_downloads".to_string()],
            });
        }
    }

    result
}

/// Rule 8: Check systemd health
fn check_systemd_health() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for failed units
    let failed_output = Command::new("systemctl")
        .args(&["--failed", "--no-pager", "--no-legend"])
        .output();

    if let Ok(output) = failed_output {
        let failed = String::from_utf8_lossy(&output.stdout);
        let failed_count = failed.lines().filter(|l| !l.is_empty()).count();

        if failed_count > 0 {
            result.push(Advice {
                id: "systemd-failed".to_string(),
                title: format!("{} failed systemd units", failed_count),
                reason: format!("{} systemd services/timers have failed - may indicate system issues", failed_count),
                action: "Review failed units with 'systemctl --failed' and fix issues".to_string(),
                command: Some("systemctl --failed".to_string()),
                risk: RiskLevel::Medium,
                priority: Priority::Recommended,
                category: "maintenance".to_string(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Systemd#Analyzing_the_system_state".to_string()],
            });
        }
    }

    result
}
