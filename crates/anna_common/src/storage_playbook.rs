//! Storage Department Playbook v0.0.67
//!
//! Evidence-based storage diagnosis with human-readable summaries.
//! - storage_mount_summary: Mount points and usage
//! - storage_btrfs_summary: BTRFS-specific health
//! - storage_smart_summary: Drive SMART health
//! - storage_recent_errors: Recent I/O errors
//! - storage_fstab_summary: fstab configuration
//!
//! Each tool returns PlaybookEvidence with:
//! - summary_human: No tool names, no IDs
//! - summary_debug: With tool names and details
//! - raw_refs: Commands used (debug only)

use crate::evidence_playbook::{
    PlaybookTopic, PlaybookEvidence, PlaybookBundle, StorageRiskLevel, StorageDiagnosis,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::process::Command;
use std::time::Instant;

// ============================================================================
// Evidence Collection Types
// ============================================================================

/// Mount point evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountEvidence {
    pub mount_point: String,
    pub device: String,
    pub fs_type: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub avail_bytes: u64,
    pub use_percent: u8,
}

/// BTRFS health evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtrfsEvidence {
    pub is_btrfs: bool,
    pub mount_point: String,
    pub device_errors: Vec<BtrfsDeviceError>,
    pub data_profile: Option<String>,
    pub metadata_profile: Option<String>,
    pub scrub_status: Option<String>,
    pub balance_status: Option<String>,
}

/// BTRFS device error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtrfsDeviceError {
    pub device: String,
    pub write_errors: u64,
    pub read_errors: u64,
    pub flush_errors: u64,
    pub corruption_errors: u64,
    pub generation_errors: u64,
}

/// SMART health evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartEvidence {
    pub device: String,
    pub smart_supported: bool,
    pub smart_enabled: bool,
    pub overall_health: Option<String>,  // PASSED, FAILED
    pub temperature_c: Option<u32>,
    pub power_on_hours: Option<u64>,
    pub reallocated_sectors: Option<u64>,
    pub pending_sectors: Option<u64>,
}

/// Recent I/O errors evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoErrorsEvidence {
    pub error_count: usize,
    pub recent_errors: Vec<String>,
    pub affected_devices: Vec<String>,
    pub time_range_minutes: u32,
}

/// fstab configuration evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FstabEvidence {
    pub entries: Vec<FstabEntry>,
    pub has_btrfs: bool,
    pub has_compression: bool,
    pub has_autodefrag: bool,
}

/// Single fstab entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FstabEntry {
    pub device: String,
    pub mount_point: String,
    pub fs_type: String,
    pub options: String,
}

// ============================================================================
// Playbook Topics (What to check)
// ============================================================================

/// Get the storage playbook topics
pub fn storage_topics() -> Vec<PlaybookTopic> {
    vec![
        PlaybookTopic::required(
            "storage_mount",
            "Mount Points & Disk Usage",
            "Check available space and mounted filesystems",
            vec!["df -h", "mount"],
        ),
        PlaybookTopic::required(
            "storage_btrfs",
            "BTRFS Health",
            "BTRFS device stats, scrub status, balance status",
            vec!["btrfs device stats", "btrfs scrub status"],
        ),
        PlaybookTopic::optional(
            "storage_smart",
            "Drive SMART Health",
            "Check drive health via SMART data",
            vec!["smartctl -a"],
        ),
        PlaybookTopic::optional(
            "storage_errors",
            "Recent I/O Errors",
            "Check journal for I/O errors",
            vec!["journalctl -k -p err"],
        ),
        PlaybookTopic::optional(
            "storage_fstab",
            "fstab Configuration",
            "Verify mount configuration",
            vec!["cat /etc/fstab"],
        ),
    ]
}

// ============================================================================
// Evidence Collection Functions
// ============================================================================

