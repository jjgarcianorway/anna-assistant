//! HW Command v7.15.0 - Structured Hardware Overview
//!
//! Sections organized by category:
//! - [CPU]        Model, cores, microcode
//! - [GPU]        Integrated and discrete GPUs with drivers
//! - [MEMORY]     Installed RAM, slots
//! - [STORAGE]    Devices with root filesystem
//! - [NETWORK]    WiFi, Ethernet, Bluetooth
//! - [AUDIO]      Controller and drivers
//! - [INPUT]      Keyboard, touchpad
//! - [SENSORS]    Temperature providers
//! - [POWER]      Battery and AC adapter
//!
//! All data sourced from:
//! - lscpu, /proc/cpuinfo (CPU)
//! - lspci -k, /sys/class/drm (GPU)
//! - free, /proc/meminfo, dmidecode (Memory)
//! - lsblk, smartctl, nvme (Storage)
//! - ip link, iw, nmcli (Network)
//! - /sys/class/bluetooth (Bluetooth)
//! - aplay, lspci (Audio)
//! - /sys/class/input (Input devices)
//! - /sys/class/hwmon, sensors (Sensors)
//! - /sys/class/power_supply (Power)

use anyhow::Result;
use owo_colors::OwoColorize;
use std::process::Command;

use anna_common::grounded::health::{
    get_battery_health, get_network_health, get_all_disk_health,
};

const THIN_SEP: &str = "------------------------------------------------------------";

/// Run the hw overview command - v7.15.0 structured format
pub async fn run() -> Result<()> {
    println!();
    println!("{}", "  Anna Hardware Inventory".bold());
    println!("{}", THIN_SEP);
    println!();

    // [CPU]
    print_cpu_section();

    // [GPU]
    print_gpu_section();

    // [MEMORY]
    print_memory_section();

    // [STORAGE]
    print_storage_section();

    // [NETWORK]
    print_network_section();

    // [AUDIO]
    print_audio_section();

    // [INPUT]
    print_input_section();

    // [SENSORS]
    print_sensors_section();

    // [POWER]
    print_power_section();

    println!("{}", THIN_SEP);
    println!("  {}", "'annactl hw NAME' for a specific component or category.".dimmed());
    println!();

    Ok(())
}

// ============================================================================
// [CPU] Section - v7.15.0
// ============================================================================

fn print_cpu_section() {
    println!("{}", "[CPU]".cyan());

    let output = Command::new("lscpu").output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);

            let mut model = String::new();
            let mut sockets = String::new();
            let mut cores = String::new();
            let mut threads = String::new();

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
                        _ => {}
                    }
                }
            }

            if !model.is_empty() {
                println!("  Model:        {}", model);
            }
            if !sockets.is_empty() {
                println!("  Sockets:      {}", sockets);
            }
            if !cores.is_empty() && !threads.is_empty() {
                println!("  Cores:        {} ({} threads)", cores, threads);
            }

            // Microcode
            if let Some((vendor, version)) = get_microcode_info() {
                println!("  Microcode:    {} (version {})", vendor, version);
            }
        }
        _ => {
            println!("  {}", "(lscpu not available)".dimmed());
        }
    }

    println!();
}

/// Get CPU microcode info from /sys
fn get_microcode_info() -> Option<(String, String)> {
    // Try reading /sys/devices/system/cpu/cpu0/microcode/version
    let version_path = "/sys/devices/system/cpu/cpu0/microcode/version";
    let version = std::fs::read_to_string(version_path).ok()?.trim().to_string();

    if version.is_empty() {
        return None;
    }

    // Determine vendor from /proc/cpuinfo
    let cpuinfo = std::fs::read_to_string("/proc/cpuinfo").ok()?;
    let vendor = if cpuinfo.contains("GenuineIntel") {
        "genuineintel"
    } else if cpuinfo.contains("AuthenticAMD") {
        "authenticamd"
    } else {
        "unknown"
    };

    Some((vendor.to_string(), version))
}

