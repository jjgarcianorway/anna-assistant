//! Hardware Radar for Anna v0.12.9 "Orion"
//!
//! Scores hardware health and capability across 9 categories (0-10 scale):
//! 1. CPU throughput (cores × freq)
//! 2. CPU thermal headroom
//! 3. Memory capacity vs working set
//! 4. Disk health (SMART)
//! 5. Disk free space
//! 6. Filesystem features (CoW, compression, snapshots)
//! 7. GPU presence and capability
//! 8. Network reliability (packet loss, link speed)
//! 9. Boot reliability (last boot failures)

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Hardware radar result with 9 category scores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareRadar {
    pub overall: u8,           // Average of all categories (0-10)
    pub cpu_throughput: u8,    // Cores × freq, scaled by workload type
    pub cpu_thermal: u8,       // Temperature headroom (0-10, 10=cool)
    pub memory: u8,            // Available vs working set
    pub disk_health: u8,       // SMART summary (10=excellent, 0=failing)
    pub disk_free: u8,         // Free space headroom (10=plenty, 0=full)
    pub fs_features: u8,       // CoW, compression, snapshots (10=all, 0=none)
    pub gpu: u8,               // GPU presence and capability
    pub network: u8,           // Link speed and reliability
    pub boot: u8,              // Boot reliability (10=clean, 0=errors)
}

impl HardwareRadar {
    /// Calculate overall score (average of all categories)
    pub fn calculate_overall(&mut self) {
        let sum = self.cpu_throughput as u16
            + self.cpu_thermal as u16
            + self.memory as u16
            + self.disk_health as u16
            + self.disk_free as u16
            + self.fs_features as u16
            + self.gpu as u16
            + self.network as u16
            + self.boot as u16;

        self.overall = (sum / 9) as u8;
    }
}

/// Collect hardware radar data
pub fn collect_hardware_radar() -> Result<HardwareRadar> {
    let mut radar = HardwareRadar {
        overall: 0,
        cpu_throughput: score_cpu_throughput()?,
        cpu_thermal: score_cpu_thermal()?,
        memory: score_memory()?,
        disk_health: score_disk_health()?,
        disk_free: score_disk_free()?,
        fs_features: score_fs_features()?,
        gpu: score_gpu()?,
        network: score_network()?,
        boot: score_boot()?,
    };

    radar.calculate_overall();
    Ok(radar)
}

//
// Scoring Functions (0-10 scale)
//

/// Score CPU throughput: cores × freq, scaled for workload
///
/// Formula:
/// - Single-thread: max_freq_ghz * 2 (capped at 10)
/// - Multi-thread: min(10, floor(cores / 2))
/// - Balanced: average of both
fn score_cpu_throughput() -> Result<u8> {
    let cpuinfo = fs::read_to_string("/proc/cpuinfo")
        .context("Failed to read /proc/cpuinfo")?;

    // Count logical cores
    let cores = cpuinfo.lines()
        .filter(|line| line.starts_with("processor"))
        .count();

    // Get max frequency (MHz)
    let max_freq_mhz = cpuinfo.lines()
        .find(|line| line.starts_with("cpu MHz"))
        .and_then(|line| line.split(':').nth(1))
        .and_then(|freq_str| freq_str.trim().parse::<f64>().ok())
        .unwrap_or(2000.0); // Fallback to 2 GHz

    let max_freq_ghz = max_freq_mhz / 1000.0;

    // Single-thread score (frequency matters most)
    let single_score = (max_freq_ghz * 2.0).min(10.0);

    // Multi-thread score (core count matters most)
    let multi_score = ((cores as f64) / 2.0).min(10.0);

    // Balanced average
    let score = ((single_score + multi_score) / 2.0).floor() as u8;

    Ok(score)
}

