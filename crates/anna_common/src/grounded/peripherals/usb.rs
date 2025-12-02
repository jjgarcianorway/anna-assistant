//! USB Discovery v7.25.0
//!
//! Discovers USB controllers and devices from lsusb and /sys.

use std::process::Command;
use super::types::{PeripheralUsbDevice, UsbController, UsbSummary};

/// Get USB summary from lsusb and /sys
pub fn get_usb_summary() -> UsbSummary {
    let mut summary = UsbSummary {
        root_hubs: 0,
        device_count: 0,
        controllers: Vec::new(),
        devices: Vec::new(),
        source: "lsusb, /sys/bus/usb/devices".to_string(),
    };

    // Get controllers from lspci
    summary.controllers = get_usb_controllers();
    summary.root_hubs = summary.controllers.len() as u32;

    // Get devices from lsusb
    summary.devices = get_usb_devices();
    summary.device_count = summary.devices.iter().filter(|d| !d.is_hub).count() as u32;

    summary
}

fn get_usb_controllers() -> Vec<UsbController> {
    let mut controllers = Vec::new();

    let output = Command::new("lspci").args(["-k"]).output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mut current_addr = String::new();
            let mut current_name = String::new();
            let mut in_usb = false;

            for line in stdout.lines() {
                if line.contains("USB controller") || line.contains("USB host") {
                    in_usb = true;
                    if let Some(idx) = line.find(' ') {
                        current_addr = line[..idx].to_string();
                        if let Some(name_idx) = line.find(": ") {
                            current_name = line[name_idx + 2..].to_string();
                        }
                    }
                } else if in_usb && line.contains("Kernel driver in use:") {
                    if let Some(drv) = line.split(':').nth(1) {
                        let driver = drv.trim().to_string();
                        let usb_version = if current_name.contains("xHCI") || current_name.contains("USB 3") {
                            "USB 3.x".to_string()
                        } else if current_name.contains("EHCI") || current_name.contains("USB 2") {
                            "USB 2.0".to_string()
                        } else if current_name.contains("UHCI") || current_name.contains("OHCI") {
                            "USB 1.x".to_string()
                        } else {
                            "USB".to_string()
                        };

                        controllers.push(UsbController {
                            pci_address: current_addr.clone(),
                            name: shorten_name(&current_name, 50),
                            driver,
                            usb_version,
                        });
                    }
                    in_usb = false;
                } else if !line.starts_with('\t') && !line.starts_with(' ') {
                    in_usb = false;
                }
            }
        }
    }

    controllers
}

fn get_usb_devices() -> Vec<PeripheralUsbDevice> {
    let mut devices = Vec::new();

    let output = Command::new("lsusb").output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);

            for line in stdout.lines() {
                if let Some(dev) = parse_lsusb_line(line) {
                    devices.push(dev);
                }
            }
        }
    }

    enrich_usb_devices_from_tree(&mut devices);
    enrich_usb_devices_from_sys(&mut devices);

    devices
}

fn parse_lsusb_line(line: &str) -> Option<PeripheralUsbDevice> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 6 {
        return None;
    }

    let bus: u32 = parts.get(1)?.parse().ok()?;
    let device: u32 = parts.get(3)?.trim_end_matches(':').parse().ok()?;

    let id_part = parts.get(5)?;
    let id_parts: Vec<&str> = id_part.split(':').collect();
    let vendor_id = id_parts.get(0)?.to_string();
    let product_id = id_parts.get(1)?.to_string();

    let name_start = line.find(&format!("{} ", id_part))? + id_part.len() + 1;
    let full_name = if name_start < line.len() {
        line[name_start..].to_string()
    } else {
        "Unknown".to_string()
    };

    let (vendor_name, product_name) = if let Some(idx) = full_name.find(", ") {
        (full_name[..idx].to_string(), full_name[idx + 2..].to_string())
    } else {
        ("Unknown".to_string(), full_name)
    };

    let is_hub = product_name.to_lowercase().contains("hub") ||
                 product_name.to_lowercase().contains("root hub");

    Some(PeripheralUsbDevice {
        bus,
        device,
        vendor_id,
        product_id,
        vendor_name,
        product_name,
        speed: String::new(),
        driver: None,
        path: format!("{}-{}", bus, device),
        device_class: String::new(),
        power_ma: None,
        is_hub,
    })
}

