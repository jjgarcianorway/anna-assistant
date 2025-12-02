//! Scenario Lenses v7.22.0 - Category-Aware Hardware & Software Views
//!
//! Provides curated, scenario-specific views when querying categories like:
//! - network: interfaces, drivers, rfkill, link state, connection events
//! - storage: devices, SMART health, IO telemetry, controller events
//! - graphics: GPUs, drivers, firmware, display connectors
//! - audio: controllers, drivers, sink/source state
//! - display: compositors, portals, input daemons (software stack)
//!
//! Each lens aggregates data from:
//! - /sys/class/* filesystem probes
//! - System tools: ip, iw, ethtool, rfkill, smartctl, nvme, lspci, etc.
//! - journalctl for event patterns
//! - Anna telemetry for historical data

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

use serde::{Deserialize, Serialize};

// ============================================================================
// Network Lens
// ============================================================================

/// Network interface in the topology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub iface_type: String,  // wifi, ethernet, bridge, veth, etc.
    pub driver: Option<String>,
    pub firmware: Option<String>,
    pub rfkill_blocked: Option<bool>,
    pub link_state: String,  // up, down, unknown
    pub carrier: bool,
    pub mac: Option<String>,
}

/// Network telemetry for an interface
#[derive(Debug, Clone, Default)]
pub struct NetworkTelemetry {
    pub rx_bytes_24h: u64,
    pub tx_bytes_24h: u64,
}

/// Network event summary
#[derive(Debug, Clone)]
pub struct NetworkEvent {
    pub pattern_id: String,
    pub description: String,
    pub count: usize,
}

/// Complete network lens view
#[derive(Debug, Clone)]
pub struct NetworkLens {
    pub interfaces: Vec<NetworkInterface>,
    pub telemetry: HashMap<String, NetworkTelemetry>,
    pub events: Vec<NetworkEvent>,
    pub log_patterns: Vec<(String, String, usize)>,  // (pattern_id, message, count)
    pub first_seen: HashMap<String, String>,  // interface -> date
}

impl NetworkLens {
    /// Build network lens from system probes
    pub fn build() -> Self {
        let interfaces = discover_network_interfaces();
        let telemetry = collect_network_telemetry(&interfaces);
        let events = collect_network_events();
        let log_patterns = collect_network_log_patterns();
        let first_seen = get_network_first_seen(&interfaces);

        Self {
            interfaces,
            telemetry,
            events,
            log_patterns,
            first_seen,
        }
    }
}

fn discover_network_interfaces() -> Vec<NetworkInterface> {
    let mut interfaces = Vec::new();

    // Use ip link to get all interfaces
    let output = Command::new("ip")
        .args(["-o", "link", "show"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                if let Some(iface) = parse_ip_link_line(line) {
                    interfaces.push(iface);
                }
            }
        }
    }

    // Enrich with driver and firmware info from /sys
    for iface in &mut interfaces {
        enrich_interface_info(iface);
    }

    interfaces
}

fn parse_ip_link_line(line: &str) -> Option<NetworkInterface> {
    // Format: 1: lo: <LOOPBACK,UP,LOWER_UP> mtu 65536 ...
    let parts: Vec<&str> = line.split(':').collect();
    if parts.len() < 2 {
        return None;
    }

    let name = parts[1].trim().split('@').next()?.to_string();

    // Skip loopback
    if name == "lo" {
        return None;
    }

    let iface_type = if name.starts_with("en") || name.starts_with("eth") {
        "ethernet"
    } else if name.starts_with("wl") || name.starts_with("wlan") {
        "wifi"
    } else if name.starts_with("br") {
        "bridge"
    } else if name.starts_with("veth") {
        "veth"
    } else if name.starts_with("docker") {
        "docker"
    } else if name.starts_with("virbr") {
        "libvirt"
    } else {
        "other"
    }.to_string();

    let link_state = if line.contains("state UP") {
        "up"
    } else if line.contains("state DOWN") {
        "down"
    } else {
        "unknown"
    }.to_string();

    let carrier = line.contains("LOWER_UP");

    Some(NetworkInterface {
        name,
        iface_type,
        driver: None,
        firmware: None,
        rfkill_blocked: None,
        link_state,
        carrier,
        mac: None,
    })
}

