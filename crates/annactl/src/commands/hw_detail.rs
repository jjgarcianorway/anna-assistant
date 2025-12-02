//! HW Detail Command v7.36.0 - Driver Guidance & Remediation (Bounded)
//!
//! Two modes:
//! 1. Category profile (cpu, memory, gpu, storage, network, audio, power/battery, sensors,
//!                      usb, bluetooth, thunderbolt, sdcard, camera, firmware, pci) - v7.29.0
//! 2. Specific device profile (gpu0, nvme0n1, wlan0, enp3s0, audio0, wifi, ethernet, BAT0)
//!
//! Profile sections:
//! - [IDENTITY]     Device identification with Bus/Vendor info
//! - [FIRMWARE]     Microcode/firmware status and sources (v7.15.0)
//! - [DRIVER]       Kernel module, loaded status, driver package
//! - [SERVICE LIFECYCLE] Related systemd unit lifecycle (v7.16.0)
//! - [DEPENDENCIES] Module chain and related services (v7.13.0)
//! - [INTERFACES]   Network interface details with state/IP (v7.13.0)
//! - [HEALTH]       Real health metrics (temps, SMART, errors)
//! - [CAPACITY]     Battery-specific capacity/wear/cycles (v7.15.0)
//! - [STATE]        Battery/power current state (v7.15.0)
//! - [HISTORY]      Kernel/driver package changes (v7.18.0)
//! - [RELATIONSHIPS] Driver, firmware, services, software using device (v7.24.0)
//! - [TELEMETRY]    Deterministic trend labels (stable/higher/lower) (v7.20.0)
//! - [LOGS]         Boot-anchored patterns with baseline tags (v7.20.0)
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
    extract_patterns_for_driver, extract_driver_patterns_with_history,
    LogPatternSummary, LogHistorySummary, format_time_short,
};
use anna_common::{find_hardware_related_units, ServiceLifecycle};
use anna_common::change_journal::get_package_history;
use anna_common::grounded::signal_quality::{
    get_wifi_signal, get_storage_signal, get_nvme_signal,
};
// v7.22.0: Scenario lenses
use anna_common::scenario_lens::{
    NetworkLens, StorageLens, GraphicsLens, AudioLens,
    format_bytes as lens_format_bytes,
};
// v7.24.0: Relationships
use anna_common::relationships::{
    get_hardware_relationships, format_hardware_relationships_section,
};
// v7.25.0: Peripherals
use anna_common::grounded::peripherals::{
    get_usb_summary, get_bluetooth_summary, get_thunderbolt_summary,
    get_firewire_summary, get_sdcard_summary, BluetoothState,
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
        "bluetooth" | "bt" => run_bluetooth_category().await,
        "audio" | "sound" => run_audio_category().await,
        "power" | "battery" => run_battery_profile().await,
        // v7.25.0: Peripheral categories
        "usb" => run_usb_category().await,
        "thunderbolt" | "tb" | "tb4" => run_thunderbolt_category().await,
        "sdcard" | "sd" | "mmc" => run_sdcard_category().await,
        "firewire" | "ieee1394" => run_firewire_category().await,
        // v7.29.0: Additional categories
        "sensors" | "hwmon" | "temp" | "thermal" => run_sensors_category().await,
        "camera" | "cam" | "webcam" | "video" => run_camera_category().await,
        "firmware" | "fw" => run_firmware_category().await,
        "pci" => run_pci_category().await,
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
            // v7.29.0: Show all flags, no truncation
            if !flags.is_empty() {
                println!("  Flags:          {}", flags);
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

    // [HISTORY] - v7.18.0: kernel package history
    print_hw_history("linux");

    // [LOGS]
    let _log_summary = print_device_logs("cpu", &["thermal", "throttl", "mce", "cpu"]);

    println!("{}", THIN_SEP);
    println!();
    Ok(())
}

/// Print pattern-based kernel logs for a device/driver - v7.20.0 with baseline tags
fn print_device_logs(device: &str, _keywords: &[&str]) -> LogPatternSummary {
    use anna_common::{find_or_create_device_baseline, tag_pattern, normalize_message};

    println!("{}", "[LOGS]".cyan());

    // v7.14.0: Use pattern extraction for driver
    let summary = extract_patterns_for_driver(device);

    // v7.20.0: Try to get or create baseline for this device
    let baseline = find_or_create_device_baseline(device, 5);

    if summary.is_empty() {
        println!();
        println!("  No warnings or errors recorded for this component in the current boot.");
        if baseline.is_some() {
            println!("  {}", "(baseline established)".dimmed());
        }
        println!();
        println!("  {}", format!("Source: {}", summary.source).dimmed());
        return summary;
    }

    // v7.20.0: Pattern summary header with baseline info
    println!();
    println!("  Patterns (this boot):");
    println!("    Total warnings/errors: {} ({} patterns)",
             summary.total_count.to_string().yellow(),
             summary.pattern_count);
    println!();

    // v7.20.0: Show top 3 patterns with baseline tags
    // v7.29.0: No truncation - show full patterns
    for (i, pattern) in summary.top_patterns(3).iter().enumerate() {
        let time_hint = format_time_short(&pattern.last_seen);

        // v7.20.0: Get baseline tag for this pattern
        let normalized = normalize_message(&pattern.pattern);
        let baseline_tag = tag_pattern(device, &normalized);
        let tag_str = baseline_tag.format();

        let count_str = if pattern.count == 1 {
            "1x".to_string()
        } else {
            format!("{}x", pattern.count)
        };

        // v7.29.0: No truncation
        if tag_str.is_empty() {
            println!("    {}) \"{}\"", i + 1, pattern.pattern);
        } else if tag_str.contains("new since") {
            println!("    {}) \"{}\" {}", i + 1, pattern.pattern, tag_str.yellow());
        } else {
            println!("    {}) \"{}\" {}", i + 1, pattern.pattern, tag_str.dimmed());
        }
        println!("       ({}, last at {})", count_str.dimmed(), time_hint);
    }

    // Show if there are more patterns
    if summary.pattern_count > 3 {
        println!();
        println!("    (and {} more patterns)",
                 summary.pattern_count - 3);
    }

    // v7.20.0: Baseline info
    if let Some(ref bl) = baseline {
        println!();
        println!("  Baseline:");
        println!("    Boot: -{}, {} known warning patterns",
                 bl.boot_id.abs(), bl.warning_count);
    }

    println!();
    println!("  {}", format!("Source: {}", summary.source).dimmed());

    summary
}

/// Print [HISTORY] section for hardware - v7.18.0
/// Shows kernel/driver package changes related to this hardware
fn print_hw_history(driver_pkg: &str) {
    use chrono::{DateTime, Local};

    let pkg_history = get_package_history(driver_pkg);

    // Skip if no history
    if pkg_history.is_empty() {
        return;
    }

    println!("{}", "[HISTORY]".cyan());
    println!("  {}", "(source: pacman.log)".dimmed());

    println!("  Driver package:");
    for event in pkg_history.iter().take(3) {
        let ts = DateTime::from_timestamp(event.timestamp as i64, 0)
            .map(|dt| {
                let local: DateTime<Local> = dt.into();
                local.format("%Y-%m-%d %H:%M").to_string()
            })
            .unwrap_or_else(|| "unknown".to_string());

        let action = event.change_type.as_str();
        let details = event.details.as_ref()
            .map(|d| {
                if let Some(ref new_ver) = d.new_version {
                    if let Some(ref old_ver) = d.old_version {
                        format!("{} -> {}", old_ver, new_ver)
                    } else {
                        new_ver.clone()
                    }
                } else if let Some(ref ver) = d.version {
                    ver.clone()
                } else {
                    String::new()
                }
            })
            .unwrap_or_default();

        if details.is_empty() {
            println!("    {}  {:<12} {}", ts, action, driver_pkg);
        } else {
            println!("    {}  {:<12} {}  {}", ts, action, driver_pkg, details.dimmed());
        }
    }
    // v7.29.0: Show all GPU driver history events (no truncation)
    // Previously limited to 3, now shows all for complete audit trail

    println!();
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

        // Firmware section - v7.10.0, v7.29.0: Show all, no truncation
        println!("  Firmware:");
        let fw_status = get_gpu_firmware_files(drv);
        if fw_status.is_empty() {
            println!("    (no firmware files detected)");
        } else {
            for (path, present) in &fw_status {
                let status = if *present {
                    "[present]".green().to_string()
                } else {
                    "[missing]".yellow().to_string()
                };
                println!("    {:<45} {}", path, status);
            }
        }
        // [HISTORY] - v7.18.0: driver package history
        let pkg_for_history = get_driver_package(drv);
        if let Some(pkg_name) = pkg_for_history {
            println!();
            print_hw_history(&pkg_name);
        }
    } else {
        println!("  Kernel module:   {}", "none".yellow());
        println!("  {}", "Note: PCI device present with no bound kernel driver".dimmed());

        // v7.28.0: Driver guidance for missing driver
        if let Some(ref dev) = pci_device {
            if let Some(ref pci_id) = dev.pci_id {
                // pci_id format: "vendor_id:device_id" e.g., "10de:2860"
                let parts: Vec<&str> = pci_id.split(':').collect();
                if parts.len() == 2 {
                    println!();
                    println!("  {}:", "Driver guidance".yellow());
                    let guidance = get_driver_guidance(parts[0], parts[1]);
                    for line in guidance {
                        println!("    {}", line);
                    }
                }
            }
        }
    }

    println!();

    // [RELATIONSHIPS] - v7.24.0
    print_hardware_relationships_section_fn(name);

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

    // v7.29.0: No truncation - return full description
    desc.to_string()
}

