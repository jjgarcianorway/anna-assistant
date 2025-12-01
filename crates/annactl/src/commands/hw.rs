//! HW Command v7.12.0 - Anna Hardware Overview
//!
//! Sections:
//! - [COMPONENTS]        CPU, GPU, WiFi, Bluetooth, Audio with drivers/firmware
//! - [HW TELEMETRY]      Real-time temp, utilization, frequencies with State summary (v7.12.0)
//! - [HEALTH HIGHLIGHTS] Real health data from sensors/SMART/logs
//! - [CATEGORIES]        Component identifiers per category
//! - [DEPENDENCIES]      Hardware tools status
//!
//! All data sourced from:
//! - lscpu, /proc/cpuinfo (CPU)
//! - free, /proc/meminfo (Memory)
//! - lspci -k (GPU, Audio controllers, PCI drivers)
//! - lsblk, smartctl (Storage)
//! - ip link, nmcli (Network)
//! - lsmod, modinfo (Module info)
//! - sensors, /sys/class/thermal (Temperatures)
//! - /sys/class/power_supply (Battery)
//! - /sys/class/hwmon (Hardware monitoring)
//! - journalctl -b -k / dmesg (Firmware messages)
//! - pacman -Qo (Driver packages)

use anyhow::Result;
use owo_colors::OwoColorize;
use std::process::Command;

use anna_common::grounded::health::{
    get_hardware_health_summary, get_cpu_health, get_gpu_health,
    get_all_disk_health, get_network_health, HealthStatus,
};
use anna_common::HardwareDeps;

const THIN_SEP: &str = "------------------------------------------------------------";

/// Run the hw overview command
pub async fn run() -> Result<()> {
    println!();
    println!("{}", "  Anna Hardware".bold());
    println!("{}", THIN_SEP);
    println!();

    // [COMPONENTS] - v7.10.0: replaces [OVERVIEW] and [DRIVERS]
    print_components_section();

    // [HW TELEMETRY] - v7.11.0: Real-time temp, utilization, frequencies
    print_hw_telemetry_section_v711();

    // [HEALTH HIGHLIGHTS]
    print_health_highlights_section();

    // [CATEGORIES]
    print_categories_section();

    // [DEPENDENCIES] - v7.6.0
    print_dependencies_section();

    println!("{}", THIN_SEP);
    println!();

    Ok(())
}

/// Print [COMPONENTS] section - v7.10.0
/// Format: Component: Model, driver: X, firmware/microcode: Y [present]
fn print_components_section() {
    println!("{}", "[COMPONENTS]".cyan());
    println!("  {}", "(source: lspci -k, lsmod, lscpu, ip link)".dimmed());
    println!();

    // CPU with driver and microcode
    print_cpu_component();

    // GPU(s) with driver and firmware
    print_gpu_components();

    // WiFi with driver and firmware
    print_wifi_components();

    // Bluetooth with driver
    print_bluetooth_components();

    // Audio with driver
    print_audio_components();

    println!();
}

/// Print CPU component line - v7.10.0
fn print_cpu_component() {
    let cpu_info = get_cpu_summary();
    let cpu_health = get_cpu_health();

    // Get CPU driver (usually intel_pstate or acpi-cpufreq)
    let driver = if cpu_health.drivers.is_empty() {
        "kernel default".to_string()
    } else {
        cpu_health.drivers.join(", ")
    };

    // Check for microcode package
    let microcode = get_cpu_microcode();

    print!("  CPU:        {}, driver: {}", cpu_info, driver);
    if let Some(mc) = microcode {
        println!(", microcode: {} {}", mc.0, mc.1);
    } else {
        println!();
    }
}

/// Get CPU microcode package if installed - v7.10.0
fn get_cpu_microcode() -> Option<(String, String)> {
    // Check for intel-ucode
    let intel = Command::new("pacman")
        .args(["-Qi", "intel-ucode"])
        .output();
    if let Ok(out) = intel {
        if out.status.success() {
            return Some(("intel-ucode".to_string(), "[present]".green().to_string()));
        }
    }

    // Check for amd-ucode
    let amd = Command::new("pacman")
        .args(["-Qi", "amd-ucode"])
        .output();
    if let Ok(out) = amd {
        if out.status.success() {
            return Some(("amd-ucode".to_string(), "[present]".green().to_string()));
        }
    }

    None
}

/// Print GPU component lines - v7.10.0
fn print_gpu_components() {
    // Get GPU info from lspci
    let output = Command::new("lspci")
        .args(["-k"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mut in_gpu = false;
            let mut gpu_name = String::new();
            let mut driver = String::new();

            for line in stdout.lines() {
                if line.contains("VGA") || line.contains("3D controller") || line.contains("Display controller") {
                    in_gpu = true;
                    // Extract name
                    if let Some(idx) = line.find(": ") {
                        gpu_name = line[idx + 2..].to_string();
                        // Shorten if needed
                        if gpu_name.len() > 50 {
                            gpu_name = format!("{}...", &gpu_name[..47]);
                        }
                    }
                } else if in_gpu && line.contains("Kernel driver in use:") {
                    if let Some(drv) = line.split(':').nth(1) {
                        driver = drv.trim().to_string();
                    }
                    // Get firmware status
                    let firmware = get_gpu_firmware_status(&driver);

                    print!("  GPU:        {}, driver: {}", gpu_name, driver);
                    if let Some((fw_name, status)) = firmware {
                        println!(", firmware: {} {}", fw_name, status);
                    } else {
                        println!();
                    }
                    in_gpu = false;
                    gpu_name.clear();
                    driver.clear();
                }
            }
        }
    }
}

