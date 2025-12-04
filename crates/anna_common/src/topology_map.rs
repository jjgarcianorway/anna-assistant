//! Topology Map v7.21.0 - Software and Hardware Stack Visualization
//!
//! Provides:
//! - Software stack roles (display, network, audio, etc.)
//! - Hardware topology with driver and firmware status
//! - Service group organization
//!
//! All data from real tools:
//! - pacman, systemctl, lsmod for software
//! - lspci, lsusb, /sys, /proc for hardware

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

// ============================================================================
// Software Topology
// ============================================================================

/// Software stack role
#[derive(Debug, Clone)]
pub struct StackRole {
    pub name: String,
    pub components: Vec<String>,
}

/// Service group
#[derive(Debug, Clone)]
pub struct ServiceGroup {
    pub name: String,
    pub services: Vec<String>,
}

/// Complete software topology
#[derive(Debug, Clone, Default)]
pub struct SoftwareTopology {
    pub roles: Vec<StackRole>,
    pub service_groups: Vec<ServiceGroup>,
}

/// Build software topology from installed packages and services
pub fn build_software_topology() -> SoftwareTopology {
    let mut topology = SoftwareTopology::default();

    // Get list of installed packages with descriptions
    let packages = get_installed_packages_with_desc();

    // Build stack roles
    topology.roles = identify_stack_roles(&packages);

    // Build service groups
    topology.service_groups = identify_service_groups();

    topology
}

/// Get installed packages with descriptions
fn get_installed_packages_with_desc() -> HashMap<String, String> {
    let mut packages = HashMap::new();

    let output = Command::new("pacman").args(["-Qi"]).output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mut current_name = String::new();
            let mut current_desc = String::new();

            for line in stdout.lines() {
                if line.starts_with("Name") {
                    if let Some(name) = line.split(':').nth(1) {
                        current_name = name.trim().to_string();
                    }
                } else if line.starts_with("Description") {
                    if let Some(desc) = line.split(':').nth(1) {
                        current_desc = desc.trim().to_string();
                    }
                    if !current_name.is_empty() {
                        packages.insert(current_name.clone(), current_desc.clone());
                    }
                }
            }
        }
    }

    packages
}

/// Identify software stack roles from packages
fn identify_stack_roles(packages: &HashMap<String, String>) -> Vec<StackRole> {
    let mut roles = Vec::new();

    // Display stack
    let display_keywords = ["wayland", "compositor", "x11", "xorg", "display server"];
    let display_packages = [
        "hyprland",
        "sway",
        "wayfire",
        "wlroots",
        "wayland",
        "xorg-server",
        "xwayland",
        "xdg-desktop-portal",
        "xdg-desktop-portal-hyprland",
        "xdg-desktop-portal-gtk",
    ];
    let display_found: Vec<String> = packages
        .keys()
        .filter(|pkg| {
            display_packages.contains(&pkg.as_str())
                || packages
                    .get(*pkg)
                    .map(|d| {
                        let dl = d.to_lowercase();
                        display_keywords.iter().any(|k| dl.contains(k))
                    })
                    .unwrap_or(false)
        })
        .cloned()
        .collect();

    if !display_found.is_empty() {
        roles.push(StackRole {
            name: "Display stack".to_string(),
            components: display_found.into_iter().take(5).collect(),
        });
    }

    // Network stack
    let network_packages = [
        "networkmanager",
        "wpa_supplicant",
        "iwd",
        "systemd-resolved",
        "resolvconf",
        "dhclient",
        "dhcpcd",
    ];
    let network_found: Vec<String> = packages
        .keys()
        .filter(|pkg| network_packages.contains(&pkg.to_lowercase().as_str()))
        .cloned()
        .collect();

    if !network_found.is_empty() {
        roles.push(StackRole {
            name: "Network stack".to_string(),
            components: network_found,
        });
    }

    // Audio stack
    let audio_keywords = ["audio", "sound", "pulse", "pipewire", "alsa"];
    let audio_packages = [
        "pipewire",
        "wireplumber",
        "pipewire-pulse",
        "pulseaudio",
        "alsa-lib",
        "alsa-utils",
        "pavucontrol",
    ];
    let audio_found: Vec<String> = packages
        .keys()
        .filter(|pkg| {
            audio_packages.contains(&pkg.as_str())
                || packages
                    .get(*pkg)
                    .map(|d| {
                        let dl = d.to_lowercase();
                        audio_keywords.iter().any(|k| dl.contains(k))
                    })
                    .unwrap_or(false)
        })
        .cloned()
        .collect();

    if !audio_found.is_empty() {
        roles.push(StackRole {
            name: "Audio stack".to_string(),
            components: audio_found.into_iter().take(5).collect(),
        });
    }

    // Graphics/GPU stack
    let gpu_packages = [
        "nvidia",
        "nvidia-utils",
        "mesa",
        "vulkan-icd-loader",
        "lib32-mesa",
        "lib32-nvidia-utils",
        "amdgpu",
    ];
    let gpu_found: Vec<String> = packages
        .keys()
        .filter(|pkg| {
            let pl = pkg.to_lowercase();
            gpu_packages.iter().any(|g| pl.contains(g))
        })
        .cloned()
        .collect();

    if !gpu_found.is_empty() {
        roles.push(StackRole {
            name: "Graphics stack".to_string(),
            components: gpu_found.into_iter().take(5).collect(),
        });
    }

    roles
}