// ============================================================================
// [GPU] Section - v7.15.0
// ============================================================================

fn print_gpu_section() {
    println!("{}", "[GPU]".cyan());

    let output = Command::new("lspci")
        .args(["-k"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mut found_gpu = false;
            let mut in_gpu = false;
            let mut gpu_name = String::new();
            let mut driver = String::new();
            let mut is_integrated = false;

            for line in stdout.lines() {
                if line.contains("VGA") || line.contains("3D controller") || line.contains("Display controller") {
                    in_gpu = true;
                    found_gpu = true;

                    // Extract name
                    if let Some(idx) = line.find(": ") {
                        gpu_name = line[idx + 2..].to_string();
                        // Shorten
                        gpu_name = shorten_gpu_name(&gpu_name);
                    }

                    // Check if integrated (Intel usually)
                    is_integrated = gpu_name.to_lowercase().contains("intel") &&
                                   !gpu_name.to_lowercase().contains("arc");
                } else if in_gpu && line.contains("Kernel driver in use:") {
                    if let Some(drv) = line.split(':').nth(1) {
                        driver = drv.trim().to_string();
                    }

                    // Print this GPU
                    let label = if is_integrated { "Integrated" } else { "Discrete" };
                    println!("  {}:   {} (driver: {})", label, gpu_name, driver.green());

                    in_gpu = false;
                    gpu_name.clear();
                    driver.clear();
                } else if in_gpu && !line.starts_with('\t') && !line.starts_with(' ') {
                    // New device, previous had no driver
                    let label = if is_integrated { "Integrated" } else { "Discrete" };
                    println!("  {}:   {} (driver: {})", label, gpu_name, "none".yellow());
                    in_gpu = false;
                    gpu_name.clear();
                }
            }

            if !found_gpu {
                println!("  {}", "not detected".dimmed());
            }
        }
    } else {
        println!("  {}", "(lspci not available)".dimmed());
    }

    println!();
}

fn shorten_gpu_name(name: &str) -> String {
    // Extract model from brackets if present
    if let Some(idx) = name.find('[') {
        if let Some(end) = name.find(']') {
            return name[idx + 1..end].to_string();
        }
    }

    // Truncate if too long
    if name.len() > 45 {
        format!("{}...", &name[..42])
    } else {
        name.to_string()
    }
}

// ============================================================================
// [MEMORY] Section - v7.15.0
// ============================================================================

fn print_memory_section() {
    println!("{}", "[MEMORY]".cyan());

    // Get total memory from /proc/meminfo
    if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                if let Some(val) = line.split_whitespace().nth(1) {
                    if let Ok(kb) = val.parse::<u64>() {
                        let gib = kb as f64 / (1024.0 * 1024.0);
                        println!("  Installed:    {:.0} GiB", gib);
                    }
                }
                break;
            }
        }
    }

    // Try to get slot info from dmidecode (requires root)
    let output = Command::new("dmidecode")
        .args(["-t", "memory"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mut total_slots = 0;
            let mut used_slots = 0;

            for line in stdout.lines() {
                if line.contains("Number Of Devices:") {
                    if let Some(val) = line.split(':').nth(1) {
                        total_slots = val.trim().parse().unwrap_or(0);
                    }
                } else if line.trim().starts_with("Size:") {
                    let size_line = line.trim();
                    if !size_line.contains("No Module") && !size_line.contains("Unknown") {
                        used_slots += 1;
                    }
                }
            }

            if total_slots > 0 {
                println!("  Layout:       {} slots ({} used)", total_slots, used_slots);
            }
        }
    }

    println!();
}

// ============================================================================
// [STORAGE] Section - v7.15.0
// ============================================================================

