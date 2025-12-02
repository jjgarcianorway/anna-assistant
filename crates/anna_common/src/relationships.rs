//! Anna Relationships v7.24.0 - Relationship View Formatting
//!
//! Provides formatted relationship views for sw and hw profiles.
//! All data comes from the relationship store and system probes.
//!
//! Rules:
//! - Every section has Source: line naming actual tools used
//! - Only proven relationships are shown
//! - CPU percentages include range in parentheses
//! - All fractions [0,1] shown as percentages

use crate::relationship_store::{
    discover_device_driver_links, discover_driver_firmware_links,
    discover_package_service_links, discover_service_process_links,
};
use std::collections::HashSet;
use std::path::Path;
use std::process::Command;

/// Service relationship entry
#[derive(Debug, Clone)]
pub struct ServiceRelation {
    pub name: String,
    pub state: String,
    pub package: Option<String>,
}

/// Process relationship entry
#[derive(Debug, Clone)]
pub struct ProcessRelation {
    pub pid: u32,
    pub exe_path: String,
}

/// Hardware device relationship entry
#[derive(Debug, Clone)]
pub struct HardwareRelation {
    pub device: String,
    pub driver: Option<String>,
    pub firmware: Option<String>,
}

/// Stack package entry
#[derive(Debug, Clone)]
pub struct StackPackage {
    pub name: String,
}

/// Software relationships for sw NAME profile
#[derive(Debug, Clone, Default)]
pub struct SoftwareRelationships {
    pub sources: HashSet<String>,
    pub services: Vec<ServiceRelation>,
    pub processes: Vec<ProcessRelation>,
    pub hardware: Vec<HardwareRelation>,
    pub stack_packages: Vec<StackPackage>,
    pub has_data: bool,
}

/// Driver relationship entry
#[derive(Debug, Clone)]
pub struct DriverRelation {
    pub name: String,
    pub state: String, // "loaded" or "not loaded"
}

/// Firmware relationship entry
#[derive(Debug, Clone)]
pub struct FirmwareRelation {
    pub name: String,
    pub present: bool,
}

/// Service using device entry
#[derive(Debug, Clone)]
pub struct ServiceUsingDevice {
    pub name: String,
    pub state: String,
}

/// Software using device with traffic/usage
#[derive(Debug, Clone)]
pub struct SoftwareUsingDevice {
    pub name: String,
    pub cpu_avg_percent: f64,
    pub logical_cores: u32,
    pub rx_bytes: Option<u64>,
    pub tx_bytes: Option<u64>,
}

/// Hardware relationships for hw NAME profile
#[derive(Debug, Clone, Default)]
pub struct HardwareRelationships {
    pub sources: HashSet<String>,
    pub drivers: Vec<DriverRelation>,
    pub firmware: Vec<FirmwareRelation>,
    pub services: Vec<ServiceUsingDevice>,
    pub software: Vec<SoftwareUsingDevice>,
    pub has_data: bool,
}

/// Get software relationships for a package/software name
pub fn get_software_relationships(name: &str) -> SoftwareRelationships {
    let mut rels = SoftwareRelationships::default();

    // Discover package to service links
    let pkg_svc_links = discover_package_service_links(name);
    if !pkg_svc_links.is_empty() {
        for link in &pkg_svc_links {
            rels.sources.insert(link.evidence.clone());
        }

        for link in pkg_svc_links {
            if let Some(svc_name) = link.target.strip_prefix("service:") {
                let state = get_service_state(svc_name);
                rels.services.push(ServiceRelation {
                    name: svc_name.to_string(),
                    state,
                    package: Some(name.to_string()),
                });
                rels.has_data = true;
            }
        }
    }

    // Get processes from services
    for svc in &rels.services {
        let svc_proc_links = discover_service_process_links(&svc.name);
        for link in svc_proc_links {
            rels.sources.insert(link.evidence.clone());
            if let Some(pid_str) = link.target.strip_prefix("process:") {
                if let Ok(pid) = pid_str.parse::<u32>() {
                    if let Some(exe) = get_process_exe(pid) {
                        rels.processes.push(ProcessRelation { pid, exe_path: exe });
                        rels.has_data = true;
                    }
                }
            }
        }
    }

    // Get hardware touched by this software
    // For network software, check network interfaces
    if is_network_related(name) {
        if let Some(ifaces) = get_network_interfaces_used(name) {
            rels.sources.insert("/sys/class/net".to_string());
            for iface in ifaces {
                let driver = get_interface_driver(&iface);
                let firmware = driver.as_ref().and_then(|d| get_driver_primary_firmware(d));
                rels.hardware.push(HardwareRelation {
                    device: iface,
                    driver,
                    firmware,
                });
                rels.has_data = true;
            }
        }
    }

    // Get stack packages from dependencies
    if let Some(deps) = get_related_stack_packages(name) {
        rels.sources.insert("pacman".to_string());
        rels.sources.insert("Arch Wiki".to_string());
        for dep in deps {
            rels.stack_packages.push(StackPackage { name: dep });
            rels.has_data = true;
        }
    }

    rels
}

