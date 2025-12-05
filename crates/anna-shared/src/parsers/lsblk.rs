//! Parser for lsblk command output.
//!
//! Parses block device information to typed structs.
//! Handles tree structure output and multiple mountpoints.

use super::atoms::{parse_size, ParseError, ParseErrorReason};
use serde::{Deserialize, Serialize};

/// Block device type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockDeviceType {
    Disk,
    Part,
    Lvm,
    Crypt,
    Loop,
    Rom,
    Unknown,
}

impl BlockDeviceType {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "disk" => Self::Disk,
            "part" => Self::Part,
            "lvm" => Self::Lvm,
            "crypt" => Self::Crypt,
            "loop" => Self::Loop,
            "rom" => Self::Rom,
            _ => Self::Unknown,
        }
    }
}

/// Parsed block device entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDevice {
    /// Device name (e.g., "nvme0n1p1")
    pub name: String,
    /// Size in bytes
    pub size_bytes: u64,
    /// Device type
    pub device_type: BlockDeviceType,
    /// Mount points (can be multiple for btrfs subvolumes)
    pub mountpoints: Vec<String>,
    /// Parent device name (for partitions)
    pub parent: Option<String>,
    /// Read-only flag
    pub read_only: bool,
}

/// Parse lsblk output to structured block device list.
///
/// Expected format (from `lsblk` or `lsblk -o NAME,SIZE,TYPE,MOUNTPOINTS`):
/// ```text
/// NAME        MAJ:MIN RM   SIZE RO TYPE MOUNTPOINTS
/// nvme0n1     259:0    0 953.9G  0 disk
/// ├─nvme0n1p1 259:1    0   100M  0 part
/// └─nvme0n1p6 259:6    0 802.1G  0 part /
/// ```
pub fn parse_lsblk(probe_id: &str, output: &str) -> Result<Vec<BlockDevice>, ParseError> {
    let mut devices: Vec<BlockDevice> = Vec::new();
    let mut current_parent: Option<String> = None;
    let lines: Vec<&str> = output.lines().collect();

    if lines.is_empty() {
        return Err(ParseError::new(
            probe_id,
            ParseErrorReason::MissingSection("empty output".to_string()),
            output,
        ));
    }

    // Find header line and column positions
    let header_idx = lines.iter().position(|l| l.contains("NAME") && l.contains("TYPE"));
    let header = match header_idx {
        Some(idx) => lines[idx],
        None => {
            return Err(ParseError::new(
                probe_id,
                ParseErrorReason::MissingSection("no header line found".to_string()),
                output,
            ))
        }
    };

    // Parse column positions from header
    let name_col = header.find("NAME").unwrap_or(0);
    let size_col = header.find("SIZE");
    let ro_col = header.find("RO");
    let type_col = header.find("TYPE");
    let mount_col = header.find("MOUNTPOINTS").or_else(|| header.find("MOUNTPOINT"));

    // Parse data lines
    for (line_num, line) in lines.iter().enumerate().skip(header_idx.unwrap_or(0) + 1) {
        if line.trim().is_empty() {
            continue;
        }

        // Check if this is a continuation line (additional mountpoint)
        let trimmed = line.trim();
        if trimmed.starts_with('/') && !devices.is_empty() {
            // This is an additional mountpoint for the previous device
            if let Some(last) = devices.last_mut() {
                last.mountpoints.push(trimmed.to_string());
            }
            continue;
        }

        // Parse device entry
        match parse_device_line(line, name_col, size_col, ro_col, type_col, mount_col, &current_parent) {
            Ok(device) => {
                // Update parent tracking for tree structure
                if device.device_type == BlockDeviceType::Disk {
                    current_parent = Some(device.name.clone());
                }
                devices.push(device);
            }
            Err(reason) => {
                return Err(ParseError::new(probe_id, reason, line).with_line(line_num + 1));
            }
        }
    }

    Ok(devices)
}

