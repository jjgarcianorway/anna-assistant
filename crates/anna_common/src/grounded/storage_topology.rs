//! Storage Topology v7.17.0 - Devices, Partitions, Filesystems and Health
//!
//! Provides structured storage information:
//! - Block devices with model, size, bus type
//! - Partitions with filesystem type and mount points
//! - SMART/NVMe health status
//! - Filesystem usage and mount options
//!
//! Sources:
//! - lsblk, lsblk -f, lsblk -o
//! - findmnt, mount
//! - smartctl, nvme list, nvme smart-log
//! - /sys/block/*, /proc/mounts

use std::process::Command;
use serde::{Deserialize, Serialize};

/// Block device type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockDeviceType {
    Nvme,
    Sata,
    Usb,
    Mmc,  // SD cards, eMMC
    Loop,
    Dm,   // Device mapper (LVM, LUKS)
    Unknown,
}

impl BlockDeviceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            BlockDeviceType::Nvme => "NVMe",
            BlockDeviceType::Sata => "SATA",
            BlockDeviceType::Usb => "USB",
            BlockDeviceType::Mmc => "MMC",
            BlockDeviceType::Loop => "loop",
            BlockDeviceType::Dm => "dm",
            BlockDeviceType::Unknown => "unknown",
        }
    }

    pub fn from_name(name: &str) -> Self {
        if name.starts_with("nvme") {
            BlockDeviceType::Nvme
        } else if name.starts_with("sd") {
            // Could be SATA or USB, check /sys
            BlockDeviceType::Sata
        } else if name.starts_with("mmcblk") {
            BlockDeviceType::Mmc
        } else if name.starts_with("loop") {
            BlockDeviceType::Loop
        } else if name.starts_with("dm-") {
            BlockDeviceType::Dm
        } else {
            BlockDeviceType::Unknown
        }
    }
}

/// Block device information
#[derive(Debug, Clone, Default)]
pub struct BlockDevice {
    pub name: String,
    pub model: Option<String>,
    pub serial: Option<String>,
    pub size_bytes: u64,
    pub size_human: String,
    pub device_type: String,
    pub transport: String,  // NVMe, SATA, USB
    pub firmware: Option<String>,
    pub partitions: Vec<Partition>,
    pub health: DeviceHealth,
}

/// Partition information
#[derive(Debug, Clone, Default)]
pub struct Partition {
    pub name: String,
    pub size_bytes: u64,
    pub size_human: String,
    pub fstype: Option<String>,
    pub label: Option<String>,
    pub uuid: Option<String>,
    pub mountpoint: Option<String>,
    pub mount_options: Option<String>,
}

/// Device health from SMART/NVMe
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeviceHealth {
    pub status: String,      // OK, WARNING, FAILING
    pub smart_available: bool,
    pub power_on_hours: Option<u64>,
    pub temperature_c: Option<i32>,
    pub media_errors: Option<u64>,
    pub reallocated_sectors: Option<u64>,
    pub percentage_used: Option<u8>,  // NVMe wear indicator
    pub warnings: Vec<String>,
    pub source: String,
}

/// Filesystem mount information
#[derive(Debug, Clone, Default)]
pub struct FilesystemMount {
    pub device: String,
    pub mountpoint: String,
    pub fstype: String,
    pub options: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub use_percent: u8,
    /// Subvolume for btrfs
    pub subvolume: Option<String>,
}

/// Storage topology summary
#[derive(Debug, Clone, Default)]
pub struct StorageTopology {
    pub devices: Vec<BlockDevice>,
    pub mounts: Vec<FilesystemMount>,
    pub root_device: Option<String>,
    pub source: String,
}

