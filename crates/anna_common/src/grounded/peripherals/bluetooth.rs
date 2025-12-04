//! Bluetooth Discovery v7.25.0
//!
//! Discovers Bluetooth adapters from /sys/class/bluetooth and rfkill.

use super::types::{BluetoothAdapter, BluetoothState, BluetoothSummary};
use std::process::Command;

/// Get Bluetooth summary
pub fn get_bluetooth_summary() -> BluetoothSummary {
    let mut summary = BluetoothSummary {
        adapter_count: 0,
        adapters: Vec::new(),
        source: "rfkill, /sys/class/bluetooth".to_string(),
    };

    let bt_path = std::path::Path::new("/sys/class/bluetooth");
    if !bt_path.exists() {
        return summary;
    }

    if let Ok(entries) = std::fs::read_dir(bt_path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.starts_with("hci") {
                continue;
            }

            let adapter_path = entry.path();
            let mut adapter = BluetoothAdapter {
                name: name.clone(),
                address: String::new(),
                manufacturer: String::new(),
                driver: String::new(),
                state: BluetoothState::Unknown,
                powered: false,
                discoverable: false,
            };

            // Read address
            if let Ok(addr) = std::fs::read_to_string(adapter_path.join("address")) {
                adapter.address = addr.trim().to_string();
            }

            // Read manufacturer from device symlink
            let device_path = adapter_path.join("device");
            if device_path.exists() {
                let driver_path = device_path.join("driver");
                if let Ok(link) = std::fs::read_link(&driver_path) {
                    adapter.driver = link
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                }

                adapter.manufacturer = get_bluetooth_manufacturer(&device_path);
            }

            adapter.state = get_bluetooth_state(&name);
            adapter.powered = adapter.state == BluetoothState::Up;

            summary.adapters.push(adapter);
        }
    }

    summary.adapter_count = summary.adapters.len() as u32;
    summary
}

fn get_bluetooth_manufacturer(device_path: &std::path::Path) -> String {
    // Try USB product name
    let product_path = device_path.join("product");
    if let Ok(product) = std::fs::read_to_string(&product_path) {
        return product.trim().to_string();
    }

    // Try modalias and parse vendor
    let modalias_path = device_path.join("modalias");
    if let Ok(modalias) = std::fs::read_to_string(&modalias_path) {
        let modalias = modalias.trim();
        if modalias.contains("usb:") {
            if modalias.contains("v8087") {
                return "Intel".to_string();
            } else if modalias.contains("v0CF3") {
                return "Qualcomm/Atheros".to_string();
            } else if modalias.contains("v0A5C") {
                return "Broadcom".to_string();
            } else if modalias.contains("v0BDA") {
                return "Realtek".to_string();
            }
        }
    }

    "Unknown".to_string()
}

fn get_bluetooth_state(adapter_name: &str) -> BluetoothState {
    // Check rfkill
    let output = Command::new("rfkill").args(["list", "bluetooth"]).output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);

            for line in stdout.lines() {
                if line.contains("Soft blocked: yes") || line.contains("Hard blocked: yes") {
                    return BluetoothState::Blocked;
                }
            }
        }
    }

    // Check /sys operstate equivalent
    let state_path = format!("/sys/class/bluetooth/{}/power/runtime_status", adapter_name);
    if let Ok(state) = std::fs::read_to_string(&state_path) {
        if state.trim() == "active" {
            return BluetoothState::Up;
        }
    }

    BluetoothState::Up
}