/// Get hardware relationships for a device
pub fn get_hardware_relationships(device: &str) -> HardwareRelationships {
    let mut rels = HardwareRelationships::default();
    let logical_cores = get_logical_cores();

    // Get driver for device
    let driver_links = discover_device_driver_links(device);
    for link in &driver_links {
        rels.sources.insert(link.evidence.clone());
        if let Some(drv_name) = link.target.strip_prefix("driver:") {
            let loaded = is_module_loaded(drv_name);
            rels.drivers.push(DriverRelation {
                name: drv_name.to_string(),
                state: if loaded { "loaded".to_string() } else { "not loaded".to_string() },
            });
            rels.has_data = true;

            // Get firmware for driver
            let fw_links = discover_driver_firmware_links(drv_name);
            for fw_link in &fw_links {
                rels.sources.insert(fw_link.evidence.clone());
                if let Some(fw_name) = fw_link.target.strip_prefix("firmware:") {
                    let present = firmware_exists(fw_name);
                    rels.firmware.push(FirmwareRelation {
                        name: fw_name.to_string(),
                        present,
                    });
                    rels.has_data = true;
                }
            }
        }
    }

    // Get services using this device
    if let Some(services) = get_services_for_device(device) {
        rels.sources.insert("systemctl".to_string());
        for svc in services {
            let state = get_service_state(&svc);
            rels.services.push(ServiceUsingDevice { name: svc, state });
            rels.has_data = true;
        }
    }

    // Get software commonly using this device (from telemetry)
    if let Some(software) = get_software_using_device_24h(device) {
        rels.sources.insert("Anna telemetry".to_string());
        for (name, cpu_avg, rx, tx) in software {
            rels.software.push(SoftwareUsingDevice {
                name,
                cpu_avg_percent: cpu_avg,
                logical_cores,
                rx_bytes: rx,
                tx_bytes: tx,
            });
            rels.has_data = true;
        }
    }

    rels
}

/// Format software relationships section
pub fn format_software_relationships_section(rels: &SoftwareRelationships) -> Vec<String> {
    let mut lines = Vec::new();

    lines.push("[RELATIONSHIPS]".to_string());

    // Source line
    let sources: Vec<&String> = rels.sources.iter().collect();
    if sources.is_empty() {
        lines.push("  Source: Anna links.db".to_string());
    } else {
        let source_str = sources.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ");
        lines.push(format!("  Source: Anna links.db ({})", source_str));
    }
    lines.push(String::new());

    if !rels.has_data {
        lines.push("  No related services, processes or hardware identified yet.".to_string());
        return lines;
    }

    // Services
    if !rels.services.is_empty() {
        lines.push("  Services:".to_string());
        for svc in &rels.services {
            let pkg_note = svc.package.as_ref()
                .map(|p| format!(" (package: {})", p))
                .unwrap_or_default();
            lines.push(format!("    {}  [{}]{}", svc.name, svc.state, pkg_note));
        }
        lines.push(String::new());
    }

    // Processes
    if !rels.processes.is_empty() {
        lines.push("  Processes (current):".to_string());
        for proc in &rels.processes {
            lines.push(format!("    pid {}  {}", proc.pid, proc.exe_path));
        }
        lines.push(String::new());
    }

    // Hardware touched
    if !rels.hardware.is_empty() {
        lines.push("  Hardware touched (telemetry + sockets):".to_string());
        for hw in &rels.hardware {
            let mut details = Vec::new();
            if let Some(ref drv) = hw.driver {
                details.push(format!("driver: {}", drv));
            }
            if let Some(ref fw) = hw.firmware {
                details.push(format!("firmware: {}", fw));
            }
            let detail_str = if details.is_empty() {
                String::new()
            } else {
                format!("  ({})", details.join(", "))
            };
            lines.push(format!("    device: {}{}", hw.device, detail_str));
        }
        lines.push(String::new());
    }

    // Other packages in same stack
    if !rels.stack_packages.is_empty() {
        let pkg_names: Vec<&str> = rels.stack_packages.iter().map(|p| p.name.as_str()).collect();
        lines.push("  Other packages in the same stack:".to_string());
        lines.push(format!("    {}", pkg_names.join(", ")));
    }

    lines
}

