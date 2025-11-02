// Anna v0.12.3 - Arch Linux Advisor Engine
// Deterministic rule-based analysis for system optimization

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::hardware_profile::HardwareProfile;
use crate::package_analysis::PackageInventory;
use crate::storage_btrfs::BtrfsProfile;

/// Advice level (severity)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Info,
    Warn,
    Error,
}

/// Single piece of advice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Advice {
    pub id: String,
    pub level: Level,
    pub category: String,
    pub title: String,
    pub reason: String,
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_cmd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_risk: Option<String>,
    pub refs: Vec<String>,
}

/// Arch Linux advisor
pub struct ArchAdvisor;

impl ArchAdvisor {
    /// Run all advisor rules
    pub fn run(hw: &HardwareProfile, pkg: &PackageInventory, btrfs: Option<&BtrfsProfile>) -> Vec<Advice> {
        let mut advice = Vec::new();

        // System rules (10 total)
        advice.extend(Self::check_nvidia_headers(hw, pkg));
        advice.extend(Self::check_vulkan_stack(hw, pkg));
        advice.extend(Self::check_microcode(hw, pkg));
        advice.extend(Self::check_cpu_governor(hw));
        advice.extend(Self::check_nvme_scheduler(hw));
        advice.extend(Self::check_tlp_power_management(hw, pkg));
        advice.extend(Self::check_swap_config(hw, pkg));
        advice.extend(Self::check_wayland_acceleration(hw, pkg));
        advice.extend(Self::check_aur_helpers(pkg));
        advice.extend(Self::check_orphan_packages(pkg));

        // Storage rules (10 total) - only if Btrfs detected
        if let Some(btrfs) = btrfs {
            if btrfs.detected {
                advice.extend(Self::check_btrfs_layout_missing_snapshots(btrfs, pkg));
                advice.extend(Self::check_pacman_autosnap_missing(btrfs));
                advice.extend(Self::check_grub_btrfs_missing(btrfs));
                advice.extend(Self::check_systemd_boot_snapshots(btrfs));
                advice.extend(Self::check_scrub_overdue(btrfs));
                advice.extend(Self::check_low_free_space(btrfs));
                advice.extend(Self::check_compression_suboptimal(btrfs));
                advice.extend(Self::check_qgroups_disabled(btrfs));
                advice.extend(Self::check_copy_on_write_exceptions(btrfs));
                advice.extend(Self::check_balance_required(btrfs));
            }
        }

        advice
    }

    /// Rule 1: NVIDIA kernel headers mismatch
    fn check_nvidia_headers(hw: &HardwareProfile, pkg: &PackageInventory) -> Vec<Advice> {
        let mut result = Vec::new();

        // Check if NVIDIA GPU present and driver loaded
        let has_nvidia = hw.gpus.iter().any(|gpu| {
            gpu.vendor.as_ref().map(|v| v.contains("NVIDIA")).unwrap_or(false)
                || gpu.driver.as_ref().map(|d| d.contains("nvidia")).unwrap_or(false)
        });

        if !has_nvidia {
            return result;
        }

        // Check if linux-headers installed for running kernel
        let kernel = &hw.kernel;
        let expected_headers = format!("linux-headers");

        let has_headers = pkg.groups.base.iter().any(|p| p.contains("linux-headers"))
            || pkg.groups.base_devel.iter().any(|p| p.contains("linux-headers"));

        if !has_headers && pkg.groups.nvidia.iter().any(|p| p.contains("nvidia-dkms")) {
            result.push(Advice {
                id: "nvidia-headers-missing".to_string(),
                level: Level::Warn,
                category: "drivers".to_string(),
                title: "NVIDIA DKMS requires kernel headers".to_string(),
                reason: format!(
                    "Running kernel {} with nvidia-dkms but linux-headers not installed",
                    kernel
                ),
                action: "Install kernel headers and rebuild DKMS modules".to_string(),
                explain: Some("NVIDIA's DKMS driver dynamically builds kernel modules. Without matching kernel headers, the driver cannot compile and will fail after kernel updates. This causes boot failures or fallback to nouveau.".to_string()),
                fix_cmd: Some("sudo pacman -S linux-headers && sudo dkms autoinstall".to_string()),
                fix_risk: Some("Low - only installs headers and rebuilds modules, no system changes".to_string()),
                refs: vec![
                    "https://wiki.archlinux.org/title/NVIDIA".to_string(),
                    "https://wiki.archlinux.org/title/Dynamic_Kernel_Module_Support".to_string(),
                ],
            });
        }

        result
    }

    /// Rule 2: Vulkan stack missing
    fn check_vulkan_stack(hw: &HardwareProfile, pkg: &PackageInventory) -> Vec<Advice> {
        let mut result = Vec::new();

        let has_gpu = !hw.gpus.is_empty();
        if !has_gpu {
            return result;
        }

        let has_vulkan_loader = pkg.groups.vulkan.iter().any(|p| p.contains("vulkan-icd-loader"));

        if !has_vulkan_loader {
            // Determine vendor-specific ICD
            let vendor_icd = if hw.gpus.iter().any(|g| g.vendor.as_ref().map(|v| v.contains("NVIDIA")).unwrap_or(false)) {
                "nvidia-utils"
            } else if hw.gpus.iter().any(|g| g.vendor.as_ref().map(|v| v.contains("AMD")).unwrap_or(false)) {
                "vulkan-radeon"
            } else if hw.gpus.iter().any(|g| g.vendor.as_ref().map(|v| v.contains("Intel")).unwrap_or(false)) {
                "vulkan-intel"
            } else {
                "vendor-specific ICD"
            };

            result.push(Advice {
                id: "vulkan-missing".to_string(),
                level: Level::Info,
                category: "graphics".to_string(),
                title: "Vulkan stack not installed".to_string(),
                reason: format!("GPU present but Vulkan ICD loader missing"),
                action: format!("Install Vulkan loader and vendor-specific ICD driver"),
                explain: Some("Vulkan is the modern, low-overhead graphics API used by games and GPU-accelerated applications. Without the ICD loader and vendor driver, Vulkan applications will fail to launch.".to_string()),
                fix_cmd: Some(format!("sudo pacman -S vulkan-icd-loader {}", vendor_icd)),
                fix_risk: Some("Low - only installs runtime libraries, no system configuration changes".to_string()),
                refs: vec!["https://wiki.archlinux.org/title/Vulkan".to_string()],
            });
        }

        result
    }

