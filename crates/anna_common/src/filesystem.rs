//! Filesystem features detection
//!
//! Detects filesystem configuration and features:
//! - TRIM/discard support and status
//! - LUKS encryption detection
//! - Btrfs subvolumes and compression

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::process::Command;

/// TRIM/Discard support status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrimStatus {
    /// fstrim.timer is enabled and active
    TimerEnabled,
    /// fstrim.timer exists but is not enabled
    TimerDisabled,
    /// Continuous discard is enabled (mount option)
    ContinuousDiscard,
    /// Both timer and continuous discard
    Both,
    /// No TRIM support detected
    None,
    /// Unknown status
    Unknown,
}

/// Btrfs compression algorithm
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BtrfsCompression {
    /// No compression
    None,
    /// zlib compression
    Zlib,
    /// lzo compression
    Lzo,
    /// zstd compression (with optional level)
    Zstd(Option<u8>),
}

/// Filesystem features information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemInfo {
    /// TRIM/discard status
    pub trim_status: TrimStatus,
    /// Has encrypted devices (LUKS)
    pub has_luks_encryption: bool,
    /// List of encrypted devices
    pub encrypted_devices: Vec<String>,
    /// Btrfs is in use
    pub has_btrfs: bool,
    /// Btrfs subvolumes (if Btrfs is used)
    pub btrfs_subvolumes: Vec<BtrfsSubvolume>,
    /// Btrfs compression settings per mount point
    pub btrfs_compression: HashMap<String, BtrfsCompression>,
}

/// Btrfs subvolume information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtrfsSubvolume {
    /// Subvolume ID
    pub id: String,
    /// Subvolume path
    pub path: String,
}

impl FilesystemInfo {
    /// Detect filesystem features
    pub fn detect() -> Self {
        let trim_status = detect_trim_status();
        let (has_luks_encryption, encrypted_devices) = detect_luks_encryption();
        let has_btrfs = detect_btrfs();
        let btrfs_subvolumes = if has_btrfs {
            detect_btrfs_subvolumes()
        } else {
            Vec::new()
        };
        let btrfs_compression = if has_btrfs {
            detect_btrfs_compression()
        } else {
            HashMap::new()
        };

        Self {
            trim_status,
            has_luks_encryption,
            encrypted_devices,
            has_btrfs,
            btrfs_subvolumes,
            btrfs_compression,
        }
    }
}

/// Detect TRIM/discard status
fn detect_trim_status() -> TrimStatus {
    let timer_enabled = is_fstrim_timer_enabled();
    let continuous_discard = has_continuous_discard();

    match (timer_enabled, continuous_discard) {
        (true, true) => TrimStatus::Both,
        (true, false) => TrimStatus::TimerEnabled,
        (false, true) => TrimStatus::ContinuousDiscard,
        (false, false) => {
            // Check if timer exists but is disabled
            if is_fstrim_timer_available() {
                TrimStatus::TimerDisabled
            } else {
                TrimStatus::None
            }
        }
    }
}

/// Check if fstrim.timer is enabled
fn is_fstrim_timer_enabled() -> bool {
    if let Ok(output) = Command::new("systemctl")
        .arg("is-enabled")
        .arg("fstrim.timer")
        .output()
    {
        if output.status.success() {
            let status = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return status == "enabled";
        }
    }
    false
}

/// Check if fstrim.timer is available
fn is_fstrim_timer_available() -> bool {
    if let Ok(output) = Command::new("systemctl")
        .arg("list-unit-files")
        .arg("fstrim.timer")
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return stdout.contains("fstrim.timer");
        }
    }
    false
}

/// Check if any filesystems use continuous discard
fn has_continuous_discard() -> bool {
    // Check mount options for discard
    if let Ok(output) = Command::new("findmnt").arg("-rno").arg("OPTIONS").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return stdout
                .lines()
                .any(|line| line.contains("discard") && !line.contains("nodiscard"));
        }
    }
    false
}

