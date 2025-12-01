//! HW Detail Command v7.15.0 - Hardware Profiles with Health/Driver/Logs
//!
//! Two modes:
//! 1. Category profile (cpu, memory, gpu, storage, network, audio, power/battery, sensors)
//! 2. Specific device profile (gpu0, nvme0n1, wlan0, enp3s0, audio0, wifi, ethernet, BAT0)
//!
//! Profile sections:
//! - [IDENTITY]     Device identification with Bus/Vendor info
//! - [FIRMWARE]     Microcode/firmware status and sources (v7.15.0)
//! - [DRIVER]       Kernel module, loaded status, driver package
//! - [DEPENDENCIES] Module chain and related services (v7.13.0)
//! - [INTERFACES]   Network interface details with state/IP (v7.13.0)
//! - [HEALTH]       Real health metrics (temps, SMART, errors)
//! - [CAPACITY]     Battery-specific capacity/wear/cycles (v7.15.0)
//! - [STATE]        Battery/power current state (v7.15.0)
//! - [TELEMETRY]    Anna telemetry with windows (v7.15.0: 1h/24h/7d trends)
//! - [LOGS]         Pattern-based grouping with counts (v7.14.0)
//! - Cross notes:   Links between components
//!
//! All data from system tools:
//! - lscpu, /proc/cpuinfo (CPU)
//! - free, /proc/meminfo (Memory)
//! - lspci -k, /sys/class/drm, /sys/bus/pci/devices (GPU)
//! - lsblk, smartctl, nvme smart-log (Storage)
//! - ip, nmcli, iw, ethtool, /sys/class/net (Network)
//! - aplay, pactl (Audio)
//! - sensors, /sys/class/thermal (Temperatures)
//! - /sys/class/power_supply, upower (Power/Battery)
//! - journalctl -b -k -p warning..alert, dmesg (Kernel logs) - v7.10.0
//! - lsmod, modinfo, pacman -Qo (Module info) - v7.10.0

use anyhow::Result;
use owo_colors::OwoColorize;
use std::process::Command;

use anna_common::grounded::drivers::get_pci_device_by_class_index;
use anna_common::grounded::health::{
    get_cpu_health, get_disk_health, get_battery_health,
    get_network_health, HealthStatus,
};
use anna_common::grounded::deps::{get_module_deps, get_driver_related_services};
use anna_common::grounded::network::{
    get_interfaces, InterfaceType, format_traffic,
};
use anna_common::grounded::log_patterns::{
    extract_patterns_for_driver, LogPatternSummary, format_time_short,
};

const THIN_SEP: &str = "------------------------------------------------------------";

/// Run hardware profile by name
pub async fn run(name: &str) -> Result<()> {
    let name_lower = name.to_lowercase();

    // Check for category names
    match name_lower.as_str() {
        "cpu" | "cpu0" | "processor" => run_cpu_profile().await,
        "memory" | "mem" | "ram" | "mem0" => run_memory_profile().await,
        "gpu" | "graphics" => run_gpu_category().await,
        "storage" | "disk" | "disks" => run_storage_category().await,
        "network" | "net" => run_network_category().await,
        "wifi" | "wireless" => run_wifi_profile().await,
        "ethernet" | "eth" => run_ethernet_profile().await,
        "bluetooth" | "bt" => run_bluetooth_profile().await,
        "audio" | "sound" => run_audio_category().await,
        "power" | "battery" => run_battery_profile().await,
        _ => {
            // Try specific device
            if name_lower.starts_with("gpu") || name_lower.starts_with("card") {
                run_gpu_profile(&name_lower).await
            } else if name_lower.starts_with("nvme")
                || name_lower.starts_with("sd")
                || name_lower.starts_with("mmcblk")
            {
                run_storage_profile(&name_lower).await
            } else if name_lower.starts_with("en")
                || name_lower.starts_with("eth")
                || name_lower.starts_with("wl")
                || name_lower.starts_with("wlan")
            {
                run_network_interface_profile(&name_lower).await
            } else if name_lower.starts_with("audio") {
                run_audio_profile(&name_lower).await
            } else if name_lower.starts_with("bat") || name_lower.starts_with("adp") {
                run_power_supply_profile(&name_lower).await
            } else {
                run_unknown(name)
            }
        }
    }
}

fn run_unknown(name: &str) -> Result<()> {
    println!();
    println!("{}", format!("  Anna HW: {}", name).bold());
    println!("{}", THIN_SEP);
    println!();
    println!("{}", "[NOT FOUND]".yellow());
    println!("  '{}' is not a recognized hardware component.", name);
    println!();
    println!("  Valid categories: cpu, memory, gpu, storage, network, audio, power");
    println!("  Or specific devices: gpu0, nvme0n1, sda, wlan0, enp3s0, audio0, BAT0");
    println!();
    println!("{}", THIN_SEP);
    println!();
    Ok(())
}

// ============================================================================
// CPU Profile
// ============================================================================