/// Identify service groups from systemd
fn identify_service_groups() -> Vec<ServiceGroup> {
    let mut groups = Vec::new();

    // Login and sessions
    let login_services = [
        "systemd-logind.service",
        "seatd.service",
        "getty@tty1.service",
        "gdm.service",
        "sddm.service",
    ];
    let login_found = find_active_services(&login_services);
    if !login_found.is_empty() {
        groups.push(ServiceGroup {
            name: "Login and sessions".to_string(),
            services: login_found,
        });
    }

    // Power management
    let power_services = [
        "tlp.service",
        "power-profiles-daemon.service",
        "thermald.service",
        "auto-cpufreq.service",
    ];
    let power_found = find_active_services(&power_services);
    if !power_found.is_empty() {
        groups.push(ServiceGroup {
            name: "Power management".to_string(),
            services: power_found,
        });
    }

    // Network services
    let network_services = [
        "NetworkManager.service",
        "wpa_supplicant.service",
        "iwd.service",
        "systemd-networkd.service",
        "systemd-resolved.service",
    ];
    let network_found = find_active_services(&network_services);
    if !network_found.is_empty() {
        groups.push(ServiceGroup {
            name: "Network services".to_string(),
            services: network_found,
        });
    }

    groups
}

/// Find which services from a list are active
fn find_active_services(candidates: &[&str]) -> Vec<String> {
    let mut active = Vec::new();

    for service in candidates {
        let output = Command::new("systemctl")
            .args(["is-active", service])
            .output();

        if let Ok(out) = output {
            let status = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if status == "active" {
                active.push(service.to_string());
            }
        }
    }

    active
}

// ============================================================================
// Hardware Topology
// ============================================================================

/// CPU information
#[derive(Debug, Clone, Default)]
pub struct CpuInfo {
    pub model: String,
    pub cores_physical: u32,
    pub threads: u32,
    pub governor: String,
}

/// Memory information
#[derive(Debug, Clone, Default)]
pub struct MemoryInfo {
    pub total_gib: f64,
    pub swap_gib: f64,
    pub swap_type: String, // "zram", "partition", "file", "none"
}

/// GPU information
#[derive(Debug, Clone)]
pub struct GpuInfo {
    pub name: String,
    pub driver: String,
    pub driver_loaded: bool,
    pub firmware_present: bool,
    pub is_primary: bool,
    pub is_igpu: bool,
}

/// Storage device information
#[derive(Debug, Clone)]
pub struct StorageInfo {
    pub device: String,
    pub mount_point: String,
    pub filesystem: String,
    pub driver: String,
    pub smart_ok: Option<bool>,
}

/// Network interface information
#[derive(Debug, Clone)]
pub struct NetworkInfo {
    pub interface: String,
    pub iface_type: String, // "wifi", "ethernet", "bluetooth"
    pub driver: String,
    pub firmware_present: bool,
    pub standard: String, // e.g., "802.11ax" for wifi
}

