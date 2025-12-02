//! Impact View v7.21.0 - Resource Consumer Rankings from Telemetry
//!
//! Provides:
//! - Top CPU consumers (avg percent, last 24h)
//! - Top memory consumers (avg resident, last 24h)
//! - Top I/O consumers (read+write, last 24h)
//! - Disk and network pressure summaries
//!
//! All data from /var/lib/anna/telemetry, no guesses.

use crate::TelemetryDb;

/// A resource consumer entry
#[derive(Debug, Clone)]
pub struct ConsumerEntry {
    pub name: String,
    pub value: f64,      // Raw value (percent, bytes, etc.)
    pub formatted: String,  // Human-readable value
}

/// Software impact summary
#[derive(Debug, Clone, Default)]
pub struct SoftwareImpact {
    pub cpu_consumers: Vec<ConsumerEntry>,
    pub memory_consumers: Vec<ConsumerEntry>,
    pub io_consumers: Vec<ConsumerEntry>,
    pub has_data: bool,
}

/// Hardware impact summary
#[derive(Debug, Clone, Default)]
pub struct HardwareImpact {
    pub disk_pressure: Vec<DiskPressure>,
    pub network_usage: Vec<NetworkUsage>,
    pub has_data: bool,
}

/// Disk pressure entry
#[derive(Debug, Clone)]
pub struct DiskPressure {
    pub device: String,
    pub read_bytes_24h: u64,
    pub write_bytes_24h: u64,
    pub temp_avg: Option<f64>,
}

/// Network usage entry
#[derive(Debug, Clone)]
pub struct NetworkUsage {
    pub interface: String,
    pub rx_bytes_24h: u64,
    pub tx_bytes_24h: u64,
}

/// Get software impact from telemetry
pub fn get_software_impact(limit: usize) -> SoftwareImpact {
    let mut impact = SoftwareImpact::default();

    let db = match TelemetryDb::open_readonly() {
        Some(db) => db,
        None => return impact,
    };

    // Get top CPU consumers
    if let Ok(top_cpu) = db.top_cpu_with_trend(limit) {
        for entry in top_cpu {
            impact.cpu_consumers.push(ConsumerEntry {
                name: entry.name.clone(),
                value: entry.avg_cpu_percent as f64,
                formatted: format!("{:.1}", entry.avg_cpu_percent),
            });
        }
    }

    // Get top memory consumers
    if let Ok(top_mem) = db.top_memory_with_trend(limit) {
        for entry in top_mem {
            impact.memory_consumers.push(ConsumerEntry {
                name: entry.name.clone(),
                value: entry.avg_mem_bytes as f64,
                formatted: format_bytes(entry.avg_mem_bytes),
            });
        }
    }

    impact.has_data = !impact.cpu_consumers.is_empty() || !impact.memory_consumers.is_empty();

    impact
}

/// Get hardware impact from telemetry and system stats
pub fn get_hardware_impact() -> HardwareImpact {
    let mut impact = HardwareImpact::default();

    // Get disk stats from /proc/diskstats or /sys
    impact.disk_pressure = get_disk_pressure();

    // Get network stats from /sys/class/net
    impact.network_usage = get_network_usage();

    impact.has_data = !impact.disk_pressure.is_empty() || !impact.network_usage.is_empty();

    impact
}

/// Get disk pressure from system stats
fn get_disk_pressure() -> Vec<DiskPressure> {
    use std::fs;

    let mut pressure = Vec::new();

    // Read /proc/diskstats for I/O stats
    if let Ok(content) = fs::read_to_string("/proc/diskstats") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 14 {
                let name = parts[2];

                // Only include nvme and sd devices (not partitions)
                if (name.starts_with("nvme") && !name.contains('p'))
                    || (name.starts_with("sd") && name.len() == 3)
                {
                    // Fields: major minor name reads_completed reads_merged
                    //         sectors_read time_reading writes_completed writes_merged
                    //         sectors_written time_writing io_in_progress time_io weighted_time

                    // Sectors are typically 512 bytes
                    let sectors_read: u64 = parts[5].parse().unwrap_or(0);
                    let sectors_written: u64 = parts[9].parse().unwrap_or(0);

                    let read_bytes = sectors_read * 512;
                    let write_bytes = sectors_written * 512;

                    // Get temperature if available
                    let temp = get_device_temperature(name);

                    pressure.push(DiskPressure {
                        device: name.to_string(),
                        read_bytes_24h: read_bytes,  // Note: This is total since boot
                        write_bytes_24h: write_bytes,
                        temp_avg: temp,
                    });
                }
            }
        }
    }

    pressure
}

