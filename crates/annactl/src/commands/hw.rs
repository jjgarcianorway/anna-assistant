//! HW Command v7.7.0 - Anna Hardware Overview
//!
//! Sections:
//! - [OVERVIEW]          Summary of CPU, Memory, GPU, Storage, Network, Audio, Power
//! - [DRIVERS]           Per-category driver info
//! - [HEALTH HIGHLIGHTS] Real health data from sensors/SMART/logs
//! - [HW TELEMETRY]      Per-component usage (placeholder - v7.7.0)
//! - [CATEGORIES]        Component identifiers per category
//! - [DEPENDENCIES]      Hardware tools status (v7.6.0)
//!
//! All data sourced from:
//! - lscpu, /proc/cpuinfo (CPU)
//! - free, /proc/meminfo (Memory)
//! - lspci (GPU, Audio controllers, PCI drivers)
//! - lsblk, smartctl (Storage)
//! - ip link, nmcli (Network)
//! - sensors, /sys/class/thermal (Temperatures)
//! - /sys/class/power_supply (Battery)
//! - journalctl -b -k / dmesg (Firmware messages)

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

    // [OVERVIEW]
    print_overview_section();

    // [DRIVERS]
    print_drivers_section();

    // [HEALTH HIGHLIGHTS]
    print_health_highlights_section();

    // [HW TELEMETRY] - v7.7.0 placeholder
    print_hw_telemetry_section();

    // [CATEGORIES]
    print_categories_section();

    // [DEPENDENCIES] - v7.6.0
    print_dependencies_section();

    println!("{}", THIN_SEP);
    println!();

    Ok(())
}

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

/// Print [HW TELEMETRY] section - v7.7.0 placeholder
/// Future: Per-component CPU/GPU utilization, temperature history, disk I/O
fn print_hw_telemetry_section() {
    println!("{}", "[HW TELEMETRY]".cyan());
    println!("  {}", "(per-component telemetry - planned for v7.8.0)".dimmed());
    println!("  Future data:");
    println!("    - CPU:      utilization %, temperature history");
    println!("    - GPU:      utilization %, VRAM usage, temperature");
    println!("    - Disks:    read/write IOPS, throughput");
    println!("    - Network:  bandwidth usage, packet rates");
    println!();
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