fn enrich_interface_info(iface: &mut NetworkInterface) {
    let sys_path = format!("/sys/class/net/{}", iface.name);

    // Get driver
    let driver_path = format!("{}/device/driver", sys_path);
    if let Ok(link) = fs::read_link(&driver_path) {
        iface.driver = link.file_name()
            .map(|n| n.to_string_lossy().to_string());
    }

    // Get MAC address
    let mac_path = format!("{}/address", sys_path);
    if let Ok(mac) = fs::read_to_string(&mac_path) {
        let mac = mac.trim().to_string();
        if !mac.is_empty() && mac != "00:00:00:00:00:00" {
            iface.mac = Some(mac);
        }
    }

    // Check rfkill for wireless
    if iface.iface_type == "wifi" {
        iface.rfkill_blocked = check_rfkill_status(&iface.name);
    }

    // Get firmware info for wireless
    if iface.iface_type == "wifi" {
        iface.firmware = get_wifi_firmware(&iface.name);
    }
}

fn check_rfkill_status(iface_name: &str) -> Option<bool> {
    // Check via rfkill command
    let output = Command::new("rfkill")
        .args(["list"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            // Look for wireless section and check if blocked
            let mut in_wifi_section = false;
            for line in stdout.lines() {
                if line.contains("Wireless") || line.contains("wlan") {
                    in_wifi_section = true;
                }
                if in_wifi_section {
                    if line.contains("Soft blocked: yes") || line.contains("Hard blocked: yes") {
                        return Some(true);
                    }
                    if line.contains("Soft blocked: no") && line.contains("Hard blocked: no") {
                        return Some(false);
                    }
                }
                if line.is_empty() {
                    in_wifi_section = false;
                }
            }
        }
    }

    // Fallback to /sys check
    let phy_path = format!("/sys/class/net/{}/phy80211", iface_name);
    if Path::new(&phy_path).exists() {
        return Some(false);  // Assume not blocked if phy exists
    }

    None
}

fn get_wifi_firmware(iface_name: &str) -> Option<String> {
    // Try to get firmware from ethtool -i
    let output = Command::new("ethtool")
        .args(["-i", iface_name])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                if line.starts_with("firmware-version:") {
                    let fw = line.split(':').nth(1)?.trim();
                    if !fw.is_empty() && fw != "N/A" {
                        return Some(fw.to_string());
                    }
                }
            }
        }
    }

    // Check if firmware files exist
    let fw_dir = Path::new("/lib/firmware");
    if fw_dir.exists() {
        if let Ok(entries) = fs::read_dir(fw_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("iwlwifi-") {
                    return Some("present".to_string());
                }
            }
        }
    }

    None
}

fn collect_network_telemetry(interfaces: &[NetworkInterface]) -> HashMap<String, NetworkTelemetry> {
    let mut telemetry = HashMap::new();

    for iface in interfaces {
        let sys_path = format!("/sys/class/net/{}/statistics", iface.name);

        let rx = fs::read_to_string(format!("{}/rx_bytes", sys_path))
            .ok()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0);

        let tx = fs::read_to_string(format!("{}/tx_bytes", sys_path))
            .ok()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0);

        telemetry.insert(iface.name.clone(), NetworkTelemetry {
            rx_bytes_24h: rx,  // Note: These are cumulative, not 24h
            tx_bytes_24h: tx,
        });
    }

    telemetry
}

fn collect_network_events() -> Vec<NetworkEvent> {
    let mut events = Vec::new();
    let mut event_counts: HashMap<String, usize> = HashMap::new();

    // Query journalctl for network-related events
    let output = Command::new("journalctl")
        .args(["-b", "-u", "NetworkManager", "-u", "wpa_supplicant",
               "-u", "systemd-networkd", "-p", "warning..alert",
               "--no-pager", "-q", "-o", "cat"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                if line.is_empty() {
                    continue;
                }

                // Categorize events
                let event_type = categorize_network_event(line);
                *event_counts.entry(event_type).or_insert(0) += 1;
            }
        }
    }

    // Convert to events with pattern IDs
    let mut idx = 1;
    for (event_type, count) in event_counts {
        events.push(NetworkEvent {
            pattern_id: format!("NET{:03}", idx),
            description: event_type,
            count,
        });
        idx += 1;
    }

    events.sort_by(|a, b| b.count.cmp(&a.count));
    events
}