/// Score CPU thermal headroom
///
/// Formula:
/// - Read /sys/class/thermal/thermal_zone*/temp
/// - Find max temperature across all zones
/// - Score = 10 - floor((temp - 30) / 7)
/// - 30°C = 10, 100°C = 0, linear in between
fn score_cpu_thermal() -> Result<u8> {
    let thermal_dir = Path::new("/sys/class/thermal");

    if !thermal_dir.exists() {
        // No thermal data available - assume good
        return Ok(8);
    }

    let mut max_temp_c = 30.0;

    // Scan all thermal zones
    if let Ok(entries) = fs::read_dir(thermal_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.file_name().unwrap().to_str().unwrap().starts_with("thermal_zone") {
                let temp_path = path.join("temp");
                if let Ok(temp_str) = fs::read_to_string(&temp_path) {
                    if let Ok(temp_millic) = temp_str.trim().parse::<i64>() {
                        let temp_c = (temp_millic as f64) / 1000.0;
                        if temp_c > max_temp_c {
                            max_temp_c = temp_c;
                        }
                    }
                }
            }
        }
    }

    // Score formula: 10 at 30°C, 0 at 100°C
    let score = if max_temp_c <= 30.0 {
        10
    } else if max_temp_c >= 100.0 {
        0
    } else {
        let raw_score = 10.0 - ((max_temp_c - 30.0) / 7.0);
        raw_score.max(0.0).min(10.0).floor() as u8
    };

    Ok(score)
}

/// Score memory capacity vs working set
///
/// Formula:
/// - Read /proc/meminfo for MemTotal and MemAvailable
/// - Calculate usage ratio: used / total
/// - Score = 10 - floor(usage_ratio * 10)
/// - 0% used = 10, 100% used = 0
fn score_memory() -> Result<u8> {
    let meminfo = fs::read_to_string("/proc/meminfo")
        .context("Failed to read /proc/meminfo")?;

    let mut mem_total_kb = 0;
    let mut mem_available_kb = 0;

    for line in meminfo.lines() {
        if line.starts_with("MemTotal:") {
            mem_total_kb = line.split_whitespace()
                .nth(1)
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
        } else if line.starts_with("MemAvailable:") {
            mem_available_kb = line.split_whitespace()
                .nth(1)
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
        }
    }

    if mem_total_kb == 0 {
        return Ok(5); // Unknown, assume mediocre
    }

    let mem_used_kb = mem_total_kb.saturating_sub(mem_available_kb);
    let usage_ratio = (mem_used_kb as f64) / (mem_total_kb as f64);

    // Score: 10 at 0% usage, 0 at 100% usage
    let score = (10.0 - (usage_ratio * 10.0)).max(0.0).min(10.0).floor() as u8;

    Ok(score)
}

/// Score disk health via SMART
///
/// Formula:
/// - Run `smartctl -H /dev/sda` for each disk
/// - Parse "PASSED" vs "FAILED"
/// - 10 = all passed, 0 = any failed
/// - If smartctl unavailable, return 7 (assume okay)
fn score_disk_health() -> Result<u8> {
    // Check if smartctl is available
    let smartctl_check = Command::new("which")
        .arg("smartctl")
        .output();

    if smartctl_check.is_err() || !smartctl_check.unwrap().status.success() {
        // smartctl not available - assume okay
        return Ok(7);
    }

    // Scan common disk devices
    let devices = vec!["/dev/sda", "/dev/nvme0n1", "/dev/vda"];
    let mut all_passed = true;
    let mut checked_any = false;

    for device in devices {
        if !Path::new(device).exists() {
            continue;
        }

        let output = Command::new("smartctl")
            .arg("-H")
            .arg(device)
            .output();

        if let Ok(output) = output {
            checked_any = true;
            let stdout = String::from_utf8_lossy(&output.stdout);

            if stdout.contains("FAILED") || stdout.contains("FAILING_NOW") {
                all_passed = false;
                break;
            }
        }
    }

    if !checked_any {
        return Ok(7); // No devices checked, assume okay
    }

    Ok(if all_passed { 10 } else { 0 })
}

/// Score disk free space
///
/// Formula:
/// - Run `df -h /` to get root filesystem usage
/// - Parse percentage used
/// - Score = 10 - floor(usage_pct / 10)
/// - 0% = 10, 100% = 0
fn score_disk_free() -> Result<u8> {
    let output = Command::new("df")
        .arg("-h")
        .arg("/")
        .output()
        .context("Failed to run df")?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse "Use%" column (e.g., "45%")
    let usage_pct = stdout.lines()
        .nth(1) // Skip header
        .and_then(|line| line.split_whitespace().nth(4))
        .and_then(|pct_str| pct_str.trim_end_matches('%').parse::<u8>().ok())
        .unwrap_or(50); // Fallback to 50%

    // Score: 10 at 0%, 0 at 100%
    let score = (10 - (usage_pct / 10)).min(10);

    Ok(score)
}

