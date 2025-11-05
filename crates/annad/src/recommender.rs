//! Recommendation engine
//!
//! Analyzes system facts and generates actionable advice with Arch Wiki citations.

use anna_common::{Advice, Alternative, Priority, RiskLevel, SystemFacts};
use std::path::Path;
use std::process::Command;

/// Helper function to check command usage in shell history
fn check_command_usage(commands: &[&str]) -> usize {
    let mut count = 0;

    // Try to read bash history
    if let Ok(home) = std::env::var("HOME") {
        let bash_history = Path::new(&home).join(".bash_history");
        if let Ok(contents) = std::fs::read_to_string(bash_history) {
            for cmd in commands {
                count += contents.lines().filter(|line| line.contains(cmd)).count();
            }
        }

        // Also try zsh history
        let zsh_history = Path::new(&home).join(".zsh_history");
        if let Ok(contents) = std::fs::read_to_string(zsh_history) {
            for cmd in commands {
                count += contents.lines().filter(|line| line.contains(cmd)).count();
            }
        }
    }

    count
}

/// Generate advice based on system facts
pub fn generate_advice(facts: &SystemFacts) -> Vec<Advice> {
    let mut advice = Vec::new();

    // Enhanced telemetry-based checks (beta.35+)
    advice.extend(check_cpu_temperature(facts));
    advice.extend(check_disk_health(facts));
    advice.extend(check_journal_errors(facts));
    advice.extend(check_degraded_services(facts));
    advice.extend(check_memory_pressure(facts));
    advice.extend(check_battery_health(facts));
    advice.extend(check_service_crashes(facts));
    advice.extend(check_kernel_errors(facts));
    advice.extend(check_disk_space_prediction(facts));

    // Environment-specific recommendations (beta.39+)
    advice.extend(check_hyprland_nvidia_config(facts));
    advice.extend(check_wayland_nvidia_config(facts));
    advice.extend(check_window_manager_recommendations(facts));
    advice.extend(check_desktop_environment_specific(facts));

    advice.extend(check_microcode(facts));
    advice.extend(check_gpu_drivers(facts));
    advice.extend(check_intel_gpu_support(facts));
    advice.extend(check_amd_gpu_enhancements(facts));
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
    advice.extend(check_gaming_setup());
    advice.extend(check_desktop_environment(facts));
    advice.extend(check_terminal_and_fonts());
    advice.extend(check_firmware_tools());
    advice.extend(check_media_tools());
    advice.extend(check_audio_system());
    advice.extend(check_power_management());
    advice.extend(check_gamepad_support());
    advice.extend(check_usb_automount());
    advice.extend(check_bluetooth());
    advice.extend(check_wifi_setup());
    advice.extend(check_snapshot_systems(facts));
    advice.extend(check_docker_support(facts));
    advice.extend(check_virtualization_support(facts));
    advice.extend(check_printer_support());
    advice.extend(check_archive_tools());
    advice.extend(check_monitoring_tools());
    advice.extend(check_firmware_updates());
    advice.extend(check_ssd_optimizations(facts));
    advice.extend(check_swap_compression());
    advice.extend(check_dns_configuration());
    advice.extend(check_journal_size());
    advice.extend(check_aur_helper_safety());
    advice.extend(check_locale_timezone());
    advice.extend(check_laptop_optimizations(facts));
    advice.extend(check_webcam_support());
    advice.extend(check_audio_enhancements());
    advice.extend(check_shell_productivity());
    advice.extend(check_filesystem_maintenance(facts));
    advice.extend(check_kernel_parameters());
    advice.extend(check_bootloader_optimization());
    advice.extend(check_vpn_tools());
    advice.extend(check_browser_recommendations());
    advice.extend(check_security_tools());
    advice.extend(check_backup_solutions());
    advice.extend(check_screen_recording());
    advice.extend(check_password_managers());
    advice.extend(check_gaming_enhancements());
    advice.extend(check_android_integration());
    advice.extend(check_text_editors());
    advice.extend(check_mail_clients());
    advice.extend(check_file_sharing());
    advice.extend(check_cloud_storage());
    advice.extend(check_golang_dev());
    advice.extend(check_java_dev());
    advice.extend(check_nodejs_dev());
    advice.extend(check_databases());
    advice.extend(check_web_servers());
    advice.extend(check_remote_desktop());
    advice.extend(check_torrent_clients());
    advice.extend(check_office_suite());
    advice.extend(check_graphics_software());
    advice.extend(check_video_editing());
    advice.extend(check_music_players());
    advice.extend(check_pdf_readers());
    advice.extend(check_monitor_tools());
    advice.extend(check_systemd_timers());
    advice.extend(check_shell_alternatives());
    advice.extend(check_compression_advanced());
    advice.extend(check_dual_boot());
    advice.extend(check_git_advanced());
    advice.extend(check_container_alternatives());
    advice.extend(check_code_editors());
    advice.extend(check_additional_databases());
    advice.extend(check_network_tools());
    advice.extend(check_dotfile_managers());
    advice.extend(check_pkgbuild_tools());
    advice.extend(check_python_tools());
    advice.extend(check_rust_tools());
    advice.extend(check_terminal_multiplexers());
    advice.extend(check_image_viewers());
    advice.extend(check_documentation_tools());
    advice.extend(check_disk_management());
    advice.extend(check_communication_apps());
    advice.extend(check_scientific_tools());
    advice.extend(check_3d_tools());
    advice.extend(check_audio_production());
    advice.extend(check_system_monitoring_advanced());
    advice.extend(check_cad_software());
    advice.extend(check_markdown_tools());
    advice.extend(check_note_taking());

    // NEW: Intelligent behavior-based recommendations
    advice.extend(check_container_orchestration());
    advice.extend(check_python_enhancements());
    advice.extend(check_git_workflow_tools());

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
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Microcode".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
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
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Microcode".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
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
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/NVIDIA".to_string()],
                                    depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
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
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/AMDGPU".to_string()],
                                    depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
                }
            }
            _ => {}
        }
    }

    result
}