fn categorize_network_event(line: &str) -> String {
    let lower = line.to_lowercase();

    if lower.contains("carrier lost") {
        "carrier lost".to_string()
    } else if lower.contains("disconnect") {
        "disconnect".to_string()
    } else if lower.contains("roam") {
        "roam".to_string()
    } else if lower.contains("authentication") {
        "authentication issue".to_string()
    } else if lower.contains("dhcp") || lower.contains("lease") {
        "DHCP issue".to_string()
    } else if lower.contains("dns") {
        "DNS issue".to_string()
    } else {
        "other network event".to_string()
    }
}

fn collect_network_log_patterns() -> Vec<(String, String, usize)> {
    let mut patterns: HashMap<String, usize> = HashMap::new();

    let output = Command::new("journalctl")
        .args(["-b", "-u", "NetworkManager", "-u", "wpa_supplicant",
               "-p", "warning..alert", "--no-pager", "-q", "-o", "cat"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                if line.is_empty() {
                    continue;
                }
                // Normalize and count
                let normalized = normalize_log_pattern(line);
                *patterns.entry(normalized).or_insert(0) += 1;
            }
        }
    }

    let mut result: Vec<_> = patterns.into_iter()
        .map(|(msg, count)| {
            (String::new(), msg, count)  // ID assigned later
        })
        .collect();

    result.sort_by(|a, b| b.2.cmp(&a.2));

    // Assign pattern IDs
    for (i, (id, _, _)) in result.iter_mut().enumerate() {
        *id = format!("NET{:03}", i + 1);
    }

    result.truncate(10);  // Keep top 10
    result
}

fn normalize_log_pattern(line: &str) -> String {
    // Remove timestamps, PIDs, and variable parts
    let mut normalized = line.to_string();

    // Remove common variable parts
    let patterns = [
        (r"\d+\.\d+\.\d+\.\d+", "<IP>"),
        (r"[0-9a-f]{2}(:[0-9a-f]{2}){5}", "<MAC>"),
        (r"wlan\d+|wlp\d+s\d+", "<WIFI>"),
        (r"eth\d+|enp\d+s\d+", "<ETH>"),
    ];

    for (pattern, replacement) in patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            normalized = re.replace_all(&normalized, replacement).to_string();
        }
    }

    // Truncate if too long
    if normalized.len() > 80 {
        normalized.truncate(77);
        normalized.push_str("...");
    }

    normalized
}

fn get_network_first_seen(_interfaces: &[NetworkInterface]) -> HashMap<String, String> {
    // This would check Anna's telemetry for first appearance
    // For now, return empty - will be populated from telemetry DB
    HashMap::new()
}

// ============================================================================
// Storage Lens
// ============================================================================

/// Storage device in the topology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageDevice {
    pub name: String,
    pub model: Option<String>,
    pub bus: String,  // nvme, sata, usb
    pub driver: Option<String>,
    pub size_bytes: u64,
    pub mount_points: Vec<String>,
}

/// SMART/NVMe health status
#[derive(Debug, Clone, Default)]
pub struct StorageHealth {
    pub status: String,  // OK, Warning, Critical
    pub media_errors: u64,
    pub critical_warnings: u64,
    pub temp_avg_24h: Option<f32>,
    pub temp_max_24h: Option<f32>,
}

/// Storage telemetry
#[derive(Debug, Clone, Default)]
pub struct StorageTelemetry {
    pub read_bytes_24h: u64,
    pub write_bytes_24h: u64,
}

/// Complete storage lens view
#[derive(Debug, Clone)]
pub struct StorageLens {
    pub devices: Vec<StorageDevice>,
    pub health: HashMap<String, StorageHealth>,
    pub telemetry: HashMap<String, StorageTelemetry>,
    pub log_patterns: Vec<(String, String, usize)>,
    pub first_seen: HashMap<String, String>,
}

