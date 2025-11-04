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
    advice.extend(check_aur_helper());
    advice.extend(check_reflector());
    advice.extend(check_ssh_config());
    advice.extend(check_swap());
    advice.extend(check_shell_enhancements(facts));
    advice.extend(check_cli_tools(facts));

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
/// Rule 11: Check for AUR helper
fn check_aur_helper() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if yay is installed
    let has_yay = Command::new("pacman")
        .args(&["-Q", "yay"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    // Check if paru is installed
    let has_paru = Command::new("pacman")
        .args(&["-Q", "paru"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_yay && !has_paru {
        result.push(Advice {
            id: "aur-helper".to_string(),
            title: "Install an AUR helper to access thousands more packages".to_string(),
            reason: "The AUR (Arch User Repository) has over 85,000 community packages that aren't in the official repos. An AUR helper like 'yay' or 'paru' makes it super easy to install them - just like using pacman. Think of it as unlocking the full power of Arch!".to_string(),
            action: "Install yay to access AUR packages easily".to_string(),
            command: Some("pacman -S --needed git base-devel && git clone https://aur.archlinux.org/yay.git && cd yay && makepkg -si --noconfirm".to_string()),
            risk: RiskLevel::Medium,
            priority: Priority::Recommended,
            category: "development".to_string(),
            wiki_refs: vec!["https://wiki.archlinux.org/title/AUR_helpers".to_string()],
        });
    }

    result
}

/// Rule 12: Check for reflector (mirror optimization)
fn check_reflector() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if reflector is installed
    let has_reflector = Command::new("pacman")
        .args(&["-Q", "reflector"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_reflector {
        result.push(Advice {
            id: "reflector".to_string(),
            title: "Speed up downloads with better mirrors".to_string(),
            reason: "Reflector automatically finds the fastest Arch mirrors near you and updates your mirror list. This can make package downloads much faster - sometimes 10x faster if you're currently using a slow mirror!".to_string(),
            action: "Install reflector to optimize your mirror list".to_string(),
            command: Some("pacman -S --noconfirm reflector".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "performance".to_string(),
            wiki_refs: vec!["https://wiki.archlinux.org/title/Reflector".to_string()],
        });
    } else {
        // Check when mirrorlist was last updated
        if let Ok(metadata) = std::fs::metadata("/etc/pacman.d/mirrorlist") {
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = modified.elapsed() {
                    let days_old = elapsed.as_secs() / 86400;
                    if days_old > 30 {
                        result.push(Advice {
                            id: "reflector-update".to_string(),
                            title: format!("Your mirror list is {} days old", days_old),
                            reason: "Your mirror list hasn't been updated in over a month. Mirrors can change speed over time, and new faster ones might be available. Running reflector will find you the best mirrors right now.".to_string(),
                            action: "Update your mirror list with reflector".to_string(),
                            command: Some("reflector --latest 20 --protocol https --sort rate --save /etc/pacman.d/mirrorlist".to_string()),
                            risk: RiskLevel::Medium,
                            priority: Priority::Recommended,
                            category: "performance".to_string(),
                            wiki_refs: vec!["https://wiki.archlinux.org/title/Reflector".to_string()],
                        });
                    }
                }
            }
        }
    }

    result
}
/// Rule 13: Check SSH configuration and hardening
fn check_ssh_config() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if SSH server is installed
    let has_sshd = Command::new("pacman")
        .args(&["-Q", "openssh"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_sshd {
        // No SSH server, nothing to check
        return result;
    }

    // Check if sshd_config exists
    if let Ok(config) = std::fs::read_to_string("/etc/ssh/sshd_config") {
        // Check for root login
        let permits_root = config.lines().any(|l| {
            l.trim().starts_with("PermitRootLogin") &&
            !l.contains("no") &&
            !l.trim().starts_with("#")
        });

        if permits_root {
            result.push(Advice {
                id: "ssh-no-root-login".to_string(),
                title: "Disable direct root login via SSH".to_string(),
                reason: "Your SSH server allows direct root login, which is a security risk. If someone guesses or cracks your root password, they have complete control. It's much safer to log in as a regular user and then use 'sudo' when you need admin rights.".to_string(),
                action: "Set 'PermitRootLogin no' in /etc/ssh/sshd_config".to_string(),
                command: Some("sed -i 's/^#\\?PermitRootLogin.*/PermitRootLogin no/' /etc/ssh/sshd_config && systemctl restart sshd".to_string()),
                risk: RiskLevel::Medium,
                priority: Priority::Mandatory,
                category: "security".to_string(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/OpenSSH#Deny".to_string()],
            });
        }

        // Check for password authentication
        let password_auth = config.lines().any(|l| {
            l.trim().starts_with("PasswordAuthentication") &&
            l.contains("yes") &&
            !l.trim().starts_with("#")
        });

        if password_auth {
            // Only suggest if user has SSH keys set up
            if std::path::Path::new("/root/.ssh/authorized_keys").exists() ||
               std::path::Path::new(&format!("/home/{}/.ssh/authorized_keys",
                   std::env::var("SUDO_USER").unwrap_or_default())).exists() {
                result.push(Advice {
                    id: "ssh-key-only".to_string(),
                    title: "Use SSH keys instead of passwords".to_string(),
                    reason: "Password authentication over SSH can be brute-forced by attackers. SSH keys are much more secure - they're like having a 4096-character password that's impossible to guess. Since you already have SSH keys set up, you can safely disable password login.".to_string(),
                    action: "Disable password authentication in SSH".to_string(),
                    command: Some("sed -i 's/^#\\?PasswordAuthentication.*/PasswordAuthentication no/' /etc/ssh/sshd_config && systemctl restart sshd".to_string()),
                    risk: RiskLevel::Medium,
                    priority: Priority::Recommended,
                    category: "security".to_string(),
                    wiki_refs: vec!["https://wiki.archlinux.org/title/OpenSSH#Force_public_key_authentication".to_string()],
                });
            }
        }

        // Check for empty passwords
        let permits_empty = config.lines().any(|l| {
            l.trim().starts_with("PermitEmptyPasswords") &&
            l.contains("yes") &&
            !l.trim().starts_with("#")
        });

        if permits_empty {
            result.push(Advice {
                id: "ssh-no-empty-passwords".to_string(),
                title: "Disable empty passwords for SSH".to_string(),
                reason: "Your SSH configuration allows accounts with empty passwords to log in. This is extremely dangerous - anyone could access your system without any authentication at all!".to_string(),
                action: "Set 'PermitEmptyPasswords no' immediately".to_string(),
                command: Some("sed -i 's/^#\\?PermitEmptyPasswords.*/PermitEmptyPasswords no/' /etc/ssh/sshd_config && systemctl restart sshd".to_string()),
                risk: RiskLevel::High,
                priority: Priority::Mandatory,
                category: "security".to_string(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/OpenSSH#Deny".to_string()],
            });
        }
    }

    result
}

/// Rule 14: Check swap configuration
fn check_swap() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if swap is active
    let swap_output = Command::new("swapon")
        .arg("--show")
        .output();

    if let Ok(output) = swap_output {
        let swap_info = String::from_utf8_lossy(&output.stdout);

        if swap_info.lines().count() <= 1 {
            // No swap configured
            // Check available RAM
            let mem_output = Command::new("free")
                .args(&["-m"])
                .output();

            if let Ok(mem) = mem_output {
                let mem_info = String::from_utf8_lossy(&mem.stdout);
                if let Some(line) = mem_info.lines().nth(1) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() > 1 {
                        if let Ok(total_ram) = parts[1].parse::<u32>() {
                            if total_ram < 16000 {
                                // Less than 16GB RAM, suggest swap
                                result.push(Advice {
                                    id: "swap-create".to_string(),
                                    title: "Consider adding swap space".to_string(),
                                    reason: format!("You have {}MB of RAM and no swap configured. Swap acts like emergency memory when RAM gets full. Without it, the system might freeze or crash if you run too many programs at once. Even with modern RAM amounts, swap is useful for hibernation and as a safety net.", total_ram),
                                    action: "Create a swap file or partition".to_string(),
                                    command: Some("# Create 4GB swapfile: dd if=/dev/zero of=/swapfile bs=1M count=4096 && chmod 600 /swapfile && mkswap /swapfile && swapon /swapfile".to_string()),
                                    risk: RiskLevel::Low,
                                    priority: Priority::Recommended,
                                    category: "maintenance".to_string(),
                                    wiki_refs: vec!["https://wiki.archlinux.org/title/Swap".to_string()],
                                });
                            }
                        }
                    }
                }
            }
        } else {
            // Check for zram (compressed RAM swap)
            let has_zram = Command::new("pacman")
                .args(&["-Q", "zram-generator"])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

            if !has_zram && swap_info.contains("/swapfile") || swap_info.contains("/dev/sd") {
                result.push(Advice {
                    id: "zram-suggest".to_string(),
                    title: "Consider using zram for faster swap".to_string(),
                    reason: "You're using traditional disk-based swap. Zram creates a compressed swap area in RAM itself, which is much faster. It's especially great for systems with limited RAM or SSDs (less wear). Think of it as having more RAM by compressing less-used memory.".to_string(),
                    action: "Install zram-generator for compressed RAM swap".to_string(),
                    command: Some("pacman -S --noconfirm zram-generator".to_string()),
                    risk: RiskLevel::Low,
                    priority: Priority::Optional,
                    category: "performance".to_string(),
                    wiki_refs: vec!["https://wiki.archlinux.org/title/Zram".to_string()],
                });
            }
        }
    }

    result
}
/// Rule 15: Check for shell enhancements
fn check_shell_enhancements(_facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    // Get user's shell
    let shell = std::env::var("SHELL").unwrap_or_default();

    if shell.contains("zsh") {
        // Check for oh-my-zsh
        let omz_path = std::path::Path::new("/root/.oh-my-zsh");
        let has_omz = omz_path.exists();

        if !has_omz {
            // Check for starship
            let has_starship = Command::new("pacman")
                .args(&["-Q", "starship"])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

            if !has_starship {
                result.push(Advice {
                    id: "shell-prompt".to_string(),
                    title: "Make your terminal beautiful with Starship".to_string(),
                    reason: "You're using zsh but your prompt is probably pretty basic. Starship is a blazing-fast, customizable prompt that shows git status, programming language versions, and looks gorgeous. It's like giving your terminal a makeover!".to_string(),
                    action: "Install Starship prompt for a beautiful terminal".to_string(),
                    command: Some("pacman -S --noconfirm starship && echo 'eval \"$(starship init zsh)\"' >> ~/.zshrc".to_string()),
                    risk: RiskLevel::Low,
                    priority: Priority::Optional,
                    category: "beautification".to_string(),
                    wiki_refs: vec!["https://wiki.archlinux.org/title/Starship".to_string()],
                });
            }
        }

        // Check for zsh-autosuggestions
        let has_autosuggestions = Command::new("pacman")
            .args(&["-Q", "zsh-autosuggestions"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_autosuggestions {
            result.push(Advice {
                id: "zsh-autosuggestions".to_string(),
                title: "Get smart command suggestions in zsh".to_string(),
                reason: "As you type commands, zsh-autosuggestions will suggest completions based on your command history. It's like having autocomplete for your terminal - super helpful and saves tons of typing!".to_string(),
                action: "Install zsh-autosuggestions".to_string(),
                command: Some("pacman -S --noconfirm zsh-autosuggestions && echo 'source /usr/share/zsh/plugins/zsh-autosuggestions/zsh-autosuggestions.zsh' >> ~/.zshrc".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "beautification".to_string(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Zsh#Autosuggestions".to_string()],
            });
        }

        // Check for zsh-syntax-highlighting
        let has_highlighting = Command::new("pacman")
            .args(&["-Q", "zsh-syntax-highlighting"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_highlighting {
            result.push(Advice {
                id: "zsh-syntax-highlighting".to_string(),
                title: "Add syntax highlighting to zsh".to_string(),
                reason: "This plugin colors your commands as you type them - valid commands are green, invalid ones are red. It helps catch typos before you hit enter and makes the terminal much easier to read.".to_string(),
                action: "Install zsh-syntax-highlighting".to_string(),
                command: Some("pacman -S --noconfirm zsh-syntax-highlighting && echo 'source /usr/share/zsh/plugins/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh' >> ~/.zshrc".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "beautification".to_string(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Zsh#Syntax_highlighting".to_string()],
            });
        }
    } else if shell.contains("bash") {
        // Check for starship on bash
        let has_starship = Command::new("pacman")
            .args(&["-Q", "starship"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_starship {
            result.push(Advice {
                id: "bash-starship".to_string(),
                title: "Upgrade your bash prompt with Starship".to_string(),
                reason: "Your bash prompt is probably showing just basic info. Starship makes it beautiful and informative, showing git status, programming languages, and more. Works great with bash!".to_string(),
                action: "Install Starship prompt".to_string(),
                command: Some("pacman -S --noconfirm starship && echo 'eval \"$(starship init bash)\"' >> ~/.bashrc".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "beautification".to_string(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Starship".to_string()],
            });
        }

        // Suggest upgrading to zsh
        result.push(Advice {
            id: "upgrade-to-zsh".to_string(),
            title: "Consider upgrading from bash to zsh".to_string(),
            reason: "You're using bash, which is great! But zsh offers powerful features like better tab completion, spelling correction, and tons of plugins. Many developers make the switch and never look back. It's not required, but if you want a more powerful shell, zsh is worth trying.".to_string(),
            action: "Install zsh and try it out".to_string(),
            command: Some("pacman -S --noconfirm zsh && chsh -s /bin/zsh".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "beautification".to_string(),
            wiki_refs: vec!["https://wiki.archlinux.org/title/Zsh".to_string()],
        });
    }

    result
}

/// Rule 16: Check for modern CLI tool alternatives
fn check_cli_tools(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    // Check command history for common tools
    let commands: Vec<String> = facts.frequently_used_commands
        .iter()
        .map(|c| c.command.clone())
        .collect();

    // ls → eza
    if commands.iter().any(|c| c.starts_with("ls ") || c == "ls") {
        let has_eza = Command::new("pacman")
            .args(&["-Q", "eza"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_eza {
            result.push(Advice {
                id: "cli-eza".to_string(),
                title: "Replace 'ls' with 'eza' for beautiful file listings".to_string(),
                reason: "You use 'ls' a lot. Eza is a modern replacement with colors, icons, git integration, and tree views built-in. It's faster and much prettier than plain ls. Once you try it, you won't go back!".to_string(),
                action: "Install eza as a better 'ls'".to_string(),
                command: Some("pacman -S --noconfirm eza && echo \"alias ls='eza'\" >> ~/.zshrc".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "beautification".to_string(),
                wiki_refs: vec!["https://github.com/eza-community/eza".to_string()],
            });
        }
    }

    // cat → bat
    if commands.iter().any(|c| c.starts_with("cat ") || c == "cat") {
        let has_bat = Command::new("pacman")
            .args(&["-Q", "bat"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_bat {
            result.push(Advice {
                id: "cli-bat".to_string(),
                title: "Replace 'cat' with 'bat' for syntax-highlighted viewing".to_string(),
                reason: "You frequently use 'cat' to view files. Bat is like cat but with syntax highlighting, line numbers, and git integration. It makes reading code and config files so much easier!".to_string(),
                action: "Install bat as a better 'cat'".to_string(),
                command: Some("pacman -S --noconfirm bat && echo \"alias cat='bat'\" >> ~/.zshrc".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "beautification".to_string(),
                wiki_refs: vec!["https://github.com/sharkdp/bat".to_string()],
            });
        }
    }

    // grep → ripgrep
    if commands.iter().any(|c| c.contains("grep")) {
        let has_ripgrep = Command::new("pacman")
            .args(&["-Q", "ripgrep"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_ripgrep {
            result.push(Advice {
                id: "cli-ripgrep".to_string(),
                title: "Use 'ripgrep' for lightning-fast code searching".to_string(),
                reason: "You use grep a lot. Ripgrep (command: 'rg') is 10x-100x faster than grep, automatically skips .gitignore files, and has better defaults. It's a game-changer for searching code!".to_string(),
                action: "Install ripgrep for faster searching".to_string(),
                command: Some("pacman -S --noconfirm ripgrep".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "performance".to_string(),
                wiki_refs: vec!["https://github.com/BurntSushi/ripgrep".to_string()],
            });
        }
    }

    // find → fd
    if commands.iter().any(|c| c.starts_with("find ")) {
        let has_fd = Command::new("pacman")
            .args(&["-Q", "fd"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_fd {
            result.push(Advice {
                id: "cli-fd".to_string(),
                title: "Replace 'find' with 'fd' for easier file searching".to_string(),
                reason: "You use 'find' command. Fd is a simpler, faster alternative with intuitive syntax. Instead of 'find . -name \"*.txt\"' you just type 'fd txt'. It's also much faster and respects .gitignore by default.".to_string(),
                action: "Install fd as a better 'find'".to_string(),
                command: Some("pacman -S --noconfirm fd".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "performance".to_string(),
                wiki_refs: vec!["https://github.com/sharkdp/fd".to_string()],
            });
        }
    }

    // Check for fzf (fuzzy finder)
    let has_fzf = Command::new("pacman")
        .args(&["-Q", "fzf"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_fzf {
        result.push(Advice {
            id: "cli-fzf".to_string(),
            title: "Install 'fzf' for fuzzy finding everything".to_string(),
            reason: "Fzf is a game-changer - it adds fuzzy finding to your terminal. Search command history with Ctrl+R, find files instantly, and integrate with other tools. It's one of those tools you wonder how you lived without!".to_string(),
            action: "Install fzf for fuzzy finding".to_string(),
            command: Some("pacman -S --noconfirm fzf".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "beautification".to_string(),
            wiki_refs: vec!["https://github.com/junegunn/fzf".to_string()],
        });
    }

    result
}