    /// Rule 3: Microcode outdated
    fn check_microcode(hw: &HardwareProfile, pkg: &PackageInventory) -> Vec<Advice> {
        let mut result = Vec::new();

        let cpu_model = hw.cpu.model.as_ref().map(|s| s.to_lowercase()).unwrap_or_default();

        let is_amd = cpu_model.contains("amd");
        let is_intel = cpu_model.contains("intel");

        if is_amd {
            let has_amd_ucode = pkg.groups.base.iter().any(|p| p == "amd-ucode");
            if !has_amd_ucode {
                result.push(Advice {
                    id: "microcode-amd-missing".to_string(),
                    level: Level::Warn,
                    category: "system".to_string(),
                    title: "AMD microcode not installed".to_string(),
                    reason: "AMD CPU detected but amd-ucode package missing".to_string(),
                    action: "Install AMD microcode and update bootloader".to_string(),
                    explain: Some("CPU microcode provides critical security patches and stability fixes from AMD. Missing microcode leaves known CPU vulnerabilities unpatched (e.g., Spectre, Meltdown variants).".to_string()),
                    fix_cmd: Some("sudo pacman -S amd-ucode && sudo grub-mkconfig -o /boot/grub/grub.cfg".to_string()),
                    fix_risk: Some("Low - microcode loads at boot, no runtime impact. Bootloader update is safe.".to_string()),
                    refs: vec!["https://wiki.archlinux.org/title/Microcode".to_string()],
                });
            }
        } else if is_intel {
            let has_intel_ucode = pkg.groups.base.iter().any(|p| p == "intel-ucode");
            if !has_intel_ucode {
                result.push(Advice {
                    id: "microcode-intel-missing".to_string(),
                    level: Level::Warn,
                    category: "system".to_string(),
                    title: "Intel microcode not installed".to_string(),
                    reason: "Intel CPU detected but intel-ucode package missing".to_string(),
                    action: "Install Intel microcode and update bootloader".to_string(),
                    explain: Some("CPU microcode provides critical security patches and stability fixes from Intel. Missing microcode leaves known CPU vulnerabilities unpatched (e.g., Spectre, Meltdown, MDS variants).".to_string()),
                    fix_cmd: Some("sudo pacman -S intel-ucode && sudo grub-mkconfig -o /boot/grub/grub.cfg".to_string()),
                    fix_risk: Some("Low - microcode loads at boot, no runtime impact. Bootloader update is safe.".to_string()),
                    refs: vec!["https://wiki.archlinux.org/title/Microcode".to_string()],
                });
            }
        }

        result
    }

    /// Rule 10: Orphan packages
    fn check_orphan_packages(pkg: &PackageInventory) -> Vec<Advice> {
        let mut result = Vec::new();

        if !pkg.orphans.is_empty() {
            result.push(Advice {
                id: "orphan-packages".to_string(),
                level: Level::Info,
                category: "maintenance".to_string(),
                title: format!("{} orphan packages found", pkg.orphans.len()),
                reason: "Packages installed as dependencies but no longer required".to_string(),
                action: "Review orphan packages and remove if safe".to_string(),
                explain: Some(format!(
                    "Orphan packages were installed as dependencies but are no longer needed. They waste disk space (~{} packages). Review before removing to ensure they're not manually needed.",
                    pkg.orphans.len()
                )),
                fix_cmd: Some("pacman -Qtd  # Review first\nsudo pacman -Rns $(pacman -Qtdq)  # Remove if safe".to_string()),
                fix_risk: Some("Medium - may remove packages you use directly. Always review pacman -Qtd output first.".to_string()),
                refs: vec!["https://wiki.archlinux.org/title/Pacman/Tips_and_tricks#Removing_unused_packages_(orphans)".to_string()],
            });
        }

        result
    }

    /// Rule 5: NVMe I/O scheduler
    fn check_nvme_scheduler(hw: &HardwareProfile) -> Vec<Advice> {
        let mut result = Vec::new();

        let has_nvme = hw.storage.block_devices.iter().any(|d| {
            d.device_type.as_ref().map(|t| t == "nvme").unwrap_or(false)
        });

        if has_nvme {
            // Note: We can't check the actual scheduler without root access to /sys
            // So this is an informational advisory
            result.push(Advice {
                id: "nvme-scheduler".to_string(),
                level: Level::Info,
                category: "performance".to_string(),
                title: "NVMe I/O scheduler recommendation".to_string(),
                reason: "NVMe devices perform best with 'none' or 'none/mq-deadline' scheduler".to_string(),
                action: "Verify NVMe scheduler and set to 'none' for optimal performance".to_string(),
                explain: Some("NVMe SSDs have sophisticated internal scheduling. The kernel 'none' scheduler bypasses legacy block I/O queueing, reducing latency and maximizing IOPS. Using mq-deadline or other schedulers adds unnecessary overhead.".to_string()),
                fix_cmd: Some("cat /sys/block/nvme*/queue/scheduler  # Check current\necho 'ACTION==\"add|change\", KERNEL==\"nvme[0-9]n[0-9]\", ATTR{queue/scheduler}=\"none\"' | sudo tee /etc/udev/rules.d/60-nvme-scheduler.rules\nsudo udevadm control --reload && sudo udevadm trigger".to_string()),
                fix_risk: Some("Low - only affects I/O scheduling, improves performance. Revertible via udev rule removal.".to_string()),
                refs: vec!["https://wiki.archlinux.org/title/Improving_performance#Input/output_schedulers".to_string()],
            });
        }

        result
    }