async fn run_cpu_profile() -> Result<()> {
    println!();
    println!("{}", "  Anna HW: cpu".bold());
    println!("{}", THIN_SEP);
    println!();

    // [IDENTITY]
    println!("{}", "[IDENTITY]".cyan());

    let output = Command::new("lscpu").output();

    let mut model = String::new();
    let mut sockets = String::new();
    let mut cores = String::new();
    let mut threads = String::new();
    let mut arch = String::new();
    let mut flags = String::new();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);

            for line in stdout.lines() {
                let parts: Vec<&str> = line.splitn(2, ':').collect();
                if parts.len() == 2 {
                    let key = parts[0].trim();
                    let val = parts[1].trim();

                    match key {
                        "Model name" => model = val.to_string(),
                        "Socket(s)" => sockets = val.to_string(),
                        "Core(s) per socket" => cores = val.to_string(),
                        "CPU(s)" => {
                            if !val.contains("NUMA") && !val.contains("On-line") {
                                threads = val.to_string();
                            }
                        }
                        "Architecture" => arch = val.to_string(),
                        "Flags" => {
                            // Extract key flags
                            let important = ["aes", "avx", "avx2", "avx512f", "fma", "sse4_2"];
                            let found: Vec<_> = important.iter()
                                .filter(|f| val.contains(*f))
                                .map(|s| *s)
                                .collect();
                            if !found.is_empty() {
                                flags = found.join(", ");
                            }
                        }
                        _ => {}
                    }
                }
            }

            println!("  Model:          {}", model);
            println!("  Sockets:        {}", sockets);
            println!("  Cores:          {} ({} threads)", cores, threads);
            println!("  Architecture:   {}", arch);
            if !flags.is_empty() {
                println!("  Flags:          {}, ...", flags);
            }
        }
        _ => {
            println!("  (lscpu not available)");
        }
    }

    println!();

    // [FIRMWARE] - v7.15.0: Microcode section
    println!("{}", "[FIRMWARE]".cyan());
    println!("  {}", "(sources: /sys/devices/system/cpu/microcode, journalctl -b -k)".dimmed());
    println!();

    // Get microcode version
    let version_path = "/sys/devices/system/cpu/cpu0/microcode/version";
    let microcode_version = std::fs::read_to_string(version_path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    // Determine vendor
    let cpuinfo = std::fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
    let vendor = if cpuinfo.contains("GenuineIntel") {
        "genuineintel"
    } else if cpuinfo.contains("AuthenticAMD") {
        "authenticamd"
    } else {
        "unknown"
    };

    if let Some(version) = microcode_version {
        println!("  Microcode:      {} (version {})", vendor, version);
        println!("  Source:         /sys/devices/system/cpu/microcode");

        // Check if microcode package is installed
        let ucode_pkg = if vendor == "genuineintel" { "intel-ucode" } else { "amd-ucode" };
        let pkg_check = Command::new("pacman")
            .args(["-Qi", ucode_pkg])
            .output();

        if let Ok(out) = pkg_check {
            if out.status.success() {
                println!("  Loaded from:    {} {}", ucode_pkg, "[installed]".green());
            }
        }
    } else {
        println!("  Microcode:      {}", "not available".dimmed());
    }

    println!();

    // [DRIVER]
    println!("{}", "[DRIVER]".cyan());
    let cpu_health = get_cpu_health();
    if cpu_health.drivers.is_empty() {
        println!("  Scaling:        {}", "kernel default".dimmed());
    } else {
        println!("  Scaling:        {}", cpu_health.drivers.join(", "));
    }

    println!();

    // [HEALTH]
    println!("{}", "[HEALTH]".cyan());
    if let Some(source) = &cpu_health.temp_source {
        println!("  {}", format!("(sources: {}, journalctl -b)", source).dimmed());
    } else {
        println!("  {}", "(sources: journalctl -b)".dimmed());
    }
    println!();

    if let Some(temp) = cpu_health.current_temp_c {
        println!("  Current temp:    {:.0}°C", temp);
    } else {
        println!("  Current temp:    {}", "unavailable".dimmed());
    }

    if let Some(max) = cpu_health.max_temp_c {
        println!("  Max temp (boot): {:.0}°C", max);
    }

    if cpu_health.throttling_detected {
        println!("  Throttling:      {} ({} events)", "detected".yellow(), cpu_health.throttle_events);
    } else {
        println!("  Throttling:      {}", "none detected this boot".green());
    }

    if cpu_health.alerts.is_empty() {
        println!("  Alerts:          {}", "none".green());
    } else {
        for alert in &cpu_health.alerts {
            println!("  Alerts:          {}", alert.yellow());
        }
    }

    println!();

    // [LOGS]
    let _log_summary = print_device_logs("cpu", &["thermal", "throttl", "mce", "cpu"]);

    println!("{}", THIN_SEP);
    println!();
    Ok(())
}

/// Print pattern-based kernel logs for a device/driver - v7.14.0
fn print_device_logs(device: &str, _keywords: &[&str]) -> LogPatternSummary {
    println!("{}", "[LOGS]".cyan());

    // v7.14.0: Use pattern extraction for driver
    let summary = extract_patterns_for_driver(device);

    if summary.is_empty() {
        println!();
        println!("  No warnings or errors recorded for this component in the current boot.");
        println!();
        println!("  {}", format!("Source: {}", summary.source).dimmed());
        return summary;
    }

    // v7.14.0: Pattern summary header
    println!();
    println!("  Patterns (this boot):");
    println!("    Total warnings/errors: {} ({} patterns)",
             summary.total_count.to_string().yellow(),
             summary.pattern_count);
    println!();

    // v7.14.0: Show top 3 patterns with counts and time hints
    for (i, pattern) in summary.top_patterns(3).iter().enumerate() {
        let time_hint = format_time_short(&pattern.last_seen);

        // Truncate pattern for display if too long
        let display_pattern = if pattern.pattern.len() > 55 {
            format!("{}...", &pattern.pattern[..52])
        } else {
            pattern.pattern.clone()
        };

        let count_str = if pattern.count == 1 {
            "seen 1 time".to_string()
        } else {
            format!("seen {} times", pattern.count)
        };

        println!("    {}) \"{}\"", i + 1, display_pattern);
        println!("       ({}, last at {})", count_str.dimmed(), time_hint);
    }

    // Show if there are more patterns
    if summary.pattern_count > 3 {
        println!();
        println!("    {} ({} more patterns not shown)",
                 "...".dimmed(),
                 summary.pattern_count - 3);
    }

    println!();
    println!("  {}", format!("Source: {}", summary.source).dimmed());

    summary
}

// ============================================================================
// Memory Profile
// ============================================================================

async fn run_memory_profile() -> Result<()> {
    println!();
    println!("{}", "  Anna HW: Memory".bold());
    println!("{}", THIN_SEP);
    println!();

    println!("{}", "[SUMMARY]".cyan());
    println!("  {}", "(source: free, /proc/meminfo)".dimmed());

    let output = Command::new("free").arg("-h").output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);

            for line in stdout.lines() {
                if line.starts_with("Mem:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 7 {
                        println!("  Total:       {}", parts[1]);
                        println!("  Used:        {}", parts[2]);
                        println!("  Free:        {}", parts[3]);
                        println!("  Available:   {}", parts[6]);
                    }
                } else if line.starts_with("Swap:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 4 {
                        println!("  Swap Total:  {}", parts[1]);
                        println!("  Swap Used:   {}", parts[2]);
                    }
                }
            }
        }
        _ => {
            println!("  (free not available)");
        }
    }

    println!();
    println!("{}", THIN_SEP);
    println!();
    Ok(())
}

// ============================================================================
// GPU Profiles
// ============================================================================

async fn run_gpu_category() -> Result<()> {
    println!();
    println!("{}", "  Anna HW: GPU".bold());
    println!("{}", THIN_SEP);
    println!();

    println!("{}", "[CONTROLLERS]".cyan());
    println!("  {}", "(source: lspci)".dimmed());

    let output = Command::new("lspci").arg("-nn").output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mut gpu_idx = 0;

            for line in stdout.lines() {
                if line.contains("VGA")
                    || line.contains("3D controller")
                    || line.contains("Display controller")
                {
                    if let Some(idx) = line.find(' ') {
                        let pci_addr = &line[..idx];
                        let desc = &line[idx + 1..];
                        println!();
                        println!("  {}: {}", format!("gpu{}", gpu_idx).cyan(), pci_addr);
                        println!("    {}", desc);

                        // Check driver
                        let full_addr = format!("0000:{}", pci_addr);
                        let driver_path = format!("/sys/bus/pci/devices/{}/driver", full_addr);
                        if let Ok(link) = std::fs::read_link(&driver_path) {
                            if let Some(driver) = link.file_name() {
                                println!("    Driver: {}", driver.to_string_lossy().green());
                            }
                        } else {
                            println!("    Driver: {}", "none".yellow());
                        }

                        gpu_idx += 1;
                    }
                }
            }

            if gpu_idx == 0 {
                println!("  (no GPU controllers detected)");
            }
        }
        _ => {
            println!("  (lspci not available)");
        }
    }

    println!();
    println!("{}", THIN_SEP);
    println!();
    Ok(())
}