/// Get all block devices using lsblk
pub fn get_block_devices() -> Vec<BlockDevice> {
    let mut devices = Vec::new();

    // Use lsblk with JSON output for reliable parsing
    let output = Command::new("lsblk")
        .args(["-J", "-b", "-o", "NAME,SIZE,TYPE,MODEL,SERIAL,TRAN,FSTYPE,LABEL,UUID,MOUNTPOINT"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&out.stdout) {
                if let Some(blockdevices) = json.get("blockdevices").and_then(|v| v.as_array()) {
                    for dev in blockdevices {
                        if let Some(name) = dev.get("name").and_then(|v| v.as_str()) {
                            // Skip loop and dm devices for main listing
                            if name.starts_with("loop") || name.starts_with("dm-") {
                                continue;
                            }

                            let dev_type = dev.get("type").and_then(|v| v.as_str()).unwrap_or("");
                            if dev_type != "disk" {
                                continue;
                            }

                            let size = dev.get("size").and_then(|v| v.as_u64()).unwrap_or(0);
                            let transport = dev.get("tran").and_then(|v| v.as_str())
                                .unwrap_or("").to_string();

                            let mut block_dev = BlockDevice {
                                name: name.to_string(),
                                model: dev.get("model").and_then(|v| v.as_str())
                                    .map(|s| s.trim().to_string()),
                                serial: dev.get("serial").and_then(|v| v.as_str())
                                    .map(|s| s.to_string()),
                                size_bytes: size,
                                size_human: format_size(size),
                                device_type: BlockDeviceType::from_name(name).as_str().to_string(),
                                transport: if transport.is_empty() {
                                    BlockDeviceType::from_name(name).as_str().to_string()
                                } else {
                                    transport.to_uppercase()
                                },
                                firmware: None,
                                partitions: Vec::new(),
                                health: DeviceHealth::default(),
                            };

                            // Parse children (partitions)
                            if let Some(children) = dev.get("children").and_then(|v| v.as_array()) {
                                for child in children {
                                    if let Some(child_name) = child.get("name").and_then(|v| v.as_str()) {
                                        let child_size = child.get("size").and_then(|v| v.as_u64()).unwrap_or(0);
                                        let partition = Partition {
                                            name: child_name.to_string(),
                                            size_bytes: child_size,
                                            size_human: format_size(child_size),
                                            fstype: child.get("fstype").and_then(|v| v.as_str())
                                                .map(|s| s.to_string()),
                                            label: child.get("label").and_then(|v| v.as_str())
                                                .map(|s| s.to_string()),
                                            uuid: child.get("uuid").and_then(|v| v.as_str())
                                                .map(|s| s.to_string()),
                                            mountpoint: child.get("mountpoint").and_then(|v| v.as_str())
                                                .map(|s| s.to_string()),
                                            mount_options: None,
                                        };
                                        block_dev.partitions.push(partition);
                                    }
                                }
                            }

                            // Get health status
                            block_dev.health = get_device_health(&block_dev.name);

                            devices.push(block_dev);
                        }
                    }
                }
            }
        }
    }

    devices
}

/// Get device health from SMART/NVMe
pub fn get_device_health(device: &str) -> DeviceHealth {
    let dev_path = format!("/dev/{}", device);

    // Try NVMe first
    if device.starts_with("nvme") {
        return get_nvme_health(&dev_path);
    }

    // Try SMART for SATA/USB
    get_smart_health(&dev_path)
}

fn get_nvme_health(dev_path: &str) -> DeviceHealth {
    let mut health = DeviceHealth {
        source: "nvme smart-log".to_string(),
        ..Default::default()
    };

    let output = Command::new("nvme")
        .args(["smart-log", dev_path, "-o", "json"])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            health.smart_available = true;
            if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&out.stdout) {
                health.temperature_c = json.get("temperature")
                    .and_then(|v| v.as_i64())
                    .map(|t| (t - 273) as i32); // Convert from Kelvin

                health.power_on_hours = json.get("power_on_hours")
                    .and_then(|v| v.as_u64());

                health.media_errors = json.get("media_errors")
                    .and_then(|v| v.as_u64());

                health.percentage_used = json.get("percent_used")
                    .and_then(|v| v.as_u64())
                    .map(|p| p as u8);

                // Determine status
                let critical_warning = json.get("critical_warning")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);

                if critical_warning > 0 || health.media_errors.unwrap_or(0) > 0 {
                    health.status = "WARNING".to_string();
                    if critical_warning > 0 {
                        health.warnings.push(format!("Critical warning flags: {}", critical_warning));
                    }
                    if let Some(errors) = health.media_errors {
                        if errors > 0 {
                            health.warnings.push(format!("{} media errors", errors));
                        }
                    }
                } else {
                    health.status = "OK".to_string();
                }
            }
        }
        _ => {
            health.status = "unknown (nvme-cli not available)".to_string();
            health.source = "nvme-cli not installed".to_string();
        }
    }

    health
}