/// Collect mount point evidence
pub fn collect_mount_evidence() -> (Vec<MountEvidence>, PlaybookEvidence) {
    let start = Instant::now();
    let mut mounts = Vec::new();
    let mut low_space_mounts = Vec::new();

    if let Ok(output) = Command::new("df")
        .args(["-B1", "--output=target,source,fstype,size,used,avail,pcent"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 7 {
                continue;
            }

            // Skip virtual filesystems
            let fs_type = parts[2];
            if fs_type == "tmpfs" || fs_type == "devtmpfs" || fs_type == "efivarfs"
                || fs_type == "squashfs" || parts[0].starts_with("/dev/loop")
            {
                continue;
            }

            let use_pct: u8 = parts[6].trim_end_matches('%').parse().unwrap_or(0);

            let mount = MountEvidence {
                mount_point: parts[0].to_string(),
                device: parts[1].to_string(),
                fs_type: fs_type.to_string(),
                total_bytes: parts[3].parse().unwrap_or(0),
                used_bytes: parts[4].parse().unwrap_or(0),
                avail_bytes: parts[5].parse().unwrap_or(0),
                use_percent: use_pct,
            };

            if use_pct >= 90 {
                low_space_mounts.push(mount.mount_point.clone());
            }

            mounts.push(mount);
        }
    }

    let duration = start.elapsed().as_millis() as u64;

    // Human summary
    let human = if !low_space_mounts.is_empty() {
        format!("{} filesystem(s) running low on space", low_space_mounts.len())
    } else if mounts.is_empty() {
        "No filesystems found".to_string()
    } else {
        format!("{} filesystem(s) mounted, space OK", mounts.len())
    };

    // Debug summary
    let debug = format!(
        "{} mounts, low space: {:?}",
        mounts.len(),
        low_space_mounts
    );

    let evidence = PlaybookEvidence::success("storage_mount", &human, &debug)
        .with_refs(vec!["df -h".to_string()])
        .with_duration(duration);

    (mounts, evidence)
}

/// Collect BTRFS health evidence
pub fn collect_btrfs_evidence() -> (Option<BtrfsEvidence>, PlaybookEvidence) {
    let start = Instant::now();

    // Find BTRFS mount points
    let btrfs_mounts: Vec<String> = fs::read_to_string("/proc/mounts")
        .unwrap_or_default()
        .lines()
        .filter(|l| l.contains(" btrfs "))
        .filter_map(|l| l.split_whitespace().nth(1).map(|s| s.to_string()))
        .collect();

    if btrfs_mounts.is_empty() {
        let duration = start.elapsed().as_millis() as u64;
        return (
            None,
            PlaybookEvidence::success(
                "storage_btrfs",
                "No BTRFS filesystems found",
                "No BTRFS mounts in /proc/mounts",
            )
            .with_duration(duration),
        );
    }

    let mount = &btrfs_mounts[0];
    let mut device_errors = Vec::new();
    let mut has_errors = false;

    // Get device stats
    if let Ok(output) = Command::new("btrfs")
        .args(["device", "stats", mount])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut current_device = String::new();
        let mut write_err = 0u64;
        let mut read_err = 0u64;
        let mut flush_err = 0u64;
        let mut corruption_err = 0u64;
        let mut generation_err = 0u64;

        for line in stdout.lines() {
            if line.starts_with('[') {
                if !current_device.is_empty() {
                    let total = write_err + read_err + flush_err + corruption_err + generation_err;
                    if total > 0 {
                        has_errors = true;
                    }
                    device_errors.push(BtrfsDeviceError {
                        device: current_device.clone(),
                        write_errors: write_err,
                        read_errors: read_err,
                        flush_errors: flush_err,
                        corruption_errors: corruption_err,
                        generation_errors: generation_err,
                    });
                }
                current_device = line.trim_start_matches('[')
                    .split(']').next()
                    .unwrap_or("")
                    .to_string();
                write_err = 0;
                read_err = 0;
                flush_err = 0;
                corruption_err = 0;
                generation_err = 0;
            } else if line.contains(".write_io_errs") {
                write_err = extract_stat_value(line);
            } else if line.contains(".read_io_errs") {
                read_err = extract_stat_value(line);
            } else if line.contains(".flush_io_errs") {
                flush_err = extract_stat_value(line);
            } else if line.contains(".corruption_errs") {
                corruption_err = extract_stat_value(line);
            } else if line.contains(".generation_errs") {
                generation_err = extract_stat_value(line);
            }
        }

        if !current_device.is_empty() {
            let total = write_err + read_err + flush_err + corruption_err + generation_err;
            if total > 0 {
                has_errors = true;
            }
            device_errors.push(BtrfsDeviceError {
                device: current_device,
                write_errors: write_err,
                read_errors: read_err,
                flush_errors: flush_err,
                corruption_errors: corruption_err,
                generation_errors: generation_err,
            });
        }
    }

    // Get scrub status
    let scrub_status = Command::new("btrfs")
        .args(["scrub", "status", mount])
        .output()
        .ok()
        .map(|o| {
            let s = String::from_utf8_lossy(&o.stdout);
            if s.contains("no stats available") {
                "never run".to_string()
            } else if s.contains("finished") {
                "completed".to_string()
            } else if s.contains("running") {
                "running".to_string()
            } else {
                "unknown".to_string()
            }
        });

    let duration = start.elapsed().as_millis() as u64;

    let btrfs = BtrfsEvidence {
        is_btrfs: true,
        mount_point: mount.clone(),
        device_errors: device_errors.clone(),
        data_profile: None,
        metadata_profile: None,
        scrub_status: scrub_status.clone(),
        balance_status: None,
    };

    // Human summary
    let human = if has_errors {
        "BTRFS device errors detected - check immediately".to_string()
    } else {
        format!("BTRFS healthy on {}", mount)
    };

    // Debug summary
    let debug = format!(
        "BTRFS {}: {} devices, errors: {}, scrub: {:?}",
        mount,
        device_errors.len(),
        has_errors,
        scrub_status
    );

    let evidence = PlaybookEvidence::success("storage_btrfs", &human, &debug)
        .with_refs(vec![
            format!("btrfs device stats {}", mount),
            format!("btrfs scrub status {}", mount),
        ])
        .with_duration(duration);

    (Some(btrfs), evidence)
}