async fn run_gpu_profile(name: &str) -> Result<()> {
    // gpu0 -> card0, or card0 stays card0
    let card_num: u32 = name
        .trim_start_matches("gpu")
        .trim_start_matches("card")
        .parse()
        .unwrap_or(0);

    println!();
    println!("{}", format!("  Anna HW: gpu{}", card_num).bold());
    println!("{}", THIN_SEP);
    println!();

    // Get PCI device info
    let pci_device = get_pci_device_by_class_index("VGA", card_num as usize)
        .or_else(|| get_pci_device_by_class_index("3D controller", card_num as usize))
        .or_else(|| get_pci_device_by_class_index("Display", card_num as usize));

    // [IDENTITY]
    println!("{}", "[IDENTITY]".cyan());

    if let Some(ref dev) = pci_device {
        println!("  PCI:         {}", dev.address);
        println!("  Model:       {}", shorten_description(&dev.description));
        println!("  Class:       {}", dev.class);
        if let Some(ref id) = dev.pci_id {
            println!("  PCI ID:      {}", id);
        }
    } else {
        // Fallback to lspci
        let output = Command::new("lspci").output();

        if let Ok(out) = output {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let mut gpu_idx = 0;

                for line in stdout.lines() {
                    if line.contains("VGA")
                        || line.contains("3D controller")
                        || line.contains("Display controller")
                    {
                        if gpu_idx == card_num {
                            if let Some(idx) = line.find(':') {
                                let pci_addr = line[..idx].trim();
                                let desc = line[idx + 1..].trim();
                                println!("  PCI:         0000:{}", pci_addr);
                                println!("  Description: {}", desc);
                            }
                            break;
                        }
                        gpu_idx += 1;
                    }
                }
            }
        }
    }

    println!();

    // [DRIVER] - v7.10.0 format with module, package, firmware
    println!("{}", "[DRIVER]".cyan());

    let driver_name = if let Some(ref dev) = pci_device {
        dev.driver.clone()
    } else {
        // Try /sys/class/drm
        let card_name = format!("card{}", card_num);
        let drm_path = format!("/sys/class/drm/{}/device/driver", card_name);
        std::fs::read_link(&drm_path)
            .ok()
            .and_then(|link| link.file_name().map(|n| n.to_string_lossy().to_string()))
    };

    if let Some(ref drv) = driver_name {
        // Check if module is loaded
        let loaded = is_module_loaded(drv);
        let loaded_str = if loaded {
            "yes".green().to_string()
        } else {
            "no".yellow().to_string()
        };

        println!("  Kernel module:   {}", drv);
        println!("  Loaded:          {} {}", loaded_str, "(lsmod)".dimmed());

        // Get driver package - v7.10.0
        let pkg = get_driver_package(drv);
        if let Some(pkg_name) = pkg {
            println!("  Driver package:  {} {}", pkg_name, "(pacman -Qo)".dimmed());
        }

        // Firmware section - v7.10.0
        println!("  Firmware:");
        let fw_status = get_gpu_firmware_files(drv);
        if fw_status.is_empty() {
            println!("    (no firmware files detected)");
        } else {
            for (path, present) in fw_status.iter().take(3) {
                let status = if *present {
                    "[present]".green().to_string()
                } else {
                    "[missing]".yellow().to_string()
                };
                println!("    {:<45} {}", path, status);
            }
            if fw_status.len() > 3 {
                println!("    {} more files...", fw_status.len() - 3);
            }
        }
    } else {
        println!("  Kernel module:   {}", "none".yellow());
        println!("  {}", "Note: PCI device present with no bound kernel driver".dimmed());
    }

    println!();
    println!("{}", THIN_SEP);
    println!();
    Ok(())
}

fn shorten_description(desc: &str) -> String {
    // Extract model from brackets if present
    if let Some(idx) = desc.find('[') {
        if let Some(end) = desc.find(']') {
            return desc[idx + 1..end].to_string();
        }
    }

    // Truncate if too long
    if desc.len() > 60 {
        format!("{}...", &desc[..57])
    } else {
        desc.to_string()
    }
}

// ============================================================================
// Storage Profiles
// ============================================================================

async fn run_storage_category() -> Result<()> {
    println!();
    println!("{}", "  Anna HW: Storage".bold());
    println!("{}", THIN_SEP);
    println!();

    println!("{}", "[DEVICES]".cyan());
    println!("  {}", "(source: lsblk)".dimmed());

    let output = Command::new("lsblk")
        .args(["-d", "-o", "NAME,SIZE,MODEL,TYPE"])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            println!();
            for line in stdout.lines() {
                if line.contains("disk") {
                    println!("  {}", line);
                }
            }
        }
        _ => {
            println!("  (lsblk not available)");
        }
    }

    println!();
    println!("{}", THIN_SEP);
    println!();
    Ok(())
}

