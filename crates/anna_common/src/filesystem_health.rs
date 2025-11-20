use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemHealth {
    pub ext4_status: Option<Ext4Status>,
    pub xfs_status: Option<XfsStatus>,
    pub zfs_status: Option<ZfsStatus>,
    pub detected_errors: Vec<FilesystemError>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ext4Status {
    pub filesystems: Vec<Ext4Filesystem>,
    pub total_errors: u32,
    pub needs_fsck: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ext4Filesystem {
    pub device: String,
    pub mount_point: Option<String>,
    pub last_checked: Option<String>,
    pub check_interval: Option<String>,
    pub error_count: u32,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XfsStatus {
    pub filesystems: Vec<XfsFilesystem>,
    pub total_log_errors: u32,
    pub corruptions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XfsFilesystem {
    pub device: String,
    pub mount_point: Option<String>,
    pub log_version: Option<String>,
    pub error_count: u32,
    pub metadata_errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZfsStatus {
    pub pools: Vec<ZfsPool>,
    pub total_pools: usize,
    pub degraded_pools: Vec<String>,
    pub scrub_needed: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZfsPool {
    pub name: String,
    pub state: String,
    pub status: String,
    pub errors_read: u64,
    pub errors_write: u64,
    pub errors_cksum: u64,
    pub last_scrub: Option<String>,
    pub scrub_in_progress: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemError {
    pub severity: ErrorSeverity,
    pub filesystem_type: String,
    pub device: String,
    pub message: String,
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Critical,
    Warning,
    Info,
}

impl FilesystemHealth {
    pub fn detect() -> Self {
        let mut health = FilesystemHealth {
            ext4_status: None,
            xfs_status: None,
            zfs_status: None,
            detected_errors: Vec::new(),
            recommendations: Vec::new(),
        };

        // Detect Ext4 filesystems
        if let Some(ext4) = detect_ext4_health() {
            if ext4.total_errors > 0 {
                health.detected_errors.push(FilesystemError {
                    severity: ErrorSeverity::Warning,
                    filesystem_type: "ext4".to_string(),
                    device: "multiple".to_string(),
                    message: format!("Ext4 filesystems have {} total errors", ext4.total_errors),
                    timestamp: None,
                });
            }
            if !ext4.needs_fsck.is_empty() {
                health
                    .recommendations
                    .push(format!("Run fsck on: {}", ext4.needs_fsck.join(", ")));
            }
            health.ext4_status = Some(ext4);
        }

        // Detect XFS filesystems
        if let Some(xfs) = detect_xfs_health() {
            if xfs.total_log_errors > 0 {
                health.detected_errors.push(FilesystemError {
                    severity: ErrorSeverity::Warning,
                    filesystem_type: "xfs".to_string(),
                    device: "multiple".to_string(),
                    message: format!("XFS filesystems have {} log errors", xfs.total_log_errors),
                    timestamp: None,
                });
            }
            if !xfs.corruptions.is_empty() {
                health.detected_errors.push(FilesystemError {
                    severity: ErrorSeverity::Critical,
                    filesystem_type: "xfs".to_string(),
                    device: "multiple".to_string(),
                    message: format!("XFS corruption detected on: {}", xfs.corruptions.join(", ")),
                    timestamp: None,
                });
                health
                    .recommendations
                    .push("Run xfs_repair on corrupted filesystems immediately".to_string());
            }
            health.xfs_status = Some(xfs);
        }

        // Detect ZFS pools
        if let Some(zfs) = detect_zfs_health() {
            if !zfs.degraded_pools.is_empty() {
                health.detected_errors.push(FilesystemError {
                    severity: ErrorSeverity::Critical,
                    filesystem_type: "zfs".to_string(),
                    device: "multiple".to_string(),
                    message: format!("Degraded ZFS pools: {}", zfs.degraded_pools.join(", ")),
                    timestamp: None,
                });
                health.recommendations.push(
                    "Check degraded ZFS pools immediately and replace failed drives".to_string(),
                );
            }
            if !zfs.scrub_needed.is_empty() {
                health
                    .recommendations
                    .push(format!("Run ZFS scrub on: {}", zfs.scrub_needed.join(", ")));
            }
            health.zfs_status = Some(zfs);
        }

        // Check dmesg for filesystem errors
        health.check_dmesg_errors();

        health
    }

    fn check_dmesg_errors(&mut self) {
        if let Ok(output) = Command::new("dmesg").arg("-T").output() {
            if let Ok(dmesg) = String::from_utf8(output.stdout) {
                let fs_error_patterns = vec![
                    ("ext4", vec!["EXT4-fs error", "ext4_error", "corruption"]),
                    ("xfs", vec!["XFS error", "xfs_error", "metadata corruption"]),
                    ("zfs", vec!["ZFS error", "zfs pool error", "checksum error"]),
                ];

                for (fs_type, patterns) in fs_error_patterns {
                    for line in dmesg.lines() {
                        for pattern in &patterns {
                            if line.to_lowercase().contains(&pattern.to_lowercase()) {
                                // Try to extract timestamp from dmesg -T output
                                let timestamp = line
                                    .split(']')
                                    .next()
                                    .and_then(|s| s.strip_prefix('['))
                                    .map(|s| s.to_string());

                                self.detected_errors.push(FilesystemError {
                                    severity: if pattern.contains("corruption") {
                                        ErrorSeverity::Critical
                                    } else {
                                        ErrorSeverity::Warning
                                    },
                                    filesystem_type: fs_type.to_string(),
                                    device: "unknown".to_string(),
                                    message: line.to_string(),
                                    timestamp,
                                });
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn has_critical_errors(&self) -> bool {
        self.detected_errors
            .iter()
            .any(|e| matches!(e.severity, ErrorSeverity::Critical))
    }

    pub fn health_score(&self) -> u8 {
        let mut score = 100u8;

        // Deduct for errors
        for error in &self.detected_errors {
            score = score.saturating_sub(match error.severity {
                ErrorSeverity::Critical => 30,
                ErrorSeverity::Warning => 10,
                ErrorSeverity::Info => 2,
            });
        }

        // Deduct for degraded ZFS pools
        if let Some(zfs) = &self.zfs_status {
            score = score.saturating_sub((zfs.degraded_pools.len() as u8) * 20);
        }

        // Deduct for filesystems needing fsck
        if let Some(ext4) = &self.ext4_status {
            score = score.saturating_sub((ext4.needs_fsck.len() as u8) * 5);
        }

        score
    }
}

fn detect_ext4_health() -> Option<Ext4Status> {
    let mut filesystems = Vec::new();
    let mut needs_fsck = Vec::new();

    // Get list of ext4 filesystems from /proc/mounts
    let ext4_devices = get_ext4_devices();

    for device in ext4_devices {
        if let Some(fs_info) = check_ext4_filesystem(&device) {
            if fs_info.error_count > 0 || fs_info.state.to_lowercase().contains("error") {
                needs_fsck.push(device.clone());
            }
            filesystems.push(fs_info);
        }
    }

    if filesystems.is_empty() {
        return None;
    }

    let total_errors: u32 = filesystems.iter().map(|fs| fs.error_count).sum();

    Some(Ext4Status {
        filesystems,
        total_errors,
        needs_fsck,
    })
}

fn get_ext4_devices() -> Vec<String> {
    let mut devices = Vec::new();

    if let Ok(mounts) = std::fs::read_to_string("/proc/mounts") {
        for line in mounts.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 && parts[2] == "ext4" {
                devices.push(parts[0].to_string());
            }
        }
    }

    devices
}

fn check_ext4_filesystem(device: &str) -> Option<Ext4Filesystem> {
    // Use tune2fs to get filesystem information
    let output = Command::new("tune2fs")
        .arg("-l")
        .arg(device)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let info = String::from_utf8(output.stdout).ok()?;
    let mut fs = Ext4Filesystem {
        device: device.to_string(),
        mount_point: get_mount_point(device),
        last_checked: None,
        check_interval: None,
        error_count: 0,
        state: "clean".to_string(),
    };

    for line in info.lines() {
        let line = line.trim();

        if line.starts_with("Last checked:") {
            fs.last_checked = Some(
                line.strip_prefix("Last checked:")
                    .unwrap_or("")
                    .trim()
                    .to_string(),
            );
        } else if line.starts_with("Check interval:") {
            fs.check_interval = Some(
                line.strip_prefix("Check interval:")
                    .unwrap_or("")
                    .trim()
                    .to_string(),
            );
        } else if line.starts_with("Filesystem state:") {
            fs.state = line
                .strip_prefix("Filesystem state:")
                .unwrap_or("unknown")
                .trim()
                .to_string();
        } else if line.starts_with("Filesystem errors:") || line.starts_with("FS Error count:") {
            if let Some(count_str) = line.split(':').nth(1) {
                fs.error_count = count_str.trim().parse().unwrap_or(0);
            }
        }
    }

    Some(fs)
}

fn detect_xfs_health() -> Option<XfsStatus> {
    let mut filesystems = Vec::new();
    let mut corruptions = Vec::new();

    // Get list of XFS filesystems from /proc/mounts
    let xfs_devices = get_xfs_devices();

    for device in xfs_devices {
        if let Some(fs_info) = check_xfs_filesystem(&device) {
            if !fs_info.metadata_errors.is_empty() {
                corruptions.push(device.clone());
            }
            filesystems.push(fs_info);
        }
    }

    if filesystems.is_empty() {
        return None;
    }

    let total_log_errors: u32 = filesystems.iter().map(|fs| fs.error_count).sum();

    Some(XfsStatus {
        filesystems,
        total_log_errors,
        corruptions,
    })
}

fn get_xfs_devices() -> Vec<String> {
    let mut devices = Vec::new();

    if let Ok(mounts) = std::fs::read_to_string("/proc/mounts") {
        for line in mounts.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 && parts[2] == "xfs" {
                devices.push(parts[0].to_string());
            }
        }
    }

    devices
}

fn check_xfs_filesystem(device: &str) -> Option<XfsFilesystem> {
    // Use xfs_info to get filesystem information
    let mount_point = get_mount_point(device)?;

    let output = Command::new("xfs_info").arg(&mount_point).output().ok()?;

    if !output.status.success() {
        return None;
    }

    let info = String::from_utf8(output.stdout).ok()?;
    let mut fs = XfsFilesystem {
        device: device.to_string(),
        mount_point: Some(mount_point.clone()),
        log_version: None,
        error_count: 0,
        metadata_errors: Vec::new(),
    };

    // Parse xfs_info output
    for line in info.lines() {
        if line.contains("log") && line.contains("version") {
            fs.log_version = Some(line.trim().to_string());
        }
    }

    // Check for errors in dmesg specific to this device
    if let Ok(output) = Command::new("dmesg").output() {
        if let Ok(dmesg) = String::from_utf8(output.stdout) {
            for line in dmesg.lines() {
                if line.contains(device)
                    && (line.contains("XFS") || line.contains("xfs"))
                    && (line.contains("error") || line.contains("corruption"))
                {
                    fs.error_count += 1;
                    fs.metadata_errors.push(line.to_string());
                }
            }
        }
    }

    Some(fs)
}

fn detect_zfs_health() -> Option<ZfsStatus> {
    // Check if zpool command exists
    if !Command::new("which")
        .arg("zpool")
        .output()
        .ok()?
        .status
        .success()
    {
        return None;
    }

    let output = Command::new("zpool").arg("list").arg("-H").output().ok()?;

    if !output.status.success() {
        return None;
    }

    let list = String::from_utf8(output.stdout).ok()?;
    let mut pools = Vec::new();
    let mut degraded_pools = Vec::new();
    let mut scrub_needed = Vec::new();

    for line in list.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if let Some(pool_name) = parts.first() {
            if let Some(pool_info) = check_zfs_pool(pool_name) {
                if pool_info.state.to_lowercase().contains("degrad")
                    || pool_info.state.to_lowercase().contains("unavail")
                    || pool_info.state.to_lowercase().contains("faulted")
                {
                    degraded_pools.push(pool_name.to_string());
                }

                if pool_info.last_scrub.is_none()
                    || pool_info.errors_read > 0
                    || pool_info.errors_write > 0
                    || pool_info.errors_cksum > 0
                {
                    scrub_needed.push(pool_name.to_string());
                }

                pools.push(pool_info);
            }
        }
    }

    if pools.is_empty() {
        return None;
    }

    Some(ZfsStatus {
        total_pools: pools.len(),
        pools,
        degraded_pools,
        scrub_needed,
    })
}

fn check_zfs_pool(pool_name: &str) -> Option<ZfsPool> {
    let output = Command::new("zpool")
        .arg("status")
        .arg(pool_name)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let status_text = String::from_utf8(output.stdout).ok()?;

    let mut pool = ZfsPool {
        name: pool_name.to_string(),
        state: "UNKNOWN".to_string(),
        status: String::new(),
        errors_read: 0,
        errors_write: 0,
        errors_cksum: 0,
        last_scrub: None,
        scrub_in_progress: false,
    };

    for line in status_text.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("state:") {
            pool.state = trimmed
                .strip_prefix("state:")
                .unwrap_or("")
                .trim()
                .to_string();
        } else if trimmed.starts_with("status:") {
            pool.status = trimmed
                .strip_prefix("status:")
                .unwrap_or("")
                .trim()
                .to_string();
        } else if trimmed.starts_with("errors:") {
            let error_info = trimmed.strip_prefix("errors:").unwrap_or("").trim();
            // Parse error counts if in "No known data errors" format or actual counts
            if !error_info.contains("No known") {
                pool.status.push_str(&format!(" | Errors: {}", error_info));
            }
        } else if trimmed.contains("scan:") || trimmed.contains("scrub") {
            if trimmed.contains("in progress") {
                pool.scrub_in_progress = true;
                pool.last_scrub = Some("In progress".to_string());
            } else if trimmed.contains("scrub repaired") || trimmed.contains("completed") {
                // Extract scrub completion time
                pool.last_scrub = Some(trimmed.to_string());
            }
        }

        // Parse error columns from device listing
        if trimmed.contains(pool_name) && trimmed.split_whitespace().count() >= 5 {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 5 {
                pool.errors_read = parts[parts.len() - 3].parse().unwrap_or(0);
                pool.errors_write = parts[parts.len() - 2].parse().unwrap_or(0);
                pool.errors_cksum = parts[parts.len() - 1].parse().unwrap_or(0);
            }
        }
    }

    Some(pool)
}

fn get_mount_point(device: &str) -> Option<String> {
    if let Ok(mounts) = std::fs::read_to_string("/proc/mounts") {
        for line in mounts.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 && parts[0] == device {
                return Some(parts[1].to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filesystem_health_detection() {
        let health = FilesystemHealth::detect();
        // Should not panic
        assert!(health.health_score() <= 100);
    }

    #[test]
    fn test_health_score_calculation() {
        let health = FilesystemHealth {
            ext4_status: None,
            xfs_status: None,
            zfs_status: None,
            detected_errors: vec![FilesystemError {
                severity: ErrorSeverity::Warning,
                filesystem_type: "ext4".to_string(),
                device: "/dev/sda1".to_string(),
                message: "Test error".to_string(),
                timestamp: None,
            }],
            recommendations: Vec::new(),
        };

        let score = health.health_score();
        assert_eq!(score, 90); // 100 - 10 for warning
    }
}