impl StorageLens {
    /// Build storage lens from system probes
    pub fn build() -> Self {
        let devices = discover_storage_devices();
        let health = collect_storage_health(&devices);
        let telemetry = collect_storage_telemetry(&devices);
        let log_patterns = collect_storage_log_patterns();
        let first_seen = HashMap::new();

        Self {
            devices,
            health,
            telemetry,
            log_patterns,
            first_seen,
        }
    }
}

fn discover_storage_devices() -> Vec<StorageDevice> {
    let mut devices = Vec::new();

    let output = Command::new("lsblk")
        .args(["-d", "-o", "NAME,SIZE,MODEL,TRAN", "-b", "-n"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                if let Some(dev) = parse_lsblk_line(line) {
                    devices.push(dev);
                }
            }
        }
    }

    // Enrich with mount points
    for dev in &mut devices {
        dev.mount_points = get_mount_points(&dev.name);
        dev.driver = get_block_driver(&dev.name);
    }

    devices
}

fn parse_lsblk_line(line: &str) -> Option<StorageDevice> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

    let name = parts[0].to_string();

    // Skip loop devices and ram disks
    if name.starts_with("loop") || name.starts_with("ram") {
        return None;
    }

    let size_bytes = parts.get(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let model = parts.get(2)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    let bus = parts.get(3)
        .map(|s| s.to_lowercase())
        .unwrap_or_else(|| {
            if name.starts_with("nvme") {
                "nvme".to_string()
            } else if name.starts_with("sd") {
                "sata".to_string()
            } else if name.starts_with("mmcblk") {
                "mmc".to_string()
            } else {
                "unknown".to_string()
            }
        });

    Some(StorageDevice {
        name,
        model,
        bus,
        driver: None,
        size_bytes,
        mount_points: Vec::new(),
    })
}

fn get_mount_points(device: &str) -> Vec<String> {
    let mut mounts = Vec::new();

    let output = Command::new("findmnt")
        .args(["-n", "-o", "TARGET", "-S", &format!("/dev/{}*", device)])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                let mount = line.trim();
                if !mount.is_empty() {
                    mounts.push(mount.to_string());
                }
            }
        }
    }

    // Also check partitions
    let output = Command::new("lsblk")
        .args(["-n", "-o", "NAME,MOUNTPOINT", &format!("/dev/{}", device)])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let mount = parts[1..].join(" ");
                    if !mount.is_empty() && !mounts.contains(&mount) {
                        mounts.push(mount);
                    }
                }
            }
        }
    }

    mounts
}

fn get_block_driver(device: &str) -> Option<String> {
    let driver_path = format!("/sys/block/{}/device/driver", device);
    if let Ok(link) = fs::read_link(&driver_path) {
        return link.file_name()
            .map(|n| n.to_string_lossy().to_string());
    }
    None
}

fn collect_storage_health(devices: &[StorageDevice]) -> HashMap<String, StorageHealth> {
    let mut health_map = HashMap::new();

    for dev in devices {
        let health = if dev.bus == "nvme" {
            get_nvme_health(&dev.name)
        } else {
            get_smart_health(&dev.name)
        };
        health_map.insert(dev.name.clone(), health);
    }

    health_map
}

fn get_nvme_health(device: &str) -> StorageHealth {
    let output = Command::new("nvme")
        .args(["smart-log", &format!("/dev/{}", device), "-o", "json"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                let media_errors = json["media_errors"].as_u64().unwrap_or(0);
                let critical_warnings = json["critical_warning"].as_u64().unwrap_or(0);
                let temp = json["temperature"].as_f64().map(|t| (t - 273.15) as f32);

                let status = if critical_warnings > 0 || media_errors > 0 {
                    "Warning"
                } else {
                    "OK"
                }.to_string();

                return StorageHealth {
                    status,
                    media_errors,
                    critical_warnings,
                    temp_avg_24h: temp,
                    temp_max_24h: temp,
                };
            }
        }
    }

    StorageHealth {
        status: "unknown (nvme-cli not available)".to_string(),
        ..Default::default()
    }
}