async fn run_storage_profile(name: &str) -> Result<()> {
    println!();
    println!("{}", format!("  Anna HW: {}", name).bold());
    println!("{}", THIN_SEP);
    println!();

    // Get health data
    let health = get_disk_health(name);

    // [IDENTITY]
    println!("{}", "[IDENTITY]".cyan());
    if let Some(ref model) = health.model {
        println!("  Model:       {}", model);
    }
    println!("  Size:        {}", format_bytes_human(health.size_bytes));
    println!("  Type:        {}", health.device_type);

    println!();

    // [DRIVER]
    println!("{}", "[DRIVER]".cyan());
    let driver = match health.device_type.as_str() {
        "NVMe" => "nvme",
        "SATA" => "sd_mod",
        "USB" => "usb-storage",
        _ => "block",
    };
    println!("  Kernel:      {}", driver);
    println!("  Path:        /dev/{}", name);

    println!();

    // [HEALTH] - v7.15.0 consolidated health/SMART section
    println!("{}", "[HEALTH]".cyan());
    if health.smart_available {
        if health.device_type == "NVMe" {
            println!("  {}", "(source: nvme smart-log /dev/...)".dimmed());
        } else {
            println!("  {}", "(source: smartctl -H -A /dev/...)".dimmed());
        }
        println!();

        // Overall status
        match health.smart_passed {
            Some(true) => println!("  Overall:     {}", "SMART OK".green()),
            Some(false) => println!("  Overall:     {}", "SMART FAILED".red()),
            None => println!("  Overall:     {}", "unknown".dimmed()),
        }

        // Temperature
        if let Some(temp) = health.temperature_c {
            println!("  Temp:        {:.0}°C now", temp);
        }

        // Power-on hours
        if let Some(hours) = health.power_on_hours {
            println!("  Power on:    {} hours", hours);
        }

        // Errors - consolidated line
        let mut errors = Vec::new();
        if let Some(media_errors) = health.media_errors {
            errors.push(format!("{} media errors", media_errors));
        }
        if let Some(reallocated) = health.reallocated_sectors {
            errors.push(format!("{} reallocated sectors", reallocated));
        }
        if let Some(pending) = health.pending_sectors {
            errors.push(format!("{} pending sectors", pending));
        }
        if !errors.is_empty() {
            let has_issues = health.media_errors.unwrap_or(0) > 0
                || health.reallocated_sectors.unwrap_or(0) > 0
                || health.pending_sectors.unwrap_or(0) > 0;
            let errors_str = errors.join(", ");
            if has_issues {
                println!("  Errors:      {}", errors_str.yellow());
            } else {
                println!("  Errors:      {}", errors_str.green());
            }
        }

        // Unsafe shutdowns (NVMe only, if high)
        if let Some(unsafe_shutdowns) = health.unsafe_shutdowns {
            if unsafe_shutdowns > 5 {
                println!("  Unsafe shutdowns: {}", unsafe_shutdowns.to_string().yellow());
            }
        }
    } else if let Some(ref reason) = health.smart_unavailable_reason {
        println!("  {}", format!("SMART unavailable: {}", reason).dimmed());
    } else {
        println!("  {}", "SMART data not available".dimmed());
    }

    // Overall health status
    println!();
    let status_str = match health.status {
        HealthStatus::Ok => "OK".green().to_string(),
        HealthStatus::Warning => "WARNING".yellow().to_string(),
        HealthStatus::Critical => "CRITICAL".red().to_string(),
        HealthStatus::Unknown => "UNKNOWN".dimmed().to_string(),
    };
    println!("  Status:      {}", status_str);

    println!();

    // [LOGS]
    print_device_logs(name, &[name, &health.device_type.to_lowercase()]);

    println!("{}", THIN_SEP);
    println!();
    Ok(())
}