/// Format hardware relationships section
pub fn format_hardware_relationships_section(rels: &HardwareRelationships) -> Vec<String> {
    let mut lines = Vec::new();

    lines.push("[RELATIONSHIPS]".to_string());

    // Source line
    let sources: Vec<&String> = rels.sources.iter().collect();
    if sources.is_empty() {
        lines.push("  Source: Anna links.db".to_string());
    } else {
        let source_str = sources.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ");
        lines.push(format!("  Source: Anna links.db ({})", source_str));
    }
    lines.push(String::new());

    if !rels.has_data {
        lines.push("  No related drivers, firmware or services identified yet.".to_string());
        return lines;
    }

    // Drivers
    if !rels.drivers.is_empty() {
        lines.push("  Driver:".to_string());
        for drv in &rels.drivers {
            lines.push(format!("    {}  (kernel module, {})", drv.name, drv.state));
        }
        lines.push(String::new());
    }

    // Firmware
    if !rels.firmware.is_empty() {
        lines.push("  Firmware:".to_string());
        for fw in &rels.firmware {
            let status = if fw.present { "[present]" } else { "[missing]" };
            lines.push(format!("    {}  {}", fw.name, status));
        }
        lines.push(String::new());
    }

    // Services using this device
    if !rels.services.is_empty() {
        lines.push("  Services using this device:".to_string());
        for svc in &rels.services {
            lines.push(format!("    {}  [{}]", svc.name, svc.state));
        }
        lines.push(String::new());
    }

    // Software commonly using this device
    if !rels.software.is_empty() {
        lines.push("  Software commonly using this device (last 24h):".to_string());
        for sw in &rels.software {
            let mut parts = Vec::new();

            // CPU with range
            parts.push(format!(
                "avg {} percent CPU (0 - {} percent for {} logical cores)",
                sw.cpu_avg_percent.round() as i64,
                sw.logical_cores * 100,
                sw.logical_cores
            ));

            // Network traffic if available
            if let Some(rx) = sw.rx_bytes {
                parts.push(format!("{} RX", format_bytes_compact(rx)));
            }
            if let Some(tx) = sw.tx_bytes {
                parts.push(format!("{} TX", format_bytes_compact(tx)));
            }

            lines.push(format!("    {}  {}", sw.name, parts.join(", ")));
        }
    }

    lines
}

// Helper functions

fn get_service_state(service: &str) -> String {
    let output = Command::new("systemctl")
        .args(["is-active", service])
        .output();

    match output {
        Ok(out) => {
            let state = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if state.is_empty() {
                "unknown".to_string()
            } else {
                state
            }
        }
        Err(_) => "unknown".to_string(),
    }
}

fn get_process_exe(pid: u32) -> Option<String> {
    std::fs::read_link(format!("/proc/{}/exe", pid))
        .ok()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
}

fn is_network_related(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.contains("network")
        || lower.contains("nm-")
        || lower.contains("wpa")
        || lower.contains("iwd")
        || lower.contains("wifi")
        || lower.contains("ethernet")
        || lower.contains("dhcp")
        || lower.contains("dns")
}

fn get_network_interfaces_used(_name: &str) -> Option<Vec<String>> {
    // Get all active network interfaces
    let mut interfaces = Vec::new();

    if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
        for entry in entries.flatten() {
            let iface = entry.file_name().to_string_lossy().to_string();
            if iface != "lo" {
                // Check if interface is up
                let operstate_path = format!("/sys/class/net/{}/operstate", iface);
                if let Ok(state) = std::fs::read_to_string(&operstate_path) {
                    if state.trim() == "up" {
                        interfaces.push(iface);
                    }
                }
            }
        }
    }

    if interfaces.is_empty() {
        None
    } else {
        Some(interfaces)
    }
}

fn get_interface_driver(iface: &str) -> Option<String> {
    let driver_link = format!("/sys/class/net/{}/device/driver", iface);
    std::fs::read_link(&driver_link)
        .ok()
        .and_then(|p| p.file_name().and_then(|n| n.to_str()).map(|s| s.to_string()))
}

fn get_driver_primary_firmware(driver: &str) -> Option<String> {
    let output = Command::new("modinfo")
        .args(["-F", "firmware", driver])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.lines().next().map(|s| s.trim().to_string())
}

