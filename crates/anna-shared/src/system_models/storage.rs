//! Storage Model - Block devices, mounts, fstab, filesystem health, SMART status.
//!
//! Models the storage hierarchy for understanding:
//! - Physical disks and partitions
//! - Mount points and filesystem types
//! - fstab configuration
//! - Disk health via SMART

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete storage model
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StorageModel {
    /// Block devices (disks)
    pub devices: HashMap<String, BlockDevice>,
    /// Mount points
    pub mounts: HashMap<String, MountPoint>,
    /// fstab entries
    pub fstab: Vec<FstabEntry>,
    /// SMART data by device
    pub smart: HashMap<String, SmartData>,
    /// Filesystem usage
    pub usage: HashMap<String, FilesystemUsage>,
}

/// A block device (disk or partition)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDevice {
    /// Device name (e.g., "sda", "nvme0n1")
    pub name: String,
    /// Full device path
    pub path: String,
    /// Device type
    pub device_type: DeviceType,
    /// Size in bytes
    pub size: u64,
    /// Model (for physical disks)
    pub model: Option<String>,
    /// Serial number
    pub serial: Option<String>,
    /// Is removable?
    pub removable: bool,
    /// Is read-only?
    pub read_only: bool,
    /// Rotational (HDD) or not (SSD/NVMe)
    pub rotational: bool,
    /// Partitions
    pub partitions: Vec<Partition>,
    /// Parent device (for partitions)
    pub parent: Option<String>,
}

/// Device types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceType {
    Disk,
    Partition,
    Loop,
    Lvm,
    Raid,
    Crypt,
    Rom,
    Unknown,
}

impl DeviceType {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "disk" => DeviceType::Disk,
            "part" => DeviceType::Partition,
            "loop" => DeviceType::Loop,
            "lvm" => DeviceType::Lvm,
            "raid" | "raid0" | "raid1" | "raid5" | "raid6" => DeviceType::Raid,
            "crypt" => DeviceType::Crypt,
            "rom" => DeviceType::Rom,
            _ => DeviceType::Unknown,
        }
    }
}

/// A partition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Partition {
    /// Partition name
    pub name: String,
    /// Full path
    pub path: String,
    /// Size in bytes
    pub size: u64,
    /// Filesystem type
    pub fstype: Option<String>,
    /// UUID
    pub uuid: Option<String>,
    /// Label
    pub label: Option<String>,
    /// Mount point (if mounted)
    pub mountpoint: Option<String>,
    /// Partition type (GPT type GUID or MBR type)
    pub parttype: Option<String>,
}

/// A mount point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountPoint {
    /// Mount path
    pub path: String,
    /// Source device or remote
    pub source: String,
    /// Filesystem type
    pub fstype: String,
    /// Mount options
    pub options: Vec<String>,
    /// Is this in fstab?
    pub in_fstab: bool,
}

/// An fstab entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FstabEntry {
    /// Device spec (UUID=..., /dev/..., etc.)
    pub spec: String,
    /// Mount point
    pub mount_point: String,
    /// Filesystem type
    pub fstype: String,
    /// Mount options
    pub options: String,
    /// Dump flag
    pub dump: u8,
    /// Pass flag
    pub pass: u8,
    /// Is currently mounted?
    pub mounted: bool,
    /// Any issues with this entry
    pub issues: Vec<String>,
}

/// SMART data for a device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartData {
    /// Device name
    pub device: String,
    /// Overall health status
    pub health: SmartHealth,
    /// Power on hours
    pub power_on_hours: Option<u64>,
    /// Temperature in Celsius
    pub temperature: Option<u8>,
    /// Reallocated sectors count
    pub reallocated_sectors: Option<u64>,
    /// Pending sectors
    pub pending_sectors: Option<u64>,
    /// Uncorrectable errors
    pub uncorrectable_errors: Option<u64>,
    /// Last self-test result
    pub last_test: Option<String>,
    /// Raw SMART attributes
    pub attributes: Vec<SmartAttribute>,
}

/// SMART health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SmartHealth {
    Passed,
    Failed,
    Unknown,
}

