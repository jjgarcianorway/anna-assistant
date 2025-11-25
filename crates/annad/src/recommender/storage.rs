//! Storage recommendations

use anna_common::{Advice, Alternative, Priority, RiskLevel, SystemFacts};
use std::path::Path;
use std::process::Command;

pub(crate) fn check_btrfs_maintenance(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if any filesystem is btrfs
    let has_btrfs = facts
        .storage_devices
        .iter()
        .any(|d| d.filesystem == "btrfs");

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
                category: "System Maintenance".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Btrfs".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
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
                        command: Some("mount | grep btrfs".to_string()),
                        risk: RiskLevel::Medium,
                        priority: Priority::Recommended,
                        category: "Performance & Optimization".to_string(),
                        alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Btrfs#Compression".to_string()],
                                    depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
                }

                // Check for noatime (performance optimization)
                if !options.contains("noatime") && !options.contains("relatime") {
                    result.push(Advice {
                        id: "btrfs-noatime".to_string(),
                        title: "Speed up file access with noatime".to_string(),
                        reason: "Every time you read a file, Linux normally writes down when you accessed it. The 'noatime' option turns this off, making your disk faster since it doesn't need to write timestamps constantly.".to_string(),
                        action: "Add noatime to /etc/fstab for faster file operations".to_string(),
                        command: Some("mount | grep btrfs".to_string()),
                        risk: RiskLevel::Low,
                        priority: Priority::Optional,
                        category: "Performance & Optimization".to_string(),
                        alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Btrfs#Mount_options".to_string()],
                                    depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
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
                category: "System Maintenance".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Btrfs#Scrub".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    result
}

pub(crate) fn check_swap() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if swap is active
    let swap_output = Command::new("swapon").arg("--show").output();

    if let Ok(output) = swap_output {
        let swap_info = String::from_utf8_lossy(&output.stdout);

        if swap_info.lines().count() <= 1 {
            // No swap configured
            // Check available RAM
            let mem_output = Command::new("free").args(&["-m"]).output();

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
                                    command: Some("swapon --show".to_string()),
                                    risk: RiskLevel::Low,
                                    priority: Priority::Recommended,
                                    category: "System Maintenance".to_string(),
                                    alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Swap".to_string()],
                                                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
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
                    category: "Performance & Optimization".to_string(),
                    alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Zram".to_string()],
                                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
            }
        }
    }

    result
}

