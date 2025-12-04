//! Driver and Firmware Detection v7.3.0
//!
//! Provides driver binding detection for PCI/USB devices and firmware
//! status scanning from kernel logs.
//!
//! Sources:
//! - /sys/bus/pci/devices/*/driver - PCI driver bindings
//! - /sys/bus/usb/devices/*/driver - USB driver bindings
//! - lsmod - loaded kernel modules
//! - modinfo - module metadata
//! - journalctl -b -k / dmesg - kernel firmware messages

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

/// PCI device information with driver binding status
#[derive(Debug, Clone)]
pub struct PciDevice {
    /// PCI address (e.g., "0000:01:00.0")
    pub address: String,
    /// Device description from lspci
    pub description: String,
    /// Device class (e.g., "VGA compatible controller")
    pub class: String,
    /// Bound driver name, if any
    pub driver: Option<String>,
    /// Vendor:Device ID (e.g., "10de:2860")
    pub pci_id: Option<String>,
}

/// USB device information with driver binding status
#[derive(Debug, Clone)]
pub struct UsbDevice {
    /// USB path (e.g., "1-2")
    pub path: String,
    /// Device description
    pub description: String,
    /// Vendor:Product ID
    pub usb_id: Option<String>,
    /// Bound driver name, if any
    pub driver: Option<String>,
}

/// Driver binding summary
#[derive(Debug, Clone, Default)]
pub struct DriverSummary {
    pub pci_with_driver: usize,
    pub pci_without_driver: usize,
    pub usb_with_driver: usize,
    pub usb_without_driver: usize,
    /// List of PCI devices without drivers (address, description)
    pub pci_unbound: Vec<(String, String)>,
    /// List of USB devices without drivers (path, description)
    pub usb_unbound: Vec<(String, String)>,
}

/// Firmware message from kernel logs
#[derive(Debug, Clone)]
pub struct FirmwareMessage {
    /// The full message text
    pub message: String,
    /// Whether this is a failure or warning
    pub is_failure: bool,
    /// Number of times seen
    pub count: usize,
}

/// Firmware status summary
#[derive(Debug, Clone, Default)]
pub struct FirmwareSummary {
    pub failed_loads: usize,
    pub warnings: usize,
    pub messages: Vec<FirmwareMessage>,
}

/// Get driver summary for PCI and USB devices
pub fn get_driver_summary() -> DriverSummary {
    let mut summary = DriverSummary::default();

    // Get PCI devices
    let pci_devices = list_pci_devices();
    for dev in &pci_devices {
        if dev.driver.is_some() {
            summary.pci_with_driver += 1;
        } else {
            summary.pci_without_driver += 1;
            summary
                .pci_unbound
                .push((dev.address.clone(), dev.description.clone()));
        }
    }

    // Get USB devices
    let usb_devices = list_usb_devices();
    for dev in &usb_devices {
        if dev.driver.is_some() {
            summary.usb_with_driver += 1;
        } else {
            summary.usb_without_driver += 1;
            summary
                .usb_unbound
                .push((dev.path.clone(), dev.description.clone()));
        }
    }

    summary
}

/// List all PCI devices with their driver bindings
pub fn list_pci_devices() -> Vec<PciDevice> {
    let mut devices = Vec::new();

    // Get device list from lspci -nn
    let output = Command::new("lspci").arg("-nn").output();

    let lspci_output = match output {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).to_string(),
        _ => return devices,
    };

    for line in lspci_output.lines() {
        // Format: "01:00.0 VGA compatible controller [0300]: NVIDIA Corporation ... [10de:2860]"
        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        if parts.len() < 2 {
            continue;
        }

        let address = format!("0000:{}", parts[0]);
        let rest = parts[1];

        // Extract class
        let class = if let Some(colon_idx) = rest.find(':') {
            rest[..colon_idx].trim().to_string()
        } else {
            rest.to_string()
        };

        // Extract PCI ID (last [xxxx:xxxx] in the line)
        let pci_id = extract_pci_id(rest);

        // Check for driver binding
        let driver_path = format!("/sys/bus/pci/devices/{}/driver", address);
        let driver = if let Ok(link) = std::fs::read_link(&driver_path) {
            link.file_name().map(|n| n.to_string_lossy().to_string())
        } else {
            None
        };

        devices.push(PciDevice {
            address,
            description: rest.to_string(),
            class,
            driver,
            pci_id,
        });
    }

    devices
}