// ============================================================================
// Storage Profiles
// ============================================================================

async fn run_storage_category() -> Result<()> {
    println!();
    println!("{}", "  Anna HW: storage".bold());
    println!("{}", THIN_SEP);
    println!();

    // Build storage lens
    let lens = StorageLens::build();

    // [IDENTITY]
    println!("{}", "[IDENTITY]".cyan());
    let bus_types: Vec<_> = lens.devices.iter()
        .map(|d| d.bus.as_str())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    println!("  Class:        Storage");
    println!("  Components:   {}", bus_types.join(", "));
    println!();

    // [TOPOLOGY]
    println!("{}", "[TOPOLOGY]".cyan());
    println!("  Devices:");
    for dev in &lens.devices {
        println!("    {}:", dev.name.bold());
        if let Some(ref model) = dev.model {
            println!("      model:     {}", model);
        }
        println!("      bus:       {}", dev.bus);
        if let Some(ref driver) = dev.driver {
            println!("      driver:    {}", driver);
        }
        if !dev.mount_points.is_empty() {
            println!("      used by:   {}", dev.mount_points.join(", "));
        }
        println!();
    }

    // [HEALTH]
    println!("{}", "[HEALTH]".cyan());
    println!("  SMART:");
    for dev in &lens.devices {
        if let Some(health) = lens.health.get(&dev.name) {
            let status_display = if health.status == "OK" {
                format!("{}", "OK".green())
            } else if health.status.contains("Warning") || health.status.contains("FAILING") {
                format!("{}", health.status.red())
            } else {
                format!("{}", health.status.dimmed())
            };

            if health.media_errors > 0 || health.critical_warnings > 0 {
                println!(
                    "    {:<12} {} (media errors {}, critical warnings {})",
                    format!("{}:", dev.name),
                    status_display,
                    health.media_errors,
                    health.critical_warnings
                );
            } else {
                println!("    {:<12} {}", format!("{}:", dev.name), status_display);
            }
        }
    }
    println!();

    // Temperature if available
    let temps: Vec<_> = lens.devices.iter()
        .filter_map(|d| {
            lens.health.get(&d.name)
                .and_then(|h| h.temp_avg_24h)
                .map(|t| (d.name.as_str(), t))
        })
        .collect();

    if !temps.is_empty() {
        println!("  Temperature (last 24h):");
        for (name, temp) in temps {
            println!("    {:<12} avg {} C", format!("{}:", name), temp as i32);
        }
        println!();
    }

    // [TELEMETRY]
    println!("{}", "[TELEMETRY]".cyan());
    println!("  IO (last 24h):");
    for dev in &lens.devices {
        if let Some(tel) = lens.telemetry.get(&dev.name) {
            println!(
                "    {:<12} read {}, write {}",
                format!("{}:", dev.name),
                lens_format_bytes(tel.read_bytes_24h),
                lens_format_bytes(tel.write_bytes_24h)
            );
        }
    }
    println!();

    // [LOGS]
    if !lens.log_patterns.is_empty() {
        println!("{}", "[LOGS]".cyan());
        println!("  Storage related patterns (current boot, warning and above):");
        // v7.29.0: No truncation - show full messages
        for (id, msg, count) in lens.log_patterns.iter().take(5) {
            println!(
                "    [{}] {} ({}x)",
                id,
                msg,
                count
            );
        }
        println!();
    }

    // [HISTORY]
    println!("{}", "[HISTORY]".cyan());
    println!("  First seen:");
    for dev in &lens.devices {
        if let Some(date) = lens.first_seen.get(&dev.name) {
            println!("    {:<12} {}", format!("{}:", dev.name), date);
        } else {
            println!("    {:<12} {}", format!("{}:", dev.name), "(from Anna telemetry)".dimmed());
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

    // [SIGNAL] - v7.19.0
    print_storage_signal_section(name, &health.device_type);

    // [RELATIONSHIPS] - v7.24.0
    print_hardware_relationships_section_fn(name);

    // [LOGS]
    print_device_logs(name, &[name, &health.device_type.to_lowercase()]);

    println!("{}", THIN_SEP);
    println!();
    Ok(())
}

/// Print [SIGNAL] section for storage device - v7.19.0
fn print_storage_signal_section(name: &str, device_type: &str) {
    let device = format!("/dev/{}", name);

    println!("{}", "[SIGNAL]".cyan());

    if device_type == "NVMe" {
        let signal = get_nvme_signal(&device);
        println!("  {}", format!("(source: {})", signal.source).dimmed());
        println!();

        // Model
        if !signal.model.is_empty() {
            println!("  Model:        {}", signal.model);
        }

        // Temperature
        if let Some(temp) = signal.temperature_c {
            let temp_color = if temp > 70 {
                format!("{}°C", temp).red().to_string()
            } else if temp > 60 {
                format!("{}°C", temp).yellow().to_string()
            } else {
                format!("{}°C", temp).green().to_string()
            };
            println!("  Temperature:  {}", temp_color);
        }

        // Percentage used (wear indicator)
        if let Some(pct) = signal.percentage_used {
            let pct_color = if pct > 90 {
                format!("{}%", pct).red().to_string()
            } else if pct > 70 {
                format!("{}%", pct).yellow().to_string()
            } else {
                format!("{}%", pct).green().to_string()
            };
            println!("  Wear:         {} used", pct_color);
        }

        // Power on hours
        if let Some(hours) = signal.power_on_hours {
            println!("  Power on:     {} hours", hours);
        }

        // Media errors
        if signal.media_errors > 0 {
            println!("  Media errors: {}", signal.media_errors.to_string().red());
        }

        // Unsafe shutdowns
        if signal.unsafe_shutdowns > 10 {
            println!("  Unsafe shutdowns: {}", signal.unsafe_shutdowns.to_string().yellow());
        }

        // Health assessment
        let health = signal.health();
        let health_str = format!("{} {}", health.emoji(),
            match health {
                anna_common::grounded::signal_quality::SignalHealth::Good => "Good",
                anna_common::grounded::signal_quality::SignalHealth::Warning => "Warning",
                anna_common::grounded::signal_quality::SignalHealth::Critical => "Critical",
                anna_common::grounded::signal_quality::SignalHealth::Unknown => "Unknown",
            });
        println!("  Assessment:   {}", health_str);
    } else {
        let signal = get_storage_signal(&device);
        println!("  {}", format!("(source: {})", signal.source).dimmed());
        println!();

        // SMART status
        if !signal.smart_status.is_empty() && signal.smart_status != "unknown" {
            let status_color = if signal.smart_status == "PASSED" {
                signal.smart_status.clone().green().to_string()
            } else {
                signal.smart_status.clone().red().to_string()
            };
            println!("  SMART:        {}", status_color);
        }

        // Temperature
        if let Some(temp) = signal.temperature_c {
            let temp_color = if temp > 60 {
                format!("{}°C", temp).red().to_string()
            } else if temp > 50 {
                format!("{}°C", temp).yellow().to_string()
            } else {
                format!("{}°C", temp).green().to_string()
            };
            println!("  Temperature:  {}", temp_color);
        }

        // Power on hours
        if let Some(hours) = signal.power_on_hours {
            println!("  Power on:     {} hours", hours);
        }

        // Bad sectors
        if signal.reallocated_sectors > 0 || signal.pending_sectors > 0 {
            println!("  Reallocated:  {} sectors", signal.reallocated_sectors.to_string().yellow());
            println!("  Pending:      {} sectors", signal.pending_sectors.to_string().yellow());
        }

        // Health assessment
        let health = signal.health();
        let health_str = format!("{} {}", health.emoji(),
            match health {
                anna_common::grounded::signal_quality::SignalHealth::Good => "Good",
                anna_common::grounded::signal_quality::SignalHealth::Warning => "Warning",
                anna_common::grounded::signal_quality::SignalHealth::Critical => "Critical",
                anna_common::grounded::signal_quality::SignalHealth::Unknown => "Unknown",
            });
        println!("  Assessment:   {}", health_str);
    }

    println!();
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
    println!("{}", "  Anna HW: network".bold());
    println!("{}", THIN_SEP);
    println!();

    // Build network lens
    let lens = NetworkLens::build();

    // [IDENTITY]
    println!("{}", "[IDENTITY]".cyan());
    let iface_types: Vec<_> = lens.interfaces.iter()
        .map(|i| i.iface_type.as_str())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    println!("  Class:        Network");
    println!("  Components:   {}", iface_types.join(", "));
    println!();

    // [TOPOLOGY]
    println!("{}", "[TOPOLOGY]".cyan());
    for iface in &lens.interfaces {
        println!("  {}:", iface.name.bold());
        println!("    interface:  {}", iface.name);
        if let Some(ref driver) = iface.driver {
            let loaded = if iface.link_state == "up" { "(loaded)" } else { "" };
            println!("    driver:     {} {}", driver.cyan(), loaded);
        }
        if let Some(ref fw) = iface.firmware {
            println!("    firmware:   {}", fw);
        } else if iface.iface_type != "wifi" {
            println!("    firmware:   {}", "not required or not reported".dimmed());
        }
        if let Some(blocked) = iface.rfkill_blocked {
            let status = if blocked { "blocked".red().to_string() } else { "unblocked".green().to_string() };
            println!("    rfkill:     {}", status);
        }
        let link_status = if iface.link_state == "up" {
            format!("{}, {}", "up".green(), if iface.carrier { "carrier" } else { "no carrier" })
        } else {
            iface.link_state.red().to_string()
        };
        println!("    link:       {}", link_status);
        println!();
    }

    // [TELEMETRY]
    println!("{}", "[TELEMETRY]".cyan());
    println!("  Last 24h:");
    for iface in &lens.interfaces {
        if let Some(tel) = lens.telemetry.get(&iface.name) {
            println!(
                "    {:<12} rx {}, tx {}",
                format!("{}:", iface.name),
                lens_format_bytes(tel.rx_bytes_24h),
                lens_format_bytes(tel.tx_bytes_24h)
            );
        }
    }
    println!();

    // [EVENTS]
    if !lens.events.is_empty() {
        println!("{}", "[EVENTS]".cyan());
        println!("  Connection changes (current boot):");
        for event in lens.events.iter().take(5) {
            println!(
                "    [{}] {:40} (seen {} {})",
                event.pattern_id,
                event.description,
                event.count,
                if event.count == 1 { "time" } else { "times" }
            );
        }
        println!();
    }

    // [LOGS]
    if !lens.log_patterns.is_empty() {
        println!("{}", "[LOGS]".cyan());
        println!("  Network related patterns (current boot, warning and above):");
        // v7.29.0: No truncation - show full messages
        for (id, msg, count) in lens.log_patterns.iter().take(5) {
            println!(
                "    [{}] {} ({}x)",
                id,
                msg,
                count
            );
        }
        println!();
    }

    // [HISTORY]
    println!("{}", "[HISTORY]".cyan());
    println!("  First seen:");
    for iface in &lens.interfaces {
        if let Some(date) = lens.first_seen.get(&iface.name) {
            println!("    {:<12} {}", format!("{}:", iface.name), date);
        } else {
            println!("    {:<12} {}", format!("{}:", iface.name), "(from Anna telemetry)".dimmed());
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

    // [SERVICE LIFECYCLE] - v7.16.0
    print_hw_service_lifecycle_section("wifi");

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

        // [SIGNAL] - v7.19.0
        print_wifi_signal_section(&iface.interface);

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

/// Print [SIGNAL] section for WiFi interface - v7.19.0
fn print_wifi_signal_section(interface: &str) {
    let signal = get_wifi_signal(interface);

    println!("{}", "[SIGNAL]".cyan());
    println!("  {}", format!("(source: {})", signal.source).dimmed());
    println!();

    // Signal strength
    if let Some(dbm) = signal.signal_dbm {
        let bars = signal.signal_bars();
        let quality = match dbm {
            d if d >= -50 => "excellent".green().to_string(),
            d if d >= -60 => "good".green().to_string(),
            d if d >= -70 => "fair".yellow().to_string(),
            d if d >= -80 => "weak".yellow().to_string(),
            _ => "very weak".red().to_string(),
        };
        println!("  Signal:       {} dBm {} ({})", dbm, bars, quality);
    } else {
        println!("  Signal:       {}", "not connected".dimmed());
    }

    // SSID
    if !signal.ssid.is_empty() {
        println!("  SSID:         {}", signal.ssid);
    }

    // Bitrates
    if let Some(ref tx) = signal.tx_bitrate {
        println!("  TX bitrate:   {}", tx);
    }
    if let Some(ref rx) = signal.rx_bitrate {
        println!("  RX bitrate:   {}", rx);
    }

    // Error counters - only show if non-zero
    let mut counters = Vec::new();
    if signal.tx_failed > 0 {
        counters.push(format!("tx_failed: {}", signal.tx_failed));
    }
    if signal.tx_retries > 0 {
        counters.push(format!("tx_retries: {}", signal.tx_retries));
    }
    if signal.beacon_loss > 0 {
        counters.push(format!("beacon_loss: {}", signal.beacon_loss));
    }
    if signal.disconnects > 0 {
        counters.push(format!("disconnects (1h): {}", signal.disconnects));
    }

    if !counters.is_empty() {
        println!("  Errors:       {}", counters.join(", ").yellow());
    }

    // Health assessment
    let health = signal.health();
    let health_str = format!("{} {}", health.emoji(),
        match health {
            anna_common::grounded::signal_quality::SignalHealth::Good => "Good",
            anna_common::grounded::signal_quality::SignalHealth::Warning => "Warning",
            anna_common::grounded::signal_quality::SignalHealth::Critical => "Critical",
            anna_common::grounded::signal_quality::SignalHealth::Unknown => "Unknown",
        });
    println!("  Assessment:   {}", health_str);

    println!();
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

    // [SERVICE LIFECYCLE] - v7.16.0
    print_hw_service_lifecycle_section("ethernet");

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

/// Bluetooth category profile - v7.25.0 enhanced with peripherals
async fn run_bluetooth_category() -> Result<()> {
    println!();
    println!("{}", "  Anna HW: bluetooth".bold());
    println!("{}", THIN_SEP);
    println!();

    // Get bluetooth summary from peripherals module
    let bt = get_bluetooth_summary();

    if bt.adapters.is_empty() {
        println!("{}", "[NOT FOUND]".yellow());
        println!("  No Bluetooth controllers detected on this system.");
        println!();
        println!("{}", THIN_SEP);
        println!();
        return Ok(());
    }

    // [SERVICE LIFECYCLE] - v7.16.0
    print_hw_service_lifecycle_section("bluetooth");

    // [ADAPTERS] - v7.25.0
    println!("{}", "[ADAPTERS]".cyan());
    println!("  {}", format!("(source: {})", bt.source).dimmed());

    for adapter in &bt.adapters {
        let state = match adapter.state {
            BluetoothState::Up => "UP".green().to_string(),
            BluetoothState::Blocked => "BLOCKED".yellow().to_string(),
            BluetoothState::Down => "DOWN".dimmed().to_string(),
            BluetoothState::Unknown => "unknown".dimmed().to_string(),
        };

        println!("  {}:", adapter.name);
        println!("    Address:      {}", adapter.address);
        println!("    Manufacturer: {}", adapter.manufacturer);
        println!("    Driver:       {}", adapter.driver);
        println!("    State:        {}", state);
    }

    println!();

    // [LOGS]
    let _log_summary = print_device_logs("bluetooth", &["bluetooth", "btusb", "hci"]);

    // [RELATIONSHIPS] - v7.24.0
    print_hardware_relationships_section_fn("bluetooth");

    println!("{}", THIN_SEP);
    println!();
    Ok(())
}

// ============================================================================
// USB Category - v7.35.1 (enhanced with power, speed, driver per device)
// ============================================================================

async fn run_usb_category() -> Result<()> {
    println!();
    println!("{}", "  Anna HW: usb".bold());
    println!("{}", THIN_SEP);
    println!();

    let usb = get_usb_summary();

    // [CONTROLLERS]
    println!("{}", "[CONTROLLERS]".cyan());
    println!("  {}", format!("(source: {})", usb.source).dimmed());

    if usb.controllers.is_empty() {
        println!("  {}", "not detected".dimmed());
    } else {
        for ctrl in &usb.controllers {
            println!("  {} {} (driver: {})", ctrl.pci_address, ctrl.usb_version, ctrl.driver.green());
        }
    }
    println!();

    // [DEVICES]
    println!("{}", "[DEVICES]".cyan());
    println!("  {} device(s), {} root hub(s)", usb.device_count, usb.root_hubs);
    println!();

    // Group by type
    let hubs: Vec<_> = usb.devices.iter().filter(|d| d.is_hub).collect();
    let storage: Vec<_> = usb.devices.iter().filter(|d| d.device_class == "Mass Storage").collect();
    let hid: Vec<_> = usb.devices.iter().filter(|d| d.device_class == "HID").collect();
    let other: Vec<_> = usb.devices.iter()
        .filter(|d| !d.is_hub && d.device_class != "Mass Storage" && d.device_class != "HID")
        .collect();

    if !hubs.is_empty() {
        println!("  {}:", "Hubs".dimmed());
        for dev in hubs.iter().take(3) {
            println!("    Bus{:02} Dev{:03}: {}", dev.bus, dev.device, dev.product_name);
        }
        if hubs.len() > 3 {
            println!("    ... and {} more", hubs.len() - 3);
        }
        println!();
    }

    // v7.35.1: Enhanced device display with speed, power, driver
    if !storage.is_empty() {
        println!("  {}:", "Storage".dimmed());
        for dev in &storage {
            let speed = if dev.speed.is_empty() { "".to_string() } else { format!(" [{}]", dev.speed) };
            let power = dev.power_ma.map(|p| format!(" {}mA", p)).unwrap_or_default();
            let driver = dev.driver.as_ref().map(|d| format!(" driver:{}", d.green())).unwrap_or_default();
            println!("    Bus{:02} Dev{:03}: {} ({}){}{}{}",
                dev.bus, dev.device, dev.product_name, dev.vendor_name, speed, power, driver);
        }
        println!();
    }

    if !hid.is_empty() {
        println!("  {}:", "HID (keyboards, mice)".dimmed());
        for dev in &hid {
            let speed = if dev.speed.is_empty() { "".to_string() } else { format!(" [{}]", dev.speed) };
            let power = dev.power_ma.map(|p| format!(" {}mA", p)).unwrap_or_default();
            let driver = dev.driver.as_ref().map(|d| format!(" driver:{}", d.green())).unwrap_or_default();
            println!("    Bus{:02} Dev{:03}: {} ({}){}{}{}",
                dev.bus, dev.device, dev.product_name, dev.vendor_name, speed, power, driver);
        }
        println!();
    }

    if !other.is_empty() {
        println!("  {}:", "Other".dimmed());
        for dev in other.iter().take(5) {
            let class = if dev.device_class.is_empty() { "?".to_string() } else { dev.device_class.clone() };
            let speed = if dev.speed.is_empty() { "".to_string() } else { format!(" [{}]", dev.speed) };
            let power = dev.power_ma.map(|p| format!(" {}mA", p)).unwrap_or_default();
            let driver = dev.driver.as_ref().map(|d| format!(" driver:{}", d.green())).unwrap_or_default();
            println!("    Bus{:02} Dev{:03}: {} [{}]{}{}{}",
                dev.bus, dev.device, dev.product_name, class, speed, power, driver);
        }
        if other.len() > 5 {
            println!("    ... and {} more", other.len() - 5);
        }
        println!();
    }

    // [LOGS]
    let _log_summary = print_device_logs("usb", &["usb", "xhci", "ehci"]);

    println!("{}", THIN_SEP);
    println!();
    Ok(())
}

// ============================================================================
// Thunderbolt Category - v7.25.0
// ============================================================================

async fn run_thunderbolt_category() -> Result<()> {
    println!();
    println!("{}", "  Anna HW: thunderbolt".bold());
    println!("{}", THIN_SEP);
    println!();

    let tb = get_thunderbolt_summary();

    if tb.controllers.is_empty() && tb.devices.is_empty() {
        println!("{}", "[NOT FOUND]".yellow());
        println!("  No Thunderbolt controllers detected on this system.");
        println!();
        println!("{}", THIN_SEP);
        println!();
        return Ok(());
    }

    // [CONTROLLERS]
    println!("{}", "[CONTROLLERS]".cyan());
    println!("  {}", format!("(source: {})", tb.source).dimmed());

    for ctrl in &tb.controllers {
        let gen = ctrl.generation.map(|g| format!("Thunderbolt {}", g))
            .unwrap_or_else(|| "Thunderbolt".to_string());
        println!("  {} {} (driver: {})", ctrl.pci_address, gen, ctrl.driver.green());
    }
    println!();

    // [DEVICES]
    if !tb.devices.is_empty() {
        println!("{}", "[DEVICES]".cyan());
        println!("  {} attached device(s)", tb.device_count);
        println!();

        for dev in &tb.devices {
            let auth = if dev.authorized {
                "authorized".green().to_string()
            } else {
                "unauthorized".yellow().to_string()
            };
            println!("  {} {} [{}]", dev.vendor, dev.name, auth);
        }
        println!();
    }

    // [LOGS]
    let _log_summary = print_device_logs("thunderbolt", &["thunderbolt", "intel_vsc"]);

    println!("{}", THIN_SEP);
    println!();
    Ok(())
}

// ============================================================================
// SD Card Category - v7.25.0
// ============================================================================

async fn run_sdcard_category() -> Result<()> {
    println!();
    println!("{}", "  Anna HW: sdcard".bold());
    println!("{}", THIN_SEP);
    println!();

    let sd = get_sdcard_summary();

    if sd.readers.is_empty() {
        println!("{}", "[NOT FOUND]".yellow());
        println!("  No SD card readers detected on this system.");
        println!();
        println!("{}", THIN_SEP);
        println!();
        return Ok(());
    }

    // [READERS]
    println!("{}", "[READERS]".cyan());
    println!("  {}", format!("(source: {})", sd.source).dimmed());
    println!("  {} reader(s) found", sd.reader_count);
    println!();

    for reader in &sd.readers {
        println!("  {}:", reader.name);
        println!("    Bus:       {}", reader.bus);
        println!("    Driver:    {}", reader.driver);

        if reader.media_present {
            let size = reader.media_size
                .map(|s| format_size_bytes_detail(s))
                .unwrap_or_else(|| "?".to_string());
            let fs = reader.media_fs.as_deref().unwrap_or("unknown");
            println!("    Media:     {} ({}, {})", "present".green(), size, fs);
            if let Some(path) = &reader.device_path {
                println!("    Device:    {}", path);
            }
        } else {
            println!("    Media:     {}", "not inserted".dimmed());
        }
        println!();
    }

    // [LOGS]
    let _log_summary = print_device_logs("sdcard", &["mmc", "sdhci", "rtsx"]);

    println!("{}", THIN_SEP);
    println!();
    Ok(())
}

fn format_size_bytes_detail(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.1} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.0} MB", bytes as f64 / MB as f64)
    } else {
        format!("{:.0} KB", bytes as f64 / KB as f64)
    }
}

// ============================================================================
// FireWire Category - v7.25.0
// ============================================================================

async fn run_firewire_category() -> Result<()> {
    println!();
    println!("{}", "  Anna HW: firewire".bold());
    println!("{}", THIN_SEP);
    println!();

    let fw = get_firewire_summary();

    if fw.controllers.is_empty() {
        println!("{}", "[NOT FOUND]".yellow());
        println!("  No FireWire (IEEE 1394) controllers detected on this system.");
        println!();
        println!("{}", THIN_SEP);
        println!();
        return Ok(());
    }

    // [CONTROLLERS]
    println!("{}", "[CONTROLLERS]".cyan());
    println!("  {}", format!("(source: {})", fw.source).dimmed());

    for ctrl in &fw.controllers {
        println!("  {} {} (driver: {})", ctrl.pci_address, ctrl.name, ctrl.driver.green());
    }
    println!();

    // [DEVICES]
    if fw.device_count > 0 {
        println!("{}", "[DEVICES]".cyan());
        println!("  {} device(s) connected", fw.device_count);
        println!();
    }

    // [LOGS]
    let _log_summary = print_device_logs("firewire", &["firewire", "ohci1394"]);

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

    // [RELATIONSHIPS] - v7.24.0
    print_hardware_relationships_section_fn(name);

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

    // [RELATIONSHIPS] - v7.24.0
    print_hardware_relationships_section_fn(name);

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

    // [RELATIONSHIPS] - v7.24.0
    print_hardware_relationships_section_fn("battery");

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

    // [RELATIONSHIPS] - v7.24.0
    print_hardware_relationships_section_fn(name);

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

// ============================================================================
// v7.16.0 Functions: Service Lifecycle and Multi-Window Log History
// ============================================================================

/// Print [SERVICE LIFECYCLE] for hardware-related systemd units - v7.16.0
fn print_hw_service_lifecycle_section(component: &str) {
    let related_units = find_hardware_related_units(component);

    if related_units.is_empty() {
        return;
    }

    println!("{}", "[SERVICE LIFECYCLE]".cyan());
    println!("  {}", "(source: systemctl show, journalctl)".dimmed());
    println!();

    for unit in related_units.iter().take(3) {
        let lifecycle = ServiceLifecycle::query(unit);

        if !lifecycle.exists || lifecycle.is_static {
            continue;
        }

        println!("  {}:", unit.cyan());
        println!("    State:       {}", lifecycle.format_state());
        println!("    Restarts:    {}", lifecycle.format_restarts());

        // Only show failures if any
        if lifecycle.failures_24h > 0 || lifecycle.failures_7d > 0 {
            println!("    Failures:");
            if lifecycle.failures_24h > 0 {
                println!("      last 24h:  {}", lifecycle.failures_24h.to_string().yellow());
            }
            if lifecycle.failures_7d > 0 {
                println!("      last 7d:   {}", lifecycle.failures_7d.to_string().yellow());
            }
        }
        println!();
    }
}

/// Print [LOGS] section with v7.16.0 multi-window history for hardware
#[allow(dead_code)]
fn print_device_logs_v716(device: &str, _keywords: &[&str]) -> LogHistorySummary {
    println!("{}", "[LOGS]".cyan());

    let summary = extract_driver_patterns_with_history(device);

    if summary.is_empty_this_boot() && summary.patterns.is_empty() {
        println!();
        println!("  No warnings or errors recorded for this component.");
        println!();
        println!("  {}", format!("Source: {}", summary.source).dimmed());
        return summary;
    }

    println!();

    // v7.16.0: Severity breakdown for this boot
    println!("  This boot:");
    let total_this_boot = summary.total_this_boot();
    if total_this_boot == 0 {
        println!("    {} {} warnings or errors", "✓".green(), "No".green());
    } else {
        if summary.this_boot_critical > 0 {
            println!("    Critical: {}", summary.this_boot_critical.to_string().red().bold());
        }
        if summary.this_boot_error > 0 {
            println!("    Errors:   {}", summary.this_boot_error.to_string().red());
        }
        if summary.this_boot_warning > 0 {
            println!("    Warnings: {}", summary.this_boot_warning.to_string().yellow());
        }
    }
    println!();

    // v7.16.0: Top patterns with history
    // v7.29.0: No truncation - show full patterns
    if !summary.patterns.is_empty() {
        println!("  Top patterns:");
        for (i, pattern) in summary.top_patterns(3).iter().enumerate() {
            // Build history string
            let mut history_parts = Vec::new();
            if pattern.count_this_boot > 0 {
                history_parts.push(format!("boot: {}", pattern.count_this_boot));
            }
            if pattern.count_7d > pattern.count_this_boot {
                history_parts.push(format!("7d: {}", pattern.count_7d));
            }

            let history_str = if history_parts.is_empty() {
                "no history".to_string()
            } else {
                history_parts.join(", ")
            };

            // v7.29.0: No truncation
            println!("    {}) \"{}\"", i + 1, pattern.pattern);
            println!("       {} ({})", pattern.priority.dimmed(), history_str.dimmed());
        }

        if summary.patterns.len() > 3 {
            println!();
            println!("    (and {} more patterns)",
                     summary.patterns.len() - 3);
        }
    }

    // v7.16.0: Recurring patterns - v7.29.0: No truncation
    let recurring = summary.patterns_with_history();
    if !recurring.is_empty() {
        println!();
        println!("  Recurring (seen in previous boots):");
        for pattern in recurring.iter().take(2) {
            // v7.29.0: No truncation
            println!("    - \"{}\" ({} boots)",
                     pattern.pattern.dimmed(),
                     pattern.boots_seen);
        }
    }

    println!();
    println!("  {}", format!("Source: {}", summary.source).dimmed());

    summary
}

/// Print [RELATIONSHIPS] section for hardware - v7.24.0
fn print_hardware_relationships_section_fn(device: &str) {
    let rels = get_hardware_relationships(device);
    let lines = format_hardware_relationships_section(&rels);
    for line in lines {
        if line.starts_with("[RELATIONSHIPS]") {
            println!("{}", line.cyan());
        } else {
            println!("{}", line);
        }
    }
    println!();
}

// ============================================================================
// v7.28.0: Driver Guidance for Missing Drivers
// ============================================================================

/// v7.28.0: Get driver guidance for a PCI device without driver
fn get_driver_guidance(vendor_id: &str, device_id: &str) -> Vec<String> {
    let mut guidance = Vec::new();

    // Common GPU vendors
    match vendor_id.to_lowercase().as_str() {
        // NVIDIA
        "10de" => {
            guidance.push("Vendor: NVIDIA Corporation".to_string());
            guidance.push("Options:".to_string());
            guidance.push("  1. nvidia (proprietary): sudo pacman -S nvidia nvidia-utils".to_string());
            guidance.push("  2. nvidia-open (open-source): sudo pacman -S nvidia-open".to_string());
            guidance.push("  3. nouveau (open-source, included): modprobe nouveau".to_string());
            guidance.push("Note: Reboot required after driver installation".to_string());
        }
        // AMD
        "1002" => {
            guidance.push("Vendor: AMD/ATI".to_string());
            guidance.push("Driver: amdgpu (included in kernel)".to_string());
            guidance.push("Try: modprobe amdgpu".to_string());
            guidance.push("If blacklisted: check /etc/modprobe.d/*.conf".to_string());
        }
        // Intel
        "8086" => {
            guidance.push("Vendor: Intel Corporation".to_string());
            guidance.push("Driver: i915 (included in kernel)".to_string());
            guidance.push("Try: modprobe i915".to_string());
            guidance.push("Firmware: sudo pacman -S linux-firmware".to_string());
        }
        _ => {
            guidance.push(format!("Unknown vendor: {}", vendor_id));
            guidance.push(format!("Device ID: {}", device_id));
            guidance.push("Search: https://linux-hardware.org".to_string());
            guidance.push(format!("Or: lspci -d {}:{}:* -v", vendor_id, device_id));
        }
    }

    guidance
}

// ============================================================================
// [SENSORS] Category - v7.29.0
// ============================================================================

async fn run_sensors_category() -> Result<()> {
    println!();
    println!("{}", "  Anna HW: Sensors".bold());
    println!("{}", THIN_SEP);
    println!();

    println!("{}", "[SENSORS]".cyan());
    println!("  {}", "(source: /sys/class/hwmon, /sys/class/thermal)".dimmed());

    // Enumerate hwmon devices
    let hwmon_path = std::path::Path::new("/sys/class/hwmon");
    if !hwmon_path.exists() {
        println!("  {}", "(hwmon not available)".dimmed());
        println!();
        println!("{}", THIN_SEP);
        return Ok(());
    }

    let mut sensors: Vec<(String, Vec<(String, f64, Option<f64>, Option<f64>)>)> = Vec::new();

    if let Ok(entries) = std::fs::read_dir(hwmon_path) {
        for entry in entries.flatten() {
            let path = entry.path();

            // Get sensor name
            let name = std::fs::read_to_string(path.join("name"))
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|_| "unknown".to_string());

            let mut readings = Vec::new();

            // Find all temp inputs
            for i in 1..=20 {
                let temp_path = path.join(format!("temp{}_input", i));
                if let Ok(temp_str) = std::fs::read_to_string(&temp_path) {
                    if let Ok(temp_millic) = temp_str.trim().parse::<i64>() {
                        let temp_c = temp_millic as f64 / 1000.0;

                        // Get label
                        let label = std::fs::read_to_string(path.join(format!("temp{}_label", i)))
                            .map(|s| s.trim().to_string())
                            .unwrap_or_else(|_| format!("temp{}", i));

                        // Get critical threshold
                        let crit = std::fs::read_to_string(path.join(format!("temp{}_crit", i)))
                            .ok()
                            .and_then(|s| s.trim().parse::<i64>().ok())
                            .map(|v| v as f64 / 1000.0);

                        // Get max threshold
                        let max = std::fs::read_to_string(path.join(format!("temp{}_max", i)))
                            .ok()
                            .and_then(|s| s.trim().parse::<i64>().ok())
                            .map(|v| v as f64 / 1000.0);

                        readings.push((label, temp_c, max, crit));
                    }
                }
            }

            if !readings.is_empty() {
                sensors.push((name, readings));
            }
        }
    }

    if sensors.is_empty() {
        println!("  {}", "(no sensors detected)".dimmed());
    } else {
        for (name, readings) in &sensors {
            println!();
            println!("  {}:", name.cyan());
            for (label, temp, max, crit) in readings {
                let temp_str = if let Some(c) = crit {
                    if *temp >= *c {
                        format!("{:.1}°C", temp).red().to_string()
                    } else if let Some(m) = max {
                        if *temp >= *m {
                            format!("{:.1}°C", temp).yellow().to_string()
                        } else {
                            format!("{:.1}°C", temp).green().to_string()
                        }
                    } else if *temp >= c * 0.9 {
                        format!("{:.1}°C", temp).yellow().to_string()
                    } else {
                        format!("{:.1}°C", temp).green().to_string()
                    }
                } else {
                    format!("{:.1}°C", temp)
                };

                let threshold_str = match (max, crit) {
                    (Some(m), Some(c)) => format!(" (max: {:.0}°C, crit: {:.0}°C)", m, c),
                    (Some(m), None) => format!(" (max: {:.0}°C)", m),
                    (None, Some(c)) => format!(" (crit: {:.0}°C)", c),
                    (None, None) => String::new(),
                };

                println!("    {:20} {}{}", label, temp_str, threshold_str.dimmed());
            }
        }
    }

    // Check thermal zones
    println!();
    println!("{}", "[THERMAL ZONES]".cyan());
    println!("  {}", "(source: /sys/class/thermal)".dimmed());

    let thermal_path = std::path::Path::new("/sys/class/thermal");
    if thermal_path.exists() {
        if let Ok(entries) = std::fs::read_dir(thermal_path) {
            let mut found_zone = false;
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("thermal_zone") {
                    let path = entry.path();

                    let zone_type = std::fs::read_to_string(path.join("type"))
                        .map(|s| s.trim().to_string())
                        .unwrap_or_else(|_| "unknown".to_string());

                    let temp = std::fs::read_to_string(path.join("temp"))
                        .ok()
                        .and_then(|s| s.trim().parse::<i64>().ok())
                        .map(|v| v as f64 / 1000.0);

                    if let Some(t) = temp {
                        found_zone = true;
                        println!("  {:20} {:.1}°C", zone_type, t);
                    }
                }
            }
            if !found_zone {
                println!("  {}", "(no thermal zones)".dimmed());
            }
        }
    }

    println!();
    println!("{}", THIN_SEP);
    Ok(())
}

// ============================================================================
// [CAMERA] Category - v7.29.0
// ============================================================================

async fn run_camera_category() -> Result<()> {
    println!();
    println!("{}", "  Anna HW: Cameras".bold());
    println!("{}", THIN_SEP);
    println!();

    println!("{}", "[CAMERAS]".cyan());
    println!("  {}", "(source: /sys/class/video4linux, lsusb)".dimmed());

    // Enumerate video4linux devices
    let v4l_path = std::path::Path::new("/sys/class/video4linux");
    if !v4l_path.exists() {
        println!("  {}", "(v4l2 not available)".dimmed());
        println!();
        println!("{}", THIN_SEP);
        return Ok(());
    }

    let mut cameras: Vec<(String, String, String)> = Vec::new();

    if let Ok(entries) = std::fs::read_dir(v4l_path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let path = entry.path();

            // Get device name
            let dev_name = std::fs::read_to_string(path.join("name"))
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|_| "unknown".to_string());

            // Get device path
            let dev_path = format!("/dev/{}", name);

            // Check if this is a capture device (not output/overlay)
            let index = std::fs::read_to_string(path.join("index"))
                .ok()
                .and_then(|s| s.trim().parse::<u32>().ok())
                .unwrap_or(0);

            // Only show primary devices (index 0)
            if index == 0 {
                cameras.push((name, dev_name, dev_path));
            }
        }
    }

    if cameras.is_empty() {
        println!("  {}", "(no cameras detected)".dimmed());
    } else {
        for (name, dev_name, dev_path) in &cameras {
            println!();
            println!("  {} ({}):", name.cyan(), dev_path);
            println!("    Name:       {}", dev_name);

            // Try to get capabilities with v4l2-ctl
            let output = Command::new("v4l2-ctl")
                .args(["--device", dev_path, "--all"])
                .output();

            if let Ok(out) = output {
                if out.status.success() {
                    let stdout = String::from_utf8_lossy(&out.stdout);

                    // Parse driver info
                    for line in stdout.lines() {
                        let line = line.trim();
                        if line.starts_with("Driver name") {
                            if let Some(val) = line.split(':').nth(1) {
                                println!("    Driver:     {}", val.trim());
                            }
                        } else if line.starts_with("Card type") {
                            if let Some(val) = line.split(':').nth(1) {
                                println!("    Card:       {}", val.trim());
                            }
                        } else if line.starts_with("Bus info") {
                            if let Some(val) = line.split(':').nth(1) {
                                println!("    Bus:        {}", val.trim());
                            }
                        }
                    }
                }
            } else {
                println!("    {}", "(v4l2-ctl not installed)".dimmed());
            }
        }
    }

    println!();
    println!("{}", THIN_SEP);
    Ok(())
}