pub(crate) fn check_snapshot_systems(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if user has Btrfs (best for snapshots)
    let has_btrfs = facts
        .storage_devices
        .iter()
        .any(|dev| dev.filesystem.to_lowercase().contains("btrfs"));

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
                category: "System Maintenance".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Snapper".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });

            result.push(Advice {
                id: "snapshot-snapper-grub".to_string(),
                title: "Add Snapper integration to GRUB bootloader".to_string(),
                reason: "After installing Snapper, add grub-btrfs to boot from snapshots. If an update breaks your system, you can boot from a snapshot at the GRUB menu and restore your system. It's like having a time machine for your entire OS!".to_string(),
                action: "Install grub-btrfs for snapshot booting".to_string(),
                command: Some("pacman -S --noconfirm grub-btrfs".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "System Maintenance".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Snapper#Boot_to_snapshots".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
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
                category: "System Maintenance".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Timeshift".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
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
                category: "System Maintenance".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Snapper#Automatic_snapshots".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }

        // Check if snapper is actually configured
        let snapper_configs = std::path::Path::new("/etc/snapper/configs");
        if !snapper_configs.exists()
            || std::fs::read_dir(snapper_configs)
                .map(|mut d| d.next().is_none())
                .unwrap_or(true)
        {
            result.push(Advice {
                id: "snapshot-snapper-config".to_string(),
                title: "Configure Snapper for your root filesystem".to_string(),
                reason: "You installed Snapper but haven't configured it yet! You need to create a config for your root subvolume. Run 'sudo snapper -c root create-config /' to set it up, then it will start taking automatic snapshots. Without configuration, Snapper won't do anything!".to_string(),
                action: "Create Snapper configuration".to_string(),
                command: Some("snapper -c root create-config /".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "System Maintenance".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Snapper#Configuration".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
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
                category: "System Maintenance".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Snapper#Boot_to_snapshots".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    result
}

pub(crate) fn check_ssd_optimizations(facts: &SystemFacts) -> Vec<Advice> {
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
        !line.trim().starts_with('#') && (line.contains("noatime") || line.contains("relatime"))
    });

    if !has_noatime {
        result.push(Advice {
            id: "ssd-noatime".to_string(),
            title: "Enable noatime for SSD performance".to_string(),
            reason: "You have an SSD but 'noatime' isn't set in fstab! By default, Linux updates the access time every time you read a file, which causes extra writes. For SSDs, this is pure overhead with no benefit. Adding 'noatime' to mount options reduces writes and improves performance. It's the #1 SSD optimization!".to_string(),
            action: "Add noatime to fstab mount options".to_string(),
            command: Some("cat /etc/fstab | grep -v '^#' | grep -E 'ext4|xfs|btrfs'".to_string()), // Show current mount options
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "Performance & Optimization".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Fstab#atime_options".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    // Check for discard support (continuous TRIM)
    let has_discard = fstab_content
        .lines()
        .any(|line| !line.trim().starts_with('#') && line.contains("discard"));

    if !has_discard {
        result.push(Advice {
            id: "ssd-discard-option".to_string(),
            title: "Consider enabling continuous TRIM (discard)".to_string(),
            reason: "Your SSD could benefit from the 'discard' mount option! This enables continuous TRIM, which tells the SSD about deleted blocks immediately instead of waiting for a weekly timer. Modern SSDs handle this well and it keeps performance more consistent. Alternative to the periodic fstrim.timer.".to_string(),
            action: "Add discard to mount options (or keep fstrim.timer)".to_string(),
            command: Some("systemctl status fstrim.timer 2>/dev/null || echo 'fstrim.timer not active'".to_string()), // Check if periodic TRIM is enabled
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Performance & Optimization".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Solid_state_drive#Continuous_TRIM".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    result
}

pub(crate) fn check_swap_compression() -> Vec<Advice> {
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
            category: "Performance & Optimization".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Zram".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    result
}

pub(crate) fn check_filesystem_maintenance(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for ext4 filesystems that might need fsck
    let has_ext4 = facts.storage_devices.iter().any(|dev| {
        dev.filesystem.to_lowercase().contains("ext4")
            || dev.filesystem.to_lowercase().contains("ext3")
    });

    if has_ext4 {
        result.push(Advice {
            id: "filesystem-fsck-reminder".to_string(),
            title: "Reminder: Run fsck on ext4 filesystems periodically".to_string(),
            reason: "You're using ext4 - remember to run filesystem checks occasionally! Modern ext4 is reliable, but errors can accumulate over time from power failures or hardware issues. Boot from a live USB and run 'fsck -f /dev/sdXY' on unmounted filesystems yearly to catch problems early.".to_string(),
            action: "Schedule periodic filesystem checks".to_string(),
            command: Some("tune2fs -l $(findmnt -n -o SOURCE /) 2>/dev/null | grep -E 'Last checked|Mount count'".to_string()), // Show last fsck info
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "System Maintenance".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Fsck".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    // Check for Btrfs scrub
    let has_btrfs = facts
        .storage_devices
        .iter()
        .any(|dev| dev.filesystem.to_lowercase().contains("btrfs"));

    if has_btrfs {
        result.push(Advice {
            id: "filesystem-btrfs-scrub".to_string(),
            title: "Run Btrfs scrub for data integrity".to_string(),
            reason: "You're using Btrfs - run scrubs regularly! Scrub reads all data and metadata, verifies checksums, and repairs corruption automatically. It's like a health check for your filesystem. Run 'btrfs scrub start /' monthly to catch bit rot and disk errors before they cause data loss.".to_string(),
            action: "Start Btrfs scrub".to_string(),
            command: Some("btrfs scrub start /".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "System Maintenance".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Btrfs#Scrub".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    result
}

pub(crate) fn check_ssd_trim_status(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    for ssd in &facts.ssd_info {
        if !ssd.trim_enabled {
            result.push(Advice::new(
                format!("ssd-trim-{}", ssd.device.replace("/dev/", "")),
                format!("Enable TRIM for SSD: {}", ssd.device),
                format!(
                    "TRIM not enabled for SSD '{}' ({}). Without TRIM, write performance degrades over time as the drive fills up.",
                    ssd.device, ssd.model
                ),
                "Enable fstrim.timer for weekly TRIM operations to maintain SSD performance and longevity".to_string(),
                Some("sudo systemctl enable --now fstrim.timer".to_string()),
                RiskLevel::Low,
                Priority::Recommended,
                vec!["https://wiki.archlinux.org/title/Solid_state_drive#TRIM".to_string()],
                "performance".to_string(),
            ).with_bundle("SSD Optimization".to_string())
             .with_popularity(85));
        }
    }

    result
}

pub(crate) fn check_swap_optimization(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    if !facts.swap_config.swap_enabled && facts.total_memory_gb < 16.0 {
        result.push(Advice::new(
            "no-swap-low-memory".to_string(),
            "No Swap Configured with Limited RAM".to_string(),
            format!(
                "You have {:.1}GB RAM with no swap. Swap acts as emergency memory and is required for hibernation. zram (compressed RAM swap) is recommended for modern systems.",
                facts.total_memory_gb
            ),
            "Set up zram for fast compressed swap in RAM".to_string(),
            Some("sudo pacman -S --noconfirm zram-generator".to_string()),
            RiskLevel::Low,
            Priority::Optional,
            vec!["https://wiki.archlinux.org/title/Zram".to_string()],
            "system".to_string(),
        ).with_popularity(60));
    }

    if facts.swap_config.swap_enabled && facts.swap_config.swappiness > 60 {
        result.push(Advice::new(
            "swappiness-too-high".to_string(),
            "Swappiness Value Too High for Desktop Use".to_string(),
            format!(
                "Swappiness is {} (default: 60). Lower values (10-20) provide better desktop responsiveness by keeping applications in RAM rather than swapping to disk.",
                facts.swap_config.swappiness
            ),
            "Lower swappiness to 10 for better desktop performance".to_string(),
            Some("echo 'vm.swappiness=10' | sudo tee /etc/sysctl.d/99-swappiness.conf && sudo sysctl -p /etc/sysctl.d/99-swappiness.conf".to_string()),
            RiskLevel::Low,
            Priority::Optional,
            vec!["https://wiki.archlinux.org/title/Swap#Swappiness".to_string()],
            "performance".to_string(),
        ).with_popularity(45));
    }

    if facts.swap_config.swap_enabled
        && !facts.swap_config.zram_enabled
        && facts.swap_config.swap_type != "zram"
    {
        result.push(Advice::new(
            "consider-zram".to_string(),
            "Consider Switching to zram for Better Performance".to_string(),
            format!(
                "You're using {} swap. zram provides compressed swap in RAM which is much faster and improves responsiveness (great for 8GB+ RAM systems).",
                facts.swap_config.swap_type
            ),
            "Switch to zram for faster swap performance".to_string(),
            Some("sudo pacman -S --noconfirm zram-generator && sudo systemctl start systemd-zram-setup@zram0.service".to_string()),
            RiskLevel::Low,
            Priority::Cosmetic,
            vec!["https://wiki.archlinux.org/title/Zram".to_string()],
            "performance".to_string(),
        ).with_bundle("Performance Optimization".to_string())
         .with_popularity(55));
    }

    result
}