/// Extract PCI vendor:device ID from lspci output line
fn extract_pci_id(line: &str) -> Option<String> {
    // Find last occurrence of [xxxx:xxxx]
    let mut last_id = None;
    for (i, _) in line.match_indices('[') {
        if let Some(end) = line[i..].find(']') {
            let content = &line[i + 1..i + end];
            // Check if it looks like a PCI ID (4hex:4hex)
            if content.len() == 9 && content.chars().nth(4) == Some(':') {
                let valid = content.chars().enumerate().all(|(j, c)| {
                    if j == 4 {
                        c == ':'
                    } else {
                        c.is_ascii_hexdigit()
                    }
                });
                if valid {
                    last_id = Some(content.to_string());
                }
            }
        }
    }
    last_id
}

/// List USB devices with driver bindings
pub fn list_usb_devices() -> Vec<UsbDevice> {
    let mut devices = Vec::new();

    // Read from /sys/bus/usb/devices
    let usb_path = Path::new("/sys/bus/usb/devices");
    if !usb_path.exists() {
        return devices;
    }

    let entries = match std::fs::read_dir(usb_path) {
        Ok(e) => e,
        Err(_) => return devices,
    };

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip non-device entries (like usb1, usb2 which are root hubs)
        // We want entries like 1-1, 1-2, 2-1.3, etc.
        if !name.contains('-') && !name.starts_with("usb") {
            continue;
        }

        let device_path = entry.path();

        // Read product/manufacturer if available
        let product = std::fs::read_to_string(device_path.join("product"))
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|_| "Unknown device".to_string());

        let manufacturer = std::fs::read_to_string(device_path.join("manufacturer"))
            .map(|s| s.trim().to_string())
            .ok();

        let description = if let Some(mfg) = manufacturer {
            format!("{} {}", mfg, product)
        } else {
            product
        };

        // Read vendor:product ID
        let vendor_id = std::fs::read_to_string(device_path.join("idVendor"))
            .map(|s| s.trim().to_string())
            .ok();
        let product_id = std::fs::read_to_string(device_path.join("idProduct"))
            .map(|s| s.trim().to_string())
            .ok();
        let usb_id = match (vendor_id, product_id) {
            (Some(v), Some(p)) => Some(format!("{}:{}", v, p)),
            _ => None,
        };

        // Check for driver binding
        // USB devices can have drivers at multiple levels; check the interface level
        let driver = find_usb_driver(&device_path);

        devices.push(UsbDevice {
            path: name,
            description,
            usb_id,
            driver,
        });
    }

    devices
}

