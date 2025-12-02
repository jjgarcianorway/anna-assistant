//! HW Command v7.17.0 - Structured Hardware Overview
//!
//! Sections organized by category:
//! - [CPU]        Model, cores, microcode
//! - [GPU]        Integrated and discrete GPUs with drivers
//! - [MEMORY]     Installed RAM, slots
//! - [STORAGE]    Devices, filesystems and health (v7.17.0)
//! - [NETWORK]    Interfaces, default route, DNS (v7.17.0)
//! - [AUDIO]      Controller and drivers
//! - [INPUT]      Keyboard, touchpad
//! - [SENSORS]    Temperature providers
//! - [POWER]      Battery and AC adapter
//!
//! All data sourced from:
//! - lscpu, /proc/cpuinfo (CPU)
//! - lspci -k, /sys/class/drm (GPU)
//! - free, /proc/meminfo, dmidecode (Memory)
//! - lsblk, smartctl, nvme, findmnt (Storage) - v7.17.0
//! - ip link/route, nmcli, resolvectl (Network) - v7.17.0
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
use anna_common::grounded::network_topology::{
    get_default_route, get_dns_config, get_interface_manager, InterfaceManager,
};
use anna_common::grounded::storage_topology::{
    get_filesystem_mounts, get_device_health,
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
// [STORAGE] Section - v7.17.0
// ============================================================================

fn print_storage_section() {
    println!("{}", "[STORAGE]".cyan());

    let disks = get_all_disk_health();

    if disks.is_empty() {
        println!("  {}", "not detected".dimmed());
        println!();
        return;
    }

    // Print each device with health status
    println!("  {}:", "Devices".dimmed());
    for disk in &disks {
        let device_name = disk.device.trim_start_matches("/dev/");
        let size = format_size_human(disk.size_bytes);

        // Get detailed health from new v7.17.0 module
        let health = get_device_health(&disk.device);
        let health_status = if health.status == "PASSED" || health.status == "healthy" {
            "OK".green().to_string()
        } else if health.status == "unknown" || health.status == "no data" {
            "?".dimmed().to_string()
        } else {
            health.status.yellow().to_string()
        };

        // Build info string
        let mut info_parts = vec![disk.device_type.clone()];
        if let Some(model) = &disk.model {
            let short_model = if model.len() > 25 {
                format!("{}...", &model[..22])
            } else {
                model.clone()
            };
            info_parts.push(short_model);
        }
        info_parts.push(size);

        println!("    {} [{}]  {}", device_name, health_status, info_parts.join(", "));
    }

    // Get filesystem mounts
    let mounts = get_filesystem_mounts();
    if !mounts.is_empty() {
        println!();
        println!("  {}:", "Filesystems".dimmed());

        // Show key mounts: /, /home, /boot, /var
        let key_mounts = ["/", "/home", "/boot", "/var", "/boot/efi"];
        for mp in &key_mounts {
            if let Some(mount) = mounts.iter().find(|m| m.mountpoint == *mp) {
                let use_pct = mount.use_percent;
                let use_color = if use_pct >= 90 {
                    format!("{}%", use_pct).red().to_string()
                } else if use_pct >= 75 {
                    format!("{}%", use_pct).yellow().to_string()
                } else {
                    format!("{}%", use_pct).green().to_string()
                };

                let device = mount.device.trim_start_matches("/dev/");
                let subvol_info = mount.subvolume.as_ref()
                    .map(|sv| format!(" [{}]", sv))
                    .unwrap_or_default();

                println!(
                    "    {:12} {:8} {:>5}  {}{}",
                    mp, mount.fstype, use_color, device, subvol_info
                );
            }
        }
    }

    println!();
}

fn format_size_human(bytes: u64) -> String {
    if bytes == 0 {
        return "?".to_string();
    }

    const GB: u64 = 1_000_000_000;
    const TB: u64 = 1_000_000_000_000;

    if bytes >= TB {
        format!("{:.1} TB", bytes as f64 / TB as f64)
    } else {
        format!("{:.0} GB", bytes as f64 / GB as f64)
    }
}

// ============================================================================
// [NETWORK] Section - v7.17.0
// ============================================================================

fn print_network_section() {
    println!("{}", "[NETWORK]".cyan());

    let networks = get_network_health();
    let mut found_any = false;

    // Interfaces subsection
    println!("  {}:", "Interfaces".dimmed());

    // WiFi interfaces
    let wifi_ifaces: Vec<_> = networks.iter()
        .filter(|n| n.interface_type == "wifi")
        .collect();

    for iface in wifi_ifaces {
        found_any = true;
        let name = &iface.interface;
        let state = if iface.link_up {
            "up".green().to_string()
        } else {
            "down".dimmed().to_string()
        };

        // Get manager from v7.17.0 module
        let manager = get_interface_manager(name);
        let manager_str = match manager {
            InterfaceManager::NetworkManager => "NetworkManager",
            InterfaceManager::SystemdNetworkd => "systemd-networkd",
            InterfaceManager::Manual => "manual",
            InterfaceManager::Unmanaged => "unmanaged",
            InterfaceManager::Unknown => "unknown",
        };

        // Get IP if up
        let ip_info = if iface.link_up {
            get_interface_ipv4(name)
                .map(|ip| format!(", {}", ip))
                .unwrap_or_default()
        } else {
            String::new()
        };

        println!("    {:8} wifi     {}, {}{}",
            name, state, manager_str, ip_info);
    }

    // Ethernet interfaces
    let eth_ifaces: Vec<_> = networks.iter()
        .filter(|n| n.interface_type == "ethernet")
        .collect();

    for iface in eth_ifaces {
        found_any = true;
        let name = &iface.interface;
        let state = if iface.link_up {
            "up".green().to_string()
        } else {
            "down".dimmed().to_string()
        };

        let manager = get_interface_manager(name);
        let manager_str = match manager {
            InterfaceManager::NetworkManager => "NetworkManager",
            InterfaceManager::SystemdNetworkd => "systemd-networkd",
            InterfaceManager::Manual => "manual",
            InterfaceManager::Unmanaged => "unmanaged",
            InterfaceManager::Unknown => "unknown",
        };

        let ip_info = if iface.link_up {
            get_interface_ipv4(name)
                .map(|ip| format!(", {}", ip))
                .unwrap_or_default()
        } else {
            String::new()
        };

        println!("    {:8} ethernet {}, {}{}",
            name, state, manager_str, ip_info);
    }

    // Loopback
    println!("    {:8} loopback {}", "lo", "up".green().to_string());

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

                println!("    {:8} bluetooth {} (driver: {})", name, "up".green().to_string(), driver);
            }
        }
    }

    if !found_any {
        println!("    {}", "not detected".dimmed());
    }

    // Default route subsection (v7.17.0)
    println!();
    println!("  {}:", "Default route".dimmed());
    let route = get_default_route();
    if let Some(gw) = &route.gateway {
        let via_iface = route.interface.as_deref().unwrap_or("?");
        println!("    via {} dev {}", gw, via_iface);
    } else {
        println!("    {}", "none".dimmed());
    }

    // DNS subsection (v7.17.0)
    println!();
    println!("  {}:", "DNS".dimmed());
    let dns = get_dns_config();
    if dns.servers.is_empty() {
        println!("    {}", "none configured".dimmed());
    } else {
        let servers_str = dns.servers.iter().take(3).cloned().collect::<Vec<_>>().join(", ");
        println!("    {} (source: {})", servers_str, dns.source);
    }

    println!();
}

/// Get IPv4 address for an interface using ip addr
fn get_interface_ipv4(iface: &str) -> Option<String> {
    let output = Command::new("ip")
        .args(["addr", "show", iface])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let line = line.trim();
        if line.starts_with("inet ") {
            // Format: "inet 192.168.1.42/24 brd 192.168.1.255 ..."
            if let Some(addr) = line.split_whitespace().nth(1) {
                return Some(addr.to_string());
            }
        }
    }

    None
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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