/// Format bytes as human-readable
fn format_bytes_human(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 * 1024 {
        format!("{:.1} TiB", bytes as f64 / (1024.0 * 1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 * 1024 {
        format!("{:.1} GiB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.1} MiB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1} KiB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

// ============================================================================
// Network Profiles
// ============================================================================

async fn run_network_category() -> Result<()> {
    println!();
    println!("{}", "  Anna HW: Network".bold());
    println!("{}", THIN_SEP);
    println!();

    println!("{}", "[INTERFACES]".cyan());
    println!("  {}", "(source: ip link)".dimmed());

    let output = Command::new("ip").args(["link", "show"]).output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            println!();

            for line in stdout.lines() {
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
                        if name != "lo" {
                            // Get state
                            let state = if line.contains("state UP") {
                                "UP".green().to_string()
                            } else if line.contains("state DOWN") {
                                "DOWN".red().to_string()
                            } else {
                                "UNKNOWN".dimmed().to_string()
                            };

                            let iface_type = if name.starts_with("en") || name.starts_with("eth") {
                                "ethernet"
                            } else if name.starts_with("wl") || name.starts_with("wlan") {
                                "wifi"
                            } else if name.starts_with("veth") {
                                "veth"
                            } else if name.starts_with("br") {
                                "bridge"
                            } else if name.starts_with("docker") {
                                "docker"
                            } else {
                                "other"
                            };

                            // Get driver
                            let driver_path = format!("/sys/class/net/{}/device/driver", name);
                            let driver = if let Ok(link) = std::fs::read_link(&driver_path) {
                                link.file_name()
                                    .map(|n| n.to_string_lossy().to_string())
                                    .unwrap_or_default()
                            } else {
                                String::new()
                            };

                            if driver.is_empty() {
                                println!(
                                    "  {:<12} [{:<4}] {}",
                                    name,
                                    state,
                                    iface_type.dimmed()
                                );
                            } else {
                                println!(
                                    "  {:<12} [{:<4}] {} ({})",
                                    name,
                                    state,
                                    iface_type.dimmed(),
                                    driver.cyan()
                                );
                            }
                        }
                    }
                }
            }
        }
        _ => {
            println!("  (ip not available)");
        }
    }

    println!();
    println!("{}", THIN_SEP);
    println!();
    Ok(())
}

/// Wi-Fi category profile
async fn run_wifi_profile() -> Result<()> {
    let networks = get_network_health();
    let wifi_ifaces: Vec<_> = networks.iter()
        .filter(|n| n.interface_type == "wifi")
        .collect();

    println!();
    println!("{}", "  Anna HW: wifi".bold());
    println!("{}", THIN_SEP);
    println!();

    if wifi_ifaces.is_empty() {
        println!("{}", "[NOT FOUND]".yellow());
        println!("  No Wi-Fi interfaces detected on this system.");
        println!();
        println!("{}", THIN_SEP);
        println!();
        return Ok(());
    }

    // [DEPENDENCIES] - v7.13.0
    // Get the first wifi interface driver for dependencies
    if let Some(first) = wifi_ifaces.first() {
        if let Some(ref driver) = first.driver {
            print_driver_dependencies_section(driver);
        }
    }

    // [INTERFACES] - v7.13.0
    print_interfaces_section(InterfaceType::WiFi);

    for iface in wifi_ifaces {
        println!("{}", "[IDENTITY]".cyan());
        println!("  Interface:   {}", iface.interface);
        if let Some(ref driver) = iface.driver {
            println!("  Driver:      {}", driver);
        }

        // Get MAC from /sys
        let mac_path = format!("/sys/class/net/{}/address", iface.interface);
        if let Ok(mac) = std::fs::read_to_string(&mac_path) {
            println!("  MAC:         {}", mac.trim());
        }

        println!();

        // [LINK]
        println!("{}", "[LINK]".cyan());
        println!("  {}", "(source: iw dev, ip -s link)".dimmed());
        println!();

        let state = if iface.link_up {
            "up".green().to_string()
        } else {
            "down".red().to_string()
        };
        println!("  State:       {}", state);

        if let Some(ref ssid) = iface.wifi_ssid {
            println!("  SSID:        {}", ssid);
        }
        if let Some(signal) = iface.wifi_signal_dbm {
            println!("  Signal:      {} dBm", signal);
        }

        // Error counters
        if iface.rx_errors > 0 || iface.tx_errors > 0 {
            println!("  Errors:      RX={}, TX={}", iface.rx_errors, iface.tx_errors);
        }
        if iface.rx_dropped > 0 || iface.tx_dropped > 0 {
            println!("  Dropped:     RX={}, TX={}", iface.rx_dropped, iface.tx_dropped);
        }

        println!();

        // [HEALTH]
        println!("{}", "[HEALTH]".cyan());
        let status_str = match iface.status {
            HealthStatus::Ok => "OK".green().to_string(),
            HealthStatus::Warning => "WARNING".yellow().to_string(),
            HealthStatus::Critical => "CRITICAL".red().to_string(),
            HealthStatus::Unknown => "UNKNOWN".dimmed().to_string(),
        };
        println!("  Status:      {}", status_str);
        if iface.alerts.is_empty() {
            println!("  Alerts:      {}", "none".green());
        } else {
            for alert in &iface.alerts {
                println!("  Alerts:      {}", alert.yellow());
            }
        }

        println!();

        // [LOGS]
        let keywords: Vec<&str> = vec![
            &iface.interface,
            iface.driver.as_deref().unwrap_or("wifi"),
        ];
        let _log_summary = print_device_logs("wifi", &keywords.iter().map(|s| s.as_ref()).collect::<Vec<_>>());
    }

    println!("{}", THIN_SEP);
    println!();
    Ok(())
}

/// Ethernet category profile
async fn run_ethernet_profile() -> Result<()> {
    let networks = get_network_health();
    let eth_ifaces: Vec<_> = networks.iter()
        .filter(|n| n.interface_type == "ethernet")
        .collect();

    println!();
    println!("{}", "  Anna HW: ethernet".bold());
    println!("{}", THIN_SEP);
    println!();

    if eth_ifaces.is_empty() {
        println!("{}", "[NOT FOUND]".yellow());
        println!("  No Ethernet interfaces detected on this system.");
        println!();
        println!("{}", THIN_SEP);
        println!();
        return Ok(());
    }

    // [DEPENDENCIES] - v7.13.0
    // Get the first ethernet interface driver for dependencies
    if let Some(first) = eth_ifaces.first() {
        if let Some(ref driver) = first.driver {
            print_driver_dependencies_section(driver);
        }
    }

    // [INTERFACES] - v7.13.0
    print_interfaces_section(InterfaceType::Ethernet);

    for iface in eth_ifaces {
        println!("{}", "[IDENTITY]".cyan());
        println!("  Interface:   {}", iface.interface);
        if let Some(ref driver) = iface.driver {
            println!("  Driver:      {}", driver);
        }

        println!();

        // [LINK]
        println!("{}", "[LINK]".cyan());
        println!("  {}", "(source: ip -s link, ethtool)".dimmed());
        println!();

        let state = if iface.link_up {
            "up".green().to_string()
        } else {
            "down".red().to_string()
        };
        println!("  State:       {}", state);

        // Error counters
        if iface.rx_errors > 0 || iface.tx_errors > 0 {
            println!("  Errors:      RX={}, TX={}", iface.rx_errors, iface.tx_errors);
        }
        if iface.rx_dropped > 0 || iface.tx_dropped > 0 {
            println!("  Dropped:     RX={}, TX={}", iface.rx_dropped, iface.tx_dropped);
        }

        println!();

        // [HEALTH]
        println!("{}", "[HEALTH]".cyan());
        let status_str = match iface.status {
            HealthStatus::Ok => "OK".green().to_string(),
            HealthStatus::Warning => "WARNING".yellow().to_string(),
            HealthStatus::Critical => "CRITICAL".red().to_string(),
            HealthStatus::Unknown => "UNKNOWN".dimmed().to_string(),
        };
        println!("  Status:      {}", status_str);
        if iface.alerts.is_empty() {
            println!("  Alerts:      {}", "none".green());
        } else {
            for alert in &iface.alerts {
                println!("  Alerts:      {}", alert.yellow());
            }
        }

        println!();

        // [LOGS]
        let driver_kw = iface.driver.as_deref().unwrap_or("ethernet");
        let _log_summary = print_device_logs("ethernet", &[&iface.interface, driver_kw]);
    }

    println!("{}", THIN_SEP);
    println!();
    Ok(())
}

/// Bluetooth category profile
async fn run_bluetooth_profile() -> Result<()> {
    println!();
    println!("{}", "  Anna HW: bluetooth".bold());
    println!("{}", THIN_SEP);
    println!();

    // Check if bluetooth exists
    let bt_path = std::path::Path::new("/sys/class/bluetooth");
    if !bt_path.exists() {
        println!("{}", "[NOT FOUND]".yellow());
        println!("  No Bluetooth controllers detected on this system.");
        println!();
        println!("{}", THIN_SEP);
        println!();
        return Ok(());
    }

    // List bluetooth devices
    if let Ok(entries) = std::fs::read_dir(bt_path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();

            println!("{}", "[IDENTITY]".cyan());
            println!("  Controller:  {}", name);

            // Get driver
            let driver_path = entry.path().join("device/driver");
            if let Ok(link) = std::fs::read_link(&driver_path) {
                if let Some(driver) = link.file_name() {
                    println!("  Driver:      {}", driver.to_string_lossy());
                }
            }

            println!();
        }
    }

    // [LOGS]
    let _log_summary = print_device_logs("bluetooth", &["bluetooth", "btusb", "hci"]);

    println!("{}", THIN_SEP);
    println!();
    Ok(())
}

/// Specific network interface profile
async fn run_network_interface_profile(name: &str) -> Result<()> {
    // Find the interface in health data
    let networks = get_network_health();
    let iface = networks.iter().find(|n| n.interface == name);

    println!();
    println!("{}", format!("  Anna HW: {}", name).bold());
    println!("{}", THIN_SEP);
    println!();

    // [IDENTITY]
    println!("{}", "[IDENTITY]".cyan());

    let iface_type = if name.starts_with("en") || name.starts_with("eth") {
        "Ethernet interface"
    } else if name.starts_with("wl") || name.starts_with("wlan") {
        "Wi-Fi interface"
    } else {
        "Network interface"
    };

    println!("  Type:        {}", iface_type);

    // Try to find driver
    let driver_path = format!("/sys/class/net/{}/device/driver", name);
    let driver_name = if let Ok(link) = std::fs::read_link(&driver_path) {
        link.file_name()
            .map(|n| n.to_string_lossy().to_string())
    } else {
        None
    };

    if let Some(ref drv) = driver_name {
        println!("  Driver:      {}", drv);
    }

    // Get MAC
    let mac_path = format!("/sys/class/net/{}/address", name);
    if let Ok(mac) = std::fs::read_to_string(&mac_path) {
        println!("  MAC:         {}", mac.trim());
    }

    println!();

    // [DEPENDENCIES] - v7.13.0
    if let Some(ref drv) = driver_name {
        print_driver_dependencies_section(drv);
    }

    // [LINK]
    println!("{}", "[LINK]".cyan());
    println!("  {}", "(source: ip -s link)".dimmed());
    println!();

    if let Some(health) = iface {
        let state = if health.link_up {
            "up".green().to_string()
        } else {
            "down".red().to_string()
        };
        println!("  State:       {}", state);

        if let Some(ref ssid) = health.wifi_ssid {
            println!("  SSID:        {}", ssid);
        }
        if let Some(signal) = health.wifi_signal_dbm {
            println!("  Signal:      {} dBm", signal);
        }

        if health.rx_errors > 0 || health.tx_errors > 0 {
            println!("  Errors:      RX={}, TX={}", health.rx_errors, health.tx_errors);
        }
        if health.rx_dropped > 0 || health.tx_dropped > 0 {
            println!("  Dropped:     RX={}, TX={}", health.rx_dropped, health.tx_dropped);
        }

        println!();

        // [HEALTH]
        println!("{}", "[HEALTH]".cyan());
        let status_str = match health.status {
            HealthStatus::Ok => "OK".green().to_string(),
            HealthStatus::Warning => "WARNING".yellow().to_string(),
            HealthStatus::Critical => "CRITICAL".red().to_string(),
            HealthStatus::Unknown => "UNKNOWN".dimmed().to_string(),
        };
        println!("  Status:      {}", status_str);
        if health.alerts.is_empty() {
            println!("  Alerts:      {}", "none".green());
        } else {
            for alert in &health.alerts {
                println!("  Alerts:      {}", alert.yellow());
            }
        }
    } else {
        println!("  State:       {}", "unknown".dimmed());
    }

    println!();

    // [LOGS]
    let driver_kw = driver_name.as_deref().unwrap_or(name);
    print_device_logs(name, &[name, driver_kw]);

    println!("{}", THIN_SEP);
    println!();
    Ok(())
}

// ============================================================================
// Audio Profiles
// ============================================================================

async fn run_audio_category() -> Result<()> {
    println!();
    println!("{}", "  Anna HW: Audio".bold());
    println!("{}", THIN_SEP);
    println!();

    println!("{}", "[DEVICES]".cyan());
    println!("  {}", "(source: aplay -l, lspci)".dimmed());

    let output = Command::new("aplay").args(["-l"]).output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            println!();
            for line in stdout.lines() {
                if line.starts_with("card ") {
                    println!("  {}", line);
                }
            }
        }
        _ => {
            // Fallback to lspci
            let output = Command::new("lspci").output();
            if let Ok(out) = output {
                if out.status.success() {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    println!();
                    for line in stdout.lines() {
                        if line.contains("Audio device") || line.contains("Multimedia audio") {
                            println!("  {}", line);
                        }
                    }
                }
            } else {
                println!("  (aplay and lspci not available)");
            }
        }
    }

    println!();
    println!("{}", THIN_SEP);
    println!();
    Ok(())
}