/// Parse a single device line
fn parse_device_line(
    line: &str,
    _name_col: usize,
    _size_col: Option<usize>,
    _ro_col: Option<usize>,
    _type_col: Option<usize>,
    _mount_col: Option<usize>,
    current_parent: &Option<String>,
) -> Result<BlockDevice, ParseErrorReason> {
    // Extract name (strip tree characters)
    let name_start = line
        .char_indices()
        .find(|(_, c)| c.is_alphanumeric())
        .map(|(i, _)| i)
        .unwrap_or(0);

    let name_end = line[name_start..]
        .find(char::is_whitespace)
        .map(|i| name_start + i)
        .unwrap_or(line.len());

    let name = line[name_start..name_end].trim().to_string();
    if name.is_empty() {
        return Err(ParseErrorReason::MissingColumn(0));
    }

    // Determine if this is a child device (has tree characters)
    let is_child = line.starts_with("├") || line.starts_with("└") || line.starts_with("│");
    let parent = if is_child {
        current_parent.clone()
    } else {
        None
    };

    // Parse remaining fields by splitting on whitespace after the name
    let rest = &line[name_end..];
    let parts: Vec<&str> = rest.split_whitespace().collect();

    // We need at least: MAJ:MIN, RM, SIZE, RO, TYPE
    // Format: "259:0    0 953.9G  0 disk"
    if parts.len() < 5 {
        return Err(ParseErrorReason::MalformedRow);
    }

    // SIZE is typically at index 2 (after MAJ:MIN, RM)
    let size_str = parts.get(2).ok_or(ParseErrorReason::MissingColumn(2))?;
    let size_bytes = parse_size(size_str).unwrap_or(0);

    // RO is at index 3
    let ro_str = parts.get(3).ok_or(ParseErrorReason::MissingColumn(3))?;
    let read_only = *ro_str == "1";

    // TYPE is at index 4
    let type_str = parts.get(4).ok_or(ParseErrorReason::MissingColumn(4))?;
    let device_type = BlockDeviceType::from_str(type_str);

    // MOUNTPOINTS is everything after TYPE (can be empty or multiple)
    let mountpoints: Vec<String> = parts
        .get(5..)
        .map(|mp| mp.iter().map(|s| s.to_string()).collect())
        .unwrap_or_default();

    Ok(BlockDevice {
        name,
        size_bytes,
        device_type,
        mountpoints,
        parent,
        read_only,
    })
}

/// Find root filesystem device
pub fn find_root_device(devices: &[BlockDevice]) -> Option<&BlockDevice> {
    devices.iter().find(|d| d.mountpoints.contains(&"/".to_string()))
}

/// Get total disk size (sum of all disk-type devices)
pub fn total_disk_size(devices: &[BlockDevice]) -> u64 {
    devices
        .iter()
        .filter(|d| d.device_type == BlockDeviceType::Disk)
        .map(|d| d.size_bytes)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_OUTPUT: &str = r#"NAME        MAJ:MIN RM   SIZE RO TYPE MOUNTPOINTS
nvme0n1     259:0    0 953.9G  0 disk
├─nvme0n1p1 259:1    0   100M  0 part
├─nvme0n1p2 259:2    0    16M  0 part
├─nvme0n1p3 259:3    0   150G  0 part
├─nvme0n1p4 259:4    0   644M  0 part
├─nvme0n1p5 259:5    0     1G  0 part /boot/efi
└─nvme0n1p6 259:6    0 802.1G  0 part /
"#;

    #[test]
    fn golden_lsblk_parse_basic() {
        let devices = parse_lsblk("lsblk", SAMPLE_OUTPUT).unwrap();
        assert_eq!(devices.len(), 7);

        // Check disk device
        let disk = &devices[0];
        assert_eq!(disk.name, "nvme0n1");
        assert_eq!(disk.device_type, BlockDeviceType::Disk);
        assert!(disk.parent.is_none());
    }

    #[test]
    fn golden_lsblk_parse_partitions() {
        let devices = parse_lsblk("lsblk", SAMPLE_OUTPUT).unwrap();

        // Check partition with mountpoint
        let efi = devices.iter().find(|d| d.name == "nvme0n1p5").unwrap();
        assert_eq!(efi.device_type, BlockDeviceType::Part);
        assert_eq!(efi.mountpoints, vec!["/boot/efi"]);
        assert_eq!(efi.parent, Some("nvme0n1".to_string()));
    }

    #[test]
    fn golden_lsblk_find_root() {
        let devices = parse_lsblk("lsblk", SAMPLE_OUTPUT).unwrap();
        let root = find_root_device(&devices).unwrap();
        assert_eq!(root.name, "nvme0n1p6");
        assert!(root.mountpoints.contains(&"/".to_string()));
    }

    #[test]
    fn golden_lsblk_size_parsing() {
        let devices = parse_lsblk("lsblk", SAMPLE_OUTPUT).unwrap();

        // 953.9G should parse correctly
        let disk = &devices[0];
        assert!(disk.size_bytes > 900_000_000_000);
        assert!(disk.size_bytes < 1_100_000_000_000);

        // 100M should parse correctly
        let p1 = devices.iter().find(|d| d.name == "nvme0n1p1").unwrap();
        assert_eq!(p1.size_bytes, 104_857_600); // 100 * 1024 * 1024
    }

    #[test]
    fn golden_lsblk_multiple_mountpoints() {
        let output = r#"NAME        MAJ:MIN RM   SIZE RO TYPE MOUNTPOINTS
nvme0n1p6 259:6    0 802.1G  0 part /tmp/btrfs_root_3776
                                      /
"#;
        let devices = parse_lsblk("lsblk", output).unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].mountpoints.len(), 2);
        assert!(devices[0].mountpoints.contains(&"/".to_string()));
    }

    #[test]
    fn golden_lsblk_empty_output_error() {
        let result = parse_lsblk("lsblk", "");
        assert!(result.is_err());
    }

    #[test]
    fn golden_lsblk_no_header_error() {
        let result = parse_lsblk("lsblk", "some random text\nwithout header");
        assert!(result.is_err());
    }
}