/// Get GPU firmware status - v7.10.0
fn get_gpu_firmware_status(driver: &str) -> Option<(String, String)> {
    // Check firmware based on driver
    match driver {
        "nvidia" => {
            // nvidia-utils or nvidia-dkms
            let check = Command::new("pacman")
                .args(["-Qo", "/usr/lib/firmware/nvidia/"])
                .output();
            if let Ok(out) = check {
                if out.status.success() {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    if let Some(pkg) = stdout.split_whitespace().nth(4) {
                        return Some((pkg.to_string(), "[present]".green().to_string()));
                    }
                }
            }
            // Check if nvidia firmware exists
            if std::path::Path::new("/usr/lib/firmware/nvidia").exists() {
                return Some(("nvidia-firmware".to_string(), "[present]".green().to_string()));
            }
            None
        }
        "amdgpu" => {
            // Check linux-firmware
            let path = std::path::Path::new("/usr/lib/firmware/amdgpu");
            if path.exists() {
                return Some(("linux-firmware".to_string(), "[present]".green().to_string()));
            }
            Some(("linux-firmware".to_string(), "[missing]".yellow().to_string()))
        }
        "i915" => {
            let path = std::path::Path::new("/usr/lib/firmware/i915");
            if path.exists() {
                return Some(("linux-firmware".to_string(), "[present]".green().to_string()));
            }
            Some(("linux-firmware".to_string(), "[missing]".yellow().to_string()))
        }
        _ => None,
    }
}

/// Print WiFi component lines - v7.10.0
fn print_wifi_components() {
    let networks = get_network_health();
    for net in &networks {
        if net.interface_type != "wifi" {
            continue;
        }

        let iface = &net.interface;
        let driver = net.driver.as_deref().unwrap_or("unknown");

        // Get WiFi device model from lspci if available
        let model = get_wifi_model(iface).unwrap_or_else(|| "Wi-Fi adapter".to_string());

        // Get firmware status
        let firmware = get_wifi_firmware_status(driver);

        print!("  WiFi:       {}, driver: {}", model, driver);
        if let Some((fw_name, status)) = firmware {
            println!(", firmware: {} {}", fw_name, status);
        } else {
            println!();
        }
    }
}

/// Get WiFi device model - v7.10.0
fn get_wifi_model(iface: &str) -> Option<String> {
    // Try to get device path from /sys
    let device_path = format!("/sys/class/net/{}/device", iface);
    let device = std::path::Path::new(&device_path);
    if !device.exists() {
        return None;
    }

    // Get PCI address from symlink
    if let Ok(link) = std::fs::read_link(&device_path) {
        let link_str = link.to_string_lossy();
        // Extract PCI address (like 0000:00:14.3)
        if let Some(pci_part) = link_str.split('/').last() {
            // Look up in lspci
            let output = Command::new("lspci")
                .args(["-s", pci_part])
                .output();
            if let Ok(out) = output {
                if out.status.success() {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    if let Some(line) = stdout.lines().next() {
                        // Extract name after first colon
                        if let Some(idx) = line.find(": ") {
                            let name = line[idx + 2..].trim();
                            // Shorten if needed
                            if name.len() > 40 {
                                return Some(format!("{}...", &name[..37]));
                            }
                            return Some(name.to_string());
                        }
                    }
                }
            }
        }
    }

    None
}

/// Get WiFi firmware status - v7.10.0
fn get_wifi_firmware_status(driver: &str) -> Option<(String, String)> {
    // Common WiFi drivers and their firmware paths
    let fw_path = match driver {
        "iwlwifi" => "/usr/lib/firmware/iwlwifi-*.ucode",
        "ath10k_pci" | "ath10k_core" => "/usr/lib/firmware/ath10k",
        "ath11k_pci" | "ath11k_core" => "/usr/lib/firmware/ath11k",
        "brcmfmac" => "/usr/lib/firmware/brcm",
        "rtw88_pci" | "rtw89_pci" => "/usr/lib/firmware/rtw88",
        "mt7921e" | "mt7922e" => "/usr/lib/firmware/mediatek",
        _ => return None,
    };

    // Check if firmware exists
    let path = std::path::Path::new(fw_path.trim_end_matches("*"));
    if path.exists() || std::path::Path::new(&fw_path.replace("*", "")).exists() {
        return Some(("linux-firmware".to_string(), "[present]".green().to_string()));
    }

    Some(("linux-firmware".to_string(), "[missing]".yellow().to_string()))
}

/// Print Bluetooth component lines - v7.10.0
fn print_bluetooth_components() {
    let bt_path = std::path::Path::new("/sys/class/bluetooth");
    if !bt_path.exists() {
        return;
    }

    if let Ok(entries) = std::fs::read_dir(bt_path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();

            // Get driver
            let driver_path = entry.path().join("device/driver");
            let driver = if let Ok(link) = std::fs::read_link(&driver_path) {
                link.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "unknown".to_string())
            } else {
                "unknown".to_string()
            };

            // Get firmware status based on driver
            let firmware = get_bluetooth_firmware_status(&driver);

            print!("  Bluetooth:  {}, driver: {}", name, driver);
            if let Some((fw_name, status)) = firmware {
                println!(", firmware: {} {}", fw_name, status);
            } else {
                println!();
            }
        }
    }
}

