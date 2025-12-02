//! Hardware Overview v7.35.1
//!
//! Aggregates all peripheral discovery into a complete hardware overview.
//! v7.35.1: Added network_interfaces for AVAILABLE QUERIES section

use std::process::Command;
use super::types::HardwareOverview;
use super::bluetooth::get_bluetooth_summary;
use super::usb::get_usb_summary;
use super::multimedia::{get_audio_summary, get_camera_summary};
use super::input::get_input_summary;
use super::thunderbolt::{get_firewire_summary, get_thunderbolt_summary};
use super::sdcard::get_sdcard_summary;

/// Get complete hardware overview
pub fn get_hardware_overview() -> HardwareOverview {
    let (cpu_sockets, cpu_logical_cores) = get_cpu_counts();
    let (gpu_discrete, gpu_integrated) = get_gpu_counts();
    let memory_gib = get_memory_gib();
    let (storage_devices, storage_names) = get_storage_info();
    let (network_wired, network_wireless, network_interfaces) = get_network_info();
    let (battery_count, ac_present) = get_power_info();

    HardwareOverview {
        cpu_sockets,
        cpu_logical_cores,
        gpu_discrete,
        gpu_integrated,
        memory_gib,
        storage_devices,
        storage_names,
        network_wired,
        network_wireless,
        network_interfaces,
        bluetooth: get_bluetooth_summary(),
        usb: get_usb_summary(),
        audio: get_audio_summary(),
        camera: get_camera_summary(),
        input: get_input_summary(),
        battery_count,
        ac_present,
        firewire: get_firewire_summary(),
        thunderbolt: get_thunderbolt_summary(),
        sdcard: get_sdcard_summary(),
    }
}

fn get_cpu_counts() -> (u32, u32) {
    let output = Command::new("lscpu").output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mut sockets = 1u32;
            let mut threads = 1u32;

            for line in stdout.lines() {
                let parts: Vec<&str> = line.splitn(2, ':').collect();
                if parts.len() == 2 {
                    let key = parts[0].trim();
                    let val = parts[1].trim();

                    match key {
                        "Socket(s)" => sockets = val.parse().unwrap_or(1),
                        "CPU(s)" => {
                            if !val.contains("NUMA") && !val.contains("On-line") {
                                threads = val.parse().unwrap_or(1);
                            }
                        }
                        _ => {}
                    }
                }
            }

            return (sockets, threads);
        }
    }

    (1, 1)
}

fn get_gpu_counts() -> (u32, u32) {
    let mut discrete = 0u32;
    let mut integrated = 0u32;

    let output = Command::new("lspci").output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);

            for line in stdout.lines() {
                if line.contains("VGA") || line.contains("3D controller") || line.contains("Display controller") {
                    let line_lower = line.to_lowercase();
                    if line_lower.contains("intel") && !line_lower.contains("arc") {
                        integrated += 1;
                    } else {
                        discrete += 1;
                    }
                }
            }
        }
    }

    (discrete, integrated)
}

fn get_memory_gib() -> f64 {
    if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                if let Some(val) = line.split_whitespace().nth(1) {
                    if let Ok(kb) = val.parse::<u64>() {
                        return kb as f64 / (1024.0 * 1024.0);
                    }
                }
            }
        }
    }
    0.0
}

fn get_storage_info() -> (u32, Vec<String>) {
    let mut devices = Vec::new();

    let output = Command::new("lsblk")
        .args(["-d", "-n", "-o", "NAME,TYPE"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);

            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 && parts[1] == "disk" {
                    devices.push(parts[0].to_string());
                }
            }
        }
    }

    (devices.len() as u32, devices)
}

/// v7.35.1: Returns (wired_count, wireless_count, interface_names)
fn get_network_info() -> (u32, u32, Vec<String>) {
    let mut wired = 0u32;
    let mut wireless = 0u32;
    let mut interfaces = Vec::new();

    let net_path = std::path::Path::new("/sys/class/net");
    if let Ok(entries) = std::fs::read_dir(net_path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name == "lo" {
                continue;
            }

            let wireless_path = entry.path().join("wireless");
            if wireless_path.exists() {
                wireless += 1;
                interfaces.push(name);
            } else if name.starts_with("e") || name.starts_with("en") {
                wired += 1;
                interfaces.push(name);
            }
        }
    }

    (wired, wireless, interfaces)
}

fn get_power_info() -> (u32, bool) {
    let mut battery_count = 0u32;
    let mut ac_present = false;

    let power_path = std::path::Path::new("/sys/class/power_supply");
    if let Ok(entries) = std::fs::read_dir(power_path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let type_path = entry.path().join("type");

            if let Ok(ptype) = std::fs::read_to_string(&type_path) {
                let ptype = ptype.trim();
                if ptype == "Battery" {
                    battery_count += 1;
                } else if ptype == "Mains" {
                    let online_path = entry.path().join("online");
                    if let Ok(online) = std::fs::read_to_string(&online_path) {
                        ac_present = online.trim() == "1";
                    }
                }
            } else if name.starts_with("BAT") {
                battery_count += 1;
            } else if name.starts_with("AC") || name.starts_with("ADP") {
                let online_path = entry.path().join("online");
                if let Ok(online) = std::fs::read_to_string(&online_path) {
                    ac_present = online.trim() == "1";
                }
            }
        }
    }

    (battery_count, ac_present)
}
