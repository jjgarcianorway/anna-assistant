//! Hardware recommendations

use super::{check_command_usage, is_package_installed};

use anna_common::{Advice, Alternative, Priority, RiskLevel, SystemFacts};
use std::path::Path;
use std::process::Command;

pub(crate) fn check_microcode(facts: &SystemFacts) -> Vec<Advice> {
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
                category: "Security & Privacy".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Microcode".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
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
                category: "Security & Privacy".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Microcode".to_string()],
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

pub(crate) fn check_gpu_drivers(facts: &SystemFacts) -> Vec<Advice> {
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
                category: "System Maintenance".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/NVIDIA".to_string()],
                                    depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
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
                category: "System Maintenance".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/AMDGPU".to_string()],
                                    depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
                }
            }
            _ => {}
        }
    }

    result
}

pub(crate) fn check_intel_gpu_support(facts: &SystemFacts) -> Vec<Advice> {
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
            category: "Hardware Support".to_string(),
            alternatives: Vec::new(),
            wiki_refs: vec!["https://wiki.archlinux.org/title/Intel_graphics".to_string()],
            depends_on: Vec::new(),
            related_to: Vec::new(),
            bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
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
            category: "Hardware Support".to_string(),
            alternatives: Vec::new(),
            wiki_refs: vec!["https://wiki.archlinux.org/title/Hardware_video_acceleration".to_string()],
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

pub(crate) fn check_amd_gpu_enhancements(facts: &SystemFacts) -> Vec<Advice> {
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
                command: Some("lspci -k | grep -A 3 -i vga".to_string()), // Show current GPU and driver
                risk: RiskLevel::Medium,
                priority: Priority::Optional,
                category: "Hardware Support".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec![
                    "https://wiki.archlinux.org/title/AMDGPU".to_string(),
                    "https://wiki.archlinux.org/title/ATI".to_string(),
                ],
                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
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
            category: "Hardware Support".to_string(),
            alternatives: Vec::new(),
            wiki_refs: vec!["https://wiki.archlinux.org/title/Hardware_video_acceleration".to_string()],
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

pub(crate) fn check_gpu_enhancements(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    // Recommend CUDA toolkit for NVIDIA GPUs with significant VRAM (for ML/compute workloads)
    if facts.is_nvidia && !facts.nvidia_cuda_support {
        // If we have VRAM info and it's >= 4GB, suggest CUDA (useful for ML/rendering)
        if let Some(vram) = facts.gpu_vram_mb {
            if vram >= 4096 {
                result.push(
                    Advice::new(
                        "install-cuda-toolkit".to_string(),
                        format!("Install CUDA toolkit for your NVIDIA GPU ({}MB VRAM)", vram),
                        format!("Your NVIDIA GPU has {}MB of VRAM, making it suitable for GPU-accelerated computing, machine learning (PyTorch, TensorFlow), video encoding (FFmpeg), and 3D rendering (Blender). The CUDA toolkit provides the libraries and compiler needed for GPU computing.", vram),
                        "Install CUDA toolkit and development tools".to_string(),
                        Some("sudo pacman -S --noconfirm cuda cuda-tools".to_string()),
                        RiskLevel::Low,
                        Priority::Optional,
                        vec![
                            "https://wiki.archlinux.org/title/GPGPU#CUDA".to_string(),
                            "https://wiki.archlinux.org/title/NVIDIA#CUDA".to_string(),
                        ],
                        "development".to_string(),
                    )
                    .with_popularity(45)
                );
            }
        }
    }

    // Recommend Vulkan tools if Vulkan is supported
    if facts.vulkan_support && !is_package_installed("vulkan-tools") {
        result.push(
            Advice::new(
                "install-vulkan-tools".to_string(),
                "Install Vulkan development and testing tools".to_string(),
                "Your system has Vulkan support installed. vulkan-tools provides utilities like vulkaninfo (check capabilities) and vkcube (test rendering), which are useful for troubleshooting graphics issues and verifying Vulkan functionality.".to_string(),
                "Install Vulkan diagnostic tools".to_string(),
                Some("sudo pacman -S --noconfirm vulkan-tools".to_string()),
                RiskLevel::Low,
                Priority::Optional,
                vec![
                    "https://wiki.archlinux.org/title/Vulkan#Verification".to_string(),
                ],
                "development".to_string(),
            )
            .with_popularity(40)
        );
    }

    // Suggest GPU monitoring tools if we have a dedicated GPU with VRAM
    if let Some(vram) = facts.gpu_vram_mb {
        if vram >= 2048 && !is_package_installed("nvtop") {
            let tool = if facts.is_nvidia { "nvtop" } else { "nvtop" };
            result.push(
                Advice::new(
                    "install-gpu-monitor".to_string(),
                    format!("Install {} to monitor GPU usage", tool),
                    format!("Your dedicated GPU ({}MB VRAM) would benefit from a monitoring tool. nvtop is like 'htop' but for GPUs - it shows real-time GPU usage, memory, temperature, and per-process GPU utilization. Essential for gaming, rendering, or ML workloads.", vram),
                    "Install GPU monitoring tool".to_string(),
                    Some(format!("sudo pacman -S --noconfirm {}", tool)),
                    RiskLevel::Low,
                    Priority::Optional,
                    vec![
                        "https://wiki.archlinux.org/title/NVIDIA#Monitoring".to_string(),
                    ],
                    "system".to_string(),
                )
                .with_popularity(55)
            );
        }
    }

    // Recommend OpenCL for compute workloads if not installed
    if (facts.is_nvidia || facts.is_amd_gpu || facts.is_intel_gpu)
        && !is_package_installed("opencl-headers")
    {
        let opencl_package = if facts.is_nvidia {
            "opencl-nvidia"
        } else if facts.is_amd_gpu {
            "opencl-mesa"
        } else {
            "intel-compute-runtime"
        };

        result.push(
            Advice::new(
                "install-opencl-support".to_string(),
                "Install OpenCL for GPU-accelerated computing".to_string(),
                format!("OpenCL enables GPU-accelerated computing for applications like video encoding (HandBrake), password cracking, scientific computing, and cryptocurrency mining. Your GPU supports it, but the runtime isn't installed. Package: {}", opencl_package),
                "Install OpenCL runtime and headers".to_string(),
                Some(format!("sudo pacman -S --noconfirm {} opencl-headers", opencl_package)),
                RiskLevel::Low,
                Priority::Optional,
                vec![
                    "https://wiki.archlinux.org/title/GPGPU#OpenCL".to_string(),
                ],
                "development".to_string(),
            )
            .with_popularity(50)
        );
    }

    result
}

pub(crate) fn check_disk_management() -> Vec<Advice> {
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
            category: "System Configuration".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/S.M.A.R.T.".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
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
            category: "System Configuration".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/GParted".to_string()],
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

pub(crate) fn check_cpu_temperature(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    if let Some(temp) = facts.hardware_monitoring.cpu_temperature_celsius {
        if temp > 85.0 {
            result.push(Advice {
                id: "cpu-temperature-critical".to_string(),
                title: "CPU Temperature is CRITICAL!".to_string(),
                reason: format!("Your CPU is running at {:.1}°C, which is dangerously high! Prolonged high temperatures can damage your hardware, reduce lifespan, and cause thermal throttling (slower performance). Normal temps: 40-60°C idle, 60-80°C load. You're in the danger zone!", temp),
                action: "Clean dust from fans, improve airflow, check thermal paste, verify cooling system".to_string(),
                command: Some("sensors 2>/dev/null | grep -E 'Core|temp' || echo 'sensors not available'".to_string()), // Show all temperature sensors
                risk: RiskLevel::Low,
                priority: Priority::Mandatory,
                category: "Hardware Support".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Fan_speed_control".to_string()],
                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        } else if temp > 75.0 {
            result.push(Advice {
                id: "cpu-temperature-high".to_string(),
                title: "CPU Temperature is High".to_string(),
                reason: format!("Your CPU is running at {:.1}°C, which is higher than ideal. This can cause thermal throttling and reduced performance. Consider cleaning dust from fans or improving case airflow.", temp),
                action: "Monitor temperature, consider cleaning cooling system".to_string(),
                command: Some("sensors 2>/dev/null | grep -E 'Core|fan' || echo 'sensors not available'".to_string()), // Show temperature and fan speeds
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Hardware Support".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Fan_speed_control".to_string()],
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

pub(crate) fn check_disk_health(facts: &SystemFacts) -> Vec<Advice> {
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
                command: Some(format!("smartctl -a {} 2>/dev/null | grep -E 'Reallocated|Pending|Temperature' || echo 'SMART data not available'", disk.device)), // Show detailed SMART data
                risk: RiskLevel::Low,
                priority: Priority::Mandatory,
                category: "Hardware Support".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/S.M.A.R.T.".to_string()],
                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
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
                category: "Hardware Support".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/S.M.A.R.T.".to_string()],
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

pub(crate) fn check_battery_health(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    if let Some(battery) = &facts.hardware_monitoring.battery_health {
        // Check for critical battery
        if battery.is_critical {
            result.push(Advice {
                id: "battery-critical".to_string(),
                title: "Battery critically low!".to_string(),
                reason: format!("Battery at {}%! Plug in your charger immediately to avoid data loss.", battery.percentage),
                action: "Plug in AC power immediately".to_string(),
                command: Some("upower -i $(upower -e | grep BAT) 2>/dev/null | grep -E 'state|percentage|time' || echo 'Battery info not available'".to_string()), // Show battery status
                risk: RiskLevel::Low,
                priority: Priority::Mandatory,
                category: "Power Management".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Laptop".to_string()],
                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
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
                    command: Some("upower -i $(upower -e | grep BAT) 2>/dev/null | grep -E 'capacity|energy' || echo 'Battery health info not available'".to_string()), // Show battery health details
                    risk: RiskLevel::Low,
                    priority: Priority::Optional,
                    category: "Power Management".to_string(),
                    alternatives: Vec::new(),
                    wiki_refs: vec!["https://wiki.archlinux.org/title/Laptop#Battery".to_string()],
                    depends_on: Vec::new(),
                    related_to: Vec::new(),
                    bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
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
                    category: "Power Management".to_string(),
                    alternatives: Vec::new(),
                    wiki_refs: vec!["https://wiki.archlinux.org/title/Laptop#Battery".to_string()],
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

pub(crate) fn check_disk_space_prediction(facts: &SystemFacts) -> Vec<Advice> {
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
                    category: "System Maintenance".to_string(),
                    alternatives: Vec::new(),
                    wiki_refs: vec!["https://wiki.archlinux.org/title/System_maintenance#Clean_the_filesystem".to_string()],
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

pub(crate) fn check_hyprland_nvidia_config(facts: &SystemFacts) -> Vec<Advice> {
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
            command: Some(
                "mkdir -p ~/.config/hypr && \
                echo '# Nvidia Wayland environment variables (added by Anna)' >> ~/.config/hypr/hyprland.conf && \
                echo 'env = GBM_BACKEND,nvidia-drm' >> ~/.config/hypr/hyprland.conf && \
                echo 'env = __GLX_VENDOR_LIBRARY_NAME,nvidia' >> ~/.config/hypr/hyprland.conf && \
                echo 'env = LIBVA_DRIVER_NAME,nvidia' >> ~/.config/hypr/hyprland.conf && \
                echo 'env = WLR_NO_HARDWARE_CURSORS,1' >> ~/.config/hypr/hyprland.conf && \
                echo 'Nvidia environment variables added to Hyprland config'"
                .to_string()
            ),
            priority: Priority::Mandatory,
            risk: RiskLevel::High,
            category: "Desktop Environment".to_string(),
            wiki_refs: vec!["https://wiki.hyprland.org/Nvidia/".to_string()],
            alternatives: vec![],
            depends_on: vec![],
            related_to: vec![],
            bundle: Some("hyprland-nvidia".to_string()),
            satisfies: Vec::new(),
            popularity: 60,
            requires: Vec::new(), // Common for Nvidia+Wayland users
        });
    }

    result
}

pub(crate) fn check_wayland_nvidia_config(facts: &SystemFacts) -> Vec<Advice> {
    let result = Vec::new();

    if facts.display_server.as_deref() != Some("wayland") || !facts.is_nvidia {
        return result;
    }

    if facts.window_manager.as_deref() == Some("Hyprland") {
        return result; // Already handled
    }

    result
}

pub(crate) fn check_microcode_updates(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    if !facts.microcode_status.microcode_installed && facts.microcode_status.vendor != "Unknown" {
        let package = if facts.microcode_status.vendor == "Intel" {
            "intel-ucode"
        } else {
            "amd-ucode"
        };

        result.push(Advice::new(
            format!("microcode-{}-install", facts.microcode_status.vendor.to_lowercase()),
            format!("Install {} CPU Microcode Updates", facts.microcode_status.vendor),
            format!(
                "Your {} CPU does not have microcode updates installed. Microcode updates fix critical CPU bugs and security vulnerabilities (Spectre, Meltdown, etc.) at the hardware level.",
                facts.microcode_status.vendor
            ),
            format!("Install {} package for essential CPU security updates", package),
            Some(format!("sudo pacman -S --noconfirm {}", package)),
            RiskLevel::Low,
            Priority::Mandatory,
            vec![
                "https://wiki.archlinux.org/title/Microcode".to_string(),
            ],
            "security".to_string(),
        ).with_popularity(95)
         .with_popularity(95));
    }

    result
}

pub(crate) fn check_battery_optimization(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    if let Some(ref battery) = facts.battery_info {
        if !battery.present {
            return result;
        }

        if let Some(health) = battery.health_percent {
            if health < 80.0 {
                result.push(Advice::new(
                    "battery-health-warning".to_string(),
                    "Battery Health Degraded".to_string(),
                    format!(
                        "Your battery health is at {:.1}% of its original capacity. Battery health below 80% means significantly shorter battery life.",
                        health
                    ),
                    "Consider battery replacement or use power management tools (TLP) to optimize remaining capacity".to_string(),
                    None,
                    RiskLevel::Low,
                    Priority::Cosmetic,
                    vec!["https://wiki.archlinux.org/title/Laptop".to_string()],
                    "power".to_string(),
                ).with_popularity(60));
            }
        }

        if !Command::new("which")
            .arg("tlp")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            result.push(Advice::new(
                "tlp-battery-management".to_string(),
                "Install TLP for Advanced Battery Management".to_string(),
                "TLP automatically adjusts power settings based on AC/battery power, significantly extending laptop battery life through CPU frequency scaling, disk power management, USB autosuspend, and more.".to_string(),
                "Install and enable TLP service for automatic power optimization".to_string(),
                Some("sudo pacman -S --noconfirm tlp && sudo systemctl enable --now tlp".to_string()),
                RiskLevel::Low,
                Priority::Recommended,
                vec!["https://wiki.archlinux.org/title/TLP".to_string()],
                "power".to_string(),
            ).with_bundle("Laptop Essentials".to_string())
             .with_popularity(85));
        }
    }

    result
}

pub(crate) fn check_disk_performance(_facts: &SystemFacts) -> Vec<Advice> {
    use std::process::Command;
    let mut advice = Vec::new();

    // 1. Check I/O wait time (high iowait = disk bottleneck)
    if let Ok(output) = Command::new("iostat").args(&["-x", "1", "2"]).output() {
        let output_str = String::from_utf8_lossy(&output.stdout);

        // Parse last iteration (2nd sample) for accurate reading
        let lines: Vec<&str> = output_str.lines().collect();
        let mut found_devices = false;

        for line in lines.iter().rev() {
            // Look for device lines (sda, nvme0n1, etc.)
            if line.contains("sd") || line.contains("nvme") || line.contains("vd") {
                let fields: Vec<&str> = line.split_whitespace().collect();

                // iostat output: Device r/s w/s rkB/s wkB/s ... %util
                if fields.len() >= 14 {
                    let device = fields[0];

                    // Check disk utilization (last field is %util)
                    if let Ok(util) = fields[fields.len() - 1].parse::<f32>() {
                        if util > 95.0 {
                            advice.push(Advice::new(
                                format!("disk-perf-high-util-{}", device),
                                format!("High disk utilization on {}", device),
                                format!(
                                    "Disk {} is at {:.1}% utilization. This indicates a disk I/O bottleneck. \
                                    Heavy disk usage can slow down your entire system. Consider: \
                                    1) Identifying and stopping I/O-heavy processes with 'iotop' \
                                    2) Moving frequently-accessed data to faster storage (SSD/NVMe) \
                                    3) Adding more RAM to reduce disk swapping \
                                    4) Checking for failing hardware with SMART diagnostics",
                                    device, util
                                ),
                                format!("iotop -o"),  // Show only processes doing I/O
                                None,
                                RiskLevel::Medium,
                                Priority::Recommended,
                                vec![
                                    "https://wiki.archlinux.org/title/Improving_performance#Storage_devices".to_string(),
                                ],
                                "performance".to_string(),
                            ));
                            found_devices = true;
                        } else if util > 80.0 {
                            advice.push(Advice::new(
                                format!("disk-perf-moderate-util-{}", device),
                                format!("Moderate disk utilization on {}", device),
                                format!(
                                    "Disk {} is at {:.1}% utilization. While not critical, sustained high disk usage \
                                    can impact performance. Monitor with 'iotop' to identify heavy I/O processes.",
                                    device, util
                                ),
                                format!("iotop -o"),
                                None,
                                RiskLevel::Low,
                                Priority::Optional,
                                vec![],
                                "performance".to_string(),
                            ));
                            found_devices = true;
                        }
                    }

                    // Check average wait time (await field)
                    if fields.len() >= 10 {
                        if let Ok(await_ms) = fields[9].parse::<f32>() {
                            if await_ms > 100.0 {
                                advice.push(Advice::new(
                                    format!("disk-perf-high-latency-{}", device),
                                    format!("High I/O latency on {}", device),
                                    format!(
                                        "Disk {} has average I/O latency of {:.1}ms. Normal latency should be <10ms. \
                                        High latency causes system slowdowns. Possible causes: \
                                        1) Failing disk hardware (check SMART status) \
                                        2) Disk fragmentation (less common on Linux) \
                                        3) Background processes doing heavy I/O \
                                        4) Slow disk (HDD vs SSD)",
                                        device, await_ms
                                    ),
                                    format!("sudo smartctl -a /dev/{}", device),
                                    None,
                                    RiskLevel::Medium,
                                    Priority::Recommended,
                                    vec![
                                        "https://wiki.archlinux.org/title/S.M.A.R.T.".to_string(),
                                    ],
                                    "performance".to_string(),
                                ));
                            }
                        }
                    }
                }
            }

            if found_devices {
                break; // We found and processed devices
            }
        }
    } else {
        // iostat not available
        advice.push(Advice::new(
            "disk-perf-iostat-missing".to_string(),
            "Install sysstat for disk performance monitoring".to_string(),
            "The sysstat package provides iostat, which monitors disk I/O performance. \
            Without it, Anna cannot detect disk bottlenecks that may be slowing your system."
                .to_string(),
            "pacman -S --noconfirm sysstat".to_string(),
            None,
            RiskLevel::Low,
            Priority::Optional,
            vec!["https://wiki.archlinux.org/title/Sysstat".to_string()],
            "Monitoring".to_string(),
        ));
    }

    // 2. Check SMART status for hardware issues
    let disks = vec!["sda", "sdb", "nvme0n1", "nvme1n1"];
    let mut checked_any_smart = false;

    for disk in &disks {
        if let Ok(output) = Command::new("smartctl")
            .args(&["-H", &format!("/dev/{}", disk)])
            .output()
        {
            if output.status.success() {
                checked_any_smart = true;
                let output_str = String::from_utf8_lossy(&output.stdout);

                if output_str.contains("PASSED") {
                    // Disk is healthy
                } else if output_str.contains("FAILED") {
                    advice.push(Advice::new(
                        format!("disk-perf-smart-failed-{}", disk),
                        format!("SMART test FAILED on {}", disk),
                        format!(
                            "CRITICAL: Disk /dev/{} is failing its SMART health check! \
                            This indicates imminent hardware failure. Backup your data IMMEDIATELY \
                            and replace this disk as soon as possible. Continued use may result in data loss.",
                            disk
                        ),
                        format!("sudo smartctl -a /dev/{}", disk),
                        None,
                        RiskLevel::High,
                        Priority::Mandatory,
                        vec![
                            "https://wiki.archlinux.org/title/S.M.A.R.T.".to_string(),
                        ],
                        "hardware".to_string(),
                    ));
                }
            }
        }
    }

    if !checked_any_smart {
        // smartctl not available or no disks found
        if Command::new("which").arg("smartctl").output().is_err()
            || !Command::new("which")
                .arg("smartctl")
                .output()
                .unwrap()
                .status
                .success()
        {
            advice.push(Advice::new(
                "disk-perf-smartmontools-missing".to_string(),
                "Install smartmontools for disk health monitoring".to_string(),
                "smartmontools provides SMART monitoring to detect failing disks before data loss occurs. \
                It's essential for proactive hardware failure detection.".to_string(),
                "pacman -S --noconfirm smartmontools && systemctl enable --now smartd".to_string(),
                None,
                RiskLevel::Low,
                Priority::Recommended,
                vec![
                    "https://wiki.archlinux.org/title/S.M.A.R.T.".to_string(),
                ],
                "Monitoring".to_string(),
            ));
        }
    }

    // 3. Check for filesystem errors in dmesg
    if let Ok(output) = Command::new("dmesg")
        .args(&["-l", "err,warn", "-T"])
        .output()
    {
        let output_str = String::from_utf8_lossy(&output.stdout);
        let fs_error_keywords = [
            "EXT4-fs error",
            "XFS",
            "Btrfs",
            "I/O error",
            "Buffer I/O error",
        ];

        for keyword in &fs_error_keywords {
            if output_str.to_lowercase().contains(&keyword.to_lowercase()) {
                advice.push(Advice::new(
                    "disk-perf-fs-errors".to_string(),
                    "Filesystem errors detected in kernel log".to_string(),
                    format!(
                        "The kernel log contains filesystem errors ({}). This may indicate: \
                        1) Disk hardware issues \
                        2) Filesystem corruption \
                        3) Driver problems \
                        Run 'dmesg | grep -i error' to see details, and consider running fsck on affected filesystems.",
                        keyword
                    ),
                    "dmesg | grep -i 'error\\|warn' | tail -20".to_string(),
                    None,
                    RiskLevel::High,
                    Priority::Recommended,
                    vec![
                        "https://wiki.archlinux.org/title/Fsck".to_string(),
                    ],
                    "system".to_string(),
                ));
                break; // Only report once
            }
        }
    }

    // 4. Check RAID status if mdadm is present
    if Command::new("which")
        .arg("mdadm")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        if let Ok(output) = Command::new("mdadm").args(&["--detail", "--scan"]).output() {
            if output.status.success() && !output.stdout.is_empty() {
                // RAID arrays exist, check their status
                if let Ok(detail_output) = Command::new("sh")
                    .args(&["-c", "cat /proc/mdstat"])
                    .output()
                {
                    let mdstat = String::from_utf8_lossy(&detail_output.stdout);

                    if mdstat.contains("_") {
                        // Underscore indicates failed disk
                        advice.push(Advice::new(
                            "disk-perf-raid-degraded".to_string(),
                            "RAID array is degraded".to_string(),
                            "One or more disks in your RAID array have failed! The array is running in degraded mode. \
                            Replace the failed disk(s) immediately and rebuild the array to restore redundancy.".to_string(),
                            "cat /proc/mdstat && mdadm --detail /dev/md0".to_string(),
                            None,
                            RiskLevel::High,
                            Priority::Mandatory,
                            vec![
                                "https://wiki.archlinux.org/title/RAID".to_string(),
                            ],
                            "hardware".to_string(),
                        ));
                    }

                    if mdstat.contains("recovery") || mdstat.contains("resync") {
                        advice.push(Advice::new(
                            "disk-perf-raid-rebuilding".to_string(),
                            "RAID array is rebuilding".to_string(),
                            "Your RAID array is currently rebuilding. This is normal after replacing a failed disk, \
                            but it will impact disk performance until complete. Monitor progress with 'cat /proc/mdstat'.".to_string(),
                            "cat /proc/mdstat".to_string(),
                            None,
                            RiskLevel::Low,
                            Priority::Cosmetic,
                            vec![],
                            "hardware".to_string(),
                        ));
                    }
                }
            }
        }
    }

    advice
}

pub(crate) fn check_ram_health(_facts: &SystemFacts) -> Vec<Advice> {
    use std::process::Command;
    let mut advice = Vec::new();

    // 1. Check overall memory usage
    if let Ok(output) = Command::new("free").args(&["-m"]).output() {
        let output_str = String::from_utf8_lossy(&output.stdout);

        // Parse free output: Mem: total used free shared buff/cache available
        for line in output_str.lines() {
            if line.starts_with("Mem:") {
                let fields: Vec<&str> = line.split_whitespace().collect();
                if fields.len() >= 7 {
                    if let (Ok(total), Ok(available)) =
                        (fields[1].parse::<f32>(), fields[6].parse::<f32>())
                    {
                        let used_percent = ((total - available) / total) * 100.0;

                        if used_percent > 95.0 {
                            advice.push(Advice::new(
                                "ram-pressure-critical".to_string(),
                                "Critical memory pressure detected".to_string(),
                                format!(
                                    "System RAM is {:.1}% full ({:.0}MB of {:.0}MB). \
                                    You're at risk of OOM (Out of Memory) kills. The kernel will forcibly terminate \
                                    processes to free memory. Actions to take: \
                                    1) Identify memory hogs with 'ps aux --sort=-rss | head -15' \
                                    2) Close unnecessary applications \
                                    3) Check for memory leaks (see other RAM advice) \
                                    4) Consider adding more RAM or swap space",
                                    used_percent, total - available, total
                                ),
                                "ps aux --sort=-rss | head -15".to_string(),
                                None,
                                RiskLevel::High,
                                Priority::Mandatory,
                                vec![
                                    "https://wiki.archlinux.org/title/Improving_performance#RAM_and_swap".to_string(),
                                ],
                                "performance".to_string(),
                            ));
                        } else if used_percent > 85.0 {
                            advice.push(Advice::new(
                                "ram-pressure-high".to_string(),
                                "High memory usage detected".to_string(),
                                format!(
                                    "System RAM is {:.1}% full ({:.0}MB of {:.0}MB). \
                                    Memory pressure is high. Consider: \
                                    1) Reviewing running applications \
                                    2) Checking for memory leaks \
                                    3) Increasing swap space if needed",
                                    used_percent,
                                    total - available,
                                    total
                                ),
                                "ps aux --sort=-rss | head -10".to_string(),
                                None,
                                RiskLevel::Medium,
                                Priority::Recommended,
                                vec![],
                                "performance".to_string(),
                            ));
                        }
                    }
                }
            }

            // Check swap usage
            if line.starts_with("Swap:") {
                let fields: Vec<&str> = line.split_whitespace().collect();
                if fields.len() >= 4 {
                    if let (Ok(total), Ok(used)) =
                        (fields[1].parse::<f32>(), fields[2].parse::<f32>())
                    {
                        if total > 0.0 {
                            let swap_percent = (used / total) * 100.0;

                            if swap_percent > 50.0 {
                                advice.push(Advice::new(
                                    "ram-swap-heavy".to_string(),
                                    "Heavy swap usage detected".to_string(),
                                    format!(
                                        "Swap is {:.1}% full ({:.0}MB of {:.0}MB). Heavy swapping severely degrades \
                                        performance as disk is much slower than RAM. This usually indicates: \
                                        1) Insufficient RAM for your workload \
                                        2) Memory leaks \
                                        3) Too many applications running \
                                        Check memory usage with 'free -h' and identify top consumers with 'ps aux --sort=-rss'",
                                        swap_percent, used, total
                                    ),
                                    "free -h && ps aux --sort=-rss | head -10".to_string(),
                                    None,
                                    RiskLevel::Medium,
                                    Priority::Recommended,
                                    vec![
                                        "https://wiki.archlinux.org/title/Swap".to_string(),
                                    ],
                                    "performance".to_string(),
                                ));
                            }
                        } else if used > 0.0 {
                            // Swap is being used but no swap configured (using zswap or zram)
                            advice.push(Advice::new(
                                "ram-no-swap-but-used".to_string(),
                                "No swap space configured".to_string(),
                                "System is using compressed memory (zram/zswap) but has no traditional swap. \
                                For systems with low RAM, consider adding swap space as a safety net to prevent OOM kills.".to_string(),
                                "fallocate -l 2G /swapfile && chmod 600 /swapfile && mkswap /swapfile".to_string(),
                                None,
                                RiskLevel::Low,
                                Priority::Optional,
                                vec![
                                    "https://wiki.archlinux.org/title/Swap#Swap_file".to_string(),
                                ],
                                "system".to_string(),
                            ));
                        }
                    }
                }
            }
        }
    }

    // 2. Detect potential memory leaks (processes with high RSS growth)
    // We'll check top memory consumers and flag those that seem excessive
    if let Ok(output) = Command::new("ps").args(&["aux", "--sort=-rss"]).output() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = output_str.lines().collect();

        // Skip header, check top 5 processes
        for (i, line) in lines.iter().skip(1).take(5).enumerate() {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() >= 11 {
                let user = fields[0];
                let pid = fields[1];
                let mem_percent = fields[3];
                let rss_kb = fields[5];
                let command = fields[10..].join(" ");

                // Parse RSS (in KB) and memory percent
                if let (Ok(rss), Ok(mem_pct)) = (rss_kb.parse::<f32>(), mem_percent.parse::<f32>())
                {
                    let rss_mb = rss / 1024.0;

                    // Flag processes using >2GB or >20% of RAM as potential leaks
                    if rss_mb > 2048.0 || mem_pct > 20.0 {
                        advice.push(Advice::new(
                            format!("ram-leak-candidate-{}", pid),
                            format!("High memory usage: {} (PID {})", command, pid),
                            format!(
                                "Process '{}' (PID {}) is using {:.1}MB ({:.1}% of RAM). \
                                This could indicate: \
                                1) Normal behavior for memory-intensive applications (browsers, IDEs, databases) \
                                2) A memory leak if usage grows over time \
                                3) Misconfigured application \
                                Monitor with: watch -n 5 'ps -p {} -o pid,rss,cmd' \
                                If RSS keeps growing, restart the application or investigate the leak.",
                                command, pid, rss_mb, mem_pct, pid
                            ),
                            format!("ps -p {} -o pid,user,%mem,rss,etime,cmd", pid),
                            None,
                            RiskLevel::Low,
                            Priority::Optional,
                            vec![],
                            "performance".to_string(),
                        ));

                        // Only report top 3 to avoid spam
                        if i >= 2 {
                            break;
                        }
                    }
                }
            }
        }
    }

    // 3. Check for OOM killer activity in dmesg
    if let Ok(output) = Command::new("dmesg")
        .args(&["-T", "--level=warn,err"])
        .output()
    {
        let output_str = String::from_utf8_lossy(&output.stdout);

        if output_str.to_lowercase().contains("out of memory")
            || output_str.to_lowercase().contains("oom")
        {
            advice.push(Advice::new(
                "ram-oom-killer-active".to_string(),
                "OOM killer has been triggered recently".to_string(),
                "The Out of Memory (OOM) killer has been forcibly terminating processes to free memory. \
                This indicates your system ran out of RAM and swap. Recent OOM events: \
                Check 'dmesg | grep -i oom' for details. \
                Solutions: \
                1) Add more RAM \
                2) Increase swap space \
                3) Identify and fix memory leaks \
                4) Reduce concurrent applications \
                The OOM killer's choices can be unpredictable - it may kill important processes!".to_string(),
                "dmesg | grep -i 'out of memory\\|oom' | tail -10".to_string(),
                None,
                RiskLevel::High,
                Priority::Mandatory,
                vec![
                    "https://wiki.archlinux.org/title/Improving_performance#Adjusting_OOM_score".to_string(),
                ],
                "system".to_string(),
            ));
        }
    }

    // 4. Check if earlyoom or systemd-oomd is installed for proactive OOM handling
    let has_earlyoom = Command::new("systemctl")
        .args(&["is-active", "earlyoom"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let has_oomd = Command::new("systemctl")
        .args(&["is-active", "systemd-oomd"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_earlyoom && !has_oomd {
        // Check if system has < 8GB RAM (more likely to hit OOM)
        if let Ok(output) = Command::new("free").args(&["-g"]).output() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.starts_with("Mem:") {
                    let fields: Vec<&str> = line.split_whitespace().collect();
                    if fields.len() >= 2 {
                        if let Ok(total_gb) = fields[1].parse::<i32>() {
                            if total_gb < 8 {
                                advice.push(Advice::new(
                                    "ram-no-oom-protection".to_string(),
                                    "No proactive OOM protection installed".to_string(),
                                    format!(
                                        "Your system has {}GB RAM and no earlyoom/systemd-oomd protection. \
                                        These tools prevent complete system freezes by terminating processes BEFORE \
                                        the system runs out of memory entirely. Without them, low memory situations \
                                        can make your system completely unresponsive. \
                                        Install earlyoom for better OOM handling.",
                                        total_gb
                                    ),
                                    "pacman -S --noconfirm earlyoom && systemctl enable --now earlyoom".to_string(),
                                    None,
                                    RiskLevel::Low,
                                    Priority::Recommended,
                                    vec![
                                        "https://wiki.archlinux.org/title/Earlyoom".to_string(),
                                    ],
                                    "system".to_string(),
                                ));
                            }
                        }
                    }
                    break;
                }
            }
        }
    }

    advice
}

pub(crate) fn check_cpu_health(_facts: &SystemFacts) -> Vec<Advice> {
    use std::process::Command;
    let mut advice = Vec::new();

    // 1. Check CPU load averages
    if let Ok(output) = Command::new("uptime").output() {
        let output_str = String::from_utf8_lossy(&output.stdout);

        // Parse load average: "... load average: 0.52, 0.58, 0.59"
        if let Some(load_part) = output_str.split("load average:").nth(1) {
            let loads: Vec<&str> = load_part.split(',').collect();
            if loads.len() >= 3 {
                // Get 1-minute, 5-minute, 15-minute averages
                if let Ok(load_1m) = loads[0].trim().parse::<f32>() {
                    // Get CPU core count
                    let cpu_count = std::thread::available_parallelism()
                        .map(|n| n.get())
                        .unwrap_or(1) as f32;

                    let load_per_cpu = load_1m / cpu_count;

                    if load_per_cpu > 2.0 {
                        advice.push(Advice::new(
                            "cpu-load-critical".to_string(),
                            "Critical CPU load detected".to_string(),
                            format!(
                                "CPU load average is {:.2} ({:.1} per core with {} cores). \
                                Load >2.0 per core means processes are waiting for CPU time. \
                                This severely impacts system responsiveness. Actions: \
                                1) Identify CPU hogs with 'top' or 'htop' \
                                2) Kill or nice heavy processes \
                                3) Check for runaway processes \
                                4) Consider if your workload needs more CPU cores",
                                load_1m, load_per_cpu, cpu_count as usize
                            ),
                            "top -b -n 1 | head -20".to_string(),
                            None,
                            RiskLevel::High,
                            Priority::Mandatory,
                            vec!["https://wiki.archlinux.org/title/Improving_performance#CPU"
                                .to_string()],
                            "performance".to_string(),
                        ));
                    } else if load_per_cpu > 1.0 {
                        advice.push(Advice::new(
                            "cpu-load-high".to_string(),
                            "High CPU load detected".to_string(),
                            format!(
                                "CPU load average is {:.2} ({:.1} per core with {} cores). \
                                Load >1.0 per core means your CPU is fully utilized. \
                                Performance may be impacted. Check running processes with 'top'.",
                                load_1m, load_per_cpu, cpu_count as usize
                            ),
                            "top -b -n 1 -o %CPU | head -15".to_string(),
                            None,
                            RiskLevel::Medium,
                            Priority::Recommended,
                            vec![],
                            "performance".to_string(),
                        ));
                    }
                }
            }
        }
    }

    // 2. Check for CPU frequency throttling
    if let Ok(output) = Command::new("sh")
        .args(&[
            "-c",
            "cat /sys/devices/system/cpu/cpu*/cpufreq/scaling_cur_freq 2>/dev/null | head -1",
        ])
        .output()
    {
        if !output.stdout.is_empty() {
            // CPU frequency scaling is available
            if let Ok(cur_freq_output) = Command::new("sh")
                .args(&[
                    "-c",
                    "cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq",
                ])
                .output()
            {
                if let Ok(max_freq_output) = Command::new("sh")
                    .args(&[
                        "-c",
                        "cat /sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq",
                    ])
                    .output()
                {
                    let cur_freq_str = String::from_utf8_lossy(&cur_freq_output.stdout);
                    let max_freq_str = String::from_utf8_lossy(&max_freq_output.stdout);

                    if let (Ok(cur_freq), Ok(max_freq)) = (
                        cur_freq_str.trim().parse::<f32>(),
                        max_freq_str.trim().parse::<f32>(),
                    ) {
                        let freq_percent = (cur_freq / max_freq) * 100.0;

                        if freq_percent < 50.0 {
                            advice.push(Advice::new(
                                "cpu-throttled".to_string(),
                                "CPU is severely throttled".to_string(),
                                format!(
                                    "CPU is running at {:.0}% of maximum frequency ({:.0} MHz vs {:.0} MHz max). \
                                    Severe throttling! This could be due to: \
                                    1) Thermal throttling (CPU too hot) \
                                    2) Power management (battery saver mode) \
                                    3) TDP/PL limits \
                                    Check: 1) CPU temperature with 'sensors' \
                                    2) CPU governor: cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor \
                                    3) Consider changing governor to 'performance' if plugged in",
                                    freq_percent, cur_freq / 1000.0, max_freq / 1000.0
                                ),
                                "cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor".to_string(),
                                None,
                                RiskLevel::Medium,
                                Priority::Recommended,
                                vec![
                                    "https://wiki.archlinux.org/title/CPU_frequency_scaling".to_string(),
                                ],
                                "performance".to_string(),
                            ));
                        } else if freq_percent < 75.0 {
                            advice.push(Advice::new(
                                "cpu-throttled-moderate".to_string(),
                                "CPU frequency reduced".to_string(),
                                format!(
                                    "CPU is running at {:.0}% of maximum frequency ({:.0} MHz vs {:.0} MHz max). \
                                    This is normal for power saving, but may impact performance. \
                                    Current governor: check with 'cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor'",
                                    freq_percent, cur_freq / 1000.0, max_freq / 1000.0
                                ),
                                "cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor".to_string(),
                                None,
                                RiskLevel::Low,
                                Priority::Optional,
                                vec![],
                                "performance".to_string(),
                            ));
                        }
                    }
                }
            }
        }
    }

    // 3. Check CPU temperature (if lm_sensors is available)
    if let Ok(output) = Command::new("sensors").output() {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);

            // Parse temperature lines (look for "Core" or "Tdie" or "Package")
            for line in output_str.lines() {
                if (line.contains("Core")
                    || line.contains("Tdie")
                    || line.contains("Package")
                    || line.contains("CPU"))
                    && line.contains("°C")
                {
                    // Extract temperature value
                    if let Some(temp_part) = line.split("°C").next() {
                        if let Some(temp_str) = temp_part.split('+').last() {
                            if let Ok(temp) = temp_str.trim().parse::<f32>() {
                                if temp > 90.0 {
                                    advice.push(Advice::new(
                                        "cpu-temp-critical".to_string(),
                                        "CPU temperature critically high".to_string(),
                                        format!(
                                            "CPU temperature is {:.0}°C! This is dangerously high and will cause: \
                                            1) Thermal throttling (reduced performance) \
                                            2) System instability/crashes \
                                            3) Potential hardware damage \
                                            Immediate actions: \
                                            1) Ensure good ventilation/cooling \
                                            2) Clean dust from fans/heatsink \
                                            3) Check if CPU cooler is properly mounted \
                                            4) Consider replacing thermal paste",
                                            temp
                                        ),
                                        "sensors".to_string(),
                                        None,
                                        RiskLevel::High,
                                        Priority::Mandatory,
                                        vec![
                                            "https://wiki.archlinux.org/title/Lm_sensors".to_string(),
                                        ],
                                        "hardware".to_string(),
                                    ));
                                    break;
                                } else if temp > 80.0 {
                                    advice.push(Advice::new(
                                        "cpu-temp-high".to_string(),
                                        "CPU temperature high".to_string(),
                                        format!(
                                            "CPU temperature is {:.0}°C. This is high and may cause throttling. \
                                            Check cooling system and ensure good airflow.",
                                            temp
                                        ),
                                        "sensors".to_string(),
                                        None,
                                        RiskLevel::Medium,
                                        Priority::Recommended,
                                        vec![],
                                        "hardware".to_string(),
                                    ));
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    } else {
        // lm_sensors not installed
        if !Command::new("which")
            .arg("sensors")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            advice.push(Advice::new(
                "cpu-no-temp-monitoring".to_string(),
                "Install lm-sensors for CPU temperature monitoring".to_string(),
                "lm-sensors provides temperature monitoring for your CPU and other hardware. \
                Without it, Anna cannot detect overheating issues that may cause throttling or damage.".to_string(),
                "pacman -S --noconfirm lm_sensors && sensors-detect --auto".to_string(),
                None,
                RiskLevel::Low,
                Priority::Optional,
                vec![
                    "https://wiki.archlinux.org/title/Lm_sensors".to_string(),
                ],
                "Monitoring".to_string(),
            ));
        }
    }

    // 4. Identify CPU-hogging processes
    if let Ok(output) = Command::new("ps").args(&["aux", "--sort=-%cpu"]).output() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = output_str.lines().collect();

        // Check top 3 CPU consumers
        for (i, line) in lines.iter().skip(1).take(3).enumerate() {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() >= 11 {
                let pid = fields[1];
                let cpu_percent = fields[2];
                let command = fields[10..].join(" ");

                if let Ok(cpu) = cpu_percent.parse::<f32>() {
                    // Flag processes using >80% of a single core consistently
                    if cpu > 80.0 {
                        advice.push(Advice::new(
                            format!("cpu-hog-{}", pid),
                            format!("High CPU usage: {} (PID {})", command, pid),
                            format!(
                                "Process '{}' (PID {}) is using {:.1}% CPU. \
                                High sustained CPU usage may indicate: \
                                1) Legitimate heavy computation \
                                2) Runaway process or infinite loop \
                                3) Misconfigured application \
                                Monitor with: watch -n 2 'ps -p {} -o pid,%cpu,etime,cmd' \
                                If CPU usage stays constant and high, investigate or restart the process.",
                                command, pid, cpu, pid
                            ),
                            format!("ps -p {} -o pid,user,%cpu,%mem,etime,cmd", pid),
                            None,
                            RiskLevel::Low,
                            Priority::Optional,
                            vec![],
                            "performance".to_string(),
                        ));

                        // Only report top 2 to avoid spam
                        if i >= 1 {
                            break;
                        }
                    }
                }
            }
        }
    }

    // 5. Check CPU governor setting
    if let Ok(output) = Command::new("cat")
        .arg("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor")
        .output()
    {
        let governor = String::from_utf8_lossy(&output.stdout).trim().to_string();

        if governor == "powersave" {
            // Check if system is on battery or AC
            let on_battery = std::path::Path::new("/sys/class/power_supply/AC/online").exists()
                && Command::new("cat")
                    .arg("/sys/class/power_supply/AC/online")
                    .output()
                    .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "0")
                    .unwrap_or(false);

            if !on_battery {
                advice.push(Advice::new(
                    "cpu-governor-powersave-ac".to_string(),
                    "CPU governor set to 'powersave' while on AC power".to_string(),
                    "Your CPU governor is set to 'powersave' which limits performance to save power. \
                    Since you're plugged into AC power, consider switching to 'performance' or 'schedutil' \
                    for better responsiveness. Note: This will increase power consumption and heat.".to_string(),
                    "echo 'To switch: echo performance | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor'".to_string(),
                    None,
                    RiskLevel::Low,
                    Priority::Optional,
                    vec![
                        "https://wiki.archlinux.org/title/CPU_frequency_scaling".to_string(),
                    ],
                    "performance".to_string(),
                ));
            }
        }
    }

    advice
}