    /// Rule 4: CPU governor detection
    fn check_cpu_governor(hw: &HardwareProfile) -> Vec<Advice> {
        let mut result = Vec::new();

        // Informational: We can't reliably detect CPU governor without reading /sys files
        // But we can advise on laptops or desktops based on battery presence
        if hw.battery.present {
            result.push(Advice {
                id: "cpu-governor-laptop".to_string(),
                level: Level::Info,
                category: "performance".to_string(),
                title: "CPU governor may be set to powersave on laptop".to_string(),
                reason: "Laptop detected - governor may be 'powersave' which reduces performance".to_string(),
                action: "Check CPU governor and consider 'schedutil' or 'performance' for better responsiveness".to_string(),
                explain: Some("CPU frequency scaling governors control performance vs power usage. 'powersave' limits CPU freq to save battery but reduces performance. 'schedutil' (default modern) or 'performance' provide better responsiveness.".to_string()),
                fix_cmd: Some("cat /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor  # Check current\nsudo pacman -S cpupower\nsudo cpupower frequency-set -g schedutil  # Or 'performance' for max speed".to_string()),
                fix_risk: Some("Low - only affects CPU frequency scaling. 'performance' increases power usage and heat.".to_string()),
                refs: vec!["https://wiki.archlinux.org/title/CPU_frequency_scaling".to_string()],
            });
        }

        result
    }

    /// Rule 6: TLP/auto-cpu-freq power management
    fn check_tlp_power_management(hw: &HardwareProfile, pkg: &PackageInventory) -> Vec<Advice> {
        let mut result = Vec::new();

        // Only advise on laptops
        if !hw.battery.present {
            return result;
        }

        let has_tlp = pkg.groups.base.iter().any(|p| p == "tlp");
        let has_auto_cpufreq = pkg.groups.base.iter().any(|p| p == "auto-cpufreq");
        let has_laptop_mode = pkg.groups.base.iter().any(|p| p.contains("laptop-mode"));

        // No power management tools installed
        if !has_tlp && !has_auto_cpufreq && !has_laptop_mode {
            result.push(Advice {
                id: "power-management-missing".to_string(),
                level: Level::Info,
                category: "power".to_string(),
                title: "No power management tool installed on laptop".to_string(),
                reason: "Laptop detected but no TLP or auto-cpufreq installed".to_string(),
                action: "Install TLP or auto-cpufreq for better battery life".to_string(),
                explain: Some("TLP and auto-cpufreq automatically tune power settings for battery life (CPU scaling, disk power management, USB autosuspend, etc.). Can extend battery life by 20-40%.".to_string()),
                fix_cmd: Some("sudo pacman -S tlp tlp-rdw\nsudo systemctl enable --now tlp.service\nsudo systemctl mask systemd-rfkill.service systemd-rfkill.socket  # Prevent conflicts".to_string()),
                fix_risk: Some("Low - TLP is well-tested. May conflict with other power tools (auto-cpufreq, laptop-mode-tools).".to_string()),
                refs: vec!["https://wiki.archlinux.org/title/TLP".to_string()],
            });
        }

        // Conflict detection
        let mut power_tools = Vec::new();
        if has_tlp { power_tools.push("tlp"); }
        if has_auto_cpufreq { power_tools.push("auto-cpufreq"); }
        if has_laptop_mode { power_tools.push("laptop-mode-tools"); }

        if power_tools.len() > 1 {
            result.push(Advice {
                id: "power-management-conflict".to_string(),
                level: Level::Warn,
                category: "power".to_string(),
                title: "Multiple power management tools detected".to_string(),
                reason: format!("Conflicting power tools installed: {}", power_tools.join(", ")),
                action: "Remove conflicting power management tools (keep only one)".to_string(),
                explain: Some("TLP, auto-cpufreq, and laptop-mode-tools all manage power settings. Running multiple tools causes conflicts, unpredictable behavior, and may reduce battery life instead of improving it.".to_string()),
                fix_cmd: Some("# Choose one tool and remove others. Example for keeping TLP:\nsudo systemctl disable --now auto-cpufreq.service\nsudo pacman -Rns auto-cpufreq laptop-mode-tools".to_string()),
                fix_risk: Some("Low - only removes conflicting packages. Ensure one power tool remains active.".to_string()),
                refs: vec!["https://wiki.archlinux.org/title/Power_management".to_string()],
            });
        }

        result
    }