/// Get Bluetooth firmware status - v7.10.0
fn get_bluetooth_firmware_status(driver: &str) -> Option<(String, String)> {
    match driver {
        "btusb" | "btintel" => {
            // Intel Bluetooth uses firmware from linux-firmware
            let path = std::path::Path::new("/usr/lib/firmware/intel");
            if path.exists() {
                return Some(("linux-firmware".to_string(), "[present]".green().to_string()));
            }
            Some(("linux-firmware".to_string(), "[missing]".yellow().to_string()))
        }
        "btqca" => {
            let path = std::path::Path::new("/usr/lib/firmware/qca");
            if path.exists() {
                return Some(("linux-firmware".to_string(), "[present]".green().to_string()));
            }
            Some(("linux-firmware".to_string(), "[missing]".yellow().to_string()))
        }
        _ => None,
    }
}

/// Print Audio component lines - v7.10.0
fn print_audio_components() {
    let output = Command::new("lspci")
        .args(["-k"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mut in_audio = false;
            let mut audio_name = String::new();
            let mut driver = String::new();

            for line in stdout.lines() {
                if line.contains("Audio device") || line.contains("Multimedia audio") {
                    in_audio = true;
                    // Extract name
                    if let Some(idx) = line.find(": ") {
                        audio_name = line[idx + 2..].to_string();
                        // Shorten if needed
                        if audio_name.len() > 40 {
                            audio_name = format!("{}...", &audio_name[..37]);
                        }
                    }
                } else if in_audio && line.contains("Kernel driver in use:") {
                    if let Some(drv) = line.split(':').nth(1) {
                        driver = drv.trim().to_string();
                    }
                    // Print the audio component
                    println!("  Audio:      {}, driver: {}", audio_name, driver);
                    in_audio = false;
                    audio_name.clear();
                    driver.clear();
                }
            }
        }
    }
}

#[allow(dead_code)]
fn print_overview_section() {
    println!("{}", "[OVERVIEW]".cyan());

    // CPU
    let cpu_info = get_cpu_summary();
    println!("  CPU:        {}", cpu_info);

    // Memory
    let mem_info = get_memory_summary();
    println!("  Memory:     {}", mem_info);

    // GPU
    let gpu_info = get_gpu_summary();
    println!("  GPU:        {}", gpu_info);

    // Storage
    let storage_info = get_storage_summary();
    println!("  Storage:    {}", storage_info);

    // Network
    let network_info = get_network_summary();
    println!("  Network:    {}", network_info);

    // Audio
    let audio_info = get_audio_summary();
    println!("  Audio:      {}", audio_info);

    // Power
    let power_info = get_power_summary();
    println!("  Power:      {}", power_info);

    println!();
}

#[allow(dead_code)]
fn print_drivers_section() {
    println!("{}", "[DRIVERS]".cyan());

    // CPU driver
    let cpu = get_cpu_health();
    if cpu.drivers.is_empty() {
        println!("  CPU:        {}", "kernel default".dimmed());
    } else {
        println!("  CPU:        {}", cpu.drivers.join(", "));
    }

    // GPU drivers
    let gpus = get_gpu_health();
    if gpus.is_empty() {
        println!("  GPU:        {}", "(none)".dimmed());
    } else {
        let gpu_drivers: Vec<String> = gpus
            .iter()
            .filter_map(|g| g.driver.clone())
            .collect();
        if gpu_drivers.is_empty() {
            println!("  GPU:        {}", "no driver bound".yellow());
        } else {
            println!("  GPU:        {}", gpu_drivers.join(", "));
        }
    }

    // Disk drivers
    let disks = get_all_disk_health();
    let disk_drivers: Vec<String> = disks
        .iter()
        .map(|d| match d.device_type.as_str() {
            "NVMe" => "nvme",
            "SATA" => "sd_mod",
            "USB" => "usb-storage",
            _ => "block",
        })
        .map(String::from)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    if disk_drivers.is_empty() {
        println!("  Disks:      {}", "(none)".dimmed());
    } else {
        println!("  Disks:      {}", disk_drivers.join(", "));
    }

    // Network drivers
    let networks = get_network_health();
    let eth_drivers: Vec<String> = networks
        .iter()
        .filter(|n| n.interface_type == "ethernet")
        .filter_map(|n| n.driver.clone())
        .collect();
    let wifi_drivers: Vec<String> = networks
        .iter()
        .filter(|n| n.interface_type == "wifi")
        .filter_map(|n| n.driver.clone())
        .collect();

    println!("  Network:");
    if !eth_drivers.is_empty() {
        println!("    Ethernet: driver: {}", eth_drivers.join(", "));
    }
    if !wifi_drivers.is_empty() {
        println!("    Wi-Fi:    driver: {}", wifi_drivers.join(", "));
    }
    if eth_drivers.is_empty() && wifi_drivers.is_empty() {
        println!("    {}", "(no physical interfaces)".dimmed());
    }

    println!();
}