fn print_storage_section() {
    println!("{}", "[STORAGE]".cyan());

    let disks = get_all_disk_health();

    if disks.is_empty() {
        println!("  {}", "not detected".dimmed());
        println!();
        return;
    }

    // Count device types
    let nvme_count = disks.iter().filter(|d| d.device_type == "NVMe").count();
    let sata_count = disks.iter().filter(|d| d.device_type == "SATA").count();
    let other_count = disks.len() - nvme_count - sata_count;

    let mut devices_str = Vec::new();
    if nvme_count > 0 {
        devices_str.push(format!("{} NVMe", nvme_count));
    }
    if sata_count > 0 {
        devices_str.push(format!("{} SATA", sata_count));
    }
    if other_count > 0 {
        devices_str.push(format!("{} other", other_count));
    }

    println!("  Devices:      {}", devices_str.join(", "));

    // Find root filesystem device
    let output = Command::new("findmnt")
        .args(["-n", "-o", "SOURCE,FSTYPE,SIZE", "/"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let parts: Vec<&str> = stdout.split_whitespace().collect();
            if parts.len() >= 3 {
                let source = parts[0].trim_start_matches("/dev/");
                let fstype = parts[1];
                let size = parts[2];
                println!("  Root:         {} ({}, {})", source, fstype, size);
            }
        }
    }

    println!();
}

// ============================================================================
// [NETWORK] Section - v7.15.0
// ============================================================================

fn print_network_section() {
    println!("{}", "[NETWORK]".cyan());

    let networks = get_network_health();
    let mut found_any = false;

    // WiFi interfaces
    let wifi_ifaces: Vec<_> = networks.iter()
        .filter(|n| n.interface_type == "wifi")
        .collect();

    for iface in wifi_ifaces {
        found_any = true;
        let driver = iface.driver.as_deref().unwrap_or("unknown");

        // Get WiFi device model
        let model = get_pci_device_name_for_interface(&iface.interface)
            .unwrap_or_else(|| "Wi-Fi adapter".to_string());

        // Check firmware status
        let fw_status = get_driver_firmware_status(driver);

        print!("  WiFi:         {} (driver: {}", model, driver);
        if let Some(status) = fw_status {
            print!(", firmware: {})", status);
        } else {
            print!(")");
        }
        println!();
    }

    // Ethernet interfaces
    let eth_ifaces: Vec<_> = networks.iter()
        .filter(|n| n.interface_type == "ethernet")
        .collect();

    for iface in eth_ifaces {
        found_any = true;
        let driver = iface.driver.as_deref().unwrap_or("unknown");

        let model = get_pci_device_name_for_interface(&iface.interface)
            .unwrap_or_else(|| "Ethernet adapter".to_string());

        println!("  Ethernet:     {} (driver: {})", model, driver);
    }

    // Bluetooth
    let bt_path = std::path::Path::new("/sys/class/bluetooth");
    if bt_path.exists() {
        if let Ok(entries) = std::fs::read_dir(bt_path) {
            for entry in entries.flatten() {
                found_any = true;
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

                // Try to get device name
                let model = get_bluetooth_model(&entry.path())
                    .unwrap_or_else(|| format!("Bluetooth {}", name));

                println!("  Bluetooth:    {} (driver: {})", model, driver);
            }
        }
    }

    if !found_any {
        println!("  {}", "not detected".dimmed());
    }

    println!();
}

fn get_pci_device_name_for_interface(iface: &str) -> Option<String> {
    let device_path = format!("/sys/class/net/{}/device", iface);
    let device = std::path::Path::new(&device_path);

    if !device.exists() {
        return None;
    }

    // Get PCI address from symlink
    if let Ok(link) = std::fs::read_link(&device_path) {
        let link_str = link.to_string_lossy();
        if let Some(pci_part) = link_str.split('/').last() {
            let output = Command::new("lspci")
                .args(["-s", pci_part])
                .output();

            if let Ok(out) = output {
                if out.status.success() {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    if let Some(line) = stdout.lines().next() {
                        if let Some(idx) = line.find(": ") {
                            let name = line[idx + 2..].trim();
                            return Some(shorten_device_name(name, 35));
                        }
                    }
                }
            }
        }
    }

    None
}