// ============================================================================
// [FIRMWARE] Category - v7.29.0
// ============================================================================

async fn run_firmware_category() -> Result<()> {
    println!();
    println!("{}", "  Anna HW: Firmware".bold());
    println!("{}", THIN_SEP);
    println!();

    println!("{}", "[FIRMWARE]".cyan());
    println!("  {}", "(source: /sys/firmware, dmesg)".dimmed());

    // BIOS/UEFI info
    println!();
    println!("  {}:", "System Firmware".dimmed());

    let bios_vendor = std::fs::read_to_string("/sys/class/dmi/id/bios_vendor")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    let bios_version = std::fs::read_to_string("/sys/class/dmi/id/bios_version")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    let bios_date = std::fs::read_to_string("/sys/class/dmi/id/bios_date")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    println!("    BIOS:         {} {} ({})", bios_vendor, bios_version, bios_date);

    // Product info
    let product_name = std::fs::read_to_string("/sys/class/dmi/id/product_name")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    let product_version = std::fs::read_to_string("/sys/class/dmi/id/product_version")
        .map(|s| s.trim().to_string())
        .ok();

    if let Some(ver) = product_version {
        if !ver.is_empty() && ver != "Not Specified" {
            println!("    Product:      {} ({})", product_name, ver);
        } else {
            println!("    Product:      {}", product_name);
        }
    } else {
        println!("    Product:      {}", product_name);
    }

    // CPU microcode
    let microcode_version = std::fs::read_to_string("/sys/devices/system/cpu/cpu0/microcode/version")
        .map(|s| s.trim().to_string())
        .ok();

    if let Some(ver) = microcode_version {
        println!("    Microcode:    {}", ver);
    }

    // List loaded firmware files from kernel logs
    println!();
    println!("  {}:", "Loaded Firmware Files".dimmed());

    let output = Command::new("journalctl")
        .args(["-b", "-k", "--no-pager", "-q", "--grep=firmware"])
        .output();

    let mut firmware_files: Vec<String> = Vec::new();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);

            for line in stdout.lines() {
                // Look for "Loading firmware" or "loaded firmware" patterns
                if line.contains("firmware:") || line.contains("Loading firmware") || line.contains("loaded firmware") {
                    // Extract firmware filename
                    if let Some(start) = line.find("firmware:") {
                        let rest = &line[start + 9..];
                        if let Some(end) = rest.find(' ') {
                            let fw = rest[..end].trim().to_string();
                            if !firmware_files.contains(&fw) && !fw.is_empty() {
                                firmware_files.push(fw);
                            }
                        } else {
                            let fw = rest.trim().to_string();
                            if !firmware_files.contains(&fw) && !fw.is_empty() {
                                firmware_files.push(fw);
                            }
                        }
                    }
                }
            }
        }
    }

    if firmware_files.is_empty() {
        println!("    {}", "(no firmware load messages found)".dimmed());
    } else {
        for fw in firmware_files.iter().take(15) {
            println!("    {}", fw);
        }
        if firmware_files.len() > 15 {
            println!("    {} ({} more)", "...".dimmed(), firmware_files.len() - 15);
        }
    }

    // Check firmware packages
    println!();
    println!("  {}:", "Firmware Packages".dimmed());

    let output = Command::new("pacman")
        .args(["-Qs", "firmware"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mut count = 0;
            for line in stdout.lines() {
                if line.starts_with("local/") {
                    count += 1;
                    // Extract package name
                    if let Some(name) = line.strip_prefix("local/") {
                        let name = name.split_whitespace().next().unwrap_or(name);
                        println!("    {}", name);
                    }
                }
            }
            if count == 0 {
                println!("    {}", "(no firmware packages found)".dimmed());
            }
        }
    }

    println!();
    println!("{}", THIN_SEP);
    Ok(())
}