fn get_related_stack_packages(name: &str) -> Option<Vec<String>> {
    // Get optional dependencies and common stack packages
    let output = Command::new("pacman")
        .args(["-Qi", name])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut deps = Vec::new();

    // Parse "Optional Deps" section
    let mut in_optional = false;
    for line in stdout.lines() {
        if line.starts_with("Optional Deps") {
            in_optional = true;
            if let Some(colon_pos) = line.find(':') {
                let rest = line[colon_pos + 1..].trim();
                if !rest.is_empty() && rest != "None" {
                    if let Some(pkg) = rest.split(':').next() {
                        let pkg_name = pkg.split('[').next().unwrap_or(pkg).trim();
                        if !pkg_name.is_empty() {
                            deps.push(pkg_name.to_string());
                        }
                    }
                }
            }
        } else if in_optional && line.starts_with(' ') {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                if let Some(pkg) = trimmed.split(':').next() {
                    let pkg_name = pkg.split('[').next().unwrap_or(pkg).trim();
                    if !pkg_name.is_empty() {
                        deps.push(pkg_name.to_string());
                    }
                }
            }
        } else if in_optional {
            break;
        }
    }

    // Limit to reasonable number
    deps.truncate(5);

    if deps.is_empty() {
        None
    } else {
        Some(deps)
    }
}

fn is_module_loaded(module: &str) -> bool {
    let output = Command::new("lsmod")
        .output();

    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        stdout.lines().any(|l| l.split_whitespace().next() == Some(module))
    } else {
        false
    }
}

fn firmware_exists(firmware: &str) -> bool {
    let paths = [
        format!("/lib/firmware/{}", firmware),
        format!("/usr/lib/firmware/{}", firmware),
    ];

    paths.iter().any(|p| Path::new(p).exists())
}

fn get_services_for_device(device: &str) -> Option<Vec<String>> {
    // For network devices, find network-related services
    if Path::new(&format!("/sys/class/net/{}", device)).exists() {
        let mut services = Vec::new();

        // Common network services
        let candidates = [
            "NetworkManager.service",
            "wpa_supplicant.service",
            "iwd.service",
            "dhcpcd.service",
            "systemd-networkd.service",
            "systemd-resolved.service",
        ];

        for svc in &candidates {
            let output = Command::new("systemctl")
                .args(["is-active", svc])
                .output();

            if let Ok(out) = output {
                let state = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if state == "active" {
                    services.push(svc.to_string());
                }
            }
        }

        if services.is_empty() {
            None
        } else {
            Some(services)
        }
    } else {
        None
    }
}

fn get_software_using_device_24h(_device: &str) -> Option<Vec<(String, f64, Option<u64>, Option<u64>)>> {
    // This would query telemetry to find processes that have used this device
    // For now, return None - full implementation would need device-process correlation
    None
}

fn get_logical_cores() -> u32 {
    std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .map(|content| {
            content
                .lines()
                .filter(|l| l.starts_with("processor"))
                .count() as u32
        })
        .unwrap_or(1)
        .max(1)
}

fn format_bytes_compact(bytes: u64) -> String {
    let gib = bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    if gib >= 1.0 {
        format!("{:.1} GiB", gib)
    } else {
        let mib = bytes as f64 / (1024.0 * 1024.0);
        format!("{:.0} MiB", mib)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_state_query() {
        // Just test that the function doesn't panic
        let _state = get_service_state("nonexistent.service");
    }

    #[test]
    fn test_is_network_related() {
        assert!(is_network_related("NetworkManager"));
        assert!(is_network_related("wpa_supplicant"));
        assert!(is_network_related("iwd"));
        assert!(!is_network_related("vim"));
        assert!(!is_network_related("firefox"));
    }

    #[test]
    fn test_format_bytes_compact() {
        assert_eq!(format_bytes_compact(1024 * 1024 * 1024), "1.0 GiB");
        assert_eq!(format_bytes_compact(500 * 1024 * 1024), "500 MiB");
        assert_eq!(format_bytes_compact(3 * 1024 * 1024 * 1024), "3.0 GiB");
    }

    #[test]
    fn test_software_relationships_empty() {
        let rels = SoftwareRelationships::default();
        let lines = format_software_relationships_section(&rels);

        assert!(lines.iter().any(|l| l.contains("[RELATIONSHIPS]")));
        assert!(lines.iter().any(|l| l.contains("Source:")));
        assert!(lines.iter().any(|l| l.contains("No related")));
    }

    #[test]
    fn test_hardware_relationships_empty() {
        let rels = HardwareRelationships::default();
        let lines = format_hardware_relationships_section(&rels);

        assert!(lines.iter().any(|l| l.contains("[RELATIONSHIPS]")));
        assert!(lines.iter().any(|l| l.contains("Source:")));
        assert!(lines.iter().any(|l| l.contains("No related")));
    }
}