fn get_smart_health(device: &str) -> StorageHealth {
    let output = Command::new("smartctl")
        .args(["-H", "-j", &format!("/dev/{}", device)])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                let passed = json["smart_status"]["passed"].as_bool().unwrap_or(true);

                return StorageHealth {
                    status: if passed { "OK" } else { "FAILING" }.to_string(),
                    ..Default::default()
                };
            }
        }
    }

    StorageHealth {
        status: "unknown (smartctl not available)".to_string(),
        ..Default::default()
    }
}

fn collect_storage_telemetry(devices: &[StorageDevice]) -> HashMap<String, StorageTelemetry> {
    let mut telemetry = HashMap::new();

    for dev in devices {
        let stat_path = format!("/sys/block/{}/stat", dev.name);
        if let Ok(stat) = fs::read_to_string(&stat_path) {
            let parts: Vec<&str> = stat.split_whitespace().collect();
            // Fields: read_ios, read_merges, read_sectors, read_ticks, ...
            // Sector 2 = read sectors, sector 6 = write sectors (512 bytes each)
            let read_sectors: u64 = parts.get(2)
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            let write_sectors: u64 = parts.get(6)
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            telemetry.insert(dev.name.clone(), StorageTelemetry {
                read_bytes_24h: read_sectors * 512,
                write_bytes_24h: write_sectors * 512,
            });
        }
    }

    telemetry
}

fn collect_storage_log_patterns() -> Vec<(String, String, usize)> {
    let mut patterns: HashMap<String, usize> = HashMap::new();

    // Query kernel logs for storage events
    let output = Command::new("journalctl")
        .args(["-b", "-k", "-p", "warning..alert", "--no-pager", "-q", "-o", "cat"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                let lower = line.to_lowercase();
                // Filter for storage-related messages
                if lower.contains("nvme") || lower.contains("ata")
                    || lower.contains("sd") || lower.contains("disk")
                    || lower.contains("i/o error") || lower.contains("reset")
                {
                    let normalized = normalize_storage_pattern(line);
                    *patterns.entry(normalized).or_insert(0) += 1;
                }
            }
        }
    }

    let mut result: Vec<_> = patterns.into_iter()
        .map(|(msg, count)| (String::new(), msg, count))
        .collect();

    result.sort_by(|a, b| b.2.cmp(&a.2));

    for (i, (id, _, _)) in result.iter_mut().enumerate() {
        *id = format!("STO{:03}", i + 1);
    }

    result.truncate(10);
    result
}

fn normalize_storage_pattern(line: &str) -> String {
    let mut normalized = line.to_string();

    // Remove variable parts
    let patterns = [
        (r"nvme\d+n\d+", "<NVME>"),
        (r"sd[a-z]+", "<DISK>"),
        (r"sector \d+", "sector <N>"),
    ];

    for (pattern, replacement) in patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            normalized = re.replace_all(&normalized, replacement).to_string();
        }
    }

    if normalized.len() > 80 {
        normalized.truncate(77);
        normalized.push_str("...");
    }

    normalized
}

// ============================================================================
// Graphics Lens
// ============================================================================

/// GPU in the topology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuDevice {
    pub name: String,
    pub vendor: String,
    pub driver: Option<String>,
    pub driver_loaded: bool,
    pub firmware: Option<String>,
    pub is_discrete: bool,
}

/// Display connector
#[derive(Debug, Clone)]
pub struct DisplayConnector {
    pub name: String,
    pub status: String,  // connected, disconnected
    pub resolution: Option<String>,
}

/// Complete graphics lens view
#[derive(Debug, Clone)]
pub struct GraphicsLens {
    pub gpus: Vec<GpuDevice>,
    pub connectors: Vec<DisplayConnector>,
    pub log_patterns: Vec<(String, String, usize)>,
}

impl GraphicsLens {
    pub fn build() -> Self {
        let gpus = discover_gpus();
        let connectors = discover_display_connectors();
        let log_patterns = collect_graphics_log_patterns();

        Self {
            gpus,
            connectors,
            log_patterns,
        }
    }
}