/// Collect SMART health evidence
pub fn collect_smart_evidence() -> (Vec<SmartEvidence>, PlaybookEvidence) {
    let start = Instant::now();
    let mut devices = Vec::new();
    let mut unhealthy = Vec::new();

    // Find block devices
    let block_devs: Vec<String> = fs::read_dir("/sys/block")
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.starts_with("sd") || n.starts_with("nvme"))
        .collect();

    for dev_name in block_devs {
        let dev_path = format!("/dev/{}", dev_name);

        // Try smartctl
        if let Ok(output) = Command::new("smartctl")
            .args(["-a", &dev_path])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);

            let smart_supported = stdout.contains("SMART support is: Available");
            let smart_enabled = stdout.contains("SMART support is: Enabled");

            let overall_health = if stdout.contains("PASSED") {
                Some("PASSED".to_string())
            } else if stdout.contains("FAILED") {
                Some("FAILED".to_string())
            } else {
                None
            };

            if overall_health.as_deref() == Some("FAILED") {
                unhealthy.push(dev_name.clone());
            }

            // Parse temperature
            let temperature_c = stdout.lines()
                .find(|l| l.contains("Temperature_Celsius") || l.contains("temperature"))
                .and_then(|l| {
                    l.split_whitespace()
                        .filter_map(|w| w.parse::<u32>().ok())
                        .find(|&t| t > 0 && t < 100)
                });

            // Parse power on hours
            let power_on_hours = stdout.lines()
                .find(|l| l.contains("Power_On_Hours"))
                .and_then(|l| {
                    l.split_whitespace().last().and_then(|w| w.parse().ok())
                });

            devices.push(SmartEvidence {
                device: dev_name,
                smart_supported,
                smart_enabled,
                overall_health,
                temperature_c,
                power_on_hours,
                reallocated_sectors: None,
                pending_sectors: None,
            });
        }
    }

    let duration = start.elapsed().as_millis() as u64;

    // Human summary
    let human = if !unhealthy.is_empty() {
        format!("SMART failure detected on {} drive(s) - backup immediately!", unhealthy.len())
    } else if devices.is_empty() {
        "No SMART-capable drives found".to_string()
    } else {
        format!("{} drive(s) healthy", devices.len())
    };

    // Debug summary
    let debug = format!(
        "{} drives checked, unhealthy: {:?}",
        devices.len(),
        unhealthy
    );

    let evidence = PlaybookEvidence::success("storage_smart", &human, &debug)
        .with_refs(vec!["smartctl -a /dev/sdX".to_string()])
        .with_duration(duration);

    (devices, evidence)
}