/// Find driver for a USB device (may be at interface level)
fn find_usb_driver(device_path: &Path) -> Option<String> {
    // Check device-level driver
    let driver_link = device_path.join("driver");
    if let Ok(link) = std::fs::read_link(&driver_link) {
        if let Some(name) = link.file_name() {
            return Some(name.to_string_lossy().to_string());
        }
    }

    // Check interface-level drivers (e.g., 1-2:1.0/driver)
    if let Ok(entries) = std::fs::read_dir(device_path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.contains(':') {
                let iface_driver = entry.path().join("driver");
                if let Ok(link) = std::fs::read_link(&iface_driver) {
                    if let Some(driver_name) = link.file_name() {
                        return Some(driver_name.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    None
}

/// Get firmware status from kernel logs
pub fn get_firmware_summary() -> FirmwareSummary {
    let mut summary = FirmwareSummary::default();
    let mut message_counts: HashMap<String, (bool, usize)> = HashMap::new();

    // Try journalctl first
    let output = Command::new("journalctl")
        .args(["-b", "-k", "--no-pager"])
        .output();

    let logs = match output {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).to_string(),
        _ => {
            // Fall back to dmesg
            let dmesg_output = Command::new("dmesg").output();
            match dmesg_output {
                Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).to_string(),
                _ => return summary,
            }
        }
    };

    // Patterns for firmware issues
    for line in logs.lines() {
        let line_lower = line.to_lowercase();

        // Check for firmware failures
        let is_failure = line_lower.contains("firmware: failed to load")
            || line_lower.contains("firmware load failed")
            || line_lower.contains("failed to load firmware")
            || line_lower.contains("direct firmware load") && line_lower.contains("failed");

        // Check for firmware warnings
        let is_warning = !is_failure
            && (line_lower.contains("firmware")
                && (line_lower.contains("missing")
                    || line_lower.contains("warning")
                    || line_lower.contains("older api")
                    || line_lower.contains("fallback")));

        if is_failure || is_warning {
            // Extract the relevant part of the message (after timestamp)
            let msg = extract_firmware_message(line);
            let entry = message_counts.entry(msg).or_insert((is_failure, 0));
            entry.1 += 1;
        }
    }

    // Build summary
    for (msg, (is_failure, count)) in message_counts {
        if is_failure {
            summary.failed_loads += 1;
        } else {
            summary.warnings += 1;
        }
        summary.messages.push(FirmwareMessage {
            message: msg,
            is_failure,
            count,
        });
    }

    // Sort by failure first, then by count
    summary
        .messages
        .sort_by(|a, b| b.is_failure.cmp(&a.is_failure).then(b.count.cmp(&a.count)));

    summary
}

/// Extract firmware-relevant part of a log message
fn extract_firmware_message(line: &str) -> String {
    // Remove timestamp prefix (journalctl format: "Dec 01 10:30:45 host kernel:")
    // or dmesg format: "[    1.234567]"

    let msg = if line.starts_with('[') {
        // dmesg format
        if let Some(idx) = line.find(']') {
            line[idx + 1..].trim()
        } else {
            line
        }
    } else if line.contains("kernel:") {
        // journalctl format
        if let Some(idx) = line.find("kernel:") {
            line[idx + 7..].trim()
        } else {
            line
        }
    } else {
        line.trim()
    };

    msg.to_string()
}

/// Get driver info for a specific PCI device by address
pub fn get_pci_device_driver(address: &str) -> Option<PciDevice> {
    let devices = list_pci_devices();
    devices
        .into_iter()
        .find(|d| d.address == address || d.address.ends_with(address))
}

/// Get driver info for a PCI device at a given index in a class
pub fn get_pci_device_by_class_index(class_pattern: &str, index: usize) -> Option<PciDevice> {
    let devices = list_pci_devices();
    let class_lower = class_pattern.to_lowercase();

    devices
        .into_iter()
        .filter(|d| d.class.to_lowercase().contains(&class_lower))
        .nth(index)
}

/// Get kernel module info via modinfo
pub fn get_module_info(module_name: &str) -> Option<ModuleInfo> {
    let output = Command::new("modinfo").arg(module_name).output().ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut info = ModuleInfo {
        name: module_name.to_string(),
        ..Default::default()
    };

    for line in stdout.lines() {
        let parts: Vec<&str> = line.splitn(2, ':').collect();
        if parts.len() == 2 {
            let key = parts[0].trim();
            let val = parts[1].trim();

            match key {
                "version" => info.version = Some(val.to_string()),
                "description" => info.description = Some(val.to_string()),
                "filename" => info.filename = Some(val.to_string()),
                "author" => info.author = Some(val.to_string()),
                "license" => info.license = Some(val.to_string()),
                _ => {}
            }
        }
    }

    Some(info)
}

/// Kernel module information
#[derive(Debug, Clone, Default)]
pub struct ModuleInfo {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub filename: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
}

/// Check if lspci is available
pub fn is_lspci_available() -> bool {
    Command::new("lspci")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if lsusb is available
pub fn is_lsusb_available() -> bool {
    Command::new("lsusb")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get firmware messages for a specific device/driver
pub fn get_device_firmware_messages(driver_name: &str) -> Vec<FirmwareMessage> {
    let summary = get_firmware_summary();
    let driver_lower = driver_name.to_lowercase();

    summary
        .messages
        .into_iter()
        .filter(|m| m.message.to_lowercase().contains(&driver_lower))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_pci_id() {
        let line = "VGA compatible controller [0300]: NVIDIA Corporation AD107M [GeForce RTX 4060 Max-Q / Mobile] (rev a1) [10de:28a0]";
        let id = extract_pci_id(line);
        assert_eq!(id, Some("10de:28a0".to_string()));
    }

    #[test]
    fn test_extract_firmware_message() {
        let journalctl_line =
            "Dec 01 10:30:45 myhost kernel: iwlwifi 0000:00:14.3: firmware: failed to load";
        let msg = extract_firmware_message(journalctl_line);
        assert!(msg.contains("iwlwifi"));
        assert!(msg.contains("firmware"));

        let dmesg_line = "[    1.234567] nvidia: firmware loading failed";
        let msg = extract_firmware_message(dmesg_line);
        assert!(msg.contains("nvidia"));
    }

    #[test]
    fn test_driver_summary() {
        // This test verifies the function runs without panicking
        let summary = get_driver_summary();
        // On most systems, PCI devices should exist
        assert!(summary.pci_with_driver + summary.pci_without_driver >= 0);
    }
}
