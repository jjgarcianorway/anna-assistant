//! Hardware scanning - PCI, USB, CPU, memory detection.

use anyhow::Result;
use std::process::Command;

use super::{HardwareProfile, PciDevice, UsbDevice};

/// Scan hardware using lspci, lsusb, etc.
pub fn scan_hardware() -> Result<HardwareProfile> {
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