fn get_bluetooth_model(device_path: &std::path::Path) -> Option<String> {
    // Try to get from USB or PCI device
    let pci_path = device_path.join("device");
    if let Ok(link) = std::fs::read_link(&pci_path) {
        let link_str = link.to_string_lossy();
        if link_str.contains("usb") {
            // USB Bluetooth - check product name
            if let Ok(product) = std::fs::read_to_string(device_path.join("device/product")) {
                return Some(product.trim().to_string());
            }
        }
    }
    None
}

fn get_driver_firmware_status(driver: &str) -> Option<String> {
    // Check kernel logs for firmware load status
    let output = Command::new("journalctl")
        .args(["-b", "-k", "--no-pager", "-q"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stdout_lower = stdout.to_lowercase();

            // Check for successful firmware load
            let loaded_patterns = [
                format!("{}: loaded firmware", driver.to_lowercase()),
                "loaded firmware version".to_string(),
                "firmware loaded successfully".to_string(),
            ];

            for pattern in &loaded_patterns {
                if stdout_lower.contains(pattern.as_str()) {
                    return Some("loaded".green().to_string());
                }
            }

            // Check for failures
            if stdout_lower.contains(&format!("{}: ", driver.to_lowercase())) &&
               (stdout_lower.contains("firmware: failed") || stdout_lower.contains("firmware not found")) {
                return Some("failed".yellow().to_string());
            }
        }
    }

    // Fallback: check if firmware files exist
    let fw_dirs = match driver {
        "iwlwifi" => vec!["/usr/lib/firmware"],
        "ath10k_pci" | "ath10k_core" => vec!["/usr/lib/firmware/ath10k"],
        "ath11k_pci" | "ath11k_core" => vec!["/usr/lib/firmware/ath11k"],
        "brcmfmac" => vec!["/usr/lib/firmware/brcm"],
        "rtw88_pci" | "rtw89_pci" => vec!["/usr/lib/firmware/rtw88"],
        "mt7921e" | "mt7922e" => vec!["/usr/lib/firmware/mediatek"],
        _ => return None,
    };

    for dir in fw_dirs {
        if std::path::Path::new(dir).exists() {
            return Some("loaded".green().to_string());
        }
    }

    Some("missing".yellow().to_string())
}

fn shorten_device_name(name: &str, max_len: usize) -> String {
    if let Some(idx) = name.find('[') {
        if let Some(end) = name.find(']') {
            return name[idx + 1..end].to_string();
        }
    }

    if name.len() > max_len {
        format!("{}...", &name[..max_len - 3])
    } else {
        name.to_string()
    }
}

// ============================================================================
// [AUDIO] Section - v7.15.0
// ============================================================================

fn print_audio_section() {
    println!("{}", "[AUDIO]".cyan());

    let output = Command::new("lspci")
        .args(["-k"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mut found_audio = false;
            let mut in_audio = false;
            let mut audio_name = String::new();
            let mut drivers = Vec::new();

            for line in stdout.lines() {
                if line.contains("Audio device") || line.contains("Multimedia audio") {
                    in_audio = true;
                    found_audio = true;

                    if let Some(idx) = line.find(": ") {
                        audio_name = line[idx + 2..].to_string();
                        audio_name = shorten_device_name(&audio_name, 40);
                    }
                } else if in_audio && line.contains("Kernel driver in use:") {
                    if let Some(drv) = line.split(':').nth(1) {
                        drivers.push(drv.trim().to_string());
                    }

                    println!("  Controller:   {}", audio_name);
                    println!("  Drivers:      {}", drivers.join(", "));

                    in_audio = false;
                    audio_name.clear();
                    drivers.clear();
                } else if in_audio && line.contains("Kernel modules:") {
                    if let Some(mods) = line.split(':').nth(1) {
                        for m in mods.split(',') {
                            let m = m.trim().to_string();
                            if !drivers.contains(&m) {
                                drivers.push(m);
                            }
                        }
                    }
                }
            }

            if !found_audio {
                println!("  {}", "not detected".dimmed());
            }
        }
    } else {
        println!("  {}", "(lspci not available)".dimmed());
    }

    println!();
}