fn print_health_highlights_section() {
    println!("{}", "[HEALTH HIGHLIGHTS]".cyan());

    let summary = get_hardware_health_summary();

    // CPU
    if let Some((status, desc)) = summary.get("CPU") {
        let status_str = format_health_status(*status);
        println!("  CPU:        {} ({})", status_str, desc);
    }

    // GPU
    if let Some((status, desc)) = summary.get("GPU") {
        let status_str = format_health_status(*status);
        println!("  GPU:        {} ({})", status_str, desc);
    }

    // Disks
    if let Some((status, desc)) = summary.get("Disks") {
        let status_str = format_health_status(*status);
        println!("  Disks:      {} ({})", status_str, desc);
    }

    // Battery
    if let Some((status, desc)) = summary.get("Battery") {
        if desc != "not present" {
            let status_str = format_health_status(*status);
            println!("  Battery:    {} ({})", status_str, desc);
        } else {
            println!("  Battery:    {}", "not present".dimmed());
        }
    }

    // Network
    if let Some((status, desc)) = summary.get("Network") {
        let status_str = format_health_status(*status);
        println!("  Network:    {} ({})", status_str, desc);
    }

    println!();
}

fn format_health_status(status: HealthStatus) -> String {
    match status {
        HealthStatus::Ok => "normal".green().to_string(),
        HealthStatus::Warning => "warning".yellow().to_string(),
        HealthStatus::Critical => "critical".red().to_string(),
        HealthStatus::Unknown => "unknown".dimmed().to_string(),
    }
}

/// Print [HW TELEMETRY] section - v7.11.0
/// Real-time temperature, utilization, and frequencies from hwmon/thermal/proc
fn print_hw_telemetry_section_v711() {
    println!("{}", "[HW TELEMETRY]".cyan());
    println!("  {}", "(source: /sys/class/hwmon, /sys/class/thermal, /proc, sensors)".dimmed());
    println!();

    // v7.12.0: State summary line
    let state = derive_hw_telemetry_state();
    println!("  State (now):    {}", state);
    println!();

    // CPU telemetry
    print_cpu_telemetry();

    // GPU telemetry
    print_gpu_telemetry();

    // Memory telemetry
    print_memory_telemetry();

    // Disk I/O telemetry
    print_disk_io_telemetry();

    println!();
}

/// v7.12.0: Derive hardware telemetry state summary
/// Temp thresholds: <70C normal, 70-85C elevated, >85C hot
fn derive_hw_telemetry_state() -> String {
    let mut states = Vec::new();

    // Check CPU temp
    if let Some(temp) = get_cpu_temperature() {
        let thermal_state = if temp < 70 {
            "normal thermals"
        } else if temp <= 85 {
            "elevated thermals"
        } else {
            "hot thermals"
        };
        states.push(thermal_state);
    }

    // Check CPU utilization
    if let Some(util) = get_cpu_utilization() {
        let util_state = if util < 30.0 {
            "normal utilization"
        } else if util <= 70.0 {
            "moderate utilization"
        } else {
            "high utilization"
        };
        states.push(util_state);
    }

    if states.is_empty() {
        "unknown".to_string()
    } else {
        states.join(", ")
    }
}

/// Print CPU telemetry (temp, frequency, utilization)
fn print_cpu_telemetry() {
    let mut parts = Vec::new();

    // Get CPU temperature from hwmon or thermal_zone
    if let Some(temp) = get_cpu_temperature() {
        parts.push(format!("{}°C", temp));
    }

    // Get CPU frequency from /sys/devices/system/cpu
    if let Some(freq) = get_cpu_frequency() {
        parts.push(format!("{} MHz", freq));
    }

    // Get CPU utilization from /proc/stat
    if let Some(util) = get_cpu_utilization() {
        parts.push(format!("{:.1}% util", util));
    }

    if parts.is_empty() {
        println!("  CPU:        {}", "(no telemetry available)".dimmed());
    } else {
        println!("  CPU:        {}", parts.join(", "));
    }
}

