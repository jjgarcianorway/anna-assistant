//! Thunderbolt and FireWire Discovery v7.25.0
//!
//! Discovers Thunderbolt controllers/devices from lspci and /sys/bus/thunderbolt.
//! Discovers FireWire controllers from lspci and /sys/bus/firewire.

use std::process::Command;
use super::types::{ThunderboltController, ThunderboltDevice, ThunderboltSummary, FirewireController, FirewireSummary};
use super::usb::shorten_name;

/// Get Thunderbolt summary
pub fn get_thunderbolt_summary() -> ThunderboltSummary {
    let mut summary = ThunderboltSummary {
        controller_count: 0,
        device_count: 0,
        controllers: Vec::new(),
        devices: Vec::new(),
        source: "/sys/bus/thunderbolt, lspci".to_string(),
    };

    // Check for Thunderbolt controllers in lspci
    let output = Command::new("lspci").args(["-k"]).output();
    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mut in_tb = false;
            let mut current_addr = String::new();
            let mut current_name = String::new();

            for line in stdout.lines() {
                if line.contains("Thunderbolt") {
                    in_tb = true;
                    if let Some(idx) = line.find(' ') {
                        current_addr = line[..idx].to_string();
                        if let Some(name_idx) = line.find(": ") {
                            current_name = line[name_idx + 2..].to_string();
                        }
                    }
                } else if in_tb && line.contains("Kernel driver in use:") {
                    if let Some(drv) = line.split(':').nth(1) {
                        let gen = if current_name.contains("USB4") || current_name.contains("4") {
                            Some(4)
                        } else if current_name.contains("3") {
                            Some(3)
                        } else {
                            None
                        };

                        summary.controllers.push(ThunderboltController {
                            name: shorten_name(&current_name, 50),
                            pci_address: current_addr.clone(),
                            driver: drv.trim().to_string(),
                            generation: gen,
                        });
                    }
                    in_tb = false;
                } else if !line.starts_with('\t') && !line.starts_with(' ') {
                    in_tb = false;
                }
            }
        }
    }

    // Check /sys/bus/thunderbolt for devices
    let tb_path = std::path::Path::new("/sys/bus/thunderbolt/devices");
    if tb_path.exists() {
        if let Ok(entries) = std::fs::read_dir(tb_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();

                if name.starts_with("domain") {
                    continue;
                }

                let vendor = std::fs::read_to_string(path.join("vendor_name"))
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|_| "Unknown".to_string());

                let device_name = std::fs::read_to_string(path.join("device_name"))
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|_| name.clone());

                let authorized = std::fs::read_to_string(path.join("authorized"))
                    .map(|s| s.trim() == "1")
                    .unwrap_or(false);

                summary.devices.push(ThunderboltDevice {
                    name: device_name,
                    vendor,
                    device_type: "Thunderbolt Device".to_string(),
                    authorized,
                });
            }
        }
    }

    summary.controller_count = summary.controllers.len() as u32;
    summary.device_count = summary.devices.len() as u32;
    summary
}

/// Get FireWire summary
pub fn get_firewire_summary() -> FirewireSummary {
    let mut summary = FirewireSummary {
        controller_count: 0,
        device_count: 0,
        controllers: Vec::new(),
        source: "/sys/bus/firewire, lspci".to_string(),
    };

    // Check lspci for FireWire controllers
    let output = Command::new("lspci").args(["-k"]).output();
    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mut in_fw = false;
            let mut current_addr = String::new();
            let mut current_name = String::new();

            for line in stdout.lines() {
                if line.contains("FireWire") || line.contains("IEEE 1394") {
                    in_fw = true;
                    if let Some(idx) = line.find(' ') {
                        current_addr = line[..idx].to_string();
                        if let Some(name_idx) = line.find(": ") {
                            current_name = line[name_idx + 2..].to_string();
                        }
                    }
                } else if in_fw && line.contains("Kernel driver in use:") {
                    if let Some(drv) = line.split(':').nth(1) {
                        summary.controllers.push(FirewireController {
                            name: shorten_name(&current_name, 50),
                            pci_address: current_addr.clone(),
                            driver: drv.trim().to_string(),
                        });
                    }
                    in_fw = false;
                } else if !line.starts_with('\t') && !line.starts_with(' ') {
                    in_fw = false;
                }
            }
        }
    }

    // Count devices from /sys/bus/firewire
    let fw_path = std::path::Path::new("/sys/bus/firewire/devices");
    if fw_path.exists() {
        if let Ok(entries) = std::fs::read_dir(fw_path) {
            summary.device_count = entries.count() as u32;
        }
    }

    summary.controller_count = summary.controllers.len() as u32;
    summary
}