/// A SMART attribute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartAttribute {
    pub id: u8,
    pub name: String,
    pub value: u64,
    pub worst: u64,
    pub threshold: u64,
    pub raw: u64,
    pub failing: bool,
}

/// Filesystem usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemUsage {
    /// Mount point
    pub mount_point: String,
    /// Total size in bytes
    pub total: u64,
    /// Used space in bytes
    pub used: u64,
    /// Available space in bytes
    pub available: u64,
    /// Usage percentage
    pub percent_used: f32,
    /// Inode usage percentage
    pub percent_inodes: f32,
}

impl StorageModel {
    /// Create new empty storage model
    pub fn new() -> Self {
        Self::default()
    }

    /// Add or update a device
    pub fn upsert_device(&mut self, device: BlockDevice) {
        self.devices.insert(device.name.clone(), device);
    }

    /// Add or update a mount
    pub fn upsert_mount(&mut self, mount: MountPoint) {
        self.mounts.insert(mount.path.clone(), mount);
    }

    /// Get device by UUID
    pub fn device_by_uuid(&self, uuid: &str) -> Option<&BlockDevice> {
        for device in self.devices.values() {
            for part in &device.partitions {
                if part.uuid.as_deref() == Some(uuid) {
                    return Some(device);
                }
            }
        }
        None
    }

    /// Count unhealthy devices
    pub fn count_unhealthy(&self) -> usize {
        let mut count = 0;

        // SMART failures
        count += self
            .smart
            .values()
            .filter(|s| s.health == SmartHealth::Failed)
            .count();

        // Filesystems over 90% full
        count += self.usage.values().filter(|u| u.percent_used > 90.0).count();

        // fstab issues
        count += self.fstab.iter().filter(|f| !f.issues.is_empty()).count();

        count
    }

    /// Get critical storage issues
    pub fn diagnose(&self) -> Vec<StorageIssue> {
        let mut issues = Vec::new();

        // Check SMART health
        for (device, smart) in &self.smart {
            if smart.health == SmartHealth::Failed {
                issues.push(StorageIssue {
                    severity: StorageIssueSeverity::Critical,
                    device: device.clone(),
                    description: "SMART health check failed".to_string(),
                    suggestion: "Backup data immediately and replace drive".to_string(),
                });
            }

            if let Some(reallocated) = smart.reallocated_sectors {
                if reallocated > 0 {
                    issues.push(StorageIssue {
                        severity: if reallocated > 100 {
                            StorageIssueSeverity::High
                        } else {
                            StorageIssueSeverity::Medium
                        },
                        device: device.clone(),
                        description: format!("{} reallocated sectors detected", reallocated),
                        suggestion: "Monitor drive health, consider backup".to_string(),
                    });
                }
            }

            if let Some(temp) = smart.temperature {
                if temp > 60 {
                    issues.push(StorageIssue {
                        severity: StorageIssueSeverity::High,
                        device: device.clone(),
                        description: format!("High drive temperature: {}C", temp),
                        suggestion: "Check cooling and airflow".to_string(),
                    });
                }
            }
        }

        // Check filesystem usage
        for (path, usage) in &self.usage {
            if usage.percent_used > 95.0 {
                issues.push(StorageIssue {
                    severity: StorageIssueSeverity::Critical,
                    device: path.clone(),
                    description: format!("Filesystem {}% full", usage.percent_used as u8),
                    suggestion: "Free up space immediately".to_string(),
                });
            } else if usage.percent_used > 90.0 {
                issues.push(StorageIssue {
                    severity: StorageIssueSeverity::High,
                    device: path.clone(),
                    description: format!("Filesystem {}% full", usage.percent_used as u8),
                    suggestion: "Clean up unnecessary files".to_string(),
                });
            }

            if usage.percent_inodes > 90.0 {
                issues.push(StorageIssue {
                    severity: StorageIssueSeverity::High,
                    device: path.clone(),
                    description: format!("Inode usage at {}%", usage.percent_inodes as u8),
                    suggestion: "Remove small files or reformat with more inodes".to_string(),
                });
            }
        }

        // Check fstab issues
        for entry in &self.fstab {
            if !entry.mounted && entry.pass > 0 {
                issues.push(StorageIssue {
                    severity: StorageIssueSeverity::Medium,
                    device: entry.spec.clone(),
                    description: format!(
                        "fstab entry for {} not mounted",
                        entry.mount_point
                    ),
                    suggestion: "Check if device exists and mount manually".to_string(),
                });
            }

            for issue in &entry.issues {
                issues.push(StorageIssue {
                    severity: StorageIssueSeverity::Medium,
                    device: entry.spec.clone(),
                    description: issue.clone(),
                    suggestion: "Review fstab entry".to_string(),
                });
            }
        }

        issues
    }