/// Get CPU temperature from hwmon or thermal_zone
fn get_cpu_temperature() -> Option<i32> {
    // Try hwmon first (more accurate)
    let hwmon_path = std::path::Path::new("/sys/class/hwmon");
    if hwmon_path.exists() {
        if let Ok(entries) = std::fs::read_dir(hwmon_path) {
            for entry in entries.flatten() {
                let name_path = entry.path().join("name");
                if let Ok(name) = std::fs::read_to_string(&name_path) {
                    let name = name.trim();
                    if name == "coretemp" || name == "k10temp" || name == "zenpower" {
                        // Read temp1_input (CPU package temp)
                        let temp_path = entry.path().join("temp1_input");
                        if let Ok(temp_str) = std::fs::read_to_string(&temp_path) {
                            if let Ok(temp_millidegrees) = temp_str.trim().parse::<i32>() {
                                return Some(temp_millidegrees / 1000);
                            }
                        }
                    }
                }
            }
        }
    }

    // Fallback to thermal_zone
    let thermal_path = std::path::Path::new("/sys/class/thermal");
    if thermal_path.exists() {
        if let Ok(entries) = std::fs::read_dir(thermal_path) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("thermal_zone") {
                    let type_path = entry.path().join("type");
                    if let Ok(zone_type) = std::fs::read_to_string(&type_path) {
                        let zone_type = zone_type.trim().to_lowercase();
                        if zone_type.contains("cpu") || zone_type.contains("x86_pkg") || zone_type == "acpitz" {
                            let temp_path = entry.path().join("temp");
                            if let Ok(temp_str) = std::fs::read_to_string(&temp_path) {
                                if let Ok(temp_millidegrees) = temp_str.trim().parse::<i32>() {
                                    return Some(temp_millidegrees / 1000);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

/// Get current CPU frequency (average across cores)
fn get_cpu_frequency() -> Option<u32> {
    let cpu_path = std::path::Path::new("/sys/devices/system/cpu");
    if !cpu_path.exists() {
        return None;
    }

    let mut total_freq: u64 = 0;
    let mut count = 0;

    if let Ok(entries) = std::fs::read_dir(cpu_path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("cpu") && name[3..].chars().all(|c| c.is_ascii_digit()) {
                let freq_path = entry.path().join("cpufreq/scaling_cur_freq");
                if let Ok(freq_str) = std::fs::read_to_string(&freq_path) {
                    if let Ok(freq_khz) = freq_str.trim().parse::<u64>() {
                        total_freq += freq_khz;
                        count += 1;
                    }
                }
            }
        }
    }

    if count > 0 {
        Some((total_freq / count / 1000) as u32) // Convert kHz to MHz
    } else {
        None
    }
}

/// Get CPU utilization from /proc/stat (quick snapshot)
fn get_cpu_utilization() -> Option<f32> {
    // Read /proc/stat twice with a small delay
    let stat1 = std::fs::read_to_string("/proc/stat").ok()?;
    std::thread::sleep(std::time::Duration::from_millis(100));
    let stat2 = std::fs::read_to_string("/proc/stat").ok()?;

    let parse_cpu_line = |line: &str| -> Option<(u64, u64)> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 8 || !parts[0].starts_with("cpu") || parts[0] != "cpu" {
            return None;
        }
        // user, nice, system, idle, iowait, irq, softirq
        let user: u64 = parts[1].parse().ok()?;
        let nice: u64 = parts[2].parse().ok()?;
        let system: u64 = parts[3].parse().ok()?;
        let idle: u64 = parts[4].parse().ok()?;
        let iowait: u64 = parts[5].parse().ok()?;
        let irq: u64 = parts[6].parse().ok()?;
        let softirq: u64 = parts[7].parse().ok()?;

        let active = user + nice + system + irq + softirq;
        let total = active + idle + iowait;
        Some((active, total))
    };

    let line1 = stat1.lines().find(|l| l.starts_with("cpu "))?;
    let line2 = stat2.lines().find(|l| l.starts_with("cpu "))?;

    let (active1, total1) = parse_cpu_line(line1)?;
    let (active2, total2) = parse_cpu_line(line2)?;

    let active_diff = active2.saturating_sub(active1);
    let total_diff = total2.saturating_sub(total1);

    if total_diff > 0 {
        Some((active_diff as f32 / total_diff as f32) * 100.0)
    } else {
        None
    }
}

/// Print GPU telemetry (temp, utilization, VRAM)
fn print_gpu_telemetry() {
    let mut parts = Vec::new();

    // Try nvidia-smi for NVIDIA GPUs
    let nvidia_output = Command::new("nvidia-smi")
        .args(["--query-gpu=temperature.gpu,utilization.gpu,memory.used,memory.total", "--format=csv,noheader,nounits"])
        .output();

    if let Ok(out) = nvidia_output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if let Some(line) = stdout.lines().next() {
                let fields: Vec<&str> = line.split(", ").collect();
                if fields.len() >= 4 {
                    if let Ok(temp) = fields[0].trim().parse::<i32>() {
                        parts.push(format!("{}°C", temp));
                    }
                    if let Ok(util) = fields[1].trim().parse::<i32>() {
                        parts.push(format!("{}% util", util));
                    }
                    if let (Ok(used), Ok(total)) = (
                        fields[2].trim().parse::<u64>(),
                        fields[3].trim().parse::<u64>()
                    ) {
                        parts.push(format!("{}/{} MiB VRAM", used, total));
                    }
                }
            }
        }
    }

    // Fallback: Try AMD GPU via hwmon
    if parts.is_empty() {
        if let Some(temp) = get_amd_gpu_temperature() {
            parts.push(format!("{}°C", temp));
        }
    }

    // Fallback: Try Intel GPU (no direct telemetry, skip)

    if parts.is_empty() {
        println!("  GPU:        {}", "(no telemetry available - need nvidia-smi or AMD hwmon)".dimmed());
    } else {
        println!("  GPU:        {}", parts.join(", "));
    }
}

/// Get AMD GPU temperature from hwmon
fn get_amd_gpu_temperature() -> Option<i32> {
    let hwmon_path = std::path::Path::new("/sys/class/hwmon");
    if !hwmon_path.exists() {
        return None;
    }

    if let Ok(entries) = std::fs::read_dir(hwmon_path) {
        for entry in entries.flatten() {
            let name_path = entry.path().join("name");
            if let Ok(name) = std::fs::read_to_string(&name_path) {
                let name = name.trim();
                if name == "amdgpu" {
                    // Read temp1_input (edge temp)
                    let temp_path = entry.path().join("temp1_input");
                    if let Ok(temp_str) = std::fs::read_to_string(&temp_path) {
                        if let Ok(temp_millidegrees) = temp_str.trim().parse::<i32>() {
                            return Some(temp_millidegrees / 1000);
                        }
                    }
                }
            }
        }
    }

    None
}

/// Print memory telemetry (usage)
fn print_memory_telemetry() {
    if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
        let mut total: u64 = 0;
        let mut available: u64 = 0;

        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                if let Some(val) = line.split_whitespace().nth(1) {
                    total = val.parse().unwrap_or(0);
                }
            } else if line.starts_with("MemAvailable:") {
                if let Some(val) = line.split_whitespace().nth(1) {
                    available = val.parse().unwrap_or(0);
                }
            }
        }

        if total > 0 {
            let used = total.saturating_sub(available);
            let used_gib = used as f64 / (1024.0 * 1024.0);
            let total_gib = total as f64 / (1024.0 * 1024.0);
            let percent = (used as f64 / total as f64) * 100.0;
            println!("  Memory:     {:.1}/{:.1} GiB ({:.0}% used)", used_gib, total_gib, percent);
        } else {
            println!("  Memory:     {}", "(unavailable)".dimmed());
        }
    }
}