/// Rule 2b: Check Intel GPU support (beta.41+)
fn check_intel_gpu_support(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    if !facts.is_intel_gpu {
        return result;
    }

    // Check for Vulkan support
    let has_vulkan = Command::new("pacman")
        .args(&["-Q", "vulkan-intel"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_vulkan {
        result.push(Advice {
            id: "intel-vulkan".to_string(),
            title: "Add Vulkan support for your Intel GPU".to_string(),
            reason: "Your Intel integrated graphics can run modern games and applications using Vulkan (a high-performance graphics API). This improves gaming performance and enables many modern graphical applications.".to_string(),
            action: "Install vulkan-intel for better graphics performance".to_string(),
            command: Some("pacman -S --noconfirm vulkan-intel".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "hardware".to_string(),
            alternatives: Vec::new(),
            wiki_refs: vec!["https://wiki.archlinux.org/title/Intel_graphics".to_string()],
            depends_on: Vec::new(),
            related_to: Vec::new(),
            bundle: None,
        });
    }

    // Check for hardware video acceleration (modern Intel)
    let has_modern_vaapi = Command::new("pacman")
        .args(&["-Q", "intel-media-driver"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    // Check for hardware video acceleration (legacy Intel)
    let has_legacy_vaapi = Command::new("pacman")
        .args(&["-Q", "libva-intel-driver"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_modern_vaapi && !has_legacy_vaapi {
        result.push(Advice {
            id: "intel-video-accel".to_string(),
            title: "Enable hardware video acceleration for Intel GPU".to_string(),
            reason: "Your Intel GPU can decode videos using hardware acceleration, which saves battery life and reduces CPU usage. intel-media-driver is for modern Intel GPUs (Broadwell and newer), while libva-intel-driver supports older models.".to_string(),
            action: "Install hardware video acceleration drivers".to_string(),
            command: Some("pacman -S --noconfirm intel-media-driver libva-intel-driver".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "hardware".to_string(),
            alternatives: Vec::new(),
            wiki_refs: vec!["https://wiki.archlinux.org/title/Hardware_video_acceleration".to_string()],
            depends_on: Vec::new(),
            related_to: Vec::new(),
            bundle: None,
        });
    }

    result
}

/// Rule 2c: Check AMD GPU enhancements (beta.41+)
fn check_amd_gpu_enhancements(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    if !facts.is_amd_gpu {
        return result;
    }

    // Check if using legacy radeon driver and suggest upgrading
    if let Some(ref driver) = facts.amd_driver_version {
        if driver.contains("radeon (legacy)") {
            result.push(Advice {
                id: "amd-driver-upgrade".to_string(),
                title: "Consider upgrading to modern AMD driver (amdgpu)".to_string(),
                reason: "You're using the legacy 'radeon' driver. The modern 'amdgpu' driver offers better performance, power management, and supports newer features. It works with GCN 1.2+ GPUs (R9 285 and newer). Check the Arch Wiki to see if your card is compatible.".to_string(),
                action: "Review compatibility and consider switching to amdgpu".to_string(),
                command: None, // No automatic command - requires research
                risk: RiskLevel::Medium,
                priority: Priority::Optional,
                category: "hardware".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec![
                    "https://wiki.archlinux.org/title/AMDGPU".to_string(),
                    "https://wiki.archlinux.org/title/ATI".to_string(),
                ],
                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    // Check for hardware video acceleration
    let has_vaapi = Command::new("pacman")
        .args(&["-Q", "libva-mesa-driver"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_vaapi {
        result.push(Advice {
            id: "amd-video-accel".to_string(),
            title: "Enable hardware video acceleration for AMD GPU".to_string(),
            reason: "Your AMD GPU can decode videos using hardware acceleration, which dramatically reduces CPU usage and power consumption when watching videos. This makes streaming smoother and extends battery life on laptops.".to_string(),
            action: "Install Mesa VA-API driver for hardware video decoding".to_string(),
            command: Some("pacman -S --noconfirm libva-mesa-driver mesa-vdpau".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "hardware".to_string(),
            alternatives: Vec::new(),
            wiki_refs: vec!["https://wiki.archlinux.org/title/Hardware_video_acceleration".to_string()],
            depends_on: Vec::new(),
            related_to: Vec::new(),
            bundle: None,
        });
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
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Pacman/Tips_and_tricks".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
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
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Btrfs".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
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
                        alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Btrfs#Compression".to_string()],
                                    depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
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
                        alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Btrfs#Mount_options".to_string()],
                                    depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
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
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Btrfs#Scrub".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
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
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/System_maintenance".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
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
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Solid_state_drive#TRIM".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
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
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Pacman#Enabling_color_output".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
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
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Pacman#Enabling_parallel_downloads".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
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
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Systemd#Analyzing_the_system_state".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
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
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/NetworkManager".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
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
                    alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/NetworkManager".to_string()],
                                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
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
                    alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Uncomplicated_Firewall".to_string()],
                                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
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
                    alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Uncomplicated_Firewall".to_string()],
                                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
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
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/AUR_helpers".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
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
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Reflector".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
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
                            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Reflector".to_string()],
                                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
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
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/OpenSSH#Deny".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
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
                    alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/OpenSSH#Force_public_key_authentication".to_string()],
                                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
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
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/OpenSSH#Deny".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }

        // Check for Protocol version (SSH-1 is insecure)
        let allows_protocol_1 = config.lines().any(|l| {
            l.trim().starts_with("Protocol") &&
            l.contains("1") &&
            !l.trim().starts_with("#")
        });

        if allows_protocol_1 {
            result.push(Advice {
                id: "ssh-protocol-2-only".to_string(),
                title: "Use SSH Protocol 2 only".to_string(),
                reason: "SSH Protocol 1 has known security vulnerabilities and should never be used. Protocol 2 has been the standard since 2006 and is much more secure with better encryption. Modern OpenSSH versions default to Protocol 2 only, but your config explicitly allows Protocol 1.".to_string(),
                action: "Remove Protocol 1 support from SSH config".to_string(),
                command: Some("sed -i 's/^#\\?Protocol.*/Protocol 2/' /etc/ssh/sshd_config && systemctl restart sshd".to_string()),
                risk: RiskLevel::High,
                priority: Priority::Mandatory,
                category: "security".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/OpenSSH#Configuration".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }

        // Check for X11 forwarding (security risk if not needed)
        let x11_forwarding = config.lines().any(|l| {
            l.trim().starts_with("X11Forwarding") &&
            l.contains("yes") &&
            !l.trim().starts_with("#")
        });

        if x11_forwarding {
            result.push(Advice {
                id: "ssh-disable-x11-forwarding".to_string(),
                title: "Consider disabling X11 forwarding in SSH".to_string(),
                reason: "X11 forwarding allows remote systems to interact with your X server, which can be a security risk if compromised. Unless you specifically need to run graphical applications over SSH, it's safer to disable this feature.".to_string(),
                action: "Set 'X11Forwarding no' in SSH config".to_string(),
                command: Some("sed -i 's/^#\\?X11Forwarding.*/X11Forwarding no/' /etc/ssh/sshd_config && systemctl restart sshd".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "security".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/OpenSSH#X11_forwarding".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }

        // Check for MaxAuthTries (limit brute force attempts)
        let has_max_auth_tries = config.lines().any(|l| {
            l.trim().starts_with("MaxAuthTries") &&
            !l.trim().starts_with("#")
        });

        if !has_max_auth_tries {
            result.push(Advice {
                id: "ssh-max-auth-tries".to_string(),
                title: "Limit SSH authentication attempts".to_string(),
                reason: "Setting MaxAuthTries limits how many password attempts someone can make before being disconnected. This slows down brute-force attacks. The default is 6, but setting it to 3 is more secure - legitimate users rarely need more than 3 tries!".to_string(),
                action: "Add 'MaxAuthTries 3' to SSH config".to_string(),
                command: Some("echo 'MaxAuthTries 3' >> /etc/ssh/sshd_config && systemctl restart sshd".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "security".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/OpenSSH#Protection".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }

        // Check for ClientAliveInterval (detect dead connections)
        let has_client_alive = config.lines().any(|l| {
            l.trim().starts_with("ClientAliveInterval") &&
            !l.trim().starts_with("#")
        });

        if !has_client_alive {
            result.push(Advice {
                id: "ssh-client-alive-interval".to_string(),
                title: "Configure SSH connection timeouts".to_string(),
                reason: "ClientAliveInterval makes the server send keepalive messages to detect dead connections. This prevents abandoned SSH sessions from staying open forever, which is both a security and resource management issue. Setting it to 300 seconds (5 minutes) is a good balance.".to_string(),
                action: "Add connection timeout settings to SSH".to_string(),
                command: Some("echo -e 'ClientAliveInterval 300\\nClientAliveCountMax 2' >> /etc/ssh/sshd_config && systemctl restart sshd".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "security".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/OpenSSH#Keep_alive".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }

        // Check for AllowUsers/AllowGroups (whitelist approach)
        let has_allow_users = config.lines().any(|l| {
            (l.trim().starts_with("AllowUsers") || l.trim().starts_with("AllowGroups")) &&
            !l.trim().starts_with("#")
        });

        if !has_allow_users {
            result.push(Advice {
                id: "ssh-allowusers-consideration".to_string(),
                title: "Consider using AllowUsers for SSH access control".to_string(),
                reason: "Instead of letting any user SSH in, you can whitelist specific users with 'AllowUsers username'. This is the most secure approach - even if someone creates a new user account on your system, they won't be able to SSH in unless you explicitly allow them. Great for single-user or small team systems!".to_string(),
                action: "Review if you should add 'AllowUsers' directive".to_string(),
                command: None, // Manual review needed
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "security".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/OpenSSH#Deny".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }

        // Check for default SSH port (22)
        let uses_default_port = !config.lines().any(|l| {
            l.trim().starts_with("Port") &&
            !l.contains("22") &&
            !l.trim().starts_with("#")
        });

        if uses_default_port {
            result.push(Advice {
                id: "ssh-non-default-port".to_string(),
                title: "Consider changing SSH to non-default port".to_string(),
                reason: "Running SSH on port 22 (the default) means your server gets hammered by automated bot attacks 24/7. Changing to a non-standard port (like 2222 or 22222) drastically reduces these attacks. It's 'security through obscurity' but it works surprisingly well for reducing noise and log spam! Just make sure you remember the new port.".to_string(),
                action: "Consider changing SSH port from 22 to something else".to_string(),
                command: None, // Manual decision needed
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "security".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/OpenSSH#Protection".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
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
                                    alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Swap".to_string()],
                                                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
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
                    alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Zram".to_string()],
                                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
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
                    alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Starship".to_string()],
                                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
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
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Zsh#Autosuggestions".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
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
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Zsh#Syntax_highlighting".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
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
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Starship".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
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
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Zsh".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
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
                alternatives: Vec::new(),
                wiki_refs: vec!["https://github.com/eza-community/eza".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
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
                alternatives: Vec::new(),
                wiki_refs: vec!["https://github.com/sharkdp/bat".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
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
                alternatives: Vec::new(),
                wiki_refs: vec!["https://github.com/BurntSushi/ripgrep".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
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
                alternatives: Vec::new(),
                wiki_refs: vec!["https://github.com/sharkdp/fd".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
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
            alternatives: Vec::new(),
                wiki_refs: vec!["https://github.com/junegunn/fzf".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    result
}

/// Rule 17: Check for gaming setup
fn check_gaming_setup() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if Steam is installed
    let has_steam = Command::new("pacman")
        .args(&["-Q", "steam"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if has_steam {
        // Check if multilib repository is enabled
        if let Ok(pacman_conf) = std::fs::read_to_string("/etc/pacman.conf") {
            let multilib_enabled = pacman_conf.lines().any(|l| {
                l.trim() == "[multilib]" && !l.trim().starts_with("#")
            });

            if !multilib_enabled {
                result.push(Advice {
                    id: "multilib-repo".to_string(),
                    title: "Enable multilib repository for gaming".to_string(),
                    reason: "You have Steam installed, but the multilib repository isn't enabled. Many games need 32-bit libraries (lib32) to run properly. Without multilib, some games just won't work!".to_string(),
                    action: "Enable the multilib repository in pacman.conf".to_string(),
                    command: Some("sed -i '/\\[multilib\\]/,/Include/s/^#//' /etc/pacman.conf && pacman -Sy".to_string()),
                    risk: RiskLevel::Low,
                    priority: Priority::Recommended,
                    category: "gaming".to_string(),
                    alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Official_repositories#multilib".to_string()],
                                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
            }
        }

        // Check for gamemode
        let has_gamemode = Command::new("pacman")
            .args(&["-Q", "gamemode"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_gamemode {
            result.push(Advice {
                id: "gamemode".to_string(),
                title: "Install GameMode for better gaming performance".to_string(),
                reason: "GameMode temporarily optimizes your system for gaming by adjusting CPU governor, I/O priority, and other settings. It can give you a noticeable FPS boost in games. Most modern games support it automatically!".to_string(),
                action: "Install gamemode to optimize gaming performance".to_string(),
                command: Some("pacman -S --noconfirm gamemode lib32-gamemode".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "gaming".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Gamemode".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }

        // Check for mangohud
        let has_mangohud = Command::new("pacman")
            .args(&["-Q", "mangohud"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_mangohud {
            result.push(Advice {
                id: "mangohud".to_string(),
                title: "Install MangoHud for in-game performance overlay".to_string(),
                reason: "MangoHud shows FPS, CPU/GPU usage, temperatures, and more right in your games. It's super helpful for monitoring performance and looks really cool! Works with Vulkan and OpenGL games.".to_string(),
                action: "Install mangohud for gaming metrics overlay".to_string(),
                command: Some("pacman -S --noconfirm mangohud lib32-mangohud".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "gaming".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/MangoHud".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }

        // Check for gamescope (Steam Deck compositor)
        let has_gamescope = Command::new("pacman")
            .args(&["-Q", "gamescope"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_gamescope {
            result.push(Advice {
                id: "gamescope".to_string(),
                title: "Install Gamescope for better game compatibility".to_string(),
                reason: "Gamescope is the compositor used by Steam Deck. It can help with games that have resolution or window mode issues, and lets you run games in a contained environment with custom resolutions and upscaling.".to_string(),
                action: "Install gamescope for advanced gaming features".to_string(),
                command: Some("pacman -S --noconfirm gamescope".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "gaming".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Gamescope".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    // Check for Lutris (GOG, Epic, etc.)
    let has_lutris = Command::new("pacman")
        .args(&["-Q", "lutris"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if has_steam && !has_lutris {
        result.push(Advice {
            id: "lutris".to_string(),
            title: "Install Lutris for non-Steam games".to_string(),
            reason: "You have Steam, but if you also play games from GOG, Epic Games Store, or want to run Windows games outside of Steam, Lutris makes it super easy. It handles Wine configuration and has installers for tons of games.".to_string(),
            action: "Install Lutris for managing all your games".to_string(),
            command: Some("pacman -S --noconfirm lutris wine-staging".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "gaming".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Lutris".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    result
}

/// Rule 18: Check desktop environment and display server setup
fn check_desktop_environment(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    // Use desktop environment from SystemFacts (more reliable than env vars when running as root)
    let desktop_env = facts.desktop_environment.as_ref()
        .map(|de| de.to_lowercase())
        .unwrap_or_default();

    // Detect display server from SystemFacts
    let display_server = facts.display_server.as_ref()
        .map(|ds| ds.to_lowercase())
        .unwrap_or_default();
    let wayland_session = display_server.contains("wayland");

    // Check for GNOME
    if desktop_env.contains("gnome") {
        // GNOME-specific recommendations
        let has_extension_manager = Command::new("pacman")
            .args(&["-Q", "gnome-shell-extensions"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_extension_manager {
            result.push(Advice {
                id: "gnome-extensions".to_string(),
                title: "Install GNOME Extensions support".to_string(),
                reason: "GNOME Extensions let you customize your desktop with features like system monitors, clipboard managers, and window tiling. They're like apps for your desktop environment - you can add the features you want!".to_string(),
                action: "Install GNOME Shell extensions support".to_string(),
                command: Some("pacman -S --noconfirm gnome-shell-extensions".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "desktop".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/GNOME#Extensions".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }

        // Check for GNOME Tweaks
        let has_tweaks = Command::new("pacman")
            .args(&["-Q", "gnome-tweaks"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_tweaks {
            result.push(Advice {
                id: "gnome-tweaks".to_string(),
                title: "Install GNOME Tweaks for more customization".to_string(),
                reason: "GNOME Tweaks gives you access to tons of settings that aren't in the default settings app. Change fonts, themes, window behavior, and more. It's essential for making GNOME truly yours!".to_string(),
                action: "Install GNOME Tweaks".to_string(),
                command: Some("pacman -S --noconfirm gnome-tweaks".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "desktop".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/GNOME#Customization".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    // Check for KDE Plasma
    if desktop_env.contains("kde") || desktop_env.contains("plasma") {
        // Check for KDE applications
        let has_dolphin = Command::new("pacman")
            .args(&["-Q", "dolphin"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_dolphin {
            result.push(Advice {
                id: "kde-dolphin".to_string(),
                title: "Install Dolphin file manager".to_string(),
                reason: "Dolphin is KDE's powerful file manager. It has tabs, split views, terminal integration, and tons of features. If you're using KDE, Dolphin makes file management a breeze!".to_string(),
                action: "Install Dolphin file manager".to_string(),
                command: Some("pacman -S --noconfirm dolphin".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "desktop".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/KDE#Dolphin".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }

        // Check for Konsole
        let has_konsole = Command::new("pacman")
            .args(&["-Q", "konsole"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_konsole {
            result.push(Advice {
                id: "kde-konsole".to_string(),
                title: "Install Konsole terminal emulator".to_string(),
                reason: "Konsole is KDE's feature-rich terminal. It integrates beautifully with Plasma, supports tabs and splits, and has great customization options. Perfect for KDE users!".to_string(),
                action: "Install Konsole".to_string(),
                command: Some("pacman -S --noconfirm konsole".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "desktop".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Konsole".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    // Check for i3 window manager
    let has_i3 = Command::new("pacman")
        .args(&["-Q", "i3-wm"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false) || desktop_env.contains("i3");

    if has_i3 {
        // Check for i3status
        let has_i3status = Command::new("pacman")
            .args(&["-Q", "i3status"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_i3status {
            result.push(Advice {
                id: "i3-status".to_string(),
                title: "Install i3status or alternative status bar".to_string(),
                reason: "i3 doesn't show system info by default. i3status gives you a status bar with battery, network, time, and more. Or try i3blocks or polybar for even more customization!".to_string(),
                action: "Install i3status for system information".to_string(),
                command: Some("pacman -S --noconfirm i3status".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "desktop".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/I3#i3status".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }

        // Check for rofi (application launcher)
        let has_rofi = Command::new("pacman")
            .args(&["-Q", "rofi"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_rofi {
            result.push(Advice {
                id: "i3-rofi".to_string(),
                title: "Install Rofi application launcher".to_string(),
                reason: "Rofi is a beautiful, fast app launcher that's way better than dmenu. It can launch apps, switch windows, and even run custom scripts. It's a must-have for i3 users!".to_string(),
                action: "Install Rofi".to_string(),
                command: Some("pacman -S --noconfirm rofi".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "desktop".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Rofi".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    // Check for Hyprland (Wayland compositor)
    let has_hyprland = Command::new("pacman")
        .args(&["-Q", "hyprland"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false) || desktop_env.contains("hyprland");

    if has_hyprland {
        // Check for waybar
        let has_waybar = Command::new("pacman")
            .args(&["-Q", "waybar"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_waybar {
            result.push(Advice {
                id: "hyprland-waybar".to_string(),
                title: "Install a status bar for Hyprland".to_string(),
                reason: "A status bar shows workspaces, system info, network status, and more. You have several great options to choose from!".to_string(),
                action: "Install Waybar (recommended) or choose an alternative".to_string(),
                command: Some("pacman -S --noconfirm waybar".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "desktop".to_string(),
                alternatives: vec![
                    Alternative {
                        name: "Waybar".to_string(),
                        description: "Most popular, highly customizable with JSON/CSS config, excellent Wayland support".to_string(),
                        install_command: "pacman -S --noconfirm waybar".to_string(),
                    },
                    Alternative {
                        name: "eww".to_string(),
                        description: "Widget system with custom Lisp-like language, extremely flexible, perfect for unique setups".to_string(),
                        install_command: "yay -S --noconfirm eww".to_string(),
                    },
                    Alternative {
                        name: "yambar".to_string(),
                        description: "Lightweight, minimal, YAML-based config, great for performance-focused builds".to_string(),
                        install_command: "pacman -S --noconfirm yambar".to_string(),
                    },
                ],
                wiki_refs: vec!["https://wiki.archlinux.org/title/Waybar".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }

        // Check for wofi (Wayland rofi alternative)
        let has_wofi = Command::new("pacman")
            .args(&["-Q", "wofi"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_wofi {
            result.push(Advice {
                id: "hyprland-wofi".to_string(),
                title: "Install an application launcher for Wayland".to_string(),
                reason: "An app launcher lets you quickly find and launch applications with a keystroke - way faster than hunting through menus!".to_string(),
                action: "Install Wofi (recommended) or choose an alternative".to_string(),
                command: Some("pacman -S --noconfirm wofi".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "desktop".to_string(),
                alternatives: vec![
                    Alternative {
                        name: "Wofi".to_string(),
                        description: "Simple, fast, native Wayland launcher with customizable CSS styling".to_string(),
                        install_command: "pacman -S --noconfirm wofi".to_string(),
                    },
                    Alternative {
                        name: "Rofi (Wayland fork)".to_string(),
                        description: "The classic, feature-rich launcher with plugins, now with Wayland support".to_string(),
                        install_command: "yay -S --noconfirm rofi-lbonn-wayland-git".to_string(),
                    },
                    Alternative {
                        name: "Fuzzel".to_string(),
                        description: "Minimal, keyboard-driven launcher inspired by dmenu, very lightweight".to_string(),
                        install_command: "pacman -S --noconfirm fuzzel".to_string(),
                    },
                ],
                wiki_refs: vec!["https://wiki.archlinux.org/title/Wofi".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }

        // Check for mako (notification daemon)
        let has_mako = Command::new("pacman")
            .args(&["-Q", "mako"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_mako {
            result.push(Advice {
                id: "hyprland-mako".to_string(),
                title: "Install a notification daemon for Wayland".to_string(),
                reason: "Wayland compositors need a notification daemon to show desktop notifications (battery warnings, app alerts, etc.).".to_string(),
                action: "Install Mako (recommended) or choose an alternative".to_string(),
                command: Some("pacman -S --noconfirm mako".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "desktop".to_string(),
                alternatives: vec![
                    Alternative {
                        name: "Mako".to_string(),
                        description: "Lightweight, minimal, perfect for Wayland, simple config".to_string(),
                        install_command: "pacman -S --noconfirm mako".to_string(),
                    },
                    Alternative {
                        name: "Dunst".to_string(),
                        description: "Highly customizable, works on both X11 and Wayland, rich configuration options".to_string(),
                        install_command: "pacman -S --noconfirm dunst".to_string(),
                    },
                    Alternative {
                        name: "SwayNC".to_string(),
                        description: "Notification center with history, control center UI, feature-rich".to_string(),
                        install_command: "yay -S --noconfirm swaync".to_string(),
                    },
                ],
                wiki_refs: vec!["https://wiki.archlinux.org/title/Desktop_notifications#Mako".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    // Check for Sway (i3-compatible Wayland compositor)
    let has_sway = Command::new("pacman")
        .args(&["-Q", "sway"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false) || desktop_env.contains("sway");

    if has_sway {
        // Waybar for sway too
        let has_waybar = Command::new("pacman")
            .args(&["-Q", "waybar"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_waybar {
            result.push(Advice {
                id: "sway-waybar".to_string(),
                title: "Install Waybar for Sway".to_string(),
                reason: "Sway works great with Waybar for a status bar. It's highly customizable and shows all your system info beautifully. Way better than the default swaybar!".to_string(),
                action: "Install Waybar".to_string(),
                command: Some("pacman -S --noconfirm waybar".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "desktop".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Sway#Waybar".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }

        // Wofi for sway
        let has_wofi = Command::new("pacman")
            .args(&["-Q", "wofi"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_wofi {
            result.push(Advice {
                id: "sway-wofi".to_string(),
                title: "Install Wofi launcher for Sway".to_string(),
                reason: "Wofi is perfect for Sway - it's a Wayland-native app launcher that integrates beautifully. Much more modern than dmenu!".to_string(),
                action: "Install Wofi".to_string(),
                command: Some("pacman -S --noconfirm wofi".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "desktop".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Sway#Application_launchers".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    // Display server recommendations
    if wayland_session {
        // Check for XWayland (for running X11 apps on Wayland)
        let has_xwayland = Command::new("pacman")
            .args(&["-Q", "xorg-xwayland"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_xwayland {
            result.push(Advice {
                id: "xwayland".to_string(),
                title: "Install XWayland for X11 app compatibility".to_string(),
                reason: "You're running Wayland, but many apps still use X11. XWayland lets you run those X11 apps on your Wayland session - best of both worlds! Without it, some apps might not work at all.".to_string(),
                action: "Install XWayland".to_string(),
                command: Some("pacman -S --noconfirm xorg-xwayland".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "desktop".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Wayland#XWayland".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    // Check for Cinnamon desktop environment
    if desktop_env.contains("cinnamon") {
        // Check for Nemo file manager (Cinnamon's default)
        let has_nemo = Command::new("pacman")
            .args(&["-Q", "nemo"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_nemo {
            result.push(Advice {
                id: "cinnamon-nemo".to_string(),
                title: "Install Nemo file manager".to_string(),
                reason: "Nemo is Cinnamon's official file manager. It's a fork of Nautilus with extra features like dual pane view, better customization, and Cinnamon-specific integrations. Essential for the full Cinnamon experience!".to_string(),
                action: "Install Nemo file manager".to_string(),
                command: Some("pacman -S --noconfirm nemo".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "desktop".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Cinnamon#File_manager".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }

        // Check for GNOME Terminal (commonly used with Cinnamon)
        let has_gnome_terminal = Command::new("pacman")
            .args(&["-Q", "gnome-terminal"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_gnome_terminal {
            result.push(Advice {
                id: "cinnamon-terminal".to_string(),
                title: "Install GNOME Terminal for Cinnamon".to_string(),
                reason: "GNOME Terminal is the recommended terminal for Cinnamon. It integrates well with the desktop, supports tabs, profiles, and has good keyboard shortcut support.".to_string(),
                action: "Install GNOME Terminal".to_string(),
                command: Some("pacman -S --noconfirm gnome-terminal".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "desktop".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/GNOME/Tips_and_tricks#Terminal".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }

        // Check for Cinnamon screensaver
        let has_screensaver = Command::new("pacman")
            .args(&["-Q", "cinnamon-screensaver"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_screensaver {
            result.push(Advice {
                id: "cinnamon-screensaver".to_string(),
                title: "Install Cinnamon screensaver".to_string(),
                reason: "Cinnamon's screensaver provides screen locking and power saving features. It's important for security (locks your screen when you're away) and extends your monitor's life.".to_string(),
                action: "Install Cinnamon screensaver".to_string(),
                command: Some("pacman -S --noconfirm cinnamon-screensaver".to_string()),
                risk: RiskLevel::Medium,
                priority: Priority::Recommended,
                category: "desktop".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Cinnamon".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    // Check for XFCE desktop environment
    if desktop_env.contains("xfce") {
        // Check for Thunar file manager
        let has_thunar = Command::new("pacman")
            .args(&["-Q", "thunar"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_thunar {
            result.push(Advice {
                id: "xfce-thunar".to_string(),
                title: "Install Thunar file manager".to_string(),
                reason: "Thunar is XFCE's official file manager. It's fast, lightweight, and has great plugin support for bulk renaming, custom actions, and archive management. Perfect for XFCE's philosophy of being light but powerful!".to_string(),
                action: "Install Thunar file manager".to_string(),
                command: Some("pacman -S --noconfirm thunar".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "desktop".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Thunar".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }

        // Check for xfce4-terminal
        let has_xfce_terminal = Command::new("pacman")
            .args(&["-Q", "xfce4-terminal"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_xfce_terminal {
            result.push(Advice {
                id: "xfce-terminal".to_string(),
                title: "Install xfce4-terminal".to_string(),
                reason: "xfce4-terminal is XFCE's native terminal emulator. It's lightweight, has dropdown mode support, and integrates perfectly with XFCE. Much better than using xterm!".to_string(),
                action: "Install xfce4-terminal".to_string(),
                command: Some("pacman -S --noconfirm xfce4-terminal".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "desktop".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Xfce#Terminal".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }

        // Check for xfce4-goodies (collection of plugins and extras)
        let has_goodies = Command::new("pacman")
            .args(&["-Q", "xfce4-goodies"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_goodies {
            result.push(Advice {
                id: "xfce-goodies".to_string(),
                title: "Install xfce4-goodies collection".to_string(),
                reason: "XFCE Goodies includes tons of useful plugins: panel plugins for weather, system monitoring, CPU graphs, battery indicators, and more. Think of it as the 'complete XFCE experience' package that makes your desktop actually useful!".to_string(),
                action: "Install xfce4-goodies".to_string(),
                command: Some("pacman -S --noconfirm xfce4-goodies".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "desktop".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Xfce#Extras".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    // Check for MATE desktop environment
    if desktop_env.contains("mate") {
        // Check for Caja file manager
        let has_caja = Command::new("pacman")
            .args(&["-Q", "caja"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_caja {
            result.push(Advice {
                id: "mate-caja".to_string(),
                title: "Install Caja file manager".to_string(),
                reason: "Caja is MATE's official file manager (a fork of the classic GNOME 2 Nautilus). It's reliable, well-tested, and has all the features you need: tabs, bookmarks, extensions, and spatial mode. It's the authentic MATE experience!".to_string(),
                action: "Install Caja file manager".to_string(),
                command: Some("pacman -S --noconfirm caja".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "desktop".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/MATE#File_manager".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }

        // Check for MATE Terminal
        let has_mate_terminal = Command::new("pacman")
            .args(&["-Q", "mate-terminal"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_mate_terminal {
            result.push(Advice {
                id: "mate-terminal".to_string(),
                title: "Install MATE Terminal".to_string(),
                reason: "MATE Terminal is the official terminal for MATE desktop. It's based on the classic GNOME 2 terminal with modern improvements. Supports tabs, profiles, transparency, and integrates perfectly with MATE.".to_string(),
                action: "Install MATE Terminal".to_string(),
                command: Some("pacman -S --noconfirm mate-terminal".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "desktop".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/MATE".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }

        // Check for MATE utilities
        let has_mate_utils = Command::new("pacman")
            .args(&["-Q", "mate-utils"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_mate_utils {
            result.push(Advice {
                id: "mate-utils".to_string(),
                title: "Install MATE utilities collection".to_string(),
                reason: "MATE Utils includes essential desktop tools: screenshot utility, search tool, dictionary, system log viewer, and disk usage analyzer. These are the 'everyday tools' that make MATE actually productive!".to_string(),
                action: "Install MATE utilities".to_string(),
                command: Some("pacman -S --noconfirm mate-utils".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "desktop".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/MATE".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    result
}

/// Rule 19: Check terminal emulators and font rendering
fn check_terminal_and_fonts() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for modern terminal emulators
    let has_alacritty = Command::new("pacman")
        .args(&["-Q", "alacritty"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let has_kitty = Command::new("pacman")
        .args(&["-Q", "kitty"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let has_wezterm = Command::new("pacman")
        .args(&["-Q", "wezterm"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    // Only suggest if they don't have any modern terminal
    if !has_alacritty && !has_kitty && !has_wezterm {
        result.push(Advice {
            id: "modern-terminal".to_string(),
            title: "Upgrade to a GPU-accelerated terminal emulator".to_string(),
            reason: "Modern terminals use your GPU for rendering, making them incredibly fast and smooth. They support true color, ligatures, and can handle massive outputs without lag.".to_string(),
            action: "Install Alacritty (recommended) or choose an alternative".to_string(),
            command: Some("pacman -S --noconfirm alacritty".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "beautification".to_string(),
            alternatives: vec![
                Alternative {
                    name: "Alacritty".to_string(),
                    description: "Blazingly fast, minimal config (TOML), extremely lightweight, best performance".to_string(),
                    install_command: "pacman -S --noconfirm alacritty".to_string(),
                },
                Alternative {
                    name: "Kitty".to_string(),
                    description: "Feature-rich with tabs, splits, images support, Lua scripting, great for power users".to_string(),
                    install_command: "pacman -S --noconfirm kitty".to_string(),
                },
                Alternative {
                    name: "WezTerm".to_string(),
                    description: "Modern with multiplexing, cross-platform, extensive Lua config, built-in tabs".to_string(),
                    install_command: "pacman -S --noconfirm wezterm".to_string(),
                },
            ],
                wiki_refs: vec!["https://wiki.archlinux.org/title/List_of_applications#Terminal_emulators".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    // Check for nerd fonts (for powerline, starship, etc.)
    let has_nerd_fonts = Command::new("pacman")
        .args(&["-Ss", "ttf-nerd-fonts"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.contains("[installed]"))
        .unwrap_or(false);

    if !has_nerd_fonts {
        result.push(Advice {
            id: "nerd-fonts".to_string(),
            title: "Install Nerd Fonts for beautiful terminal icons".to_string(),
            reason: "Nerd Fonts include thousands of glyphs and icons that make your terminal look amazing. They're essential for Starship prompt, file managers like lsd/eza, and many terminal apps. Without them, you'll see broken squares instead of cool icons!".to_string(),
            action: "Install Nerd Fonts".to_string(),
            command: Some("pacman -S --noconfirm ttf-nerd-fonts-symbols-mono".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "beautification".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Fonts#Patched_fonts".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    result
}

/// Rule 20: Check firmware update tools
fn check_firmware_tools() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for fwupd (firmware updates)
    let has_fwupd = Command::new("pacman")
        .args(&["-Q", "fwupd"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_fwupd {
        result.push(Advice {
            id: "fwupd".to_string(),
            title: "Install fwupd for firmware updates".to_string(),
            reason: "Firmware updates are important for security and hardware stability! fwupd lets you update device firmware (BIOS, SSD, USB devices, etc.) right from Linux. Many vendors now support it officially. Keep your hardware up to date!".to_string(),
            action: "Install fwupd for firmware management".to_string(),
            command: Some("pacman -S --noconfirm fwupd".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "security".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Fwupd".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    result
}

/// Rule 21: Check media players and downloaders
fn check_media_tools() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for mpv (modern video player)
    let has_mpv = Command::new("pacman")
        .args(&["-Q", "mpv"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_mpv {
        result.push(Advice {
            id: "mpv".to_string(),
            title: "Install mpv media player".to_string(),
            reason: "mpv is a powerful, lightweight video player that plays everything. It's keyboard-driven, highly customizable, and handles any format you throw at it. It's also great for streaming and can be controlled via scripts!".to_string(),
            action: "Install mpv".to_string(),
            command: Some("pacman -S --noconfirm mpv".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "multimedia".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Mpv".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    // Check for yt-dlp (youtube-dl fork)
    let has_ytdlp = Command::new("pacman")
        .args(&["-Q", "yt-dlp"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_ytdlp {
        result.push(Advice {
            id: "yt-dlp".to_string(),
            title: "Install yt-dlp for downloading videos".to_string(),
            reason: "yt-dlp is the best way to download videos from YouTube and hundreds of other sites. It's actively maintained (unlike youtube-dl), supports playlist downloads, can extract audio, and has tons of options. Essential tool for media archiving!".to_string(),
            action: "Install yt-dlp".to_string(),
            command: Some("pacman -S --noconfirm yt-dlp".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "multimedia".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Yt-dlp".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    // Check for ffmpeg (video processing)
    let has_ffmpeg = Command::new("pacman")
        .args(&["-Q", "ffmpeg"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_ffmpeg {
        result.push(Advice {
            id: "ffmpeg".to_string(),
            title: "Install FFmpeg for video/audio processing".to_string(),
            reason: "FFmpeg is the Swiss Army knife of media processing. Convert videos, extract audio, resize, crop, merge files - it can do everything! Many apps depend on it, so it's practically essential for any media work.".to_string(),
            action: "Install FFmpeg".to_string(),
            command: Some("pacman -S --noconfirm ffmpeg".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "multimedia".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/FFmpeg".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    result
}

/// Rule 22: Check audio system
fn check_audio_system() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if PipeWire is installed
    let has_pipewire = Command::new("pacman")
        .args(&["-Q", "pipewire"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let has_pulseaudio = Command::new("pacman")
        .args(&["-Q", "pulseaudio"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    // Suggest PipeWire if they're still on PulseAudio
    if has_pulseaudio && !has_pipewire {
        result.push(Advice {
            id: "pipewire".to_string(),
            title: "Consider switching to PipeWire audio system".to_string(),
            reason: "PipeWire is the modern replacement for PulseAudio. It has lower latency, better Bluetooth support, and can handle both audio AND video routing. Most distros are switching to it - it's more reliable and feature-rich than PulseAudio!".to_string(),
            action: "Switch to PipeWire (requires removal of PulseAudio)".to_string(),
            command: Some("pacman -S --noconfirm pipewire pipewire-pulse pipewire-alsa pipewire-jack wireplumber".to_string()),
            risk: RiskLevel::Medium,
            priority: Priority::Optional,
            category: "multimedia".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/PipeWire".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    // If they have PipeWire, check for additional useful tools
    if has_pipewire {
        let has_pavucontrol = Command::new("pacman")
            .args(&["-Q", "pavucontrol"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_pavucontrol {
            result.push(Advice {
                id: "pavucontrol".to_string(),
                title: "Install PulseAudio Volume Control (works with PipeWire)".to_string(),
                reason: "pavucontrol gives you a GUI to control audio devices, application volumes, and routing. Even though it says PulseAudio, it works perfectly with PipeWire! Much easier than command-line audio management.".to_string(),
                action: "Install pavucontrol".to_string(),
                command: Some("pacman -S --noconfirm pavucontrol".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "multimedia".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/PulseAudio#pavucontrol".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    result
}

/// Rule 23: Check power management for laptops
fn check_power_management() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if system is a laptop (has battery)
    let is_laptop = std::path::Path::new("/sys/class/power_supply/BAT0").exists() ||
                    std::path::Path::new("/sys/class/power_supply/BAT1").exists();

    if is_laptop {
        // Check for TLP
        let has_tlp = Command::new("pacman")
            .args(&["-Q", "tlp"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_tlp {
            result.push(Advice {
                id: "laptop-tlp".to_string(),
                title: "Install TLP for better laptop battery life".to_string(),
                reason: "TLP automatically optimizes your laptop's power settings. It can significantly extend your battery life by managing CPU frequencies, disk write behavior, USB power, and more. Just install it and forget it - it works automatically!".to_string(),
                action: "Install TLP for power management".to_string(),
                command: Some("pacman -S --noconfirm tlp && systemctl enable tlp.service".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "power".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/TLP".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }

        // Check for powertop
        let has_powertop = Command::new("pacman")
            .args(&["-Q", "powertop"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_powertop {
            result.push(Advice {
                id: "laptop-powertop".to_string(),
                title: "Install powertop to analyze power consumption".to_string(),
                reason: "powertop shows you exactly what's draining your battery. It lists processes, devices, and can even suggest tuning options. Great for diagnosing battery issues and seeing the impact of your power settings!".to_string(),
                action: "Install powertop".to_string(),
                command: Some("pacman -S --noconfirm powertop".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "power".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Powertop".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    result
}

/// Rule 24: Check gamepad and controller support
fn check_gamepad_support() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for gamepad support packages
    let has_steam = Command::new("pacman")
        .args(&["-Q", "steam"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    // Check if any gamepad-related packages are installed
    let has_xpadneo = Command::new("pacman")
        .args(&["-Q", "xpadneo-dkms"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let has_xone = Command::new("pacman")
        .args(&["-Q", "xone-dkms"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    // If user has Steam but no Xbox controller drivers
    if has_steam && !has_xpadneo && !has_xone {
        result.push(Advice {
            id: "gamepad-xbox".to_string(),
            title: "Install Xbox controller drivers for better support".to_string(),
            reason: "If you use Xbox controllers (especially Xbox One/Series), the default kernel drivers have limited functionality. xpadneo or xone give you full support - battery level, rumble, proper button mapping, and better wireless connectivity!".to_string(),
            action: "Install xpadneo for Xbox controller support".to_string(),
            command: Some("paru -S --noconfirm xpadneo-dkms".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "gaming".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Gamepad".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    // Check for antimicrox (gamepad to keyboard/mouse mapping)
    let has_antimicrox = Command::new("pacman")
        .args(&["-Q", "antimicrox"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if has_steam && !has_antimicrox {
        result.push(Advice {
            id: "gamepad-antimicrox".to_string(),
            title: "Install AntiMicroX for gamepad mapping".to_string(),
            reason: "AntiMicroX lets you map gamepad buttons to keyboard and mouse actions. Super useful for games without native controller support, or for using your controller outside of games!".to_string(),
            action: "Install AntiMicroX".to_string(),
            command: Some("pacman -S --noconfirm antimicrox".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "gaming".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Gamepad#Button_mapping".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    result
}

/// Rule 25: Check USB automount setup
fn check_usb_automount() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for udisks2 (required for automounting)
    let has_udisks2 = Command::new("pacman")
        .args(&["-Q", "udisks2"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_udisks2 {
        result.push(Advice {
            id: "udisks2".to_string(),
            title: "Install udisks2 for USB drive management".to_string(),
            reason: "udisks2 handles mounting and unmounting USB drives, external hard drives, and SD cards. Most file managers depend on it for automatic mounting. Without it, you'll have to mount drives manually with command line!".to_string(),
            action: "Install udisks2".to_string(),
            command: Some("pacman -S --noconfirm udisks2".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "desktop".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Udisks".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    } else {
        // If they have udisks2, suggest udiskie for automatic mounting
        let has_udiskie = Command::new("pacman")
            .args(&["-Q", "udiskie"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_udiskie {
            result.push(Advice {
                id: "udiskie".to_string(),
                title: "Install udiskie for automatic USB mounting".to_string(),
                reason: "udiskie automatically mounts USB drives when you plug them in and unmounts when you unplug. No more clicking 'mount' every time! Just plug and play. It's especially great for minimal window managers without built-in automounting.".to_string(),
                action: "Install udiskie".to_string(),
                command: Some("pacman -S --noconfirm udiskie".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "desktop".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Udisks#Udiskie".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    result
}

/// Rule 26: Check Bluetooth setup
fn check_bluetooth() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for Bluetooth hardware
    let has_bluetooth_hw = Command::new("rfkill")
        .args(&["list", "bluetooth"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| !s.is_empty() && s.contains("bluetooth"))
        .unwrap_or(false);

    if has_bluetooth_hw {
        // Check if bluez is installed
        let has_bluez = Command::new("pacman")
            .args(&["-Q", "bluez"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_bluez {
            result.push(Advice {
                id: "bluetooth-bluez".to_string(),
                title: "Install BlueZ for Bluetooth support".to_string(),
                reason: "Your system has Bluetooth hardware, but BlueZ (the Linux Bluetooth stack) isn't installed! Without it, you can't connect any Bluetooth devices - headphones, mice, keyboards, game controllers, etc. BlueZ is essential for Bluetooth to work at all.".to_string(),
                action: "Install BlueZ and enable bluetooth service".to_string(),
                command: Some("pacman -S --noconfirm bluez bluez-utils && systemctl enable --now bluetooth.service".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Mandatory,
                category: "hardware".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Bluetooth".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        } else {
            // Check for GUI tools
            let has_blueman = Command::new("pacman")
                .args(&["-Q", "blueman"])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

            if !has_blueman {
                result.push(Advice {
                    id: "bluetooth-blueman".to_string(),
                    title: "Install Blueman for easy Bluetooth management".to_string(),
                    reason: "Blueman gives you a nice GUI to manage Bluetooth devices. Pair headphones, connect mice, transfer files - all with a simple interface. Much friendlier than command-line bluetoothctl!".to_string(),
                    action: "Install Blueman".to_string(),
                    command: Some("pacman -S --noconfirm blueman".to_string()),
                    risk: RiskLevel::Low,
                    priority: Priority::Recommended,
                    category: "hardware".to_string(),
                    alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Bluetooth#Blueman".to_string()],
                                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
            }
        }
    }

    result
}

/// Rule 27: Check WiFi firmware and setup
fn check_wifi_setup() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for WiFi hardware
    let has_wifi = Command::new("iw")
        .arg("dev")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.contains("Interface"))
        .unwrap_or(false);

    if has_wifi {
        // Check for common firmware packages
        let has_linux_firmware = Command::new("pacman")
            .args(&["-Q", "linux-firmware"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_linux_firmware {
            result.push(Advice {
                id: "wifi-firmware".to_string(),
                title: "Install linux-firmware for WiFi support".to_string(),
                reason: "Your system has WiFi hardware, but the firmware package isn't installed! WiFi cards need firmware to work, and linux-firmware contains drivers for most WiFi chips (Intel, Realtek, Atheros, Broadcom). Without it, your WiFi probably doesn't work at all!".to_string(),
                action: "Install linux-firmware".to_string(),
                command: Some("pacman -S --noconfirm linux-firmware".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Mandatory,
                category: "hardware".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Wireless#WiFi".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }

        // Check for Intel WiFi specific firmware
        let cpu_info = std::fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
        let is_intel_cpu = cpu_info.to_lowercase().contains("intel");

        if is_intel_cpu {
            let _has_intel_ucode = Command::new("pacman")
                .args(&["-Q", "linux-firmware-iwlwifi"])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

            // Note: linux-firmware-iwlwifi might not exist as separate package in all repos
            // This is informational - could be used for future Intel-specific WiFi advice
        }

        // Check for network management GUI
        let has_nm = Command::new("pacman")
            .args(&["-Q", "networkmanager"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if has_nm {
            let has_nm_applet = Command::new("pacman")
                .args(&["-Q", "network-manager-applet"])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

            if !has_nm_applet {
                result.push(Advice {
                    id: "wifi-nm-applet".to_string(),
                    title: "Install NetworkManager applet for WiFi management".to_string(),
                    reason: "You have NetworkManager but no system tray applet! The applet gives you a nice GUI to connect to WiFi networks, see signal strength, and manage connections. Much easier than using nmcli commands!".to_string(),
                    action: "Install network-manager-applet".to_string(),
                    command: Some("pacman -S --noconfirm network-manager-applet".to_string()),
                    risk: RiskLevel::Low,
                    priority: Priority::Recommended,
                    category: "networking".to_string(),
                    alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/NetworkManager#nm-applet".to_string()],
                                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
            }
        }
    }

    result
}

/// Check for snapshot/backup system (Timeshift, Snapper)
fn check_snapshot_systems(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if user has Btrfs (best for snapshots)
    let has_btrfs = facts.storage_devices.iter().any(|dev| dev.filesystem.to_lowercase().contains("btrfs"));

    // Check for existing snapshot tools
    let has_timeshift = Command::new("pacman")
        .args(&["-Q", "timeshift"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let has_snapper = Command::new("pacman")
        .args(&["-Q", "snapper"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    // If no snapshot tool is installed, recommend one
    if !has_timeshift && !has_snapper {
        if has_btrfs {
            // Recommend Snapper for Btrfs users (more native integration)
            result.push(Advice {
                id: "snapshot-snapper-btrfs".to_string(),
                title: "Install Snapper for Btrfs snapshots".to_string(),
                reason: "You're using Btrfs but have no snapshot system! Snapper automatically creates snapshots before package updates, so you can rollback if something breaks. Think of it as an 'undo button' for your entire system. It integrates perfectly with Btrfs and can save you from disastrous updates!".to_string(),
                action: "Install Snapper for automatic Btrfs snapshots".to_string(),
                command: Some("pacman -S --noconfirm snapper snap-pac".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "maintenance".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Snapper".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });

            result.push(Advice {
                id: "snapshot-snapper-grub".to_string(),
                title: "Add Snapper integration to GRUB bootloader".to_string(),
                reason: "After installing Snapper, add grub-btrfs to boot from snapshots. If an update breaks your system, you can boot from a snapshot at the GRUB menu and restore your system. It's like having a time machine for your entire OS!".to_string(),
                action: "Install grub-btrfs for snapshot booting".to_string(),
                command: Some("pacman -S --noconfirm grub-btrfs".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "maintenance".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Snapper#Boot_to_snapshots".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        } else {
            // Recommend Timeshift for non-Btrfs users (works with ext4)
            result.push(Advice {
                id: "snapshot-timeshift".to_string(),
                title: "Install Timeshift for system snapshots".to_string(),
                reason: "You have no snapshot/backup system! Timeshift creates incremental snapshots of your system, so you can restore if something goes wrong. It works great with ext4 using rsync. Think of it as 'System Restore' for Linux - it can save you from bad updates, misconfigurations, or accidental deletions!".to_string(),
                action: "Install Timeshift for system backups".to_string(),
                command: Some("pacman -S --noconfirm timeshift".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "maintenance".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Timeshift".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    // If they have Snapper but not snap-pac (pre/post hooks)
    if has_snapper {
        let has_snappac = Command::new("pacman")
            .args(&["-Q", "snap-pac"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_snappac {
            result.push(Advice {
                id: "snapshot-snap-pac".to_string(),
                title: "Install snap-pac for automatic pacman snapshots".to_string(),
                reason: "You have Snapper but not snap-pac! snap-pac automatically creates snapshots before and after every pacman transaction. This means you can always rollback bad package updates. It's like having an 'undo' button for every system change!".to_string(),
                action: "Install snap-pac for pacman integration".to_string(),
                command: Some("pacman -S --noconfirm snap-pac".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "maintenance".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Snapper#Automatic_snapshots".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }

        // Check if snapper is actually configured
        let snapper_configs = std::path::Path::new("/etc/snapper/configs");
        if !snapper_configs.exists() || std::fs::read_dir(snapper_configs).map(|mut d| d.next().is_none()).unwrap_or(true) {
            result.push(Advice {
                id: "snapshot-snapper-config".to_string(),
                title: "Configure Snapper for your root filesystem".to_string(),
                reason: "You installed Snapper but haven't configured it yet! You need to create a config for your root subvolume. Run 'sudo snapper -c root create-config /' to set it up, then it will start taking automatic snapshots. Without configuration, Snapper won't do anything!".to_string(),
                action: "Create Snapper configuration".to_string(),
                command: Some("snapper -c root create-config /".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "maintenance".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Snapper#Configuration".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    // If they have Snapper and Btrfs, suggest grub-btrfs if not installed
    if has_snapper && has_btrfs {
        let has_grub_btrfs = Command::new("pacman")
            .args(&["-Q", "grub-btrfs"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_grub_btrfs {
            result.push(Advice {
                id: "snapshot-grub-btrfs".to_string(),
                title: "Add snapshot boot support with grub-btrfs".to_string(),
                reason: "You have Snapper creating snapshots, but you can't boot from them yet! Install grub-btrfs to add snapshot entries to your GRUB menu. If a system update breaks everything, you can reboot and select a snapshot from before the update. It's your escape hatch!".to_string(),
                action: "Install grub-btrfs".to_string(),
                command: Some("pacman -S --noconfirm grub-btrfs && grub-mkconfig -o /boot/grub/grub.cfg".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "maintenance".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Snapper#Boot_to_snapshots".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    result
}

/// Check for Docker and container support
fn check_docker_support(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if Docker is installed
    let has_docker = Command::new("pacman")
        .args(&["-Q", "docker"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    // Check if user has docker in command history (wants to use it)
    let uses_docker = facts.frequently_used_commands.iter()
        .any(|cmd| cmd.command.contains("docker"));

    // Check if user has container-related files
    let has_dockerfile = facts.common_file_types.iter()
        .any(|t| t.contains("docker") || t.contains("container"));

    if !has_docker && (uses_docker || has_dockerfile) {
        result.push(Advice {
            id: "docker-install".to_string(),
            title: "Install Docker for containerization".to_string(),
            reason: "You seem to be working with containers (Dockerfiles found or docker commands in history), but Docker isn't installed! Docker lets you run applications in isolated containers - perfect for development, testing, and deploying apps consistently across systems.".to_string(),
            action: "Install Docker".to_string(),
            command: Some("pacman -S --noconfirm docker".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "development".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Docker".to_string()],
                        depends_on: Vec::new(),
                related_to: vec!["docker-compose-install".to_string(), "lazydocker-install".to_string()],
                bundle: Some("Container Development Stack".to_string()),
            });
    }

    if has_docker {
        // Check if Docker service is enabled
        let docker_enabled = Command::new("systemctl")
            .args(&["is-enabled", "docker"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !docker_enabled {
            result.push(Advice {
                id: "docker-enable-service".to_string(),
                title: "Enable Docker service".to_string(),
                reason: "You have Docker installed but the service isn't enabled! Docker needs its daemon running to work. Enabling it makes Docker start automatically on boot, so you don't have to start it manually every time.".to_string(),
                action: "Enable and start Docker service".to_string(),
                command: Some("systemctl enable --now docker".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "development".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Docker#Installation".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }

        // Check if user is in docker group
        let current_user = std::env::var("SUDO_USER").unwrap_or_else(|_| std::env::var("USER").unwrap_or_default());
        if !current_user.is_empty() {
            let in_docker_group = Command::new("groups")
                .arg(&current_user)
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).contains("docker"))
                .unwrap_or(false);

            if !in_docker_group {
                result.push(Advice {
                    id: "docker-user-group".to_string(),
                    title: "Add your user to docker group".to_string(),
                    reason: format!("You're not in the 'docker' group, so you need to use 'sudo' for every Docker command! Adding yourself to the docker group lets you run Docker without sudo. Much more convenient for development! (You'll need to log out and back in for this to take effect)"),
                    action: format!("Add user '{}' to docker group", current_user),
                    command: Some(format!("usermod -aG docker {}", current_user)),
                    risk: RiskLevel::Low,
                    priority: Priority::Recommended,
                    category: "development".to_string(),
                    alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Docker#Installation".to_string()],
                                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
            }
        }

        // Suggest Docker Compose
        let has_compose = Command::new("pacman")
            .args(&["-Q", "docker-compose"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_compose {
            result.push(Advice {
                id: "docker-compose".to_string(),
                title: "Install Docker Compose for multi-container apps".to_string(),
                reason: "You have Docker but not Docker Compose! Compose makes it easy to define and run multi-container applications with a simple YAML file. Instead of running multiple 'docker run' commands, you define everything in docker-compose.yml and start it all with one command. Essential for modern development!".to_string(),
                action: "Install Docker Compose".to_string(),
                command: Some("pacman -S --noconfirm docker-compose".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "development".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Docker#Docker_Compose".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    result
}

/// Check for virtualization support (QEMU/KVM)
fn check_virtualization_support(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if CPU supports virtualization
    let cpu_has_virt = facts.cpu_model.to_lowercase().contains("amd") ||
                       facts.cpu_model.to_lowercase().contains("intel");

    if !cpu_has_virt {
        return result; // No virtualization support
    }

    // Check if virtualization is enabled in BIOS
    let virt_enabled = std::path::Path::new("/dev/kvm").exists();

    if !virt_enabled {
        result.push(Advice {
            id: "virtualization-enable-bios".to_string(),
            title: "Enable virtualization in BIOS".to_string(),
            reason: "Your CPU supports virtualization (KVM), but /dev/kvm doesn't exist! This means virtualization is disabled in your BIOS/UEFI. You need to enable Intel VT-x (Intel) or AMD-V (AMD) in your BIOS settings to use virtual machines with hardware acceleration. Without it, VMs will be extremely slow!".to_string(),
            action: "Reboot and enable VT-x/AMD-V in BIOS".to_string(),
            command: None, // Manual BIOS change required
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "system".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/KVM#Checking_support".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        return result; // Don't suggest KVM tools if virtualization is disabled
    }

    // Check for QEMU
    let has_qemu = Command::new("pacman")
        .args(&["-Q", "qemu-full"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    // Check if user seems interested in virtualization
    let uses_virt = facts.frequently_used_commands.iter()
        .any(|cmd| cmd.command.contains("qemu") || cmd.command.contains("virt") || cmd.command.contains("kvm"));

    if !has_qemu && (uses_virt || virt_enabled) {
        result.push(Advice {
            id: "qemu-install".to_string(),
            title: "Install QEMU for virtual machines".to_string(),
            reason: "Your system supports hardware virtualization (KVM), but QEMU isn't installed! QEMU with KVM gives you near-native performance for running virtual machines. Perfect for testing different Linux distros, running Windows VMs, or development environments. With KVM, VMs run almost as fast as bare metal!".to_string(),
            action: "Install QEMU with full system emulation".to_string(),
            command: Some("pacman -S --noconfirm qemu-full".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "system".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/QEMU".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    if has_qemu {
        // Suggest virt-manager for GUI management
        let has_virt_manager = Command::new("pacman")
            .args(&["-Q", "virt-manager"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_virt_manager {
            result.push(Advice {
                id: "virt-manager".to_string(),
                title: "Install virt-manager for easy VM management".to_string(),
                reason: "You have QEMU but no graphical manager! virt-manager provides a beautiful GUI for creating and managing VMs. It's way easier than typing QEMU commands - just click to create VMs, attach ISOs, configure networks, etc. Think of it as VirtualBox but for KVM!".to_string(),
                action: "Install virt-manager".to_string(),
                command: Some("pacman -S --noconfirm virt-manager".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "system".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Virt-manager".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }

        // Check if libvirt service is running
        let libvirt_running = Command::new("systemctl")
            .args(&["is-active", "libvirtd"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !libvirt_running {
            result.push(Advice {
                id: "libvirt-enable".to_string(),
                title: "Enable libvirt service for VM management".to_string(),
                reason: "You have QEMU installed but libvirtd service isn't running! Libvirt provides the management layer for VMs - it's what virt-manager and other tools use to control QEMU. Start it to manage your virtual machines properly.".to_string(),
                action: "Enable and start libvirtd".to_string(),
                command: Some("systemctl enable --now libvirtd".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "system".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Libvirt#Installation".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    result
}

/// Check for printer support (CUPS)
fn check_printer_support() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if CUPS is installed
    let has_cups = Command::new("pacman")
        .args(&["-Q", "cups"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    // Check for USB printers (lsusb)
    let has_printer = Command::new("lsusb")
        .output()
        .map(|o| {
            let output = String::from_utf8_lossy(&o.stdout).to_lowercase();
            output.contains("printer") || output.contains("canon") || 
            output.contains("hp") || output.contains("epson") || 
            output.contains("brother")
        })
        .unwrap_or(false);

    if !has_cups && has_printer {
        result.push(Advice {
            id: "cups-install".to_string(),
            title: "Install CUPS for printer support".to_string(),
            reason: "A printer was detected via USB, but CUPS isn't installed! CUPS (Common Unix Printing System) is what Linux uses to manage printers. Without it, you can't print anything. It provides a web interface at http://localhost:631 for easy printer setup.".to_string(),
            action: "Install CUPS printing system".to_string(),
            command: Some("pacman -S --noconfirm cups".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "hardware".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/CUPS".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    if has_cups {
        // Check if CUPS service is enabled
        let cups_enabled = Command::new("systemctl")
            .args(&["is-enabled", "cups"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !cups_enabled {
            result.push(Advice {
                id: "cups-enable-service".to_string(),
                title: "Enable CUPS service for printing".to_string(),
                reason: "CUPS is installed but not enabled! The CUPS service needs to be running for printers to work. Enable it so it starts automatically on boot, then you can access the web interface at http://localhost:631 to add printers.".to_string(),
                action: "Enable and start CUPS".to_string(),
                command: Some("systemctl enable --now cups".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "hardware".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/CUPS#Installation".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }

        // Suggest printer drivers
        let has_gutenprint = Command::new("pacman")
            .args(&["-Q", "gutenprint"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_gutenprint {
            result.push(Advice {
                id: "printer-drivers".to_string(),
                title: "Install Gutenprint for wide printer support".to_string(),
                reason: "You have CUPS but no printer drivers! Gutenprint provides drivers for hundreds of printer models (Canon, Epson, HP, etc.). Without proper drivers, your printer might not work or print at low quality. Think of it as the 'universal driver pack' for printers.".to_string(),
                action: "Install Gutenprint drivers".to_string(),
                command: Some("pacman -S --noconfirm gutenprint".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "hardware".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/CUPS#Printer_drivers".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    result
}

/// Check for archive management tools
fn check_archive_tools() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for common archive formats support
    let has_unzip = Command::new("pacman")
        .args(&["-Q", "unzip"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let has_unrar = Command::new("pacman")
        .args(&["-Q", "unrar"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let has_p7zip = Command::new("pacman")
        .args(&["-Q", "p7zip"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_unzip {
        result.push(Advice {
            id: "archive-unzip".to_string(),
            title: "Install unzip for ZIP archive support".to_string(),
            reason: "You don't have unzip installed! ZIP is one of the most common archive formats - downloaded files, GitHub repos, Windows files all use it. Without unzip, you can't extract .zip files. It's a tiny package that you'll definitely need.".to_string(),
            action: "Install unzip".to_string(),
            command: Some("pacman -S --noconfirm unzip".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "utilities".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Archiving_and_compression".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    if !has_unrar {
        result.push(Advice {
            id: "archive-unrar".to_string(),
            title: "Install unrar for RAR archive support".to_string(),
            reason: "No RAR support detected! RAR files are super common, especially for downloads, game files, and Windows software. Without unrar, .rar files will just sit there looking useless. Small install, huge convenience!".to_string(),
            action: "Install unrar".to_string(),
            command: Some("pacman -S --noconfirm unrar".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "utilities".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Archiving_and_compression".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    if !has_p7zip {
        result.push(Advice {
            id: "archive-p7zip".to_string(),
            title: "Install p7zip for 7z archive support".to_string(),
            reason: "7z archives offer better compression than ZIP but you can't extract them! p7zip handles .7z files which are increasingly popular for software distribution and large file compression. It also provides better ZIP handling than the basic unzip command.".to_string(),
            action: "Install p7zip".to_string(),
            command: Some("pacman -S --noconfirm p7zip".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "utilities".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Archiving_and_compression".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    result
}

/// Check for system monitoring tools
fn check_monitoring_tools() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for htop (better than top)
    let has_htop = Command::new("pacman")
        .args(&["-Q", "htop"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_htop {
        result.push(Advice {
            id: "monitoring-htop".to_string(),
            title: "Install htop for interactive process monitoring".to_string(),
            reason: "'top' is okay, but htop is WAY better! It's colorful, interactive, shows CPU cores individually, makes it easy to kill processes, and generally makes system monitoring actually pleasant. You can sort by memory, CPU, or any column with a keystroke. Every Linux user should have this!".to_string(),
            action: "Install htop".to_string(),
            command: Some("pacman -S --noconfirm htop".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "utilities".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Htop".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    // Check for btop (even better than htop)
    let has_btop = Command::new("pacman")
        .args(&["-Q", "btop"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_btop && has_htop {
        result.push(Advice {
            id: "monitoring-btop".to_string(),
            title: "Consider btop for gorgeous system monitoring".to_string(),
            reason: "You have htop, but btop is the next evolution! It's like htop on steroids - beautiful graphs, detailed stats, disk I/O, network monitoring, GPU stats, all in a stunning TUI. It's eye candy AND functional. If you like htop, you'll love btop!".to_string(),
            action: "Install btop".to_string(),
            command: Some("pacman -S --noconfirm btop".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "utilities".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Btop".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    // Check for iotop (disk I/O monitoring)
    let has_iotop = Command::new("pacman")
        .args(&["-Q", "iotop"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_iotop {
        result.push(Advice {
            id: "monitoring-iotop".to_string(),
            title: "Install iotop to monitor disk I/O".to_string(),
            reason: "When your system is slow and disk activity is high, iotop tells you exactly which process is hammering your disk! It's like 'top' but for disk I/O. Essential for debugging slow systems or figuring out what's writing to your SSD constantly.".to_string(),
            action: "Install iotop".to_string(),
            command: Some("pacman -S --noconfirm iotop".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "utilities".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Iotop".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    result
}

/// Check for firmware update tools (fwupd)
fn check_firmware_updates() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if fwupd is installed
    let has_fwupd = Command::new("pacman")
        .args(&["-Q", "fwupd"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_fwupd {
        result.push(Advice {
            id: "firmware-fwupd".to_string(),
            title: "Install fwupd for automatic firmware updates".to_string(),
            reason: "Your system hardware probably has firmware that needs updates! fwupd provides firmware updates for your motherboard, SSD, GPU, USB devices, and more - all from within Linux. It's like Windows Update but for your hardware firmware. Keeping firmware updated fixes bugs, improves performance, and patches security vulnerabilities.".to_string(),
            action: "Install fwupd for firmware management".to_string(),
            command: Some("pacman -S --noconfirm fwupd".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "maintenance".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Fwupd".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    } else {
        // Suggest running firmware check
        result.push(Advice {
            id: "firmware-check-updates".to_string(),
            title: "Check for available firmware updates".to_string(),
            reason: "You have fwupd installed - run 'fwupdmgr get-updates' to check if your hardware has firmware updates available! This checks your motherboard, SSD, peripherals, and more for security patches and improvements.".to_string(),
            action: "Check for firmware updates".to_string(),
            command: Some("fwupdmgr refresh && fwupdmgr get-updates".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "maintenance".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Fwupd#Usage".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    result
}

/// Check for SSD optimizations
fn check_ssd_optimizations(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if any storage device is an SSD
    let has_ssd = facts.storage_devices.iter().any(|dev| {
        // SSDs are typically nvme or have "ssd" in name
        dev.name.contains("nvme") || dev.filesystem.to_lowercase().contains("ssd")
    });

    // Better SSD detection via /sys/block
    let ssd_detected = std::fs::read_dir("/sys/block")
        .ok()
        .map(|entries| {
            entries.filter_map(|e| e.ok()).any(|entry| {
                let path = entry.path().join("queue/rotational");
                std::fs::read_to_string(path)
                    .map(|content| content.trim() == "0")
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false);

    if !has_ssd && !ssd_detected {
        return result; // No SSD detected
    }

    // Check for noatime mount option
    let fstab_content = std::fs::read_to_string("/etc/fstab").unwrap_or_default();
    let has_noatime = fstab_content.lines().any(|line| {
        !line.trim().starts_with('#') && 
        (line.contains("noatime") || line.contains("relatime"))
    });

    if !has_noatime {
        result.push(Advice {
            id: "ssd-noatime".to_string(),
            title: "Enable noatime for SSD performance".to_string(),
            reason: "You have an SSD but 'noatime' isn't set in fstab! By default, Linux updates the access time every time you read a file, which causes extra writes. For SSDs, this is pure overhead with no benefit. Adding 'noatime' to mount options reduces writes and improves performance. It's the #1 SSD optimization!".to_string(),
            action: "Add noatime to fstab mount options".to_string(),
            command: None, // Manual fstab edit needed
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "performance".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Fstab#atime_options".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    // Check for discard support (continuous TRIM)
    let has_discard = fstab_content.lines().any(|line| {
        !line.trim().starts_with('#') && line.contains("discard")
    });

    if !has_discard {
        result.push(Advice {
            id: "ssd-discard-option".to_string(),
            title: "Consider enabling continuous TRIM (discard)".to_string(),
            reason: "Your SSD could benefit from the 'discard' mount option! This enables continuous TRIM, which tells the SSD about deleted blocks immediately instead of waiting for a weekly timer. Modern SSDs handle this well and it keeps performance more consistent. Alternative to the periodic fstrim.timer.".to_string(),
            action: "Add discard to mount options (or keep fstrim.timer)".to_string(),
            command: None, // Manual decision - discard vs fstrim.timer
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "performance".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Solid_state_drive#Continuous_TRIM".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    result
}

/// Check for swap compression (zram/zswap)
fn check_swap_compression() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if zram is loaded
    let has_zram = std::path::Path::new("/dev/zram0").exists();

    // Check if zswap is enabled
    let zswap_enabled = std::fs::read_to_string("/sys/module/zswap/parameters/enabled")
        .map(|s| s.trim() == "Y")
        .unwrap_or(false);

    if !has_zram && !zswap_enabled {
        result.push(Advice {
            id: "swap-zram".to_string(),
            title: "Consider zram for compressed swap in RAM".to_string(),
            reason: "zram creates a compressed swap device in RAM! Instead of swapping to disk (slow), data gets compressed in RAM first. This can double your effective RAM and makes swapping much faster. Perfect for systems with limited RAM - you get the memory benefits of swap without the disk slowdown!".to_string(),
            action: "Install zram-generator for automatic zram".to_string(),
            command: Some("pacman -S --noconfirm zram-generator".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "performance".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Zram".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    result
}

/// Check DNS configuration (systemd-resolved)
fn check_dns_configuration() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if systemd-resolved is available but not used
    let has_resolved = std::path::Path::new("/usr/lib/systemd/systemd-resolved").exists();
    let resolved_active = Command::new("systemctl")
        .args(&["is-active", "systemd-resolved"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    // Check current DNS setup
    let resolv_conf = std::fs::read_to_string("/etc/resolv.conf").unwrap_or_default();
    let using_stub_resolver = resolv_conf.contains("127.0.0.53");

    if has_resolved && !resolved_active && !using_stub_resolver {
        result.push(Advice {
            id: "dns-systemd-resolved".to_string(),
            title: "Consider systemd-resolved for modern DNS".to_string(),
            reason: "systemd-resolved provides modern DNS with caching, DNSSEC validation, and per-interface DNS settings. It's faster than traditional DNS (caches queries) and more secure (validates DNSSEC). Especially useful with NetworkManager or multiple network interfaces!".to_string(),
            action: "Enable systemd-resolved".to_string(),
            command: Some("systemctl enable --now systemd-resolved && ln -sf /run/systemd/resolve/stub-resolv.conf /etc/resolv.conf".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "networking".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Systemd-resolved".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    // Check for public DNS servers
    if !resolv_conf.contains("127.0.0.") {
        let using_isp_dns = !resolv_conf.contains("1.1.1.1") && 
                           !resolv_conf.contains("8.8.8.8") && 
                           !resolv_conf.contains("9.9.9.9");

        if using_isp_dns {
            result.push(Advice {
                id: "dns-public-servers".to_string(),
                title: "Consider using public DNS servers".to_string(),
                reason: "You're using your ISP's DNS servers, which may be slow, unreliable, or log your queries! Public DNS like Cloudflare (1.1.1.1), Google (8.8.8.8), or Quad9 (9.9.9.9) are usually faster, more reliable, and respect privacy better. Cloudflare is the fastest with strong privacy!".to_string(),
                action: "Consider switching to public DNS".to_string(),
                command: None, // Manual decision on which DNS to use
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "networking".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Domain_name_resolution".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    result
}

/// Check systemd journal size
fn check_journal_size() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check journal size
    let journal_size = Command::new("journalctl")
        .args(&["--disk-usage"])
        .output()
        .ok()
        .and_then(|o| {
            let output = String::from_utf8_lossy(&o.stdout);
            // Parse "Archived and active journals take up 512.0M in the file system."
            output.split_whitespace()
                .find(|s| s.ends_with("M") || s.ends_with("G"))
                .and_then(|s| {
                    let num: String = s.chars().take_while(|c| c.is_numeric() || *c == '.').collect();
                    num.parse::<f64>().ok().map(|n| {
                        if s.ends_with("G") { n * 1024.0 } else { n }
                    })
                })
        });

    if let Some(size_mb) = journal_size {
        if size_mb > 500.0 {
            result.push(Advice {
                id: "journal-large-size".to_string(),
                title: format!("Journal logs are using {:.0}MB of disk space", size_mb),
                reason: format!("Your systemd journal has grown to {:.0}MB! Journal logs accumulate over time and can waste significant disk space. You can safely clean old logs - they're mainly useful for debugging recent issues. systemd can automatically limit journal size.", size_mb),
                action: "Clean old journal logs and set size limit".to_string(),
                command: Some("journalctl --vacuum-size=100M && echo 'SystemMaxUse=100M' >> /etc/systemd/journald.conf".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "maintenance".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Systemd/Journal#Journal_size_limit".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    // Check if journal size limit is configured
    let journald_conf = std::fs::read_to_string("/etc/systemd/journald.conf").unwrap_or_default();
    let has_size_limit = journald_conf.lines().any(|line| {
        !line.trim().starts_with('#') && line.contains("SystemMaxUse")
    });

    if !has_size_limit {
        result.push(Advice {
            id: "journal-set-limit".to_string(),
            title: "Set systemd journal size limit".to_string(),
            reason: "No journal size limit is configured! Without a limit, logs can grow indefinitely and fill your disk over time. Setting 'SystemMaxUse=100M' in journald.conf keeps logs under control while still keeping enough history for troubleshooting. Set it and forget it!".to_string(),
            action: "Configure journal size limit".to_string(),
            command: Some("echo 'SystemMaxUse=100M' >> /etc/systemd/journald.conf && systemctl restart systemd-journald".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "maintenance".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Systemd/Journal#Journal_size_limit".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    result
}

/// Check AUR helper safety
fn check_aur_helper_safety() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check which AUR helper is installed
    let has_yay = Command::new("pacman").args(&["-Q", "yay"]).output().map(|o| o.status.success()).unwrap_or(false);
    let has_paru = Command::new("pacman").args(&["-Q", "paru"]).output().map(|o| o.status.success()).unwrap_or(false);
    let has_pikaur = Command::new("pacman").args(&["-Q", "pikaur"]).output().map(|o| o.status.success()).unwrap_or(false);

    if has_yay || has_paru || has_pikaur {
        result.push(Advice {
            id: "aur-safety-reminder".to_string(),
            title: "AUR Safety Reminder: Always review PKGBUILDs".to_string(),
            reason: "You're using an AUR helper - that's great! But remember: ALWAYS review the PKGBUILD before installing AUR packages. AUR packages can run any code during installation, so malicious packages could compromise your system. Think of it like downloading a random script from the internet - check it first!".to_string(),
            action: "Always review PKGBUILDs (use --editmenu or --show)".to_string(),
            command: None, // Educational reminder
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "security".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/AUR_helpers#Safety".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });

        // Check if user has devel packages installed (needs regular updates)
        let devel_packages = Command::new("pacman")
            .args(&["-Qq"])
            .output()
            .ok()
            .map(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .lines()
                    .filter(|line| line.ends_with("-git") || line.ends_with("-svn") || line.ends_with("-hg"))
                    .count()
            })
            .unwrap_or(0);

        if devel_packages > 0 {
            result.push(Advice {
                id: "aur-devel-update".to_string(),
                title: format!("You have {} -git/-svn development packages", devel_packages),
                reason: "Development packages (-git, -svn, -hg) don't get automatic updates! They track upstream development, so you need to rebuild them periodically to get new features and fixes. Run your AUR helper with the devel flag (yay -Syu --devel or paru -Syu --devel) to update them.".to_string(),
                action: "Rebuild development packages regularly".to_string(),
                command: if has_yay { Some("yay -Syu --devel".to_string()) } else if has_paru { Some("paru -Syu --devel".to_string()) } else { None },
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "maintenance".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/AUR_helpers".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    result
}

/// Check locale and timezone configuration
fn check_locale_timezone() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if locale is configured
    let locale_conf = std::fs::read_to_string("/etc/locale.conf").unwrap_or_default();
    if locale_conf.is_empty() || !locale_conf.contains("LANG=") {
        result.push(Advice {
            id: "locale-not-set".to_string(),
            title: "System locale is not configured".to_string(),
            reason: "Your system locale isn't properly set! This affects how dates, times, numbers, and currency are displayed. It can cause weird formatting in applications and sometimes break programs that expect a specific locale. Set it to match your language/region (e.g., en_US.UTF-8 for US English).".to_string(),
            action: "Configure system locale".to_string(),
            command: Some("echo 'LANG=en_US.UTF-8' > /etc/locale.conf && locale-gen".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "system".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Locale".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    // Check timezone
    let tz_link = std::fs::read_link("/etc/localtime").ok();
    if tz_link.is_none() {
        result.push(Advice {
            id: "timezone-not-set".to_string(),
            title: "System timezone is not configured".to_string(),
            reason: "Your timezone isn't set! This means your system clock shows UTC time instead of your local time. It affects file timestamps, logs, scheduled tasks, and anything time-related. Set it to your actual timezone so times make sense!".to_string(),
            action: "Set system timezone".to_string(),
            command: Some("timedatectl set-timezone America/New_York".to_string()), // Example
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "system".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/System_time#Time_zone".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    // Check if NTP is enabled for time sync
    let ntp_enabled = Command::new("timedatectl")
        .args(&["show", "--property=NTP", "--value"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "yes")
        .unwrap_or(false);

    if !ntp_enabled {
        result.push(Advice {
            id: "ntp-not-enabled".to_string(),
            title: "Automatic time synchronization is disabled".to_string(),
            reason: "NTP (Network Time Protocol) isn't enabled! Your system clock will drift over time, leading to incorrect timestamps. This can break SSL certificates, cause authentication failures, and mess up logs. Enable NTP to keep your clock accurate automatically - systemd-timesyncd does this perfectly!".to_string(),
            action: "Enable NTP time synchronization".to_string(),
            command: Some("timedatectl set-ntp true".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "system".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Systemd-timesyncd".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    result
}

/// Check for laptop-specific optimizations
fn check_laptop_optimizations(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if this is a laptop (has battery)
    let has_battery = std::path::Path::new("/sys/class/power_supply/BAT0").exists() ||
                      std::path::Path::new("/sys/class/power_supply/BAT1").exists();

    if !has_battery {
        return result; // Not a laptop
    }

    // Check for powertop
    let has_powertop = Command::new("pacman")
        .args(&["-Q", "powertop"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_powertop {
        result.push(Advice {
            id: "laptop-powertop".to_string(),
            title: "Install powertop for battery optimization".to_string(),
            reason: "You're on a laptop but don't have powertop! Powertop shows detailed power consumption and can auto-tune your system for better battery life. It can easily add 1-2 hours of battery by optimizing USB power, CPU C-states, and more. Run 'powertop --auto-tune' for automatic optimizations!".to_string(),
            action: "Install powertop".to_string(),
            command: Some("pacman -S --noconfirm powertop".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "performance".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Powertop".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    // Check for touchpad drivers (libinput)
    let has_libinput = Command::new("pacman")
        .args(&["-Q", "xf86-input-libinput"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_libinput {
        result.push(Advice {
            id: "laptop-touchpad".to_string(),
            title: "Install libinput for modern touchpad support".to_string(),
            reason: "You're on a laptop and need good touchpad drivers! libinput is the modern input driver that handles touchpads, trackpoints, and gestures. It provides smooth scrolling, multi-finger gestures, palm detection, and tap-to-click. Essential for a good laptop experience!".to_string(),
            action: "Install xf86-input-libinput".to_string(),
            command: Some("pacman -S --noconfirm xf86-input-libinput".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "hardware".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Libinput".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    // Check for backlight control
    let has_backlight_control = Command::new("which")
        .arg("brightnessctl")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false) || Command::new("which")
        .arg("light")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_backlight_control {
        result.push(Advice {
            id: "laptop-backlight".to_string(),
            title: "Install brightnessctl for screen brightness control".to_string(),
            reason: "You can't easily control your laptop screen brightness! brightnessctl lets you adjust brightness from the command line or bind it to keyboard shortcuts. Essential for laptops - you'll want to dim the screen to save battery or brighten it in sunlight.".to_string(),
            action: "Install brightnessctl".to_string(),
            command: Some("pacman -S --noconfirm brightnessctl".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "hardware".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Backlight".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    // Check for laptop-mode-tools (advanced power management)
    let has_laptop_mode = Command::new("pacman")
        .args(&["-Q", "laptop-mode-tools"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_laptop_mode && !facts.dev_tools_detected.iter().any(|t| t == "tlp") {
        result.push(Advice {
            id: "laptop-mode-tools".to_string(),
            title: "Consider laptop-mode-tools for advanced power saving".to_string(),
            reason: "Want even more battery life? laptop-mode-tools provides aggressive power management: spins down HDDs, manages CPU frequency, controls device power states, and more. It's more configurable than TLP but requires more setup. Great for squeezing every minute out of your battery!".to_string(),
            action: "Install laptop-mode-tools".to_string(),
            command: Some("pacman -S --noconfirm laptop-mode-tools".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "performance".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Laptop_Mode_Tools".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    result
}

/// Check for webcam support
fn check_webcam_support() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if webcam exists
    let has_webcam = std::path::Path::new("/dev/video0").exists() ||
                     std::path::Path::new("/dev/video1").exists();

    if !has_webcam {
        return result; // No webcam detected
    }

    // Check for v4l-utils
    let has_v4l = Command::new("pacman")
        .args(&["-Q", "v4l-utils"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_v4l {
        result.push(Advice {
            id: "webcam-v4l-utils".to_string(),
            title: "Install v4l-utils for webcam control".to_string(),
            reason: "You have a webcam but no tools to control it! v4l-utils provides v4l2-ctl for adjusting brightness, contrast, focus, and other camera settings. Super useful for video calls - you can tweak your camera to look good in any lighting!".to_string(),
            action: "Install v4l-utils".to_string(),
            command: Some("pacman -S --noconfirm v4l-utils".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "hardware".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Webcam_setup".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    // Suggest cheese for testing
    let has_cheese = Command::new("pacman")
        .args(&["-Q", "cheese"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_cheese {
        result.push(Advice {
            id: "webcam-cheese".to_string(),
            title: "Install Cheese for webcam testing".to_string(),
            reason: "Want to test your webcam? Cheese is a simple, fast webcam viewer. Perfect for checking if your camera works, adjusting position, or just making sure you look good before a video call! It can also take photos and videos.".to_string(),
            action: "Install Cheese webcam viewer".to_string(),
            command: Some("pacman -S --noconfirm cheese".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "utilities".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Webcam_setup#Cheese".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    result
}

/// Check for audio enhancements
fn check_audio_enhancements() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if PipeWire or PulseAudio is running
    let has_pipewire = Command::new("systemctl")
        .args(&["--user", "is-active", "pipewire"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let has_pulseaudio = Command::new("systemctl")
        .args(&["--user", "is-active", "pulseaudio"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_pipewire && !has_pulseaudio {
        return result; // No audio server detected
    }

    // Check for EasyEffects (PipeWire) or PulseEffects (PulseAudio)
    let has_easyeffects = Command::new("pacman")
        .args(&["-Q", "easyeffects"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if has_pipewire && !has_easyeffects {
        result.push(Advice {
            id: "audio-easyeffects".to_string(),
            title: "Install EasyEffects for audio enhancement".to_string(),
            reason: "You're using PipeWire but missing EasyEffects! It's an amazing audio processor that can add bass boost, equalizer, noise reduction, reverb, and more. Make cheap headphones sound expensive, improve microphone quality, or just make everything sound better. It's like having a professional audio engineer built in!".to_string(),
            action: "Install EasyEffects".to_string(),
            command: Some("pacman -S --noconfirm easyeffects".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "audio".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/PipeWire#EasyEffects".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    // Check for pavucontrol (volume control GUI)
    let has_pavucontrol = Command::new("pacman")
        .args(&["-Q", "pavucontrol"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_pavucontrol {
        result.push(Advice {
            id: "audio-pavucontrol".to_string(),
            title: "Install pavucontrol for advanced volume control".to_string(),
            reason: "You have no GUI volume mixer! pavucontrol lets you control volume per-application, switch audio devices, adjust balance, and manage recording sources. Way better than basic volume controls - you can route Discord to headphones while music plays on speakers!".to_string(),
            action: "Install pavucontrol".to_string(),
            command: Some("pacman -S --noconfirm pavucontrol".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "audio".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/PulseAudio#pavucontrol".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    result
}

/// Check for shell productivity tools
fn check_shell_productivity() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for bash/zsh completion
    let shell = std::env::var("SHELL").unwrap_or_default();
    
    if shell.contains("bash") {
        let has_completion = Command::new("pacman")
            .args(&["-Q", "bash-completion"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_completion {
            result.push(Advice {
                id: "shell-bash-completion".to_string(),
                title: "Install bash-completion for better tab completion".to_string(),
                reason: "You're missing bash-completion! This adds intelligent tab-completion for commands, options, and file paths. Press tab and it completes git commands, package names, SSH hosts, and hundreds of other things. Makes the terminal SO much faster!".to_string(),
                action: "Install bash-completion".to_string(),
                command: Some("pacman -S --noconfirm bash-completion".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "utilities".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Bash#Tab_completion".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
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
            id: "shell-fzf".to_string(),
            title: "Install fzf for fuzzy finding".to_string(),
            reason: "fzf is a GAME CHANGER for terminal productivity! It adds fuzzy search to command history (Ctrl+R), file finding, directory jumping, and more. Instead of scrolling through history or typing paths, just type a few letters and fzf finds what you want. Every power user has this!".to_string(),
            action: "Install fzf fuzzy finder".to_string(),
            command: Some("pacman -S --noconfirm fzf".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "utilities".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Fzf".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    // Check for tmux/screen
    let has_tmux = Command::new("pacman")
        .args(&["-Q", "tmux"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let has_screen = Command::new("pacman")
        .args(&["-Q", "screen"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_tmux && !has_screen {
        result.push(Advice {
            id: "shell-tmux".to_string(),
            title: "Install tmux for terminal multiplexing".to_string(),
            reason: "tmux is essential for terminal work! It lets you split your terminal into panes, create multiple windows, and most importantly - detach and reattach sessions. Start a long process over SSH, disconnect, come back later and it's still running! Also great for organizing workflows with split panes.".to_string(),
            action: "Install tmux".to_string(),
            command: Some("pacman -S --noconfirm tmux".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "utilities".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Tmux".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    result
}

/// Check filesystem maintenance
fn check_filesystem_maintenance(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for ext4 filesystems that might need fsck
    let has_ext4 = facts.storage_devices.iter().any(|dev| 
        dev.filesystem.to_lowercase().contains("ext4") || 
        dev.filesystem.to_lowercase().contains("ext3")
    );

    if has_ext4 {
        result.push(Advice {
            id: "filesystem-fsck-reminder".to_string(),
            title: "Reminder: Run fsck on ext4 filesystems periodically".to_string(),
            reason: "You're using ext4 - remember to run filesystem checks occasionally! Modern ext4 is reliable, but errors can accumulate over time from power failures or hardware issues. Boot from a live USB and run 'fsck -f /dev/sdXY' on unmounted filesystems yearly to catch problems early.".to_string(),
            action: "Schedule periodic filesystem checks".to_string(),
            command: None, // Must be done from live USB
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "maintenance".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Fsck".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    // Check for Btrfs scrub
    let has_btrfs = facts.storage_devices.iter().any(|dev| 
        dev.filesystem.to_lowercase().contains("btrfs")
    );

    if has_btrfs {
        result.push(Advice {
            id: "filesystem-btrfs-scrub".to_string(),
            title: "Run Btrfs scrub for data integrity".to_string(),
            reason: "You're using Btrfs - run scrubs regularly! Scrub reads all data and metadata, verifies checksums, and repairs corruption automatically. It's like a health check for your filesystem. Run 'btrfs scrub start /' monthly to catch bit rot and disk errors before they cause data loss.".to_string(),
            action: "Start Btrfs scrub".to_string(),
            command: Some("btrfs scrub start /".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "maintenance".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Btrfs#Scrub".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    result
}

/// Check kernel parameters
fn check_kernel_parameters() -> Vec<Advice> {
    let mut result = Vec::new();

    // Read current kernel parameters
    let cmdline = std::fs::read_to_string("/proc/cmdline").unwrap_or_default();

    // Check for quiet parameter (reduces boot messages)
    if !cmdline.contains("quiet") {
        result.push(Advice {
            id: "kernel-quiet-boot".to_string(),
            title: "Add 'quiet' kernel parameter for cleaner boot".to_string(),
            reason: "Your boot shows all kernel messages! Adding 'quiet' to kernel parameters makes boot look cleaner by hiding verbose kernel output. You'll still see important errors, but not the flood of driver messages. Makes your system look more polished!".to_string(),
            action: "Add 'quiet' to GRUB_CMDLINE_LINUX in /etc/default/grub".to_string(),
            command: None, // Manual GRUB config edit
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "beautification".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Kernel_parameters".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    // Check for splash (boot splash screen)
    if !cmdline.contains("splash") {
        result.push(Advice {
            id: "kernel-splash-screen".to_string(),
            title: "Add 'splash' for graphical boot screen".to_string(),
            reason: "Want a pretty boot screen instead of text? Add 'splash' kernel parameter and install plymouth for a graphical boot animation. Makes your system boot look professional and modern instead of showing raw text. Purely cosmetic but nice!".to_string(),
            action: "Add 'splash' parameter and install plymouth".to_string(),
            command: None, // Requires plymouth setup
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "beautification".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Plymouth".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    result
}

/// Check bootloader optimization
fn check_bootloader_optimization() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check GRUB timeout
    let grub_config = std::fs::read_to_string("/etc/default/grub").unwrap_or_default();
    
    if grub_config.contains("GRUB_TIMEOUT=5") || grub_config.contains("GRUB_TIMEOUT=10") {
        result.push(Advice {
            id: "bootloader-reduce-timeout".to_string(),
            title: "Reduce GRUB timeout for faster boot".to_string(),
            reason: "Your GRUB waits 5-10 seconds before booting! If you have one OS and don't need to pick kernels, reduce GRUB_TIMEOUT to 1 or 2 seconds. Saves time on every boot! You can still access the menu by holding Shift during boot if needed.".to_string(),
            action: "Set GRUB_TIMEOUT=1 in /etc/default/grub".to_string(),
            command: Some("sed -i 's/^GRUB_TIMEOUT=.*/GRUB_TIMEOUT=1/' /etc/default/grub && grub-mkconfig -o /boot/grub/grub.cfg".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "performance".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/GRUB/Tips_and_tricks#Speeding_up_GRUB".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    // Check for GRUB background
    if !grub_config.contains("GRUB_BACKGROUND") {
        result.push(Advice {
            id: "bootloader-custom-background".to_string(),
            title: "Consider adding custom GRUB background".to_string(),
            reason: "Your GRUB menu is plain! You can set a custom background image with GRUB_BACKGROUND in /etc/default/grub. Makes your bootloader look personalized and professional. Any PNG/JPG image works!".to_string(),
            action: "Set GRUB_BACKGROUND=/path/to/image.png".to_string(),
            command: None, // Needs image file path
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "beautification".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/GRUB/Tips_and_tricks#Background_image_and_bitmap_fonts".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    result
}

/// Check for VPN tools
fn check_vpn_tools() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for WireGuard
    let has_wireguard = Command::new("pacman")
        .args(&["-Q", "wireguard-tools"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    // Check for OpenVPN
    let has_openvpn = Command::new("pacman")
        .args(&["-Q", "openvpn"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_wireguard && !has_openvpn {
        result.push(Advice {
            id: "vpn-wireguard".to_string(),
            title: "Consider installing WireGuard for modern VPN".to_string(),
            reason: "WireGuard is the modern, fast, and secure VPN protocol! It's built into the Linux kernel, incredibly fast (faster than OpenVPN), and super easy to configure. Perfect for secure remote access, privacy, or connecting to VPN services. Way simpler than OpenVPN!".to_string(),
            action: "Install WireGuard tools".to_string(),
            command: Some("pacman -S --noconfirm wireguard-tools".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "networking".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/WireGuard".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    // Check for NetworkManager VPN plugins
    let has_nm_vpn = Command::new("pacman")
        .args(&["-Q", "networkmanager-openvpn"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if has_openvpn && !has_nm_vpn {
        result.push(Advice {
            id: "vpn-nm-plugin".to_string(),
            title: "Install NetworkManager VPN plugin".to_string(),
            reason: "You have OpenVPN but no NetworkManager plugin! The plugin adds VPN support to NetworkManager's GUI - you can import .ovpn files and connect to VPNs with a single click instead of command line. Much more convenient!".to_string(),
            action: "Install NetworkManager OpenVPN plugin".to_string(),
            command: Some("pacman -S --noconfirm networkmanager-openvpn".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "networking".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/NetworkManager#VPN_support".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    result
}

/// Check browser recommendations
fn check_browser_recommendations() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for browsers
    let has_firefox = Command::new("pacman").args(&["-Q", "firefox"]).output().map(|o| o.status.success()).unwrap_or(false);
    let has_chromium = Command::new("pacman").args(&["-Q", "chromium"]).output().map(|o| o.status.success()).unwrap_or(false);
    let has_chrome = Command::new("pacman").args(&["-Q", "google-chrome"]).output().map(|o| o.status.success()).unwrap_or(false);

    if has_firefox {
        // Suggest uBlock Origin reminder
        result.push(Advice {
            id: "browser-firefox-ublock".to_string(),
            title: "Reminder: Install uBlock Origin in Firefox".to_string(),
            reason: "You have Firefox! Make sure to install uBlock Origin extension for ad blocking and privacy. It's the best ad blocker - blocks ads, trackers, and malware. Essential for web browsing today! Also consider Privacy Badger and HTTPS Everywhere.".to_string(),
            action: "Install uBlock Origin from Firefox Add-ons".to_string(),
            command: None, // Browser extension
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "utilities".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Firefox#Privacy".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    if !has_firefox && !has_chromium && !has_chrome {
        result.push(Advice {
            id: "browser-install".to_string(),
            title: "Install a web browser".to_string(),
            reason: "No web browser detected! You need a browser to access the web. Choose based on your privacy and performance preferences.".to_string(),
            action: "Install Firefox (recommended) or choose an alternative".to_string(),
            command: Some("pacman -S --noconfirm firefox".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "utilities".to_string(),
            alternatives: vec![
                Alternative {
                    name: "Firefox".to_string(),
                    description: "Privacy-focused, open-source, excellent extension support, independent engine (Gecko)".to_string(),
                    install_command: "pacman -S --noconfirm firefox".to_string(),
                },
                Alternative {
                    name: "Chromium".to_string(),
                    description: "Fast, open-source base of Chrome, Blink engine, without Google services".to_string(),
                    install_command: "pacman -S --noconfirm chromium".to_string(),
                },
                Alternative {
                    name: "LibreWolf".to_string(),
                    description: "Firefox fork with enhanced privacy, no telemetry, uBlock Origin built-in".to_string(),
                    install_command: "yay -S --noconfirm librewolf-bin".to_string(),
                },
            ],
                wiki_refs: vec!["https://wiki.archlinux.org/title/Firefox".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    result
}

/// Check security tools
fn check_security_tools() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for rkhunter (rootkit detection)
    let has_rkhunter = Command::new("pacman")
        .args(&["-Q", "rkhunter"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_rkhunter {
        result.push(Advice {
            id: "security-rkhunter".to_string(),
            title: "Consider rkhunter for rootkit detection".to_string(),
            reason: "rkhunter scans your system for rootkits, backdoors, and security issues! It checks for suspicious files, hidden processes, and system modifications. Run it monthly to catch compromises early. Think of it as a security health check!".to_string(),
            action: "Install rkhunter".to_string(),
            command: Some("pacman -S --noconfirm rkhunter".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "security".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Rkhunter".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    // Check for ClamAV (antivirus)
    let has_clamav = Command::new("pacman")
        .args(&["-Q", "clamav"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_clamav {
        result.push(Advice {
            id: "security-clamav".to_string(),
            title: "Consider ClamAV for antivirus scanning".to_string(),
            reason: "While Linux malware is rare, ClamAV is useful for scanning files from Windows users or downloaded files! It catches Windows viruses in shared files, email attachments, and USB drives. Protects your Windows-using friends when you share files!".to_string(),
            action: "Install ClamAV".to_string(),
            command: Some("pacman -S --noconfirm clamav".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "security".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/ClamAV".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    // Check for LUKS encrypted partitions
    let has_encrypted = Command::new("lsblk")
        .args(&["-o", "TYPE"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("crypt"))
        .unwrap_or(false);

    if has_encrypted {
        result.push(Advice {
            id: "security-luks-reminder".to_string(),
            title: "LUKS encryption detected - Remember your backup!".to_string(),
            reason: "You're using LUKS encryption - great for security! Remember: if you lose your encryption password, your data is GONE FOREVER. Make sure you have a secure backup of your passphrase. Consider using a password manager or writing it down in a safe place!".to_string(),
            action: "Verify you have passphrase backup".to_string(),
            command: None, // Manual reminder
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "security".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Dm-crypt".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    result
}

/// Check backup solutions
fn check_backup_solutions() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for rsync (basic backup)
    let has_rsync = Command::new("pacman")
        .args(&["-Q", "rsync"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_rsync {
        result.push(Advice {
            id: "backup-rsync".to_string(),
            title: "Install rsync for file synchronization and backups".to_string(),
            reason: "You don't have rsync! It's THE tool for backups and file syncing - efficient, incremental, and powerful. Perfect for backing up to external drives, NAS, or remote servers. Everyone should have this! 'rsync -av source/ destination/' is all you need for basic backups.".to_string(),
            action: "Install rsync".to_string(),
            command: Some("pacman -S --noconfirm rsync".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "utilities".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Rsync".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    // Check for borg backup
    let has_borg = Command::new("pacman")
        .args(&["-Q", "borg"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_borg && has_rsync {
        result.push(Advice {
            id: "backup-borg".to_string(),
            title: "Consider BorgBackup for encrypted backups".to_string(),
            reason: "Borg is an AMAZING backup tool! It does deduplication (saves tons of space), encryption, compression, and makes backups super fast. You can keep dozens of snapshots without using much disk space. Way better than rsync for regular backups!".to_string(),
            action: "Install BorgBackup".to_string(),
            command: Some("pacman -S --noconfirm borg".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "utilities".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Borg_backup".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    result
}

/// Check screen recording tools
fn check_screen_recording() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for OBS Studio
    let has_obs = Command::new("pacman")
        .args(&["-Q", "obs-studio"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_obs {
        result.push(Advice {
            id: "recording-obs".to_string(),
            title: "Install OBS Studio for screen recording and streaming".to_string(),
            reason: "OBS Studio is THE tool for screen recording, streaming, and video capture! Record tutorials, gameplay, video calls, or live stream to Twitch/YouTube. It's professional-grade software used by streamers worldwide. Captures screen, webcam, audio, and more with tons of customization!".to_string(),
            action: "Install OBS Studio".to_string(),
            command: Some("pacman -S --noconfirm obs-studio".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "multimedia".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Open_Broadcaster_Software".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    // Check for SimpleScreenRecorder (lighter alternative)
    let has_ssr = Command::new("pacman")
        .args(&["-Q", "simplescreenrecorder"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_obs && !has_ssr {
        result.push(Advice {
            id: "recording-ssr".to_string(),
            title: "Or try SimpleScreenRecorder for easy recording".to_string(),
            reason: "Want something simpler than OBS? SimpleScreenRecorder is lightweight and easy - just open, select area, and record! Great for quick screen recordings, tutorials, or capturing bugs. Less features than OBS but way simpler to use!".to_string(),
            action: "Install SimpleScreenRecorder".to_string(),
            command: Some("pacman -S --noconfirm simplescreenrecorder".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "multimedia".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Screen_capture#SimpleScreenRecorder".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    result
}

/// Check password managers
fn check_password_managers() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for KeePassXC
    let has_keepass = Command::new("pacman")
        .args(&["-Q", "keepassxc"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    // Check for Bitwarden
    let has_bitwarden = Command::new("pacman")
        .args(&["-Q", "bitwarden"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_keepass && !has_bitwarden {
        result.push(Advice {
            id: "password-manager-keepass".to_string(),
            title: "Install a password manager (KeePassXC recommended)".to_string(),
            reason: "You don't have a password manager! In 2025, this is ESSENTIAL for security! KeePassXC stores all passwords in an encrypted database - you only need to remember one master password. Generate strong unique passwords for every site, sync across devices, auto-fill forms. Stop reusing passwords!".to_string(),
            action: "Install KeePassXC".to_string(),
            command: Some("pacman -S --noconfirm keepassxc".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "security".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/KeePass".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    result
}

/// Check gaming enhancements
fn check_gaming_enhancements() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if Steam is installed
    let has_steam = Command::new("pacman")
        .args(&["-Q", "steam"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if has_steam {
        // Check for Proton-GE (better Windows game compatibility)
        result.push(Advice {
            id: "gaming-proton-ge".to_string(),
            title: "Consider Proton-GE for better game compatibility".to_string(),
            reason: "You have Steam! Proton-GE is a community version of Proton with extra patches, better codec support, and fixes for specific games. Many Windows games run better on Proton-GE than stock Proton! Install via ProtonUp-Qt for easy management.".to_string(),
            action: "Install ProtonUp-Qt to manage Proton-GE".to_string(),
            command: Some("pacman -S --noconfirm protonup-qt".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "gaming".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Steam#Proton".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });

        // Check for MangoHud (performance overlay)
        let has_mangohud = Command::new("pacman")
            .args(&["-Q", "mangohud"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_mangohud {
            result.push(Advice {
                id: "gaming-mangohud".to_string(),
                title: "Install MangoHud for in-game performance overlay".to_string(),
                reason: "MangoHud shows FPS, CPU/GPU usage, temps, and more while gaming! It's like MSI Afterburner for Linux - see exactly how your games perform. Launch games with 'mangohud %command%' in Steam to enable it. Essential for PC gamers!".to_string(),
                action: "Install MangoHud".to_string(),
                command: Some("pacman -S --noconfirm mangohud".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "gaming".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/MangoHud".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    // Check for Wine
    let has_wine = Command::new("pacman")
        .args(&["-Q", "wine"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_wine && !has_steam {
        result.push(Advice {
            id: "gaming-wine".to_string(),
            title: "Install Wine to run Windows applications".to_string(),
            reason: "Wine lets you run Windows programs on Linux! Great for games, old software, or Windows-only apps. For gaming, Steam's Proton is easier, but Wine works for non-Steam games and applications. 'wine program.exe' is all you need!".to_string(),
            action: "Install Wine".to_string(),
            command: Some("pacman -S --noconfirm wine".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "gaming".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Wine".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    result
}

/// Check Android integration
fn check_android_integration() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for KDE Connect
    let has_kdeconnect = Command::new("pacman")
        .args(&["-Q", "kdeconnect"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_kdeconnect {
        result.push(Advice {
            id: "mobile-kdeconnect".to_string(),
            title: "Install KDE Connect for phone integration".to_string(),
            reason: "KDE Connect is AMAZING for phone integration! Get phone notifications on your PC, send/receive texts, share clipboard, transfer files, use phone as remote control, and more. Works with Android (and iOS via third-party). Makes your phone and PC work together seamlessly!".to_string(),
            action: "Install KDE Connect".to_string(),
            command: Some("pacman -S --noconfirm kdeconnect".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "utilities".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/KDE_Connect".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    // Check for scrcpy (screen mirroring)
    let has_scrcpy = Command::new("pacman")
        .args(&["-Q", "scrcpy"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_scrcpy {
        result.push(Advice {
            id: "mobile-scrcpy".to_string(),
            title: "Install scrcpy for Android screen mirroring".to_string(),
            reason: "scrcpy mirrors your Android screen to your PC with low latency! Control your phone from your computer - great for demos, testing apps, or just using your phone on a big screen. Works over USB or WiFi. Super smooth and fast!".to_string(),
            action: "Install scrcpy".to_string(),
            command: Some("pacman -S --noconfirm scrcpy".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "utilities".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Android#Screen_mirroring".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    result
}

/// Check for text editor improvements (Vim/Neovim)
fn check_text_editors() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check command history for vim usage
    let vim_usage = check_command_usage(&["vim", "vi"]);

    if vim_usage > 10 {
        // User uses vim frequently, suggest neovim
        let has_neovim = Command::new("pacman")
            .args(&["-Q", "neovim"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_neovim {
            result.push(Advice {
                id: "editor-neovim".to_string(),
                title: "Upgrade to Neovim for modern Vim experience".to_string(),
                reason: format!("You use vim {} times in your history! Neovim is vim with modern features: built-in LSP support, better async performance, Lua scripting, Tree-sitter for syntax highlighting, and an active plugin ecosystem. It's fully compatible with vim configs but way more powerful. Think of it as vim 2.0!", vim_usage),
                action: "Install Neovim".to_string(),
                command: Some("pacman -S --noconfirm neovim".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "development".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Neovim".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    result
}

/// Check for mail clients
fn check_mail_clients() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if user has email-related packages but no GUI client
    let has_thunderbird = Command::new("pacman")
        .args(&["-Q", "thunderbird"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let has_evolution = Command::new("pacman")
        .args(&["-Q", "evolution"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_thunderbird && !has_evolution {
        result.push(Advice {
            id: "mail-thunderbird".to_string(),
            title: "Install Thunderbird for email management".to_string(),
            reason: "Need an email client? Thunderbird is Mozilla's excellent email app! It handles multiple accounts (Gmail, Outlook, custom IMAP), has great spam filtering, calendar integration, and full PGP encryption support. Modern, fast, and privacy-focused. Perfect for managing all your email in one place!".to_string(),
            action: "Install Thunderbird".to_string(),
            command: Some("pacman -S --noconfirm thunderbird".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "productivity".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Thunderbird".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    result
}

/// Check for file sharing (Samba/NFS)
fn check_file_sharing() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for Samba (Windows file sharing)
    let has_samba = Command::new("pacman")
        .args(&["-Q", "samba"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_samba {
        result.push(Advice {
            id: "fileshare-samba".to_string(),
            title: "Install Samba for Windows file sharing".to_string(),
            reason: "Samba lets you share files with Windows computers on your network! Super useful for mixed Windows/Linux environments. Share folders, access Windows shares, print to network printers. It's how Linux speaks 'Windows file sharing' - essential for home networks or offices with Windows machines!".to_string(),
            action: "Install Samba".to_string(),
            command: Some("pacman -S --noconfirm samba".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "networking".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Samba".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    // Check for NFS utilities
    let has_nfs = Command::new("pacman")
        .args(&["-Q", "nfs-utils"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_nfs {
        result.push(Advice {
            id: "fileshare-nfs".to_string(),
            title: "Install NFS for Unix/Linux file sharing".to_string(),
            reason: "NFS (Network File System) is the native Linux/Unix way to share files across networks! Much faster than Samba for Linux-to-Linux sharing. Great for home servers, NAS devices, or accessing files from multiple Linux machines. Works seamlessly with proper permissions!".to_string(),
            action: "Install NFS utilities".to_string(),
            command: Some("pacman -S --noconfirm nfs-utils".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "networking".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/NFS".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    result
}

/// Check for cloud storage tools
fn check_cloud_storage() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for rclone (universal cloud sync)
    let has_rclone = Command::new("pacman")
        .args(&["-Q", "rclone"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_rclone {
        result.push(Advice {
            id: "cloud-rclone".to_string(),
            title: "Install rclone for cloud storage sync".to_string(),
            reason: "rclone is like rsync for cloud storage! It supports 40+ cloud providers (Google Drive, Dropbox, OneDrive, S3, Backblaze, etc.). Sync, copy, mount cloud storage as local drive, encrypt files, compare directories. Think of it as the Swiss Army knife for cloud storage. One tool to rule them all!".to_string(),
            action: "Install rclone".to_string(),
            command: Some("pacman -S --noconfirm rclone".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "utilities".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Rclone".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    result
}

/// Check for programming language support - Go
fn check_golang_dev() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for Go files
    let has_go_files = Path::new(&format!("{}/.cache", std::env::var("HOME").unwrap_or_default())).exists()
        && Command::new("find")
            .args(&[&std::env::var("HOME").unwrap_or_default(), "-name", "*.go", "-type", "f"])
            .output()
            .map(|o| !o.stdout.is_empty())
            .unwrap_or(false);

    let go_usage = check_command_usage(&["go"]);

    if has_go_files || go_usage > 5 {
        // Check for Go compiler
        let has_go = Command::new("pacman")
            .args(&["-Q", "go"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_go {
            result.push(Advice {
                id: "dev-go".to_string(),
                title: "Install Go compiler for Go development".to_string(),
                reason: format!("You have Go files or use 'go' commands ({} times)! Install the Go compiler to build and run Go programs. Go is fast, simple, and great for concurrent programming, web services, and system tools. 'go run main.go' and you're off!", go_usage),
                action: "Install Go".to_string(),
                command: Some("pacman -S --noconfirm go".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "development".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Go".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        } else {
            // Check for gopls (Go LSP)
            let has_gopls = Command::new("which")
                .arg("gopls")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

            if !has_gopls {
                result.push(Advice {
                    id: "dev-gopls".to_string(),
                    title: "Install gopls for Go LSP support".to_string(),
                    reason: "You're developing in Go but missing gopls (Go Language Server)! It provides autocomplete, go-to-definition, refactoring, and error checking in your editor. Works with VSCode, Neovim, Emacs, any LSP-compatible editor. Makes Go development SO much better!".to_string(),
                    action: "Install gopls via go install".to_string(),
                    command: Some("go install golang.org/x/tools/gopls@latest".to_string()),
                    risk: RiskLevel::Low,
                    priority: Priority::Recommended,
                    category: "development".to_string(),
                    alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Go#Language_Server".to_string()],
                                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
            }
        }
    }

    result
}

/// Check for Java development
fn check_java_dev() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for Java files
    let has_java_files = Command::new("find")
        .args(&[&std::env::var("HOME").unwrap_or_default(), "-name", "*.java", "-type", "f"])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    let java_usage = check_command_usage(&["java", "javac", "mvn", "gradle"]);

    if has_java_files || java_usage > 5 {
        // Check for JDK
        let has_jdk = Command::new("pacman")
            .args(&["-Q", "jdk-openjdk"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_jdk {
            result.push(Advice {
                id: "dev-java-jdk".to_string(),
                title: "Install OpenJDK for Java development".to_string(),
                reason: format!("You have Java files or use Java commands ({} times)! OpenJDK is the open-source Java Development Kit - compile and run Java programs, build Android apps, develop enterprise software. 'javac Main.java && java Main' - Java everywhere!", java_usage),
                action: "Install OpenJDK".to_string(),
                command: Some("pacman -S --noconfirm jdk-openjdk".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "development".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Java".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }

        // Check for Maven
        let has_maven = Command::new("pacman")
            .args(&["-Q", "maven"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_maven {
            result.push(Advice {
                id: "dev-maven".to_string(),
                title: "Install Maven for Java project management".to_string(),
                reason: "Maven is the standard build tool for Java! It handles dependencies, builds projects, runs tests, packages JARs. Essential for any serious Java development. If you see a pom.xml file, you need Maven!".to_string(),
                action: "Install Maven".to_string(),
                command: Some("pacman -S --noconfirm maven".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "development".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Java#Maven".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    result
}

/// Check for Node.js development
fn check_nodejs_dev() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for JavaScript/TypeScript files and package.json
    let has_package_json = Path::new(&format!("{}/package.json", std::env::var("HOME").unwrap_or_default())).exists()
        || Command::new("find")
            .args(&[&std::env::var("HOME").unwrap_or_default(), "-name", "package.json", "-type", "f"])
            .output()
            .map(|o| !o.stdout.is_empty())
            .unwrap_or(false);

    let node_usage = check_command_usage(&["node", "npm", "npx", "yarn"]);

    if has_package_json || node_usage > 5 {
        // Check for Node.js
        let has_nodejs = Command::new("pacman")
            .args(&["-Q", "nodejs"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_nodejs {
            result.push(Advice {
                id: "dev-nodejs".to_string(),
                title: "Install Node.js for JavaScript development".to_string(),
                reason: format!("You have Node.js projects or use node/npm commands ({} times)! Node.js runs JavaScript outside browsers - build web apps, CLIs, servers, desktop apps with Electron. Comes with npm for package management. JavaScript everywhere!", node_usage),
                action: "Install Node.js and npm".to_string(),
                command: Some("pacman -S --noconfirm nodejs npm".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "development".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Node.js".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }

        // Check for TypeScript
        let has_typescript = Command::new("npm")
            .args(&["list", "-g", "typescript"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_typescript {
            result.push(Advice {
                id: "dev-typescript".to_string(),
                title: "Install TypeScript for type-safe JavaScript".to_string(),
                reason: "TypeScript adds types to JavaScript - catch bugs before runtime, better IDE support, clearer code. Used by major frameworks (Angular, Vue 3, NestJS). If you're doing serious JavaScript development, TypeScript makes everything better!".to_string(),
                action: "Install TypeScript globally".to_string(),
                command: Some("npm install -g typescript".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "development".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Node.js#TypeScript".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    result
}

/// Check for database support
fn check_databases() -> Vec<Advice> {
    let mut result = Vec::new();

    let db_usage = check_command_usage(&["psql", "mysql", "mongod", "redis-cli", "sqlite3"]);

    if db_usage > 5 {
        // Check for PostgreSQL
        let has_postgresql = Command::new("pacman")
            .args(&["-Q", "postgresql"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_postgresql {
            result.push(Advice {
                id: "database-postgresql".to_string(),
                title: "Install PostgreSQL database".to_string(),
                reason: "PostgreSQL is the world's most advanced open-source database! ACID-compliant, supports JSON, full-text search, geospatial data, and advanced indexing. Great for web apps, data analytics, anything needing a robust relational database. The database developers love!".to_string(),
                action: "Install PostgreSQL".to_string(),
                command: Some("pacman -S --noconfirm postgresql".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "development".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/PostgreSQL".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    result
}

/// Check for web servers
fn check_web_servers() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if user is doing web development
    let has_html_files = Command::new("find")
        .args(&[&std::env::var("HOME").unwrap_or_default(), "-name", "*.html", "-type", "f"])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    let web_usage = check_command_usage(&["nginx", "apache", "httpd"]);

    if has_html_files || web_usage > 3 {
        // Check for nginx
        let has_nginx = Command::new("pacman")
            .args(&["-Q", "nginx"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_nginx {
            result.push(Advice {
                id: "webserver-nginx".to_string(),
                title: "Install nginx web server".to_string(),
                reason: "nginx is a fast, lightweight web server and reverse proxy! Perfect for serving static sites, acting as a load balancer, or proxying to Node.js/Python apps. Used by 40% of the busiest websites. Easy to configure, incredibly fast, and rock-solid stable!".to_string(),
                action: "Install nginx".to_string(),
                command: Some("pacman -S --noconfirm nginx".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "development".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Nginx".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    result
}

/// Check for remote desktop tools
fn check_remote_desktop() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for SSH but suggest VNC for GUI access
    let has_tigervnc = Command::new("pacman")
        .args(&["-Q", "tigervnc"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_tigervnc {
        result.push(Advice {
            id: "remote-vnc".to_string(),
            title: "Install TigerVNC for remote desktop access".to_string(),
            reason: "Need to access your desktop remotely? TigerVNC lets you control your Linux desktop from anywhere! Great for remote work, helping family/friends with tech support, or accessing your home PC from laptop. SSH is for terminal, VNC is for full desktop. Works cross-platform!".to_string(),
            action: "Install TigerVNC".to_string(),
            command: Some("pacman -S --noconfirm tigervnc".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "networking".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/TigerVNC".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    result
}

/// Check for torrent clients
fn check_torrent_clients() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for torrent files
    let has_torrent_files = Command::new("find")
        .args(&[&std::env::var("HOME").unwrap_or_default(), "-name", "*.torrent", "-type", "f"])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    if has_torrent_files {
        // Check for qBittorrent
        let has_qbittorrent = Command::new("pacman")
            .args(&["-Q", "qbittorrent"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_qbittorrent {
            result.push(Advice {
                id: "torrent-qbittorrent".to_string(),
                title: "Install qBittorrent for torrent downloads".to_string(),
                reason: "You have torrent files! qBittorrent is an excellent, ad-free torrent client. Clean interface, built-in search, RSS support, sequential downloading, and full torrent creation. It's like uTorrent but open-source and without the bloat. Perfect for Linux ISOs and other legal torrents!".to_string(),
                action: "Install qBittorrent".to_string(),
                command: Some("pacman -S --noconfirm qbittorrent".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "utilities".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/QBittorrent".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    result
}

/// Check for office suites
fn check_office_suite() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for office documents
    let has_office_files = Command::new("find")
        .args(&[&std::env::var("HOME").unwrap_or_default(), "-name", "*.docx", "-o", "-name", "*.xlsx", "-o", "-name", "*.pptx", "-type", "f"])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    if has_office_files {
        // Check for LibreOffice
        let has_libreoffice = Command::new("pacman")
            .args(&["-Q", "libreoffice-fresh"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_libreoffice {
            result.push(Advice {
                id: "office-libreoffice".to_string(),
                title: "Install LibreOffice for document editing".to_string(),
                reason: "You have Office documents! LibreOffice is a full-featured office suite - Writer (Word), Calc (Excel), Impress (PowerPoint), plus Draw and Base. Opens Microsoft Office files, exports to PDF, fully compatible. It's the gold standard for open-source office software!".to_string(),
                action: "Install LibreOffice".to_string(),
                command: Some("pacman -S --noconfirm libreoffice-fresh".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "productivity".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/LibreOffice".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    result
}

/// Check for graphics software
fn check_graphics_software() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for image files
    let has_image_files = Command::new("find")
        .args(&[&format!("{}/Pictures", std::env::var("HOME").unwrap_or_default()), "-type", "f"])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    if has_image_files {
        // Check for GIMP
        let has_gimp = Command::new("pacman")
            .args(&["-Q", "gimp"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_gimp {
            result.push(Advice {
                id: "graphics-gimp".to_string(),
                title: "Install GIMP for photo editing".to_string(),
                reason: "GIMP is the open-source Photoshop! Professional photo editing, retouching, image manipulation, graphic design. Layers, masks, filters, brushes, everything you need. Used by professional designers and photographers. If you edit images, you need GIMP!".to_string(),
                action: "Install GIMP".to_string(),
                command: Some("pacman -S --noconfirm gimp".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "multimedia".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/GIMP".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }

        // Check for Inkscape (vector graphics)
        let has_inkscape = Command::new("pacman")
            .args(&["-Q", "inkscape"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_inkscape {
            result.push(Advice {
                id: "graphics-inkscape".to_string(),
                title: "Install Inkscape for vector graphics".to_string(),
                reason: "Inkscape is the open-source Illustrator! Create logos, icons, diagrams, illustrations - anything that needs to scale without losing quality. SVG-native, professional features, used by designers worldwide. Perfect companion to GIMP!".to_string(),
                action: "Install Inkscape".to_string(),
                command: Some("pacman -S --noconfirm inkscape".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "multimedia".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Inkscape".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    result
}

/// Check for video editing software
fn check_video_editing() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for video files
    let has_video_files = Command::new("find")
        .args(&[&format!("{}/Videos", std::env::var("HOME").unwrap_or_default()), "-type", "f"])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    if has_video_files {
        // Check for Kdenlive
        let has_kdenlive = Command::new("pacman")
            .args(&["-Q", "kdenlive"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_kdenlive {
            result.push(Advice {
                id: "video-kdenlive".to_string(),
                title: "Install Kdenlive for video editing".to_string(),
                reason: "You have video files! Kdenlive is a powerful, intuitive video editor. Multi-track editing, effects, transitions, color correction, audio mixing. Great for YouTube videos, family movies, or professional projects. It's like Adobe Premiere but free and open-source!".to_string(),
                action: "Install Kdenlive".to_string(),
                command: Some("pacman -S --noconfirm kdenlive".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "multimedia".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Kdenlive".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    result
}

/// Check for music players
fn check_music_players() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for music files
    let has_music_files = Command::new("find")
        .args(&[&format!("{}/Music", std::env::var("HOME").unwrap_or_default()), "-type", "f"])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    if has_music_files {
        // Check for music players
        let has_mpd = Command::new("pacman")
            .args(&["-Q", "mpd"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_mpd {
            result.push(Advice {
                id: "music-mpd".to_string(),
                title: "Install MPD for music playback".to_string(),
                reason: "You have music files! MPD (Music Player Daemon) is a flexible, powerful music server. Control it from your phone, web browser, CLI, or GUI. Gapless playback, playlists, streaming, multiple outputs. It's the audiophile's choice - lightweight and feature-rich!".to_string(),
                action: "Install MPD and ncmpcpp client".to_string(),
                command: Some("pacman -S --noconfirm mpd ncmpcpp".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "multimedia".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Music_Player_Daemon".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    result
}

/// Check for PDF readers
fn check_pdf_readers() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for PDF files
    let has_pdf_files = Command::new("find")
        .args(&[&std::env::var("HOME").unwrap_or_default(), "-name", "*.pdf", "-type", "f"])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    if has_pdf_files {
        // Check for PDF readers
        let has_zathura = Command::new("pacman")
            .args(&["-Q", "zathura"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        let has_okular = Command::new("pacman")
            .args(&["-Q", "okular"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_zathura && !has_okular {
            result.push(Advice {
                id: "pdf-zathura".to_string(),
                title: "Install Zathura for PDF viewing".to_string(),
                reason: "You have PDF files! Zathura is a minimal, vim-like PDF viewer. Keyboard-driven, fast, no bloat. Perfect for reading papers, books, or documents. If you prefer mouse-based, try Okular (KDE) or Evince (GNOME), but Zathura is the power user's choice!".to_string(),
                action: "Install Zathura with plugins".to_string(),
                command: Some("pacman -S --noconfirm zathura zathura-pdf-mupdf".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "utilities".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Zathura".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    result
}

/// Check for multiple monitor setup tools  
fn check_monitor_tools() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if using X11
    let is_x11 = std::env::var("XDG_SESSION_TYPE").unwrap_or_default() == "x11";

    if is_x11 {
        // Check for arandr (GUI for xrandr)
        let has_arandr = Command::new("pacman")
            .args(&["-Q", "arandr"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_arandr {
            result.push(Advice {
                id: "monitor-arandr".to_string(),
                title: "Install arandr for easy monitor configuration".to_string(),
                reason: "arandr is a visual GUI for xrandr! Drag and drop monitors to arrange them, change resolutions, adjust refresh rates. Way easier than typing xrandr commands. Great for laptops with external monitors or multi-monitor desktops!".to_string(),
                action: "Install arandr".to_string(),
                command: Some("pacman -S --noconfirm arandr".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "desktop".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Xrandr".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    result
}

/// Check for systemd timers (cron alternative)
fn check_systemd_timers() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if user has cron jobs but not using systemd timers
    let has_crontab = Command::new("crontab")
        .arg("-l")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if has_crontab {
        result.push(Advice {
            id: "timers-systemd".to_string(),
            title: "Consider using systemd timers instead of cron".to_string(),
            reason: "You have cron jobs! Systemd timers are the modern alternative - better logging, easier debugging, dependency management, and integrated with systemctl. Plus they can run on boot, handle missed runs, and have calendar-based scheduling. Arch recommends timers over cron!".to_string(),
            action: "Learn about systemd timers".to_string(),
            command: None,
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "system".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Systemd/Timers".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    result
}

/// Check for shell alternatives
fn check_shell_alternatives() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check current shell
    let current_shell = std::env::var("SHELL").unwrap_or_default();

    if current_shell.contains("bash") {
        // Suggest fish for beginners
        let has_fish = Command::new("pacman")
            .args(&["-Q", "fish"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_fish {
            result.push(Advice {
                id: "shell-fish".to_string(),
                title: "Try Fish shell for modern shell experience".to_string(),
                reason: "Fish (Friendly Interactive SHell) is amazing! Autosuggestions as you type, syntax highlighting, excellent completions out-of-box, web-based configuration. No setup needed - it just works. Try it with 'fish' command, change default with 'chsh -s /usr/bin/fish'!".to_string(),
                action: "Install Fish shell".to_string(),
                command: Some("pacman -S --noconfirm fish".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "shell".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Fish".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    result
}

/// Check for advanced compression tools
fn check_compression_advanced() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for zstd (modern compression)
    let has_zstd = Command::new("pacman")
        .args(&["-Q", "zstd"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_zstd {
        result.push(Advice {
            id: "compression-zstd".to_string(),
            title: "Install zstd for fast modern compression".to_string(),
            reason: "Zstandard (zstd) is the modern compression algorithm! Faster than gzip with better compression ratios. Used by Facebook, Linux kernel, package managers. Great for backups, archives, or compressing data. Command: 'zstd file' to compress, 'unzstd file.zst' to decompress!".to_string(),
            action: "Install zstd".to_string(),
            command: Some("pacman -S --noconfirm zstd".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "utilities".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Zstd".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    result
}

/// Check for dual boot support
fn check_dual_boot() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for GRUB
    let has_grub = Command::new("which")
        .arg("grub-mkconfig")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if has_grub {
        // Check for os-prober
        let has_os_prober = Command::new("pacman")
            .args(&["-Q", "os-prober"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_os_prober {
            result.push(Advice {
                id: "dualboot-osprober".to_string(),
                title: "Install os-prober for dual boot detection".to_string(),
                reason: "You have GRUB! os-prober automatically detects other operating systems (Windows, other Linux distros) and adds them to GRUB menu. Essential for dual boot setups. After installing, run 'sudo grub-mkconfig -o /boot/grub/grub.cfg' to regenerate config!".to_string(),
                action: "Install os-prober".to_string(),
                command: Some("pacman -S --noconfirm os-prober".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "system".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/GRUB#Detecting_other_operating_systems".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    result
}

/// Check for advanced git configuration
fn check_git_advanced() -> Vec<Advice> {
    let mut result = Vec::new();

    let git_usage = check_command_usage(&["git"]);

    if git_usage > 20 {
        // Check for delta (better git diff)
        let has_delta = Command::new("which")
            .arg("delta")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_delta {
            result.push(Advice {
                id: "git-delta".to_string(),
                title: "Install delta for beautiful git diffs".to_string(),
                reason: format!("You use git {} times! Delta makes git diff beautiful - syntax highlighting, side-by-side diffs, line numbers, better merge conflict visualization. Configure with: git config --global core.pager delta. Your diffs will never be the same!", git_usage),
                action: "Install git-delta".to_string(),
                command: Some("pacman -S --noconfirm git-delta".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "development".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Git#Diff_and_merge_tools".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }

        // Check for lazygit
        let has_lazygit = Command::new("pacman")
            .args(&["-Q", "lazygit"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_lazygit {
            result.push(Advice {
                id: "git-lazygit".to_string(),
                title: "Install lazygit for terminal UI git client".to_string(),
                reason: "lazygit is a gorgeous terminal UI for git! Stage files, create commits, manage branches, resolve conflicts - all with keyboard shortcuts. Way faster than typing git commands. Just run 'lazygit' in any repo. Git power users love it!".to_string(),
                action: "Install lazygit".to_string(),
                command: Some("pacman -S --noconfirm lazygit".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "development".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Git#Graphical_tools".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    result
}

/// Check for container alternatives
fn check_container_alternatives() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for Docker
    let has_docker = Command::new("pacman")
        .args(&["-Q", "docker"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if has_docker {
        // Suggest Podman as alternative
        let has_podman = Command::new("pacman")
            .args(&["-Q", "podman"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_podman {
            result.push(Advice {
                id: "container-podman".to_string(),
                title: "Try Podman as Docker alternative".to_string(),
                reason: "Podman is Docker without the daemon! Rootless by default (more secure), drop-in replacement for Docker CLI. 'alias docker=podman' and you're good. No root daemon, better security, same containers. Great for developers who want Docker-compatible tools without Docker's architecture!".to_string(),
                action: "Install Podman".to_string(),
                command: Some("pacman -S --noconfirm podman".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "development".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Podman".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    result
}

/// Check for code editors
fn check_code_editors() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for development activity
    let has_dev_files = check_command_usage(&["vim", "nano", "code", "emacs"]) > 10;

    if has_dev_files {
        // Check for VS Code
        let has_vscode = Command::new("pacman")
            .args(&["-Q", "code"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_vscode {
            result.push(Advice {
                id: "editor-vscode".to_string(),
                title: "Install Visual Studio Code for modern development".to_string(),
                reason: "VS Code is the most popular code editor! IntelliSense, debugging, Git integration, thousands of extensions, remote development. Works with every language. Industry standard for many developers. The open-source version 'code' is fully featured!".to_string(),
                action: "Install VS Code".to_string(),
                command: Some("pacman -S --noconfirm code".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "development".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Visual_Studio_Code".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    result
}

/// Check for additional databases
fn check_additional_databases() -> Vec<Advice> {
    let mut result = Vec::new();

    let db_usage = check_command_usage(&["mysql", "mongod", "redis-cli"]);

    if db_usage > 3 {
        // Check for MySQL/MariaDB
        let has_mysql = Command::new("pacman")
            .args(&["-Q", "mariadb"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_mysql && db_usage > 5 {
            result.push(Advice {
                id: "database-mariadb".to_string(),
                title: "Install MariaDB for MySQL compatibility".to_string(),
                reason: "MariaDB is the drop-in replacement for MySQL! Fully compatible, often faster, more features, truly open-source. Great for web apps, WordPress, Drupal, or any MySQL application. 'systemctl start mariadb' and you're MySQL-compatible!".to_string(),
                action: "Install MariaDB".to_string(),
                command: Some("pacman -S --noconfirm mariadb".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "development".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/MariaDB".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }

        // Check for Redis
        let has_redis = Command::new("pacman")
            .args(&["-Q", "redis"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_redis {
            result.push(Advice {
                id: "database-redis".to_string(),
                title: "Install Redis for in-memory data storage".to_string(),
                reason: "Redis is blazingly fast in-memory database! Perfect for caching, session storage, queues, real-time analytics. Used by Twitter, GitHub, Snapchat. Simple key-value store with rich data types. If your app needs speed, Redis is the answer!".to_string(),
                action: "Install Redis".to_string(),
                command: Some("pacman -S --noconfirm redis".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "development".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Redis".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    result
}

/// Check for network analysis tools
fn check_network_tools() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for advanced network usage
    let net_usage = check_command_usage(&["ping", "traceroute", "netstat", "ss"]);

    if net_usage > 10 {
        // Check for Wireshark
        let has_wireshark = Command::new("pacman")
            .args(&["-Q", "wireshark-qt"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_wireshark {
            result.push(Advice {
                id: "network-wireshark".to_string(),
                title: "Install Wireshark for network analysis".to_string(),
                reason: "Wireshark is THE network protocol analyzer! Capture and inspect packets, debug network issues, analyze traffic, learn protocols. Essential for network admins, security researchers, or anyone debugging network problems. GUI and CLI (tshark) included!".to_string(),
                action: "Install Wireshark".to_string(),
                command: Some("pacman -S --noconfirm wireshark-qt".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "networking".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Wireshark".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }

        // Check for nmap
        let has_nmap = Command::new("pacman")
            .args(&["-Q", "nmap"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_nmap {
            result.push(Advice {
                id: "network-nmap".to_string(),
                title: "Install nmap for network scanning".to_string(),
                reason: "nmap is the network exploration tool! Scan networks, discover hosts, identify services, detect OS. Used by security professionals worldwide. 'nmap 192.168.1.0/24' scans your local network. Essential for network administration and security auditing!".to_string(),
                action: "Install nmap".to_string(),
                command: Some("pacman -S --noconfirm nmap".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "networking".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Nmap".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    result
}

/// Check for dotfile managers
fn check_dotfile_managers() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if user has many dotfiles
    let has_many_dotfiles = Command::new("find")
        .args(&[&std::env::var("HOME").unwrap_or_default(), "-maxdepth", "1", "-name", ".*", "-type", "f"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).lines().count() > 10)
        .unwrap_or(false);

    if has_many_dotfiles {
        // Check for GNU Stow
        let has_stow = Command::new("pacman")
            .args(&["-Q", "stow"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_stow {
            result.push(Advice {
                id: "dotfiles-stow".to_string(),
                title: "Install GNU Stow for dotfile management".to_string(),
                reason: "You have lots of dotfiles! GNU Stow makes managing them easy. Keep configs in git repo, use symlinks to deploy. Switch between different configs, share across machines, version control everything. Simple: 'stow vim' creates symlinks from ~/dotfiles/vim/ to ~/. Game changer!".to_string(),
                action: "Install GNU Stow".to_string(),
                command: Some("pacman -S --noconfirm stow".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "utilities".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Dotfiles#Version_control".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    result
}

/// Check for package development tools
fn check_pkgbuild_tools() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if user builds packages
    let builds_packages = Command::new("find")
        .args(&[&std::env::var("HOME").unwrap_or_default(), "-name", "PKGBUILD", "-type", "f"])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    if builds_packages {
        // Check for namcap
        let has_namcap = Command::new("pacman")
            .args(&["-Q", "namcap"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_namcap {
            result.push(Advice {
                id: "pkgbuild-namcap".to_string(),
                title: "Install namcap for PKGBUILD linting".to_string(),
                reason: "You build packages! namcap checks PKGBUILDs for errors, missing dependencies, naming issues, and packaging problems. Essential for AUR maintainers or anyone building custom packages. Run 'namcap PKGBUILD' before uploading to AUR!".to_string(),
                action: "Install namcap".to_string(),
                command: Some("pacman -S --noconfirm namcap".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "development".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Namcap".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }

        // Check for devtools
        let has_devtools = Command::new("pacman")
            .args(&["-Q", "devtools"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_devtools {
            result.push(Advice {
                id: "pkgbuild-devtools".to_string(),
                title: "Install devtools for clean chroot builds".to_string(),
                reason: "Build packages in clean chroots! devtools provides 'extra-x86_64-build' and friends - build in isolated environment, catch missing dependencies, ensure reproducibility. Professional package builders use this. If you're serious about packaging, you need devtools!".to_string(),
                action: "Install devtools".to_string(),
                command: Some("pacman -S --noconfirm devtools".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "development".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/DeveloperWiki:Building_in_a_clean_chroot".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    result
}

/// Check for Python development tools
fn check_python_tools() -> Vec<Advice> {
    let mut result = Vec::new();

    let python_usage = check_command_usage(&["python", "python3", "pip"]);

    if python_usage > 10 {
        // Check for poetry
        let has_poetry = Command::new("which")
            .arg("poetry")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_poetry {
            result.push(Advice {
                id: "python-poetry".to_string(),
                title: "Install Poetry for Python dependency management".to_string(),
                reason: format!("You use Python {} times! Poetry is THE modern Python package manager. No more pip freeze, no more requirements.txt hell. Dependency resolution, virtual environments, publishing - all in one tool. 'poetry add requests' just works!", python_usage),
                action: "Install Poetry".to_string(),
                command: Some("pacman -S --noconfirm python-poetry".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "development".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Python#Package_management".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }

        // Check for virtualenv
        let has_virtualenv = Command::new("pacman")
            .args(&["-Q", "python-virtualenv"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_virtualenv {
            result.push(Advice {
                id: "python-virtualenv".to_string(),
                title: "Install virtualenv for isolated Python environments".to_string(),
                reason: "Virtual environments are essential for Python development! Isolate project dependencies, avoid conflicts, test different versions. Every Python developer needs this. 'python -m venv myenv' creates isolated environment!".to_string(),
                action: "Install python-virtualenv".to_string(),
                command: Some("pacman -S --noconfirm python-virtualenv".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "development".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Python#Virtual_environment".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }

        // Check for ipython
        let has_ipython = Command::new("pacman")
            .args(&["-Q", "ipython"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_ipython {
            result.push(Advice {
                id: "python-ipython".to_string(),
                title: "Install IPython for enhanced Python REPL".to_string(),
                reason: "IPython is Python REPL on steroids! Syntax highlighting, tab completion, magic commands, inline plots, history. Way better than plain 'python' prompt. Data scientists and developers love it. Try 'ipython' and never go back!".to_string(),
                action: "Install IPython".to_string(),
                command: Some("pacman -S --noconfirm ipython".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "development".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Python#IPython".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    result
}

/// Check for Rust development tools
fn check_rust_tools() -> Vec<Advice> {
    let mut result = Vec::new();

    let rust_usage = check_command_usage(&["cargo", "rustc"]);

    if rust_usage > 10 {
        // Check for cargo-watch
        let has_cargo_watch = Command::new("which")
            .arg("cargo-watch")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_cargo_watch {
            result.push(Advice {
                id: "rust-cargo-watch".to_string(),
                title: "Install cargo-watch for automatic rebuilds".to_string(),
                reason: format!("You use Rust {} times! cargo-watch automatically rebuilds on file changes. 'cargo watch -x check -x test' runs checks and tests on every save. Essential for fast development iterations. No more manual cargo build!", rust_usage),
                action: "Install cargo-watch".to_string(),
                command: Some("pacman -S --noconfirm cargo-watch".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "development".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Rust#Cargo".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }

        // Check for cargo-audit
        let has_cargo_audit = Command::new("which")
            .arg("cargo-audit")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_cargo_audit {
            result.push(Advice {
                id: "rust-cargo-audit".to_string(),
                title: "Install cargo-audit for security vulnerability scanning".to_string(),
                reason: "cargo-audit checks your Cargo.lock for known security vulnerabilities! Scans against RustSec database, finds CVEs in dependencies. 'cargo audit' shows security issues. Essential for production Rust code!".to_string(),
                action: "Install cargo-audit".to_string(),
                command: Some("cargo install cargo-audit".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "development".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Rust#Security".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    result
}

/// Check for terminal multiplexers
fn check_terminal_multiplexers() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for tmux
    let has_tmux = Command::new("pacman")
        .args(&["-Q", "tmux"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_tmux {
        result.push(Advice {
            id: "terminal-tmux".to_string(),
            title: "Install tmux for terminal multiplexing".to_string(),
            reason: "tmux is a terminal multiplexer! Split terminals, detach sessions, work across SSH disconnects, multiple windows. Essential for remote work and power users. 'tmux new -s mysession' creates session, Ctrl+b d detaches. Never lose your work again!".to_string(),
            action: "Install tmux".to_string(),
            command: Some("pacman -S --noconfirm tmux".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "utilities".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Tmux".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    result
}

/// Check for image viewers
fn check_image_viewers() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for image files
    let has_images = Command::new("find")
        .args(&[&format!("{}/Pictures", std::env::var("HOME").unwrap_or_default()), "-type", "f"])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    if has_images {
        // Check display server
        let is_x11 = std::env::var("XDG_SESSION_TYPE").unwrap_or_default() == "x11";
        let is_wayland = std::env::var("XDG_SESSION_TYPE").unwrap_or_default() == "wayland";

        if is_x11 {
            let has_feh = Command::new("pacman")
                .args(&["-Q", "feh"])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

            if !has_feh {
                result.push(Advice {
                    id: "image-feh".to_string(),
                    title: "Install feh for lightweight image viewing".to_string(),
                    reason: "You have images! feh is a fast, lightweight image viewer for X11. View images, set wallpapers, create slideshows. Keyboard-driven, minimal, perfect for tiling WMs. 'feh image.jpg' or 'feh --bg-scale wallpaper.jpg' to set wallpaper!".to_string(),
                    action: "Install feh".to_string(),
                    command: Some("pacman -S --noconfirm feh".to_string()),
                    risk: RiskLevel::Low,
                    priority: Priority::Optional,
                    category: "multimedia".to_string(),
                    alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Feh".to_string()],
                                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
            }
        } else if is_wayland {
            let has_imv = Command::new("pacman")
                .args(&["-Q", "imv"])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

            if !has_imv {
                result.push(Advice {
                    id: "image-imv".to_string(),
                    title: "Install imv for Wayland image viewing".to_string(),
                    reason: "You have images and use Wayland! imv is a fast image viewer for Wayland (also works on X11). Lightweight, keyboard-driven, supports multiple formats. Like feh but for Wayland. 'imv image.jpg' to view!".to_string(),
                    action: "Install imv".to_string(),
                    command: Some("pacman -S --noconfirm imv".to_string()),
                    risk: RiskLevel::Low,
                    priority: Priority::Optional,
                    category: "multimedia".to_string(),
                    alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Wayland#Image_viewers".to_string()],
                                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
            }
        }
    }

    result
}

/// Check for documentation tools
fn check_documentation_tools() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for tldr (simplified man pages)
    let has_tldr = Command::new("which")
        .arg("tldr")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_tldr {
        result.push(Advice {
            id: "docs-tldr".to_string(),
            title: "Install tldr for quick command examples".to_string(),
            reason: "tldr gives you practical command examples! Simpler than man pages, shows common use cases. 'tldr tar' shows how to actually use tar. Community-driven, works offline, way faster than googling. Every command should have a tldr page!".to_string(),
            action: "Install tldr".to_string(),
            command: Some("pacman -S --noconfirm tldr".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "utilities".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Tldr-pages".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    result
}

/// Check for disk management tools
fn check_disk_management() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for SMART monitoring
    let has_smartmontools = Command::new("pacman")
        .args(&["-Q", "smartmontools"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_smartmontools {
        result.push(Advice {
            id: "disk-smartmontools".to_string(),
            title: "Install smartmontools for disk health monitoring".to_string(),
            reason: "SMART monitoring detects disk failures before they happen! Check disk health, temperature, error counts. 'smartctl -a /dev/sda' shows all info. Essential for data safety - know when your disk is dying before you lose data!".to_string(),
            action: "Install smartmontools".to_string(),
            command: Some("pacman -S --noconfirm smartmontools".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "system".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/S.M.A.R.T.".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    // Check for gparted
    let has_gparted = Command::new("pacman")
        .args(&["-Q", "gparted"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_gparted {
        result.push(Advice {
            id: "disk-gparted".to_string(),
            title: "Install GParted for partition management".to_string(),
            reason: "GParted is the best GUI for disk partitioning! Resize, move, create, delete partitions. Supports all filesystems. Way easier than fdisk/parted. Essential for dual boot, adding disks, or managing storage. Visual, safe, powerful!".to_string(),
            action: "Install GParted".to_string(),
            command: Some("pacman -S --noconfirm gparted".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "system".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/GParted".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    result
}

/// Check for communication apps
fn check_communication_apps() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for Discord
    let has_discord = Command::new("pacman")
        .args(&["-Q", "discord"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_discord {
        result.push(Advice {
            id: "chat-discord".to_string(),
            title: "Install Discord for gaming and community chat".to_string(),
            reason: "Discord is THE platform for gaming communities, developer groups, and online communities. Voice chat, screen sharing, servers, bots. Whether you're gaming, learning, or collaborating - Discord is where communities live!".to_string(),
            action: "Install Discord".to_string(),
            command: Some("pacman -S --noconfirm discord".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "communication".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Discord".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    result
}

/// Check for scientific computing tools
fn check_scientific_tools() -> Vec<Advice> {
    let mut result = Vec::new();

    let python_usage = check_command_usage(&["python", "python3"]);

    if python_usage > 20 {
        // Check for Jupyter
        let has_jupyter = Command::new("pacman")
            .args(&["-Q", "jupyter-notebook"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_jupyter {
            result.push(Advice {
                id: "science-jupyter".to_string(),
                title: "Install Jupyter for interactive Python notebooks".to_string(),
                reason: "Jupyter notebooks are essential for data science! Interactive Python, inline plots, markdown notes, shareable. Used by researchers, data scientists, educators worldwide. 'jupyter notebook' starts web interface. Perfect for analysis, teaching, or exploration!".to_string(),
                action: "Install Jupyter Notebook".to_string(),
                command: Some("pacman -S --noconfirm jupyter-notebook".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "development".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Jupyter".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    result
}

/// Check for 3D graphics tools
fn check_3d_tools() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for image files that might indicate 3D work
    let does_graphics = Command::new("find")
        .args(&[&std::env::var("HOME").unwrap_or_default(), "-name", "*.blend", "-o", "-name", "*.obj", "-o", "-name", "*.stl"])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    if does_graphics {
        let has_blender = Command::new("pacman")
            .args(&["-Q", "blender"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_blender {
            result.push(Advice {
                id: "3d-blender".to_string(),
                title: "Install Blender for 3D modeling and animation".to_string(),
                reason: "You have 3D files! Blender is THE free 3D creation suite. Modeling, sculpting, animation, rendering, video editing, game creation. Used by professionals for movies, games, architecture. Industry-standard, open-source, incredibly powerful!".to_string(),
                action: "Install Blender".to_string(),
                command: Some("pacman -S --noconfirm blender".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "multimedia".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Blender".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    result
}

/// Check for audio production tools
fn check_audio_production() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for audio files
    let has_audio_files = Command::new("find")
        .args(&[&format!("{}/Music", std::env::var("HOME").unwrap_or_default()), "-type", "f"])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    if has_audio_files {
        // Check for Audacity
        let has_audacity = Command::new("pacman")
            .args(&["-Q", "audacity"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_audacity {
            result.push(Advice {
                id: "audio-audacity".to_string(),
                title: "Install Audacity for audio editing".to_string(),
                reason: "You have audio files! Audacity is the free audio editor. Record, edit, mix, add effects, export to any format. Perfect for podcasts, music editing, audio cleanup. Simple interface, professional results. The go-to audio editor!".to_string(),
                action: "Install Audacity".to_string(),
                command: Some("pacman -S --noconfirm audacity".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "multimedia".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Audacity".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    result
}

/// Check for system monitoring advanced
fn check_system_monitoring_advanced() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for s-tui (stress terminal UI)
    let has_stui = Command::new("pacman")
        .args(&["-Q", "s-tui"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_stui {
        result.push(Advice {
            id: "monitor-stui".to_string(),
            title: "Install s-tui for CPU stress testing and monitoring".to_string(),
            reason: "s-tui is a terminal UI for CPU stress testing! Monitor CPU frequency, temperature, power, utilization in real-time. Built-in stress test. Great for testing cooling, overclocking, or just seeing your CPU at full load. Beautiful TUI interface!".to_string(),
            action: "Install s-tui".to_string(),
            command: Some("pacman -S --noconfirm s-tui stress".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "system".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Stress_testing".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
    }

    result
}

/// Check for CAD software
fn check_cad_software() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for CAD files
    let has_cad_files = Command::new("find")
        .args(&[&std::env::var("HOME").unwrap_or_default(), "-name", "*.scad", "-o", "-name", "*.FCStd"])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    if has_cad_files {
        let has_freecad = Command::new("pacman")
            .args(&["-Q", "freecad"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_freecad {
            result.push(Advice {
                id: "cad-freecad".to_string(),
                title: "Install FreeCAD for parametric 3D modeling".to_string(),
                reason: "You have CAD files! FreeCAD is open-source parametric CAD. Design parts, assemblies, mechanical systems. Great for 3D printing, engineering, product design. Like SolidWorks but free. Parametric means you can easily modify designs!".to_string(),
                action: "Install FreeCAD".to_string(),
                command: Some("pacman -S --noconfirm freecad".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "engineering".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/FreeCAD".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    result
}

/// Check for markdown viewers
fn check_markdown_tools() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for markdown files
    let has_markdown = Command::new("find")
        .args(&[&std::env::var("HOME").unwrap_or_default(), "-name", "*.md", "-type", "f"])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    if has_markdown {
        // Check for glow (markdown renderer)
        let has_glow = Command::new("which")
            .arg("glow")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_glow {
            result.push(Advice {
                id: "markdown-glow".to_string(),
                title: "Install glow for beautiful markdown rendering".to_string(),
                reason: "You have markdown files! glow renders markdown beautifully in the terminal. Syntax highlighting, styled text, images. Read READMEs, documentation, notes in style. 'glow README.md' or just 'glow' to browse. Way better than raw markdown!".to_string(),
                action: "Install glow".to_string(),
                command: Some("pacman -S --noconfirm glow".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "utilities".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/List_of_applications#Markdown".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    result
}

/// Check for Note-taking apps
fn check_note_taking() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for many text/markdown files
    let has_notes = Command::new("find")
        .args(&[&std::env::var("HOME").unwrap_or_default(), "-name", "*.md", "-o", "-name", "*.txt", "-type", "f"])
        .output()
        .map(|o| {
            let count = String::from_utf8_lossy(&o.stdout).lines().count();
            count > 20
        })
        .unwrap_or(false);

    if has_notes {
        let has_obsidian = Command::new("pacman")
            .args(&["-Q", "obsidian"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_obsidian {
            result.push(Advice {
                id: "notes-obsidian".to_string(),
                title: "Install Obsidian for powerful note-taking".to_string(),
                reason: "You have lots of notes! Obsidian is a powerful knowledge base using markdown files. Backlinks, graph view, plugins, themes. Local-first, your files stay yours. Perfect for PKM (Personal Knowledge Management), research, or journaling. Build your second brain!".to_string(),
                action: "Install Obsidian".to_string(),
                command: Some("pacman -S --noconfirm obsidian".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "productivity".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/List_of_applications#Note-taking_organizers".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    result
}

/// Check for container orchestration needs based on docker usage
fn check_container_orchestration() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if docker is heavily used
    let docker_usage = check_command_usage(&["docker run", "docker-compose", "docker build"]);

    if docker_usage > 50 {
        // Check for docker-compose
        let has_compose = Command::new("which")
            .arg("docker-compose")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_compose {
            result.push(Advice {
                id: "containers-compose".to_string(),
                title: "Install Docker Compose for multi-container applications".to_string(),
                reason: format!("You use docker heavily ({}+ times in history)! Docker Compose simplifies running multi-container apps. Define services in YAML, manage with one command. Essential for development environments, microservices, complex stacks. 'docker-compose up' beats managing containers manually!", docker_usage),
                action: "Install Docker Compose".to_string(),
                command: Some("pacman -S --noconfirm docker-compose".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "development".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Docker#Docker_Compose".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    result
}

/// Check for Python development enhancements based on usage
fn check_python_enhancements() -> Vec<Advice> {
    let mut result = Vec::new();

    let python_usage = check_command_usage(&["python", "python3", "pip", "poetry", "virtualenv"]);

    if python_usage > 30 {
        // Check for pyenv (Python version manager)
        let has_pyenv = Command::new("which")
            .arg("pyenv")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_pyenv {
            result.push(Advice {
                id: "python-pyenv".to_string(),
                title: "Install pyenv for managing multiple Python versions".to_string(),
                reason: format!("You code in Python regularly ({}+ commands)! pyenv lets you install and switch between Python versions easily. Per-project Python versions, test across versions, use latest features. Like nvm for Node, but for Python. Essential for serious Python dev!", python_usage),
                action: "Install pyenv".to_string(),
                command: Some("yay -S --noconfirm pyenv".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "development".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Python#Multiple_versions".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    result
}

/// Check for Git workflow enhancements based on usage patterns
fn check_git_workflow_tools() -> Vec<Advice> {
    let mut result = Vec::new();

    let git_usage = check_command_usage(&["git commit", "git push", "git pull", "git log"]);

    if git_usage > 50 {
        // Check for lazygit (Git TUI)
        let has_lazygit = Command::new("which")
            .arg("lazygit")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_lazygit {
            result.push(Advice {
                id: "git-lazygit".to_string(),
                title: "Install lazygit for visual Git operations".to_string(),
                reason: format!("You're a Git power user ({}+ commands)! lazygit is a gorgeous TUI for Git. Visual staging, committing, branching, rebasing. See your repo status at a glance. Much faster than memorizing git commands. Vim-like keybindings, highly productive!", git_usage),
                action: "Install lazygit".to_string(),
                command: Some("pacman -S --noconfirm lazygit".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "development".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Git#Tips_and_tricks".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    result
}

// ============================================================================
// Enhanced Telemetry Recommendations (beta.35+)
// ============================================================================

/// Check CPU temperature and warn if too high
fn check_cpu_temperature(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    if let Some(temp) = facts.hardware_monitoring.cpu_temperature_celsius {
        if temp > 85.0 {
            result.push(Advice {
                id: "cpu-temperature-critical".to_string(),
                title: "CPU Temperature is CRITICAL!".to_string(),
                reason: format!("Your CPU is running at {:.1}°C, which is dangerously high! Prolonged high temperatures can damage your hardware, reduce lifespan, and cause thermal throttling (slower performance). Normal temps: 40-60°C idle, 60-80°C load. You're in the danger zone!", temp),
                action: "Clean dust from fans, improve airflow, check thermal paste, verify cooling system".to_string(),
                command: None,
                risk: RiskLevel::Low,
                priority: Priority::Mandatory,
                category: "hardware".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Fan_speed_control".to_string()],
                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        } else if temp > 75.0 {
            result.push(Advice {
                id: "cpu-temperature-high".to_string(),
                title: "CPU Temperature is High".to_string(),
                reason: format!("Your CPU is running at {:.1}°C, which is higher than ideal. This can cause thermal throttling and reduced performance. Consider cleaning dust from fans or improving case airflow.", temp),
                action: "Monitor temperature, consider cleaning cooling system".to_string(),
                command: None,
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "hardware".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Fan_speed_control".to_string()],
                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    result
}

/// Check disk health from SMART data
fn check_disk_health(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    for disk in &facts.disk_health {
        if disk.has_errors || disk.health_status == "FAILING" {
            result.push(Advice {
                id: format!("disk-health-failing-{}", disk.device.replace("/dev/", "")),
                title: format!("CRITICAL: Disk {} is FAILING!", disk.device),
                reason: format!("SMART data shows disk {} has errors! Reallocated sectors: {}, Pending sectors: {}. This disk is failing and could lose all data at any moment. BACKUP IMMEDIATELY and replace this drive!",
                    disk.device,
                    disk.reallocated_sectors.unwrap_or(0),
                    disk.pending_sectors.unwrap_or(0)),
                action: "BACKUP ALL DATA IMMEDIATELY, then replace this drive".to_string(),
                command: None,
                risk: RiskLevel::Low,
                priority: Priority::Mandatory,
                category: "hardware".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/S.M.A.R.T.".to_string()],
                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        } else if disk.reallocated_sectors.unwrap_or(0) > 0 {
            result.push(Advice {
                id: format!("disk-health-reallocated-{}", disk.device.replace("/dev/", "")),
                title: format!("Disk {} has reallocated sectors", disk.device),
                reason: format!("Disk {} has {} reallocated sectors. This means the disk detected bad sectors and remapped them. This is an early warning sign - backup your data and monitor closely!",
                    disk.device, disk.reallocated_sectors.unwrap_or(0)),
                action: "Backup data and monitor disk health regularly with smartctl".to_string(),
                command: Some(format!("sudo smartctl -a {}", disk.device)),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "hardware".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/S.M.A.R.T.".to_string()],
                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }
    }

    result
}

/// Check for excessive journal errors
fn check_journal_errors(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    let errors = facts.system_health_metrics.journal_errors_last_24h;

    if errors > 100 {
        result.push(Advice {
            id: "journal-errors-excessive".to_string(),
            title: format!("EXCESSIVE system errors detected ({} in 24h)", errors),
            reason: format!("Your system logged {} errors in the last 24 hours! This is abnormal and indicates serious problems. Normal systems have very few errors. Check journalctl to identify failing services or hardware issues.", errors),
            action: "Review system journal for errors and fix underlying issues".to_string(),
            command: Some("journalctl -p err --since '24 hours ago' --no-pager".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Mandatory,
            category: "maintenance".to_string(),
            alternatives: Vec::new(),
            wiki_refs: vec!["https://wiki.archlinux.org/title/Systemd/Journal".to_string()],
            depends_on: Vec::new(),
            related_to: Vec::new(),
            bundle: None,
        });
    } else if errors > 20 {
        result.push(Advice {
            id: "journal-errors-many".to_string(),
            title: format!("Multiple system errors detected ({} in 24h)", errors),
            reason: format!("Your system has {} errors in the last 24 hours. While not critical, this is worth investigating to prevent future problems.", errors),
            action: "Review system journal to identify error sources".to_string(),
            command: Some("journalctl -p err --since '24 hours ago' --no-pager | head -50".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "maintenance".to_string(),
            alternatives: Vec::new(),
            wiki_refs: vec!["https://wiki.archlinux.org/title/Systemd/Journal".to_string()],
            depends_on: Vec::new(),
            related_to: Vec::new(),
            bundle: None,
        });
    }

    result
}

/// Check for degraded services
fn check_degraded_services(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    if !facts.system_health_metrics.degraded_services.is_empty() {
        let services_list = facts.system_health_metrics.degraded_services.join(", ");
        result.push(Advice {
            id: "degraded-services".to_string(),
            title: "Services in degraded state detected".to_string(),
            reason: format!("The following services are in a degraded state: {}. Degraded services may not function properly and should be investigated.", services_list),
            action: "Check service status and logs to identify issues".to_string(),
            command: Some("systemctl status --failed".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "system".to_string(),
            alternatives: Vec::new(),
            wiki_refs: vec!["https://wiki.archlinux.org/title/Systemd#Basic_systemctl_usage".to_string()],
            depends_on: Vec::new(),
            related_to: Vec::new(),
            bundle: None,
        });
    }

    result
}

/// Check for high memory pressure
fn check_memory_pressure(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    match facts.predictive_insights.memory_pressure_risk {
        RiskLevel::High => {
            let available_gb = facts.hardware_monitoring.memory_available_gb;
            result.push(Advice {
                id: "memory-pressure-critical".to_string(),
                title: "CRITICAL: Very low memory available!".to_string(),
                reason: format!("Only {:.1}GB of RAM available! Your system is under severe memory pressure. This causes swap thrashing, slow performance, and potential OOM kills. Close unnecessary programs or add more RAM.", available_gb),
                action: "Close memory-heavy applications or add more RAM".to_string(),
                command: Some("ps aux --sort=-%mem | head -15".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Mandatory,
                category: "performance".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Improving_performance#Memory".to_string()],
                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        },
        RiskLevel::Medium => {
            let available_gb = facts.hardware_monitoring.memory_available_gb;
            result.push(Advice {
                id: "memory-pressure-moderate".to_string(),
                title: "Low memory available".to_string(),
                reason: format!("Only {:.1}GB of RAM available. Your system may start swapping soon, which degrades performance. Consider closing some applications.", available_gb),
                action: "Monitor memory usage and close unnecessary applications".to_string(),
                command: Some("ps aux --sort=-%mem | head -10".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "performance".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Improving_performance#Memory".to_string()],
                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        },
        _ => {}
    }

    // Check OOM events
    if facts.system_health_metrics.oom_events_last_week > 0 {
        result.push(Advice {
            id: "oom-events-detected".to_string(),
            title: format!("{} Out-of-Memory kills in the last week!", facts.system_health_metrics.oom_events_last_week),
            reason: format!("The kernel killed {} processes due to memory exhaustion! This means you're running out of RAM regularly. Add more RAM, reduce workload, or enable zram/swap.", facts.system_health_metrics.oom_events_last_week),
            action: "Add more RAM or enable swap/zram compression".to_string(),
            command: Some("journalctl -k --since '7 days ago' | grep -i 'out of memory'".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Mandatory,
            category: "performance".to_string(),
            alternatives: Vec::new(),
            wiki_refs: vec!["https://wiki.archlinux.org/title/Improving_performance#Memory".to_string()],
            depends_on: Vec::new(),
            related_to: Vec::new(),
            bundle: None,
        });
    }

    result
}

/// Check battery health for laptops
fn check_battery_health(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    if let Some(battery) = &facts.hardware_monitoring.battery_health {
        // Check for critical battery
        if battery.is_critical {
            result.push(Advice {
                id: "battery-critical".to_string(),
                title: "Battery critically low!".to_string(),
                reason: format!("Battery at {}%! Plug in your charger immediately to avoid data loss.", battery.percentage),
                action: "Plug in AC power immediately".to_string(),
                command: None,
                risk: RiskLevel::Low,
                priority: Priority::Mandatory,
                category: "power".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Laptop".to_string()],
                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            });
        }

        // Check battery health degradation
        if let Some(health) = battery.health_percentage {
            if health < 60 {
                result.push(Advice {
                    id: "battery-health-poor".to_string(),
                    title: "Battery health degraded significantly".to_string(),
                    reason: format!("Battery capacity at {}% of design capacity. Your battery holds much less charge than when new. Consider replacing the battery for better runtime.", health),
                    action: "Consider replacing battery or plan for shorter runtime".to_string(),
                    command: None,
                    risk: RiskLevel::Low,
                    priority: Priority::Optional,
                    category: "power".to_string(),
                    alternatives: Vec::new(),
                    wiki_refs: vec!["https://wiki.archlinux.org/title/Laptop#Battery".to_string()],
                    depends_on: Vec::new(),
                    related_to: Vec::new(),
                    bundle: None,
                });
            }
        }

        // Check high cycle count
        if let Some(cycles) = battery.cycles {
            if cycles > 500 {
                result.push(Advice {
                    id: "battery-high-cycles".to_string(),
                    title: format!("Battery has {} charge cycles", cycles),
                    reason: format!("Your battery has completed {} charge cycles. Most laptop batteries are rated for 300-500 cycles before significant degradation. Monitor battery health closely.", cycles),
                    action: "Monitor battery health and plan for eventual replacement".to_string(),
                    command: Some("cat /sys/class/power_supply/BAT0/cycle_count".to_string()),
                    risk: RiskLevel::Low,
                    priority: Priority::Optional,
                    category: "power".to_string(),
                    alternatives: Vec::new(),
                    wiki_refs: vec!["https://wiki.archlinux.org/title/Laptop#Battery".to_string()],
                    depends_on: Vec::new(),
                    related_to: Vec::new(),
                    bundle: None,
                });
            }
        }
    }

    result
}

/// Check for recent service crashes
fn check_service_crashes(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    let crashes = facts.system_health_metrics.recent_crashes.len();

    if crashes > 5 {
        result.push(Advice {
            id: "service-crashes-many".to_string(),
            title: format!("{} service crashes detected in the last week", crashes),
            reason: format!("Multiple services have crashed recently ({} crashes). This indicates system instability. Check logs to identify failing services and fix root causes.", crashes),
            action: "Review service crash logs and fix unstable services".to_string(),
            command: Some("journalctl -p err --since '7 days ago' | grep -i failed".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "system".to_string(),
            alternatives: Vec::new(),
            wiki_refs: vec!["https://wiki.archlinux.org/title/Systemd#Journal".to_string()],
            depends_on: Vec::new(),
            related_to: Vec::new(),
            bundle: None,
        });
    }

    result
}

/// Check for kernel errors
fn check_kernel_errors(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    if !facts.system_health_metrics.kernel_errors.is_empty() {
        result.push(Advice {
            id: "kernel-errors-detected".to_string(),
            title: "Kernel errors detected".to_string(),
            reason: format!("{} kernel errors found in the last 24 hours. Kernel errors can indicate hardware problems, driver issues, or system instability. Review dmesg for details.", facts.system_health_metrics.kernel_errors.len()),
            action: "Review kernel log for hardware or driver issues".to_string(),
            command: Some("journalctl -k -p err --since '24 hours ago' --no-pager".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "system".to_string(),
            alternatives: Vec::new(),
            wiki_refs: vec!["https://wiki.archlinux.org/title/Kernel_parameters".to_string()],
            depends_on: Vec::new(),
            related_to: Vec::new(),
            bundle: None,
        });
    }

    result
}

/// Check disk space predictions
fn check_disk_space_prediction(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    if let Some(prediction) = &facts.predictive_insights.disk_full_prediction {
        if let Some(days) = prediction.days_until_full {
            if days < 30 {
                result.push(Advice {
                    id: format!("disk-space-low-{}", prediction.mount_point.replace("/", "root")),
                    title: format!("Disk {} will be full in ~{} days!", prediction.mount_point, days),
                    reason: format!("At current growth rate ({:.2} GB/day), {} will be full in ~{} days! Low disk space causes system instability, failed updates, and data loss. Clean up now!",
                        prediction.current_growth_gb_per_day, prediction.mount_point, days),
                    action: "Free up disk space or expand storage".to_string(),
                    command: Some(format!("df -h {} && du -sh /* 2>/dev/null | sort -hr | head -20", prediction.mount_point)),
                    risk: RiskLevel::Low,
                    priority: Priority::Mandatory,
                    category: "maintenance".to_string(),
                    alternatives: Vec::new(),
                    wiki_refs: vec!["https://wiki.archlinux.org/title/System_maintenance#Clean_the_filesystem".to_string()],
                    depends_on: Vec::new(),
                    related_to: Vec::new(),
                    bundle: None,
                });
            }
        }
    }

    result
}

/// Check Hyprland + Nvidia configuration (beta.39+)
fn check_hyprland_nvidia_config(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    // Only applicable if using Hyprland and has Nvidia GPU
    if facts.window_manager.as_deref() != Some("Hyprland") {
        return result;
    }

    if !facts.is_nvidia {
        return result;
    }

    // Check if Wayland+Nvidia environment variables are configured
    if !facts.has_wayland_nvidia_support {
        result.push(Advice {
            id: "hyprland-nvidia-env-vars".to_string(),
            title: "Configure Nvidia Environment Variables for Hyprland".to_string(),
            reason: format!(
                "You're running Hyprland with an Nvidia GPU{}, but the required environment variables for Wayland+Nvidia are not configured. \
                This can cause flickering, crashes, and poor performance.",
                if let Some(ver) = &facts.nvidia_driver_version {
                    format!(" (driver {})", ver)
                } else {
                    String::new()
                }
            ),
            action: "Add the following to ~/.config/hypr/hyprland.conf:\n\n\
                env = GBM_BACKEND,nvidia-drm\n\
                env = __GLX_VENDOR_LIBRARY_NAME,nvidia\n\
                env = LIBVA_DRIVER_NAME,nvidia\n\
                env = WLR_NO_HARDWARE_CURSORS,1".to_string(),
            command: None,
            priority: Priority::Mandatory,
            risk: RiskLevel::High,
            category: "desktop".to_string(),
            wiki_refs: vec!["https://wiki.hyprland.org/Nvidia/".to_string()],
            alternatives: vec![],
            depends_on: vec![],
            related_to: vec![],
            bundle: Some("hyprland-nvidia".to_string()),
        });
    }

    result
}

/// Check Wayland + Nvidia configuration
fn check_wayland_nvidia_config(facts: &SystemFacts) -> Vec<Advice> {
    let result = Vec::new();

    if facts.display_server.as_deref() != Some("wayland") || !facts.is_nvidia {
        return result;
    }

    if facts.window_manager.as_deref() == Some("Hyprland") {
        return result; // Already handled
    }

    result
}

/// Window manager recommendations
fn check_window_manager_recommendations(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    match facts.window_manager.as_deref() {
        Some("i3") => {
            if !is_package_installed("rofi") && !is_package_installed("dmenu") {
                result.push(Advice {
                    id: "i3-application-launcher".to_string(),
                    title: "Install Application Launcher for i3".to_string(),
                    reason: "i3 needs an application launcher for quick application access with keyboard shortcuts.".to_string(),
                    action: "Install rofi for modern launcher or dmenu for classic".to_string(),
                    command: Some("sudo pacman -S rofi".to_string()),
                    priority: Priority::Recommended,
                    risk: RiskLevel::Low,
                    category: "desktop".to_string(),
                    wiki_refs: vec!["https://wiki.archlinux.org/title/I3".to_string()],
                    alternatives: vec![],
                    depends_on: vec![],
                    related_to: vec![],
                    bundle: Some("i3-setup".to_string()),
                });
            }
        }
        _ => {}
    }

    result
}

/// Desktop environment recommendations
fn check_desktop_environment_specific(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    match facts.desktop_environment.as_deref() {
        Some("GNOME") => {
            if !is_package_installed("gnome-tweaks") {
                result.push(Advice {
                    id: "gnome-tweaks".to_string(),
                    title: "Install GNOME Tweaks for Customization".to_string(),
                    reason: "GNOME Tweaks provides advanced customization options not available in standard Settings.".to_string(),
                    action: "Install GNOME Tweaks to customize themes, fonts, startup applications, and more".to_string(),
                    command: Some("sudo pacman -S gnome-tweaks".to_string()),
                    priority: Priority::Optional,
                    risk: RiskLevel::Low,
                    category: "desktop".to_string(),
                    wiki_refs: vec!["https://wiki.gnome.org/Apps/Tweaks".to_string()],
                    alternatives: vec![],
                    depends_on: vec![],
                    related_to: vec![],
                    bundle: Some("gnome-enhancements".to_string()),
                });
            }
        }
        _ => {}
    }

    result
}

/// Helper to check if package is installed
fn is_package_installed(package: &str) -> bool {
    std::process::Command::new("pacman")
        .args(&["-Qq", package])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