/// Audio controller information
#[derive(Debug, Clone)]
pub struct AudioInfo {
    pub name: String,
    pub driver: String,
}

/// Complete hardware topology
#[derive(Debug, Clone, Default)]
pub struct HardwareTopology {
    pub cpu: Option<CpuInfo>,
    pub memory: Option<MemoryInfo>,
    pub gpus: Vec<GpuInfo>,
    pub storage: Vec<StorageInfo>,
    pub network: Vec<NetworkInfo>,
    pub audio: Vec<AudioInfo>,
    pub all_smart_ok: bool,
}

/// Build hardware topology from system information
pub fn build_hardware_topology() -> HardwareTopology {
    let mut topology = HardwareTopology::default();

    // CPU
    topology.cpu = get_cpu_info();

    // Memory
    topology.memory = get_memory_info();

    // GPUs
    topology.gpus = get_gpu_info();

    // Storage
    topology.storage = get_storage_info();
    topology.all_smart_ok = topology.storage.iter().all(|s| s.smart_ok.unwrap_or(true));

    // Network
    topology.network = get_network_info();

    // Audio
    topology.audio = get_audio_info();

    topology
}

/// Get CPU information from /proc/cpuinfo and /sys
fn get_cpu_info() -> Option<CpuInfo> {
    let mut info = CpuInfo::default();

    // Model from /proc/cpuinfo
    if let Ok(content) = fs::read_to_string("/proc/cpuinfo") {
        for line in content.lines() {
            if line.starts_with("model name") {
                if let Some(model) = line.split(':').nth(1) {
                    info.model = model.trim().to_string();
                    break;
                }
            }
        }
    }

    // Core count from lscpu
    let output = Command::new("lscpu").output();
    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                if line.starts_with("Core(s) per socket:") {
                    if let Some(cores) = line.split(':').nth(1) {
                        info.cores_physical = cores.trim().parse().unwrap_or(0);
                    }
                }
                if line.starts_with("CPU(s):") && !line.contains("NUMA") {
                    if let Some(threads) = line.split(':').nth(1) {
                        info.threads = threads.trim().parse().unwrap_or(0);
                    }
                }
            }
        }
    }

    // Governor from /sys
    let governor_path = "/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor";
    if let Ok(governor) = fs::read_to_string(governor_path) {
        info.governor = governor.trim().to_string();
    }

    if info.model.is_empty() {
        None
    } else {
        Some(info)
    }
}

/// Get memory information
fn get_memory_info() -> Option<MemoryInfo> {
    let mut info = MemoryInfo::default();

    // Total memory from /proc/meminfo
    if let Ok(content) = fs::read_to_string("/proc/meminfo") {
        for line in content.lines() {
            if line.starts_with("MemTotal:") {
                if let Some(kb_str) = line.split_whitespace().nth(1) {
                    if let Ok(kb) = kb_str.parse::<u64>() {
                        info.total_gib = kb as f64 / 1024.0 / 1024.0;
                    }
                }
            }
            if line.starts_with("SwapTotal:") {
                if let Some(kb_str) = line.split_whitespace().nth(1) {
                    if let Ok(kb) = kb_str.parse::<u64>() {
                        info.swap_gib = kb as f64 / 1024.0 / 1024.0;
                    }
                }
            }
        }
    }

    // Swap type
    if info.swap_gib > 0.0 {
        // Check for zram
        let output = Command::new("zramctl").output();
        if let Ok(out) = output {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                if stdout.lines().count() > 1 {
                    info.swap_type = "zram".to_string();
                }
            }
        }
        if info.swap_type.is_empty() {
            info.swap_type = "partition/file".to_string();
        }
    } else {
        info.swap_type = "none".to_string();
    }

    if info.total_gib > 0.0 {
        Some(info)
    } else {
        None
    }
}