/// Print disk I/O telemetry (read/write rates)
fn print_disk_io_telemetry() {
    // Get disk stats from /proc/diskstats
    let diskstats1 = std::fs::read_to_string("/proc/diskstats").ok();
    std::thread::sleep(std::time::Duration::from_millis(100));
    let diskstats2 = std::fs::read_to_string("/proc/diskstats").ok();

    let (Some(stats1), Some(stats2)) = (diskstats1, diskstats2) else {
        println!("  Disk I/O:   {}", "(unavailable)".dimmed());
        return;
    };

    // Parse diskstats - only look at main block devices (nvme0n1, sda, etc.)
    let parse_stats = |content: &str| -> std::collections::HashMap<String, (u64, u64)> {
        let mut result = std::collections::HashMap::new();
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 10 {
                let name = parts[2];
                // Only main devices, not partitions
                if (name.starts_with("nvme") && name.ends_with("n1"))
                    || (name.starts_with("sd") && name.len() == 3)
                {
                    // Field 6 = sectors read, Field 10 = sectors written
                    let sectors_read: u64 = parts[5].parse().unwrap_or(0);
                    let sectors_written: u64 = parts[9].parse().unwrap_or(0);
                    result.insert(name.to_string(), (sectors_read, sectors_written));
                }
            }
        }
        result
    };

    let stats1_map = parse_stats(&stats1);
    let stats2_map = parse_stats(&stats2);

    let mut total_read_sectors: u64 = 0;
    let mut total_write_sectors: u64 = 0;

    for (name, (read1, write1)) in &stats1_map {
        if let Some((read2, write2)) = stats2_map.get(name) {
            total_read_sectors += read2.saturating_sub(*read1);
            total_write_sectors += write2.saturating_sub(*write1);
        }
    }

    // Convert to MB/s (512 bytes per sector, 100ms sample)
    let read_mbs = (total_read_sectors * 512) as f64 / (1024.0 * 1024.0) * 10.0;
    let write_mbs = (total_write_sectors * 512) as f64 / (1024.0 * 1024.0) * 10.0;

    println!("  Disk I/O:   {:.1} MB/s read, {:.1} MB/s write", read_mbs, write_mbs);
}

fn print_categories_section() {
    println!("{}", "[CATEGORIES]".cyan());

    // CPU
    println!("  CPU:        cpu0");

    // Memory
    println!("  Memory:     mem0");

    // GPU
    let gpus = list_gpus();
    if gpus.is_empty() {
        println!("  GPU:        (none detected)");
    } else {
        println!("  GPU:        {}", gpus.join(", "));
    }

    // Storage
    let storage = list_storage_devices();
    if storage.is_empty() {
        println!("  Storage:    (none detected)");
    } else {
        println!("  Storage:    {}", storage.join(", "));
    }

    // Network
    let network = list_network_interfaces();
    if network.is_empty() {
        println!("  Network:    (none detected)");
    } else {
        println!("  Network:    {}", network.join(", "));
    }

    // Audio
    let audio = list_audio_devices();
    if audio.is_empty() {
        println!("  Audio:      (none detected)");
    } else {
        println!("  Audio:      {}", audio.join(", "));
    }

    // Power supplies (batteries)
    let power = list_power_supplies();
    if power.is_empty() {
        println!("  Power:      (no batteries)");
    } else {
        println!("  Power:      {}", power.join(", "));
    }

    println!();
}

// ============================================================================
// CPU Info
// ============================================================================