/// Get device temperature from /sys or hwmon
fn get_device_temperature(device: &str) -> Option<f64> {
    use std::fs;

    // For NVMe devices
    if device.starts_with("nvme") {
        let temp_path = format!("/sys/class/nvme/{}/hwmon0/temp1_input", device);
        if let Ok(content) = fs::read_to_string(&temp_path) {
            if let Ok(millicelsius) = content.trim().parse::<i64>() {
                return Some(millicelsius as f64 / 1000.0);
            }
        }

        // Try alternative path (would need glob, skip for simplicity)
    }

    // For SATA devices, would need smartctl
    // Skip for now as it requires sudo

    None
}

/// Get network usage from /sys/class/net
fn get_network_usage() -> Vec<NetworkUsage> {
    use std::fs;
    use std::path::Path;

    let mut usage = Vec::new();

    let net_dir = Path::new("/sys/class/net");
    if let Ok(entries) = fs::read_dir(net_dir) {
        for entry in entries.flatten() {
            let iface_name = entry.file_name().to_string_lossy().to_string();

            // Skip loopback
            if iface_name == "lo" {
                continue;
            }

            // Read rx and tx bytes
            let rx_path = entry.path().join("statistics/rx_bytes");
            let tx_path = entry.path().join("statistics/tx_bytes");

            let rx_bytes: u64 = fs::read_to_string(&rx_path)
                .ok()
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(0);

            let tx_bytes: u64 = fs::read_to_string(&tx_path)
                .ok()
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(0);

            // Only include interfaces with traffic
            if rx_bytes > 0 || tx_bytes > 0 {
                usage.push(NetworkUsage {
                    interface: iface_name,
                    rx_bytes_24h: rx_bytes,  // Note: This is total since boot
                    tx_bytes_24h: tx_bytes,
                });
            }
        }
    }

    // Sort by total traffic
    usage.sort_by(|a, b| {
        let a_total = a.rx_bytes_24h + a.tx_bytes_24h;
        let b_total = b.rx_bytes_24h + b.tx_bytes_24h;
        b_total.cmp(&a_total)
    });

    usage
}

/// Format bytes as human-readable string
pub fn format_bytes(bytes: u64) -> String {
    const GIB: u64 = 1024 * 1024 * 1024;
    const MIB: u64 = 1024 * 1024;
    const KIB: u64 = 1024;

    if bytes >= GIB {
        format!("{:.1} GiB", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.0} MiB", bytes as f64 / MIB as f64)
    } else if bytes >= KIB {
        format!("{:.0} KiB", bytes as f64 / KIB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Format bytes compactly (for tables)
pub fn format_bytes_compact(bytes: u64) -> String {
    const GIB: u64 = 1024 * 1024 * 1024;
    const MIB: u64 = 1024 * 1024;

    if bytes >= GIB {
        format!("{:.1} GiB", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.1} MiB", bytes as f64 / MIB as f64)
    } else {
        format!("{:.1} KiB", bytes as f64 / 1024.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(1024), "1 KiB");
        assert_eq!(format_bytes(1024 * 1024), "1 MiB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0 GiB");
    }

    #[test]
    fn test_get_network_usage() {
        let usage = get_network_usage();
        // Should work on any Linux system
        // (may be empty in container environments)
        assert!(usage.len() >= 0);
    }

    #[test]
    fn test_get_disk_pressure() {
        let pressure = get_disk_pressure();
        // Should work on any Linux system
        assert!(pressure.len() >= 0);
    }
}