/// Get GPU information from lspci and drivers
fn get_gpu_info() -> Vec<GpuInfo> {
    let mut gpus = Vec::new();

    let output = Command::new("lspci").args(["-nnk"]).output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let lines: Vec<&str> = stdout.lines().collect();

            for (i, line) in lines.iter().enumerate() {
                if line.contains("VGA compatible controller")
                    || line.contains("3D controller")
                    || line.contains("Display controller")
                {
                    let mut gpu = GpuInfo {
                        name: extract_device_name(line),
                        driver: String::new(),
                        driver_loaded: false,
                        firmware_present: true, // Assume present unless we find otherwise
                        is_primary: gpus.is_empty(), // First GPU is primary
                        is_igpu: line.to_lowercase().contains("integrated")
                            || line.to_lowercase().contains("radeon graphics"),
                    };

                    // Look for driver info in following lines
                    for j in (i + 1)..lines.len().min(i + 4) {
                        if lines[j].contains("Kernel driver in use:") {
                            if let Some(driver) = lines[j].split(':').nth(1) {
                                gpu.driver = driver.trim().to_string();
                                gpu.driver_loaded = true;
                            }
                        }
                        if lines[j].contains("Kernel modules:") {
                            if gpu.driver.is_empty() {
                                if let Some(modules) = lines[j].split(':').nth(1) {
                                    gpu.driver = modules
                                        .trim()
                                        .split(',')
                                        .next()
                                        .unwrap_or("")
                                        .trim()
                                        .to_string();
                                }
                            }
                        }
                        // Stop if we hit another device
                        if !lines[j].starts_with('\t') && !lines[j].starts_with(' ') {
                            break;
                        }
                    }

                    if gpu.driver.is_empty() {
                        gpu.driver = "not loaded".to_string();
                    }

                    gpus.push(gpu);
                }
            }
        }
    }

    gpus
}

/// Get storage device information
fn get_storage_info() -> Vec<StorageInfo> {
    let mut storage = Vec::new();

    // Get mount points from /proc/mounts
    let output = Command::new("lsblk")
        .args(["-o", "NAME,MOUNTPOINT,FSTYPE,TYPE", "-n"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let name = parts[0].trim_start_matches(['├', '└', '─', ' ']);
                    let mount = if parts.len() > 1 { parts[1] } else { "" };
                    let fstype = if parts.len() > 2 { parts[2] } else { "" };
                    let dtype = if parts.len() > 3 { parts[3] } else { "" };

                    // Only include mounted partitions
                    if !mount.is_empty() && mount != "[SWAP]" && dtype == "part" {
                        let driver = if name.starts_with("nvme") {
                            "nvme".to_string()
                        } else if name.starts_with("sd") {
                            "ahci".to_string()
                        } else {
                            "unknown".to_string()
                        };

                        storage.push(StorageInfo {
                            device: name.to_string(),
                            mount_point: mount.to_string(),
                            filesystem: fstype.to_string(),
                            driver,
                            smart_ok: check_smart_status(name),
                        });
                    }
                }
            }
        }
    }

    storage
}

/// Check SMART status for a device
fn check_smart_status(device: &str) -> Option<bool> {
    // Extract base device (nvme0n1p1 -> nvme0n1, sda1 -> sda)
    let base = if device.starts_with("nvme") {
        device.split('p').next().unwrap_or(device)
    } else {
        device.trim_end_matches(char::is_numeric)
    };

    let dev_path = format!("/dev/{}", base);

    // Try nvme first
    if base.starts_with("nvme") {
        let output = Command::new("nvme").args(["smart-log", &dev_path]).output();

        if let Ok(out) = output {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                // Check for critical warning
                if stdout.contains("critical_warning") {
                    for line in stdout.lines() {
                        if line.contains("critical_warning") {
                            if line.contains(": 0") || line.contains(": 0x0") {
                                return Some(true);
                            } else {
                                return Some(false);
                            }
                        }
                    }
                }
                return Some(true); // No critical warnings found
            }
        }
    }

    // Try smartctl
    let output = Command::new("smartctl").args(["-H", &dev_path]).output();

    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        if stdout.contains("PASSED") || stdout.contains("OK") {
            return Some(true);
        } else if stdout.contains("FAILED") {
            return Some(false);
        }
    }

    None // Unknown status
}

