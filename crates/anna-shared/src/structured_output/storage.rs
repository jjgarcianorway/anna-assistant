//! Storage structured output - parse lsblk and findmnt JSON output.

use super::ParseResult;
use serde::{Deserialize, Serialize};

/// Block device from `lsblk -J`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDevice {
    /// Device name (e.g., "sda", "nvme0n1")
    pub name: String,
    /// Device path
    #[serde(default, rename = "path")]
    pub path: Option<String>,
    /// Size in bytes (as string)
    #[serde(default)]
    pub size: String,
    /// Device type (disk, part, lvm, etc.)
    #[serde(default, rename = "type")]
    pub device_type: String,
    /// Mount point
    #[serde(default)]
    pub mountpoint: Option<String>,
    /// Filesystem type
    #[serde(default)]
    pub fstype: Option<String>,
    /// Label
    #[serde(default)]
    pub label: Option<String>,
    /// UUID
    #[serde(default)]
    pub uuid: Option<String>,
    /// Model (for disks)
    #[serde(default)]
    pub model: Option<String>,
    /// Is removable
    #[serde(default)]
    pub rm: bool,
    /// Is read-only
    #[serde(default)]
    pub ro: bool,
    /// Rotational (HDD vs SSD)
    #[serde(default)]
    pub rota: bool,
    /// Child devices (partitions)
    #[serde(default)]
    pub children: Vec<Partition>,
}

impl BlockDevice {
    /// Get size as bytes (parsing human-readable size)
    pub fn size_bytes(&self) -> Option<u64> {
        parse_size(&self.size)
    }

    /// Check if this is an SSD
    pub fn is_ssd(&self) -> bool {
        !self.rota
    }

    /// Check if this is mounted
    pub fn is_mounted(&self) -> bool {
        self.mountpoint.is_some()
    }

    /// Get all partitions (flattened)
    pub fn all_partitions(&self) -> Vec<&Partition> {
        self.children.iter().collect()
    }
}

/// Partition from lsblk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Partition {
    /// Partition name
    pub name: String,
    /// Size
    #[serde(default)]
    pub size: String,
    /// Device type
    #[serde(default, rename = "type")]
    pub device_type: String,
    /// Mount point
    #[serde(default)]
    pub mountpoint: Option<String>,
    /// Filesystem type
    #[serde(default)]
    pub fstype: Option<String>,
    /// Label
    #[serde(default)]
    pub label: Option<String>,
    /// UUID
    #[serde(default)]
    pub uuid: Option<String>,
    /// Nested children (for LVM, etc.)
    #[serde(default)]
    pub children: Vec<Partition>,
}

impl Partition {
    /// Get size as bytes
    pub fn size_bytes(&self) -> Option<u64> {
        parse_size(&self.size)
    }

    /// Check if mounted
    pub fn is_mounted(&self) -> bool {
        self.mountpoint.is_some()
    }
}

/// Parse `lsblk -J` output
pub fn parse_lsblk_output(output: &str) -> ParseResult<Vec<BlockDevice>> {
    #[derive(Deserialize)]
    struct LsblkOutput {
        blockdevices: Vec<BlockDevice>,
    }

    match super::parse_json::<LsblkOutput>(output) {
        ParseResult::Ok(lsblk) => ParseResult::Ok(lsblk.blockdevices),
        ParseResult::RawText(t) => ParseResult::RawText(t),
        ParseResult::ParseError(e) => ParseResult::ParseError(e),
        ParseResult::CommandError(e) => ParseResult::CommandError(e),
    }
}

/// Parse human-readable size to bytes
fn parse_size(size: &str) -> Option<u64> {
    let size = size.trim().to_uppercase();

    // Try direct parse first
    if let Ok(bytes) = size.parse::<u64>() {
        return Some(bytes);
    }

    // Parse with suffix
    let (num_str, multiplier) = if size.ends_with("T") || size.ends_with("TB") {
        (size.trim_end_matches('T').trim_end_matches("TB"), 1024u64.pow(4))
    } else if size.ends_with("G") || size.ends_with("GB") {
        (size.trim_end_matches('G').trim_end_matches("GB"), 1024u64.pow(3))
    } else if size.ends_with("M") || size.ends_with("MB") {
        (size.trim_end_matches('M').trim_end_matches("MB"), 1024u64.pow(2))
    } else if size.ends_with("K") || size.ends_with("KB") {
        (size.trim_end_matches('K').trim_end_matches("KB"), 1024)
    } else if size.ends_with("B") {
        (size.trim_end_matches('B'), 1)
    } else {
        return None;
    };

    num_str.trim().parse::<f64>().ok().map(|n| (n * multiplier as f64) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_size() {
        assert_eq!(parse_size("1024"), Some(1024));
        assert_eq!(parse_size("1G"), Some(1024 * 1024 * 1024));
        assert_eq!(parse_size("500M"), Some(500 * 1024 * 1024));
        assert_eq!(parse_size("2T"), Some(2 * 1024 * 1024 * 1024 * 1024));
    }

    #[test]
    fn test_parse_lsblk() {
        let json = r#"{
            "blockdevices": [
                {
                    "name": "sda",
                    "size": "500G",
                    "type": "disk",
                    "rota": false,
                    "model": "Samsung SSD",
                    "children": [
                        {
                            "name": "sda1",
                            "size": "512M",
                            "type": "part",
                            "fstype": "vfat",
                            "mountpoint": "/boot"
                        }
                    ]
                }
            ]
        }"#;

        let result = parse_lsblk_output(json);
        assert!(result.is_ok());

        let devices = result.ok().unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].name, "sda");
        assert!(devices[0].is_ssd());
        assert_eq!(devices[0].children.len(), 1);
    }
}