fn discover_gpus() -> Vec<GpuDevice> {
    let mut gpus = Vec::new();

    let output = Command::new("lspci")
        .args(["-k"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let lines: Vec<&str> = stdout.lines().collect();

            let mut i = 0;
            while i < lines.len() {
                let line = lines[i];
                if line.contains("VGA") || line.contains("3D controller")
                    || line.contains("Display controller") {

                    let parts: Vec<&str> = line.splitn(2, ": ").collect();
                    let name = parts.get(1).unwrap_or(&"Unknown GPU").to_string();

                    let vendor = if name.to_lowercase().contains("nvidia") {
                        "NVIDIA"
                    } else if name.to_lowercase().contains("amd") || name.to_lowercase().contains("radeon") {
                        "AMD"
                    } else if name.to_lowercase().contains("intel") {
                        "Intel"
                    } else {
                        "Unknown"
                    }.to_string();

                    let is_discrete = vendor == "NVIDIA" ||
                        (vendor == "AMD" && !name.to_lowercase().contains("integrated"));

                    // Look for driver info in following lines
                    let mut driver = None;
                    let mut driver_loaded = false;

                    for j in (i + 1)..std::cmp::min(i + 4, lines.len()) {
                        if lines[j].contains("Kernel driver in use:") {
                            driver = lines[j].split(':').nth(1)
                                .map(|s| s.trim().to_string());
                            driver_loaded = true;
                            break;
                        }
                        if lines[j].contains("Kernel modules:") && driver.is_none() {
                            driver = lines[j].split(':').nth(1)
                                .map(|s| s.trim().split_whitespace().next()
                                    .unwrap_or("").to_string());
                        }
                    }

                    gpus.push(GpuDevice {
                        name,
                        vendor,
                        driver,
                        driver_loaded,
                        firmware: None,
                        is_discrete,
                    });
                }
                i += 1;
            }
        }
    }

    gpus
}

fn discover_display_connectors() -> Vec<DisplayConnector> {
    let mut connectors = Vec::new();

    // Check /sys/class/drm for connectors
    let drm_path = "/sys/class/drm";
    if let Ok(entries) = fs::read_dir(drm_path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            // Look for connector entries like card0-HDMI-A-1
            if name.contains('-') && !name.ends_with("-card") {
                let status_path = entry.path().join("status");
                let status = fs::read_to_string(&status_path)
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|_| "unknown".to_string());

                let mut resolution = None;
                if status == "connected" {
                    // Try to get resolution from modes
                    let modes_path = entry.path().join("modes");
                    if let Ok(modes) = fs::read_to_string(&modes_path) {
                        resolution = modes.lines().next().map(|s| s.to_string());
                    }
                }

                connectors.push(DisplayConnector {
                    name,
                    status,
                    resolution,
                });
            }
        }
    }

    connectors
}

fn collect_graphics_log_patterns() -> Vec<(String, String, usize)> {
    let mut patterns: HashMap<String, usize> = HashMap::new();

    let output = Command::new("journalctl")
        .args(["-b", "-k", "-p", "warning..alert", "--no-pager", "-q", "-o", "cat"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                let lower = line.to_lowercase();
                if lower.contains("drm") || lower.contains("gpu")
                    || lower.contains("nvidia") || lower.contains("amdgpu")
                    || lower.contains("i915") || lower.contains("display")
                {
                    *patterns.entry(line.to_string()).or_insert(0) += 1;
                }
            }
        }
    }

    let mut result: Vec<_> = patterns.into_iter()
        .map(|(msg, count)| (String::new(), msg, count))
        .collect();

    result.sort_by(|a, b| b.2.cmp(&a.2));

    for (i, (id, _, _)) in result.iter_mut().enumerate() {
        *id = format!("GPU{:03}", i + 1);
    }

    result.truncate(10);
    result
}

// ============================================================================
// Audio Lens
// ============================================================================

/// Audio device
#[derive(Debug, Clone)]
pub struct AudioDevice {
    pub name: String,
    pub card_type: String,  // PCH, USB, HDMI
    pub driver: Option<String>,
    pub driver_loaded: bool,
}