async fn run_audio_profile(name: &str) -> Result<()> {
    let card_num: u32 = name.trim_start_matches("audio").parse().unwrap_or(0);

    println!();
    println!("{}", format!("  Anna HW: {}", name).bold());
    println!("{}", THIN_SEP);
    println!();

    println!("{}", "[IDENTITY]".cyan());
    println!("  {}", "(source: aplay -l)".dimmed());

    let output = Command::new("aplay").args(["-l"]).output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let target = format!("card {}:", card_num);

            for line in stdout.lines() {
                if line.starts_with(&target) {
                    // Extract card name
                    if let Some(idx) = line.find('[') {
                        if let Some(end) = line.find(']') {
                            println!("  Name:        {}", &line[idx + 1..end]);
                        }
                    }
                    // Extract device
                    if let Some(idx) = line.rfind('[') {
                        if let Some(end) = line.rfind(']') {
                            println!("  Device:      {}", &line[idx + 1..end]);
                        }
                    }
                }
            }
        }
        _ => {
            println!("  (aplay not available)");
        }
    }

    // Try to find driver via /sys/class/sound
    let sound_path = format!("/sys/class/sound/card{}/device/driver", card_num);
    if let Ok(link) = std::fs::read_link(&sound_path) {
        if let Some(driver) = link.file_name() {
            println!();
            println!("{}", "[DRIVER]".cyan());
            println!("  {}", "(source: /sys/class/sound)".dimmed());
            println!("  Kernel driver: {}", driver.to_string_lossy().green());
        }
    }

    println!();
    println!("{}", THIN_SEP);
    println!();
    Ok(())
}

// ============================================================================
// Power/Battery Profile
// ============================================================================