// ============================================================================
// [PCI] Category - v7.29.0
// ============================================================================

async fn run_pci_category() -> Result<()> {
    println!();
    println!("{}", "  Anna HW: PCI Devices".bold());
    println!("{}", THIN_SEP);
    println!();

    println!("{}", "[PCI DEVICES]".cyan());
    println!("  {}", "(source: lspci -k)".dimmed());

    let output = Command::new("lspci")
        .args(["-k"])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);

            // Group devices by category
            let mut categories: std::collections::HashMap<String, Vec<(String, String, Option<String>)>> =
                std::collections::HashMap::new();

            let mut current_addr = String::new();
            let mut current_name = String::new();
            let mut current_driver: Option<String> = None;
            let mut current_category = String::new();

            for line in stdout.lines() {
                if !line.starts_with('\t') && !line.starts_with(' ') {
                    // New device - save previous if exists
                    if !current_name.is_empty() {
                        categories.entry(current_category.clone())
                            .or_default()
                            .push((current_addr.clone(), current_name.clone(), current_driver.clone()));
                    }

                    // Parse new device line: "00:00.0 Host bridge: Intel Corporation ..."
                    let parts: Vec<&str> = line.splitn(2, ' ').collect();
                    if parts.len() == 2 {
                        current_addr = parts[0].to_string();

                        // Split on ": " to get category and name
                        if let Some(idx) = parts[1].find(": ") {
                            current_category = parts[1][..idx].to_string();
                            current_name = parts[1][idx + 2..].to_string();
                        } else {
                            current_category = "Other".to_string();
                            current_name = parts[1].to_string();
                        }
                        current_driver = None;
                    }
                } else if line.contains("Kernel driver in use:") {
                    if let Some(drv) = line.split(':').nth(1) {
                        current_driver = Some(drv.trim().to_string());
                    }
                }
            }

            // Save last device
            if !current_name.is_empty() {
                categories.entry(current_category)
                    .or_default()
                    .push((current_addr, current_name, current_driver));
            }

            // Print by category
            let order = [
                "Host bridge", "PCI bridge", "ISA bridge",
                "VGA compatible controller", "3D controller", "Display controller",
                "Audio device", "Multimedia audio controller",
                "Network controller", "Ethernet controller", "Wireless Network Adapter",
                "USB controller", "SATA controller", "NVMe controller",
                "Communication controller", "Serial controller",
                "Signal processing controller", "System peripheral",
            ];

            for cat in order {
                if let Some(devices) = categories.get(cat) {
                    println!();
                    println!("  {}:", cat.cyan());
                    for (addr, name, driver) in devices {
                        let driver_str = driver.as_ref()
                            .map(|d| format!(" [{}]", d.green()))
                            .unwrap_or_else(|| format!(" [{}]", "no driver".yellow()));

                        // Shorten name if it has brackets
                        let short_name = if let Some(idx) = name.find('[') {
                            if let Some(end) = name.find(']') {
                                name[idx + 1..end].to_string()
                            } else {
                                name.clone()
                            }
                        } else {
                            name.clone()
                        };

                        println!("    {} {}{}",
                            addr.dimmed(),
                            short_name,
                            driver_str);
                    }
                }
            }

            // Print remaining categories
            for (cat, devices) in &categories {
                if !order.contains(&cat.as_str()) {
                    println!();
                    println!("  {}:", cat.cyan());
                    for (addr, name, driver) in devices {
                        let driver_str = driver.as_ref()
                            .map(|d| format!(" [{}]", d.green()))
                            .unwrap_or_else(|| format!(" [{}]", "no driver".yellow()));
                        println!("    {} {}{}",
                            addr.dimmed(),
                            name,
                            driver_str);
                    }
                }
            }
        }
        _ => {
            println!("  {}", "(lspci not available)".dimmed());
        }
    }

    println!();
    println!("{}", THIN_SEP);
    Ok(())
}