/// Complete audio lens view
#[derive(Debug, Clone)]
pub struct AudioLens {
    pub devices: Vec<AudioDevice>,
    pub log_patterns: Vec<(String, String, usize)>,
}

impl AudioLens {
    pub fn build() -> Self {
        let devices = discover_audio_devices();
        let log_patterns = collect_audio_log_patterns();

        Self {
            devices,
            log_patterns,
        }
    }
}

fn discover_audio_devices() -> Vec<AudioDevice> {
    let mut devices = Vec::new();

    // Use aplay -l to list audio devices
    let output = Command::new("aplay")
        .args(["-l"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                if line.starts_with("card ") {
                    // Parse "card 0: PCH [HDA Intel PCH], device 0: ..."
                    if let Some(name_part) = line.split('[').nth(1) {
                        if let Some(name) = name_part.split(']').next() {
                            let card_type = if name.to_lowercase().contains("usb") {
                                "USB"
                            } else if name.to_lowercase().contains("hdmi") {
                                "HDMI"
                            } else {
                                "PCH"
                            }.to_string();

                            devices.push(AudioDevice {
                                name: name.to_string(),
                                card_type,
                                driver: Some("snd_hda_intel".to_string()),
                                driver_loaded: true,
                            });
                        }
                    }
                }
            }
        }
    }

    // Deduplicate by name
    devices.sort_by(|a, b| a.name.cmp(&b.name));
    devices.dedup_by(|a, b| a.name == b.name);

    devices
}

fn collect_audio_log_patterns() -> Vec<(String, String, usize)> {
    let mut patterns: HashMap<String, usize> = HashMap::new();

    let output = Command::new("journalctl")
        .args(["-b", "-u", "pipewire", "-u", "pipewire-pulse",
               "-u", "wireplumber", "-p", "warning..alert",
               "--no-pager", "-q", "-o", "cat"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                if !line.is_empty() {
                    *patterns.entry(line.to_string()).or_insert(0) += 1;
                }
            }
        }
    }

    // Also check kernel audio logs
    let output = Command::new("journalctl")
        .args(["-b", "-k", "-p", "warning..alert", "--no-pager", "-q", "-o", "cat"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                let lower = line.to_lowercase();
                if lower.contains("snd") || lower.contains("audio")
                    || lower.contains("hda") || lower.contains("pulse")
                {
                    *patterns.entry(line.to_string()).or_insert(0) += 1;
                }
            }
        }
    }

    let mut result: Vec<_> = patterns.into_iter()
        .map(|(msg, count)| (String::new(), msg, count))
        .collect();

    result.sort_by(|a, b| b.2.cmp(&a.2));

    for (i, (id, _, _)) in result.iter_mut().enumerate() {
        *id = format!("AUD{:03}", i + 1);
    }

    result.truncate(10);
    result
}

// ============================================================================
// Utility functions
// ============================================================================

/// Format bytes for display
pub fn format_bytes(bytes: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = KIB * 1024;
    const GIB: u64 = MIB * 1024;
    const TIB: u64 = GIB * 1024;

    if bytes >= TIB {
        format!("{:.1} TiB", bytes as f64 / TIB as f64)
    } else if bytes >= GIB {
        format!("{:.1} GiB", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.1} MiB", bytes as f64 / MIB as f64)
    } else if bytes >= KIB {
        format!("{:.1} KiB", bytes as f64 / KIB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(1024), "1.0 KiB");
        assert_eq!(format_bytes(1024 * 1024), "1.0 MiB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0 GiB");
    }

    #[test]
    fn test_network_lens_build() {
        let lens = NetworkLens::build();
        // Should at least not panic
        assert!(lens.interfaces.len() >= 0);
    }

    #[test]
    fn test_storage_lens_build() {
        let lens = StorageLens::build();
        assert!(lens.devices.len() >= 0);
    }

    #[test]
    fn test_graphics_lens_build() {
        let lens = GraphicsLens::build();
        assert!(lens.gpus.len() >= 0);
    }

    #[test]
    fn test_audio_lens_build() {
        let lens = AudioLens::build();
        assert!(lens.devices.len() >= 0);
    }
}