/// Detect LUKS encryption
fn detect_luks_encryption() -> (bool, Vec<String>) {
    let mut encrypted_devices = Vec::new();

    // Method 1: Check lsblk for crypto_LUKS
    if let Ok(output) = Command::new("lsblk").arg("-no").arg("NAME,FSTYPE").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains("crypto_LUKS") {
                    if let Some(device) = line.split_whitespace().next() {
                        encrypted_devices.push(device.to_string());
                    }
                }
            }
        }
    }

    // Method 2: Check /proc/mounts for dm-crypt
    if let Ok(mounts) = fs::read_to_string("/proc/mounts") {
        for line in mounts.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let device = parts[0];
                if device.starts_with("/dev/mapper/") {
                    // Check if this is a dm-crypt device
                    if let Ok(output) = Command::new("dmsetup")
                        .arg("table")
                        .arg(device.trim_start_matches("/dev/mapper/"))
                        .output()
                    {
                        if output.status.success() {
                            let stdout = String::from_utf8_lossy(&output.stdout);
                            if stdout.contains("crypt") {
                                if !encrypted_devices.contains(&device.to_string()) {
                                    encrypted_devices.push(device.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let has_encryption = !encrypted_devices.is_empty();
    (has_encryption, encrypted_devices)
}

/// Detect if Btrfs is in use
fn detect_btrfs() -> bool {
    // Check if any mounted filesystem is btrfs
    if let Ok(output) = Command::new("findmnt")
        .arg("-t")
        .arg("btrfs")
        .arg("-no")
        .arg("TARGET")
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return !stdout.trim().is_empty();
        }
    }
    false
}

/// Detect Btrfs subvolumes
fn detect_btrfs_subvolumes() -> Vec<BtrfsSubvolume> {
    let mut subvolumes = Vec::new();

    // Try to list subvolumes for root
    if let Ok(output) = Command::new("btrfs")
        .arg("subvolume")
        .arg("list")
        .arg("/")
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                // Format: ID 256 gen 1234 top level 5 path @home
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 9 && parts[0] == "ID" {
                    let id = parts[1].to_string();
                    let path = parts[8..].join(" ");
                    subvolumes.push(BtrfsSubvolume { id, path });
                }
            }
        }
    }

    subvolumes
}

/// Detect Btrfs compression settings
fn detect_btrfs_compression() -> HashMap<String, BtrfsCompression> {
    let mut compression_map = HashMap::new();

    // Get all btrfs mount points and their options
    if let Ok(output) = Command::new("findmnt")
        .arg("-t")
        .arg("btrfs")
        .arg("-no")
        .arg("TARGET,OPTIONS")
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let mount_point = parts[0].to_string();
                    let options = parts[1];

                    // Parse compression option
                    let compression = parse_compression_option(options);
                    compression_map.insert(mount_point, compression);
                }
            }
        }
    }

    compression_map
}

/// Parse compression option from mount options string
fn parse_compression_option(options: &str) -> BtrfsCompression {
    for option in options.split(',') {
        if option == "compress" || option.starts_with("compress=") {
            if option == "compress" {
                // Default compression (zlib)
                return BtrfsCompression::Zlib;
            } else if let Some(algo) = option.strip_prefix("compress=") {
                if algo == "zlib" {
                    return BtrfsCompression::Zlib;
                } else if algo == "lzo" {
                    return BtrfsCompression::Lzo;
                } else if algo.starts_with("zstd") {
                    // Parse optional level (e.g., "zstd:3")
                    if let Some(level_str) = algo.strip_prefix("zstd:") {
                        if let Ok(level) = level_str.parse::<u8>() {
                            return BtrfsCompression::Zstd(Some(level));
                        }
                    }
                    return BtrfsCompression::Zstd(None);
                }
            }
        } else if option.starts_with("compress-force=") {
            if let Some(algo) = option.strip_prefix("compress-force=") {
                if algo == "zlib" {
                    return BtrfsCompression::Zlib;
                } else if algo == "lzo" {
                    return BtrfsCompression::Lzo;
                } else if algo.starts_with("zstd") {
                    if let Some(level_str) = algo.strip_prefix("zstd:") {
                        if let Ok(level) = level_str.parse::<u8>() {
                            return BtrfsCompression::Zstd(Some(level));
                        }
                    }
                    return BtrfsCompression::Zstd(None);
                }
            }
        }
    }

    BtrfsCompression::None
}

impl std::fmt::Display for TrimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrimStatus::TimerEnabled => write!(f, "Timer Enabled"),
            TrimStatus::TimerDisabled => write!(f, "Timer Disabled"),
            TrimStatus::ContinuousDiscard => write!(f, "Continuous Discard"),
            TrimStatus::Both => write!(f, "Timer + Continuous"),
            TrimStatus::None => write!(f, "None"),
            TrimStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

impl std::fmt::Display for BtrfsCompression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BtrfsCompression::None => write!(f, "None"),
            BtrfsCompression::Zlib => write!(f, "zlib"),
            BtrfsCompression::Lzo => write!(f, "lzo"),
            BtrfsCompression::Zstd(Some(level)) => write!(f, "zstd:{}", level),
            BtrfsCompression::Zstd(None) => write!(f, "zstd"),
        }
    }
}