// ============================================================================
// [INPUT] Section - v7.15.0
// ============================================================================

fn print_input_section() {
    println!("{}", "[INPUT]".cyan());

    let mut found_keyboard = false;
    let mut found_touchpad = false;

    // Skip these pseudo-devices when looking for keyboard
    let skip_devices = [
        "sleep", "power", "video", "lid", "wmi", "button",
        "pc speaker", "hda intel",
    ];

    // Parse /proc/bus/input/devices for input devices
    if let Ok(content) = std::fs::read_to_string("/proc/bus/input/devices") {
        let mut current_name = String::new();

        for line in content.lines() {
            if line.starts_with("N: Name=") {
                current_name = line.trim_start_matches("N: Name=")
                    .trim_matches('"')
                    .to_string();
            } else if line.starts_with("H: Handlers=") {
                let handlers = line.trim_start_matches("H: Handlers=");
                let name_lower = current_name.to_lowercase();

                // Check if keyboard - skip pseudo-devices
                let is_skip = skip_devices.iter().any(|s| name_lower.contains(s));

                if handlers.contains("kbd") && handlers.contains("event")
                   && !is_skip
                   && !found_keyboard {
                    // Prefer actual keyboard names (AT keyboard, USB keyboard, etc.)
                    if name_lower.contains("keyboard") || name_lower.contains("at translated") {
                        println!("  Keyboard:     {}", current_name);
                        found_keyboard = true;
                    }
                }

                // Check if touchpad/mouse
                if (handlers.contains("mouse") || name_lower.contains("touchpad"))
                   && !name_lower.contains("trackpoint")
                   && !found_touchpad {
                    if name_lower.contains("touchpad") {
                        println!("  Touchpad:     {}", current_name);
                        found_touchpad = true;
                    }
                }
            }
        }
    }

    if !found_keyboard && !found_touchpad {
        println!("  {}", "(no input devices detected)".dimmed());
    }

    println!();
}

// ============================================================================
// [SENSORS] Section - v7.15.0
// ============================================================================

fn print_sensors_section() {
    println!("{}", "[SENSORS]".cyan());

    let hwmon_path = std::path::Path::new("/sys/class/hwmon");
    if !hwmon_path.exists() {
        println!("  {}", "not available".dimmed());
        println!();
        return;
    }

    let mut providers = Vec::new();

    if let Ok(entries) = std::fs::read_dir(hwmon_path) {
        for entry in entries.flatten() {
            let name_path = entry.path().join("name");
            if let Ok(name) = std::fs::read_to_string(&name_path) {
                let name = name.trim().to_string();
                if !providers.contains(&name) {
                    providers.push(name);
                }
            }
        }
    }

    if providers.is_empty() {
        println!("  {}", "none detected".dimmed());
    } else {
        println!("  Providers:    {}", providers.join(", "));
    }

    println!();
}

// ============================================================================
// [POWER] Section - v7.15.0
// ============================================================================

fn print_power_section() {
    println!("{}", "[POWER]".cyan());

    let batteries = get_battery_health();

    if batteries.is_empty() {
        println!("  Battery:      {}", "not present".dimmed());
    } else {
        for bat in &batteries {
            if let Some(design) = bat.design_capacity_wh {
                println!("  Battery:      present (design {:.0} Wh)", design);
            } else {
                println!("  Battery:      present");
            }
        }
    }

    // Check for AC adapter
    let power_path = std::path::Path::new("/sys/class/power_supply");
    if power_path.exists() {
        if let Ok(entries) = std::fs::read_dir(power_path) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("AC") || name.starts_with("ADP") {
                    // Check if online
                    let online_path = entry.path().join("online");
                    let online = std::fs::read_to_string(&online_path)
                        .map(|s| s.trim() == "1")
                        .unwrap_or(false);

                    if online {
                        println!("  AC adapter:   {}", "connected".green());
                    } else {
                        println!("  AC adapter:   {}", "disconnected".dimmed());
                    }
                    break;
                }
            }
        }
    }

    println!();
}