fn enrich_usb_devices_from_tree(devices: &mut [PeripheralUsbDevice]) {
    let output = Command::new("lsusb").args(["-t"]).output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);

            for line in stdout.lines() {
                if let Some((bus, dev, speed, driver)) = parse_lsusb_tree_line(line) {
                    for device in devices.iter_mut() {
                        if device.bus == bus && device.device == dev {
                            device.speed = speed.clone();
                            if device.driver.is_none() && !driver.is_empty() {
                                device.driver = Some(driver.clone());
                            }
                            break;
                        }
                    }
                }
            }
        }
    }
}

fn parse_lsusb_tree_line(line: &str) -> Option<(u32, u32, String, String)> {
    if !line.contains("Bus") || !line.contains("Dev") {
        return None;
    }

    let bus = line.split("Bus ").nth(1)?.split('.').next()?.trim().parse::<u32>().ok()?;
    let dev = line.split("Dev ").nth(1)?.split(',').next()?.trim().parse::<u32>().ok()?;

    let speed = if line.contains("5000M") || line.contains("10000M") || line.contains("20000M") {
        "SuperSpeed".to_string()
    } else if line.contains("480M") {
        "High Speed".to_string()
    } else if line.contains("12M") {
        "Full Speed".to_string()
    } else if line.contains("1.5M") {
        "Low Speed".to_string()
    } else {
        String::new()
    };

    let driver = line.split("Driver=")
        .nth(1)
        .map(|s| s.split('/').next().unwrap_or("").split(',').next().unwrap_or("").to_string())
        .unwrap_or_default();

    Some((bus, dev, speed, driver))
}

fn enrich_usb_devices_from_sys(devices: &mut [PeripheralUsbDevice]) {
    let usb_devices_path = std::path::Path::new("/sys/bus/usb/devices");
    if !usb_devices_path.exists() {
        return;
    }

    if let Ok(entries) = std::fs::read_dir(usb_devices_path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.contains(':') {
                continue;
            }

            let path = entry.path();
            let vendor = std::fs::read_to_string(path.join("idVendor"))
                .map(|s| s.trim().to_string())
                .unwrap_or_default();
            let product = std::fs::read_to_string(path.join("idProduct"))
                .map(|s| s.trim().to_string())
                .unwrap_or_default();

            for device in devices.iter_mut() {
                if device.vendor_id == vendor && device.product_id == product {
                    if device.speed.is_empty() {
                        if let Ok(speed) = std::fs::read_to_string(path.join("speed")) {
                            device.speed = format_usb_speed(speed.trim());
                        }
                    }

                    if let Ok(power) = std::fs::read_to_string(path.join("bMaxPower")) {
                        let power_str = power.trim().trim_end_matches("mA");
                        device.power_ma = power_str.parse().ok();
                    }

                    if let Ok(class) = std::fs::read_to_string(path.join("bDeviceClass")) {
                        device.device_class = format_usb_class(class.trim());
                    }

                    break;
                }
            }
        }
    }
}

pub fn format_usb_speed(speed: &str) -> String {
    match speed {
        "1.5" => "1.5 Mbit/s (Low Speed)".to_string(),
        "12" => "12 Mbit/s (Full Speed)".to_string(),
        "480" => "480 Mbit/s (High Speed)".to_string(),
        "5000" => "5 Gbit/s (SuperSpeed)".to_string(),
        "10000" => "10 Gbit/s (SuperSpeed+)".to_string(),
        "20000" => "20 Gbit/s (USB4)".to_string(),
        _ => format!("{} Mbit/s", speed),
    }
}

pub fn format_usb_class(class: &str) -> String {
    match class {
        "00" => "Per Interface".to_string(),
        "01" => "Audio".to_string(),
        "02" => "Communications".to_string(),
        "03" => "HID".to_string(),
        "05" => "Physical".to_string(),
        "06" => "Image".to_string(),
        "07" => "Printer".to_string(),
        "08" => "Mass Storage".to_string(),
        "09" => "Hub".to_string(),
        "0a" => "CDC Data".to_string(),
        "0b" => "Smart Card".to_string(),
        "0e" => "Video".to_string(),
        "e0" => "Wireless".to_string(),
        "ef" => "Miscellaneous".to_string(),
        "fe" => "Application Specific".to_string(),
        "ff" => "Vendor Specific".to_string(),
        _ => format!("Class {}", class),
    }
}

pub fn shorten_name(name: &str, max_len: usize) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usb_speed_format() {
        assert!(format_usb_speed("480").contains("High Speed"));
        assert!(format_usb_speed("5000").contains("SuperSpeed"));
    }

    #[test]
    fn test_usb_class_format() {
        assert_eq!(format_usb_class("03"), "HID");
        assert_eq!(format_usb_class("08"), "Mass Storage");
        assert_eq!(format_usb_class("09"), "Hub");
    }
}