fn get_smart_health(dev_path: &str) -> DeviceHealth {
    let mut health = DeviceHealth {
        source: "smartctl".to_string(),
        ..Default::default()
    };

    let output = Command::new("smartctl")
        .args(["-H", "-A", "-j", dev_path])
        .output();

    match output {
        Ok(out) if out.status.success() || out.status.code() == Some(4) => {
            // smartctl returns 4 for "SMART check returned PASSED"
            health.smart_available = true;

            if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&out.stdout) {
                // Get overall health
                if let Some(smart_status) = json.get("smart_status").and_then(|v| v.get("passed")) {
                    health.status = if smart_status.as_bool().unwrap_or(false) {
                        "OK".to_string()
                    } else {
                        "FAILING".to_string()
                    };
                }

                // Get temperature
                if let Some(temp) = json.get("temperature").and_then(|v| v.get("current")) {
                    health.temperature_c = temp.as_i64().map(|t| t as i32);
                }

                // Get power on hours
                if let Some(poh) = json.get("power_on_time").and_then(|v| v.get("hours")) {
                    health.power_on_hours = poh.as_u64();
                }

                // Get reallocated sectors from SMART attributes
                if let Some(attrs) = json.get("ata_smart_attributes").and_then(|v| v.get("table")) {
                    if let Some(table) = attrs.as_array() {
                        for attr in table {
                            let id = attr.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
                            if id == 5 { // Reallocated Sector Count
                                health.reallocated_sectors = attr.get("raw")
                                    .and_then(|v| v.get("value"))
                                    .and_then(|v| v.as_u64());
                                if let Some(sectors) = health.reallocated_sectors {
                                    if sectors > 0 {
                                        health.status = "WARNING".to_string();
                                        health.warnings.push(format!("{} reallocated sectors", sectors));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        _ => {
            health.status = "unknown (smartctl not available)".to_string();
            health.source = "smartmontools not installed".to_string();
        }
    }

    health
}

/// Get filesystem mounts using findmnt
pub fn get_filesystem_mounts() -> Vec<FilesystemMount> {
    let mut mounts = Vec::new();

    let output = Command::new("findmnt")
        .args(["-J", "-o", "SOURCE,TARGET,FSTYPE,OPTIONS,USE%"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&out.stdout) {
                if let Some(filesystems) = json.get("filesystems").and_then(|v| v.as_array()) {
                    for fs in filesystems {
                        parse_mount_entry(fs, &mut mounts);
                    }
                }
            }
        }
    }

    // Enrich with df data for usage
    enrich_mounts_with_usage(&mut mounts);

    mounts
}

fn parse_mount_entry(entry: &serde_json::Value, mounts: &mut Vec<FilesystemMount>) {
    if let (Some(source), Some(target), Some(fstype)) = (
        entry.get("source").and_then(|v| v.as_str()),
        entry.get("target").and_then(|v| v.as_str()),
        entry.get("fstype").and_then(|v| v.as_str()),
    ) {
        // Skip pseudo filesystems
        if matches!(fstype, "sysfs" | "proc" | "devtmpfs" | "devpts" | "tmpfs" |
                   "securityfs" | "cgroup2" | "pstore" | "efivarfs" | "bpf" |
                   "autofs" | "hugetlbfs" | "mqueue" | "debugfs" | "tracefs" |
                   "fusectl" | "configfs" | "ramfs" | "fuse.portal") {
            // Still include /tmp if it's tmpfs
            if !(fstype == "tmpfs" && target == "/tmp") {
                return;
            }
        }

        let options = entry.get("options").and_then(|v| v.as_str())
            .unwrap_or("").to_string();

        // Extract subvolume for btrfs
        let subvolume = if fstype == "btrfs" {
            options.split(',')
                .find(|o| o.starts_with("subvol="))
                .map(|s| s.trim_start_matches("subvol=").to_string())
        } else {
            None
        };

        mounts.push(FilesystemMount {
            device: source.to_string(),
            mountpoint: target.to_string(),
            fstype: fstype.to_string(),
            options,
            subvolume,
            ..Default::default()
        });
    }

    // Recursively parse children
    if let Some(children) = entry.get("children").and_then(|v| v.as_array()) {
        for child in children {
            parse_mount_entry(child, mounts);
        }
    }
}

fn enrich_mounts_with_usage(mounts: &mut [FilesystemMount]) {
    let output = Command::new("df")
        .args(["-B1", "--output=target,size,used,avail,pcent"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 5 {
                    let target = parts[0];
                    if let Some(mount) = mounts.iter_mut().find(|m| m.mountpoint == target) {
                        mount.total_bytes = parts[1].parse().unwrap_or(0);
                        mount.used_bytes = parts[2].parse().unwrap_or(0);
                        mount.available_bytes = parts[3].parse().unwrap_or(0);
                        mount.use_percent = parts[4].trim_end_matches('%').parse().unwrap_or(0);
                    }
                }
            }
        }
    }
}

/// Get storage topology
pub fn get_storage_topology() -> StorageTopology {
    let devices = get_block_devices();
    let mounts = get_filesystem_mounts();

    // Find root device
    let root_device = mounts.iter()
        .find(|m| m.mountpoint == "/")
        .map(|m| {
            // Extract base device from partition (e.g., nvme0n1 from /dev/nvme0n1p2)
            let dev = m.device.trim_start_matches("/dev/");
            if dev.starts_with("nvme") {
                // NVMe: remove pN suffix
                dev.split('p').next().unwrap_or(dev).to_string()
            } else if dev.starts_with("sd") || dev.starts_with("vd") {
                // SATA/virtio: remove numeric suffix
                dev.trim_end_matches(|c: char| c.is_ascii_digit()).to_string()
            } else {
                dev.to_string()
            }
        });

    StorageTopology {
        devices,
        mounts,
        root_device,
        source: "lsblk, findmnt, smartctl/nvme".to_string(),
    }
}

/// Format size in human-readable form
pub fn format_size(bytes: u64) -> String {
    const GIB: u64 = 1024 * 1024 * 1024;
    const MIB: u64 = 1024 * 1024;

    if bytes >= GIB {
        format!("{:.1} GiB", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.1} MiB", bytes as f64 / MIB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

/// Format device summary for hw overview
pub fn format_devices_summary(devices: &[BlockDevice]) -> Vec<String> {
    devices.iter()
        .map(|d| {
            let health_str = if d.health.smart_available {
                format!("({} {})", d.health.source.split_whitespace().next().unwrap_or("SMART"), d.health.status)
            } else {
                "(health unknown)".to_string()
            };
            format!("{:<10} {:<10} {:<6} {}",
                    d.name, d.size_human, d.device_type, health_str)
        })
        .collect()
}

/// Format filesystem summary for hw overview
pub fn format_filesystems_summary(mounts: &[FilesystemMount]) -> Vec<String> {
    mounts.iter()
        .filter(|m| matches!(m.mountpoint.as_str(), "/" | "/home" | "/boot" | "/boot/efi" | "/var" | "/tmp"))
        .map(|m| {
            let subvol = m.subvolume.as_ref()
                .map(|s| format!(" subvolume {}", s))
                .unwrap_or_default();
            format!("{:<10} {} on {}{} ({} percent used)",
                    m.mountpoint, m.fstype, m.device.trim_start_matches("/dev/"),
                    subvol, m.use_percent)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_device_type() {
        assert_eq!(BlockDeviceType::from_name("nvme0n1").as_str(), "NVMe");
        assert_eq!(BlockDeviceType::from_name("sda").as_str(), "SATA");
        assert_eq!(BlockDeviceType::from_name("mmcblk0").as_str(), "MMC");
        assert_eq!(BlockDeviceType::from_name("loop0").as_str(), "loop");
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(1024 * 1024 * 1024), "1.0 GiB");
        assert_eq!(format_size(500 * 1024 * 1024 * 1024), "500.0 GiB");
        assert_eq!(format_size(256 * 1024 * 1024), "256.0 MiB");
    }

    #[test]
    fn test_get_storage_topology() {
        let topo = get_storage_topology();
        // Should have at least source
        assert!(!topo.source.is_empty());
    }
}