    /// Rule 7: ZRAM/Swap configuration
    fn check_swap_config(hw: &HardwareProfile, pkg: &PackageInventory) -> Vec<Advice> {
        let mut result = Vec::new();

        // Check if low memory system (<8GB)
        let total_gb = hw.memory.total_gb.unwrap_or(16.0);
        let is_low_memory = total_gb < 8.0;

        // Check if zram installed
        let has_zram = pkg.groups.base.iter().any(|p| p.contains("zram"));

        // Advise ZRAM on low-memory systems
        if is_low_memory && !has_zram {
            result.push(Advice {
                id: "zram-recommended".to_string(),
                level: Level::Info,
                category: "performance".to_string(),
                title: format!("ZRAM recommended for low-memory system ({:.1}GB RAM)", total_gb),
                reason: "Systems with <8GB RAM benefit from ZRAM compressed swap in memory".to_string(),
                action: "Install zram-generator for compressed in-memory swap".to_string(),
                explain: Some("ZRAM creates a compressed block device in RAM for swap. This is faster than disk swap and helps low-memory systems handle memory pressure without thrashing to disk. Typical compression ratio is 2-3x.".to_string()),
                fix_cmd: Some("sudo pacman -S zram-generator\necho -e '[zram0]\\nzram-size = ram / 2\\ncompression-algorithm = zstd' | sudo tee /etc/systemd/zram-generator.conf\nsudo systemctl daemon-reload\nsudo systemctl start systemd-zram-setup@zram0.service".to_string()),
                fix_risk: Some("Low - ZRAM is safe and well-tested. May slightly increase CPU usage during memory pressure.".to_string()),
                refs: vec!["https://wiki.archlinux.org/title/Zram".to_string()],
            });
        }

        result
    }

    /// Rule 8: Wayland/Xorg acceleration
    fn check_wayland_acceleration(hw: &HardwareProfile, pkg: &PackageInventory) -> Vec<Advice> {
        let mut result = Vec::new();

        // Check for NVIDIA GPU with Wayland
        let has_nvidia = hw.gpus.iter().any(|gpu| {
            gpu.vendor.as_ref().map(|v| v.contains("NVIDIA")).unwrap_or(false)
        });

        if !has_nvidia {
            return result;
        }

        // Check if using NVIDIA driver (not nouveau)
        let has_nvidia_driver = pkg.groups.nvidia.iter().any(|p| p.contains("nvidia") && !p.contains("nouveau"));

        if has_nvidia_driver {
            result.push(Advice {
                id: "nvidia-wayland-modesetting".to_string(),
                level: Level::Info,
                category: "graphics".to_string(),
                title: "NVIDIA Wayland modesetting may not be enabled".to_string(),
                reason: "NVIDIA GPU with proprietary driver - DRM modesetting required for Wayland".to_string(),
                action: "Enable NVIDIA DRM kernel modesetting for Wayland support".to_string(),
                explain: Some("Wayland requires DRM kernel modesetting (KMS) for proper operation. NVIDIA's proprietary driver disables KMS by default. Without it, Wayland sessions fail or fall back to Xorg. GBM (Generic Buffer Management) also needs nvidia-drm.modeset=1.".to_string()),
                fix_cmd: Some("echo 'options nvidia-drm modeset=1' | sudo tee /etc/modprobe.d/nvidia-drm-modesetting.conf\nsudo mkinitcpio -P  # Rebuild initramfs\n# Then reboot and verify: cat /sys/module/nvidia_drm/parameters/modeset  # Should show 'Y'".to_string()),
                fix_risk: Some("Medium - requires reboot and initramfs rebuild. May cause boot issues if driver is broken. Keep fallback kernel.".to_string()),
                refs: vec![
                    "https://wiki.archlinux.org/title/NVIDIA#DRM_kernel_mode_setting".to_string(),
                    "https://wiki.archlinux.org/title/Wayland#NVIDIA".to_string(),
                ],
            });
        }

        result
    }

    /// Rule 9: AUR helpers detection
    fn check_aur_helpers(pkg: &PackageInventory) -> Vec<Advice> {
        let mut result = Vec::new();

        // Check if AUR packages exist
        if pkg.aur_packages.is_empty() {
            return result;
        }

        // Check for common AUR helpers
        let has_yay = pkg.groups.base.iter().any(|p| p == "yay");
        let has_paru = pkg.groups.base.iter().any(|p| p == "paru");
        let has_pikaur = pkg.groups.base.iter().any(|p| p == "pikaur");
        let has_aurutils = pkg.groups.base.iter().any(|p| p == "aurutils");

        if !has_yay && !has_paru && !has_pikaur && !has_aurutils {
            result.push(Advice {
                id: "aur-helper-missing".to_string(),
                level: Level::Info,
                category: "maintenance".to_string(),
                title: format!("{} AUR packages detected but no AUR helper installed", pkg.aur_packages.len()),
                reason: "AUR packages present but no yay/paru/pikaur detected for easy updates".to_string(),
                action: "Install an AUR helper (yay or paru recommended)".to_string(),
                explain: Some("AUR helpers automate building and updating AUR packages. Without one, you must manually check for updates, download PKGBUILDs, and run makepkg. Yay and Paru are actively maintained and pacman-compatible.".to_string()),
                fix_cmd: Some("# Install yay:\ngit clone https://aur.archlinux.org/yay.git\ncd yay && makepkg -si\n\n# Or install paru:\ngit clone https://aur.archlinux.org/paru.git\ncd paru && makepkg -si".to_string()),
                fix_risk: Some("Low - AUR helpers are userspace tools. Building from AUR requires reviewing PKGBUILDs first.".to_string()),
                refs: vec!["https://wiki.archlinux.org/title/AUR_helpers".to_string()],
            });
        }

        result
    }

    // ========== STORAGE ADVISOR RULES (Btrfs) ==========