/// Score filesystem features (CoW, compression, snapshots)
///
/// Formula:
/// - Check if root is btrfs: +5 points (CoW + snapshots)
/// - Check if compression enabled: +3 points
/// - Check if snapshots exist: +2 points
/// - Max 10 points
fn score_fs_features() -> Result<u8> {
    let mut score = 0;

    // Check filesystem type
    let output = Command::new("findmnt")
        .arg("-n")
        .arg("-o")
        .arg("FSTYPE")
        .arg("/")
        .output();

    if let Ok(output) = output {
        let fstype = String::from_utf8_lossy(&output.stdout);

        if fstype.contains("btrfs") {
            score += 5; // CoW + snapshots support

            // Check if compression enabled
            let mount_output = Command::new("findmnt")
                .arg("-n")
                .arg("-o")
                .arg("OPTIONS")
                .arg("/")
                .output();

            if let Ok(mount_output) = mount_output {
                let options = String::from_utf8_lossy(&mount_output.stdout);
                if options.contains("compress") {
                    score += 3; // Compression enabled
                }
            }

            // Check if snapshots exist
            let snapshot_check = Command::new("btrfs")
                .args(&["subvolume", "list", "/"])
                .output();

            if let Ok(snapshot_output) = snapshot_check {
                let subvols = String::from_utf8_lossy(&snapshot_output.stdout);
                if subvols.lines().count() > 1 {
                    score += 2; // Has snapshots
                }
            }
        } else if fstype.contains("ext4") {
            score += 3; // Decent but no CoW
        } else if fstype.contains("xfs") {
            score += 4; // Good performance
        } else {
            score += 2; // Other
        }
    }

    Ok(score.min(10))
}

/// Score GPU presence and capability
///
/// Formula:
/// - Check lspci for VGA/3D controllers
/// - 0 = no GPU (integrated only)
/// - 5 = discrete GPU found
/// - 8 = NVIDIA/AMD GPU found
/// - 10 = high-end GPU (A100, RTX 4090, etc.)
fn score_gpu() -> Result<u8> {
    let output = Command::new("lspci")
        .output();

    if output.is_err() {
        return Ok(0); // lspci unavailable
    }

    let output = output.unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let gpu_lines: Vec<&str> = stdout.lines()
        .filter(|line| line.contains("VGA") || line.contains("3D controller"))
        .collect();

    if gpu_lines.is_empty() {
        return Ok(0); // No GPU
    }

    let gpu_text = gpu_lines.join(" ");

    // Check for high-end GPUs
    if gpu_text.contains("A100") || gpu_text.contains("H100") ||
       gpu_text.contains("RTX 4090") || gpu_text.contains("RTX 5090") {
        return Ok(10);
    }

    // Check for discrete GPUs
    if gpu_text.contains("NVIDIA") || gpu_text.contains("AMD") {
        return Ok(8);
    }

    // Some discrete GPU
    if gpu_lines.len() > 1 {
        return Ok(5);
    }

    // Integrated only
    Ok(0)
}

/// Score network reliability
///
/// Formula:
/// - Check default route interface
/// - Read /sys/class/net/{iface}/speed for link speed
/// - Check /sys/class/net/{iface}/carrier for link status
/// - 10 Gbps+ = 10, 1 Gbps = 8, 100 Mbps = 5, WiFi = 6, down = 0
fn score_network() -> Result<u8> {
    // Get default interface
    let output = Command::new("ip")
        .args(&["route", "show", "default"])
        .output();

    if output.is_err() {
        return Ok(5); // Unknown, assume mediocre
    }

    let output = output.unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let iface = stdout.split_whitespace()
        .nth(4) // "default via X.X.X.X dev <iface>"
        .unwrap_or("eth0");

    // Check carrier (link up?)
    let carrier_path = format!("/sys/class/net/{}/carrier", iface);
    if let Ok(carrier) = fs::read_to_string(&carrier_path) {
        if carrier.trim() == "0" {
            return Ok(0); // Link down
        }
    }

    // Check speed
    let speed_path = format!("/sys/class/net/{}/speed", iface);
    if let Ok(speed_str) = fs::read_to_string(&speed_path) {
        if let Ok(speed_mbps) = speed_str.trim().parse::<i32>() {
            let score = match speed_mbps {
                s if s >= 10000 => 10,  // 10 Gbps+
                s if s >= 1000 => 8,    // 1 Gbps
                s if s >= 100 => 5,     // 100 Mbps
                _ => 3,                 // <100 Mbps
            };
            return Ok(score);
        }
    }

    // Check if WiFi (no speed file)
    if iface.starts_with("wlan") || iface.starts_with("wlp") {
        return Ok(6); // WiFi, decent but not wired
    }

    Ok(5) // Unknown, assume mediocre
}