/// Collect recent I/O errors
pub fn collect_io_errors_evidence(minutes: u32) -> (IoErrorsEvidence, PlaybookEvidence) {
    let start = Instant::now();

    let mut error_count = 0;
    let mut recent_errors = Vec::new();
    let mut affected_devices: Vec<String> = Vec::new();

    // Check kernel messages for I/O errors
    if let Ok(output) = Command::new("journalctl")
        .args([
            "-k",
            "--since", &format!("{} minutes ago", minutes),
            "-p", "err",
            "--no-pager", "-q",
        ])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let lower = line.to_lowercase();
            if lower.contains("i/o error") || lower.contains("blk_update_request")
                || lower.contains("buffer i/o error") || lower.contains("ata")
            {
                error_count += 1;
                if recent_errors.len() < 5 {
                    recent_errors.push(line.to_string());
                }

                // Extract device name
                for dev in ["sd", "nvme", "mmcblk"] {
                    if let Some(idx) = line.find(dev) {
                        let dev_name: String = line[idx..]
                            .chars()
                            .take_while(|c| c.is_alphanumeric())
                            .collect();
                        if !affected_devices.contains(&dev_name) {
                            affected_devices.push(dev_name);
                        }
                    }
                }
            }
        }
    }

    let duration = start.elapsed().as_millis() as u64;

    let errors = IoErrorsEvidence {
        error_count,
        recent_errors: recent_errors.clone(),
        affected_devices: affected_devices.clone(),
        time_range_minutes: minutes,
    };

    // Human summary
    let human = if error_count == 0 {
        format!("No I/O errors in the last {} minutes", minutes)
    } else {
        format!("{} I/O error(s) detected - investigate affected drives", error_count)
    };

    // Debug summary
    let debug = format!(
        "{} I/O errors in {} min, devices: {:?}",
        error_count, minutes, affected_devices
    );

    let evidence = PlaybookEvidence::success("storage_errors", &human, &debug)
        .with_refs(vec![
            format!("journalctl -k --since '{} minutes ago' -p err", minutes),
        ])
        .with_duration(duration);

    (errors, evidence)
}

/// Collect fstab evidence
pub fn collect_fstab_evidence() -> (FstabEvidence, PlaybookEvidence) {
    let start = Instant::now();

    let mut entries = Vec::new();
    let mut has_btrfs = false;
    let mut has_compression = false;
    let mut has_autodefrag = false;

    if let Ok(content) = fs::read_to_string("/etc/fstab") {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 4 {
                continue;
            }

            let fs_type = parts[2];
            let options = parts[3];

            if fs_type == "btrfs" {
                has_btrfs = true;
                if options.contains("compress") {
                    has_compression = true;
                }
                if options.contains("autodefrag") {
                    has_autodefrag = true;
                }
            }

            entries.push(FstabEntry {
                device: parts[0].to_string(),
                mount_point: parts[1].to_string(),
                fs_type: fs_type.to_string(),
                options: options.to_string(),
            });
        }
    }

    let duration = start.elapsed().as_millis() as u64;

    let fstab = FstabEvidence {
        entries: entries.clone(),
        has_btrfs,
        has_compression,
        has_autodefrag,
    };

    // Human summary
    let human = format!(
        "{} fstab entries{}",
        entries.len(),
        if has_btrfs { " (BTRFS detected)" } else { "" }
    );

    // Debug summary
    let debug = format!(
        "{} entries, btrfs: {}, compress: {}, autodefrag: {}",
        entries.len(), has_btrfs, has_compression, has_autodefrag
    );

    let evidence = PlaybookEvidence::success("storage_fstab", &human, &debug)
        .with_refs(vec!["cat /etc/fstab".to_string()])
        .with_duration(duration);

    (fstab, evidence)
}

// ============================================================================
// Diagnosis Engine
// ============================================================================