    /// Storage Rule 1: Btrfs layout missing snapshots
    fn check_btrfs_layout_missing_snapshots(btrfs: &BtrfsProfile, pkg: &PackageInventory) -> Vec<Advice> {
        let mut result = Vec::new();

        // Check if snapshots subvolume exists
        let has_snapshot_subvol = btrfs.layout.snapshots_dir.is_some();

        // Check if any snapshot tool is configured
        let has_tool = btrfs.tools.snapper || btrfs.tools.timeshift;

        if !has_snapshot_subvol || !has_tool {
            result.push(Advice {
                id: "btrfs-layout-missing-snapshots".to_string(),
                level: Level::Warn,
                category: "storage".to_string(),
                title: "Btrfs system without snapshot configuration".to_string(),
                reason: if !has_snapshot_subvol {
                    "No snapshots subvolume detected".to_string()
                } else {
                    "No snapshot tool (snapper/timeshift) configured".to_string()
                },
                action: "Install and configure snapshot tool for system recovery".to_string(),
                explain: Some("Btrfs snapshots are instant, zero-copy backups of subvolumes. Without snapshots, you cannot recover from failed updates or accidental deletions. Snapper automates snapshot creation and integrates with pacman.".to_string()),
                fix_cmd: Some("# Install snapper\nsudo pacman -S snapper\n\n# Create config for root\nsudo snapper -c root create-config /\n\n# Create snapshots subvolume if needed\nsudo btrfs subvolume create /.snapshots".to_string()),
                fix_risk: Some("Low - only installs tool and creates config. No system changes until snapshots are taken.".to_string()),
                refs: vec![
                    "https://wiki.archlinux.org/title/Snapper".to_string(),
                    "https://wiki.archlinux.org/title/Btrfs#Snapshots".to_string(),
                ],
            });
        }

        result
    }

    /// Storage Rule 2: Pacman autosnap missing
    fn check_pacman_autosnap_missing(btrfs: &BtrfsProfile) -> Vec<Advice> {
        let mut result = Vec::new();

        if !btrfs.tools.pacman_hook {
            result.push(Advice {
                id: "pacman-autosnap-missing".to_string(),
                level: Level::Info,
                category: "storage".to_string(),
                title: "Pacman snapshot hooks not installed".to_string(),
                reason: "Automatic pre-transaction snapshots not configured".to_string(),
                action: "Install pacman hooks for automatic snapshots before package operations".to_string(),
                explain: Some("Pacman hooks automatically create Btrfs snapshots before package installations, upgrades, and removals. If an update breaks your system, you can boot into a snapshot and rollback. This is essential for safe system updates.".to_string()),
                fix_cmd: Some("# Install snap-pac (integrates with snapper)\nsudo pacman -S snap-pac\n\n# Or create manual hook\nsudo mkdir -p /etc/pacman.d/hooks\ncat << 'EOF' | sudo tee /etc/pacman.d/hooks/90-btrfs-autosnap.hook\n[Trigger]\nOperation = Upgrade\nOperation = Install\nOperation = Remove\nType = Package\nTarget = *\n\n[Action]\nDepends = btrfs-progs\nWhen = PreTransaction\nExec = /usr/local/bin/btrfs-autosnap-pre.sh\nEOF".to_string()),
                fix_risk: Some("Low - only creates snapshots before package operations. Can be disabled anytime.".to_string()),
                refs: vec![
                    "https://wiki.archlinux.org/title/Snapper#Automatic_timeline_snapshots".to_string(),
                    "https://github.com/wesbarnett/snap-pac".to_string(),
                ],
            });
        }

        result
    }

    /// Storage Rule 3: GRUB-btrfs missing on GRUB systems
    fn check_grub_btrfs_missing(btrfs: &BtrfsProfile) -> Vec<Advice> {
        let mut result = Vec::new();

        if btrfs.bootloader.detected == "grub" && !btrfs.bootloader.grub_btrfs_installed {
            result.push(Advice {
                id: "grub-btrfs-missing-on-grub".to_string(),
                level: Level::Info,
                category: "storage".to_string(),
                title: "GRUB without btrfs snapshot boot entries".to_string(),
                reason: "grub-btrfs not installed - cannot boot into snapshots".to_string(),
                action: "Install grub-btrfs for snapshot boot menu entries".to_string(),
                explain: Some("grub-btrfs automatically generates GRUB menu entries for Btrfs snapshots. This allows you to boot into any snapshot directly from the GRUB menu, essential for recovering from failed updates or system breakage.".to_string()),
                fix_cmd: Some("# Install grub-btrfs\nsudo pacman -S grub-btrfs\n\n# Enable automatic regeneration\nsudo systemctl enable --now grub-btrfsd.service\n\n# Regenerate GRUB config\nsudo grub-mkconfig -o /boot/grub/grub.cfg".to_string()),
                fix_risk: Some("Low - only adds menu entries. Does not modify existing boot options.".to_string()),
                refs: vec![
                    "https://github.com/Antynea/grub-btrfs".to_string(),
                    "https://wiki.archlinux.org/title/Btrfs#Mounting_subvolumes".to_string(),
                ],
            });
        }

        result
    }

    /// Storage Rule 4: systemd-boot missing snapshot entries
    fn check_systemd_boot_snapshots(btrfs: &BtrfsProfile) -> Vec<Advice> {
        let mut result = Vec::new();

        if btrfs.bootloader.detected == "systemd-boot" {
            let has_snapshots = !btrfs.layout.subvolumes.iter()
                .filter(|s| s.is_snapshot && s.readonly)
                .collect::<Vec<_>>()
                .is_empty();

            if has_snapshots && btrfs.bootloader.snapshot_entries.is_empty() {
                result.push(Advice {
                    id: "sd-boot-snapshots-missing".to_string(),
                    level: Level::Info,
                    category: "storage".to_string(),
                    title: "systemd-boot without snapshot boot entries".to_string(),
                    reason: "Snapshots exist but no boot entries configured".to_string(),
                    action: "Generate systemd-boot entries for Btrfs snapshots".to_string(),
                    explain: Some("Unlike GRUB, systemd-boot requires manual boot entry creation for snapshots. Generated entries allow you to boot into any read-only snapshot directly from the boot menu for system recovery.".to_string()),
                    fix_cmd: Some("# Generate entries for last 5 snapshots (manual script needed)\n# See /usr/local/bin/btrfs-sdboot-gen.sh\n# Or use: annactl storage btrfs gen-boot-entries --limit 5".to_string()),
                    fix_risk: Some("Low - only creates boot entries. Original entries remain unchanged.".to_string()),
                    refs: vec![
                        "https://wiki.archlinux.org/title/Systemd-boot".to_string(),
                        "https://wiki.archlinux.org/title/Btrfs#Mounting_subvolumes".to_string(),
                    ],
                });
            }
        }

        result
    }

