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
    advice.extend(check_network_manager(facts));
    advice.extend(check_firewall());

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
                title: "Install CPU security updates".to_string(),
                reason: "Your AMD processor needs microcode updates to protect against security vulnerabilities like Spectre and Meltdown. Think of it like a security patch for your CPU itself.".to_string(),
                action: "Install the amd-ucode package to keep your CPU secure".to_string(),
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
                title: "Install CPU security updates".to_string(),
                reason: "Your Intel processor needs microcode updates to protect against security vulnerabilities like Spectre and Meltdown. Think of it like a security patch for your CPU itself.".to_string(),
                action: "Install the intel-ucode package to keep your CPU secure".to_string(),
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
                        title: "Install NVIDIA graphics driver".to_string(),
                        reason: "I found an NVIDIA graphics card, but it's not using the official driver yet. Your graphics will be much faster and smoother with the proper driver installed.".to_string(),
                        action: "Install the nvidia driver for better gaming and graphics performance".to_string(),
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
                        title: "Add Vulkan support for your AMD GPU".to_string(),
                        reason: "Your AMD graphics card can run modern games and apps using Vulkan (a high-performance graphics API), but the driver isn't installed yet.".to_string(),
                        action: "Install vulkan-radeon for better gaming performance".to_string(),
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
        let msg = if count == 1 {
            "Clean up 1 unused package".to_string()
        } else {
            format!("Clean up {} unused packages", count)
        };
        result.push(Advice {
            id: "orphan-packages".to_string(),
            title: msg,
            reason: format!("You have {} {} that were installed to support other software, but nothing needs them anymore. They're just taking up space.",
                count, if count == 1 { "package" } else { "packages" }),
            action: "Remove unused packages to free up disk space".to_string(),
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
                title: "Install tools for your Btrfs filesystem".to_string(),
                reason: "You're using Btrfs for your storage, but you don't have the maintenance tools installed yet. These help keep your filesystem healthy and let you check for problems.".to_string(),
                action: "Install btrfs-progs to maintain your filesystem".to_string(),
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
                        title: "Save disk space with Btrfs compression".to_string(),
                        reason: "Btrfs can automatically compress your files as it saves them. You'll typically get 20-30% of your disk space back, and it barely uses any extra CPU power. It's almost like free storage!".to_string(),
                        action: "Enable transparent compression in /etc/fstab".to_string(),
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
                        title: "Speed up file access with noatime".to_string(),
                        reason: "Every time you read a file, Linux normally writes down when you accessed it. The 'noatime' option turns this off, making your disk faster since it doesn't need to write timestamps constantly.".to_string(),
                        action: "Add noatime to /etc/fstab for faster file operations".to_string(),
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
                title: "Check your filesystem for problems".to_string(),
                reason: "Btrfs has a built-in health check called 'scrub' that reads through all your data to make sure nothing has gotten corrupted. It's like a regular checkup for your files.".to_string(),
                action: "Run a scrub to verify your filesystem is healthy".to_string(),
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
            let msg = if update_count == 1 {
                "1 package update available".to_string()
            } else {
                format!("{} package updates available", update_count)
            };
            let package_str = if update_count == 1 {
                "1 package".to_string()
            } else {
                format!("{} packages", update_count)
            };
            result.push(Advice {
                id: "system-updates".to_string(),
                title: msg,
                reason: format!("There {} {} waiting to be updated. Updates usually include security fixes, bug fixes, and new features.",
                    if update_count == 1 { "is" } else { "are" },
                    package_str),
                action: "Update your system to stay secure and up-to-date".to_string(),
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
                title: "Keep your SSD healthy with TRIM".to_string(),
                reason: "I noticed you have a solid-state drive. SSDs need regular 'TRIM' operations to stay fast and last longer. Think of it like taking out the trash - it tells the SSD which data blocks are no longer in use.".to_string(),
                action: "Enable automatic weekly TRIM to keep your SSD running smoothly".to_string(),
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
                title: "Make pacman output colorful".to_string(),
                reason: "Right now pacman (the package manager) shows everything in plain text. Turning on colors makes it much easier to see what's being installed, updated, or removed.".to_string(),
                action: "Enable colored output in pacman".to_string(),
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
                title: "Download packages 5x faster".to_string(),
                reason: "By default, pacman downloads one package at a time. Enabling parallel downloads lets it download 5 packages simultaneously, making updates much faster.".to_string(),
                action: "Enable parallel downloads in pacman".to_string(),
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
            let msg = if failed_count == 1 {
                "1 system service has failed".to_string()
            } else {
                format!("{} system services have failed", failed_count)
            };
            let count_str = if failed_count == 1 {
                "a".to_string()
            } else {
                failed_count.to_string()
            };
            result.push(Advice {
                id: "systemd-failed".to_string(),
                title: msg,
                reason: format!("I found {} background {} that tried to start but failed. This could mean something isn't working properly on your system.",
                    count_str,
                    if failed_count == 1 { "service" } else { "services" }),
                action: "Take a look at what failed so you can fix any problems".to_string(),
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
/// Rule 9: Check for NetworkManager
fn check_network_manager(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if system has wifi
    let has_wifi = facts.network_interfaces.iter().any(|iface| iface.starts_with("wl"));

    if has_wifi {
        // Check if NetworkManager is installed
        let has_nm = Command::new("pacman")
            .args(&["-Q", "networkmanager"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_nm {
            result.push(Advice {
                id: "networkmanager".to_string(),
                title: "Install NetworkManager for easier WiFi".to_string(),
                reason: "You have a wireless card, but NetworkManager isn't installed. NetworkManager makes it super easy to connect to WiFi networks, switch between them, and manage VPNs. It's especially helpful if you use a laptop or move between different networks.".to_string(),
                action: "Install NetworkManager to simplify network management".to_string(),
                command: Some("pacman -S --noconfirm networkmanager && systemctl enable --now NetworkManager".to_string()),
                risk: RiskLevel::Medium,
                priority: Priority::Recommended,
                category: "maintenance".to_string(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/NetworkManager".to_string()],
            });
        } else {
            // Check if it's enabled
            let is_enabled = Command::new("systemctl")
                .args(&["is-enabled", "NetworkManager"])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

            if !is_enabled {
                result.push(Advice {
                    id: "networkmanager-enable".to_string(),
                    title: "Enable NetworkManager".to_string(),
                    reason: "You have NetworkManager installed, but it's not running yet. Without it running, you can't use its nice WiFi management features.".to_string(),
                    action: "Start NetworkManager so you can manage your WiFi connections".to_string(),
                    command: Some("systemctl enable --now NetworkManager".to_string()),
                    risk: RiskLevel::Low,
                    priority: Priority::Recommended,
                    category: "maintenance".to_string(),
                    wiki_refs: vec!["https://wiki.archlinux.org/title/NetworkManager".to_string()],
                });
            }
        }
    }

    result
}

/// Rule 10: Check for firewall
fn check_firewall() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if ufw is installed
    let has_ufw = Command::new("pacman")
        .args(&["-Q", "ufw"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if has_ufw {
        // Check if ufw is enabled
        let ufw_status = Command::new("ufw")
            .arg("status")
            .output();

        if let Ok(output) = ufw_status {
            let status = String::from_utf8_lossy(&output.stdout);
            if status.contains("Status: inactive") {
                result.push(Advice {
                    id: "ufw-enable".to_string(),
                    title: "Turn on your firewall".to_string(),
                    reason: "You have UFW (Uncomplicated Firewall) installed, but it's not turned on. A firewall acts like a security guard for your computer, blocking unwanted network connections while allowing the ones you trust.".to_string(),
                    action: "Enable UFW to protect your system from network threats".to_string(),
                    command: Some("ufw enable".to_string()),
                    risk: RiskLevel::Low,
                    priority: Priority::Mandatory,
                    category: "security".to_string(),
                    wiki_refs: vec!["https://wiki.archlinux.org/title/Uncomplicated_Firewall".to_string()],
                });
            }
        }
    } else {
        // Check if iptables is being actively used
        let iptables_rules = Command::new("iptables")
            .args(&["-L", "-n"])
            .output();

        if let Ok(output) = iptables_rules {
            let rules = String::from_utf8_lossy(&output.stdout);
            // If only default policies exist, no firewall is configured
            let lines: Vec<&str> = rules.lines().collect();
            if lines.len() < 10 {  // Very few rules = probably no firewall
                result.push(Advice {
                    id: "firewall-missing".to_string(),
                    title: "Set up a firewall for security".to_string(),
                    reason: "Your system doesn't have a firewall configured yet. A firewall protects you by controlling which network connections are allowed in and out of your computer. It's especially important if you connect to public WiFi or run any services.".to_string(),
                    action: "Install and configure UFW (Uncomplicated Firewall)".to_string(),
                    command: Some("pacman -S --noconfirm ufw && ufw default deny && ufw enable".to_string()),
                    risk: RiskLevel::Medium,
                    priority: Priority::Mandatory,
                    category: "security".to_string(),
                    wiki_refs: vec!["https://wiki.archlinux.org/title/Uncomplicated_Firewall".to_string()],
                });
            }
        }
    }

    result
}