/// Run the full storage diagnosis playbook
pub fn run_storage_playbook() -> StorageDiagnosis {
    let topics = storage_topics();
    let mut bundle = PlaybookBundle::new("storage");
    let mut findings = Vec::new();
    let mut risk_signals = Vec::new();
    let mut risk_level = StorageRiskLevel::None;

    // Collect all evidence
    let (mounts, mount_ev) = collect_mount_evidence();
    bundle.add(mount_ev);

    let (btrfs, btrfs_ev) = collect_btrfs_evidence();
    bundle.add(btrfs_ev);

    let (smart, smart_ev) = collect_smart_evidence();
    bundle.add(smart_ev);

    let (io_errors, errors_ev) = collect_io_errors_evidence(60);
    bundle.add(errors_ev);

    let (fstab, fstab_ev) = collect_fstab_evidence();
    bundle.add(fstab_ev);

    // Finalize coverage
    bundle.finalize(&topics);

    // Analyze findings and risk level

    // Check disk space
    for mount in &mounts {
        if mount.use_percent >= 95 {
            findings.push(format!("{} is {}% full - critical", mount.mount_point, mount.use_percent));
            risk_signals.push(format!("disk_full:{}", mount.mount_point));
            risk_level = StorageRiskLevel::High;
        } else if mount.use_percent >= 90 {
            findings.push(format!("{} is {}% full - warning", mount.mount_point, mount.use_percent));
            risk_signals.push(format!("disk_warning:{}", mount.mount_point));
            if risk_level == StorageRiskLevel::None {
                risk_level = StorageRiskLevel::Low;
            }
        }
    }

    // Check BTRFS health
    if let Some(ref btrfs) = btrfs {
        let total_errors: u64 = btrfs.device_errors.iter()
            .map(|e| e.write_errors + e.read_errors + e.flush_errors + e.corruption_errors)
            .sum();

        if total_errors > 0 {
            findings.push(format!("BTRFS device errors detected ({} total)", total_errors));
            risk_signals.push("btrfs_device_errors".to_string());
            risk_level = StorageRiskLevel::High;
        }
    }

    // Check SMART health
    for dev in &smart {
        if dev.overall_health.as_deref() == Some("FAILED") {
            findings.push(format!("SMART failure on {} - backup immediately!", dev.device));
            risk_signals.push(format!("smart_failed:{}", dev.device));
            risk_level = StorageRiskLevel::High;
        }
    }

    // Check I/O errors
    if io_errors.error_count > 0 {
        findings.push(format!("{} I/O errors in last hour", io_errors.error_count));
        risk_signals.push("io_errors".to_string());
        if risk_level != StorageRiskLevel::High {
            risk_level = StorageRiskLevel::Medium;
        }
    }

    // Calculate confidence based on coverage
    let confidence = bundle.coverage_score;

    StorageDiagnosis::new(bundle)
        .with_findings(findings)
        .with_risk(risk_level, risk_signals)
        .with_confidence(confidence)
}

// ============================================================================
// Helpers
// ============================================================================

fn extract_stat_value(line: &str) -> u64 {
    line.split_whitespace()
        .last()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_topics() {
        let topics = storage_topics();
        assert_eq!(topics.len(), 5);
        assert!(topics.iter().filter(|t| t.required).count() >= 2);
    }

    #[test]
    fn test_fstab_parsing() {
        // This is a unit test for the parsing logic
        let entry = FstabEntry {
            device: "UUID=abc123".to_string(),
            mount_point: "/".to_string(),
            fs_type: "btrfs".to_string(),
            options: "compress=zstd,autodefrag".to_string(),
        };

        assert!(entry.options.contains("compress"));
        assert!(entry.options.contains("autodefrag"));
    }

    #[test]
    fn test_btrfs_device_error() {
        let err = BtrfsDeviceError {
            device: "/dev/sda".to_string(),
            write_errors: 0,
            read_errors: 1,
            flush_errors: 0,
            corruption_errors: 0,
            generation_errors: 0,
        };

        let total = err.write_errors + err.read_errors + err.flush_errors
            + err.corruption_errors + err.generation_errors;
        assert_eq!(total, 1);
    }
}