    /// Storage Rule 5: Scrub overdue
    fn check_scrub_overdue(btrfs: &BtrfsProfile) -> Vec<Advice> {
        let mut result = Vec::new();

        if let Some(days) = btrfs.health.last_scrub_days {
            if days > 30 {
                result.push(Advice {
                    id: "scrub-overdue".to_string(),
                    level: Level::Warn,
                    category: "storage".to_string(),
                    title: format!("Btrfs scrub overdue ({} days since last scrub)", days),
                    reason: "Regular scrubs detect and repair data corruption".to_string(),
                    action: "Run Btrfs scrub to verify filesystem integrity".to_string(),
                    explain: Some("Btrfs scrub reads all data and metadata, verifying checksums and repairing corruption. Monthly scrubs catch bit rot and disk errors before they cause data loss. Scrubs run in the background without unmounting.".to_string()),
                    fix_cmd: Some("# Manual scrub (background)\nsudo btrfs scrub start -Bd /\n\n# Or setup monthly timer\nsudo systemctl enable --now btrfs-scrub@-.timer".to_string()),
                    fix_risk: Some("Minimal - read-only operation. May increase I/O load temporarily.".to_string()),
                    refs: vec![
                        "https://wiki.archlinux.org/title/Btrfs#Scrub".to_string(),
                    ],
                });
            }
        }

        result
    }

    /// Storage Rule 6: Low free space
    fn check_low_free_space(btrfs: &BtrfsProfile) -> Vec<Advice> {
        let mut result = Vec::new();

        if btrfs.health.free_percent < 10.0 {
            result.push(Advice {
                id: "low-free-space".to_string(),
                level: Level::Warn,
                category: "storage".to_string(),
                title: format!("Low Btrfs free space ({:.1}% remaining)", btrfs.health.free_percent),
                reason: "Btrfs performs poorly when nearly full".to_string(),
                action: "Free up space or run balance to reclaim unused chunks".to_string(),
                explain: Some("Btrfs allocates space in chunks. When free space drops below 10%, performance degrades and write operations may fail. Delete old snapshots, clear package cache, or run balance to reclaim unallocated chunks.".to_string()),
                fix_cmd: Some("# Delete old snapshots\nsudo snapper -c root list\nsudo snapper -c root delete <id>\n\n# Clear package cache\nsudo pacman -Sc\n\n# Balance to reclaim space\nsudo btrfs balance start -dusage=50 /".to_string()),
                fix_risk: Some("Medium - deleting snapshots is irreversible. Balance is I/O intensive.".to_string()),
                refs: vec![
                    "https://wiki.archlinux.org/title/Btrfs#Balance".to_string(),
                ],
            });
        }

        result
    }

    /// Storage Rule 7: Compression suboptimal
    fn check_compression_suboptimal(btrfs: &BtrfsProfile) -> Vec<Advice> {
        let mut result = Vec::new();

        let is_optimal = match &btrfs.mount_opts.compression {
            Some(comp) => comp.starts_with("zstd"),
            None => false,
        };

        if !is_optimal {
            let current = btrfs.mount_opts.compression.as_ref()
                .map(|s| s.as_str())
                .unwrap_or("none");

            result.push(Advice {
                id: "compression-suboptimal".to_string(),
                level: Level::Info,
                category: "storage".to_string(),
                title: format!("Suboptimal Btrfs compression (current: {})", current),
                reason: "zstd compression provides better performance and ratio than lzo/zlib".to_string(),
                action: "Update fstab to use zstd compression".to_string(),
                explain: Some("Zstd compression (level 3) provides the best balance of compression ratio, speed, and CPU usage on modern systems. It typically reduces disk usage by 30-50% with negligible performance impact. lzo is faster but compresses less; zlib is slower.".to_string()),
                fix_cmd: Some("# Update /etc/fstab\n# Change mount options to include: compress=zstd:3\n# Example:\n# UUID=xxx  /  btrfs  rw,noatime,compress=zstd:3,space_cache=v2  0 0\n\n# Remount (effective after reboot or remount)\nsudo mount -o remount,compress=zstd:3 /".to_string()),
                fix_risk: Some("Low - only affects newly written data. Existing data unchanged unless rewritten.".to_string()),
                refs: vec![
                    "https://wiki.archlinux.org/title/Btrfs#Compression".to_string(),
                ],
            });
        }

        result
    }