    /// Get total storage capacity
    pub fn total_capacity(&self) -> u64 {
        self.devices
            .values()
            .filter(|d| d.device_type == DeviceType::Disk)
            .map(|d| d.size)
            .sum()
    }

    /// Get total used storage
    pub fn total_used(&self) -> u64 {
        self.usage.values().map(|u| u.used).sum()
    }
}

/// A diagnosed storage issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageIssue {
    pub severity: StorageIssueSeverity,
    pub device: String,
    pub description: String,
    pub suggestion: String,
}

/// Issue severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageIssueSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl BlockDevice {
    /// Create a new block device
    pub fn new(name: &str, path: &str, device_type: DeviceType) -> Self {
        Self {
            name: name.to_string(),
            path: path.to_string(),
            device_type,
            size: 0,
            model: None,
            serial: None,
            removable: false,
            read_only: false,
            rotational: true,
            partitions: Vec::new(),
            parent: None,
        }
    }
}

impl FilesystemUsage {
    /// Create from raw values
    pub fn new(mount_point: &str, total: u64, used: u64, available: u64) -> Self {
        let percent_used = if total > 0 {
            (used as f32 / total as f32) * 100.0
        } else {
            0.0
        };

        Self {
            mount_point: mount_point.to_string(),
            total,
            used,
            available,
            percent_used,
            percent_inodes: 0.0,
        }
    }
}

/// Parse human-readable size to bytes
pub fn parse_size(s: &str) -> Option<u64> {
    let s = s.trim().to_uppercase();
    let (num_str, multiplier) = if s.ends_with("TB") || s.ends_with('T') {
        (s.trim_end_matches(['T', 'B']), 1_099_511_627_776u64)
    } else if s.ends_with("GB") || s.ends_with('G') {
        (s.trim_end_matches(['G', 'B']), 1_073_741_824u64)
    } else if s.ends_with("MB") || s.ends_with('M') {
        (s.trim_end_matches(['M', 'B']), 1_048_576u64)
    } else if s.ends_with("KB") || s.ends_with('K') {
        (s.trim_end_matches(['K', 'B']), 1024u64)
    } else if s.ends_with('B') {
        (s.trim_end_matches('B'), 1u64)
    } else {
        (&s[..], 1u64)
    };

    num_str.trim().parse::<f64>().ok().map(|n| (n * multiplier as f64) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_size() {
        assert_eq!(parse_size("1G"), Some(1_073_741_824));
        assert_eq!(parse_size("500M"), Some(524_288_000));
        assert_eq!(parse_size("1TB"), Some(1_099_511_627_776));
        assert_eq!(parse_size("1024K"), Some(1_048_576));
    }

    #[test]
    fn test_filesystem_usage() {
        let usage = FilesystemUsage::new("/", 100_000_000_000, 50_000_000_000, 50_000_000_000);
        assert!((usage.percent_used - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_storage_diagnosis() {
        let mut model = StorageModel::new();

        // Add a nearly full filesystem
        model.usage.insert(
            "/".to_string(),
            FilesystemUsage {
                mount_point: "/".to_string(),
                total: 100_000_000_000,
                used: 96_000_000_000,
                available: 4_000_000_000,
                percent_used: 96.0,
                percent_inodes: 10.0,
            },
        );

        let issues = model.diagnose();
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| matches!(i.severity, StorageIssueSeverity::Critical)));
    }
}
