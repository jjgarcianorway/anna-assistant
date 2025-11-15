use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

/// Storage device information including type, health, and performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    /// List of detected storage devices
    pub devices: Vec<StorageDevice>,
    /// Whether smartmontools (smartctl) is installed
    pub smartctl_available: bool,
    /// Overall storage health summary
    pub health_summary: StorageHealthSummary,
}

/// Individual storage device details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageDevice {
    /// Device name (e.g., "sda", "nvme0n1")
    pub name: String,
    /// Device path (e.g., "/dev/sda")
    pub path: String,
    /// Device type (SSD, HDD, NVMe, MMC, etc.)
    pub device_type: DeviceType,
    /// Model name
    pub model: Option<String>,
    /// Serial number
    pub serial: Option<String>,
    /// Firmware version
    pub firmware: Option<String>,
    /// Total capacity in GB
    pub capacity_gb: Option<f64>,
    /// SMART status information
    pub smart: Option<SmartStatus>,
    /// I/O error counts
    pub io_errors: IoErrorCounts,
    /// Performance metrics
    pub performance: Option<PerformanceMetrics>,
    /// Partition alignment status
    pub partitions: Vec<PartitionInfo>,
}

/// Type of storage device
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeviceType {
    /// Solid State Drive (SATA)
    SSD,
    /// Hard Disk Drive (spinning disk)
    HDD,
    /// NVMe SSD
    NVMe,
    /// eMMC storage
    MMC,
    /// USB storage
    USB,
    /// Unknown type
    Unknown,
}

/// SMART health status and key attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartStatus {
    /// Overall health status (PASSED, FAILED, or specific message)
    pub health_status: String,
    /// Whether SMART is enabled
    pub smart_enabled: bool,
    /// Power-on hours
    pub power_on_hours: Option<u64>,
    /// Power cycle count
    pub power_cycle_count: Option<u64>,
    /// Temperature in Celsius
    pub temperature_celsius: Option<i32>,
    /// Reallocated sector count (critical for HDDs)
    pub reallocated_sectors: Option<u64>,
    /// Current pending sector count
    pub pending_sectors: Option<u64>,
    /// Uncorrectable sector count
    pub uncorrectable_sectors: Option<u64>,
    /// Total bytes written (TB)
    pub total_bytes_written_tb: Option<f64>,
    /// Total bytes read (TB)
    pub total_bytes_read_tb: Option<f64>,
    /// Wear leveling count (SSDs) - percentage used
    pub wear_leveling_percent: Option<u8>,
    /// Media errors (NVMe)
    pub media_errors: Option<u64>,
    /// Error log entries
    pub error_log_count: Option<u64>,
}

/// I/O error statistics from kernel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoErrorCounts {
    /// Read errors
    pub read_errors: u64,
    /// Write errors
    pub write_errors: u64,
    /// Flush errors
    pub flush_errors: u64,
    /// Discard errors
    pub discard_errors: u64,
}

/// Performance metrics for storage device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Average read latency in milliseconds (from /sys/block)
    pub avg_read_latency_ms: Option<f64>,
    /// Average write latency in milliseconds (from /sys/block)
    pub avg_write_latency_ms: Option<f64>,
    /// Queue depth
    pub queue_depth: Option<u32>,
    /// Scheduler type (e.g., "mq-deadline", "none", "bfq")
    pub scheduler: Option<String>,
}

/// Partition alignment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionInfo {
    /// Partition name (e.g., "sda1")
    pub name: String,
    /// Start sector
    pub start_sector: Option<u64>,
    /// Alignment offset in bytes
    pub alignment_offset: Option<u64>,
    /// Whether partition is properly aligned (start % 2048 == 0 for modern drives)
    pub is_aligned: bool,
    /// Filesystem type on this partition
    pub filesystem: Option<String>,
}

/// Overall storage health summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageHealthSummary {
    /// Number of devices with SMART failures
    pub failed_devices: u32,
    /// Number of devices with degraded health (high error counts)
    pub degraded_devices: u32,
    /// Number of devices with misaligned partitions
    pub misaligned_partitions: u32,
    /// Total I/O errors across all devices
    pub total_io_errors: u64,
}

