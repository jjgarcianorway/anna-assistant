//! System scanning - Gathers hardware and config information.
//!
//! Uses standard Linux tools to discover system state.
//! NO HARDCODING - just captures what's there.

use anyhow::Result;
use std::process::Command;

use super::*;

/// Perform a full system scan and return a profile
pub fn scan_system() -> Result<SystemProfile> {
    tracing::info!("Scanning system profile...");

    let mut profile = SystemProfile::default();

    // Scan hardware
    profile.hardware = scan_hardware()?;

    // Scan configs
    profile.configs = scan_configs()?;

    // Scan system info
    profile.system = scan_system_info()?;

    // Update timestamp
    profile.last_updated = Some(chrono::Utc::now().to_rfc3339());

    tracing::info!(
        "System scan complete: {} PCI devices, {} configs",
        profile.hardware.pci_devices.len(),
        profile.configs.modprobe.len()
            + profile.configs.udev_rules.len()
            + profile.configs.systemd_overrides.len()
    );

    Ok(profile)
}

/// Scan hardware using lspci, lsusb, etc.
fn scan_hardware() -> Result<HardwareProfile> {
    let mut hw = HardwareProfile::default();

    // PCI devices
    hw.pci_devices = scan_pci_devices().unwrap_or_default();

    // USB devices
    hw.usb_devices = scan_usb_devices().unwrap_or_default();

    // CPU
    hw.cpu = get_cpu_info();

    // Memory
    hw.memory_gb = get_memory_gb();

    Ok(hw)
}

/// Scan PCI devices using lspci
fn scan_pci_devices() -> Result<Vec<PciDevice>> {
    let output = Command::new("lspci").args(["-mm", "-nn"]).output()?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();

    for line in stdout.lines() {
        if let Some(dev) = parse_lspci_line(line) {
            devices.push(dev);
        }
    }

    // Get drivers for devices
    if let Ok(driver_output) = Command::new("lspci").args(["-k"]).output() {
        let driver_stdout = String::from_utf8_lossy(&driver_output.stdout);
        update_drivers(&mut devices, &driver_stdout);
    }

    Ok(devices)
}

/// Parse a line from lspci -mm -nn
fn parse_lspci_line(line: &str) -> Option<PciDevice> {
    // Format: Slot "Class" "Vendor" "Device" ...
    let parts: Vec<&str> = line.split('"').collect();
    if parts.len() >= 6 {
        Some(PciDevice {
            slot: parts[0].trim().to_string(),
            class: parts[1].to_string(),
            vendor: parts[3].to_string(),
            device: parts[5].to_string(),
            driver: None,
        })
    } else {
        None
    }
}

/// Update devices with driver information
fn update_drivers(devices: &mut [PciDevice], lspci_k_output: &str) {
    let mut current_slot = String::new();

    for line in lspci_k_output.lines() {
        if !line.starts_with('\t') && !line.starts_with(' ') {
            // New device line - extract slot
            current_slot = line.split_whitespace().next().unwrap_or("").to_string();
        } else if line.contains("Kernel driver in use:") {
            // Driver line
            let driver = line.split(':').nth(1).map(|s| s.trim().to_string());
            if let Some(dev) = devices.iter_mut().find(|d| d.slot.starts_with(&current_slot)) {
                dev.driver = driver;
            }
        }
    }
}

/// Scan USB devices using lsusb
fn scan_usb_devices() -> Result<Vec<UsbDevice>> {
    let output = Command::new("lsusb").output()?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();

    for line in stdout.lines() {
        if let Some(dev) = parse_lsusb_line(line) {
            devices.push(dev);
        }
    }

    Ok(devices)
}

/// Parse a line from lsusb
fn parse_lsusb_line(line: &str) -> Option<UsbDevice> {
    // Format: Bus XXX Device YYY: ID VVVV:PPPP Description
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 7 {
        let bus = parts[1].to_string();
        let device = parts[3].trim_end_matches(':').to_string();
        let id = parts[5];
        let id_parts: Vec<&str> = id.split(':').collect();
        let vendor_id = id_parts.first().unwrap_or(&"").to_string();
        let product_id = id_parts.get(1).unwrap_or(&"").to_string();
        let description = parts[6..].join(" ");

        Some(UsbDevice {
            bus,
            device,
            vendor_id,
            product_id,
            description,
        })
    } else {
        None
    }
}