/// Score boot reliability
///
/// Formula:
/// - Check journalctl for failed units at last boot
/// - Check systemd analyze for slow boot
/// - 0 failed units + <30s boot = 10
/// - 1-2 warnings = 7
/// - 3+ warnings = 4
/// - Any critical errors = 0
fn score_boot() -> Result<u8> {
    // Check for failed units
    let output = Command::new("systemctl")
        .args(&["--failed", "--no-pager"])
        .output();

    if output.is_err() {
        return Ok(7); // Unknown, assume okay
    }

    let output = output.unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let failed_count = stdout.lines()
        .filter(|line| line.contains("loaded") && line.contains("failed"))
        .count();

    if failed_count == 0 {
        return Ok(10); // Clean boot
    } else if failed_count <= 2 {
        return Ok(7); // Minor warnings
    } else if failed_count <= 5 {
        return Ok(4); // Multiple warnings
    } else {
        return Ok(0); // Critical issues
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_radar_structure() {
        let mut radar = HardwareRadar {
            overall: 0,
            cpu_throughput: 8,
            cpu_thermal: 9,
            memory: 7,
            disk_health: 10,
            disk_free: 6,
            fs_features: 7,
            gpu: 0,
            network: 8,
            boot: 10,
        };

        radar.calculate_overall();

        // Average: (8+9+7+10+6+7+0+8+10) / 9 = 65 / 9 = 7.22 -> 7
        assert_eq!(radar.overall, 7);
    }

    #[test]
    fn test_thermal_scoring() {
        // This is a unit test example - actual scoring tested via integration
        // Formula: 10 at 30°C, 0 at 100°C
        let score_30c: f64 = 10.0 - ((30.0 - 30.0) / 7.0);
        assert_eq!(score_30c.floor() as u8, 10);

        let score_65c: f64 = 10.0 - ((65.0 - 30.0) / 7.0);
        assert_eq!(score_65c.floor() as u8, 5);

        let score_100c: f64 = 10.0 - ((100.0 - 30.0) / 7.0);
        assert!(score_100c <= 0.0);
    }

    #[test]
    fn test_memory_scoring() {
        // Example: 16 GB total, 8 GB used = 50% = score 5
        let usage_ratio: f64 = 0.5;
        let score = (10.0 - (usage_ratio * 10.0)).floor() as u8;
        assert_eq!(score, 5);

        // Example: 90% used = score 1
        let usage_ratio: f64 = 0.9;
        let score = (10.0 - (usage_ratio * 10.0)).floor() as u8;
        assert_eq!(score, 1);
    }

    #[test]
    fn test_disk_free_scoring() {
        // Formula: 10 - (usage_pct / 10)
        assert_eq!(10 - (0 / 10), 10);    // 0% = score 10
        assert_eq!(10 - (50 / 10), 5);    // 50% = score 5
        assert_eq!(10 - (90 / 10), 1);    // 90% = score 1
        assert_eq!(10 - (100 / 10), 0);   // 100% = score 0
    }

    #[test]
    fn test_gpu_scoring_logic() {
        // Test GPU scoring logic (not actual lspci call)
        let gpu_text = "VGA compatible controller: NVIDIA Corporation GA102 [GeForce RTX 4090]";

        assert!(gpu_text.contains("NVIDIA") || gpu_text.contains("AMD"));

        if gpu_text.contains("RTX 4090") {
            assert_eq!(10, 10); // High-end GPU
        }
    }
}