fn get_cpu_summary() -> String {
    // Try lscpu first
    let output = Command::new("lscpu").output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);

            let mut model = String::new();
            let mut cores = String::new();
            let mut threads = String::new();

            for line in stdout.lines() {
                if line.starts_with("Model name:") {
                    model = line
                        .split(':')
                        .nth(1)
                        .map(|s| s.trim().to_string())
                        .unwrap_or_default();
                } else if line.starts_with("Core(s) per socket:") {
                    cores = line
                        .split(':')
                        .nth(1)
                        .map(|s| s.trim().to_string())
                        .unwrap_or_default();
                } else if line.starts_with("CPU(s):")
                    && !line.contains("NUMA")
                    && !line.contains("On-line")
                {
                    threads = line
                        .split(':')
                        .nth(1)
                        .map(|s| s.trim().to_string())
                        .unwrap_or_default();
                }
            }

            if model.is_empty() {
                "(unknown CPU)".to_string()
            } else {
                format!("{}, {} cores, {} threads", model, cores, threads)
            }
        }
        _ => "(lscpu not available)".to_string(),
    }
}

// ============================================================================
// Memory Info
// ============================================================================

fn get_memory_summary() -> String {
    // Try free -h
    let output = Command::new("free").arg("-h").output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);

            // Parse the "Mem:" line
            for line in stdout.lines() {
                if line.starts_with("Mem:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        return format!("{} RAM", parts[1]);
                    }
                }
            }
            "(unknown)".to_string()
        }
        _ => "(free not available)".to_string(),
    }
}

// ============================================================================
// GPU Info
// ============================================================================

fn get_gpu_summary() -> String {
    let gpus = list_gpus();
    if gpus.is_empty() {
        "(none detected)".to_string()
    } else {
        // Get GPU names from lspci
        let output = Command::new("lspci").output();

        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let mut gpu_names = Vec::new();

                for line in stdout.lines() {
                    if line.contains("VGA")
                        || line.contains("3D controller")
                        || line.contains("Display controller")
                    {
                        // Extract the device name after the bracket
                        if let Some(idx) = line.find(':') {
                            let name = line[idx + 1..].trim();
                            // Shorten common names
                            let short_name = shorten_gpu_name(name);
                            gpu_names.push(short_name);
                        }
                    }
                }

                if gpu_names.is_empty() {
                    format!("{} controller(s)", gpus.len())
                } else {
                    format!("{} controller(s) ({})", gpus.len(), gpu_names.join(", "))
                }
            }
            _ => format!("{} controller(s)", gpus.len()),
        }
    }
}

fn shorten_gpu_name(name: &str) -> String {
    // Extract key identifying info
    if name.contains("NVIDIA") {
        if let Some(idx) = name.find('[') {
            if let Some(end) = name.find(']') {
                return name[idx + 1..end].to_string();
            }
        }
        // Try to find model number
        if name.contains("RTX") || name.contains("GTX") {
            let parts: Vec<&str> = name.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                if *part == "RTX" || *part == "GTX" {
                    if i + 1 < parts.len() {
                        return format!("NVIDIA {} {}", part, parts[i + 1]);
                    }
                }
            }
        }
    }
    if name.contains("AMD") || name.contains("Radeon") {
        if let Some(idx) = name.find('[') {
            if let Some(end) = name.find(']') {
                return name[idx + 1..end].to_string();
            }
        }
    }
    if name.contains("Intel") {
        if let Some(idx) = name.find('[') {
            if let Some(end) = name.find(']') {
                return format!("Intel {}", &name[idx + 1..end]);
            }
        }
    }

    // Truncate if too long
    if name.len() > 40 {
        format!("{}...", &name[..37])
    } else {
        name.to_string()
    }
}

fn list_gpus() -> Vec<String> {
    let mut gpus = Vec::new();

    // Check /sys/class/drm for GPU devices
    if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            // card0, card1, etc. (not renderD*)
            if name.starts_with("card") && !name.contains('-') {
                gpus.push(name);
            }
        }
    }

    gpus.sort();
    // Convert card0 -> gpu0 for cleaner naming
    gpus.iter().map(|g| g.replace("card", "gpu")).collect()
}

// ============================================================================
// Storage Info
// ============================================================================

fn get_storage_summary() -> String {
    let output = Command::new("lsblk")
        .args(["-d", "-n", "-o", "NAME,TYPE"])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);

            let mut nvme = 0;
            let mut sata = 0;
            let mut usb = 0;

            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 && parts[1] == "disk" {
                    let name = parts[0];
                    if name.starts_with("nvme") {
                        nvme += 1;
                    } else if name.starts_with("sd") {
                        // Could be SATA or USB, check removable
                        let removable_path = format!("/sys/block/{}/removable", name);
                        if let Ok(content) = std::fs::read_to_string(&removable_path) {
                            if content.trim() == "1" {
                                usb += 1;
                            } else {
                                sata += 1;
                            }
                        } else {
                            sata += 1;
                        }
                    } else if name.starts_with("mmcblk") {
                        // SD card
                        usb += 1;
                    }
                }
            }

            format!("{} NVMe, {} SATA, {} USB disks", nvme, sata, usb)
        }
        _ => "(lsblk not available)".to_string(),
    }
}

fn list_storage_devices() -> Vec<String> {
    let output = Command::new("lsblk")
        .args(["-d", "-n", "-o", "NAME,TYPE"])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mut devices = Vec::new();

            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 && parts[1] == "disk" {
                    devices.push(parts[0].to_string());
                }
            }

            devices
        }
        _ => Vec::new(),
    }
}

// ============================================================================
// Network Info
// ============================================================================