/// Get CPU info
fn get_cpu_info() -> Option<String> {
    let output = Command::new("lscpu").output().ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        if line.starts_with("Model name:") {
            return Some(line.split(':').nth(1)?.trim().to_string());
        }
    }

    None
}

/// Get memory in GB
fn get_memory_gb() -> Option<u64> {
    let output = Command::new("free").args(["-g"]).output().ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        if line.starts_with("Mem:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            return parts.get(1)?.parse().ok();
        }
    }

    None
}

/// Scan existing configurations
fn scan_configs() -> Result<ConfigProfile> {
    let mut configs = ConfigProfile::default();

    // Scan modprobe.d
    configs.modprobe = scan_directory("/etc/modprobe.d", &["conf"]);

    // Scan udev rules (custom ones)
    configs.udev_rules = scan_directory("/etc/udev/rules.d", &["rules"]);

    // Scan systemd overrides
    configs.systemd_overrides = scan_systemd_overrides();

    // Scan X11 configs
    configs.xorg_configs = scan_directory("/etc/X11/xorg.conf.d", &["conf"]);

    Ok(configs)
}

/// Scan a directory for config files
fn scan_directory(path: &str, extensions: &[&str]) -> Vec<ConfigFile> {
    let mut files = Vec::new();

    let dir = match std::fs::read_dir(path) {
        Ok(d) => d,
        Err(_) => return files,
    };

    for entry in dir.flatten() {
        let path = entry.path();
        if path.is_file() {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if extensions.is_empty() || extensions.contains(&ext) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    // Skip empty files and very large files
                    if !content.trim().is_empty() && content.len() < 10000 {
                        files.push(ConfigFile {
                            path: path.display().to_string(),
                            content,
                        });
                    }
                }
            }
        }
    }

    files
}

/// Scan systemd override directories
fn scan_systemd_overrides() -> Vec<ConfigFile> {
    let mut files = Vec::new();

    // Check /etc/systemd/system for overrides
    let system_dir = "/etc/systemd/system";
    if let Ok(dir) = std::fs::read_dir(system_dir) {
        for entry in dir.flatten() {
            let path = entry.path();

            // Look for .d directories (override directories)
            if path.is_dir() && path.display().to_string().ends_with(".d") {
                files.extend(scan_directory(&path.display().to_string(), &["conf"]));
            }

            // Also check direct override files
            if path.is_file() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if name.ends_with(".service") || name.ends_with(".timer") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if !content.trim().is_empty() && content.len() < 10000 {
                            files.push(ConfigFile {
                                path: path.display().to_string(),
                                content,
                            });
                        }
                    }
                }
            }
        }
    }

    files
}

/// Scan system information
fn scan_system_info() -> Result<SystemInfo> {
    let mut info = SystemInfo::default();

    // OS info from /etc/os-release
    if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if line.starts_with("NAME=") {
                info.os_name = Some(line[5..].trim_matches('"').to_string());
            } else if line.starts_with("VERSION=") {
                info.os_version = Some(line[8..].trim_matches('"').to_string());
            }
        }
    }

    // Kernel version
    if let Ok(output) = Command::new("uname").arg("-r").output() {
        if output.status.success() {
            info.kernel = Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
        }
    }

    // Hostname
    if let Ok(output) = Command::new("hostname").output() {
        if output.status.success() {
            info.hostname = Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
        }
    }

    // Desktop environment
    info.desktop = std::env::var("XDG_CURRENT_DESKTOP").ok();

    // Display server (check for running display managers)
    info.display_server = detect_display_server();

    Ok(info)
}

/// Detect display server (Wayland vs X11)
fn detect_display_server() -> Option<String> {
    // Check loginctl for session type
    if let Ok(output) = Command::new("sh")
        .arg("-c")
        .arg("loginctl show-session $(loginctl list-sessions --no-legend | head -1 | awk '{print $1}') -p Type --value 2>/dev/null")
        .output()
    {
        let session_type = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !session_type.is_empty() && session_type != "tty" {
            return Some(session_type);
        }
    }

    // Check XDG_SESSION_TYPE
    if let Ok(session_type) = std::env::var("XDG_SESSION_TYPE") {
        if !session_type.is_empty() {
            return Some(session_type);
        }
    }

    // Check for running display servers
    if let Ok(output) = Command::new("sh")
        .arg("-c")
        .arg("pgrep -x Xorg || pgrep -x Xwayland")
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if !stdout.trim().is_empty() {
                return Some("x11/wayland".to_string());
            }
        }
    }

    None
}