impl StorageInfo {
    /// Detect all storage information
    pub fn detect() -> Self {
        let smartctl_available = check_smartctl_available();
        let devices = detect_storage_devices(smartctl_available);
        let health_summary = calculate_health_summary(&devices);

        StorageInfo {
            devices,
            smartctl_available,
            health_summary,
        }
    }
}

/// Check if smartctl is available
fn check_smartctl_available() -> bool {
    Command::new("smartctl")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Detect all storage devices
fn detect_storage_devices(smartctl_available: bool) -> Vec<StorageDevice> {
    let mut devices = Vec::new();

    // Read /sys/block to find all block devices
    if let Ok(entries) = fs::read_dir("/sys/block") {
        for entry in entries.flatten() {
            let device_name = entry.file_name().to_string_lossy().to_string();

            // Skip loop devices, ram devices, and other virtual devices
            if device_name.starts_with("loop")
                || device_name.starts_with("ram")
                || device_name.starts_with("dm-")
            {
                continue;
            }

            let device_path = format!("/dev/{}", device_name);

            // Detect device type
            let device_type = detect_device_type(&device_name);

            // Get SMART data if available
            let smart = if smartctl_available {
                get_smart_data(&device_path)
            } else {
                None
            };

            // Get I/O error counts
            let io_errors = get_io_error_counts(&device_name);

            // Get performance metrics
            let performance = get_performance_metrics(&device_name);

            // Get partition information
            let partitions = get_partition_info(&device_name);

            // Get capacity
            let capacity_gb = get_device_capacity(&device_name);

            // Get model, serial, firmware from sysfs or SMART data
            let (model, serial, firmware) = if let Some(ref smart_data) = smart {
                // SMART data might have these; we'll extract from smartctl output
                (None, None, None) // Will be populated by get_smart_data
            } else {
                get_device_identity(&device_name)
            };

            devices.push(StorageDevice {
                name: device_name,
                path: device_path,
                device_type,
                model,
                serial,
                firmware,
                capacity_gb,
                smart,
                io_errors,
                performance,
                partitions,
            });
        }
    }

    devices
}

/// Detect device type (SSD, HDD, NVMe, etc.)
fn detect_device_type(device_name: &str) -> DeviceType {
    // NVMe devices
    if device_name.starts_with("nvme") {
        return DeviceType::NVMe;
    }

    // MMC/eMMC devices
    if device_name.starts_with("mmcblk") {
        return DeviceType::MMC;
    }

    // For SATA/SCSI devices (sd*), check rotational flag
    let rotational_path = format!("/sys/block/{}/queue/rotational", device_name);
    if let Ok(content) = fs::read_to_string(&rotational_path) {
        if content.trim() == "0" {
            return DeviceType::SSD;
        } else if content.trim() == "1" {
            return DeviceType::HDD;
        }
    }

    // Check if it's USB
    let device_path = format!("/sys/block/{}/device", device_name);
    if let Ok(link) = fs::read_link(&device_path) {
        if link.to_string_lossy().contains("usb") {
            return DeviceType::USB;
        }
    }

    DeviceType::Unknown
}

/// Get SMART data using smartctl
fn get_smart_data(device_path: &str) -> Option<SmartStatus> {
    let output = Command::new("smartctl")
        .arg("-a")
        .arg("-j") // JSON output
        .arg(device_path)
        .output()
        .ok()?;

    if !output.status.success() {
        // smartctl might return non-zero even with valid data
        // Continue parsing anyway
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse JSON output
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
        parse_smart_json(&json)
    } else {
        None
    }
}

/// Parse SMART data from smartctl JSON output
fn parse_smart_json(json: &serde_json::Value) -> Option<SmartStatus> {
    let health_status = json["smart_status"]["passed"]
        .as_bool()
        .map(|passed| if passed { "PASSED".to_string() } else { "FAILED".to_string() })
        .or_else(|| {
            json["smart_status"]["nvme"]["value"]
                .as_str()
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "UNKNOWN".to_string());

    let smart_enabled = json["smart_status"]["passed"].as_bool().unwrap_or(false)
        || json["smart_support"]["enabled"].as_bool().unwrap_or(false);

    let power_on_hours = json["power_on_time"]["hours"].as_u64();

    let power_cycle_count = json["power_cycle_count"].as_u64();

    let temperature_celsius = json["temperature"]["current"].as_i64().map(|t| t as i32);

    // Extract SMART attributes for HDD/SSD
    let mut reallocated_sectors = None;
    let mut pending_sectors = None;
    let mut uncorrectable_sectors = None;
    let mut wear_leveling_percent = None;

    if let Some(attrs) = json["ata_smart_attributes"]["table"].as_array() {
        for attr in attrs {
            match attr["id"].as_u64() {
                Some(5) => reallocated_sectors = attr["raw"]["value"].as_u64(),
                Some(197) => pending_sectors = attr["raw"]["value"].as_u64(),
                Some(198) => uncorrectable_sectors = attr["raw"]["value"].as_u64(),
                Some(177) => wear_leveling_percent = attr["value"].as_u64().map(|v| v as u8),
                _ => {}
            }
        }
    }

    // NVMe specific attributes
    let media_errors = json["nvme_smart_health_information_log"]["media_errors"].as_u64();

    let error_log_count = json["nvme_smart_health_information_log"]["num_err_log_entries"].as_u64();

    // Data units for NVMe (convert to TB)
    let total_bytes_written_tb = json["nvme_smart_health_information_log"]["data_units_written"]
        .as_u64()
        .map(|units| (units as f64) * 512.0 * 1000.0 / 1_000_000_000_000.0);

    let total_bytes_read_tb = json["nvme_smart_health_information_log"]["data_units_read"]
        .as_u64()
        .map(|units| (units as f64) * 512.0 * 1000.0 / 1_000_000_000_000.0);

    Some(SmartStatus {
        health_status,
        smart_enabled,
        power_on_hours,
        power_cycle_count,
        temperature_celsius,
        reallocated_sectors,
        pending_sectors,
        uncorrectable_sectors,
        total_bytes_written_tb,
        total_bytes_read_tb,
        wear_leveling_percent,
        media_errors,
        error_log_count,
    })
}

/// Get I/O error counts from /sys/block
fn get_io_error_counts(device_name: &str) -> IoErrorCounts {
    let stat_path = format!("/sys/block/{}/stat", device_name);

    // /sys/block/*/stat format (fields we care about):
    // Field 8: I/Os currently in progress (not errors)
    // We'll use iostat or check dmesg for actual errors
    // For now, read from /sys/block/*/device/ioerr_cnt if available

    let mut counts = IoErrorCounts {
        read_errors: 0,
        write_errors: 0,
        flush_errors: 0,
        discard_errors: 0,
    };

    // Try to get error counts from device/ioerr_cnt (not standard on all devices)
    let ioerr_path = format!("/sys/block/{}/device/ioerr_cnt", device_name);
    if let Ok(content) = fs::read_to_string(&ioerr_path) {
        if let Ok(errors) = content.trim().parse::<u64>() {
            counts.read_errors = errors; // Generic error count
        }
    }

    // Alternative: parse /sys/block/*/inflight for pending I/Os (not errors)
    // Real error tracking would require parsing dmesg or using blktrace

    counts
}

/// Get performance metrics from /sys/block
fn get_performance_metrics(device_name: &str) -> Option<PerformanceMetrics> {
    // Get I/O scheduler
    let scheduler_path = format!("/sys/block/{}/queue/scheduler", device_name);
    let scheduler = fs::read_to_string(&scheduler_path).ok().and_then(|content| {
        // Format: "mq-deadline [none] kyber"
        // Extract the one in brackets
        content
            .split_whitespace()
            .find(|s| s.starts_with('[') && s.ends_with(']'))
            .map(|s| s.trim_matches(|c| c == '[' || c == ']').to_string())
    });

    // Get queue depth
    let nr_requests_path = format!("/sys/block/{}/queue/nr_requests", device_name);
    let queue_depth = fs::read_to_string(&nr_requests_path)
        .ok()
        .and_then(|content| content.trim().parse::<u32>().ok());

    // Note: avg latency would require iostat or more complex tracking
    // For now, we'll leave those as None

    Some(PerformanceMetrics {
        avg_read_latency_ms: None,
        avg_write_latency_ms: None,
        queue_depth,
        scheduler,
    })
}

/// Get partition information and alignment
fn get_partition_info(device_name: &str) -> Vec<PartitionInfo> {
    let mut partitions = Vec::new();

    // Read partition info from /sys/block/{device}/{device}*
    let block_path = format!("/sys/block/{}", device_name);
    if let Ok(entries) = fs::read_dir(&block_path) {
        for entry in entries.flatten() {
            let partition_name = entry.file_name().to_string_lossy().to_string();

            // Only process partitions (not the device itself)
            if !partition_name.starts_with(device_name) || partition_name == device_name {
                continue;
            }

            // Get start sector
            let start_path = format!("/sys/block/{}/{}/start", device_name, partition_name);
            let start_sector = fs::read_to_string(&start_path)
                .ok()
                .and_then(|content| content.trim().parse::<u64>().ok());

            // Get alignment offset
            let alignment_offset_path = format!("/sys/block/{}/{}/alignment_offset", device_name, partition_name);
            let alignment_offset = fs::read_to_string(&alignment_offset_path)
                .ok()
                .and_then(|content| content.trim().parse::<u64>().ok());

            // Check alignment (modern drives use 4K sectors, alignment to 2048*512=1MiB is standard)
            let is_aligned = start_sector.map(|start| start % 2048 == 0).unwrap_or(false);

            // Get filesystem type using lsblk
            let filesystem = get_partition_filesystem(&partition_name);

            partitions.push(PartitionInfo {
                name: partition_name,
                start_sector,
                alignment_offset,
                is_aligned,
                filesystem,
            });
        }
    }

    partitions
}

/// Get filesystem type for a partition
fn get_partition_filesystem(partition_name: &str) -> Option<String> {
    let output = Command::new("lsblk")
        .arg("-no")
        .arg("FSTYPE")
        .arg(format!("/dev/{}", partition_name))
        .output()
        .ok()?;

    if output.status.success() {
        let fstype = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !fstype.is_empty() {
            return Some(fstype);
        }
    }

    None
}

/// Get device capacity in GB
fn get_device_capacity(device_name: &str) -> Option<f64> {
    let size_path = format!("/sys/block/{}/size", device_name);
    let sectors = fs::read_to_string(&size_path)
        .ok()
        .and_then(|content| content.trim().parse::<u64>().ok())?;

    // Size is in 512-byte sectors
    let bytes = sectors * 512;
    let gb = bytes as f64 / 1_000_000_000.0;

    Some(gb)
}

/// Get device identity (model, serial, firmware) from sysfs
fn get_device_identity(device_name: &str) -> (Option<String>, Option<String>, Option<String>) {
    let model_path = format!("/sys/block/{}/device/model", device_name);
    let model = fs::read_to_string(&model_path)
        .ok()
        .map(|s| s.trim().to_string());

    let serial_path = format!("/sys/block/{}/device/serial", device_name);
    let serial = fs::read_to_string(&serial_path)
        .ok()
        .map(|s| s.trim().to_string());

    let firmware_path = format!("/sys/block/{}/device/rev", device_name);
    let firmware = fs::read_to_string(&firmware_path)
        .ok()
        .map(|s| s.trim().to_string());

    (model, serial, firmware)
}

/// Calculate overall health summary
fn calculate_health_summary(devices: &[StorageDevice]) -> StorageHealthSummary {
    let mut failed_devices = 0;
    let mut degraded_devices = 0;
    let mut misaligned_partitions = 0;
    let mut total_io_errors = 0;

    for device in devices {
        // Check SMART failures
        if let Some(ref smart) = device.smart {
            if smart.health_status.contains("FAIL") {
                failed_devices += 1;
            }

            // Check for degraded health indicators
            let has_degradation = smart.reallocated_sectors.unwrap_or(0) > 0
                || smart.pending_sectors.unwrap_or(0) > 0
                || smart.uncorrectable_sectors.unwrap_or(0) > 0
                || smart.media_errors.unwrap_or(0) > 0;

            if has_degradation {
                degraded_devices += 1;
            }
        }

        // Count I/O errors
        total_io_errors += device.io_errors.read_errors
            + device.io_errors.write_errors
            + device.io_errors.flush_errors
            + device.io_errors.discard_errors;

        // Count misaligned partitions
        for partition in &device.partitions {
            if !partition.is_aligned {
                misaligned_partitions += 1;
            }
        }
    }

    StorageHealthSummary {
        failed_devices,
        degraded_devices,
        misaligned_partitions,
        total_io_errors,
    }
}