    /// Storage Rule 8: Qgroups disabled
    fn check_qgroups_disabled(btrfs: &BtrfsProfile) -> Vec<Advice> {
        let mut result = Vec::new();

        let has_snapshots = !btrfs.layout.subvolumes.iter()
            .filter(|s| s.is_snapshot)
            .collect::<Vec<_>>()
            .is_empty();

        if has_snapshots && !btrfs.health.qgroups_enabled {
            result.push(Advice {
                id: "qgroups-disabled".to_string(),
                level: Level::Info,
                category: "storage".to_string(),
                title: "Btrfs qgroups disabled with snapshots present".to_string(),
                reason: "Cannot track snapshot space usage without qgroups".to_string(),
                action: "Enable qgroups to monitor snapshot disk usage".to_string(),
                explain: Some("Qgroups (quota groups) track space used by each subvolume and snapshot. Without qgroups, you cannot see how much space snapshots consume or enforce quotas. Enabling qgroups allows automatic snapshot cleanup based on space usage.".to_string()),
                fix_cmd: Some("# Enable qgroups\nsudo btrfs quota enable /\n\n# Rescan (may take time on large filesystems)\nsudo btrfs quota rescan /\n\n# Check space usage\nsudo btrfs qgroup show /".to_string()),
                fix_risk: Some("Moderate - qgroups add metadata overhead (~5%). Rescan is I/O intensive.".to_string()),
                refs: vec![
                    "https://wiki.archlinux.org/title/Btrfs#Quota".to_string(),
                ],
            });
        }

        result
    }

    /// Storage Rule 9: Copy-on-write exceptions needed
    fn check_copy_on_write_exceptions(btrfs: &BtrfsProfile) -> Vec<Advice> {
        let mut result = Vec::new();

        // This is informational - we don't detect actual workloads
        // Only suggest if Btrfs is used
        if btrfs.detected {
            result.push(Advice {
                id: "copy-on-write-exceptions".to_string(),
                level: Level::Info,
                category: "storage".to_string(),
                title: "Consider disabling CoW for heavy-write workloads".to_string(),
                reason: "Databases and VM images perform better with CoW disabled".to_string(),
                action: "Use chattr +C for database directories and VM disk images".to_string(),
                explain: Some("Btrfs copy-on-write (CoW) causes fragmentation with random writes. For databases (PostgreSQL, MySQL) and VM disk images, disabling CoW with chattr +C improves performance significantly. Note: disabling CoW disables checksums for those files.".to_string()),
                fix_cmd: Some("# For new directories (before creating files)\nsudo chattr +C /var/lib/postgresql\nsudo chattr +C /var/lib/libvirt/images\nsudo chattr +C /var/lib/docker\n\n# For existing files, must copy to new location\n# CoW attribute only affects new data".to_string()),
                fix_risk: Some("Moderate - disables data checksums for affected files. Only use for known heavy-write workloads.".to_string()),
                refs: vec![
                    "https://wiki.archlinux.org/title/Btrfs#Disabling_CoW".to_string(),
                ],
            });
        }

        result
    }