fn get_network_summary() -> String {
    let interfaces = list_network_interfaces();

    let mut ethernet = 0;
    let mut wifi = 0;
    let mut bluetooth = 0;

    for iface in &interfaces {
        if iface.starts_with("en") || iface.starts_with("eth") {
            ethernet += 1;
        } else if iface.starts_with("wl") || iface.starts_with("wlan") {
            wifi += 1;
        } else if iface.starts_with("bt") || iface.contains("bluetooth") {
            bluetooth += 1;
        }
    }

    // Check for Bluetooth separately via /sys
    if bluetooth == 0 {
        if std::path::Path::new("/sys/class/bluetooth").exists() {
            if let Ok(entries) = std::fs::read_dir("/sys/class/bluetooth") {
                bluetooth = entries.count();
            }
        }
    }

    format!(
        "{} Ethernet, {} Wi-Fi, {} Bluetooth controller(s)",
        ethernet, wifi, bluetooth
    )
}

fn list_network_interfaces() -> Vec<String> {
    let output = Command::new("ip").args(["link", "show"]).output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mut interfaces = Vec::new();

            for line in stdout.lines() {
                // Lines with interface names start with a number and colon
                if line
                    .chars()
                    .next()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false)
                {
                    if let Some(name_part) = line.split(':').nth(1) {
                        let name = name_part
                            .trim()
                            .split('@')
                            .next()
                            .unwrap_or(name_part.trim());
                        // Skip loopback
                        if name != "lo" {
                            interfaces.push(name.to_string());
                        }
                    }
                }
            }

            interfaces
        }
        _ => Vec::new(),
    }
}

// ============================================================================
// Audio Info
// ============================================================================

fn get_audio_summary() -> String {
    let audio = list_audio_devices();
    if audio.is_empty() {
        "(none detected)".to_string()
    } else {
        format!("{} audio controller(s)", audio.len())
    }
}

fn list_audio_devices() -> Vec<String> {
    // Try aplay -l
    let output = Command::new("aplay").args(["-l"]).output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mut devices = Vec::new();

            for line in stdout.lines() {
                if line.starts_with("card ") {
                    // Extract card number
                    if let Some(num_str) = line.split(':').next() {
                        let num = num_str.trim_start_matches("card ").trim();
                        if let Ok(n) = num.parse::<u32>() {
                            let device_name = format!("audio{}", n);
                            if !devices.contains(&device_name) {
                                devices.push(device_name);
                            }
                        }
                    }
                }
            }

            if !devices.is_empty() {
                return devices;
            }
        }
        _ => {}
    }

    // Fallback: check lspci for audio controllers
    let output = Command::new("lspci").output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mut count = 0;

            for line in stdout.lines() {
                if line.contains("Audio device") || line.contains("Multimedia audio") {
                    count += 1;
                }
            }

            (0..count).map(|i| format!("audio{}", i)).collect()
        }
        _ => Vec::new(),
    }
}

// ============================================================================
// Power Info
// ============================================================================

fn get_power_summary() -> String {
    let supplies = list_power_supplies();
    let batteries: Vec<_> = supplies.iter().filter(|s| s.starts_with("BAT")).collect();

    if batteries.is_empty() {
        "(no batteries - desktop system?)".to_string()
    } else {
        format!("{} battery(ies)", batteries.len())
    }
}

fn list_power_supplies() -> Vec<String> {
    let power_path = std::path::Path::new("/sys/class/power_supply");
    if !power_path.exists() {
        return Vec::new();
    }

    let mut supplies = Vec::new();

    if let Ok(entries) = std::fs::read_dir(power_path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            // Only include batteries and AC adapters
            if name.starts_with("BAT") || name.starts_with("AC") || name.starts_with("ADP") {
                supplies.push(name);
            }
        }
    }

    supplies.sort();
    supplies
}

// ============================================================================
// Dependencies Section - v7.6.0
// ============================================================================

fn print_dependencies_section() {
    println!("{}", "[DEPENDENCIES]".cyan());
    println!("  {}", "(hardware monitoring tools)".dimmed());

    let deps = HardwareDeps::check();

    // Show tool status
    println!("  Hardware tools:");
    print_tool_status("smartctl", deps.smartctl.0, "smartmontools", "disk SMART");
    print_tool_status("nvme", deps.nvme.0, "nvme-cli", "NVMe health");
    print_tool_status("sensors", deps.sensors.0, "lm_sensors", "temperature");
    print_tool_status("iw", deps.iw.0, "iw", "wireless info");
    print_tool_status("ethtool", deps.ethtool.0, "ethtool", "ethernet info");

    // NVIDIA is optional
    if deps.nvidia_smi.0 {
        print_tool_status("nvidia-smi", true, "nvidia-utils", "NVIDIA GPU");
    }

    // Count missing
    let missing = deps.missing_tools();
    if !missing.is_empty() {
        println!();
        println!("  Missing tools limit health data. See annactl status [ANNA NEEDS].");
    }

    println!();
}

fn print_tool_status(tool: &str, available: bool, package: &str, purpose: &str) {
    let status = if available {
        "installed".green().to_string()
    } else {
        "missing".yellow().to_string()
    };
    println!("    {:<12} {} {} {}",
        format!("{}:", tool),
        status,
        format!("({})", package).dimmed(),
        format!("- {}", purpose).dimmed()
    );
}
