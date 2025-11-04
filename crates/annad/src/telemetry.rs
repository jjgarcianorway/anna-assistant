//! System telemetry collection
//!
//! Gathers hardware, software, and system state information.

use anna_common::{SystemFacts, StorageDevice};
use anyhow::Result;
use chrono::Utc;
use sysinfo::System;
use std::process::Command;

/// Collect current system facts
pub async fn collect_facts() -> Result<SystemFacts> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let hostname = get_hostname()?;
    let kernel = get_kernel_version()?;
    let cpu_model = get_cpu_model(&sys);
    let cpu_cores = sys.cpus().len();
    let total_memory_gb = sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
    let gpu_vendor = detect_gpu();
    let storage_devices = get_storage_devices()?;
    let installed_packages = count_packages()?;
    let orphan_packages = find_orphan_packages()?;
    let network_interfaces = get_network_interfaces();

    Ok(SystemFacts {
        timestamp: Utc::now(),
        hostname,
        kernel,
        cpu_model,
        cpu_cores,
        total_memory_gb,
        gpu_vendor,
        storage_devices,
        installed_packages,
        orphan_packages,
        network_interfaces,
    })
}

fn get_hostname() -> Result<String> {
    let output = Command::new("hostname").output()?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn get_kernel_version() -> Result<String> {
    let output = Command::new("uname").arg("-r").output()?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn get_cpu_model(sys: &System) -> String {
    sys.cpus()
        .first()
        .map(|cpu| cpu.brand().to_string())
        .unwrap_or_else(|| "Unknown".to_string())
}

fn detect_gpu() -> Option<String> {
    // Try lspci to detect GPU
    let output = Command::new("lspci")
        .output()
        .ok()?;
    let lspci_output = String::from_utf8_lossy(&output.stdout);

    if lspci_output.contains("NVIDIA") {
        Some("NVIDIA".to_string())
    } else if lspci_output.contains("AMD") || lspci_output.contains("Radeon") {
        Some("AMD".to_string())
    } else if lspci_output.contains("Intel") {
        Some("Intel".to_string())
    } else {
        None
    }
}

fn get_storage_devices() -> Result<Vec<StorageDevice>> {
    // Parse df output for mounted filesystems
    let output = Command::new("df")
        .arg("-h")
        .arg("--output=source,fstype,size,used,target")
        .output()?;

    let df_output = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();

    for line in df_output.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 5 {
            let name = parts[0].to_string();
            let filesystem = parts[1].to_string();
            let size_gb = parse_size(parts[2]);
            let used_gb = parse_size(parts[3]);
            let mount_point = parts[4].to_string();

            // Filter out tmpfs and other virtual filesystems
            if !filesystem.starts_with("tmp") && !name.starts_with("/dev/loop") {
                devices.push(StorageDevice {
                    name,
                    filesystem,
                    size_gb,
                    used_gb,
                    mount_point,
                });
            }
        }
    }

    Ok(devices)
}

fn parse_size(size_str: &str) -> f64 {
    // Parse size string like "100G" or "500M"
    let size_str = size_str.trim_end_matches(|c: char| !c.is_numeric() && c != '.');
    size_str.parse().unwrap_or(0.0)
}

fn count_packages() -> Result<usize> {
    // Count installed packages on Arch Linux
    let output = Command::new("pacman")
        .arg("-Q")
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.lines().count())
}

fn find_orphan_packages() -> Result<Vec<String>> {
    // Find orphaned packages (installed as dependencies but no longer needed)
    let output = Command::new("pacman")
        .arg("-Qdtq")
        .output()?;

    let orphans = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.to_string())
        .collect();

    Ok(orphans)
}

fn get_network_interfaces() -> Vec<String> {
    // Get network interfaces from ip command
    let output = Command::new("ip")
        .args(&["link", "show"])
        .output();

    if let Ok(output) = output {
        let ip_output = String::from_utf8_lossy(&output.stdout);
        ip_output
            .lines()
            .filter_map(|line| {
                if line.contains(": <") {
                    let parts: Vec<&str> = line.split(':').collect();
                    parts.get(1).map(|s| s.trim().to_string())
                } else {
                    None
                }
            })
            .collect()
    } else {
        vec![]
    }
}