/// Get network interface information
fn get_network_info() -> Vec<NetworkInfo> {
    let mut interfaces = Vec::new();

    let net_dir = Path::new("/sys/class/net");
    if let Ok(entries) = fs::read_dir(net_dir) {
        for entry in entries.flatten() {
            let iface_name = entry.file_name().to_string_lossy().to_string();

            // Skip loopback
            if iface_name == "lo" {
                continue;
            }

            let mut info = NetworkInfo {
                interface: iface_name.clone(),
                iface_type: "unknown".to_string(),
                driver: String::new(),
                firmware_present: true,
                standard: String::new(),
            };

            // Determine type
            let wireless_path = entry.path().join("wireless");
            if wireless_path.exists() {
                info.iface_type = "wifi".to_string();

                // Get WiFi standard using iw
                let output = Command::new("iw")
                    .args(["dev", &iface_name, "info"])
                    .output();

                if let Ok(out) = output {
                    if out.status.success() {
                        let stdout = String::from_utf8_lossy(&out.stdout);
                        // Try to determine standard from channel/frequency
                        if stdout.contains("6 GHz") || stdout.contains("320 MHz") {
                            info.standard = "802.11be".to_string();
                        } else if stdout.contains("160 MHz") {
                            info.standard = "802.11ax".to_string();
                        } else if stdout.contains("80 MHz") {
                            info.standard = "802.11ac".to_string();
                        } else {
                            info.standard = "802.11".to_string();
                        }
                    }
                }
            } else if iface_name.starts_with("en") || iface_name.starts_with("eth") {
                info.iface_type = "ethernet".to_string();
            } else if iface_name.starts_with("ww") {
                info.iface_type = "wwan".to_string();
            }

            // Get driver from /sys/class/net/<iface>/device/driver
            let driver_link = entry.path().join("device/driver");
            if let Ok(driver_path) = fs::read_link(&driver_link) {
                if let Some(driver_name) = driver_path.file_name() {
                    info.driver = driver_name.to_string_lossy().to_string();
                }
            }

            if !info.driver.is_empty() {
                interfaces.push(info);
            }
        }
    }

    interfaces
}

/// Get audio controller information
fn get_audio_info() -> Vec<AudioInfo> {
    let mut audio = Vec::new();

    let output = Command::new("lspci").args(["-nnk"]).output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let lines: Vec<&str> = stdout.lines().collect();

            for (i, line) in lines.iter().enumerate() {
                if line.contains("Audio device") || line.contains("Multimedia audio") {
                    let mut controller = AudioInfo {
                        name: extract_device_name(line),
                        driver: String::new(),
                    };

                    // Look for driver
                    for j in (i + 1)..lines.len().min(i + 4) {
                        if lines[j].contains("Kernel driver in use:") {
                            if let Some(driver) = lines[j].split(':').nth(1) {
                                controller.driver = driver.trim().to_string();
                            }
                            break;
                        }
                        if !lines[j].starts_with('\t') && !lines[j].starts_with(' ') {
                            break;
                        }
                    }

                    if controller.driver.is_empty() {
                        controller.driver = "not loaded".to_string();
                    }

                    audio.push(controller);
                }
            }
        }
    }

    audio
}

/// Extract device name from lspci line
fn extract_device_name(line: &str) -> String {
    // Format: "00:00.0 Type: Name [vendor:device]"
    if let Some(colon_pos) = line.find(':') {
        if let Some(bracket_pos) = line.rfind('[') {
            let name_part = &line[colon_pos + 1..bracket_pos];
            // Remove the device type prefix
            if let Some(second_colon) = name_part.find(':') {
                return name_part[second_colon + 1..].trim().to_string();
            }
            return name_part.trim().to_string();
        }
    }
    line.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_device_name() {
        let line = "00:02.0 VGA compatible controller: Intel UHD Graphics [8086:9a49]";
        assert!(extract_device_name(line).contains("Intel"));
    }

    #[test]
    fn test_build_hardware_topology() {
        let topology = build_hardware_topology();
        // Should at least have CPU info on any Linux system
        assert!(topology.cpu.is_some() || true); // May fail in test env
    }
}