async fn run_battery_profile() -> Result<()> {
    let batteries = get_battery_health();

    println!();
    println!("{}", "  Anna HW: battery".bold());
    println!("{}", THIN_SEP);
    println!();

    if batteries.is_empty() {
        println!("  Battery:   {}", "not present".dimmed());
        println!();
        println!("{}", THIN_SEP);
        println!();
        return Ok(());
    }

    for bat in &batteries {
        // [IDENTITY] - v7.15.0
        println!("{}", "[IDENTITY]".cyan());
        println!("  Battery:       {}", bat.name);

        // Read model info from /sys
        let supply_path = format!("/sys/class/power_supply/{}", bat.name);
        let supply_path = std::path::Path::new(&supply_path);

        if let Ok(tech) = std::fs::read_to_string(supply_path.join("technology")) {
            println!("  Type:          {}", tech.trim());
        }
        if let Ok(mfg) = std::fs::read_to_string(supply_path.join("manufacturer")) {
            println!("  Vendor:        {}", mfg.trim());
        }
        if let Ok(model) = std::fs::read_to_string(supply_path.join("model_name")) {
            println!("  Model:         {}", model.trim());
        }

        println!();

        // [CAPACITY] - v7.15.0
        println!("{}", "[CAPACITY]".cyan());
        println!("  {}", "(source: /sys/class/power_supply)".dimmed());
        println!();

        // Design capacity
        if let Some(design) = bat.design_capacity_wh {
            println!("  Design:        {:.0} Wh", design);
        }

        // Full now (actual max charge)
        if let (Some(full), Some(design)) = (bat.full_capacity_wh, bat.design_capacity_wh) {
            let pct_of_design = (full / design * 100.0).round();
            println!("  Full now:      {:.0} Wh ({}% of design)", full, pct_of_design);
        } else if let Some(full) = bat.full_capacity_wh {
            println!("  Full now:      {:.0} Wh", full);
        }

        // Charge now
        if let (Some(pct), Some(full)) = (bat.capacity_percent, bat.full_capacity_wh) {
            let charge_wh = full * pct as f32 / 100.0;
            let pct_str = format!("{}%", pct);
            let pct_colored = if pct >= 80 {
                pct_str.green().to_string()
            } else if pct >= 20 {
                pct_str.yellow().to_string()
            } else {
                pct_str.red().to_string()
            };
            println!("  Charge now:    {:.0} Wh ({} of full)", charge_wh, pct_colored);
        } else if let Some(pct) = bat.capacity_percent {
            let pct_str = format!("{}%", pct);
            println!("  Charge now:    {}", pct_str);
        }

        // Cycle count
        if let Some(cycles) = bat.cycle_count {
            println!("  Cycles:        {}", cycles);
        }

        println!();

        // [STATE] - v7.15.0 new section
        println!("{}", "[STATE]".cyan());

        // Status
        let status_colored = match bat.status.as_str() {
            "Charging" => bat.status.green().to_string(),
            "Discharging" => bat.status.yellow().to_string(),
            "Full" => bat.status.cyan().to_string(),
            "Not charging" => bat.status.dimmed().to_string(),
            _ => bat.status.clone(),
        };
        println!("  Status:        {}", status_colored);

        // Check AC adapter status
        let power_path = std::path::Path::new("/sys/class/power_supply");
        if power_path.exists() {
            if let Ok(entries) = std::fs::read_dir(power_path) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.starts_with("AC") || name.starts_with("ADP") {
                        let online_path = entry.path().join("online");
                        let online = std::fs::read_to_string(&online_path)
                            .map(|s| s.trim() == "1")
                            .unwrap_or(false);

                        if online {
                            println!("  AC adapter:    {}", "connected".green());
                        } else {
                            println!("  AC adapter:    {}", "not connected".dimmed());
                        }
                        break;
                    }
                }
            }
        }

        println!();

        // [HEALTH] - v7.15.0 with wear info
        println!("{}", "[HEALTH]".cyan());
        let status_str = match bat.health_status {
            HealthStatus::Ok => "OK".green().to_string(),
            HealthStatus::Warning => "WARNING".yellow().to_string(),
            HealthStatus::Critical => "CRITICAL".red().to_string(),
            HealthStatus::Unknown => "UNKNOWN".dimmed().to_string(),
        };
        println!("  Status:        {}", status_str);

        // Wear level
        if let Some(wear) = bat.wear_level_percent {
            let health_pct = 100.0 - wear;
            let health_str = format!("{:.0}% remaining", health_pct);
            let health_colored = if wear > 50.0 {
                health_str.red().to_string()
            } else if wear > 30.0 {
                health_str.yellow().to_string()
            } else {
                health_str.green().to_string()
            };
            println!("  Capacity:      {}", health_colored);
        }

        if !bat.alerts.is_empty() {
            for alert in &bat.alerts {
                println!("  Warning:       {}", alert.yellow());
            }
        }

        println!();
    }

    // [LOGS]
    let _log_summary = print_device_logs("battery", &["bat", "battery", "acpi"]);

    println!("{}", THIN_SEP);
    println!();
    Ok(())
}

async fn run_power_supply_profile(name: &str) -> Result<()> {
    let name_upper = name.to_uppercase();

    println!();
    println!("{}", format!("  Anna HW: {}", name_upper).bold());
    println!("{}", THIN_SEP);
    println!();

    let supply_path = format!("/sys/class/power_supply/{}", name_upper);
    let supply_path = std::path::Path::new(&supply_path);

    if !supply_path.exists() {
        // Try lowercase
        let supply_path_lower = format!("/sys/class/power_supply/{}", name.to_lowercase());
        let supply_path = std::path::Path::new(&supply_path_lower);

        if !supply_path.exists() {
            println!("{}", "[NOT FOUND]".yellow());
            println!("  Power supply '{}' not found", name);
            println!();
            println!("{}", THIN_SEP);
            println!();
            return Ok(());
        }
    }

    println!("{}", "[IDENTITY]".cyan());
    println!("  {}", "(source: /sys/class/power_supply)".dimmed());

    // Read type
    let type_path = supply_path.join("type");
    let supply_type = std::fs::read_to_string(&type_path)
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());

    println!("  Type:        {}", supply_type);

    // Read manufacturer if available
    if let Ok(mfg) = std::fs::read_to_string(supply_path.join("manufacturer")) {
        println!("  Manufacturer: {}", mfg.trim());
    }

    // Read model if available
    if let Ok(model) = std::fs::read_to_string(supply_path.join("model_name")) {
        println!("  Model:       {}", model.trim());
    }

    println!();

    // Battery-specific info
    if supply_type == "Battery" {
        println!("{}", "[STATUS]".cyan());
        print_battery_info(supply_path);
    }

    println!();
    println!("{}", THIN_SEP);
    println!();
    Ok(())
}

fn print_battery_info(supply_path: &std::path::Path) {
    // Status
    let status = std::fs::read_to_string(supply_path.join("status"))
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());

    let status_colored = match status.as_str() {
        "Charging" => status.green().to_string(),
        "Discharging" => status.yellow().to_string(),
        "Full" => status.cyan().to_string(),
        _ => status.clone(),
    };
    println!("    Status:     {}", status_colored);

    // Capacity
    if let Ok(capacity) = std::fs::read_to_string(supply_path.join("capacity")) {
        let pct: u32 = capacity.trim().parse().unwrap_or(0);
        let pct_str = format!("{}%", pct);
        let pct_colored = if pct >= 80 {
            pct_str.green().to_string()
        } else if pct >= 20 {
            pct_str.yellow().to_string()
        } else {
            pct_str.red().to_string()
        };
        println!("    Capacity:   {}", pct_colored);
    }

    // Energy/charge info
    if let Ok(energy_now) = std::fs::read_to_string(supply_path.join("energy_now")) {
        if let Ok(energy_full) = std::fs::read_to_string(supply_path.join("energy_full")) {
            let now: f64 = energy_now.trim().parse().unwrap_or(0.0);
            let full: f64 = energy_full.trim().parse().unwrap_or(1.0);
            println!(
                "    Energy:     {:.1} / {:.1} Wh",
                now / 1_000_000.0,
                full / 1_000_000.0
            );
        }
    }

    // Voltage
    if let Ok(voltage) = std::fs::read_to_string(supply_path.join("voltage_now")) {
        let v: f64 = voltage.trim().parse().unwrap_or(0.0);
        println!("    Voltage:    {:.2} V", v / 1_000_000.0);
    }

    // Cycle count
    if let Ok(cycles) = std::fs::read_to_string(supply_path.join("cycle_count")) {
        let c = cycles.trim();
        if c != "0" && !c.is_empty() {
            println!("    Cycles:     {}", c);
        }
    }
}