    /// Storage Rule 10: Balance required
    fn check_balance_required(btrfs: &BtrfsProfile) -> Vec<Advice> {
        let mut result = Vec::new();

        if btrfs.health.needs_balance {
            result.push(Advice {
                id: "balance-required".to_string(),
                level: Level::Warn,
                category: "storage".to_string(),
                title: "Btrfs balance recommended".to_string(),
                reason: "Metadata chunks full or too many small chunks detected".to_string(),
                action: "Run targeted balance to reclaim space".to_string(),
                explain: Some("Btrfs allocates space in chunks. Over time, partially-used chunks accumulate, wasting space. Balance rewrites data to fewer chunks, reclaiming space. A targeted balance (usage filter) is safe and faster than full balance.".to_string()),
                fix_cmd: Some("# Light balance (only rewrite highly fragmented chunks)\nsudo btrfs balance start -dusage=50 -musage=50 /\n\n# Check status\nsudo btrfs balance status /\n\n# If balance running, can pause/resume\nsudo btrfs balance pause /\nsudo btrfs balance resume /".to_string()),
                fix_risk: Some("Moderate - I/O intensive operation. May take hours on large filesystems. Safe to pause.".to_string()),
                refs: vec![
                    "https://wiki.archlinux.org/title/Btrfs#Balance".to_string(),
                ],
            });
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hardware_profile::*;
    use crate::package_analysis::*;

    fn mock_hardware() -> HardwareProfile {
        HardwareProfile {
            version: "1".to_string(),
            generated_at: "2025-01-01T00:00:00Z".to_string(),
            kernel: "6.17.6-arch1-1".to_string(),
            board: BoardInfo {
                vendor: None,
                product: None,
                bios_date: None,
            },
            cpu: CpuInfo {
                model: Some("AMD Ryzen 9 5900X".to_string()),
                sockets: Some(1),
                cores_total: Some(16),
            },
            memory: MemoryInfo { total_gb: Some(32.0) },
            battery: BatteryInfo {
                present: false,
                count: 0,
            },
            gpus: vec![],
            network: vec![],
            storage: StorageInfo {
                controller: vec![],
                block_devices: vec![],
            },
            usb: vec![],
            notes: vec![],
        }
    }

    fn mock_packages() -> PackageInventory {
        PackageInventory {
            version: "1".to_string(),
            generated_at: "2025-01-01T00:00:00Z".to_string(),
            total_packages: 500,
            explicit_packages: 200,
            orphans: vec![],
            aur_packages: vec![],
            groups: PackageGroups {
                base: vec!["bash".to_string(), "coreutils".to_string()],
                base_devel: vec![],
                xorg: vec![],
                multimedia: vec![],
                nvidia: vec![],
                vulkan: vec![],
                cuda: vec![],
            },
            recent_events: vec![],
        }
    }

    #[test]
    fn test_microcode_amd_missing() {
        let hw = mock_hardware();
        let pkg = mock_packages();

        let advice = ArchAdvisor::check_microcode(&hw, &pkg);
        assert_eq!(advice.len(), 1);
        assert_eq!(advice[0].id, "microcode-amd-missing");
        assert_eq!(advice[0].level, Level::Warn);
    }

    #[test]
    fn test_orphan_packages() {
        let hw = mock_hardware();
        let mut pkg = mock_packages();
        pkg.orphans = vec!["old-lib".to_string(), "unused-dep".to_string()];

        let advice = ArchAdvisor::check_orphan_packages(&pkg);
        assert_eq!(advice.len(), 1);
        assert_eq!(advice[0].id, "orphan-packages");
        assert!(advice[0].title.contains("2 orphan"));
    }

    #[test]
    fn test_nvidia_vulkan() {
        let mut hw = mock_hardware();
        hw.gpus.push(GpuInfo {
            name: "NVIDIA RTX 3080".to_string(),
            vendor: Some("NVIDIA".to_string()),
            driver: Some("nvidia".to_string()),
            vram_mb: Some(10240),
            notes: vec![],
        });

        let pkg = mock_packages();

        let advice = ArchAdvisor::check_vulkan_stack(&hw, &pkg);
        assert_eq!(advice.len(), 1);
        assert_eq!(advice[0].id, "vulkan-missing");
        assert!(advice[0].fix_cmd.as_ref().unwrap().contains("nvidia-utils"));
    }

    #[test]
    fn test_nvme_scheduler() {
        let mut hw = mock_hardware();
        hw.storage.block_devices.push(BlockDevice {
            name: "nvme0n1".to_string(),
            model: Some("Samsung 980 PRO".to_string()),
            size_gb: Some(1000.0),
            rotational: Some(false),
            device_type: Some("nvme".to_string()),
            mounts: vec![],
        });

        let advice = ArchAdvisor::check_nvme_scheduler(&hw);
        assert_eq!(advice.len(), 1);
        assert_eq!(advice[0].id, "nvme-scheduler");
    }

    #[test]
    fn test_cpu_governor_laptop() {
        let mut hw = mock_hardware();
        hw.battery.present = true;
        hw.battery.count = 1;

        let advice = ArchAdvisor::check_cpu_governor(&hw);
        assert_eq!(advice.len(), 1);
        assert_eq!(advice[0].id, "cpu-governor-laptop");
        assert_eq!(advice[0].level, Level::Info);
    }

    #[test]
    fn test_cpu_governor_desktop() {
        let hw = mock_hardware(); // No battery
        let advice = ArchAdvisor::check_cpu_governor(&hw);
        assert_eq!(advice.len(), 0); // No advice for desktops
    }

    #[test]
    fn test_tlp_missing_on_laptop() {
        let mut hw = mock_hardware();
        hw.battery.present = true;
        let pkg = mock_packages();

        let advice = ArchAdvisor::check_tlp_power_management(&hw, &pkg);
        assert_eq!(advice.len(), 1);
        assert_eq!(advice[0].id, "power-management-missing");
    }

    #[test]
    fn test_tlp_conflict_detection() {
        let mut hw = mock_hardware();
        hw.battery.present = true;
        let mut pkg = mock_packages();
        pkg.groups.base.push("tlp".to_string());
        pkg.groups.base.push("auto-cpufreq".to_string());

        let advice = ArchAdvisor::check_tlp_power_management(&hw, &pkg);
        assert_eq!(advice.len(), 1);
        assert_eq!(advice[0].id, "power-management-conflict");
        assert_eq!(advice[0].level, Level::Warn);
    }

    #[test]
    fn test_zram_recommended() {
        let mut hw = mock_hardware();
        hw.memory.total_gb = Some(6.0); // Low memory
        let pkg = mock_packages();

        let advice = ArchAdvisor::check_swap_config(&hw, &pkg);
        assert_eq!(advice.len(), 1);
        assert_eq!(advice[0].id, "zram-recommended");
        assert!(advice[0].title.contains("6.0GB"));
    }

    #[test]
    fn test_zram_not_recommended_high_memory() {
        let hw = mock_hardware(); // 32GB RAM
        let pkg = mock_packages();

        let advice = ArchAdvisor::check_swap_config(&hw, &pkg);
        assert_eq!(advice.len(), 0);
    }

    #[test]
    fn test_nvidia_wayland_modesetting() {
        let mut hw = mock_hardware();
        hw.gpus.push(GpuInfo {
            name: "NVIDIA RTX 3080".to_string(),
            vendor: Some("NVIDIA".to_string()),
            driver: Some("nvidia".to_string()),
            vram_mb: Some(10240),
            notes: vec![],
        });

        let mut pkg = mock_packages();
        pkg.groups.nvidia.push("nvidia".to_string());

        let advice = ArchAdvisor::check_wayland_acceleration(&hw, &pkg);
        assert_eq!(advice.len(), 1);
        assert_eq!(advice[0].id, "nvidia-wayland-modesetting");
    }

    #[test]
    fn test_aur_helper_missing() {
        let hw = mock_hardware();
        let mut pkg = mock_packages();
        pkg.aur_packages = vec!["some-aur-pkg".to_string(), "another-aur-pkg".to_string()];

        let advice = ArchAdvisor::check_aur_helpers(&pkg);
        assert_eq!(advice.len(), 1);
        assert_eq!(advice[0].id, "aur-helper-missing");
        assert!(advice[0].title.contains("2 AUR"));
    }

    #[test]
    fn test_aur_helper_present() {
        let hw = mock_hardware();
        let mut pkg = mock_packages();
        pkg.aur_packages = vec!["some-aur-pkg".to_string()];
        pkg.groups.base.push("yay".to_string());

        let advice = ArchAdvisor::check_aur_helpers(&pkg);
        assert_eq!(advice.len(), 0); // No advice when helper present
    }
}