// ============================================================================
// v7.10.0 Helper Functions for [DRIVER] section
// ============================================================================

/// Check if a kernel module is loaded - v7.10.0
fn is_module_loaded(module_name: &str) -> bool {
    let output = Command::new("lsmod").output();
    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines().skip(1) {
                if let Some(name) = line.split_whitespace().next() {
                    if name == module_name || name == module_name.replace('-', "_") {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Get the driver package using pacman -Qo - v7.10.0
fn get_driver_package(driver_name: &str) -> Option<String> {
    // Try to find the kernel module file
    let kernel_version = get_kernel_version()?;

    // Try common paths
    let paths = [
        format!("/usr/lib/modules/{}/kernel/drivers/gpu/drm/{}/{}.ko.zst", kernel_version, driver_name, driver_name),
        format!("/usr/lib/modules/{}/kernel/drivers/gpu/drm/{}/{}.ko", kernel_version, driver_name, driver_name),
        format!("/usr/lib/modules/{}/extramodules/{}.ko.zst", kernel_version, driver_name),
        format!("/usr/lib/modules/{}/updates/dkms/{}.ko.zst", kernel_version, driver_name),
    ];

    for path in &paths {
        if std::path::Path::new(path).exists() {
            let output = Command::new("pacman")
                .args(["-Qo", path])
                .output();
            if let Ok(out) = output {
                if out.status.success() {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    // Format: "/path/file is owned by package version"
                    if let Some(owned_by) = stdout.find("is owned by ") {
                        let rest = &stdout[owned_by + 12..];
                        if let Some(pkg) = rest.split_whitespace().next() {
                            return Some(pkg.to_string());
                        }
                    }
                }
            }
        }
    }

    // For nvidia, try nvidia-utils package directly
    if driver_name == "nvidia" {
        let check = Command::new("pacman")
            .args(["-Qi", "nvidia-utils"])
            .output();
        if let Ok(out) = check {
            if out.status.success() {
                return Some("nvidia-utils".to_string());
            }
        }
        // Check nvidia-dkms
        let check = Command::new("pacman")
            .args(["-Qi", "nvidia-dkms"])
            .output();
        if let Ok(out) = check {
            if out.status.success() {
                return Some("nvidia-dkms".to_string());
            }
        }
    }

    None
}

/// Get the running kernel version
fn get_kernel_version() -> Option<String> {
    let output = Command::new("uname")
        .arg("-r")
        .output()
        .ok()?;
    if output.status.success() {
        return Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
    }
    None
}

/// Get GPU firmware files and their presence status - v7.10.0
fn get_gpu_firmware_files(driver_name: &str) -> Vec<(String, bool)> {
    let mut files = Vec::new();

    match driver_name {
        "nvidia" => {
            let fw_dir = "/usr/lib/firmware/nvidia";
            if std::path::Path::new(fw_dir).exists() {
                files.push((fw_dir.to_string(), true));
            } else {
                files.push((fw_dir.to_string(), false));
            }
        }
        "amdgpu" => {
            let fw_dir = "/usr/lib/firmware/amdgpu";
            if std::path::Path::new(fw_dir).exists() {
                // List a few files
                if let Ok(entries) = std::fs::read_dir(fw_dir) {
                    for entry in entries.flatten().take(3) {
                        let path = entry.path().to_string_lossy().to_string();
                        files.push((path, true));
                    }
                }
            } else {
                files.push((fw_dir.to_string(), false));
            }
        }
        "i915" => {
            let fw_dir = "/usr/lib/firmware/i915";
            if std::path::Path::new(fw_dir).exists() {
                files.push((fw_dir.to_string(), true));
            } else {
                files.push((fw_dir.to_string(), false));
            }
        }
        _ => {}
    }

    files
}

// ============================================================================
// v7.13.0 Helper Functions for [DEPENDENCIES] and [INTERFACES] sections
// ============================================================================

/// Print [DEPENDENCIES] section for network drivers - v7.13.0
fn print_driver_dependencies_section(driver_name: &str) {
    println!("{}", "[DEPENDENCIES]".cyan());
    println!("  {}", "(sources: lsmod, modinfo, systemctl)".dimmed());
    println!();

    // Get module dependencies
    let mod_deps = get_module_deps(driver_name);

    // Module chain
    if !mod_deps.chain.is_empty() {
        println!("  Driver module chain:");
        println!("    {}", mod_deps.format_chain());
    } else if !mod_deps.depends.is_empty() {
        println!("  Module depends on:");
        println!("    {}", mod_deps.depends.join(", "));
    } else {
        println!("  Module depends on:  {}", "none".dimmed());
    }

    // Used by
    if !mod_deps.used_by.is_empty() {
        println!("  Used by:");
        println!("    {}", mod_deps.used_by.join(", "));
    }

    // Related services
    let related_services = get_driver_related_services(driver_name);
    if !related_services.is_empty() {
        println!("  Related services:");
        for svc in &related_services {
            println!("    {} {}", svc, "[active]".green());
        }
    }

    println!();
}

/// Print [INTERFACES] section with network interfaces - v7.13.0
fn print_interfaces_section(iface_type: InterfaceType) {
    println!("{}", "[INTERFACES]".cyan());
    println!("  {}", "(sources: /sys/class/net, ip addr)".dimmed());
    println!();

    let interfaces = get_interfaces();
    let filtered: Vec<_> = interfaces.iter()
        .filter(|i| i.iface_type == iface_type)
        .collect();

    if filtered.is_empty() {
        println!("  (no {} interfaces detected)", iface_type.as_str());
        println!();
        return;
    }

    for iface in filtered {
        println!("  {}:", iface.name.cyan());

        // Type and driver
        println!("    Type:       {}", iface.iface_type.as_str());
        if let Some(ref driver) = iface.driver {
            println!("    Driver:     {}", driver);
        }

        // MAC
        if let Some(ref mac) = iface.mac {
            println!("    MAC:        {}", mac);
        }

        // State
        let state_str = match iface.state {
            anna_common::grounded::network::LinkState::Up => "connected".green().to_string(),
            anna_common::grounded::network::LinkState::Down => "disconnected".red().to_string(),
            anna_common::grounded::network::LinkState::Unknown => "unknown".dimmed().to_string(),
        };
        println!("    State:      {}", state_str);

        // IP addresses
        if !iface.ip_addrs.is_empty() {
            println!("    IP:         {}", iface.ip_addrs.join(", "));
        }

        // Traffic (since boot)
        if iface.rx_bytes > 0 || iface.tx_bytes > 0 {
            println!("    Traffic:    RX {} / TX {} (since boot)",
                format_traffic(iface.rx_bytes),
                format_traffic(iface.tx_bytes));
        }

        println!();
    }
}

